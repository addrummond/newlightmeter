use std::str;
use std::error;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub err: String
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Line {} col {}: {}", self.line, self.col, self.err)
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        "Parse error"
    }
    fn cause(&self) -> Option<&error::Error> {
        Some(self)
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
pub trait Parser<T>: FnMut (&mut ParseState) -> ParseResult<T> { }
impl <T,U> Parser<T> for U where U: FnMut (&mut ParseState) -> ParseResult<T> { }

// The 'next_code_point' function isn't in stable yet. It should be possible
// to implement the function below a little more simply and efficiently once it
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
    peek: usize,
    i_at_last_peek: isize
}

impl<'a> ParseState<'a> {
    pub fn new(input: &'a [u8]) -> ParseState {
        ParseState {
            input: input,
            i: 0,
            line: 1,
            col: 0,
            peek: 0,
            i_at_last_peek: -1
        }
    }
}

pub fn save_position(st: &mut ParseState) -> usize {
    st.i
}

pub fn at_eof(st: &mut ParseState) -> bool {
    st.i >= st.input.len()
}

pub fn peek_char(st: &mut ParseState) -> ParseResult<Option<char>> {
    if st.i >= st.input.len()
        { return Ok(None); }
    
    let sl = &st.input[st.i ..];
    match get_next_utf8_codepoint_as_char(sl) {
        None => { parse_error_string(st, format!("UTF-8 decode error at byte {}", st.i)) },
        Some((c, n)) => {
            st.peek = n;
            st.i_at_last_peek = st.i as isize;
            Ok(Some(c))
        }
    }
}

pub fn update_line_col(st: &mut ParseState, c: char) {
    if c == '\n' {
        st.line += 1;
        st.col = 0;
    }
    else {
        st.col += 1;
    }
}

pub fn skip_peeked(st: &mut ParseState, c: char) {
    // If this isn't true, then more was read since the last peek
    // and we can't skip the last peek.
    assert!((st.i_at_last_peek as usize) == st.i);

    st.i += st.peek;
    update_line_col(st, c);
}

pub fn next_char(st: &mut ParseState) -> ParseResult<Option<char>> {
    if st.i >= st.input.len()
        { return Ok(None); }
    
    let sl = &st.input[st.i ..];
    match get_next_utf8_codepoint_as_char(sl) {
        None => {
            parse_error_string(st, format!("UTF-8 decode error at byte {}", st.i))
        },
        Some((c, n)) => {
            st.i += n;
            update_line_col(st, c);
            Ok(Some(c))
        }
    }
}

pub fn skip_nchars(st: &mut ParseState, mut n: usize) ->
ParseResult<()> {
    assert!(n >= 1);

    n += 1;
    while n > 0 {
        if let None = next_char(st)?
            { return parse_error(st, "Unexpected EOF"); }
        n -= 1;
    }

    Ok(())
}

pub fn take_while<F>(st: &mut ParseState, mut filter: F) ->
ParseResult<Vec<char>>
where F: FnMut(char) -> bool {
    let mut r: Vec<char> = Vec::new();

    while let Some(c) = peek_char(st)? {
        if filter(c) {
            skip_peeked(st, c);
            r.push(c);
        }
        else {
            break;
        }
    }

    Ok(r)
}

pub fn parse_error_string<X>(st: &ParseState, error_msg: String) -> ParseResult<X> {
    Err(ParseError {
        line: st.line,
        col: st.col,
        err: error_msg
    })
}

pub fn parse_error<X>(st: &ParseState, error_msg: &str) -> ParseResult<X> {
    parse_error_string(st, error_msg.to_string())
}

pub fn expect_str(st: &mut ParseState, expected: &str) -> ParseResult<()> {
    let mut it = expected.chars();
    let mut error = false;

    while let Some(c) = peek_char(st)? {
        match it.next() {
            None => { break; },
            Some(cc) => {
                if c != cc {
                    error = true;
                    break;
                }
                skip_peeked(st, c);
            }
        }
    }

    match it.next() {
        None => {
            if error {
                parse_error_string(st, format!("Expected '{}'", expected))
            }
            else {
                Ok(())
            }
        },
        Some(_) => {
            parse_error_string(st, format!("Expected '{}'", expected))
        }
    }
}

pub fn skip_space_wc(st: &mut ParseState, include_nl: bool) ->
ParseResult<(Option<char>, usize)> {
    let mut cc: Option<char> = None;
    let mut in_comment = false;
    let mut count = 0;

    while let Some(c) = peek_char(st)? {
        cc = Some(c);
        count += 1;

        if c == '\n' {
            in_comment = false;
            if !include_nl
                { return Ok((cc, count)); }
            skip_peeked(st, c);
        }
        else if char::is_whitespace(c) {
            skip_peeked(st, c);
        }
        else if c == '#' {
            in_comment = true;
            skip_peeked(st, c);
        }
        else if in_comment {
            skip_peeked(st, c);
        }
        else {
            count -= 1;
            return Ok((cc, count));
        }
    }

    Ok((cc, count))
}

pub fn skip_space(st: &mut ParseState) -> ParseResult<Option<char>> {
    Ok(skip_space_wc(st, false)?.0)
}

pub fn skip_at_least_one_space(st: &mut ParseState) -> ParseResult<Option<char>> {
    let (r, c) = skip_space_wc(st, false)?;
    if c > 0
        { Ok(r) }
    else
        { parse_error(st, "Expected whitespace, found '{}'") }
}

pub fn skip_space_inc_nl(st: &mut ParseState) -> ParseResult<Option<char>> {
    Ok(skip_space_wc(st, true)?.0)
}

pub fn identifier(st: &mut ParseState) -> ParseResult<String> {
    let mut current_str: Vec<char> = Vec::new();
    while let Some(c) = peek_char(st)? {
        if char::is_alphanumeric(c) || c == '_' {
            current_str.push(c);
            skip_peeked(st, c);
        }
        else {
            break;
        }
    }

    if current_str.len() == 0 {
        parse_error(st, "Expected identifier")
    }
    else {
        Ok(current_str.into_iter().collect())
    }
}

pub fn sep_by<R1,R2,F1,F2>(st: &mut ParseState, mut sep: F1, mut parser: F2) -> ParseResult<(Vec<R2>, ParseError)>
where F1: Parser<R1>,
      F2: Parser<R2> {

    let mut rs: Vec<R2> = Vec::new();
    let mut pos = save_position(st);

    loop {
        match parser(st) {
            Ok(r) => {
                rs.push(r);
                pos = save_position(st);
            }
            Err(e) => {
                if pos != save_position(st)
                    { return Err(e); }
                else
                    { return Ok((rs, e)); }
            }
        }

        if let Err(e) = sep(st) {
            if pos != save_position(st)
                { return Err(e); }
            else
                { return Ok((rs, e)); }
        }

        pos = save_position(st);
    }
}

pub fn space_separated<R,F>(st: &mut ParseState, parser: F) -> ParseResult<(Vec<R>, ParseError)>
where F: Parser<R> {
    sep_by(st, skip_at_least_one_space, parser)
}

pub fn numeric_constant(st: &mut ParseState) -> ParseResult<f64> {
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
        match s.as_str().parse::<f64>() {
            Err(_) => {
                parse_error_string(st, format!("Error in syntax of numeric constant '{}'", s))
            },
            Ok(v) => {
                Ok(v)
            }
        }
    }
}