use geom as g;
use std::str;
use std::collections::HashMap;
use std::iter;

pub struct ImportedGeometry {
    segments: Vec<g::Segment>,
    left_material_properties: Vec<g::MaterialProperties>,
    right_material_properties: Vec<g::MaterialProperties>
}

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub err: String
}

type ParseResult<X> = Result<X, ParseError>;

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

fn parse_error<X>(st: &ParseState, error_msg: &str) -> ParseResult<X> {
    Err(ParseError {
        line: st.line,
        col: st.col,
        err: error_msg.to_string()
    })
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
                return parse_error(st, "Unexpected");
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

fn skip_space_(st: &mut ParseState, allow_newlines: bool) -> Option<char> {
    let mut cc: Option<char> = None;
    go(st, |c| {
        if !char::is_whitespace(c) || (!allow_newlines && c == '\n') {
            cc = Some(c);
            return Decision::End;
        }
        else {
            return Decision::Continue;
        }
    });
    return cc;
}

fn skip_space_include_nl(st: &mut ParseState) -> Option<char> { skip_space_(st, true) }
fn skip_space_no_nl(st: &mut ParseState) -> Option<char> { skip_space_(st, false) }

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

fn space_separated<R,F>(st: &mut ParseState, mut parser: F) -> ParseResult<Vec<R>>
where F: FnMut(&mut ParseState) -> ParseResult<R> {
    let mut rs: Vec<R> = Vec::new();

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

        skip_space_no_nl(st);
    }
}

fn numeric_constant(st: &mut ParseState) -> ParseResult<g::Scalar> {
    let chars = take_while(st, |c| {
        c == '0' || c == '1' || c == '2' || c == '3' || c == '4' ||
        c == '5' || c == '6' || c == '7' || c == '8' || c == '9' ||
        c == 'e' || c == '+' || c == '-'
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

pub fn entry<'a>(st: &'a mut ParseState<'a>) -> ParseResult<Entry> {
    match identifier(st) {
        Err(e) => { return Err(e) },
        Ok(ident) => {
            if ident == "line" {
                return line_entry(st);
            }
            else if ident == "material" {
                return line_entry(st);
            }
            else {
                return parse_error(st, "Unrecognized entry type");
            }
        }
    }
}

fn line_entry(st: &mut ParseState) -> ParseResult<Entry> {
    skip_space_no_nl(st);

    match identifier(st) {
        Err(e) => { return Err(e); },
        Ok(i1) => {
            skip_space_no_nl(st);
            if let Err(e) = expect_str(st, "/")
                { return Err(e); }
            skip_space_no_nl(st);
            
            match identifier(st) {
                Err(e) => { return Err(e); },
                Ok(i2) => {
                    let mut coords: [g::Scalar; 4] = [0.0; 4];

                    for i in 0..4 {
                        skip_space_no_nl(st);
                        
                        match numeric_constant(st) {
                            Err(e) => { return Err(e); },
                            Ok(n) => {
                                coords[i] = n;
                            }
                        }
                    }

                    // We expect possible whitespace followed by newline
                    // or EOF.
                    let term = skip_space_no_nl(st);
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

mod tests {
    use geom_import;

    #[test]
    fn parse_to_segments_test1() {
        let input = "line abcd/ef 5 5 5 5";
        let mut st = geom_import::ParseState::new(input);
        println!("===> {:?}", geom_import::entry(&mut st));

        //println!("===> {:?}", geom_import::run_parser(geom_import::entry, "line oooo/oo 5 5 5 5\nline oooo/oo 5 5 5 5\n"))
        //let input = "lineseg 0.0 1.0 2.0 3.0";
        //println!("STARTING");
        //let r = geom_import::parse_to_segments(input);
        //println!("{:?}", r);
    }
}