extern crate nalgebra;
extern crate simplesvg;
extern crate rand;
#[macro_use]

pub mod geom;
pub mod geom_import;
pub mod trace;
pub mod render;

use std::fs::File;
use std::io::{Write, BufWriter};
use std::env;

use geom as g;
use trace as t;
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
                    //println!("{:#?}", geom);

                    let mut rays: Vec<(g::Ray, t::LightProperties)> = Vec::new();
                    for b in &geom.beams {
                        match *b {
                            gi::Beam::Collimated { from, to, shiny_side_is_left, n_rays, light_properties } => {
                                let it = g::CollimatedBeamRayIterator::new(from, to, shiny_side_is_left, n_rays);
                                for (p1, p2) in it {
                                    let new_ray = g::Ray {
                                        p1: p1,
                                        p2: p2
                                    };
                                    rays.push((new_ray, light_properties));
                                }
                            },
                            gi::Beam::Ray { from, to, light_properties } => {
                                rays.push((g::Ray { p1: from, p2: to }, light_properties));
                            }
                        }
                    }

                    let mut qtree: g::QTree<t::RayTraceSegmentInfo> = g::QTree::make_empty_qtree();
                    qtree.insert_segments(&geom.segments, |i| i);

                    let mut new_rays: Vec<(g::Ray, t::LightProperties)> = Vec::new();
                    let tracing_props = t::TracingProperties {
                        random_seed: [1],
                        intensity_threshold: 0.01
                    };

                    let rt_init = t::RayTraceInitArgs {
                        tracing_properties: &tracing_props,
                        qtree: &qtree,
                        segment_names: &geom.segment_names,
                        materials: &geom.materials,
                        left_material_properties: &geom.left_material_properties,
                        right_material_properties: &geom.right_material_properties,
                        recursion_limit: 16,
                        ray_limit: 0
                    };
                    let mut st = t::RayTraceState::initial(&rt_init);

                    let mut rayb = t::RayBuffer {
                        old_rays: &mut rays,
                        new_rays: &mut new_rays,
                    };
      
                    let mut figs: Vec<simplesvg::Fig> = Vec::new();
                    let mut count = 0;
                    loop {
                        let t = render::get_display_transform(
                            &geom.segments,
                            &(rayb.old_rays),
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
                        figs.push(render::render_rays(rayb.get_rays(), &t, [1.0, 0.0, 0.0]));
                        count += 1;
                        let finished = t::ray_trace_step(&mut st, &mut rayb, |e: &t::Event| -> () { 
                            match *e {
                                t::Event::Hit { ref segment_index, ref segment_name, ref point } => {
                                    println!("HIT {} {} ({}, {})", segment_index, segment_name, point.coords[0], point.coords[1]);
                                }
                            }
                        });
                        if finished
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