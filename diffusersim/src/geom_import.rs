use geom as g;
use std::str;
use std::collections::HashMap;
use std::iter;

#[derive(Debug)]
pub struct ImportedGeometry {
    segments: Vec<g::Segment>,
    materials: Vec<g::MaterialProperties>,
    left_material_properties: Vec<usize>,
    right_material_properties: Vec<usize>,
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
enum SegmentType {
    Line,
    Arc
}

#[derive(Debug)]
pub enum Entry {
    Segment(String, String, g::Segment),
    Material(String, g::MaterialProperties)
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

fn skip_space(st: &mut ParseState) -> Option<char> {
    let mut cc: Option<char> = None;
    go(st, |c| {
        if !char::is_whitespace(c) || c == '\n' {
            cc = Some(c);
            return Decision::End;
        }
        else {
            return Decision::Continue;
        }
    });
    return cc;
}

fn skip_at_least_one_space(st: &mut ParseState) -> ParseResult<Option<char>> {
    let mut cc: Option<char> = None;
    let mut gotone = false;
    go(st, |c| {
        if !char::is_whitespace(c) || c == '\n' {
            cc = Some(c);
            return Decision::End;
        }
        else {
            gotone = true;
            return Decision::Continue;
        }
    });

    if gotone {
        return Ok(cc);
    }
    else {
        return parse_error(st, "Expected whitespace");
    }
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

fn sep_by<R1,R2,F1,F2>(st: &mut ParseState, mut sep: F1, mut parser: F2) -> ParseResult<Vec<R2>>
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
                    return Err(e);
                }
                else {
                    return Ok(rs);
                }
            }
        }

        if let Err(e) = sep(st) {
            if rs.len() == 0 {
                return Err(e);
            }
            else {
                return Ok(rs);
            }
        }
    }
}

fn space_separated<R,F>(st: &mut ParseState, mut parser: F) -> ParseResult<Vec<R>>
where F: Parser<R> {
    sep_by(st, skip_at_least_one_space, parser)
}

fn numeric_constant(st: &mut ParseState) -> ParseResult<g::Scalar> {
    let chars = take_while(st, |c| {
        c == '0' || c == '1' || c == '2' || c == '3' || c == '4' ||
        c == '5' || c == '6' || c == '7' || c == '8' || c == '9' ||
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

pub fn entry(st: &mut ParseState) -> ParseResult<Entry> {
    match identifier(st) {
        Err(e) => { return Err(e) },
        Ok(ident) => {
            if let Err(e) = skip_at_least_one_space(st)
                { return Err(e); }

            if ident == "line" {
                return line_entry(st);
            }
            else if ident == "material" {
                return material_entry(st);
            }
            else {
                return parse_error(st, "Unrecognized entry type");
            }
        }
    }
}

fn line_entry(st: &mut ParseState) -> ParseResult<Entry> {
    match identifier(st) {
        Err(e) => { return Err(e); },
        Ok(i1) => {
            skip_space(st);
            if let Err(e) = expect_str(st, "/")
                { return Err(e); }
            skip_space(st);
            
            match identifier(st) {
                Err(e) => { return Err(e); },
                Ok(i2) => {
                    let mut coords: [g::Scalar; 4] = [0.0; 4];

                    for i in 0..4 {
                        if let Err(e) = skip_at_least_one_space(st)
                            { return Err(e); }
                        
                        match numeric_constant(st) {
                            Err(e) => { return Err(e); },
                            Ok(n) => {
                                coords[i] = n;
                            }
                        }
                    }

                    // We expect possible whitespace followed by newline
                    // or EOF.
                    let term = skip_space(st);
                    if !st.eof && term.is_some() && term.unwrap() != '\n' {
                        return parse_error(st, "Junk at end of 'line' def");
                    }

                    Ok(Entry::Segment(
                        i1,
                        i2,
                        g::Segment { 
                            p1: g::Point2::new(coords[0], coords[1]),
                            p2: g::Point2::new(coords[1], coords[2])
                        }
                    ))
                }
            }
        }
    }
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

fn material_properties_from_assignments(assignments: &Vec<(String, g::Scalar)>) -> g::MaterialProperties {
    let mut m = g::make_dummy_material_properties();

    for &(ref n, ref v) in assignments {
        if n == "ri" {
            m.refractive_index = *v;
        }
        else if n == "ex" {
            m.extinction = *v;
        }
    }

    m
}

fn material_entry(st: &mut ParseState) -> ParseResult<Entry> {
    match identifier(st) {
        Err(e) => { Err(e) },
        Ok(name) => {
            if let Err(e) = skip_at_least_one_space(st)
                { return Err(e); }

            match space_separated(st, assignment) {
                Err(e) => { Err(e) },
                Ok(assignments) => {
                    Ok(Entry::Material(name, material_properties_from_assignments(&assignments)))
                }
            }
        }
    }
}

fn entry_sep(st: &mut ParseState) -> ParseResult<()> {
    skip_space(st);
    if let Err(e) = expect_str(st, "\n")
        { return parse_error(st, "Expecting newline separator"); }
    drop_while(st, |c| char::is_whitespace(c));
    Ok(())
}

pub fn document(st: &mut ParseState) -> ParseResult<ImportedGeometry> {
    skip_space(st);
    match sep_by(st, entry_sep, entry) {
        Err(e) => { Err(e) },
        Ok(r) => {
            take_while(st, |c| { println!("TAKING: {}", c); char::is_whitespace(c) });
            if !st.eof {
                return parse_error(st, "Junk at end of file");
            }

            entries_to_imported_geometry(st, &r)
        }
    }
}

fn entries_to_imported_geometry(st: &mut ParseState, entries: &Vec<Entry>) -> ParseResult<ImportedGeometry> {
    let mut material_lookup: HashMap<&str, usize> = HashMap::new();
    let mut materials: Vec<g::MaterialProperties> = Vec::new();

    let mut mi = 0;
    for e in entries {
        if let Entry::Material(ref name, ref props) = *e {
            materials.push(props.clone());
            material_lookup.insert(name, mi);
            mi += 1;
        }
    }

    let mut segs: Vec<g::Segment> = Vec::new();
    let mut lmat: Vec<usize> = Vec::new();
    let mut rmat: Vec<usize> = Vec::new();

    for e in entries {
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
    }

    Ok(ImportedGeometry {
        segments: segs,
        materials: materials,
        left_material_properties: lmat,
        right_material_properties: rmat
    })
}

mod tests {
    use geom_import;

    #[test]
    fn parse_to_segments_test1() {
        //let input = "line abcd/ef 5 5 5 5";
        let input = "material foo yy=5.3 ex=5 zz=54 zz=5\nline foo/foo 1 2 3 4";
        let mut st = geom_import::ParseState::new(input);
        println!("===> {:?}", geom_import::document(&mut st));

        //println!("===> {:?}", geom_import::run_parser(geom_import::entry, "line oooo/oo 5 5 5 5\nline oooo/oo 5 5 5 5\n"))
        //let input = "lineseg 0.0 1.0 2.0 3.0";
        //println!("STARTING");
        //let r = geom_import::parse_to_segments(input);
        //println!("{:?}", r);
    }
}