extern crate nalgebra as na;
extern crate rustracer as rt;
extern crate docopt;
extern crate rustc_serialize;
extern crate num_cpus;

use std::f32::consts::*;
use std::path::Path;
use std::sync::Arc;
use std::num::ParseIntError;
use na::zero;
use docopt::Docopt;

use rt::{Point, Vector, Transform, Dim};
use rt::scene::Scene;
use rt::colour::Colourf;
use rt::camera::Camera;
use rt::geometry::*;
use rt::instance::Instance;
use rt::integrator::{SamplerIntegrator, Whitted, Normal};
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
     of worker threads to start.
  -d <dim>, --dimension=<dim>                 \
     Dimension of the output image [default: 800x600].
  --spp N                                     \
     Samples per pixel [default: 4].
  --block-size=N                              Block size \
     used for rendering [default: 32].
  -i <integrator>, --integrator=<integrator>  SamplerIntegrator \
     to use [default: whitted].
                                              Valid values: \
     whitted, normal.
  --whitted-max-ray-depth=N                   Maximum ray depth for \
     Whitted integrator. [default: 8].
  --ao-samples=N                              Number of \
     samples for ambient occlusion integrator [default: 16].
";

#[derive(RustcDecodable)]
struct Args {
    flag_output: String,
    flag_threads: Option<usize>,
    flag_integrator: SamplerIntegratorType,
    flag_whitted_max_ray_depth: u8,
    flag_ao_samples: usize,
    flag_dimension: String,
    flag_spp: usize,
    flag_block_size: usize,
}

#[derive(RustcDecodable)]
enum SamplerIntegratorType {
    Whitted,
    // Ao,
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

    let scene = bunny_buddah(dim, &args);

    let start_time = std::time::Instant::now();
    let stats = renderer::render(Arc::new(scene),
                                 dim,
                                 &args.flag_output,
                                 args.flag_threads.unwrap_or_else(num_cpus::get),
                                 args.flag_spp,
                                 args.flag_block_size);
    let duration = start_time.elapsed();
    println!("Render time                : {}", duration.human_display());
    println!("Primary rays               : {}", stats.primary_rays);
    println!("Secondary rays             : {}", stats.secondary_rays);
    println!("Num triangles              : {}", stats.triangles);
    println!("Ray-triangle tests         : {}", stats.ray_triangle_tests);
    println!("Ray-triangle intersections : {}", stats.ray_triangle_isect);
    println!("Fast bounding-box test     : {}", stats.fast_bbox_isect);
}

fn bunny_buddah(dim: Dim, args: &Args) -> Scene {
    let camera = Camera::new(Point::new(0.0, 0.0, 5.0), dim, 50.0);
    let integrator: Box<SamplerIntegrator + Send + Sync> = match args.flag_integrator {
        SamplerIntegratorType::Whitted => {
            println!("Using Whitted integrator with max ray depth of {}",
                     args.flag_whitted_max_ray_depth);
            Box::new(Whitted::new(args.flag_whitted_max_ray_depth))
        }
        // SamplerIntegratorType::Ao => {
        //     println!("Using Ambient Occlusion integrator with {} samples",
        //              args.flag_ao_samples);
        //     Box::new(AmbientOcclusion::new(args.flag_ao_samples))
        // }
        SamplerIntegratorType::Normal => {
            println!("Using normal facing ratio integrator");
            Box::new(Normal {})
        }
    };

    let mut objs = Vec::new();
    let mut lights: Vec<Box<Light + Send + Sync>> = Vec::new();
    let height = 5.0;

    // {
    //     let mesh = Mesh::load(Path::new("models/bunny.obj"), "bunny");
    //     objs.push(Instance::new(Box::new(mesh),
    //                             Arc::new(Material::new(Colourf::rgb(0.0, 0.0, 0.5), 0.0, 0.0)),
    //                             Transform::new(Vector::new(1.0, 2.0, -15.0),
    //                                            Vector::new(0.0, 0.0, 0.0),
    //                                            2.0)));
    // }
    // {
    //     let mesh = Mesh::load(Path::new("models/buddha.obj"), "buddha");
    //     objs.push(Instance::new(Box::new(mesh),
    //                             Arc::new(Material::new(Colourf::rgb(0.0, 0.0, 0.5), 0.0, 0.0)),
    //                             Transform::new(Vector::new(6.0, 6.0, -15.0),
    //                                            Vector::new(0.0, PI, 0.0),
    //                                            10.0)));
    // }
    // objs.push(Instance::new(Box::new(Sphere::new(3.0)),
    //                         Arc::new(Material::new(Colourf::rgb(0.90, 0.90, 0.90), 1.0, 1.0)),
    //                         Transform::new(Vector::new(-6.5, 4.0, -15.0), zero(), 1.0)));
    // objs.push(Instance::new(Box::new(Plane),
    //                         Arc::new(Material::new(Colourf::rgb(1.0, 1.0, 1.0), 0.0, 0.0)),
    //                         Transform::new(Vector::new(0.0, height - 4.0, 0.0),
    //                                        Vector::new(FRAC_PI_2, 0.0, 0.0),
    //                                        20.0)));
    // Light
    // lights.push(Box::new(PointLight::new(Point::new(-5.0, 5.0, 5.0),
    //                                      Colourf::rgb(3000.0, 2000.0, 2000.0))));
    lights.push(Box::new(DistantLight::new(-Vector::y() - Vector::z(), Colourf::rgb(1.0, 1.0, 1.0))));

    Scene::new(camera, integrator, &mut objs, lights)
}


trait HumanDisplay {
    fn human_display(&self) -> String;
}
impl HumanDisplay for std::time::Duration {
    fn human_display(&self) -> String {
        let mut hours = 0;
        let mut minutes = 0;
        let mut seconds = self.as_secs();
        if seconds >= 60 {
            seconds %= 60;
            minutes = seconds / 60;
        }
        if minutes >= 60 {
            minutes %= 60;
            hours = minutes / 60;
        }
        let millis = self.subsec_nanos() / 1000000;
        format!("{}:{}:{}.{}", hours, minutes, seconds, millis)
    }
}
