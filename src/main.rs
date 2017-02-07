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

mod logging;

use std::sync::Arc;
use std::num::ParseIntError;
use std::path::Path;
use std::process;

use clap::{Arg, ArgMatches, App};

use rt::bvh::BVH;
use rt::camera::Camera;
use rt::display::{DisplayUpdater, MinifbDisplayUpdater, NoopDisplayUpdater};
use rt::integrator::{SamplerIntegrator, Whitted, DirectLightingIntegrator, Normal, AmbientOcclusion,
                     PathIntegrator};
use rt::light::{Light, DistantLight, DiffuseAreaLight, InfiniteAreaLight};
use rt::material::matte::MatteMaterial;
use rt::material::metal::Metal;
use rt::material::plastic::Plastic;
use rt::material::glass::GlassMaterial;
use rt::primitive::{Primitive, GeometricPrimitive};
use rt::renderer;
use rt::scene::Scene;
use rt::shapes::disk::Disk;
use rt::shapes::sphere::Sphere;
use rt::spectrum::Spectrum;
use rt::transform;
use rt::{Transform, Vector3f, Dim, Point2f};

arg_enum! {
  #[derive(Debug)]
  enum SamplerIntegratorType {
      Whitted,
      DirectLighting,
      PathTracing,
      Ao,
      Normal
  }
}

fn main() {
    let matches = parse_args();

    // configure logger
    let level = if matches.is_present("verbose") {
        slog::Level::Debug
    } else {
        slog::Level::Info
    };
    logging::configure_logger(level);


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

    let scene = build_scene(dim);

    let integrator: Box<SamplerIntegrator + Send + Sync> =
        match value_t!(matches.value_of("integrator"), SamplerIntegratorType)
            .unwrap_or(SamplerIntegratorType::Whitted) {
            SamplerIntegratorType::Whitted => {
                info!("Using Whitted integrator with max ray depth of {}", 8);
                // Box::new(Whitted::new(args.flag_whitted_max_ray_depth))
                Box::new(Whitted::new(8))
            }
            SamplerIntegratorType::DirectLighting => {
                info!("Using direct lighting integrator with max ray depth of {}",
                      8);
                // Box::new(Whitted::new(args.flag_whitted_max_ray_depth))
                Box::new(DirectLightingIntegrator::new(8))
            }
            SamplerIntegratorType::PathTracing => {
                info!("Using path tracing integrator");
                Box::new(PathIntegrator::new(&scene))
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

    let disp: Box<DisplayUpdater + Send> = if matches.is_present("display") {
        Box::new(MinifbDisplayUpdater::new(dim))
    } else {
        Box::new(NoopDisplayUpdater)
    };

    let start_time = std::time::Instant::now();
    let stats =
        renderer::render(scene,
                         integrator,
                         dim,
                         matches.value_of("output").unwrap(),
                         matches.value_of("threads")
                             .and_then(|s| s.parse::<usize>().ok())
                             .unwrap_or_else(num_cpus::get),
                         matches.value_of("spp").and_then(|s| s.parse::<usize>().ok()).unwrap(),
                         16,
                         disp);
    // args.flag_block_size);
    let duration = start_time.elapsed();
    println!("Render time                : {}", duration.human_display());
    println!("Primary rays               : {}", stats.primary_rays);
    println!("Secondary rays             : {}", stats.secondary_rays);
    println!("Num triangles              : {}", stats.triangles);
    println!("Ray-triangle tests         : {}", stats.ray_triangle_tests);
    println!("Ray-triangle intersections : {}\t({}%)",
             stats.ray_triangle_isect,
             stats.ray_triangle_isect as f32 / stats.ray_triangle_tests as f32 * 100.0);
    println!("Fast bounding-box test     : {}", stats.fast_bbox_isect);

    Ok(())
}

fn build_scene2(dim: Dim) -> Scene {
    info!("Building scene");
    let camera = Camera::new(na::one(),
                             Point2f::new(dim.0 as f32, dim.1 as f32),
                             0.0,
                             0.0,
                             50.0);
    let mut lights: Vec<Arc<Light + Send + Sync>> = Vec::new();

    let shape = Arc::new(Disk::new(5.0, 1.0, 0.0, 360.0, na::one()));
    let material = Arc::new(MatteMaterial::new_image("grid.png"));
    // let material = Arc::new(MatteMaterial::new_uv_texture());

    let disk = Box::new(GeometricPrimitive {
        shape: shape,
        area_light: None,
        material: Some(material.clone()),
    });

    let primitives: Vec<Box<Primitive + Sync + Send>> = vec![disk];
    // Light
    lights.push(Arc::new(DistantLight::new(Vector3f::z(), Spectrum::white())));

    Scene::new(camera, primitives, lights)
}

fn build_scene(dim: Dim) -> Scene {
    info!("Building scene");
    let camera = Camera::new(transform::translate_z(-3.0),
                             Point2f::new(dim.0 as f32, dim.1 as f32),
                             0.05,
                             2.5,
                             60.0);
    let mut lights: Vec<Arc<Light + Send + Sync>> = Vec::new();

    let disk = Arc::new(Disk::new(-2.0, 0.8, 0.0, 360.0, transform::rot_x(90.0)));
    let area_light =
        Arc::new(DiffuseAreaLight::new(Spectrum::rgb(1.0, 1.0, 1.0), disk.clone(), 16));
    let area_light_prim = Box::new(GeometricPrimitive {
        shape: disk.clone(),
        area_light: Some(area_light.clone()),
        material: Some(Arc::new(MatteMaterial::default())),
    });

    let bronze = Arc::new(Metal::new());
    let gold = Arc::new(Metal::gold());
    let plastic = Arc::new(Plastic::new(Spectrum::rgb(0.3, 0.3, 1.0), Spectrum::white()));
    let matte_red = Arc::new(MatteMaterial::new(Spectrum::rgb(1.0, 0.0, 0.0), 0.0));
    let plastic_white = Arc::new(Plastic::new(Spectrum::rgb(1.0, 1.0, 1.0), Spectrum::white()));
    let plastic_lines = Arc::new(Plastic::new_tex("grid.png", Spectrum::white()));
    // let plastic_lines = Arc::new(MatteMaterial::new_uv_texture());
    // let sphere = Box::new(GeometricPrimitive {
    //     shape: Arc::new(Sphere::new().radius(0.7).transform(transform::translate_y(-0.3))),
    //     area_light: None,
    //     material: Some(plastic_white.clone()),
    // });
    // let bunny =
    //     Box::new(BVH::<GeometricPrimitive>::from_mesh_file(Path::new("models/bunny.obj"),
    //                                                        "bunny",
    //                                                        plastic.clone(),
    //                                                        &Transform::new(
    //                                                          Vector3f::new(2.0, -0.8, 0.0),
    //                                                          Vector3f::new(0.0, 20.0f32.to_radians(), 0.0),
    //                                                          0.5
    //                                                          ))) as Box<Primitive + Send + Sync>;
    // let buddha =
    //     Box::new(BVH::<GeometricPrimitive>::from_mesh_file(Path::new("models/buddha.obj"),
    //                                                        "buddha",
    //                                                        gold.clone(),
    //                                                        &Transform::new(
    //                                                          Vector3f::new(-2.0, 0.0, 0.0),
    //                                                          Vector3f::new(0.0, 0.0, 0.0),
    //                                                          2.0
    //                                                          ))) as Box<Primitive + Send + Sync>;
    let dragon =
        Box::new(BVH::<GeometricPrimitive>::from_mesh_file(Path::new("models/dragon.obj"),
                                                           "dragon",
                                                           gold.clone(),
                                                           &Transform::new(
                                                             Vector3f::new(-0.2, 0.0, 0.0),
                                                             Vector3f::new(0.0, -70.0f32.to_radians(), 0.0),
                                                             3.0
                                                             ))) as Box<Primitive + Send + Sync>;
    let floor = Box::new(GeometricPrimitive {
        shape: Arc::new(Disk::new(-1.0, 20.0, 0.0, 360.0, transform::rot_x(-90.0))),
        area_light: None,
        material: Some(matte_red.clone()),
    });

    let primitives: Vec<Box<Primitive + Sync + Send>> = vec![dragon, floor];
    // Light
    // lights.push(area_light);
    // lights.push(Arc::new(DistantLight::new(Vector3f::new(0.0, -1.0, 5.0),
    //                                        Spectrum::rgb(1.0, 1.0, 1.0))));
    lights.push(Arc::new(InfiniteAreaLight::new(na::one(),
                                                16,
                                                Spectrum::grey(1.0),
                                                Path::new("sky_sanmiguel.tga"))));

    Scene::new(camera, primitives, lights)
}

fn parse_args<'a>() -> ArgMatches<'a> {
    // TODO add block-size and ao-samples and whitted-max-ray? Or maybe not since eventually it
    // will be read from the scene file...
    App::new("rustracer")
        .version("0.1")
        .author("Antoine Büsch")
        .about("Toy raytracer in Rust based on PBRTv3")
        .arg(Arg::with_name("output")
            .long("output")
            .short("o")
            .help("Output file name")
            .default_value("image.png"))
        .arg(Arg::with_name("threads")
            .long("threads")
            .short("t")
            .help("Number of worker threads")
            .takes_value(true))
        .arg(Arg::with_name("dim")
            .long("dim")
            .short("d")
            .help("Dimension of the output image")
            .default_value("800x600"))
        .arg(Arg::with_name("integrator")
            .long("integrator")
            .short("i")
            .help("SamplerIntegrator to use")
            .possible_values(&SamplerIntegratorType::variants())
            .default_value("Whitted"))
        .arg(Arg::with_name("spp")
            .long("spp")
            .help("Sample per pixels")
            .default_value("4"))
        .arg(Arg::with_name("verbose")
            .short("v")
            .help("log debug information"))
        .arg(Arg::with_name("display")
            .short("p")
            .help("Display image as it is rendered"))
        .get_matches()
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
            minutes = seconds / 60;
            seconds %= 60;
        }
        if minutes >= 60 {
            hours = minutes / 60;
            minutes %= 60;
        }
        let millis = self.subsec_nanos() / 1000000;
        format!("{}:{}:{}.{}", hours, minutes, seconds, millis)
    }
}
