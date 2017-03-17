use geom as g;
use nom::{self, digit, space, multispace, alphanumeric};
use std::str;

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub err: String
}

pub struct ImportedGeometry {
    segments: Vec<g::Segment>,
    left_material_properties: Vec<g::MaterialProperties>,
    right_material_properties: Vec<g::MaterialProperties>
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

fn make_material_properties(assignments: &Vec<(&str, g::Scalar)>) -> g::MaterialProperties {
    let mut mp = g::make_dummy_material_properties();

    for &(name, val) in assignments {
        if (name == "ri") {
            mp.refractive_index = val;
        }
        else if (name == "ex") {
            mp.extinction = val;
        }
    }

    mp
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

named!(material_name<&str>, map_res!(alphanumeric, str::from_utf8));
named!(var_name<&str>, map_res!(alphanumeric, str::from_utf8));
named!(assignment_name<&str>, map_res!(alphanumeric, str::from_utf8));

#[derive(Debug)]
enum Entry<'a> {
    Segment(&'a str, &'a str, g::Segment),
    Material(&'a str, g::MaterialProperties)
}

named!(
    segment_entry<Entry>,
    map_res!(
        do_parse!(
            t: segtype >>
            space >>
            m1: material_name >>
            ws!(tag!("/")) >>
            m2: material_name >>
            space >>
            coords: separated_nonempty_list!(space, numlit) >>
            (t, m1, m2, coords)
        ),
        |(t, m1, m2, coords)| {
            let r = make_segment(&t, &coords);
            if let Ok(seg) = r {
                let mt = g::make_dummy_material_properties();
                return Ok(Entry::Segment(m1, m2, seg));
            }
            else {
                return Err("My error message");
            }
        }
    )
);

named!(
    assignment<(&str, g::Scalar)>,
    do_parse!(
        vn: var_name >>
        //ws!(tag!("=")) >>
        ws!(tag!("=")) >>
        n: numlit >>
        (vn, n)
    )
);

named!(
    matprops_entry<Entry>,
    map_res!(
        do_parse!(
            tag!("material") >>
            space >>
            name: material_name >>
            space >>
            ass: separated_nonempty_list!(space, assignment) >>
            (name, ass)
        ),
        |(name, assignments)| -> Result<Entry, ()> {
            let mp = make_material_properties(&assignments);
            Ok(Entry::Material(name, mp))
        }
    )
);

named!(entry<Entry>, alt!(segment_entry | matprops_entry));
named!(entry_sep<()>, do_parse!(opt!(space) >> tag!("\n") >> opt!(multispace) >> ()));
//named!(entry_sep<()>, do_parse!(tag!("\n") >> ()));
//named!(entry_sep<()>, do_parse!(tag!("|") >> ()));
named!(
    document<Vec<Entry>>,
    do_parse!(
        opt!(multispace) >>
        lst: separated_nonempty_list!(entry_sep, entry) >>
        (lst)
    )
);

mod tests {
    use geom_import;

    #[test]
    fn parse_to_segments_test1() {
        println!("===> {:?}", geom_import::document(b"line oooo/oo 5 5 5 5\nline oooo/oo 5 5 5 5\nmaterial foo x=6.0\nline oooo/oo 5 5.0 5 5"))
        //let input = "lineseg 0.0 1.0 2.0 3.0";
        //println!("STARTING");
        //let r = geom_import::parse_to_segments(input);
        //println!("{:?}", r);
    }
}