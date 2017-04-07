use std::collections::HashSet;
use rand::{Rng, SeedableRng, StdRng};

//
// Basic types.
//

use nalgebra::Vector2 as Vector2_;
use nalgebra::Point2 as Point2_;
use nalgebra;
use std::f64::consts;
use std::mem;

pub type Scalar = f64;
pub type Vector2 = Vector2_<Scalar>;
pub type Point2 = Point2_<Scalar>;

const EPSILON: Scalar = 0.0001;

//
// Basic geometric primitive.
//

#[derive(Copy, Clone, Debug)]
pub struct Segment {
    // p1.x < p2.x || (p1.x == p2.x && p1.y < p2.y)
    pub p1: Point2,
    pub p2: Point2
}

//
// Utilities for constructing sequences of segments that approximate various
// bits of geometry. Note that these do not need to be particularly efficient
// as they are just used for constructing geometry, and are not called during
// ray tracing.
//

pub fn arc_to_segments(center: Point2, start: Point2, end: Point2, n_segs: usize)
-> Vec<Segment> {
    assert!(n_segs > 0);

    let l1 = (start - center).normalize();
    let l2 = (end - center).normalize();

    let rad = nalgebra::distance(&center, &start);

    let dot = nalgebra::dot(&l1, &l2);
    let ac = dot.acos();
    let angle;
    if ac == 0.0 {
        angle = 2.0*consts::PI;
    }
    else {
        angle = ac;
    }
    let dot2 = nalgebra::dot(&Vector2::new(0.0, 1.0), &l1);
    let angle_offset = dot2.acos();

    let nsf = n_segs as Scalar;
    let mut segments: Vec<Segment> = Vec::new();
    let mut current_point = Point2::new(angle_offset.sin() * rad, angle_offset.cos() * rad);
    for i in 1_usize..(n_segs+1) {
        let nf = i as Scalar;
        let a = ((angle / nsf) * nf) + angle_offset;
        let y = a.cos() * rad;
        let x = a.sin() * rad;
        let to = Point2::new(x, y);
        segments.push(Segment { p1: current_point, p2: to });
        current_point = to;
    }

    segments
}

//
// QTrees
//

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    // Origin at p1, pointing in direction of p2.
    pub p1: Point2,
    pub p2: Point2
}

pub fn seg(x1: Scalar, y1: Scalar, x2: Scalar, y2: Scalar) -> Segment {
    let p1: Point2;
    let p2: Point2;

    if x1 < x2 || (x1 == x2 && y1 < y2) {
        p1 = Point2::new(x1, y1);
        p2 = Point2::new(x2, y2);
    }
    else {
        p2 = Point2::new(x1, y1);
        p1 = Point2::new(x2, y2);
    }

    Segment {
        p1: p1,
        p2: p2
    }
}

pub fn ray(x1: Scalar, y1: Scalar, x2: Scalar, y2: Scalar) -> Ray {
    Ray {
        p1: Point2::new(x1, y1),
        p2: Point2::new(x2, y2)
    }
}

const QTREE_BIN_SIZE : usize = 8;

#[derive(Debug)]
pub struct QTreeChildInfo<'a,SI>
where SI: 'a + Copy {
    pub center: Point2,
    pub children: [Box<QTreeNode<'a,SI>>; 4] // Clockwise from NW
}

#[derive(Debug)]
pub struct QTreeNode<'a, SI>
where SI: 'a + Copy {
    pub segments: Vec<(&'a Segment, SI)>,
    pub child_info: Option<QTreeChildInfo<'a,SI>>
}

#[derive(Debug)]
pub struct QTree<'a, SI>
where SI: 'a + Copy {
    root: Box<QTreeNode<'a, SI>>
}

fn get_point_quad(p: Point2, c: Point2) -> i32 {
    if p.coords[0] <= c.coords[0] {
        if p.coords[1] <= c.coords[1] {
            return 3;
        }
        else {
            return 0;
        }
    }
    else {
        if p.coords[1] <= c.coords[1] {
            return 2;
        }
        else {
            return 1;
        }
    }
}

fn get_segment_quad_mask(segment: &Segment, c: Point2) -> i32
{
    let q1 = get_point_quad(segment.p1, c);
    let q2 = get_point_quad(segment.p2, c);

    if q1 == q2 {
        return 1 << q1;
    }
    else {
        let m = (1 << q1) | (1 << q2);
        if m == 0b0101 || m == 0b1010 {
            // The diagonal case. We now need to determine whether
            // or not the line passes above or below the origin.
            let slope = (segment.p2.coords[1] - segment.p1.coords[1]) /
                        (segment.p2.coords[0] - segment.p1.coords[0]);
            let py = slope * segment.p1.coords[0];
            let c = segment.p1.coords[1] - py;
            if c < 0.0 {
                // If the line starts in NW and ends in SE, then it also passes
                // through SW. If the line starts in SW and ends in NE, then it
                // also passes through SE. So we can just add both SW and SE to
                // the mask.
                return m | 0b1100;
            }
            else {
                // If the line starts in NW and ends in SE, then it also passes
                // through NE. If the line starts in SW and ends in NE, then it
                // also passes through NW. So we can just add both NW and NE to
                // the mask.
                return m | 0b0011;
            }
        }
        else {
            return m;
        }
    }
}

pub struct QTreeInOrderIterator<'a, SI>
where SI: 'a + Copy {
    stack: Vec<(usize, &'a QTreeNode<'a,SI>)>,
}

impl<'a, SI> Iterator for QTreeInOrderIterator<'a, SI>
where SI: 'a + Copy
{
    type Item = (usize, &'a QTreeNode<'a,SI>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.stack.pop() {
            None => { None },
            Some((depth, t)) => {
                if let Some(ref x) = t.child_info {
                    self.stack.extend(x.children.iter().map(|x| (depth+1, &**x)));
                }
                Some((depth, t))
            }
        }
    }
}

fn ray_intersects_segment(ray: &Ray, segment: &Segment) -> Option<Point2> {
    //println!("TESTINTER RAY {} {} {} {}", segment.p1.coords[0],segment.p1.coords[1],segment.p2.coords[0],segment.p2.coords[1]);

    let ray_slope_num = ray.p2.coords[1] - ray.p1.coords[1];
    let seg_slope_num = segment.p2.coords[1] - segment.p1.coords[1];
    let ray_slope_denom = ray.p2.coords[0] - ray.p1.coords[0];
    let seg_slope_denom = segment.p2.coords[0] - segment.p1.coords[0];

    if ray_slope_num == 0.0 && seg_slope_num == 0.0 {
        // Parallel horizontal lines.
        // Does not count as an intersection for purposes of ray tracing,
        // even if the lines overlap.
        return None;
    }
    else if ray_slope_denom == 0.0 && seg_slope_denom == 0.0 {
        // Parallel vertical lines.
        return None;
    }

    let ray_slope = ray_slope_num / ray_slope_denom;
    let seg_slope = seg_slope_num / seg_slope_denom;

    if ray_slope == seg_slope {
        // Parallel lines.
        return None;
    }

    // Calculate intersection point;
    let x;
    let y;
    if ray_slope_denom == 0.0 {
        // We can quickly determine that there's no intersection if the ray is vertical
        // and the segment doesn't overlap the relevant y coordinate.
        if ! (ray.p1.coords[0] >= segment.p1.coords[0] && ray.p1.coords[0] <= segment.p2.coords[0]) ||
             (ray.p1.coords[0] <= segment.p1.coords[0] && ray.p1.coords[0] >= segment.p2.coords[0]) {
            return None;
        }
        x = ray.p1.coords[0];
        let seg_k = segment.p1.coords[1] - (seg_slope * segment.p1.coords[0]);
        y = (seg_slope * x) + seg_k;
    }
    else if seg_slope_denom == 0.0 {
        x = segment.p1.coords[0];
        let ray_k = ray.p1.coords[1] - (ray_slope * ray.p1.coords[0]);
        y = (ray_slope * x) + ray_k;
    }
    else {
        let ray_k = ray.p1.coords[1] - (ray_slope * ray.p1.coords[0]);
        let seg_k = segment.p1.coords[1] - (seg_slope * segment.p1.coords[0]);

        x = (seg_k - ray_k) / (ray_slope - seg_slope);
        y = (seg_k*ray_slope - ray_k*seg_slope) / (ray_slope - seg_slope);
    }

    //println!("CALC: {} {}", x, y);

    // Is the intersection point on the ray?
    if ray_slope_num > 0.0 && y < ray.p1.coords[1]
        { return None; }
    else if ray_slope_num < 0.0 && y > ray.p1.coords[1]
        { return None; }
    if ray_slope_denom > 0.0 && x < ray.p1.coords[0]
        { return None; }
    else if ray_slope_denom < 0.0 && x > ray.p1.coords[0]
        { return None; }
    
    // It's on the ray. Is it on the segment?
    // Because of the ordering of segment points, we know that
    // the x value of the first point <= the x value of the second point.
    if x >= segment.p1.coords[0] - EPSILON && x <= segment.p2.coords[0] + EPSILON &&
       ((y >= segment.p1.coords[1] - EPSILON && y <= segment.p2.coords[1] + EPSILON) ||
        (y <= segment.p1.coords[1] + EPSILON && y >= segment.p2.coords[1] - EPSILON)) {
        return Some(Point2::new(x, y));
    }
    else {
        //println!("ULTIFAIL {} {} {} {}", segment.p1.coords[0], segment.p1.coords[1], segment.p2.coords[0], segment.p2.coords[1]);
        return None;
    }
}

pub struct CollimatedBeamRayIterator {
    p1: Point2,
    x: Scalar,
    y: Scalar,
    normal: Vector2,
    i: usize,
    n_rays: usize
}

impl CollimatedBeamRayIterator {
    pub fn new(p1: Point2, p2: Point2, shiny_side_is_left: bool, n_rays: usize)
    -> CollimatedBeamRayIterator {
        let x = p2.coords[0] - p1.coords[0];
        let y = p2.coords[1] - p1.coords[1];

        // The left normal (looking "along" the line)
        let mut nx = -y;
        let mut ny = x;

        if !shiny_side_is_left {
            nx = -nx;
            ny = -ny;
        }

        let n_rays_s = n_rays as Scalar;

        CollimatedBeamRayIterator {
            p1: p1,
            x: x / n_rays_s,
            y: y / n_rays_s,
            normal: Vector2::new(nx, ny),
            i: 0,
            n_rays: n_rays
        }
    }
}

impl Iterator for CollimatedBeamRayIterator {
    type Item = (Point2, Point2);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.n_rays
            { return None; }
        
        let ii = self.i as Scalar;

        let mut origin = self.p1;
        origin.coords[0] += ii * self.x;
        origin.coords[1] += ii * self.y;

        let dest = origin + self.normal;

        self.i += 1;

        Some((origin, dest))
    }
}

pub struct QTreeRayTraceIterator<'a, 'b, SI>
where SI: 'a + Copy {
    ray: &'b Ray,
    ray_m: Scalar,
    ray_k: Scalar,
    stack: Vec<(bool, &'a QTreeNode<'a,SI>)>
}

impl<'a,'b, SI> Iterator for QTreeRayTraceIterator<'a, 'b, SI>
where SI: 'a + Copy {
    type Item = &'a Vec<(&'a Segment, SI)>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(&(already, r)) = self.stack.last() {
            if !already {
                let i = self.stack.len() - 1;
                self.stack[i].0 = true;
                return Some(&r.segments);
            }

            self.stack.pop();

            if let Some(ref child_info) = r.child_info {
                // The ray starts from p1, so at least the quad
                // that q1 is in should be added to the mask.
                let ref center = child_info.center;
                let mut quad_mask = 1 << get_point_quad(self.ray.p1, *center);

                if self.ray.p1.coords[1] != self.ray.p2.coords[1] {
                    //println!("FIRST TEST {} {} {}", child_info.center.coords[1], k, m);
                    let x_intercept = (child_info.center.coords[1] - self.ray_k) / self.ray_m;
                    let s1 = self.ray.p2.coords[1] - self.ray.p1.coords[1] >= 0.0;
                    let s2 = child_info.center.coords[1] - self.ray.p1.coords[1] >= 0.0;
                    //println!("CRUCIAL {} {} {}", s1, s2, x_intercept);
                    if s1 == s2 {
                        if x_intercept > child_info.center.coords[0] {
                            quad_mask |= 0b0110;
                        }
                        else {
                            //println!("OH!");
                            quad_mask |= 0b1001;
                        }
                    }
                }

                if self.ray.p1.coords[0] != self.ray.p2.coords[0] {
                    let y_intercept = (self.ray_m * child_info.center.coords[0]) + self.ray_k;
                    let s1 = self.ray.p2.coords[0] - self.ray.p1.coords[0] >= 0.0;
                    let s2 = child_info.center.coords[0] - self.ray.p1.coords[0] >= 0.0;
                    //println!("SECOND TEST {} {} {}", y_intercept, s1, s2);
                    if s1 == s2 {
                        if y_intercept > child_info.center.coords[1] {
                            quad_mask |= 0b0011;
                        }
                        else {
                            quad_mask |= 0b1100;
                        }
                    }
                }

                for i in 0..4 {
                    if quad_mask & (1 << i) != 0 {
                        //println!("PUSHING [{}] {}", quad_mask, i);
                        self.stack.push((false,&(child_info.children[i])));
                    }
                }
            }
        }

        None
    }
}

// -1 left, 0 on line, 1 right (looking from first point to second point).
pub fn point_side_of_line_segment(lp1: Point2, lp2: Point2, p: Point2) -> i32 {
    let ax = lp1.coords[0];
    let ay = lp1.coords[1];
    let bx = lp2.coords[0];
    let by = lp2.coords[1];
    let cx = p.coords[0];
    let cy = p.coords[1];

    let determinant = (bx - ax)*(cy - ay) - (by - ay)*(cx - ax);

    if determinant > 0.0
        { return -1 }
    else if determinant < 0.0
        { return 1 }
    else
        { return 0 }
}

impl<'a, SI> QTree<'a, SI>
where SI: 'a + Copy {
    pub fn make_empty_qtree() -> QTree<'a,SI>
    {
        let root = QTreeNode {
            segments: vec! [],
            child_info: None
        };
        return QTree {
            root: Box::new(root),
        };
    }

    pub fn in_order_iter(&self) -> QTreeInOrderIterator<SI> {
        QTreeInOrderIterator { stack: vec![(0, &*self.root)] }
    }

    pub fn ray_trace_iter<'b>(&'a self, ray: &'b Ray) -> QTreeRayTraceIterator<'a,'b,SI> {
        let m = (ray.p2.coords[1] - ray.p1.coords[1]) /
                (ray.p2.coords[0] - ray.p1.coords[0]);
        let k = ray.p2.coords[1] - (m * ray.p1.coords[0]);

        QTreeRayTraceIterator {
            ray: ray,
            ray_m: m,
            ray_k: k,
            stack: vec![(false, &*self.root)]
        }
    }

    pub fn insert_segment(&mut self, s: &'a Segment, info: SI)
    {
        let mut stack : Vec<&mut QTreeNode<'a,SI>> = Vec::new();
        stack.push(&mut*self.root);

        while let Some(r) = stack.pop() {
            if r.child_info.is_some() {
                let child_info = r.child_info.as_mut().unwrap();
                let mask = get_segment_quad_mask(s, child_info.center);

                let mut i = 1;
                for child in child_info.children.as_mut() {
                    if mask & i != 0 {
                        stack.push(child);
                    }

                    i <<= 1;
                }
            }
            else if r.segments.len() < QTREE_BIN_SIZE {
                r.segments.push((s, info));
            }
            else {
                // Given the sorting order for the points of a segment,
                // if we choose p2 as our new center point, the segment
                // will either be in NW or in SW.
                let new_center = s.p2;
                let in_nw = s.p1.coords[1] >= new_center.coords[0];

                let mut new_children = [
                    Box::new(QTreeNode {
                        child_info: None,
                        segments: if in_nw { vec![(s, info)] } else { vec![] }
                    }),
                    Box::new(QTreeNode {
                        child_info: None,
                        segments: vec![]
                    }),
                    Box::new(QTreeNode {
                        child_info: None,
                        segments: vec![]
                    }),
                    Box::new(QTreeNode {
                        child_info: None,
                        segments: if !in_nw { vec![(s, info)] } else { vec![] }
                    }),
                ];

                // Move segments downstairs if they're contained in only
                // one quad.
                let mut to_delete: HashSet<usize> = HashSet::new();
                let mut segi = 0;
                for &(seg, info) in &r.segments {
                    let mask = get_segment_quad_mask(seg, new_center);
                    for i in 1..4 {
                        if mask == (1 << i) {
                            new_children[i].segments.push((seg, info));
                            to_delete.insert(segi);
                        }
                    }
                    segi += 1;
                }

                if to_delete.len() > 0 {
                    r.segments =
                        r.segments
                        .iter()
                        .enumerate()
                        .filter(|&(i,_)| !to_delete.contains(&i))
                        .map(|(_,x)| *x)
                        .collect();
                }

                let new_child_info = QTreeChildInfo {
                    children: new_children,
                    center: s.p2
                };
                r.child_info = Some(new_child_info);
            }
        }
    }

    pub fn insert_segments<F>(&mut self, segments: &'a Vec<Segment>, get_info: F) 
    where F: Fn(usize) -> SI
    {
        // Our aim here is to find a good order for segment insertion.
        // Generally, alternately going from the outside in and the inside out
        // works well. So we find the center point and then inversely order segments
        // by |p1|^2 + |p2|^2, where |p| is the distance of a point
        // p from the center.

        let mut avg_x: Scalar = 0.0;
        let mut avg_y: Scalar = 0.0;
        for s in segments {
            avg_x += s.p1.coords[0] + s.p2.coords[0];
            avg_y += s.p1.coords[1] + s.p2.coords[1];
        }
        avg_x /= segments.len() as Scalar;
        avg_y /= segments.len() as Scalar;
        
        let mut sls: Vec<(Scalar, &'a Segment, usize)> = Vec::new();
        let mut ind = 0;
        for s in segments {
            let x1d = s.p1.coords[0] - avg_x;
            let x2d = s.p2.coords[0] - avg_x;
            let y1d = s.p1.coords[1] - avg_y;
            let y2d = s.p2.coords[1] - avg_y;

            sls.push((
                (x1d*x1d + x2d*x2d + y1d*y1d + y2d*y2d),
                &s,
                ind
            ));

            ind += 1;
        }

        sls.sort_by(|&(d1,_,_), &(d2,_,_)| d2.partial_cmp(&d1).unwrap());

        let mut si = 0;
        let mut ei = sls.len()-1;
        while si < ei {
            self.insert_segment(sls[si].1, get_info(sls[si].2));
            self.insert_segment(sls[ei].1, get_info(sls[ei].2));
            si += 1;
            ei -= 1;
        }
        if si == ei {
            self.insert_segment(sls[si].1, get_info(sls[si].2));
        }
    }

    pub fn get_segments_possibly_touched_by_ray(&'a self, ray: &Ray) -> Vec<(&'a Segment, SI)> {
        return self.ray_trace_iter(ray).flat_map(|x| x.iter()).map(|x| *x).collect();
    }

    pub fn get_segments_touched_by_ray(&'a self, ray: &Ray)
    -> Option <(Vec<(&'a Segment, SI)>, Point2, Scalar)> {
        let segs = self.get_segments_possibly_touched_by_ray(ray);

        let mut intersects: Vec<(&'a Segment, SI, Point2, Scalar)> = Vec::new();
        for (s,si) in segs {
            if let Some(pt) = ray_intersects_segment(ray, s) {
                let xd = pt.coords[0] - ray.p1.coords[0];
                let yd = pt.coords[1] - ray.p1.coords[1];
                let d = xd*xd + yd*yd;
                intersects.push((s, si, pt, d));
            }
        }

        if intersects.len() == 0
            { return None; }
        
        // Find the intersects closest to the start of the ray.
        intersects.sort_by(|&(_,_,_,d1),&(_,_,_,d2)| {
            d1.partial_cmp(&d2).unwrap()
        });

        // Skip any initial zero intercepts.
        let it = intersects.iter().skip_while(|&&(_,_,_,d)| d - EPSILON < 0.0);
        
        let mut last_d: Scalar = 0.0;
        let mut last_pt: Point2 = Point2::new(0.0, 0.0);
        let mut rs: Vec<(&'a Segment, SI)> = Vec::new();
        for &(s, si, pt, d) in it {
            if last_d == 0.0 || d == last_d {
                last_d = d;
                last_pt = pt;
                rs.push((s, si));
            }
            else {
                break;
            }
        }

        if rs.len() == 0 {
            None
        }
        else {
            let pt = last_pt;
            let d = last_d;
            //println!("INTERSECT {} {} FROM {} {} at {}", pt.coords[0], pt.coords[1], ray.p1.coords[0], ray.p1.coords[1], d);
            Some((rs, pt, d))
        }
    }
}

//
// Surfaces, rays, etc.
//

#[derive(Copy, Clone)]
pub struct RayProperties {
    pub wavelength: Scalar, // um
    pub intensity: Scalar
}

pub struct TracingProperties {
    pub random_seed: [usize; 1],
    // If a new ray is generated with intensity below
    // this threshold, it will be discarded.
    pub intensity_threshold: Scalar
}

#[derive(Debug, Clone)]
pub struct MaterialProperties {
    pub diffuse_reflect_fraction: Scalar,
    pub specular_reflect_fraction: Scalar,
    pub refraction_fraction: Scalar,
    pub attenuation_coeff: Scalar,
    pub cauchy_coeffs: Vec<Scalar>
}

impl MaterialProperties {
    pub fn default() -> MaterialProperties {
        MaterialProperties {
            diffuse_reflect_fraction:  0.5,
            specular_reflect_fraction: 0.5,
            refraction_fraction: 0.0,
            attenuation_coeff: 0.0,
            cauchy_coeffs: vec![ 1.0 ]
        }
    }
}

pub type RayTraceSegmentInfo = usize;

struct TraceRayArgs<'a, R>
where R: Rng + 'a {
    ray: &'a Ray,
    ray_props: &'a RayProperties,
    tp: &'a TracingProperties,
    qtree: &'a QTree<'a, RayTraceSegmentInfo>,
    materials: &'a Vec<MaterialProperties>,
    left_matprops_indices: &'a Vec<u8>,
    right_matprops_indices: &'a Vec<u8>,
    new_rays: &'a mut Vec<(Ray, RayProperties)>,
    rng: &'a mut R
}

pub fn trace_ray<R>(
    ray: &Ray,
    ray_props: &RayProperties,
    tp: &TracingProperties,
    qtree: &QTree<RayTraceSegmentInfo>,
    materials: &Vec<MaterialProperties>,
    left_matprops_indices: &Vec<u8>,
    right_matprops_indices: &Vec<u8>,
    new_rays: &mut Vec<(Ray, RayProperties)>,
    rng: &mut R
)
-> usize
where R: Rng {

    trace_ray_args(&mut TraceRayArgs {
        ray: ray,
        ray_props: ray_props,
        tp: tp,
        qtree: qtree,
        materials: materials,
        left_matprops_indices: left_matprops_indices,
        right_matprops_indices: right_matprops_indices,
        new_rays: new_rays,
        rng: rng
    })
}

fn trace_ray_args<R>(args: &mut TraceRayArgs<R>)
-> usize
where R: Rng { // Returns number of new rays traced.

    let rayline = args.ray.p2 - args.ray.p1;

    let mut num_new_rays = 0;
    if let Some((segs_with_info, intersect, _)) = args.qtree.get_segments_touched_by_ray(args.ray) {
        for (seg, segi) in segs_with_info {
            // Is the ray hitting the left surface or the right surface of
            // the segment?
            let side = point_side_of_line_segment(seg.p1, seg.p2, args.ray.p1);

            // If the ray actually originates on this segment, ignore it.
            if side == 0
                { continue; }
            //println!("SIDE: ({}, {}, {}, {}) segi={} {}", seg.p1.coords[0], seg.p1.coords[1], seg.p2.coords[0], seg.p2.coords[1], segi, side);
            
            let segline = seg.p2 - seg.p1;

            // The left normal (looking "along" the line from the origin.)
            let mut surface_normal = Vector2::new(-segline.data[1], segline.data[0]);

            // Ensure that surface normal is pointing in opposite direction to ray.
            if side == 1 {
                surface_normal = -surface_normal;
            }

            let into_matprops_i;
            let from_matprops_i;
            if side == -1 {
                into_matprops_i = args.right_matprops_indices[segi];
                from_matprops_i = args.left_matprops_indices[segi];
            }
            else {
                into_matprops_i = args.left_matprops_indices[segi];
                from_matprops_i = args.right_matprops_indices[segi];
            }

            let ref into_matprops = args.materials[into_matprops_i as usize];
            let ref from_matprops = args.materials[from_matprops_i as usize];

            // We need to calculate the extent to which the ray's intensity has been attenuated
            // by traveling through the relevant material for whatever distance.
            let distance2 = nalgebra::distance_squared(&intersect, &(args.ray.p1));
            let att = from_matprops.attenuation_coeff * distance2;
            let new_intensity = args.ray_props.intensity - att;

            // Decide whether we're going to do diffuse reflection, specular reflection,
            // or refraction, based on the relative amount of intensity they preserve.
            let tot = into_matprops.diffuse_reflect_fraction + into_matprops.specular_reflect_fraction;
            let rnd = args.rng.next_f64() * tot;
            if rnd < into_matprops.diffuse_reflect_fraction {
                num_new_rays += add_diffuse(args, new_intensity, &segline, &into_matprops, &intersect, &surface_normal);
            }
            else if rnd < into_matprops.diffuse_reflect_fraction + into_matprops.specular_reflect_fraction {
                num_new_rays += add_specular(args, new_intensity, &rayline, &into_matprops, &intersect, &surface_normal);
            }
            else if rnd < into_matprops.diffuse_reflect_fraction + into_matprops.specular_reflect_fraction + into_matprops.refraction_fraction {
                num_new_rays += add_refraction(args, new_intensity, &rayline, &from_matprops, &into_matprops, &intersect, &surface_normal, side);
            }
        }
    }

    num_new_rays
}

fn add_diffuse<R>(
    args: &mut TraceRayArgs<R>,
    new_intensity: Scalar,
    segline: &Vector2,
    matprops: &MaterialProperties,
    intersect: &Point2,
    surface_normal: &Vector2
)
-> usize
where R: Rng {
    let _ = matprops; // Not used currently; suppress compiler warning.

    //print!("DIFFMAT {:?} {:?}", matprops, segline);
    let mut num_new_rays = 0;
            
    // If the intensity of the reflected ray is above the thresholed,
    // then cast it in a randomly chosen direction.
    if new_intensity > args.tp.intensity_threshold {
        num_new_rays += 1;

        let mut new_diffuse_ray_props = *(args.ray_props);
        new_diffuse_ray_props.intensity = new_intensity;
                
        let angle = (args.rng.next_f64() as Scalar) * consts::PI;

        let along_seg = angle.cos();
        let normal_to_seg = angle.sin();
        let new_ray_p2 = intersect + (along_seg * segline) + (normal_to_seg * surface_normal);

        let new_ray = Ray {
            p1: *intersect,
            p2: new_ray_p2
        };

        //println!("NEW RAY {} {} {} {}", intersect.coords[0], intersect.coords[1], new_ray_p2.coords[0], new_ray_p2.coords[1]);

        args.new_rays.push((new_ray, new_diffuse_ray_props));
    }

    num_new_rays
}

fn add_specular<R>(
    args: &mut TraceRayArgs<R>,
    new_intensity: Scalar,
    rayline: &Vector2,
    matprops: &MaterialProperties,
    intersect: &Point2,
    surface_normal: &Vector2
)
-> usize
where R: Rng {
    let _ = matprops; // Not used currently; suppress compiler warning.

    //print!("SPECMAT {:?} {:?}", matprops, surface_normal);
    let mut num_new_rays = 0;
            
    if new_intensity > args.tp.intensity_threshold {
        num_new_rays += 1;

        let mut new_specular_ray_props = *(args.ray_props);
        new_specular_ray_props.intensity = new_intensity;
        // Get a normalized normal vector and ray vector.
        let surface_normal_n = surface_normal.normalize();
        let ray_n = rayline.normalize();

        let dot = nalgebra::dot(&ray_n, &surface_normal_n);
        let reflection = ray_n  -((2.0 * dot) * surface_normal_n);

        let new_ray = Ray {
            p1: *intersect,
            p2: intersect + reflection
        };

        args.new_rays.push((new_ray, new_specular_ray_props));
    }

    num_new_rays
}

fn add_refraction<R>(
    args: &mut TraceRayArgs<R>,
    new_intensity: Scalar,
    rayline: &Vector2,
    from_matprops: &MaterialProperties,
    into_matprops: &MaterialProperties,
    intersect: &Point2,
    surface_normal: &Vector2,
    side: i32
)
-> usize
where R: Rng {
    assert!(side != 0);
    assert!(from_matprops.cauchy_coeffs.len() > 0);
    assert!(into_matprops.cauchy_coeffs.len() > 0);

    let mut num_new_rays = 0;

    if new_intensity > args.tp.intensity_threshold {
        num_new_rays += 1;

        // Calculate the refractive index for each material given
        // the wavelength and the material properties.
        let mut from_ri = from_matprops.cauchy_coeffs[0];
        let mut pow: i32 = 2;
        for c in from_matprops.cauchy_coeffs.iter().skip(1) {
            from_ri += c / args.ray_props.wavelength.powi(pow);
            pow += 2;
        }
        let mut into_ri = into_matprops.cauchy_coeffs[0];
        for c in into_matprops.cauchy_coeffs.iter().skip(1) {
            into_ri += c / args.ray_props.wavelength.powi(pow);
            pow += 2;
        }

        let ri = from_ri / into_ri;

        let nsn = surface_normal.normalize();
        let rayline = rayline.normalize();
        let n_1 = -nsn;
        let c = nalgebra::dot(&n_1, &rayline);  
        assert!(c >= 0.0);

        let vrefract =
            (ri * rayline) +
            (((ri * c) -
              (1.0 - ri*ri*(1.0 - c*c)).sqrt())
             *nsn);
    
        let mut new_refracted_ray_props = *(args.ray_props);
        new_refracted_ray_props.intensity = new_intensity;
        let new_ray = Ray {
            p1: *intersect,
            p2: intersect + vrefract
        };

        args.new_rays.push((new_ray, new_refracted_ray_props));
    }

    num_new_rays
}

pub struct RayTraceState<'a> {
    tracing_properties: &'a TracingProperties,
    qtree: &'a QTree<'a, RayTraceSegmentInfo>,
    materials: &'a Vec<MaterialProperties>,
    left_matprops_indices: &'a Vec<u8>,
    right_matprops_indices: &'a Vec<u8>,
    pub old_rays: &'a mut Vec<(Ray, RayProperties)>,
    pub new_rays: &'a mut Vec<(Ray, RayProperties)>,
    recursion_limit: usize,
    ray_limit: usize,
    ray_count: usize,
    recursion_level: usize,
    rng: StdRng
}

impl<'a> RayTraceState<'a> {
    pub fn initial(
        tp: &'a TracingProperties,
        qtree: &'a QTree<RayTraceSegmentInfo>,
        materials: &'a Vec<MaterialProperties>,
        left_matprops_indices: &'a Vec<u8>,
        right_matprops_indices: &'a Vec<u8>,
        old_rays: &'a mut Vec<(Ray, RayProperties)>,
        new_rays: &'a mut Vec<(Ray, RayProperties)>,
        recursion_limit: usize,
        ray_limit: usize
    ) -> RayTraceState<'a> {
        RayTraceState {
            tracing_properties: tp,
            qtree: qtree,
            materials: materials,
            left_matprops_indices: left_matprops_indices,
            right_matprops_indices: right_matprops_indices,
            old_rays: old_rays,
            new_rays: new_rays,
            recursion_limit: recursion_limit,
            ray_limit: ray_limit,
            ray_count: 0,
            recursion_level: 0,
            rng: SeedableRng::from_seed(&(tp.random_seed)[..])
        }
    }

    pub fn get_rays(&'a self) -> &'a Vec<(Ray, RayProperties)> {
        assert!(self.old_rays.len() == 0 || self.new_rays.len() == 0);
        if self.old_rays.len() == 0 { self.new_rays } else { self.old_rays }
    }
}

pub fn ray_trace_step(st: &mut RayTraceState) -> bool {
    if (st.ray_limit != 0 && st.ray_count >= st.ray_limit) ||
       (st.recursion_limit != 0 && st.recursion_level >= st.recursion_limit) ||
       (st.old_rays.len() == 0) {
        return true;
    }

    for &(ref ray, ref ray_props) in st.old_rays.iter() {
        let n_new_rays = trace_ray(
            ray, ray_props,
            st.tracing_properties,
            st.qtree,
            st.materials,
            st.left_matprops_indices,
            st.right_matprops_indices,
            st.new_rays,
            &mut st.rng
        );
        st.ray_count += n_new_rays;
    }
    st.old_rays.clear();
    mem::swap(&mut (st.old_rays), &mut (st.new_rays));
    st.recursion_level += 1;

    false
}
