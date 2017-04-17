use geom as g;
use trace as t;
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
    pub materials: Vec<t::MaterialProperties>,
    pub beams: Vec<Beam>,
    pub left_material_properties: Vec<u8>,
    pub right_material_properties: Vec<u8>,
    pub segment_names: HashMap<usize, String>
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
    Segment {
        name: Option<String>,
        left_material: String,
        right_material: String,
        segment: g::Segment
    },
    Material(String, t::MaterialProperties),
    Beam(Beam)
}

// The 'next_code_point' function isn't in stable yet. It should be possible
// to implement this function a little more simply and efficiently once it
// is available.
fn get_next_utf8_codepoint_as_char(arr: &[u8]) -> Option<(char,usize)> {
    // As non-ASCII chars will be rare in practice, try decoding
    // just one byte first, then two, then three, etc.
    for i in 1..5 { // Max length of UTF-8 codepoint is 4 bytes.
        let r = str::from_utf8(&arr[0..i]);
        if let Ok(s) = r {
            if let Some(c) = s.chars().next()
                { return Some((c, i)); }
        }
    }
    
    None
}

pub struct ParseState<'a> {
    input: &'a [u8],
    i: usize,
    line: usize,
    col: usize,
    eof: bool
}

impl<'a> ParseState<'a> {
    pub fn new(input: &'a [u8]) -> ParseState {
        ParseState {
            input: input,
            i: 0,
            line: 1,
            col: 0,
            eof: false
        }
    }

    pub fn save_position(&self) -> (usize, bool) {
        (self.i, self.eof)
    }
}

enum Decision {
    Continue,
    End
}

fn go<F>(st: &mut ParseState, mut action: F)
-> ParseResult<()>
where F: FnMut (char) -> Decision {
    loop {
        let sl = &st.input[st.i ..];
        if sl.len() == 0 {
            st.eof = true;
            return Ok(());
        }
        else {
            match get_next_utf8_codepoint_as_char(sl) {
                None => {
                    return parse_error(st, "UTF-8 decode error");
                },
                Some((c,n)) => {
                    if let Decision::End = action(c)
                        { return Ok(()); }
                    
                    st.i += n;
                    if c == '\n' {
                        st.line += 1;
                        st.col = 0;
                    }
                    else {
                        st.col += 1;
                    }
                }
            }
        }
    }
}

fn peek(st: &mut ParseState) -> ParseResult<Option<char>> {
    let mut c: Option<char> = None;
    go(st, |c2| {
        c = Some(c2);
        Decision::End
    })?;
    Ok(c)
}

fn skip_nchars(st: &mut ParseState, mut n: usize) ->
ParseResult<()> {
    assert!(n >= 1);
    n += 1;
    go(st, |_| {
        n -= 1;
        if n > 0 { Decision::Continue } else { Decision::End }
    })
}

fn take_while<F>(st: &mut ParseState, mut filter: F) ->
ParseResult<Vec<char>>
where F: FnMut(char) -> bool {
    let mut r: Vec<char> = Vec::new();

    go(st, |c| {
        if filter(c) {
            r.push(c);
            Decision::Continue
        }
        else {
            Decision::End
        }
    })?;

    Ok(r)
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
                Decision::End
            },
            Some(cc) => {
                if c != cc {
                    error = true;
                    Decision::End
                }
                else {
                    Decision::Continue
                }
            }
        }
    })?;

    match it.next() {
        None => {
            if error {
                parse_error_string(st, ("Expected: ".to_string() + expected))
            }
            else {
                Ok(())
            }
        },
        Some(_) => {
            parse_error_string(st, ("Expected: ".to_string() + expected))
        }
    }
}

fn skip_space_wc(st: &mut ParseState, include_nl: bool) ->
ParseResult<(Option<char>, usize)> {
    let mut cc: Option<char> = None;
    let mut in_comment = false;
    let mut count = 0;

    go(st, |c| {
        cc = Some(c);

        if c == '\n' {
            in_comment = false;
            if include_nl { Decision::Continue } else { Decision::End }
        }
        else if char::is_whitespace(c) {
            count += 1;
            Decision::Continue
        }
        else if c == '#' {
            count += 1;
            in_comment = true;
            Decision::Continue
        }
        else if in_comment {
            count += 1;
            Decision::Continue
        }
        else {
            Decision::End
        }
    })?;

    Ok((cc, count))
}

fn skip_space(st: &mut ParseState) -> ParseResult<Option<char>> {
    Ok(skip_space_wc(st, false)?.0)
}

fn skip_at_least_one_space(st: &mut ParseState) -> ParseResult<Option<char>> {
    let (r, c) = skip_space_wc(st, false)?;
    if c > 0
        { Ok(r) }
    else
        { parse_error(st, "Expected whitespace") }
}

fn skip_space_inc_nl(st: &mut ParseState) -> ParseResult<Option<char>> {
    Ok(skip_space_wc(st, true)?.0)
}

fn identifier(st: &mut ParseState) -> ParseResult<String> {
    let mut current_str: Vec<char> = Vec::new();
    go(st, |c| {
        if char::is_alphanumeric(c) || c == '_' {
            current_str.push(c);
            Decision::Continue
        }
        else {
            Decision::End
        }
    })?;

    if current_str.len() == 0 {
        parse_error(st, "Expected identifier")
    }
    else {
        Ok(current_str.into_iter().collect())
    }
}

fn sep_by<R1,R2,F1,F2>(st: &mut ParseState, mut sep: F1, mut parser: F2) -> ParseResult<(Vec<R2>, ParseError)>
where F1: Parser<R1>,
      F2: Parser<R2> {

    let mut rs: Vec<R2> = Vec::new();
    let mut pos = st.save_position();

    loop {
        match parser(st) {
            Ok(r) => {
                rs.push(r);
                pos = st.save_position();
            }
            Err(e) => {
                if pos != st.save_position()
                    { return Err(e); }
                else
                    { return Ok((rs, e)); }
            }
        }

        if let Err(e) = sep(st) {
            if pos != st.save_position()
                { return Err(e); }
            else
                { return Ok((rs, e)); }
        }

        pos = st.save_position();
    }
}

fn space_separated<R,F>(st: &mut ParseState, parser: F) -> ParseResult<(Vec<R>, ParseError)>
where F: Parser<R> {
    sep_by(st, skip_at_least_one_space, parser)
}

fn numeric_constant(st: &mut ParseState) -> ParseResult<g::Scalar> {
    let mut n = 0;
    let chars = take_while(st, |c| {
        n += 1;
        char::is_digit(c, 10) ||
        c == 'e' || c == '+' || (n == 1 && c == '-') || c == '.'
    })?;

    if chars.len() == 0 {
        parse_error(st, "Expecting numeric constant")
    }
    else {
        let s: String = chars.into_iter().collect();
        match s.as_str().parse::<g::Scalar>() {
            Err(_) => {
                parse_error(st, "Error in numeric constant syntax")
            },
            Ok(v) => {
                Ok(v)
            }
        }
    }
}

fn entry(st: &mut ParseState) -> ParseResult<Vec<Entry>> {
    match identifier(st) {
        Err(e) => { Err(e) },
        Ok(ident) => {
            if let Err(e) = skip_at_least_one_space(st)
                { return Err(e); }

            if ident == "line" {
                line_entry(st)
            }
            else if ident == "arc" {
                arc_entry(st, false)
            }
            else if ident == "circle" {
                arc_entry(st, true)
            }
            else if ident == "material" {
                material_entry(st)
            }
            else if ident == "colbeam" {
                colbeam_entry(st)
            }
            else {
                parse_error(st, "Unrecognized entry type")
            }
        }
    }
}

fn material_pair(st: &mut ParseState) -> ParseResult<(String, String)> {
    let i1 = identifier(st)?;
    skip_space(st)?;
    expect_str(st, "/")?;
    skip_space(st)?;
    let i2 = identifier(st)?;
    Ok((i1, i2))
}

fn make_segment_entry(x1: g::Scalar, y1: g::Scalar, x2: g::Scalar, y2: g::Scalar, name: Option<String>, mat1: String, mat2: String)
-> Entry {
    let newseg = g::seg(x1, y1, x2, y2);
    // If the points were reordered by the 'seg' constructor, then
    // we also want to swap mat1 and mat2.
    if newseg.p1.coords[0] == x1 && newseg.p1.coords[1] == y1 {
        Entry::Segment { name: name, left_material: mat1, right_material: mat2, segment: newseg }
    }
    else {
        Entry::Segment { name: name, left_material: mat2, right_material: mat1, segment: newseg }
    }
}

fn optional_name(st: &mut ParseState) -> ParseResult<Option<String>> {
    match peek(st)? {
        None => { return Ok(None) },
        Some(c) => {
            if c == '\n'
                { return Ok(None); }
            else if c != 'n'
                { return parse_error(st, "Expected end of segment/arc definition or optional name"); }
            expect_str(st, "named")?;
            skip_at_least_one_space(st)?;
            let name = identifier(st)?;
            Ok(Some(name))
        }
    }
}

fn line_entry(st: &mut ParseState) -> ParseResult<Vec<Entry>> {
    let (i1, i2) = material_pair(st)?;
    let mut coords: [g::Scalar; 4] = [0.0; 4];

    for i in 0..4 {
        skip_at_least_one_space(st)?;
        let n = numeric_constant(st)?;
        coords[i] = n;
    }

    skip_space(st)?;
    let name = optional_name(st)?;

    // We expect possible whitespace followed by newline
    // or EOF.
    let term = skip_space(st)?;
    if !st.eof && term.is_some() && term.unwrap() != '\n' {
        return parse_error(st, "Junk at end of 'line' def");
    }

    Ok(vec![make_segment_entry(
        coords[0], coords[1], coords[2], coords[3],
        name,
        i1, i2
    )])
}

fn arc_entry(st: &mut ParseState, is_circle: bool) -> ParseResult<Vec<Entry>> {
    let (i1, i2) = material_pair(st)?;

    skip_space(st)?;

    expect_str(st, "(")?;

    skip_space(st)?;
    let n_segs_f = numeric_constant(st)?;
    if n_segs_f < 3.0 || n_segs_f != n_segs_f.floor()
        { return parse_error(st, "Number of segments must be an integer >= 3 for arc/circle"); }
    let n_segs = n_segs_f as usize;
    skip_space(st)?;

    let mut from = -1.0;
    let mut to = -1.0;

    match peek(st)? {
        None => { return parse_error(st, "Unexpected end of file in middle of arc/circle definition"); },
        Some(c) => {
            if c == ':' {
                skip_nchars(st, 1)?;
                skip_space(st)?;
                from = numeric_constant(st)?;
                if from < 0.0 || from != from.floor()
                    { return parse_error(st, "Beginning of segment range must be integer >= 0"); }
                skip_space(st)?;
                expect_str(st, "-")?;
                skip_space(st)?;
                to = numeric_constant(st)?;
                if to < 1.0 || to != to.floor() || to > n_segs as f64
                    { return parse_error(st, "End of segment range must be integer >= 1.0 and <= number of segments"); }
                skip_space(st)?;
            }
        }
    }

    expect_str(st, ")")?;

    skip_space(st)?;
    let n_coords = if is_circle { 4 } else { 6 };
    let mut coords: [g::Scalar; 6] = [0.0; 6];
    for i in 0..n_coords {
        if i != 0
            { skip_at_least_one_space(st)?; }
        let n = numeric_constant(st)?;
        coords[i] = n;
    }
    if is_circle {
        coords[4] = coords[2];
        coords[5] = coords[3];
    }

    skip_space(st)?;
    let name = optional_name(st)?;

    let segs = g::arc_to_segments(
        g::Point2::new(coords[0], coords[1]),
        g::Point2::new(coords[2], coords[3]),
        g::Point2::new(coords[4], coords[5]),
        n_segs,
        from as i32,
        to as i32
    );

    let mut i: usize = 1;
    let entries: Vec<Entry> = segs.iter().map(|&s| {
        let segname;
        match name {
            None => { segname = None; },
            Some(ref n) => {
                segname = Some(format!("{}_{}", n, i));
            }
        }
        i += 1;
        make_segment_entry(
            s.p1.coords[0], s.p1.coords[1], s.p2.coords[0], s.p2.coords[1],
            segname,
            i1.clone(), i2.clone()
        )
    }).collect();

    Ok(entries)
}

fn assignment(st: &mut ParseState) -> ParseResult<(String,g::Scalar)> {
    match identifier(st) {
        Err(e) => { return Err(e); },
        Ok(ident) => {
            skip_space(st)?;
            if let Err(e) = expect_str(st, "=")
                { return Err(e); }
            skip_space(st)?;

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
    let (assignments, _) = space_separated(st, assignment)?;

    let mut m: HashMap<String, g::Scalar> = HashMap::new();
    for (n, v) in assignments {
        if m.contains_key(&n) {
            return parse_error(st, "Duplicate name in assignments");
        }

        m.insert(n, v);
    }
            
    Ok(m)
}

fn material_properties_from_assignments(st: &mut ParseState, assignments: &Vec<(String, g::Scalar)>)
-> ParseResult<t::MaterialProperties> {
    let mut m = t::MaterialProperties::default();
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
                    if !char::is_digit(cc, 10) {
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

            let (assignments, _) = space_separated(st, assignment)?;
            
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
                return parse_error(st, "Collimated beam must be specified for 'rays' (number of rays), 'l' (wavelength) and 'i' (intensity)");
            }
        }
    }
    
    skip_space(st)?;
    
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
    })?;

    if err
        { return parse_error(st, "Expecting |- or -|"); }

    skip_space(st)?;

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
    skip_space(st)?;
    if let Err(_) = expect_str(st, "\n")
        { return parse_error(st, "Expecting newline separator"); }
    skip_space_inc_nl(st)?;
    Ok(())
}

fn document(st: &mut ParseState) -> ParseResult<ImportedGeometry> {
    skip_space(st)?;
    let (r, e) = sep_by(st, entry_sep, entry)?;

    skip_space_inc_nl(st)?;
    if !st.eof {
        return Err(e);
    }

    entries_to_imported_geometry(st, &r)
}

fn entries_to_imported_geometry(st: &mut ParseState, entries: &Vec<Vec<Entry>>) -> ParseResult<ImportedGeometry> {
    let mut material_lookup: HashMap<&str, u8> = HashMap::new();
    let mut materials: Vec<t::MaterialProperties> = Vec::new();
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
    let mut segment_names: HashMap<usize, String> = HashMap::new();

    let mut seg_i = 0;
    for v in entries {
        for e in v {
            if let Entry::Segment { ref name, left_material: ref ml, right_material: ref mr, segment: ref seg } = *e {
                match material_lookup.get(ml.as_str()) {
                    None => { return parse_error(st, "Unknown material"); },
                    Some (pl) => {
                        match material_lookup.get(mr.as_str()) {
                            None => { return parse_error(st, "Unknown material"); },
                            Some (pr) => {
                                segs.push(seg.clone());
                                lmat.push(*pl);
                                rmat.push(*pr);

                                if let Some(ref n) = *name {
                                    segment_names.insert(seg_i, n.clone());
                                }

                                seg_i += 1;
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
        right_material_properties: rmat,
        segment_names: segment_names
    })
}

pub fn parse_geometry(input: &[u8]) -> ParseResult<ImportedGeometry> {
    let mut st = ParseState::new(input);
    document(&mut st)
}

pub fn parse_geometry_file(filename: &str) -> io::Result<ParseResult<ImportedGeometry>> {
    let file = File::open(filename)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents: Vec<u8> = Vec::new();
    buf_reader.read_to_end(&mut contents)?;

    Ok(parse_geometry(contents.as_slice()))
}
