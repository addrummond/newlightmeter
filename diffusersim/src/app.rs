use getopts;
use getopts::Options;
use geom as g;
use geom_import as gi;
use trace as t;
use std::io;
use std::io::{Write, BufWriter};
use std::fs::File;
use std::error;
use std::fmt;
use std::num;
use std::fs;
use simplesvg;
use render;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

pub struct RunParams {
    pub max_depth: usize,
    pub output_filename: Option<String>, // stdout if not specified
    pub geom_filenames: Vec<String>    // stdin if empty
}

#[derive(Debug)]
pub enum CommandLineError {
    Custom(String),
    Getopts(getopts::Fail),
    UIntParse(num::ParseIntError)
}
impl From<getopts::Fail> for CommandLineError {
    fn from(err: getopts::Fail) -> CommandLineError {
        CommandLineError::Getopts(err)
    }
}
// Using <usize as str::FromStr>::Err instead of the concrete type
// gives rise to complicated errors and appears not to be worth the bother.
impl From<num::ParseIntError> for CommandLineError {
    fn from(err: num::ParseIntError) -> CommandLineError {
        CommandLineError::UIntParse(err)
    }
}
impl error::Error for CommandLineError {
    fn description(&self) -> &str {
        match *self {
            CommandLineError::Custom(ref s) => { s.as_str() },
            CommandLineError::Getopts(ref e) => { e.description() },
            CommandLineError::UIntParse(ref e) => { e.description() }
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            CommandLineError::Custom(_) => { None },
            CommandLineError::Getopts(ref e) => { Some(e) },
            CommandLineError::UIntParse(ref e) => { Some(e) }
        }
    }
}
impl fmt::Display for CommandLineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CommandLineError::Custom(ref s) => { write!(f, "{}", s) },
            CommandLineError::Getopts(ref e) => { e.fmt(f) },
            CommandLineError::UIntParse(ref e) => { e.fmt(f) }
        }
    }
}

pub fn parse_command_line(args: &Vec<String>) -> Result<RunParams, CommandLineError> {
    let mut opts = Options::new();
    opts.optopt("o", "", "set file output name", "NAME");
    opts.optopt("d", "maxdepth", "set the maximum recursion depth for tracing", "DEPTH");
    let matches = opts.parse(&args[1..])?;

    let mut max_depth = 0;
    let mut output_filename: Option<String> = None;

    //
    // The 'getopts' library is a bit clunky. There is no way to insist that a flag option must
    // have an argument. So after detecting than a flag is present, we must then check whether
    // it has an argument, signaling an error if not, and then set the value of the relevant
    // variable.
    //
    if matches.opt_present("d") {
        match matches.opt_str("d") {
            None => { return Err(CommandLineError::Custom("'d' option must be given an argument".to_string())) },
            Some(v) => { max_depth = v.parse::<usize>()? }
        }
    }
    if matches.opt_present("o") {
        match matches.opt_str("o") {
            None => { return Err(CommandLineError::Custom("'o' option must be given an argument".to_string())) },
            Some(v) => { output_filename = Some(v) }
        }
    }

    Ok(RunParams {
        max_depth: max_depth,
        output_filename: output_filename,
        geom_filenames: matches.free.clone()
    })
}

fn beams_to_rays(beams: &Vec<gi::Beam>) -> Vec<(g::Ray, t::LightProperties)> {
    let mut rays: Vec<(g::Ray, t::LightProperties)> = Vec::new();
    for b in beams {
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
    rays
}

fn spit_out_svg(svg: &simplesvg::Svg, outname: Option<&str>) -> Result<(), Box<error::Error>> {
    match outname {
        Some(name) => {
            let f = File::create(name)?;
            let mut f = BufWriter::new(f);
            f.write_all(svg.to_string().as_bytes())
            .map_err(|x| -> Box<error::Error> { Box::new(x) })
        },
        None => {
            io::stdout().write(svg.to_string().as_bytes())
            .map(|_| ()).map_err(|x| -> Box<error::Error> { Box::new(x) })
        }
    }
}

pub fn do_run(params: &RunParams) -> Result<(), Box<error::Error>> {
    let mut geoms: Vec<gi::ImportedGeometry> = Vec::new();

    if params.geom_filenames.len() == 0 {
        geoms.push(gi::parse_geometry_file(Box::new(io::stdin()))??);
    }
    for filename in &params.geom_filenames {
        let file = Box::new(fs::File::open(filename)?);
        geoms.push(gi::parse_geometry_file(file)??);
    }

    assert!(geoms.len() > 0);
    let geom;
    if geoms.len() == 1 {
        geom = &geoms[0];
    }
    else {
        let (a, b) = geoms.split_at_mut(1);
        gi::combine_imported_geometry(&mut a[0], b)?;
        geom = &a[0];
    }

    let mut rays = beams_to_rays(&geom.beams);

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
    spit_out_svg(&svg, params.output_filename.as_ref().map(|x| x.as_str()))
}