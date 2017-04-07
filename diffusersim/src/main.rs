extern crate nalgebra;
extern crate simplesvg;
extern crate rand;
#[macro_use]

pub mod geom;
pub mod geom_import;
pub mod render;

use std::fs::File;
use std::io::{Write, BufWriter};
use std::env;

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
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Bad usage");
        return;
    }
    let geom_filename = args[1].clone();

    match gi::parse_geometry_file(geom_filename.as_str()) {
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

                    let mut rays: Vec<(g::Ray, g::RayProperties)> = Vec::new();
                    for b in &geom.beams {
                        let gi::Beam::Collimated { from, to, shiny_side_is_left, n_rays, wavelength, intensity } = *b;
                        let it = g::CollimatedBeamRayIterator::new(from, to, shiny_side_is_left, n_rays);
                        for (p1, p2) in it {
                            let new_ray = g::Ray {
                                p1: p1,
                                p2: p2
                            };
                            let props = g::RayProperties {
                                wavelength: wavelength,
                                intensity: intensity
                            };
                            rays.push((new_ray, props));
                        }
                    }

                    let mut qtree: g::QTree<g::RayTraceSegmentInfo> = g::QTree::make_empty_qtree();
                    qtree.insert_segments(&geom.segments, |i| i);

                    let mut new_rays: Vec<(g::Ray, g::RayProperties)> = Vec::new();
                    let tracing_props = g::TracingProperties {
                        random_seed: [1],
                        intensity_threshold: 0.01
                    };

                    let mut st = g::RayTraceState::initial(
                        &tracing_props,
                        &qtree,
                        &(geom.materials),
                        &(geom.left_material_properties),
                        &(geom.right_material_properties),
                        &mut rays,
                        &mut new_rays,
                        16,
                        0,
                    );
      
                    let mut figs: Vec<simplesvg::Fig> = Vec::new();
                    let mut count = 0;
                    loop {
                        let t = render::get_display_transform(
                            &geom.segments,
                            &(st.old_rays),
                            render::DisplayTransformArgs {
                                width: WIDTH,
                                height: HEIGHT,
                                border: 40,
                                offset_x: 0.0,
                                offset_y: (count * HEIGHT) as g::Scalar,
                                scale_factor: 1.0,
                                keep_aspect_ratio: true
                            }
                        );
                        figs.push(render::render_segments(&geom.segments, &t, [0.0, 1.0, 0.0]));
                        figs.push(render::render_rays(st.get_rays(), &t, [1.0, 0.0, 0.0]));
                        count += 1;
                        if g::ray_trace_step(&mut st)
                            { break; }
                    }

                    let svg = simplesvg::Svg(figs, WIDTH, (count*HEIGHT));
                    println!("SVG OUT");
                    spit_out_svg(&svg);
                }
            }
        }
    }
}

fn main() {
    test1();
}