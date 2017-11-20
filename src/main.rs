extern crate clap;
extern crate failure;
extern crate rustracer_core as rt;
#[macro_use]
extern crate slog;
extern crate slog_scope;
extern crate slog_term;

mod logging;
mod argparse;

use clap::ArgMatches;
use failure::Error;

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

fn run(matches: &ArgMatches) -> Result<(), Error> {
    let filename = matches.value_of("INPUT").unwrap();
    pbrt::parse_scene(filename)?;

    Ok(())
}
