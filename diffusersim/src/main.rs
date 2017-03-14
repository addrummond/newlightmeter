extern crate nalgebra;
use std::fmt;

type Scalar = f64;
use nalgebra::Vector2 as Vector2_;
use nalgebra::Point2 as Point2_;
type Vector2 = Vector2_<Scalar>;
type Point2 = Point2_<Scalar>;

enum SegmentInfo {
    NoInfo,
    Info {
        opacity: Scalar
    }
}

struct Segment {
    // p1.x < p2.x || (p1.x == p2.x && p1.y < p2.y)
    p1: Point2,
    p2: Point2,
    info: Box<SegmentInfo>
}

impl fmt::Debug for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(Segment ({}, {}) -> ({}, {}))",
               self.p1.coords[0], self.p1.coords[1],
               self.p2.coords[0], self.p2.coords[1])
    }
}

fn seg(x1: Scalar, y1: Scalar, x2: Scalar, y2: Scalar) -> Segment {
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

    return Segment {
        p1: p1,
        p2: p2,
        info: Box::new(SegmentInfo::NoInfo)
    };
}

const QTREE_BIN_SIZE : usize = 8;

struct QTreeChildInfo<'a> {
    center: Point2,
    children: [Box<QTreeNode<'a>>; 4] // Clockwise from NW
}

struct QTreeNode<'a> {
    segments: Vec<&'a Segment>,
    child_info: Option<QTreeChildInfo<'a>>
}

struct QTree<'a> {
    root: Box<QTreeNode<'a>>,
    n_nodes: usize,
    n_nonempty_nodes: usize
}

fn get_point_quad(p: Point2, c: Point2) -> i32 {
    if p.coords[0] <= c.coords[0] {
        if p.coords[1] <= c.coords[1] {
            return 0;
        }
        else {
            return 3;
        }
    }
    else {
        if p.coords[1] <= c.coords[1] {
            return 1;
        }
        else {
            return 2;
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

impl<'a> QTree<'a> {
    pub fn make_empty_qtree() -> QTree<'a>
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

    fn insert_segment(&mut self, s: &'a Segment)
    {
        let mut stack : Vec<&mut Box<QTreeNode<'a>>> = Vec::new();
        stack.push(&mut self.root);

        while let Some(r) = stack.pop() {
            if r.segments.len() < QTREE_BIN_SIZE {
                if r.segments.len() == 0 {
                    self.n_nonempty_nodes += 1;
                }
                r.segments.push(s);
            }
            else if r.child_info.is_some() {
                if let Some(child_info) = r.child_info.as_mut() {
                    let mask = get_segment_quad_mask(s, child_info.center);

                    let mut i = 1;
                    for child in child_info.children.as_mut() {
                        if mask & i != 0 {
                            stack.push(child);
                        }

                        i <<= 1;
                    }
                }
            }
            else {
                // Given the sorting order for the points of a segment,
                // if we choose p2 as our new center point, the segment
                // will either be in NW or in SW.
                let in_nw = s.p1.coords[0] == s.p2.coords[0] ||
                            s.p1.coords[1] <= s.p2.coords[0];

                let new_children = [
                    Box::new(QTreeNode {
                        child_info: None,
                        segments: if in_nw { vec![s] } else { vec![] }
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
                        segments: if !in_nw { vec![s] } else { vec![] }
                    }),
                ];
                let new_child_info = QTreeChildInfo {
                    children: new_children,
                    center: s.p2
                };
                r.child_info = Some(new_child_info);
                self.n_nodes += 4;
            }
        }
    }

    fn get_segments_possibly_touched_by_ray(&'a self, ray: Segment) -> Vec<&'a Segment>
    {
        let mut segments : Vec<&'a Segment> = Vec::new();
        let mut stack : Vec<&Box<QTreeNode<'a>>> = Vec::new();

        let slope = (ray.p2.coords[1] - ray.p2.coords[1]) /
                    (ray.p2.coords[0] - ray.p2.coords[0]);
        let k = ray.p2.coords[1] - (slope * ray.p1.coords[0]);

        stack.push(&self.root);

        loop {
            match stack.pop() {
                None => { break; }
                Some(r) => {
                    segments.extend(r.segments.iter());

                    if let Some(ref child_info) = r.child_info {
                        // The ray starts from p1, so at least the quad
                        // that q1 is in should be added to the mask.
                        let ref center = child_info.center;
                        let mut quad_mask = get_point_quad(ray.p1, *center);

                        let y_intercept = (slope * center.coords[0]) + k;
                        let x_intercept = (center.coords[1] - k) / slope;

                        let ray_x_direction = (ray.p2.coords[0] - ray.p1.coords[0]) >= 0.0;
                        let ray_y_direction = (ray.p2.coords[1] - ray.p1.coords[1]) >= 0.0;

                        let x_direction_to_y_intercept = -ray.p1.coords[0] >= 0.0;
                        let y_direction_to_x_intercept = -ray.p2.coords[1] >= 0.0;

                        let crosses_x_axis = y_direction_to_x_intercept == ray_y_direction;
                        let crosses_y_axis = x_direction_to_y_intercept == ray_x_direction;

                        if crosses_y_axis {
                            quad_mask |= if y_intercept > 0.0 { 0b0011 } else { 0b1100 };
                        }
                        if crosses_x_axis {
                            quad_mask |= if x_intercept > 0.0 { 0b0110 } else { 0b1001 };
                        }
                        
                        for i in 0..4 {
                            if quad_mask & (1 << i) != 0 {
                                stack.push(&(child_info.children[0]));
                            }
                        }
                    }
                }
            }
        }

        return segments;
    }
}

fn main() {
    let mut test_segments: Vec<Segment> = Vec::new();
    for i in 1..100 {
        let v = i as f64;
        test_segments.push(seg(-2.0*v, v, -v, 2.0*v));
        test_segments.push(seg(2.0*v, -v, v, -2.0*v));
    }

    let mut qtree = QTree::make_empty_qtree();
    for seg in &test_segments {        
        qtree.insert_segment(seg);
    }

    println!("N NODES: {} {}", qtree.n_nodes, qtree.n_nonempty_nodes);

    let segs = qtree.get_segments_possibly_touched_by_ray(seg(-1.0, -1.0, 1.0, -1.0));

    println!("{:?}", segs);
}