extern crate nalgebra;
extern crate piston_window;

mod geom;
mod render;

use piston_window::*;
use geom as g;
use render as r;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn do_graphics(qtree: &g::QTree, segments: &Vec<g::Segment>, rays: &Vec<g::Ray>, touched: &Vec<g::Segment>) {
    let t = render::get_display_transform(segments, WIDTH, HEIGHT);

    let mut window: PistonWindow =
        WindowSettings::new("Hello Piston!", [WIDTH, HEIGHT])
        .exit_on_esc(true).build().unwrap();
    while let Some(e) = window.next() {
        //let lines = r::render_segments(segments, WIDTH, HEIGHT);

        window.draw_2d(&e, |c, g| { clear([1.0; 4], g); });
        render::render_segments(segments, &mut window, &e, &t, [0.0,1.0,0.0,1.0]);
        render::render_rays(rays, &mut window, &e, &t);
        render::render_qtree(qtree, &mut window, &e, &t);
        render::render_segments(touched, &mut window, &e, &t, [1.0,0.0,0.0,1.0])
    }
}

fn main() {
    let mut test_segments: Vec<g::Segment> = Vec::new();
    for i in 1..100 {
        let v = i as f64;
        test_segments.push(g::seg(-2.0*v, v, -v, 2.0*v));
        test_segments.push(g::seg(2.0*v, -v, v, -2.0*v));
    }

    let rays = vec![
        g::ray(-1.0, -1.0, 1.0, -1.0),
        g::ray(10.0, -40.0, 20.0, 50.0),
        //g::ray(-100.0, 100.0, -120.0, 200.0),
        //g::ray(100.0, -100.0, 120.0, -200.0)
    ];

    let mut qtree = g::QTree::make_empty_qtree();
    qtree.insert_segments(&test_segments);
    //for seg in &test_segments {        
    //    qtree.insert_segment(seg);
    //}   

    println!("N NODES: {} {}", qtree.get_n_nodes(), qtree.get_n_nonempty_nodes());

    let mut segs: Vec<&g::Segment> = Vec::new();
    for r in &rays {
        segs.extend(qtree.get_segments_possibly_touched_by_ray(r).iter());
    }
    let touched: Vec<g::Segment> = segs.iter().map(|x| (*x).clone()).collect();

    println!("COMPUTED TOUCHES");

    do_graphics(&qtree, &test_segments, &rays, &touched);
}