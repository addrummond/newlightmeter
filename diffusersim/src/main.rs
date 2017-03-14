extern crate nalgebra;

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;

mod geom;

use std::fmt;
use geom as g;
use gfx::Device;

type ColorFormat = gfx::format::Rgba8;
type DepthFormat = gfx::format::DepthStencil;

fn do_graphics() {
    let builder = glutin::WindowBuilder::new()
        .with_title("Triangle example".to_string())
        .with_dimensions(1024, 768)
        .with_vsync();
    let (window, mut device, mut factory, main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
    'main: loop {
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                glutin::Event::Closed => break 'main,
                _ => {}
            }
        }
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}

fn main() {
    let mut test_segments: Vec<g::Segment> = Vec::new();
    for i in 1..100 {
        let v = i as f64;
        test_segments.push(g::seg(-2.0*v, v, -v, 2.0*v));
        test_segments.push(g::seg(2.0*v, -v, v, -2.0*v));
    }

    let mut qtree = g::QTree::make_empty_qtree();
    for seg in &test_segments {        
        qtree.insert_segment(seg);
    }

    println!("N NODES: {} {}", qtree.get_n_nodes(), qtree.get_n_nonempty_nodes());

    let segs = qtree.get_segments_possibly_touched_by_ray(g::seg(-1.0, -1.0, 1.0, -1.0));

    println!("{:?}", segs);

    do_graphics();
}