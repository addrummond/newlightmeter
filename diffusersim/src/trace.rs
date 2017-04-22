use std::f64::consts;
use rand::{Rng, SeedableRng, StdRng};
use std::mem;
use std::collections::HashMap;

use geom::{Ray,Scalar,Point2,Vector2};
use geom as g;
use nalgebra;

#[derive(Copy, Clone, Debug)]
pub struct LightProperties {
    pub wavelength: Scalar, // um
    pub intensity: Scalar
}

pub struct TracingProperties {
    pub random_seed: [usize; 1],
    // If a new ray is generated with intensity below
    // this threshold, it will be discarded.
    pub intensity_threshold: Scalar
}

pub enum Event<'a> {
    Hit {
        segment_index: usize,
        segment_name: &'a str,
        point: Point2
    }
}

pub trait EventHandler<E> where Self: FnMut(&Event) -> Result<(), E> { }
impl <'a,E,F> EventHandler<E> for F where F: FnMut(&Event) -> Result<(), E> { }

#[derive(Debug, Clone)]
pub struct MaterialProperties {
    pub diffuse_reflect_fraction: Scalar,
    pub specular_reflect_fraction: Scalar,
    pub refraction_fraction: Scalar,
    pub attenuation_coeff: Scalar,
    pub cauchy_coeffs: Vec<Scalar>
}

impl MaterialProperties {
    pub fn default() -> MaterialProperties {
        MaterialProperties {
            diffuse_reflect_fraction:  0.5,
            specular_reflect_fraction: 0.5,
            refraction_fraction: 0.0,
            attenuation_coeff: 0.0,
            cauchy_coeffs: vec![ 1.0 ]
        }
    }
}

pub type RayTraceSegmentInfo = usize;

pub struct RayBuffer<'a> {
    pub old_rays: &'a mut Vec<(Ray, LightProperties)>,
    pub new_rays: &'a mut Vec<(Ray, LightProperties)>
}

impl<'a> RayBuffer<'a> {
    pub fn get_rays(&'a self) -> &'a Vec<(Ray, LightProperties)> {
        assert!(self.old_rays.len() == 0 || self.new_rays.len() == 0);
        if self.old_rays.len() == 0 { self.new_rays } else { self.old_rays }
    }
}

pub struct RayTraceInitArgs<'a> {
    pub tracing_properties: &'a TracingProperties,
    pub qtree: &'a g::QTree<'a, RayTraceSegmentInfo>,
    pub segment_names: &'a HashMap<usize, String>,
    pub materials: &'a Vec<MaterialProperties>,
    pub left_material_properties: &'a Vec<u8>,
    pub right_material_properties: &'a Vec<u8>,
    pub recursion_limit: usize,
    pub ray_limit: usize
}

pub struct RayTraceState<'a> {
    args: &'a RayTraceInitArgs<'a>,
    ray_count: usize,
    recursion_level: usize,
    rng: StdRng
}

impl<'a> RayTraceState<'a> {
    pub fn initial(args: &'a RayTraceInitArgs) -> RayTraceState<'a> {
        RayTraceState {
            args: args,
            ray_count: 0,
            recursion_level: 0,
            rng: SeedableRng::from_seed(&(args.tracing_properties.random_seed)[..])
        }
    }
}

struct TraceRayArgs<'a> {
    ray: &'a Ray,
    ray_props: &'a LightProperties,
    new_rays: &'a mut Vec<(Ray,LightProperties)>
}

fn trace_ray<F,E>(st: &mut RayTraceState, args: &mut TraceRayArgs, mut handle_event: &mut F)
-> Result<usize,E> // Returns number of new rays traced
where F: EventHandler<E> {

    let rayline = args.ray.p2 - args.ray.p1;

    let mut num_new_rays = 0;
    if let Some((segs_with_info, intersect, _)) = st.args.qtree.get_segments_touched_by_ray(args.ray) {
        for (seg, segi) in segs_with_info {
            if let Some(ref name) = st.args.segment_names.get(&segi) {
                handle_event(&Event::Hit {
                    segment_index: segi,
                    segment_name: name.as_str(),
                    point: intersect
                })?;
            }

            // Is the ray hitting the left surface or the right surface of
            // the segment?
            let side = g::point_side_of_line_segment(seg.p1, seg.p2, args.ray.p1);

            // If the ray actually originates on this segment, ignore it.
            if side == 0
                { continue; }
            //println!("SIDE: ({}, {}, {}, {}) segi={} {}", seg.p1.coords[0], seg.p1.coords[1], seg.p2.coords[0], seg.p2.coords[1], segi, side);
            
            let segline = seg.p2 - seg.p1;

            // The left normal (looking "along" the line from the origin.)
            let mut surface_normal = Vector2::new(-segline.data[1], segline.data[0]);

            // Ensure that surface normal is pointing in opposite direction to ray.
            if side == 1 {
                surface_normal = -surface_normal;
            }

            let into_matprops_i;
            let from_matprops_i;
            if side == -1 {
                into_matprops_i = st.args.right_material_properties[segi];
                from_matprops_i = st.args.left_material_properties[segi];
            }
            else {
                into_matprops_i = st.args.left_material_properties[segi];
                from_matprops_i = st.args.right_material_properties[segi];
            }

            let ref into_matprops = st.args.materials[into_matprops_i as usize];
            let ref from_matprops = st.args.materials[from_matprops_i as usize];

            // We need to calculate the extent to which the ray's intensity has been attenuated
            // by traveling through the relevant material for whatever distance.
            let distance2 = nalgebra::distance_squared(&intersect, &(args.ray.p1));
            let att = from_matprops.attenuation_coeff * distance2;
            let new_intensity = args.ray_props.intensity - att;

            // Decide whether we're going to do diffuse reflection, specular reflection,
            // or refraction, based on the relative amount of intensity they preserve.
            let tot = into_matprops.diffuse_reflect_fraction + into_matprops.specular_reflect_fraction;
            let rnd = st.rng.next_f64() * tot;
            if rnd < into_matprops.diffuse_reflect_fraction {
                num_new_rays += add_diffuse(st, args, new_intensity, &segline, &into_matprops, &intersect, &surface_normal);
            }
            else if rnd < into_matprops.diffuse_reflect_fraction + into_matprops.specular_reflect_fraction {
                num_new_rays += add_specular(st, args, new_intensity, &rayline, &into_matprops, &intersect, &surface_normal);
            }
            else if rnd < into_matprops.diffuse_reflect_fraction + into_matprops.specular_reflect_fraction + into_matprops.refraction_fraction {
                num_new_rays += add_refraction(st, args, new_intensity, &rayline, &from_matprops, &into_matprops, &intersect, &surface_normal, side);
            }
        }
    }

    Ok(num_new_rays)
}

fn add_diffuse(
    st: &mut RayTraceState,
    args: &mut TraceRayArgs,
    new_intensity: Scalar,
    segline: &Vector2,
    matprops: &MaterialProperties,
    intersect: &Point2,
    surface_normal: &Vector2
)
-> usize
{
    let _ = matprops; // Not used currently; suppress compiler warning.

    //print!("DIFFMAT {:?} {:?}", matprops, segline);
    let mut num_new_rays = 0;
            
    // If the intensity of the reflected ray is above the thresholed,
    // then cast it in a randomly chosen direction.
    if new_intensity > st.args.tracing_properties.intensity_threshold {
        num_new_rays += 1;

        let mut new_diffuse_ray_props = *(args.ray_props);
        new_diffuse_ray_props.intensity = new_intensity;
                
        let angle = (st.rng.next_f64() as Scalar) * consts::PI;

        let along_seg = angle.cos();
        let normal_to_seg = angle.sin();
        let new_ray_p2 = intersect + (along_seg * segline) + (normal_to_seg * surface_normal);

        let new_ray = Ray {
            p1: *intersect,
            p2: new_ray_p2
        };

        //println!("NEW RAY {} {} {} {}", intersect.coords[0], intersect.coords[1], new_ray_p2.coords[0], new_ray_p2.coords[1]);

        args.new_rays.push((new_ray, new_diffuse_ray_props));
    }

    num_new_rays
}

fn add_specular(
    st: &mut RayTraceState,
    args: &mut TraceRayArgs,
    new_intensity: Scalar,
    rayline: &Vector2,
    matprops: &MaterialProperties,
    intersect: &Point2,
    surface_normal: &Vector2
)
-> usize
{
    let _ = matprops; // Not used currently; suppress compiler warning.

    //print!("SPECMAT {:?} {:?}", matprops, surface_normal);
    let mut num_new_rays = 0;
            
    if new_intensity > st.args.tracing_properties.intensity_threshold {
        num_new_rays += 1;

        let mut new_specular_ray_props = *(args.ray_props);
        new_specular_ray_props.intensity = new_intensity;
        // Get a normalized normal vector and ray vector.
        let surface_normal_n = surface_normal.normalize();
        let ray_n = rayline.normalize();

        let dot = nalgebra::dot(&ray_n, &surface_normal_n);
        let reflection = ray_n  -((2.0 * dot) * surface_normal_n);

        let new_ray = Ray {
            p1: *intersect,
            p2: intersect + reflection
        };

        args.new_rays.push((new_ray, new_specular_ray_props));
    }

    num_new_rays
}

fn add_refraction(
    st: &mut RayTraceState,
    args: &mut TraceRayArgs,
    new_intensity: Scalar,
    rayline: &Vector2,
    from_matprops: &MaterialProperties,
    into_matprops: &MaterialProperties,
    intersect: &Point2,
    surface_normal: &Vector2,
    side: i32
)
-> usize
{
    assert!(side != 0);
    assert!(from_matprops.cauchy_coeffs.len() > 0);
    assert!(into_matprops.cauchy_coeffs.len() > 0);

    let mut num_new_rays = 0;

    if new_intensity > st.args.tracing_properties.intensity_threshold {
        num_new_rays += 1;

        // Calculate the refractive index for each material given
        // the wavelength and the material properties.
        let mut from_ri = from_matprops.cauchy_coeffs[0];
        let mut pow: i32 = 2;
        for c in from_matprops.cauchy_coeffs.iter().skip(1) {
            from_ri += c / args.ray_props.wavelength.powi(pow);
            pow += 2;
        }
        let mut into_ri = into_matprops.cauchy_coeffs[0];
        for c in into_matprops.cauchy_coeffs.iter().skip(1) {
            into_ri += c / args.ray_props.wavelength.powi(pow);
            pow += 2;
        }

        let ri = from_ri / into_ri;

        let nsn = surface_normal.normalize();
        let rayline = rayline.normalize();
        let n_1 = -nsn;
        let c = nalgebra::dot(&n_1, &rayline);  
        assert!(c >= 0.0);

        let vrefract =
            (ri * rayline) +
            (((ri * c) -
              (1.0 - ri*ri*(1.0 - c*c)).sqrt())
             *nsn);
    
        let mut new_refracted_ray_props = *(args.ray_props);
        new_refracted_ray_props.intensity = new_intensity;
        let new_ray = Ray {
            p1: *intersect,
            p2: intersect + vrefract
        };

        args.new_rays.push((new_ray, new_refracted_ray_props));
    }

    num_new_rays
}

pub fn ray_trace_step<F,E>(st: &mut RayTraceState, rayb: &mut RayBuffer, mut handle_event: F)
-> Result<bool, E>
where F: EventHandler<E> {
    if (st.args.ray_limit != 0 && st.ray_count >= st.args.ray_limit) ||
       (st.args.recursion_limit != 0 && st.recursion_level >= st.args.recursion_limit) ||
       (rayb.old_rays.len() == 0) {
        return Ok(true);
    }

    for &(ref ray, ref ray_props) in rayb.old_rays.iter() {
        let n_new_rays = trace_ray(
            st,
            &mut TraceRayArgs {
                ray: ray,
                ray_props: ray_props,
                new_rays: rayb.new_rays
            },
            &mut handle_event
        )?;
        st.ray_count += n_new_rays;
    }
    rayb.old_rays.clear();
    mem::swap(&mut (rayb.old_rays), &mut (rayb.new_rays));
    st.recursion_level += 1;

    Ok(false)
}
