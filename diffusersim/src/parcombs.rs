use std::str;

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub err: String
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

    pub fn at_eof(&self) -> bool { self.eof }
}

pub enum Decision {
    Continue,
    End
}

pub fn go<F>(st: &mut ParseState, mut action: F)
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
                    return parse_error_string(st, format!("UTF-8 decode error at byte {}", st.i));
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

pub fn peek(st: &mut ParseState) -> ParseResult<Option<char>> {
    let mut c: Option<char> = None;
    go(st, |c2| {
        c = Some(c2);
        Decision::End
    })?;
    Ok(c)
}

pub fn skip_nchars(st: &mut ParseState, mut n: usize) ->
ParseResult<()> {
    assert!(n >= 1);
    n += 1;
    go(st, |_| {
        n -= 1;
        if n > 0 { Decision::Continue } else { Decision::End }
    })
}

pub fn take_while<F>(st: &mut ParseState, mut filter: F) ->
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

pub fn sep_by<R1,R2,F1,F2>(st: &mut ParseState, mut sep: F1, mut parser: F2) -> ParseResult<(Vec<R2>, ParseError)>
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