extern crate chrono;
extern crate clap;
extern crate error_chain;
extern crate rustracer as rt;
#[macro_use]
extern crate slog;
extern crate slog_scope;
extern crate slog_term;
extern crate thread_id;

mod logging;
mod argparse;

use std::fs;
use std::io::Read;

use clap::ArgMatches;

use rt::errors::*;
use rt::pbrt;

fn main() {
    let matches = argparse::parse_args();

    // configure logger
    let level = if matches.is_present("verbose") {
        slog::Level::Debug
    } else {
        slog::Level::Info
    };
    let _guard = logging::configure_logger(level);


    if let Err(ref e) = run(&matches) {
        println!("Application error: {}", e);
        ::std::process::exit(1);
    }
}

fn run(matches: &ArgMatches) -> Result<()> {
    let filename = matches.value_of("INPUT").unwrap();
    let mut file = fs::File::open(filename).chain_err(|| "Failed to open scene file")?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)
        .chain_err(|| "Failed to read content of scene file")?;
    pbrt::parse_scene(&file_content[..])?;

    /*
    let (scene, camera) = samplescenes::build_scene(dim);

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
                Box::new(DirectLightingIntegrator::new(8, LightStrategy::UniformSampleOne))
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
    let spp = matches
        .value_of("spp")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap();

    let start_time = std::time::Instant::now();
    let stats = renderer::render(Box::new(scene),
                                 integrator,
                                 camera,
                                 matches
                                     .value_of("threads")
                                     .and_then(|s| s.parse::<usize>().ok())
                                     .unwrap_or_else(num_cpus::get),
                                 Box::new(ZeroTwoSequence::new(spp, 4)),
                                 16,
                                 disp)?;
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
*/
    Ok(())
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
        let millis = self.subsec_nanos() / 1_000_000;
        format!("{}:{}:{}.{}", hours, minutes, seconds, millis)
    }
}
