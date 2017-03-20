extern crate nalgebra;
extern crate simplesvg;
#[macro_use]

pub mod geom;
pub mod geom_import;
pub mod render;

use std::fs::File;
use std::io::{Write, BufWriter};

use geom as g;
use geom_import as gi;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn do_graphics<T>(
    qtree: &g::QTree<T>,
    segments: &Vec<g::Segment>,
    rays: &Vec<(g::Ray,g::RayProperties)>,
    touched: &Vec<g::Segment>)
-> simplesvg::Svg {

    let t = render::get_display_transform(segments, WIDTH, HEIGHT, 0.0, 0.0);

    let fseg = render::render_segments(segments, &t, [0.0, 1.0, 0.0]);
    let frays = render::render_rays(rays, &t, [1.0, 0.0, 0.0]);

    simplesvg::Svg(vec![ fseg, frays ], WIDTH, HEIGHT)
    //simplesvg::Fig::Multiple(vec![ fseg, frays ])
/*

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
    }*/
}

fn spit_out_svg(svg: &simplesvg::Svg) {
    let f = File::create("./foo.svg").expect("Unable to open figure file");
    let mut f = BufWriter::new(f);
    f.write_all(svg.to_string().as_bytes()).expect("Unable to write figure file");
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
                        1,
                        100
                    );
                    rays.extend(new_rays.into_iter());

                    println!("DOING GRAPHICS!");
                    let svg = do_graphics(
                        &qtree,
                        &geom.segments,
                        &rays,
                        &Vec::new()/*&touched*/
                    );

                    spit_out_svg(&svg);
                }
            }
        }
    }
}

fn main() {
    test1();
}