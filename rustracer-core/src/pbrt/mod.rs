mod lexer;
mod parser;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use anyhow::*;

use crate::api::{Api, RealApi};
use crate::fileutil;
use crate::pbrt::lexer::Tokens;
use crate::PbrtOptions;

pub fn parse_scene<P: AsRef<Path>>(opts: PbrtOptions, filename: P) -> Result<()> {
    let filename = filename.as_ref();
    let tokens = tokenize_file(filename)?;
    fileutil::set_search_directory(fileutil::directory_containing(filename));
    let api = RealApi::with_options(opts);
    api.init()?;
    parser::parse(Tokens::new(&tokens[..]), &api)
        .map_err(|e| format_err!("Failed to parse scene file: {:?}", e))?;

    Ok(())
}

pub fn tokenize_file<P: AsRef<Path>>(filename: P) -> Result<Vec<lexer::Token>> {
    let resolved_filename = fileutil::resolve_filename(filename.as_ref().to_str().unwrap());
    let mut file = File::open(&resolved_filename).context("Failed to open scene file")?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)
        .context("Failed to read content of scene file")?;

    // TODO handle errors
    let (_rest, tokens) = lexer::tokenize(&file_content[..])
        .map_err(|e| format_err!("Failed to tokenize scene file: {:?}", e))?;
    // strip comments
    let filtered_tokens = tokens
        .into_iter()
        .filter(|x| *x != lexer::Token::COMMENT)
        .collect::<Vec<_>>();

    Ok(filtered_tokens)
}

#[ignore]
#[test]
fn test_parse_scene() {
    let scene = r##"
LookAt 0 0 5 0 0 0 0 1 0
Camera "perspective" "float fov" [50]
Sampler "02sequence"

Film "image" "integer xresolution" [800] "integer yresolution" [600]
    "string filename" "test-whitted.tga"

Integrator "whitted"
#Integrator "directlighting"

WorldBegin
  LightSource "distant" "point from" [0 1 5] "point to" [0 0 0]

  #Material "matte" "rgb Kd" [1.0 0.0 0.0] "float sigma" [20]
  AttributeBegin
    #Material "matte" "rgb Kd" [1.0 0.0 0.0]
    Material "plastic" "rgb Kd" [1.0 0.0 0.0] "rgb Ks" [1.0 1.0 1.0]

    Shape "sphere"
  AttributeEnd

  AttributeBegin
    Rotate -90 1 0 0
    Material "matte" "rgb Kd" [1.0 1.0 1.0]
    Shape "disk" "float radius" [20] "float height" [-1]
  AttributeEnd

  AttributeBegin
    AreaLightSource "diffuse" "rgb L" [2.0 2.0 2.0]
    Rotate 90 1 0 0
    Shape "disk" "float height" [-2] "float radius" [0.5]
  AttributeEnd
WorldEnd
        "##;

    parse_scene(PbrtOptions::default(), scene).unwrap();
}
