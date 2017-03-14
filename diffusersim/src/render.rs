use piston_window;
use geom as g;
use nalgebra as n;

type GScalar = piston_window::math::Scalar;
type PistonWindow = piston_window::PistonWindow;

pub type DisplayTransformMatrix = n::Matrix3<g::Scalar>;

pub fn get_display_transform(segments: &Vec<g::Segment>, width: u32, height: u32) -> n::Matrix3<g::Scalar> {
    let w = width as g::Scalar;
    let h = height as g::Scalar;

    let mut min_x : Option<g::Scalar> = None;
    let mut max_x : Option<g::Scalar> = None;
    let mut min_y : Option<g::Scalar> = None;
    let mut max_y : Option<g::Scalar> = None;

    for s in segments {
        for x in &[s.p1.coords[0], s.p2.coords[0]] {
            match min_x {
                None => { min_x = Some(*x) },
                Some(x2) => { if (*x < x2) { min_x = Some(*x) } }
            }
            match max_x {
                None => { max_x = Some(*x) }
                Some (x2) => { if (*x > x2) { max_x = Some(*x) } }
            }
        }
        for y in &[s.p1.coords[1], s.p2.coords[1]] {
            match min_y {
                None => { min_y = Some(*y) },
                Some(y2) => { if (*y < y2) { min_y = Some(*y) } }
            }
            match max_y {
                None => { max_y = Some(*y) }
                Some (y2) => { if (*y > y2) { max_y = Some(*y) } }
            }
        }
    }

    if segments.len() > 0 {
        let max_x = max_x.unwrap();
        let max_y = max_y.unwrap();
        let min_x = min_x.unwrap();
        let min_y = min_y.unwrap();

        let mut ww = max_x - min_x;
        let mut hh = max_y - min_y;
        if ww == 0.0 { ww = 1.0 }
        if hh == 0.0 { hh = 1.0 }

        let xscale = w/ww;
        let yscale = h/hh;

        return n::Matrix3::new(
            xscale, 0.0,     -min_x*xscale,
            0.0,    -yscale, -min_y*yscale,
            0.0,    0.0,     1.0
        );
    }
    else {
        return n::Matrix3::new(
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0
        );
    }
}

pub fn render_segments<E: piston_window::generic_event::GenericEvent>(segments: &Vec<g::Segment>, window: &mut PistonWindow, e: &E, t: &DisplayTransformMatrix)
where E: piston_window::generic_event::GenericEvent {
    for s in segments {
        let mut p1: n::Vector3<g::Scalar> = n::Vector3::new(s.p1.coords[0], s.p1.coords[1], 1.0);
        let mut p2: n::Vector3<g::Scalar> = n::Vector3::new(s.p2.coords[0], s.p2.coords[1], 1.0);

        p1 = t * p1;
        p2 = t * p2;
        
        window.draw_2d(e, |c, g| {
            piston_window::line(
                [1.0, 0.0, 0.0, 1.0], // Color
                0.5, // Radius
                [p1[0], p1[1], p2[0], p2[1]],
                c.transform,
                g
            );
        });
    }
}