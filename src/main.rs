extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate nalgebra as na;
extern crate rustracer as rt;
#[macro_use]
extern crate clap;
extern crate num_cpus;
#[macro_use(o, slog_info, slog_debug, slog_warn, slog_error, slog_trace, slog_log)]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate slog_stream;
extern crate thread_id;

mod logging;
mod argparse;
mod samplescenes;

use std::fs;
use std::io::Read;

use clap::ArgMatches;

use argparse::SamplerIntegratorType;
use rt::Point2i;
use rt::display::{DisplayUpdater, MinifbDisplayUpdater, NoopDisplayUpdater};
use rt::errors::*;
use rt::integrator::{SamplerIntegrator, Whitted, DirectLightingIntegrator, Normal,
                     AmbientOcclusion, PathIntegrator, LightStrategy};
use rt::parser;
use rt::renderer;
use rt::sampler::zerotwosequence::ZeroTwoSequence;

fn main() {
    let matches = argparse::parse_args();

    // configure logger
    let level = if matches.is_present("verbose") {
        slog::Level::Debug
    } else {
        slog::Level::Info
    };
    logging::configure_logger(level);


    if let Err(ref e) = run(matches) {
        println!("Application error: {}", e);
        ::std::process::exit(1);
    }
}

fn run(matches: ArgMatches) -> Result<()> {
    let dims = matches
        .value_of("dim")
        .unwrap()
        .split('x')
        .map(|s| {
                 s.parse::<u32>()
                     .chain_err(|| "Unable to parse dimension")
             })
        .collect::<Result<Vec<u32>>>()?;
    if dims.len() != 2 {
        bail!("Error: invalid dimension specification");
    }
    let dim = Point2i::new(dims[0] as i32, dims[1] as i32);

    let filename = matches.value_of("INPUT").unwrap();
    let mut file = fs::File::open(filename)
        .chain_err(|| "Failed to open scene file")?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)
        .chain_err(|| "Failed to read content of scene file")?;
    parser::parse_scene(&file_content[..])?;

    let scene = samplescenes::build_scene(dim);

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
        let millis = self.subsec_nanos() / 1000000;
        format!("{}:{}:{}.{}", hours, minutes, seconds, millis)
    }
}
