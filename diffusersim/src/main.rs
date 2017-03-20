extern crate nalgebra;
extern crate piston_window;
#[macro_use]

pub mod geom;
pub mod geom_import;
pub mod render;

use piston_window::*;
use geom as g;
use geom_import as gi;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn do_graphics<T>(
    qtree: &g::QTree<T>,
    segments: &Vec<g::Segment>,
    rays: &Vec<(g::Ray,g::RayProperties)>,
    touched: &Vec<g::Segment>) {

    let t = render::get_display_transform(segments, WIDTH, HEIGHT);

    let mut window: PistonWindow =
        WindowSettings::new("Hello Piston!", [WIDTH, HEIGHT])
        .exit_on_esc(true).build().unwrap();
    while let Some(e) = window.next() {
        //let lines = r::render_segments(segments, WIDTH, HEIGHT);

        window.draw_2d(&e, |_, g| { clear([1.0; 4], g); });
        render::render_segments(segments, &mut window, &e, &t, [0.0,1.0,0.0,1.0]);
        render::render_rays(rays, &mut window, &e, &t);
        render::render_qtree(qtree, &mut window, &e, &t);
        render::render_segments(touched, &mut window, &e, &t, [1.0,0.25,0.0,1.0])
    }
}

#[allow(dead_code)]
fn test1() {
    match gi::parse_geometry_file("src/test.geom") {
        Err(e) => {
            println!("Error: {:?}", e);
        },
        Ok(p) => {
            match p {
                Err(e) => {
                    println!("Error: {:?}", e);
                }
                Ok(geom) => {
                    println!("{:#?}", geom);

                    let bare_rays = vec![
                        //g::ray(-30.0, -3.0, 30.0, -7.0),
                        //g::ray(-10.0, 30.0, -10.0, -30.0)
                        g::ray(0.5, 0.0, 3.0, 2.0),
                        //g::ray(0.0, 0.0, -1.0, 0.0),
                        //g::ray(0.0, 0.0, 0.0, 1.0)
                        //g::ray(18.0, 10.0, 16.0, -5.0)
                    ];
                    let mut rays: Vec<(g::Ray, g::RayProperties)> = bare_rays.into_iter().map(|r| {
                        (r, g::RayProperties { wavelength: 0.0, intensity: 1.0 })
                    }).collect();

                    let inf = g::MaterialProperties::default();

                    let mut qtree: g::QTree<g::MaterialProperties> = g::QTree::make_empty_qtree();
                    qtree.insert_segments(&geom.segments, |_| &inf);
                    println!("QTREE: {:#?}", qtree);

                    let mut new_rays: Vec<(g::Ray, g::RayProperties)> = Vec::new();
                    g::recursive_trace_ray(
                        &g::TracingProperties {
                            new_rays: 16,
                            intensity_threshold: 0.01
                        },
                        &qtree,
                        &mut rays,
                        &mut new_rays,
                        100
                    );
                    rays.extend(new_rays.into_iter());

                    println!("DOING GRAPHICS!");
                    do_graphics(
                        &qtree,
                        &geom.segments,
                        &rays,
                        &Vec::new()/*&touched*/
                    );   
                }
            }
        }
    }
}

#[allow(dead_code)]
fn test2() {
    let mut test_segments: Vec<g::Segment> = Vec::new();
    for i in 1..100 {
        let v = i as f64;
        test_segments.push(g::seg(-2.0*v, v, -v, 2.0*v));
        test_segments.push(g::seg(2.0*v, -v, v, -2.0*v));
    }

    let bare_rays = vec![
        g::ray(-1.0, -1.0, 1.0, -1.0),
        g::ray(10.0, -40.0, 20.0, 50.0),
        g::ray(-100.0, 100.0, -120.0, 200.0),
        g::ray(100.0, -100.0, 120.0, -200.0)
    ];
    let rays: Vec<(g::Ray, g::RayProperties)> = bare_rays.into_iter().map(|r| {
        (r, g::RayProperties { wavelength: 0.0, intensity: 0.0 })
    }).collect();

    let inf = g::MaterialProperties::default();
    let mut qtree: g::QTree<g::MaterialProperties> = g::QTree::make_empty_qtree();
    qtree.insert_segments(&test_segments, |_| &inf);

    println!("N NODES: {} {}", qtree.get_n_nodes(), qtree.get_n_nonempty_nodes());

    let mut segs: Vec<&g::Segment> = Vec::new();
    for r in &rays {
        if let Some((sts, _, _)) = qtree.get_segments_touched_by_ray(&r.0) {
            segs.extend(sts.iter().map(|&(s,_)| s));
        }
    }

    let touched: Vec<g::Segment> = segs.into_iter().map(|x| x.clone()).collect();
    println!("COMPUTED TOUCHES");
    do_graphics(&qtree, &test_segments, &rays, &touched);
}

fn main() {
    test1();
}