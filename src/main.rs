extern crate nalgebra as na;
extern crate rustracer as rt;
extern crate chrono;
extern crate docopt;
extern crate rustc_serialize;

use std::f32::consts::*;
use std::path::Path;
use std::sync::Arc;
use std::num::ParseIntError;
use na::zero;
use chrono::*;
use docopt::Docopt;

use rt::{Point, Vector, Transform};
use rt::scene::Scene;
use rt::colour::Colourf;
use rt::camera::Camera;
use rt::geometry::*;
use rt::instance::Instance;
use rt::integrator::{Integrator, Whitted, AmbientOcclusion, Normal};
use rt::light::{Light, PointLight, DistantLight};
use rt::material::Material;
use rt::renderer;

const USAGE: &'static str =
    "
Toy Ray-Tracer in Rust

Usage:
  rustracer [options]

Options:
  -h, --help                                  Show this screen.
  -o <file>, --output=<file>                  \
     Output file name [default: image.png].
  -t N, --threads=N                           Number \
     of worker threads to start [default: 8].
  -d <dim>, --dimension=<dim>                 \
     Dimension of the output image [default: 800x600].
  --spp N                                     \
     Samples per pixel [default: 4].
  --block-size=N                              Block size \
     used for rendering [default: 32].
  -i <integrator>, --integrator=<integrator>  Integrator \
     to use [default: whitted].
                                              Valid values: \
     whitted, ao, normal.
  --whitted-max-ray-depth=N                   Maximum ray depth for \
     Whitted integrator. [default: 8].
  --ao-samples=N                              Number of \
     samples for ambient occlusion integrator [default: 16].
";

#[derive(RustcDecodable)]
struct Args {
    flag_output: String,
    flag_threads: usize,
    flag_integrator: IntegratorType,
    flag_whitted_max_ray_depth: u8,
    flag_ao_samples: usize,
    flag_dimension: String,
    flag_spp: usize,
    flag_block_size: usize,
}

#[derive(RustcDecodable)]
enum IntegratorType {
    Whitted,
    Ao,
    Normal,
}

fn main() {
    // Parse args
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.help(true).decode())
        .unwrap_or_else(|e| e.exit());

    let parsed_dims: Result<Vec<usize>, ParseIntError> =
        args.flag_dimension.split('x').map(|s| s.parse::<usize>()).collect();
    let dims = parsed_dims.expect("Invalid dimensions");
    if dims.len() != 2 {
        panic!("Error: invalid dimension specification: {}",
               args.flag_dimension);

    }
    let dim = (dims[0], dims[1]);

    let camera = Camera::new(Point::new(0.0, 4.0, 3.0), dim, 50.0);
    let integrator: Box<Integrator + Send + Sync> = match args.flag_integrator {
        IntegratorType::Whitted => {
            println!("Using Whitted integrator with max ray depth of {}",
                     args.flag_whitted_max_ray_depth);
            Box::new(Whitted::new(args.flag_whitted_max_ray_depth))
        }
        IntegratorType::Ao => {
            println!("Using Ambient Occlusion integrator with {} samples",
                     args.flag_ao_samples);
            Box::new(AmbientOcclusion::new(args.flag_ao_samples))
        }
        IntegratorType::Normal => {
            println!("Using normal facing ratio integrator");
            Box::new(Normal {})
        }
    };

    let mut objs = Vec::new();
    let mut lights: Vec<Box<Light + Send + Sync>> = Vec::new();
    let height = 5.0;

    {
        let mesh = Mesh::load(Path::new("models/bunny.obj"), "bunny");
        objs.push(Instance::new(Box::new(mesh),
                                Material::new(Colourf::rgb(0.0, 0.0, 0.5), 0.0, 0.0),
                                Transform::new(Vector::new(1.0, 2.0, -15.0),
                                               Vector::new(0.0, 0.0, 0.0),
                                               2.0)));
    }
    {
        let mesh = Mesh::load(Path::new("models/buddha.obj"), "buddha");
        objs.push(Instance::new(Box::new(mesh),
                                Material::new(Colourf::rgb(0.0, 0.0, 0.5), 0.0, 0.0),
                                Transform::new(Vector::new(6.0, 6.0, -15.0),
                                               Vector::new(0.0, PI, 0.0),
                                               10.0)));
    }
    // scene.push_mesh(Path::new("models/lucy.obj"),
    //                 "lucy",
    //                 Transform::new(Vector::new(4.0, 5.0, -10.0),
    //                                Vector::new(FRAC_PI_2, 0.0, 0.0),
    //                                4.0));
    // scene.push_sphere(3.0,
    //                   Colourf::rgb(0.65, 0.77, 0.97),
    //                   0.0,
    //                   0.0,
    //                   Transform::new(Vector::new(5.0, height, -25.0), zero(), 1.0));
    objs.push(Instance::new(Box::new(Sphere::new(3.0)),
                            Material::new(Colourf::rgb(0.90, 0.90, 0.90), 1.0, 1.0),
                            Transform::new(Vector::new(-6.5, 4.0, -15.0), zero(), 1.0)));
    objs.push(Instance::new(Box::new(Plane),
                            Material::new(Colourf::rgb(1.0, 1.0, 1.0), 0.0, 0.0),
                            Transform::new(Vector::new(0.0, height - 4.0, 0.0),
                                           Vector::new(FRAC_PI_2, 0.0, 0.0),
                                           20.0)));
    // Light
    lights.push(Box::new(PointLight::new(Point::new(-10.0, 10.0, -5.0),
                                         Colourf::rgb(3000.0, 2000.0, 2000.0))));
    lights.push(Box::new(DistantLight::new(-Vector::y() - Vector::z(), Colourf::rgb(3.0, 3.0, 3.0))));

    let scene = Scene::new(camera, integrator, &mut objs, lights);

    let duration = Duration::span(|| {
        renderer::render(Arc::new(scene),
                         dim,
                         &args.flag_output,
                         args.flag_threads,
                         args.flag_spp,
                         args.flag_block_size)
    });
    let stats = rt::stats::get_stats();
    println!("Render time                : {}", duration);
    println!("Primary rays               : {}", stats.primary_rays);
    println!("Secondary rays             : {}", stats.secondary_rays);
    println!("Num triangles              : {}", stats.triangles);
    println!("Ray-triangle tests         : {}", stats.ray_triangle_tests);
    println!("Ray-triangle intersections : {}", stats.ray_triangle_isect);
    println!("Fast bounding-box test     : {}", stats.fast_bbox_isect);
}
