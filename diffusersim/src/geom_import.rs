use geom as g;
use std::char;
use std;

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub err: String
}

fn parse_to_segments(input: &str) -> Result<Vec<g::Segment>, ParseError> {
    enum SegmentType {
        Line
    }
    
    #[derive(Debug)]
    enum St {
        SkipSpace,
        LookForSegType,
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
        current_segment_type: SegmentType
    }

    let mut state_ = State {
        kind: St::SkipSpace,
        i: input.chars(),
        c: ' ',
        eof: false,
        line: 1,
        col: 0,
        current_str: Vec::new(),
        current_coords: Vec::new(),
        current_segment_type: SegmentType::Line
    };
    let state = &mut state_;

    let mut segments: Vec<g::Segment> = Vec::new();

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
        else {
            state.eof = true;
        }
    }

    fn skip_space(state: &mut State) -> bool {
        let mut found_newline = false;
        while !state.eof && char::is_whitespace(state.c) {
            if state.c == '\n' {
                found_newline = true;
            }
            next(state);
        }
        return found_newline;
    }

    fn add_segment(state: &mut State, segments: &mut Vec<g::Segment>) -> Option<ParseError> {
        match state.current_segment_type {
            SegmentType::Line => {
                if state.current_coords.len() != 4 {
                    return Some(ParseError {
                        line: state.line,
                        col: state.col,
                        err: "Expected four coordinates for line".to_string()
                    });
                }

                segments.push(g::Segment {
                    p1: g::Point2::new(state.current_coords[0], state.current_coords[1]),
                    p2: g::Point2::new(state.current_coords[2], state.current_coords[3])
                });

                None
            }
        }
    }

    fn make_error(state: &State, msg: &str) -> Result<Vec<g::Segment>, ParseError> {
        Err (ParseError {
            line: state.line,
            col: state.col,
            err: msg.to_string()
        })
    }

    let mut second_eof = false;
    loop {
        // States get one chance to look at the EOF value.
        if state.eof {
            if !second_eof {
                second_eof = true;
            }
            else {
                match state.kind {
                    St::SkipSpace => {
                        return Ok(segments);
                    }
                    St::LookForSegType => {
                        return make_error(state, "Unexpected end of input");
                    }
                    St::GetCoord => {
                        return make_error(state, "Unexpected end of input");
                    }
                }
            }
        }

        match state.kind {
            St::SkipSpace => {
                skip_space(state);
                state.kind = St::LookForSegType;
            },
            St::LookForSegType => {
                if char::is_alphanumeric(state.c) {
                    state.current_str.push(state.c);
                }
                else if char::is_whitespace(state.c) {
                    let s: String = state.current_str.iter().cloned().collect();
                    if s == "lineseg" {
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
                next(state);
            },
            St::GetCoord => {
                let c = state.c;
                if !state.eof &&
                   (c == '0' || c == '1' || c == '2' || c == '3' || c == '4' || c == '5' ||
                   c == '6' || c == '7' || c == '8' || c == '9' || c == '-' || c == 'e' || c == '.') {
                    state.current_str.push(c);
                    next(state);
                }
                else if state.eof || char::is_whitespace(c) {
                    let newline = skip_space(state);
                    let c = state.c;
                    let ns: String = state.current_str.iter().cloned().collect();
                    match ns.as_str().parse::<f64>() {
                        Ok(n) => {
                            let nn = n as g::Scalar;
                            state.current_coords.push(nn);
                            state.current_str.clear();
                            if newline || state.eof {
                                add_segment(state, &mut segments);
                                state.current_coords.clear();
                                state.kind = St::SkipSpace;
                            }
                        },
                        Err(_) => {
                            return make_error(state, "Error parsing coordinate floating point value.");
                        }
                    }
                }
                else {
                    return make_error(state, "Unexpected character.");
                }
            }
        }
    }
}

mod tests {
    use geom_import;

    #[test]
    fn parse_to_segments_test1() {
        let input = "lineseg 0.0 1.0 2.0 3.0";
        println!("STARTING");
        let r = geom_import::parse_to_segments(input);
        println!("{:?}", r);
    }
}