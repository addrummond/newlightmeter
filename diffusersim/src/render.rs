use geom as g;
use nalgebra as n;
use simplesvg;

pub type DisplayTransformMatrix = n::Matrix3<g::Scalar>;

pub struct DisplayTransform {
    pub matrix: DisplayTransformMatrix,
    pub width: g::Scalar,
    pub height: g::Scalar,
    pub min_x: g::Scalar,
    pub min_y: g::Scalar
}

pub fn get_display_transform(
    segments: &Vec<g::Segment>,
    width: u32,
    height: u32,
    border_factor: g::Scalar,
    offset_x: g::Scalar,
    offset_y: g::Scalar)
-> DisplayTransform {
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

        let xscale = (1.0-border_factor)*(w/ww);
        let yscale = (1.0-border_factor)*(h/hh);

        let ox = offset_x + ((border_factor)*ww)/2.0;
        let oy = offset_y + ((border_factor)*hh)/2.0;

        DisplayTransform {
            matrix: n::Matrix3::new(
                xscale, 0.0,     (-min_x*xscale) + ox,
                0.0,    -yscale, (-min_y*yscale) + oy,
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

fn tp(p: g::Point2, t: &DisplayTransform) -> [g::Scalar; 2] {
    let pv: n::Vector3<g::Scalar> = n::Vector3::new(p.coords[0], p.coords[1], 1.0);
    let r = t.matrix * pv;
    return [r[0] as g::Scalar, r[1] as g::Scalar];
}

fn tl(ln: [g::Scalar; 4], t: &DisplayTransform) -> [g::Scalar; 4] {
    let p1v: n::Vector3<g::Scalar> = n::Vector3::new(ln[0], ln[1], 1.0);
    let p2v: n::Vector3<g::Scalar> = n::Vector3::new(ln[2], ln[3], 1.0);
    let r1 = t.matrix * p1v;
    let r2 = t.matrix * p2v;
    return [r1[0], r1[1], r2[0], r2[1]];
}

fn edge_clip_ray_dest(r: &g::Ray, t: &DisplayTransform) -> g::Point2 {
    let v1 = r.p1.coords;
    let v2 = r.p2.coords;

    // Handle the vertical case.
    if v1[0] == v2[0] {
        if v1[1] <= v2[1]
            { return g::Point2::new(v1[0], t.min_y + t.height); }
        else
            { return g::Point2::new(v1[0], t.min_y); }
    }

    let m = (v2[1] - v1[1]) / (v2[0] - v1[0]);
    let k = v1[1] - m*v1[0];
        
    let yatxmin = m * t.min_x + k;
    let yatxmax = m * (t.min_x + t.width) + k;
    
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
        let xup = v2[0] - v1[0] >= 0.0;
        let yup = v2[1] - v1[1] >= 0.0;

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

pub fn to_svg_color(color: [f32; 3]) -> simplesvg::ColorAttr {
    simplesvg::ColorAttr::Color(
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8
    )
}

pub fn render_segments(segments: &Vec<g::Segment>, t: &DisplayTransform, color: [f32; 3])
-> simplesvg::Fig {
    let mut segs: Vec<simplesvg::Fig> = Vec::new();
    for s in segments {
        let tp1 = tp(s.p1, t);
        let tp2 = tp(s.p2, t);

        let mut attr = simplesvg::Attr::default();
        attr.fill = None;
        attr.stroke = Some(to_svg_color(color));
        attr.stroke_width = Some(2.0);
        attr.opacity = Some(1.0);
        attr.font_family = None;

        segs.push(
            simplesvg::Fig::Styled(
                attr,
                Box::new(simplesvg::Fig::Line(
                    tp1[0] as f32,
                    tp1[1] as f32,
                    tp2[0] as f32,
                    tp2[1] as f32
                ))
            )
        );
    }

    simplesvg::Fig::Multiple(segs)
}

pub fn render_rays(rays: &Vec<(g::Ray, g::RayProperties)>, t: &DisplayTransform, color: [f32; 3])
-> simplesvg::Fig {
    let mut figs: Vec<simplesvg::Fig> = Vec::new();
    for &(ref r, _) in rays {
        let tp1 = tp(r.p1, t);
        let tp2 = tp(edge_clip_ray_dest(r, t), t);

        let diam = 3.0;

        let mut line_attr = simplesvg::Attr::default();
        line_attr.fill = None;
        line_attr.stroke = Some(to_svg_color(color));
        line_attr.stroke_width = Some(1.0);
        line_attr.opacity = Some(1.0);
        line_attr.font_family = None;

        let mut circle_attr = simplesvg::Attr::default();
        circle_attr.fill = Some(to_svg_color(color));
        circle_attr.stroke = Some(to_svg_color(color));
        circle_attr.stroke_width = Some(2.0);
        circle_attr.opacity = Some(1.0);
        circle_attr.font_family = None;

        figs.push(
            simplesvg::Fig::Multiple(
                vec![                        
                    simplesvg::Fig::Styled(line_attr, Box::new(simplesvg::Fig::Line(
                        tp1[0] as f32,
                        tp1[1] as f32,
                        tp2[0] as f32,
                        tp2[1] as f32
                    ))),
                    simplesvg::Fig::Styled(circle_attr, Box::new(simplesvg::Fig::Circle(
                        tp1[0] as f32,
                        tp1[1] as f32,
                        diam as f32
                    )))
                ]
            )
        );
    }

    simplesvg::Fig::Multiple(figs)
}
