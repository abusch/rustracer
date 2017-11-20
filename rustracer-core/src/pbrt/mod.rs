mod lexer;
mod parser;

use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use failure::*;

use api::{Api, RealApi};
use fileutil;

pub fn parse_scene<P: AsRef<Path>>(filename: P) -> Result<(), Error> {
    let filename = filename.as_ref();
    let mut file = File::open(filename)
        .context("Failed to open scene file")?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)
        .context("Failed to read content of scene file")?;

    // TODO handle errors
    let tokens = lexer::tokenize(&file_content)
        .map_err(|e| format_err!("Failed to tokenize scene file: {:?}", e))?;
    // strip comments
    let filtered_tokens = tokens
        .0
        .into_iter()
        .filter(|x| *x != lexer::Tokens::COMMENT)
        .collect::<Vec<_>>();
    fileutil::set_search_directory(fileutil::directory_containing(filename));
    let api = RealApi::default();
    api.init()?;
    parser::parse(&filtered_tokens[..], &api)
        .map_err(|e| format_err!("Failed to parse scene file: {:?}", e))?;

    Ok(())
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

    parse_scene(scene).unwrap();
}
