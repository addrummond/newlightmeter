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
    pub max_depth: usize,                // 0 if no maximum
    pub max_rays: usize,                 // 0 if no maximum
    pub output_filename: Option<String>, // stdout if not specified
    pub hit_filename: Option<String>,    // hits not written if not specified
    pub geom_filenames: Vec<String>,     // stdin if empty
    pub looping: bool
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
    opts.optopt("o", "output", "set svg file output name", "NAME");
    opts.optopt("h", "hitfile", "set hit file output name", "NAME");
    opts.optopt("d", "maxdepth", "set the maximum recursion depth for tracing", "DEPTH");
    opts.optopt("r", "maxrays", "set the maximum number of rays to trace", "N");
    opts.optflag("l", "loop", "loop when trace ends naturally before maximum number of rays is reached");
    let matches = opts.parse(&args[1..])?;

    let mut max_depth = 0;
    let mut max_rays = 0;
    let mut output_filename: Option<String> = None;
    let mut hit_filename: Option<String> = None;
    let mut looping = false;

    fn parse_uint(v: &str, err: &str) -> Result<usize, CommandLineError> {
        v.parse::<usize>().map_err(|_| {
            CommandLineError::Custom(err.to_string())
        })
    }

    if let Some(v) = matches.opt_str("d") {
        max_depth = parse_uint(v.as_str(), "Argument to 'd' option must be an integer >= 0")?
    }
    if let Some(v) = matches.opt_str("r") {
        max_rays = parse_uint(v.as_str(), "Argument to 'r' option must be an integer >= 0")?
    }
    if let Some(v) = matches.opt_str("o") {
        output_filename = Some(v)
    }
    if let Some(v) = matches.opt_str("h") {
        hit_filename = Some(v)
    }
    
    if matches.opt_present("l") {
        if ! matches.opt_present("r")
            { return Err(CommandLineError::Custom("'l' option can only be used with 'r' option".to_string())) }
        looping = true;
    }

    Ok(RunParams {
        max_depth: max_depth,
        max_rays: max_rays,
        output_filename: output_filename,
        hit_filename: hit_filename,
        geom_filenames: matches.free.clone(),
        looping: looping
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

fn output_svg(svg: &simplesvg::Svg, outname: &Option<String>) -> Result<(), Box<error::Error>> {
    match *outname {
        Some(ref name) => {
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

struct DevSlashNull { }
impl Write for DevSlashNull {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { Ok(buf.len()) }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

fn get_hit_writer(name: &Option<String>) -> Result<Box<Write>, Box<error::Error>> {
    Ok(match *name {
        None => { Box::new(DevSlashNull { }) },
        Some(ref name) => {
            let f = File::create(name)?;
            let f = BufWriter::new(f);
            Box::new(f)
        }
    })
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

    let mut hit_writer = get_hit_writer(&params.hit_filename)?;
    write!(hit_writer, "event_type,segment_index,segment_name,x,y\n")?;

    let starting_rays = beams_to_rays(&geom.beams);

    let mut qtree: g::QTree<t::RayTraceSegmentInfo> = g::QTree::make_empty_qtree();
    qtree.insert_segments(&geom.segments, |i| i);

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
        right_material_properties: &geom.right_material_properties
    };
    let mut st = t::RayTraceState::initial(&rt_init);

    let mut rayb = t::RayBuffer {
        old_rays: &mut starting_rays.clone(),
        new_rays: &mut Vec::new()
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
        let result = t::ray_trace_step(&mut st, &mut rayb, |e: &t::Event| -> Result<(),Box<error::Error>> { 
            match *e {
                t::Event::Hit { ref segment_index, ref segment_name, ref point } => {
                    write!(hit_writer, "ray_hit_segment,{},{},{},{}\n", segment_index, segment_name, point.coords[0], point.coords[1])?;
                    Ok(())
                }
            }
        })?;

        // Is it time to stop tracing?
        if params.max_rays != 0 && result.ray_count >= params.max_rays {
            break;
        }
        if params.max_depth != 0 && result.recursion_level >= params.max_depth {
            break;
        }
        if rayb.get_n_rays() == 0 {
            if params.looping {
                rayb.old_rays.truncate(0); // Hopefully this won't deallocate already-allocated space.
                rayb.old_rays.extend(starting_rays.iter());
                rayb.new_rays.truncate(0);
            }
            else {
                break;
            }
        }
    }

    let svg = simplesvg::Svg(figs, WIDTH, (count*HEIGHT));
    output_svg(&svg, &params.output_filename)
}