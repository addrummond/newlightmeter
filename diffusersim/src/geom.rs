use std::fmt;
use std::rc::Rc;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::iter;

//
// Basic types.
//

pub type Scalar = f64;
use nalgebra::Vector2 as Vector2_;
use nalgebra::Point2 as Point2_;
pub type Vector2 = Vector2_<Scalar>;
pub type Point2 = Point2_<Scalar>;

//
// QTrees
//

#[derive(Clone)]
pub struct Segment {
    // p1.x < p2.x || (p1.x == p2.x && p1.y < p2.y)
    pub p1: Point2,
    pub p2: Point2
}

pub struct Ray {
    // Origin at p1, pointing in direction of p2.
    pub p1: Point2,
    pub p2: Point2
}

impl fmt::Debug for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(Segment ({}, {}) -> ({}, {}))",
               self.p1.coords[0], self.p1.coords[1],
               self.p2.coords[0], self.p2.coords[1])
    }
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
pub struct QTreeChildInfo<'a,SegmentInfo>
where SegmentInfo: 'a {
    pub center: Point2,
    pub children: [Box<QTreeNode<'a,SegmentInfo>>; 4] // Clockwise from NW
}

#[derive(Debug)]
pub struct QTreeNode<'a, SegmentInfo>
where SegmentInfo: 'a {
    pub segments: Vec<(&'a Segment, &'a SegmentInfo)>,
    pub child_info: Option<QTreeChildInfo<'a,SegmentInfo>>
}

pub struct QTree<'a, SegmentInfo>
where SegmentInfo: 'a {
    root: Box<QTreeNode<'a, SegmentInfo>>,
    n_nodes: usize,
    n_nonempty_nodes: usize
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

pub struct QTreeInOrderIterator<'a, SegmentInfo>
where SegmentInfo: 'a {
    stack: Vec<(usize, &'a QTreeNode<'a,SegmentInfo>)>,
}

impl<'a, SegmentInfo> Iterator for QTreeInOrderIterator<'a, SegmentInfo> {
    type Item = (usize, &'a QTreeNode<'a,SegmentInfo>);

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
    let ray_slope_num = ray.p2.coords[1] - ray.p1.coords[1];
    let seg_slope_num = segment.p2.coords[1] - segment.p2.coords[1];

    if (ray_slope_num == 0.0 && seg_slope_num == 0.0) {
        // Parallel horizontal lines.
        // Does not count as an intersection for purposes of ray tracing,
        // even if the lines overlap.
        return None;
    }

    let ray_slope_denom = ray.p2.coords[0] - ray.p1.coords[0];
    let seg_slope_denom = segment.p2.coords[0] - segment.p1.coords[0];
    let ray_slope = ray_slope_num / ray_slope_denom;
    let seg_slope = seg_slope_num / seg_slope_denom;

    if (ray_slope == seg_slope) {
        // Parallel lines.
        // Does not count as an intersection for purposes of ray tracing,
        // even if the lines overlap.
        return None;
    }

    let ray_k = ray.p1.coords[1] - (ray_slope * ray.p1.coords[0]);
    let seg_k = segment.p1.coords[1] - (seg_slope * segment.p1.coords[0]);

    // Calculate intersection point.
    let x = (seg_k - ray_k) / (ray_slope - seg_slope);
    let y = (seg_k*ray_slope - ray_k-seg_slope) / (ray_slope - seg_slope);

    // Is the intersection point on the ray?
    if ray_slope_num > 0.0 && y < ray.p1.coords[1]
        { return None }
    else if ray_slope_num < 0.0 && y > ray.p1.coords[1]
        { return None }
    if ray_slope_denom > 0.0 && x < ray.p1.coords[0]
        { return None }
    else if ray_slope_denom < 0.0 && x > ray.p1.coords[0]
        { return None }
        
    // It's on the ray. Is it on the segment?
    if x >= segment.p1.coords[0] && x <= segment.p2.coords[0] &&
       y >= segment.p1.coords[1] && y <= segment.p2.coords[1] {
        return Some(Point2::new(x, y));
    }
    else {
        return None;
    }
}

pub struct QTreeRayTraceIterator<'a, 'b, SegmentInfo>
where SegmentInfo: 'a {
    ray: &'b Ray,
    ray_m: Scalar,
    ray_k: Scalar,
    stack: Vec<(bool, &'a QTreeNode<'a,SegmentInfo>)>,
    backlog: VecDeque<(&'a Segment, &'a SegmentInfo)>,
}

impl<'a,'b, SegmentInfo> Iterator for QTreeRayTraceIterator<'a, 'b, SegmentInfo> {
    type Item = (&'a Segment, &'a SegmentInfo);

    fn next(&mut self) -> Option<(&'a Segment, &'a SegmentInfo)> {
        loop {
            if self.backlog.len() > 0
                { return self.backlog.pop_front(); }
            
            if (self.stack.len() == 0)
                { return None; }

            let &(already, r) = self.stack.last().unwrap();

            if !already {
                self.backlog.extend(r.segments.iter().skip(1));
                let l = self.stack.len();
                self.stack[l-1].0 = true;
                if (r.segments.len() > 0) {
                    return Some(r.segments[0]);
                }
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

impl<'a, SegmentInfo> QTree<'a, SegmentInfo> {
    pub fn make_empty_qtree() -> QTree<'a,SegmentInfo>
    {
        let root = QTreeNode {
            segments: vec! [],
            child_info: None
        };
        return QTree {
            root: Box::new(root),
            n_nodes: 0,
            n_nonempty_nodes: 0
        };
    }

    pub fn get_n_nodes(&self) -> usize { self.n_nodes }
    pub fn get_n_nonempty_nodes(&self) -> usize { self.n_nonempty_nodes }

    pub fn in_order_iter(&self) -> QTreeInOrderIterator<SegmentInfo> {
        QTreeInOrderIterator { stack: vec![(0, &*self.root)] }
    }

    pub fn ray_trace_iter<'b>(&'a self, ray: &'b Ray) -> QTreeRayTraceIterator<'a,'b,SegmentInfo> {
        let m = (ray.p2.coords[1] - ray.p1.coords[1]) /
                (ray.p2.coords[0] - ray.p1.coords[0]);
        let k = ray.p2.coords[1] - (m * ray.p1.coords[0]);

        QTreeRayTraceIterator {
            ray: ray,
            ray_m: m,
            ray_k: k,
            stack: vec![(false, &*self.root)],
            backlog: VecDeque::new()
        }
    }

    pub fn insert_segment(&mut self, s: &'a Segment, info: &'a SegmentInfo)
    {
        let mut stack : Vec<&mut QTreeNode<'a,SegmentInfo>> = Vec::new();
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
                if r.segments.len() == 1 {
                    self.n_nonempty_nodes += 1;
                }
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
                            if (new_children[i].segments.len() == 1) {
                                self.n_nonempty_nodes += 1;
                            }
                        }
                    }
                    segi += 1;
                }

                if to_delete.len() > 0 {
                    r.segments =
                        r.segments
                        .iter()
                        .enumerate()
                        .filter(|&(i,x)| !to_delete.contains(&i))
                        .map(|(i,x)| *x)
                        .collect();
                }

                let new_child_info = QTreeChildInfo {
                    children: new_children,
                    center: s.p2
                };
                r.child_info = Some(new_child_info);
                self.n_nodes += 4;
            }
        }
    }

    pub fn insert_segments<F>(&mut self, segments: &'a Vec<Segment>, get_info: F) 
    where F: Fn(usize) -> &'a SegmentInfo
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
        
        let mut sls: Vec<(Scalar, &'a Segment)> = Vec::new();
        for s in segments {
            let x1d = s.p1.coords[0] - avg_x;
            let x2d = s.p2.coords[0] - avg_x;
            let y1d = s.p1.coords[1] - avg_y;
            let y2d = s.p2.coords[1] - avg_y;

            sls.push((
                (x1d*x1d + x2d*x2d + y1d*y1d + y2d*y2d),
                &s
            ));
        }

        sls.sort_by(|&(d1,_), &(d2,_)| d2.partial_cmp(&d1).unwrap());

        let mut si = 0;
        let mut ei = sls.len()-1;
        while (si < ei) {
            self.insert_segment(sls[si].1, get_info(si));
            self.insert_segment(sls[ei].1, get_info(ei));
            si += 1;
            ei -= 1;
        }
        if (si == ei) {
            self.insert_segment(sls[si].1, get_info(si));
        }
    }

    pub fn get_segments_possibly_touched_by_ray(&'a self, ray: &Ray) -> Vec<(&'a Segment, &'a SegmentInfo)> {
        return self.ray_trace_iter(ray).collect();
    }
}

//
// Surfaces, rays, etc.
//

struct RayProperties {
    wavelength: Scalar, // um
    intensity: Scalar
}

// A surface that does something to rays that land on it,
// rather than just recording their intensity/wavelength/etc.
struct ActiveSurfaceProperties {
    refractive_index: Scalar,
    extinction: Scalar,
    cauchy_coeffs: Vec<Scalar>
}

enum SurfaceType {
    Active(ActiveSurfaceProperties),
    Passive
}

fn trace_ray(ray: Ray, ray_props: RayProperties, qtree: QTree<SurfaceType>) {
    
}