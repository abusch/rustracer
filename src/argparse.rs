use clap::{App, Arg, ArgMatches};

pub fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("rustracer")
        .version("0.1")
        .author("Antoine Büsch")
        .about("Toy raytracer in Rust based on PBRTv3")
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
                .help("Output file name")
                .default_value("image.png"),
        )
        .arg(
            Arg::with_name("threads")
                .long("threads")
                .short("t")
                .help("Number of worker threads")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .help("log debug information"),
        )
        .arg(
            Arg::with_name("display")
                .short("p")
                .help("Display image as it is rendered"),
        )
        .arg(
            Arg::with_name("INPUT")
                .required(true)
                .index(1)
                .help("PBRT scene file to render"),
        )
        .get_matches()
}
