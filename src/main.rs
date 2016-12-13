extern crate chrono;
extern crate nalgebra as na;
extern crate rustracer as rt;
extern crate docopt;
extern crate rustc_serialize;
extern crate num_cpus;
#[macro_use(o, slog_info, slog_debug, slog_warn, slog_error, slog_trace, slog_log)]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate slog_stream;

use std::f32::consts;
use std::sync::Arc;
use std::num::ParseIntError;
use std::fs::OpenOptions;
use std::io;
use std::thread;

use chrono::Local;
use docopt::Docopt;
use slog::*;

use rt::{Point, Vector, Transform, Dim};
use rt::scene::Scene;
use rt::spectrum::Spectrum;
use rt::camera::Camera;
use rt::integrator::{SamplerIntegrator, Whitted, Normal, AmbientOcclusion};
use rt::light::{Light, PointLight, DistantLight, DiffuseAreaLight};
use rt::material::matte::MatteMaterial;
use rt::material::plastic::Plastic;
use rt::primitive::{Primitive, GeometricPrimitive};
use rt::shapes::disk::Disk;
use rt::shapes::sphere::Sphere;
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
     whitted, normal, ao.
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
    flag_block_size: u32,
}

#[derive(RustcDecodable)]
enum SamplerIntegratorType {
    Whitted,
    Ao,
    Normal,
}

fn main() {
    // configure logger
    configure_logger();

    // Parse args
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.help(true).decode())
        .unwrap_or_else(|e| e.exit());

    let parsed_dims: Result<Vec<u32>, ParseIntError> =
        args.flag_dimension.split('x').map(|s| s.parse::<u32>()).collect();
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
    info!("Building scene");
    let camera = Camera::new(Point::new(0.0, 0.0, 5.0), dim, 50.0);
    let integrator: Box<SamplerIntegrator + Send + Sync> = match args.flag_integrator {
        SamplerIntegratorType::Whitted => {
            info!("Using Whitted integrator with max ray depth of {}",
                  args.flag_whitted_max_ray_depth);
            Box::new(Whitted::new(args.flag_whitted_max_ray_depth))
        }
        SamplerIntegratorType::Ao => {
            info!("Using Ambient Occlusion integrator with {} samples",
                  args.flag_ao_samples);
            Box::new(AmbientOcclusion::new(args.flag_ao_samples))
        }
        SamplerIntegratorType::Normal => {
            info!("Using normal facing ratio integrator");
            Box::new(Normal {})
        }
    };

    let mut lights: Vec<Arc<Light + Send + Sync>> = Vec::new();

    let disk = Arc::new(Disk::new(-2.0,
                                  0.5,
                                  0.0,
                                  360.0,
                                  Transform::new(na::zero(),
                                                 Vector::new(consts::FRAC_PI_2, 0.0, 0.0),
                                                 1.0)));
    let area_light =
        Arc::new(DiffuseAreaLight::new(Spectrum::rgb(2.0, 2.0, 2.0), disk.clone(), 16));
    let area_light_prim = Box::new(GeometricPrimitive {
        shape: disk.clone(),
        area_light: Some(area_light.clone()),
        material: Some(Arc::new(MatteMaterial::default())),
    });

    let primitives: Vec<Box<Primitive + Sync + Send>> = vec![Box::new(GeometricPrimitive {
                 shape: Arc::new(Sphere::default()),
                 area_light: None,
                 // material: Some(Arc::new(Plastic::new(Spectrum::red(), Spectrum::white()))),
                 material: Some(Arc::new(Plastic::new_tex("lines.png", Spectrum::white()))),
             }),
             Box::new(GeometricPrimitive {
                 shape: Arc::new(Disk::new(-1.0,
                                           20.0,
                                           0.0,
                                           360.0,
                                           Transform::new(na::zero(),
                                                          Vector::new(-consts::PI / 2.0,
                                                                      0.0,
                                                                      0.0),
                                                          1.0))),
                 area_light: None,
                 // material: Some(Arc::new(MatteMaterial::checkerboard(0.0))),
                 material: Some(Arc::new(MatteMaterial::new(Spectrum::white(), 0.0))),
             }),
             area_light_prim];
    // Light
    lights.push(area_light);
    // lights.push(Box::new(PointLight::new(Point::new(-3.0, 3.0, 3.0),
    //                                      Spectrum::rgb(100.0, 100.0, 100.0))));
    lights.push(Arc::new(DistantLight::new(Vector::new(0.0, -1.0, -5.0),
                                           Spectrum::rgb(1.0, 1.0, 1.0))));

    Scene::new(camera, integrator, primitives, lights)
}

fn configure_logger() {
    let log_path = "/tmp/rustracer.log";
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .unwrap();

    let drain = slog_stream::async_stream(file, MyFormat).fuse();
    let log = Logger::root(drain, o!());
    slog_scope::set_global_logger(log);

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

macro_rules! now {
    () => ( Local::now().format("%m-%d %H:%M:%S%.3f") )
}

struct MyFormat;

impl slog_stream::Format for MyFormat {
    fn format(&self,
              io: &mut io::Write,
              rinfo: &slog::Record,
              _logger_values: &slog::OwnedKeyValueList)
              -> io::Result<()> {
        let msg = format!("{} [{}][{}][{}:{} {}] - {}\n",
                          now!(),
                          rinfo.level(),
                          thread::current().name().unwrap_or("unnamed"),
                          rinfo.file(),
                          rinfo.line(),
                          rinfo.module(),
                          rinfo.msg());
        let _ = try!(io.write_all(msg.as_bytes()));
        Ok(())
    }
}
