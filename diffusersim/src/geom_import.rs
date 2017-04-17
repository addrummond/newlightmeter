use geom as g;
use trace as t;
use std::str;
use std::collections::HashMap;
use std::iter;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;

use parcombs as p;

#[derive(Debug, Clone)]
pub enum Beam {
    Collimated {
        from: g::Point2,
        to: g::Point2,
        n_rays: usize,
        shiny_side_is_left: bool,
        light_properties: t::LightProperties
    },
    Ray {
        from: g::Point2,
        to: g::Point2,
        light_properties: t::LightProperties
    }
}

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

#[derive(Debug)]
pub struct ImportedGeometry {
    pub segments: Vec<g::Segment>,
    pub materials: Vec<t::MaterialProperties>,
    pub beams: Vec<Beam>,
    pub left_material_properties: Vec<u8>,
    pub right_material_properties: Vec<u8>,
    pub segment_names: HashMap<usize, String>
}

fn entry(st: &mut p::ParseState) -> p::ParseResult<Vec<Entry>> {
    match p::identifier(st) {
        Err(e) => { Err(e) },
        Ok(ident) => {
            if let Err(e) = p::skip_at_least_one_space(st)
                { return Err(e); }

            let r: p::ParseResult<Vec<Entry>>;
            if ident == "line" {
                r = line_entry(st);
            }
            else if ident == "arc" {
                r = arc_entry(st, false);
            }
            else if ident == "circle" {
                r = arc_entry(st, true);
            }
            else if ident == "material" {
                r = material_entry(st);
            }
            else if ident == "ray" {
                r = ray_entry(st);
            }
            else if ident == "colbeam" {
                r = colbeam_entry(st);
            }
            else {
                return p::parse_error_string(st, format!("Unrecognized entry type '{}'", ident));
            }

            // We expect possible whitespace followed by newline
            // or EOF.
            let term = p::skip_space(st)?;
            if !st.at_eof() && term.is_some() && term.unwrap() != '\n' {
                return p::parse_error_string(st, format!("Junk at end of '{}' def", ident));
            }

            r
        }
    }
}

fn material_pair(st: &mut p::ParseState) -> p::ParseResult<(String, String)> {
    let i1 = p::identifier(st)?;
    p::skip_space(st)?;
    p::expect_str(st, "/")?;
    p::skip_space(st)?;
    let i2 = p::identifier(st)?;
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

fn optional_name(st: &mut p::ParseState) -> p::ParseResult<Option<String>> {
    match p::peek_char(st)? {
        None => { return Ok(None) },
        Some(c) => {
            if c == '\n'
                { return Ok(None); }
            else if c != 'n'
                { return p::parse_error(st, "Expected end of segment/arc definition or optional name"); }
            p::expect_str(st, "named")?;
            p::skip_at_least_one_space(st)?;
            let name = p::identifier(st)?;
            Ok(Some(name))
        }
    }
}

fn line_entry(st: &mut p::ParseState) -> p::ParseResult<Vec<Entry>> {
    let (i1, i2) = material_pair(st)?;
    let mut coords: [g::Scalar; 4] = [0.0; 4];

    for i in 0..4 {
        p::skip_at_least_one_space(st)?;
        let n = p::numeric_constant(st)? as g::Scalar;
        coords[i] = n;
    }

    p::skip_space(st)?;
    let name = optional_name(st)?;

    Ok(vec![make_segment_entry(
        coords[0], coords[1], coords[2], coords[3],
        name,
        i1, i2
    )])
}

fn arc_entry(st: &mut p::ParseState, is_circle: bool) -> p::ParseResult<Vec<Entry>> {
    let (i1, i2) = material_pair(st)?;

    p::skip_space(st)?;

    p::expect_str(st, "(")?;

    p::skip_space(st)?;
    let n_segs_f = p::numeric_constant(st)? as g::Scalar;
    if n_segs_f < 3.0 || n_segs_f != n_segs_f.floor()
        { return p::parse_error_string(st, format!("Number of segments must be an integer >= 3 for arc/circle, not {}", n_segs_f)); }
    let n_segs = n_segs_f as usize;
    p::skip_space(st)?;

    let mut from = -1.0;
    let mut to = -1.0;

    match p::peek_char(st)? {
        None => { return p::parse_error(st, "Unexpected end of file in middle of arc/circle definition"); },
        Some(c) => {
            if c == ':' {
                p::skip_nchars(st, 1)?;
                p::skip_space(st)?;
                from = p::numeric_constant(st)? as g::Scalar;
                if from < 0.0 || from != from.floor()
                    { return p::parse_error_string(st, format!("Beginning of segment range must be integer >= 0, not {}", from)); }
                p::skip_space(st)?;
                p::expect_str(st, "-")?;
                p::skip_space(st)?;
                to = p::numeric_constant(st)? as g::Scalar;
                if to < 1.0 || to != to.floor() || to > n_segs as f64
                    { return p::parse_error_string(st, format!("End of segment range must be integer >= 1.0 and <= number of segments, not {}", to)); }
                p::skip_space(st)?;
            }
        }
    }

    p::expect_str(st, ")")?;

    p::skip_space(st)?;
    let n_coords = if is_circle { 4 } else { 6 };
    let mut coords: [g::Scalar; 6] = [0.0; 6];
    for i in 0..n_coords {
        if i != 0
            { p::skip_at_least_one_space(st)?; }
        let n = p::numeric_constant(st)?;
        coords[i] = n;
    }
    if is_circle {
        coords[4] = coords[2];
        coords[5] = coords[3];
    }

    p::skip_space(st)?;
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

fn assignment(st: &mut p::ParseState) -> p::ParseResult<(String,g::Scalar)> {
    match p::identifier(st) {
        Err(e) => { return Err(e); },
        Ok(ident) => {
            p::skip_space(st)?;
            if let Err(e) = p::expect_str(st, "=")
                { return Err(e); }
            p::skip_space(st)?;

            let v = p::numeric_constant(st)? as g::Scalar;
            Ok((ident, v))
        }
    }
}

fn assignment_hash(st: &mut p::ParseState) -> p::ParseResult<HashMap<String, g::Scalar>> {
    let (assignments, _) = p::space_separated(st, assignment)?;

    let mut m: HashMap<String, g::Scalar> = HashMap::new();
    for (n, v) in assignments {
        if m.contains_key(&n) {
            return p::parse_error_string(st, format!("Duplicate name '{}' in assignments", n));
        }

        m.insert(n, v);
    }
            
    Ok(m)
}

fn material_properties_from_assignments(st: &mut p::ParseState, assignments: &Vec<(String, g::Scalar)>)
-> p::ParseResult<t::MaterialProperties> {
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
                    return p::parse_error_string(st, "Unrecognized attribute assigned in material: ".to_string() + n);
                }

                let mut ncs: Vec<char> = Vec::new();
                let mut i = 0;
                for cc in it {
                    if !char::is_digit(cc, 10) {
                        return p::parse_error(st, "No digit following 'c' in attribute name");
                    }
                    if i > 3
                        { return p::parse_error(st, "Coefficient number has too many digits"); }
                    ncs.push(cc);
                    i += 1;
                }

                let ns: String = ncs.into_iter().collect();
                let coeffn = ns.parse::<usize>().unwrap();
                if coeffn == 0
                    { return p::parse_error(st, "Coefficients numbered from 1"); }

                coeffs.insert(coeffn, *v);
                if coeffn > max_coeff_n
                    { max_coeff_n = coeffn }
            }
            else {
                return p::parse_error(st, "Weird: empty attribute name?");
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

fn material_entry(st: &mut p::ParseState) -> p::ParseResult<Vec<Entry>> {
    match p::identifier(st) {
        Err(e) => { Err(e) },
        Ok(name) => {
            if let Err(e) = p::skip_at_least_one_space(st)
                { return Err(e); }

            let (assignments, _) = p::space_separated(st, assignment)?;
            
            match material_properties_from_assignments(st, &assignments) {
                Err(e) => { Err(e) },
                Ok(props) => { Ok(vec![Entry::Material(name, props)]) }
            }
        }
    }
}

fn ray_entry(st: &mut p::ParseState) -> p::ParseResult<Vec<Entry>> {
    let mut wavelength: g::Scalar = 0.0;
    let mut intensity: g::Scalar = 0.0;

    let mut n = 0;
    let assignments = assignment_hash(st)?;
    for (k, v) in assignments {
        if k == "l" {
            n += 1;
            wavelength = v;
        }
        else if k == "i" {
            n += 1;
            intensity = v;
        }
        else {
            return p::parse_error(st, "Unrecognized ray property");
        }
    }
    if n != 2
        { return p::parse_error(st, "Ray must be specified for 'l' (wavelength) and i (intensity)"); }
    
    p::skip_space(st)?;
    p::expect_str(st, "|")?;
    p::skip_space(st)?;

    let mut coords: [g::Scalar; 4] = [0.0; 4];
    for i in 0..4 {
        if i != 0 {
            p::skip_at_least_one_space(st)?;
        }

        let n = p::numeric_constant(st)? as g::Scalar;
        coords[i] = n;
    }

    let ray = Beam::Ray {
        from: g::Point2::new(coords[0], coords[1]),
        to: g::Point2::new(coords[2], coords[3]),
        light_properties: t::LightProperties {
            wavelength: wavelength,
            intensity: intensity
        }
    };

    Ok(vec![Entry::Beam(ray)])
}

fn colbeam_entry(st: &mut p::ParseState) -> p::ParseResult<Vec<Entry>> {
    let mut n_rays: usize = 0;
    let mut wavelength: g::Scalar = 0.0;
    let mut intensity: g::Scalar = 0.0;

    let mut n = 0;
    let assignments = assignment_hash(st)?;
    for (k, v) in assignments {
        if k == "n" {
            if v < 1.0 || v.floor() != v
                { return p::parse_error(st, "Number of rays must be a positive integer"); }
             n += 1;
             n_rays = v as usize;
        }
        else if k == "l" {
            n += 1;
            wavelength = v;
        }
        else if k == "i" {
            n += 1;
            intensity = v;
        }
        else {
            return p::parse_error(st, "Unrecognized beam property");
        }
    }
    if n != 3
        { return p::parse_error(st, "Collimated beam must be specified for 'n' (number of rays), 'l' (wavelength) and 'i' (intensity)"); }
    
    p::skip_space(st)?;
    
    let mut i = 0;
    let mut first_was_dash = false;
    let mut err = false;
    while let Some(c) = p::peek_char(st)? {
        if i > 1
            { break; }

        if c == '-' {
            if i == 0
                { first_was_dash = true; }
        }
        else if c != '|' {
            err = true;
            break;
        }

        p::skip_peeked(st, c);
        i += 1;
    }

    if err
        { return p::parse_error(st, "Expecting |- or -|"); }

    p::skip_space(st)?;

    let mut coords: [g::Scalar; 4] = [0.0; 4];
    for i in 0..4 {
        if i != 0 {
            p::skip_at_least_one_space(st)?;
        }

        let n = p::numeric_constant(st)? as g::Scalar;
        coords[i] = n;
    }

    let beam = Beam::Collimated {
        from: g::Point2::new(coords[0], coords[1]),
        to:   g::Point2::new(coords[2], coords[3]),
        n_rays: n_rays,
        shiny_side_is_left: first_was_dash,
        light_properties: t::LightProperties {
            wavelength: wavelength,
            intensity: intensity
        }
    };

    Ok(vec![Entry::Beam(beam)])
}

fn entry_sep(st: &mut p::ParseState) -> p::ParseResult<()> {
    p::skip_space(st)?;
    if let Err(_) = p::expect_str(st, "\n")
        { return p::parse_error(st, "Expecting newline separator"); }
    p::skip_space_inc_nl(st)?;
    Ok(())
}

fn document(st: &mut p::ParseState) -> p::ParseResult<ImportedGeometry> {
    p::skip_space(st)?;
    let (r, e) = p::sep_by(st, entry_sep, entry)?;

    p::skip_space_inc_nl(st)?;
    if !st.at_eof() {
        return Err(e);
    }

    entries_to_imported_geometry(st, &r)
}

fn entries_to_imported_geometry(st: &mut p::ParseState, entries: &Vec<Vec<Entry>>) -> p::ParseResult<ImportedGeometry> {
    let mut material_lookup: HashMap<&str, u8> = HashMap::new();
    let mut materials: Vec<t::MaterialProperties> = Vec::new();
    let mut beams: Vec<Beam> = Vec::new();

    let mut mi = 0;
    for v in entries {
        for e in v {
            if let Entry::Material(ref name, ref props) = *e {
                if materials.len() >= 255 {
                    return p::parse_error(st, "Cannot have more than 255 materials.");
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
    let mut existing_segment_names: HashMap<String, bool> = HashMap::new();

    let mut seg_i = 0;
    for v in entries {
        for e in v {
            if let Entry::Segment { ref name, left_material: ref ml, right_material: ref mr, segment: ref seg } = *e {
                match material_lookup.get(ml.as_str()) {
                    None => { return p::parse_error(st, "Unknown material"); },
                    Some (pl) => {
                        match material_lookup.get(mr.as_str()) {
                            None => { return p::parse_error(st, "Unknown material"); },
                            Some (pr) => {
                                segs.push(seg.clone());
                                lmat.push(*pl);
                                rmat.push(*pr);

                                if let Some(ref n) = *name {
                                    if let Some(_) = existing_segment_names.insert(n.clone(), true) {
                                        return p::parse_error_string(st, format!("Duplicate segment name {}", n));
                                    }
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

pub fn parse_geometry(input: &[u8]) -> p::ParseResult<ImportedGeometry> {
    let mut st = p::ParseState::new(input);
    document(&mut st)
}

pub fn parse_geometry_file(filename: &str) -> io::Result<p::ParseResult<ImportedGeometry>> {
    let file = File::open(filename)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents: Vec<u8> = Vec::new();
    buf_reader.read_to_end(&mut contents)?;

    Ok(parse_geometry(contents.as_slice()))
}
