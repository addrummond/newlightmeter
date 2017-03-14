use piston_window;
use geom as g;
use nalgebra as n;

type GScalar = piston_window::math::Scalar;
type PistonWindow = piston_window::PistonWindow;

pub type DisplayTransformMatrix = n::Matrix3<g::Scalar>;

pub struct DisplayTransform {
    pub matrix: DisplayTransformMatrix,
    pub width: g::Scalar,
    pub height: g::Scalar,
    pub min_x: g::Scalar,
    pub min_y: g::Scalar
}

pub fn get_display_transform(segments: &Vec<g::Segment>, width: u32, height: u32) -> DisplayTransform {
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
                Some(x2) => { if *x < x2 { min_x = Some(*x) } }
            }
            match max_x {
                None => { max_x = Some(*x) }
                Some (x2) => { if *x > x2 { max_x = Some(*x) } }
            }
        }
        for y in &[s.p1.coords[1], s.p2.coords[1]] {
            match min_y {
                None => { min_y = Some(*y) },
                Some(y2) => { if *y < y2 { min_y = Some(*y) } }
            }
            match max_y {
                None => { max_y = Some(*y) }
                Some (y2) => { if *y > y2 { max_y = Some(*y) } }
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

        DisplayTransform {
            matrix: n::Matrix3::new(
                xscale, 0.0,     -min_x*xscale,
                0.0,    -yscale, -min_y*yscale,
                0.0,    0.0,     1.0
            ),
            width: ww,
            height: hh,
            min_x: min_x,
            min_y: min_y
        }
    }
    else {
        DisplayTransform {
            matrix: n::Matrix3::new(
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
                0.0, 0.0, 1.0
            ),
            width: 1.0,
            height: 1.0,
            min_x: 0.0,
            min_y: 0.0
        }
    }
}

fn tp(p: g::Point2, t: &DisplayTransform) -> [GScalar; 2] {
    let pv: n::Vector3<g::Scalar> = n::Vector3::new(p.coords[0], p.coords[1], 1.0);
    let r = t.matrix * pv;
    return [r[0] as GScalar, r[1] as GScalar];
}

fn tl(ln: [g::Scalar; 4], t: &DisplayTransform) -> [GScalar; 4] {
    let p1v: n::Vector3<g::Scalar> = n::Vector3::new(ln[0], ln[1], 1.0);
    let p2v: n::Vector3<g::Scalar> = n::Vector3::new(ln[2], ln[3], 1.0);
    let r1 = t.matrix * p1v;
    let r2 = t.matrix * p2v;
    return [r1[0], r1[1], r2[0], r2[1]];
}

fn edge_clip_ray_dest(r: &g::Ray, t: &DisplayTransform) -> g::Point2 {
    let v1 = r.p1.coords;
    let v2 = r.p2.coords;
    let d = v2 - v1;

    let m = (v2[1] - v1[1]) / (v2[0] - v1[0]);
    let k = v1[1] - m*v1[0];
        
    let yatxmin = m * t.min_x + k;
    let yatxmax = m * (t.min_x + t.width) + k;
    
    let xup = v2[0] - v1[0] >= 0.0;
    let yup = v2[1] - v1[1] >= 0.0;

    let ex: g::Scalar;
    let ey: g::Scalar;

    if m == 0.0 {
        if v1[0] < v2[0] {
            // Right edge.
            ex = t.min_x + t.width;
            ey = yatxmax;
        }
        else {
            // Left edge.
            ex = t.min_x;
            ey = yatxmin;
        }
    }
    else {
        let xatymin = (t.min_y - k) / m;
        let xatymax = (t.min_y + t.height - k) / m;

        if !xup && yatxmin >= t.min_y && yatxmin <= t.min_y + t.height {
            // Left edge.
            ex = t.min_x;
            ey = yatxmin;
        }
        else if xup && yatxmax >= t.min_y && yatxmax <= t.min_y + t.height {
            // Right edge.
            ex = t.min_x + t.height;
            ey = yatxmax;
        }
        else if !yup && xatymin >= t.min_x && xatymin <= t.min_x + t.width {
            // Bottom edge.
            ex = xatymin;
            ey = t.min_y;
        }
        else {
            // Top edge.
            ex = xatymax;
            ey = t.min_y + t.height;
        }
    }

    g::Point2::new(ex, ey)
}

pub fn render_segments<E>(segments: &Vec<g::Segment>, window: &mut PistonWindow, e: &E, t: &DisplayTransform, color: [f32; 4])
where E: piston_window::generic_event::GenericEvent {
    for s in segments {
        let tp1 = tp(s.p1, t);
        let tp2 = tp(s.p2, t);
        
        window.draw_2d(e, |c, g| {
            piston_window::line(
                color,
                0.5, // Radius
                [tp1[0], tp1[1], tp2[0], tp2[1]],
                c.transform,
                g
            );
        });
    }
}

pub fn render_rays<E>(rays: &Vec<g::Ray>, window: &mut PistonWindow, e: &E, t: &DisplayTransform)
where E: piston_window::generic_event::GenericEvent {
    for r in rays {
        let tp1 = tp(r.p1, t);
        let tp2 = tp(edge_clip_ray_dest(r, t), t);

        let diam = 0.01 * t.height;

        window.draw_2d(e, |c, g| {
            piston_window::line(
                [1.0, 0.0, 0.0, 1.0], // Color
                0.5, // Radius
                [tp1[0], tp1[1], tp2[0], tp2[1]],
                c.transform,
                g
            );
            piston_window::ellipse(
                [1.0, 0.0, 0.0, 1.0],
                [tp1[0] - diam/2.0, tp1[1] - diam/2.0, diam, diam],
                c.transform,
                g
            )
        });
    }
}

pub fn render_qtree<E>(qtree: &g::QTree, window: &mut PistonWindow, e: &E, t: &DisplayTransform)
where E: piston_window::generic_event::GenericEvent {
    let diam = t.height*0.01;

    let mut first = true;
    for n in qtree.in_order_iter() {
        if let Some(ref ci) = n.child_info {
            let lines = [
                [ ci.center.coords[0], t.min_y, ci.center.coords[0], t.min_y + t.height],
                [ t.min_x, ci.center.coords[1], t.min_x + t.width, ci.center.coords[1] ]
            ];

            window.draw_2d(e, |c, g| {
                for l in &lines {
                    piston_window::line(
                        [0.0, 0.0, 1.0, 1.0], // Color
                        0.5, // Radius
                        tl(*l, t),
                        c.transform,
                        g
                    );
                    if first {
                        let cp = tp(ci.center, t);
                        piston_window::ellipse(
                            [0.0, 0.0, 1.0, 1.0],
                            [cp[0]-diam/2.0, cp[1]-diam/2.0, diam, diam],
                            c.transform,
                            g
                        )
                    }
                }
            });
        }

        first = false;
    }
}