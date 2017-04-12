use clap::{Arg, ArgMatches, App};

arg_enum! {
  #[derive(Debug)]
  pub enum SamplerIntegratorType {
      Whitted,
      DirectLighting,
      PathTracing,
      Ao,
      Normal
  }
}

pub fn parse_args<'a>() -> ArgMatches<'a> {
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
                 .default_value("DirectLighting"))
        .arg(Arg::with_name("spp").long("spp").help("Sample per pixels").default_value("4"))
        .arg(Arg::with_name("verbose").short("v").help("log debug information"))
        .arg(Arg::with_name("display").short("p").help("Display image as it is rendered"))
        .arg(Arg::with_name("INPUT").required(true).index(1).help("PBRT scene file to render"))
        .get_matches()
}
