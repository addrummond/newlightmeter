use geom as g;
use nom::{self, digit, space};
use std::str;

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub err: String
}

pub struct ImportedGeometry {
    segments: Vec<g::Segment>,
    material_properties: Vec<g::MaterialProperties>
}

#[derive(Debug)]
enum SegmentType {
    Line,
    Arc
}

fn make_segment(t: &SegmentType, coords: &Vec<g::Scalar>) -> Result<g::Segment, ()> {
    match *t {
        SegmentType::Line => {
            if (coords.len() != 4)
                { return Err(()); }
            
            return Ok(g::Segment {
                p1: g::Point2::new(coords[0], coords[1]),
                p2: g::Point2::new(coords[2], coords[3])
            });
        },
        SegmentType::Arc => {
            return Err(());
        }
    }
}

fn is_floatchar(v: u8) -> bool {
    return nom::is_digit(v) || v == '.' as u8 || v == '+' as u8 || v == '-' as u8 || v == 'e' as u8;
}

named!(
    floatchars<&str>,
    map_res!(take_while1!(is_floatchar), str::from_utf8)
);

named!(
    numlit<g::Scalar>,
    map_res!(
        floatchars,
        |x: &str| {
            x.parse::<f64>()
        }
    )
);

named!(segtype<SegmentType>,
    alt!(tag!("line") => { |_| SegmentType::Line }
    |
    tag!("arc") => { |_| SegmentType::Arc }
));

named!(
    entry<(g::Segment, g::MaterialProperties)>,
    map_res!(
        do_parse!(
            t: segtype >>
            space >>
            coords: separated_nonempty_list!(space, numlit) >>
            (t, coords)
        ),
        |(t, coords)| {
            let r = make_segment(&t, &coords);
            if let Ok(seg) = r {
                let mt = g::make_dummy_material_properties();
                return Ok((seg, mt));
            }
            else {
                return Err("My error message");
            }
        }
    )
);

mod tests {
    use geom_import;

    #[test]
    fn parse_to_segments_test1() {
        println!("===> {:?}", geom_import::entry(b"line 1.0 2.0 3.0 4.0"))
        //let input = "lineseg 0.0 1.0 2.0 3.0";
        //println!("STARTING");
        //let r = geom_import::parse_to_segments(input);
        //println!("{:?}", r);
    }
}