extern crate chrono;
extern crate nalgebra as na;
extern crate rustracer as rt;
#[macro_use]
extern crate clap;
extern crate rustc_serialize;
extern crate num_cpus;
#[macro_use(o, slog_info, slog_debug, slog_warn, slog_error, slog_trace, slog_log)]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate slog_stream;
extern crate thread_id;

use std::sync::Arc;
use std::num::ParseIntError;
use std::fs::OpenOptions;
use std::path::Path;
use std::io;
use std::process;

use chrono::Local;
use clap::{Arg, ArgMatches, App};
use slog::*;

use rt::bvh::BVH;
use rt::camera::Camera;
use rt::integrator::{SamplerIntegrator, Whitted, Normal, AmbientOcclusion};
use rt::light::{Light, DistantLight, DiffuseAreaLight};
use rt::material::matte::MatteMaterial;
use rt::material::plastic::Plastic;
use rt::material::metal::Metal;
use rt::{Point, Vector, Dim};
use rt::primitive::{Primitive, GeometricPrimitive};
use rt::renderer;
use rt::scene::Scene;
use rt::shapes::disk::Disk;
use rt::shapes::sphere::Sphere;
use rt::spectrum::Spectrum;
use rt::transform;

arg_enum! {
  #[derive(Debug)]
  enum SamplerIntegratorType {
      Whitted,
      Ao,
      Normal
  }
}

fn main() {
    // configure logger
    configure_logger();

    let matches = parse_args();
    if let Err(e) = run(matches) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}

fn run(matches: ArgMatches) -> Result<(), String> {
    let parsed_dims: Result<Vec<u32>, ParseIntError> =
        matches.value_of("dim").unwrap().split('x').map(|s| s.parse::<u32>()).collect();
    let dims = parsed_dims.expect("Invalid dimensions");
    if dims.len() != 2 {
        panic!("Error: invalid dimension specification");
    }
    let dim = (dims[0], dims[1]);

    let integrator: Box<SamplerIntegrator + Send + Sync> =
        match value_t!(matches.value_of("integrator"), SamplerIntegratorType)
            .unwrap_or(SamplerIntegratorType::Whitted) {
            SamplerIntegratorType::Whitted => {
                info!("Using Whitted integrator with max ray depth of {}", 8);
                // Box::new(Whitted::new(args.flag_whitted_max_ray_depth))
                Box::new(Whitted::new(8))
            }
            SamplerIntegratorType::Ao => {
                info!("Using Ambient Occlusion integrator with {} samples", 32);
                Box::new(AmbientOcclusion::new(32))
            }
            SamplerIntegratorType::Normal => {
                info!("Using normal facing ratio integrator");
                Box::new(Normal {})
            }
        };

    let scene = build_scene(dim, integrator);

    let start_time = std::time::Instant::now();
    let stats =
        renderer::render(scene,
                         dim,
                         matches.value_of("output").unwrap(),
                         matches.value_of("threads")
                             .and_then(|s| s.parse::<usize>().ok())
                             .unwrap_or_else(num_cpus::get),
                         matches.value_of("spp").and_then(|s| s.parse::<usize>().ok()).unwrap(),
                         32);
    // args.flag_block_size);
    let duration = start_time.elapsed();
    println!("Render time                : {}", duration.human_display());
    println!("Primary rays               : {}", stats.primary_rays);
    println!("Secondary rays             : {}", stats.secondary_rays);
    println!("Num triangles              : {}", stats.triangles);
    println!("Ray-triangle tests         : {}", stats.ray_triangle_tests);
    println!("Ray-triangle intersections : {}", stats.ray_triangle_isect);
    println!("Fast bounding-box test     : {}", stats.fast_bbox_isect);

    Ok(())
}

fn build_scene(dim: Dim, integrator: Box<SamplerIntegrator + Send + Sync>) -> Scene {
    info!("Building scene");
    let camera = Camera::new(Point::new(0.0, 0.0, 5.0), dim, 50.0);
    let mut lights: Vec<Arc<Light + Send + Sync>> = Vec::new();

    let disk = Arc::new(Disk::new(-2.0, 0.5, 0.0, 360.0, transform::rot_x(90.0)));
    let area_light =
        Arc::new(DiffuseAreaLight::new(Spectrum::rgb(2.0, 2.0, 2.0), disk.clone(), 16));
    let area_light_prim = Box::new(GeometricPrimitive {
        shape: disk.clone(),
        area_light: Some(area_light.clone()),
        material: Some(Arc::new(MatteMaterial::default())),
    });

    let bronze = Arc::new(Metal::new());
    let sphere = Box::new(GeometricPrimitive {
        shape: Arc::new(Sphere::new().transform(transform::rot(45.0, 45.0, 0.0))),
        area_light: None,
        // material: Some(Arc::new(Plastic::new(Spectrum::red(), Spectrum::white()))),
        // material: Some(Arc::new(Plastic::new_tex("lines.png", Spectrum::white()))),
        material: Some(bronze.clone()),
    });
    let bunny =
        Box::new(BVH::<GeometricPrimitive>::from_mesh_file(&Path::new("models/bunny.obj"),
                                                           "bunny",
                                                           bronze.clone(),
                                                           &na::one())) as Box<Primitive + Send + Sync>;
    let floor = Box::new(GeometricPrimitive {
        shape: Arc::new(Disk::new(-1.0, 20.0, 0.0, 360.0, transform::rot_x(-90.0))),
        area_light: None,
        // material: Some(Arc::new(MatteMaterial::checkerboard(0.0))),
        material: Some(Arc::new(MatteMaterial::new(Spectrum::red(), 0.0))),
    });

    let primitives: Vec<Box<Primitive + Sync + Send>> = vec![bunny, floor, area_light_prim];
    // Light
    lights.push(area_light);
    lights.push(Arc::new(DistantLight::new(Vector::new(0.0, -1.0, -5.0),
                                           Spectrum::rgb(1.0, 1.0, 1.0))));

    Scene::new(camera, integrator, primitives, lights)
}

fn parse_args<'a>() -> ArgMatches<'a> {
    // TODO add block-size and ao-samples and whitted-max-ray? Or maybe not since eventually it
    // will be read from the scene file...
    App::new("rustracer")
        .version("0.1")
        .author("Antoine Bűsch")
        .about("Toy raytracer in Rust based on PBRTv3")
        .arg(Arg::with_name("output")
            .long("output")
            .short("o")
            .help("Output file name")
            .default_value("image.png"))
        .arg(Arg::with_name("threads")
            .long("threads")
            .short("t")
            .help("Number of worker threads"))
        .arg(Arg::with_name("dim")
            .long("dim")
            .short("d")
            .help("Dimension of the output image")
            .default_value("800x600"))
        .arg(Arg::with_name("integrator")
            .long("integrator")
            .short("i")
            .help("SamplerIntegrator to use")
            .possible_values(&SamplerIntegratorType::variants()))
        .arg(Arg::with_name("spp")
            .long("spp")
            .help("Sample per pixels")
            .default_value("4"))
        .get_matches()
}

fn configure_logger() {
    let log_path = "/tmp/rustracer.log";
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .unwrap();

    let drain = slog_stream::stream(file, MyFormat).fuse();
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
        let msg = format!("{} [{}][{:x}][{}:{} {}] - {}\n",
                          now!(),
                          rinfo.level(),
                          thread_id::get(),
                          rinfo.file(),
                          rinfo.line(),
                          rinfo.module(),
                          rinfo.msg());
        let _ = try!(io.write_all(msg.as_bytes()));
        Ok(())
    }
}
