use geom as g;
use std::str;
use std::collections::HashMap;
use std::iter;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;

#[derive(Debug, Clone)]
pub enum Beam {
    Collimated {
        from: g::Point2,
        to: g::Point2,
        n_rays: usize,
        shiny_side_is_left: bool,
        wavelength: g::Scalar,
        intensity: g::Scalar
    }
}

#[derive(Debug)]
pub struct ImportedGeometry {
    pub segments: Vec<g::Segment>,
    pub materials: Vec<g::MaterialProperties>,
    pub beams: Vec<Beam>,
    pub left_material_properties: Vec<u8>,
    pub right_material_properties: Vec<u8>,
}

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub err: String
}

type ParseResult<T> = Result<T, ParseError>;
trait Parser<T> : FnMut (&mut ParseState) -> ParseResult<T> { }
impl <T,U> Parser<T> for U where U: FnMut (&mut ParseState) -> ParseResult<T> { }

#[derive(Debug)]
enum Entry {
    Segment(String, String, g::Segment),
    Material(String, g::MaterialProperties),
    Beam(Beam)
}

pub struct ParseState<'a> {
    it: iter::Peekable<str::Chars<'a>>,
    line: usize,
    col: usize,
    eof: bool
}

impl<'a> ParseState<'a> {
    pub fn new(s: &str) -> ParseState {
        ParseState {
            it: s.chars().peekable(),
            line: 1,
            col: 0,
            eof: false
        }
    }
}

enum Decision {
    Continue,
    End
}

fn go<F>(st: &mut ParseState, mut action: F)
where F: FnMut (char) -> Decision {
    loop {
        match st.it.peek() {
            None => {
                st.eof = true;
                return;
            }
            Some(cref) => {
                if let Decision::End = action(*cref) {
                    return;
                }

                if *cref == '\n' {
                    st.line += 1;
                    st.col = 0;
                }
                else {
                    st.col += 1;
                }
            }
        }

        st.it.next();
    }
}

#[allow(unused)]
fn drop_while<F>(st: &mut ParseState, filter: F)
where F: Fn(char) -> bool {
    go(st, |c| {
        if filter(c) {
            return Decision::Continue;
        }
        else {
            return Decision::End;
        }
    });
}

fn take_while<F>(st: &mut ParseState, filter: F) -> Vec<char>
where F: Fn(char) -> bool {
    let mut r: Vec<char> = Vec::new();

    go(st, |c| {
        if filter(c) {
            r.push(c);
            return Decision::Continue;
        }
        else {
            return Decision::End;
        }
    });

    r
}

fn parse_error_string<X>(st: &ParseState, error_msg: String) -> ParseResult<X> {
    Err(ParseError {
        line: st.line,
        col: st.col,
        err: error_msg
    })
}

fn parse_error<X>(st: &ParseState, error_msg: &str) -> ParseResult<X> {
    parse_error_string(st, error_msg.to_string())
}

fn expect_str(st: &mut ParseState, expected: &str) -> ParseResult<()> {
    let mut it = expected.chars();
    let mut error = false;

    go(st, |c| {
        match it.next() {
            None => {
                return Decision::End;
            },
            Some(cc) => {
                if c != cc {
                    error = true;
                    return Decision::End;
                }
                else {
                    return Decision::Continue;
                }
            }
        }
    });

    match it.next() {
        None => {
            if error {
                return parse_error_string(st, ("Expected: ".to_string() + expected));
            }
            else {
                return Ok(());
            }
        },
        Some(_) => {
            return parse_error(st, "Unexpected EOF")
        }
    }
}

fn skip_space_wc(st: &mut ParseState, include_nl: bool) -> (Option<char>, usize) {
    let mut cc: Option<char> = None;
    let mut in_comment = false;
    let mut count = 0;

    go(st, |c| {
        cc = Some(c);

        if c == '\n' {
            in_comment = false;
            return if include_nl { Decision::Continue } else { Decision::End };
        }
        else if char::is_whitespace(c) {
            count += 1;
            return Decision::Continue;
        }
        else if c == '#' {
            count += 1;
            in_comment = true;
            return Decision::Continue;
        }
        else if in_comment {
            count += 1;
            return Decision::Continue;
        }
        else {
            return Decision::End;
        }
    });

    return (cc, count);
}

fn skip_space(st: &mut ParseState) -> Option<char> {
    skip_space_wc(st, false).0
}

fn skip_at_least_one_space(st: &mut ParseState) -> ParseResult<Option<char>> {
    let (r, c) = skip_space_wc(st, false);
    if c > 0 {
        return Ok(r);
    }
    else {
        return parse_error(st, "Expected whitespace");
    }
}

fn skip_space_inc_nl(st: &mut ParseState) -> Option<char> {
    skip_space_wc(st, true).0
}

fn identifier(st: &mut ParseState) -> ParseResult<String> {
    let mut current_str: Vec<char> = Vec::new();
    go(st, |c| {
        if char::is_alphanumeric(c) {
            current_str.push(c);
            return Decision::Continue;
        }
        else {
            return Decision::End;
        }
    });

    if current_str.len() == 0 {
        return parse_error(st, "Expected identifier");
    }
    else {
        return Ok(current_str.into_iter().collect());
    }
}

fn sep_by<R1,R2,F1,F2>(st: &mut ParseState, mut sep: F1, mut parser: F2) -> (Vec<R2>, ParseError)
where F1: Parser<R1>,
      F2: Parser<R2> {
    let mut rs: Vec<R2> = Vec::new();

    loop {
        match parser(st) {
            Ok(r) => {
                rs.push(r);
            }
            Err(e) => {
                if rs.len() == 0 {
                    return (rs, e);
                }
                else {
                    return (rs, e);
                }
            }
        }

        if let Err(e) = sep(st) {
            return (rs, e);
        }
    }
}

fn space_separated<R,F>(st: &mut ParseState, parser: F) -> (Vec<R>, ParseError)
where F: Parser<R> {
    sep_by(st, skip_at_least_one_space, parser)
}

// Rust's standard lib seems a bit underdeveloped w.r.t. character classes.
fn is_digit(c: char) -> bool {
    c == '0' || c == '1' || c == '2' || c == '3' || c == '4' ||
    c == '5' || c == '6' || c == '7' || c == '8' || c == '9'
}

fn numeric_constant(st: &mut ParseState) -> ParseResult<g::Scalar> {
    let chars = take_while(st, |c| {
        is_digit(c) ||
        c == 'e' || c == '+' || c == '-' || c == '.'
    });

    if chars.len() == 0 {
        return parse_error(st, "Expecting numeric constant");
    }
    else {
        let s: String = chars.into_iter().collect();
        match s.as_str().parse::<g::Scalar>() {
            Err(_) => {
                return parse_error(st, "Error in numeric constant syntax");
            },
            Ok(v) => {
                return Ok(v);
            }
        }
    }
}

fn entry(st: &mut ParseState) -> ParseResult<Vec<Entry>> {
    match identifier(st) {
        Err(e) => { return Err(e) },
        Ok(ident) => {
            if let Err(e) = skip_at_least_one_space(st)
                { return Err(e); }

            if ident == "line" {
                return line_entry(st);
            }
            if ident == "arc" {
                return arc_entry(st);
            }
            else if ident == "material" {
                return material_entry(st);
            }
            else if ident == "colbeam" {
                return colbeam_entry(st);
            }
            else {
                return parse_error(st, "Unrecognized entry type");
            }
        }
    }
}

fn material_pair(st: &mut ParseState) -> ParseResult<(String, String)> {
    let i1 = identifier(st)?;
    skip_space(st);
    expect_str(st, "/")?;
    skip_space(st);
    let i2 = identifier(st)?;
    Ok((i1, i2))
}

fn line_entry(st: &mut ParseState) -> ParseResult<Vec<Entry>> {
    let (i1, i2) = material_pair(st)?;
    let mut coords: [g::Scalar; 4] = [0.0; 4];

    for i in 0..4 {
        skip_at_least_one_space(st)?;
        let n = numeric_constant(st)?;
        coords[i] = n;
    }

    // We expect possible whitespace followed by newline
    // or EOF.
    let term = skip_space(st);
    if !st.eof && term.is_some() && term.unwrap() != '\n' {
        return parse_error(st, "Junk at end of 'line' def");
    }

    let newseg = g::seg(coords[0], coords[1], coords[2], coords[3]);
    // If the points were reordered by the 'seg' constructor, then
    // we also want to swap i1 and i2.
    let ii1;
    let ii2;
    if  newseg.p1.coords[0] == coords[0] && newseg.p1.coords[1] == coords[1] {
        ii1 = i1;
        ii2 = i2;
    }
    else {
        ii1 = i2;
        ii2 = i1;
    }

    Ok(vec![Entry::Segment(
        ii1,
        ii2,
        newseg
    )])
}

fn arc_entry(st: &mut ParseState) -> ParseResult<Vec<Entry>> {
    let (i1, i2) = material_pair(st)?;

    skip_space(st);

    expect_str(st, "(")?;
    skip_space(st);
    let n_segs_f = numeric_constant(st)?;
    if (n_segs_f < 1.0 || n_segs_f != n_segs_f.floor())
        { return parse_error(st, "Number of segments must be a positive integer"); }
    let n_segs = n_segs_f as usize;
    skip_space(st);
    expect_str(st, ")")?;

    skip_space(st);

    let mut coords: [g::Scalar; 6] = [0.0; 6];
    for i in 0..6 {
        if i != 0
            { skip_at_least_one_space(st)?; }
        let n = numeric_constant(st)?;
        coords[i] = n;
    }

    let segs = g::arc_to_segments(
        g::Point2::new(coords[0], coords[1]),
        g::Point2::new(coords[2], coords[3]),
        g::Point2::new(coords[4], coords[5]),
        n_segs
    );

    let entries: Vec<Entry> = segs.iter().map(|&s| {
        Entry::Segment(
            i1.clone(), i2.clone(),
            s
        )
    }).collect();

    Ok(entries)
}

fn assignment(st: &mut ParseState) -> ParseResult<(String,g::Scalar)> {
    match identifier(st) {
        Err(e) => { return Err(e); },
        Ok(ident) => {
            skip_space(st);
            if let Err(e) = expect_str(st, "=")
                { return Err(e); }
            skip_space(st);

            match numeric_constant(st) {
                Err(e) => { Err(e) }
                Ok(v) => {
                    Ok((ident, v))
                }
            }
        }
    }
}

fn assignment_hash(st: &mut ParseState) -> ParseResult<HashMap<String, g::Scalar>> {
    let (assignments, e) = space_separated(st, assignment);
    if assignments.len() == 0 {
        return Err(e);
    }

    let mut m: HashMap<String, g::Scalar> = HashMap::new();
    for (n, v) in assignments {
        if m.contains_key(&n) {
            return parse_error(st, "Duplicate name in assignments");
        }

        m.insert(n, v);
    }
            
    Ok(m)
}

fn material_properties_from_assignments(st: &mut ParseState, assignments: &Vec<(String, g::Scalar)>) -> ParseResult<g::MaterialProperties> {
    let mut m = g::MaterialProperties::default();
    let mut coeffs: HashMap<usize, g::Scalar> = HashMap::new();
    let mut max_coeff_n = 0;
    
    for &(ref n, ref v) in assignments {
        if n == "drf" {
            m.diffuse_reflect_fraction = *v;
        }
        else if n == "srf" {
            m.specular_reflect_fraction = *v;
        }
        else if n == "rff" {
            m.refraction_fraction = *v;
        }
        else if n == "at" {
            m.attenuation_coeff = *v;
        }
        else {
            let mut it = n.chars();
            if let Some(c) = it.next() {
                if c != 'c' {
                    return parse_error_string(st, "Unrecognized attribute assigned in material: ".to_string() + n);
                }

                let mut ncs: Vec<char> = Vec::new();
                let mut i = 0;
                for cc in it {
                    if !is_digit(cc) {
                        return parse_error(st, "No digit following 'c' in attribute name");
                    }
                    if i > 3
                        { return parse_error(st, "Coefficient number has too many digits"); }
                    ncs.push(cc);
                    i += 1;
                }

                let ns: String = ncs.into_iter().collect();
                let coeffn = ns.parse::<usize>().unwrap();
                if coeffn == 0
                    { return parse_error(st, "Coefficients numbered from 1"); }

                coeffs.insert(coeffn, *v);
                if coeffn > max_coeff_n
                    { max_coeff_n = coeffn }
            }
            else {
                return parse_error(st, "Weird: empty attribute name?");
            }
        }
    }

    let mut coeffs_vec: Vec<g::Scalar> = iter::repeat(0.0).take(max_coeff_n).collect();
    for i in 1..max_coeff_n+1 {
        if let Some(v) = coeffs.get(&i) {
            coeffs_vec[i-1] = *v;
        }
    }

    m.cauchy_coeffs = coeffs_vec;
    Ok(m)
}

fn material_entry(st: &mut ParseState) -> ParseResult<Vec<Entry>> {
    match identifier(st) {
        Err(e) => { Err(e) },
        Ok(name) => {
            if let Err(e) = skip_at_least_one_space(st)
                { return Err(e); }

            let (assignments, e) = space_separated(st, assignment);
            if assignments.len() == 0
                { return Err(e); }
            
            match material_properties_from_assignments(st, &assignments) {
                Err(e) => { Err(e) },
                Ok(props) => { Ok(vec![Entry::Material(name, props)]) }
            }
        }
    }
}

fn colbeam_entry(st: &mut ParseState) -> ParseResult<Vec<Entry>> {
    let mut n_rays: usize = 0;
    let mut wavelength: g::Scalar = 0.0;
    let mut intensity: g::Scalar = 0.0;

    match assignment_hash(st) {
        Err(e) => { return Err(e); },
        Ok(assignments) => {
            let mut i = 0;
            for (k, v) in assignments {
                if k == "n" {
                    if v < 1.0 || v.floor() != v
                        { return parse_error(st, "Number of rays must be a positive integer"); }

                    i += 1;
                    n_rays = v as usize;
                }
                else if k == "l" {
                    i += 1;
                    wavelength = v;
                }
                else if k == "i" {
                    i += 1;
                    intensity = v;
                }
                else {
                    return parse_error(st, "Unrecognized ray property");
                }
            }
            if i < 3 {
                return parse_error(st, "Ray must be specified for 'rays' (number of rays), 'l' (wavelength) and 'i' (intensity)");
            }
        }
    }
    
    skip_space(st);
    
    let mut i = 0;
    let mut first_was_dash = false;
    let mut err = false;
    go(st, |c| {
        if i > 1
            { return Decision::End; }

        if c == '-' {
            if i == 0
                { first_was_dash = true; }
        }
        else if c != '|' {
            err = true;
            return Decision::End;
        }

        i += 1;
        return Decision::Continue;
    });

    if err
        { return parse_error(st, "Expecting |- or -|"); }

    skip_space(st);

    let mut coords: [g::Scalar; 4] = [0.0; 4];
    for i in 0..4 {
        if i != 0 {
            if let Err(e) = skip_at_least_one_space(st)
                { return Err(e); }
        }

        match numeric_constant(st) {
            Err(e) => { return Err(e); },
            Ok(n) => {
                coords[i] = n;
            }
        }
    }

    let beam = Beam::Collimated {
        from: g::Point2::new(coords[0], coords[1]),
        to:   g::Point2::new(coords[2], coords[3]),
        n_rays: n_rays,
        shiny_side_is_left: first_was_dash,
        wavelength: wavelength,
        intensity: intensity
    };

    Ok(vec![Entry::Beam(beam)])
}

fn entry_sep(st: &mut ParseState) -> ParseResult<()> {
    skip_space(st);
    if let Err(_) = expect_str(st, "\n")
        { return parse_error(st, "Expecting newline separator"); }
    skip_space_inc_nl(st);
    Ok(())
}

fn document(st: &mut ParseState) -> ParseResult<ImportedGeometry> {
    skip_space(st);
    let (r, e) = sep_by(st, entry_sep, entry);

    skip_space_inc_nl(st);
    if !st.eof {
        return Err(e);
    }

    entries_to_imported_geometry(st, &r)
}

fn entries_to_imported_geometry(st: &mut ParseState, entries: &Vec<Vec<Entry>>) -> ParseResult<ImportedGeometry> {
    let mut material_lookup: HashMap<&str, u8> = HashMap::new();
    let mut materials: Vec<g::MaterialProperties> = Vec::new();
    let mut beams: Vec<Beam> = Vec::new();

    let mut mi = 0;
    for v in entries {
        for e in v {
            if let Entry::Material(ref name, ref props) = *e {
                if materials.len() >= 255 {
                    return parse_error(st, "Cannot have more than 255 materials.");
                }

                materials.push(props.clone());
                material_lookup.insert(name, mi);
                mi += 1;
            }
        }
    }

    let mut segs: Vec<g::Segment> = Vec::new();
    let mut lmat: Vec<u8> = Vec::new();
    let mut rmat: Vec<u8> = Vec::new();

    for v in entries {
        for e in v {
            if let Entry::Segment(ref ml, ref mr, ref seg) = *e {
                match material_lookup.get(ml.as_str()) {
                    None => { return parse_error(st, "Unknown material"); },
                    Some (pl) => {
                        match material_lookup.get(mr.as_str()) {
                            None => { return parse_error(st, "Unknown material"); },
                            Some (pr) => {
                                segs.push(seg.clone());
                                lmat.push(*pl);
                                rmat.push(*pr);
                            }
                        }
                    }
                }
            }
            else if let Entry::Beam(ref beam) = *e {
                beams.push(beam.clone());
            }
        }
    }

    Ok(ImportedGeometry {
        segments: segs,
        materials: materials,
        beams: beams,
        left_material_properties: lmat,
        right_material_properties: rmat
    })
}

pub fn parse_geometry_str(input: &str) -> ParseResult<ImportedGeometry> {
    let mut st = ParseState::new(input);
    document(&mut st)
}

pub fn parse_geometry_file(filename: &str) -> io::Result<ParseResult<ImportedGeometry>> {
    let file = File::open(filename)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    Ok(parse_geometry_str(contents.as_str()))
}
