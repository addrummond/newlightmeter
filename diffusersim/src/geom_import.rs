use geom as g;
use std::char;
use std;

pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub err: String
}

fn parse_to_segments(input: &str) -> Result<Vec<g::Segment>, ParseError> {
    enum SegmentType {
        Line
    }
    
    enum St {
        Initial,
        GetCoord
    }

    struct State<'a> {
        kind: St,
        i: std::str::Chars<'a>,
        c: char,
        eof: bool,
        line: usize,
        col: usize,
        current_str: Vec<char>,
        current_coords: Vec<g::Scalar>,
        current_segment_type: SegmentType,
        segments: Vec<g::Segment>
    }

    let mut state_ = State {
        kind: St::Initial,
        i: input.chars(),
        c: ' ',
        eof: false,
        line: 1,
        col: 0,
        current_str: Vec::new(),
        current_coords: Vec::new(),
        current_segment_type: SegmentType::Line,
        segments: Vec::new()
    };
    let state = &mut state_;

    fn next(state: &mut State) {
        if let Some(cc) = state.i.next() {
            state.c = cc;
            if cc == '\n' {
                state.line += 1;
                state.col = 0;
            }
            else {
                state.col += 1;
            }
        }
    }

    fn skip_space(state: &mut State) {
        while !state.eof && char::is_whitespace(state.c) {
            next(state);
        }
    }

    fn make_error(state: &State, msg: &str) -> Result<Vec<g::Segment>, ParseError> {
        Err (ParseError {
            line: state.line,
            col: state.col,
            err: msg.to_string()
        })
    }

    skip_space(state);
    loop {
        if (state.eof) {
            match state.kind {
                St::Initial => { return make_error(state, "Unexpected end of input."); }
                St::GetCoord => { return make_error(state, "TODO REPLACE"); }
            }
        }

        match state.kind {
            St::Initial => {
                if char::is_alphanumeric(state.c) {
                    state.current_str.push(state.c);
                    // TODO TODO INSERT CALL TO NEXT
                    next(state);
                }
                else if (char::is_whitespace(state.c)) {
                    let s: String = state.current_str.iter().cloned().collect();
                    if (s == "lineseg") {
                        skip_space(state);
                        state.current_str.clear();
                        state.current_segment_type = SegmentType::Line;
                        state.kind = St::GetCoord;
                    }
                    else {
                        return make_error(state, "Bad element name");
                    }
                }
                else {
                    return make_error(state, "Expecting element name");
                }
            },
            St::GetCoord => {
                skip_space(state);
                let c = state.c;
                if c == '0' || c == '1' || c == '2' || c == '3' || c == '4' || c == '5' ||
                   c == '6' || c == '7' || c == '8' || c == '9' || c == '-' || c == 'e' || c == '.' {
                    state.current_str.push(c);
                }
                else if c == '\n' {
                    let ns: String = state.current_str.iter().cloned().collect();
                    match ns.as_str().parse::<f64>() {
                        Ok(n) => {
                            let nn = n as g::Scalar;
                            state.current_coords.push(nn);
                            return make_error(state, "TEMP!");
                        },
                        Err(_) => {
                            return make_error(state, "Error parsing coordinate floating point value.");
                        }
                    }
                }
                else if char::is_whitespace(c) {
                    state.current_coords.push(0.0);
                    skip_space(state);
                }
            }
        }
    }

    make_error(state, "TODO RESULT")
}