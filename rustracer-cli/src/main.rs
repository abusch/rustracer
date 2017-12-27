extern crate clap;
extern crate failure;
extern crate rustracer_core as rt;
#[macro_use]
extern crate log;
extern crate flexi_logger;

mod argparse;

use clap::ArgMatches;
use failure::Error;

use rt::pbrt;

fn main() {
    let matches = argparse::parse_args();

    // configure logger
    // let level = if matches.is_present("verbose") {
    //     slog::Level::Debug
    // } else {
    //     slog::Level::Info
    // };
    flexi_logger::Logger::with_str("rustracer=info")
        .log_to_file()
        .suppress_timestamp()
        .directory("/tmp")
        .format(flexi_logger::opt_format)
        .start()
        .unwrap_or_else(|e| panic!("Failed to initialize logger: {}", e));


    if let Err(ref e) = run(&matches) {
        println!("Application error: {}", e);
        ::std::process::exit(1);
    }
}

fn run(matches: &ArgMatches) -> Result<(), Error> {
    rt::init_stats();
    let filename = matches.value_of("INPUT").unwrap();
    pbrt::parse_scene(filename)?;

    Ok(())
}
