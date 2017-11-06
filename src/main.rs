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
    pbrt::parse_scene(filename)?;

    Ok(())
}