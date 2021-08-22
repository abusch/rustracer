#![recursion_limit = "128"]

mod argparse;

use anyhow::Result;
use clap::ArgMatches;

use flexi_logger::FileSpec;
use rustracer_core::{init_stats, pbrt, PbrtOptions};

fn main() {
    println!("Rustracer 0.1 [Detected {} cores]", num_cpus::get());
    println!("Copyright (c)2016-2018 Antoine Büsch.");
    println!("Based on the original PBRTv3 code by Matt Pharr, Greg Humphreys, and Wenzel Jacob.");
    let matches = argparse::parse_args();

    flexi_logger::Logger::try_with_str("rustracer=info,rustracer_core=info")
        .unwrap()
        .log_to_file(FileSpec::default().suppress_timestamp().directory("/tmp"))
        .format(flexi_logger::opt_format)
        .start()
        .unwrap_or_else(|e| panic!("Failed to initialize logger: {}", e));

    if let Err(ref e) = run(&matches) {
        println!("Application error: {}", e);
        ::std::process::exit(1);
    }
}

fn run(matches: &ArgMatches) -> Result<()> {
    init_stats();
    let nthreads = matches
        .value_of("nthreads")
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(0);
    let opts = PbrtOptions {
        num_threads: nthreads as u8,
        ..PbrtOptions::default()
    };
    let filename = matches.value_of("INPUT").unwrap();
    pbrt::parse_scene(opts, filename)?;

    Ok(())
}
