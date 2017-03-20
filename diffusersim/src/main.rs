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
                    let tracing_props = g::TracingProperties {
                        new_rays: 16,
                        intensity_threshold: 0.01
                    };

                    let mut st = g::RayTraceState::initial(
                        &tracing_props,
                        &qtree,
                        &mut rays,
                        &mut new_rays,
                        10,
                        0,
                    );
      
                    let mut figs: Vec<simplesvg::Fig> = Vec::new();
                    let mut count = 0;
                    while !g::ray_trace_step(&mut st) {
                        let t = render::get_display_transform(&geom.segments, WIDTH, HEIGHT, 0.05, 0.0, (count*HEIGHT) as g::Scalar);
                        figs.push(render::render_segments(&geom.segments, &t, [0.0, 1.0, 0.0]));
                        figs.push(render::render_rays(st.get_rays(), &t, [1.0, 0.0, 0.0]));
                        count += 1;
                    }

                    let svg = simplesvg::Svg(figs, WIDTH, (count*HEIGHT));
                    spit_out_svg(&svg);
                }
            }
        }
    }
}

fn main() {
    test1();
}