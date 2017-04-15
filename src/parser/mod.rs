mod lexer;
mod parser;

use api::{Api, RealApi};
use errors::*;

#[allow(dead_code)]
pub fn parse_scene(input: &str) -> Result<()> {
    // TODO handle errors
    let tokens = lexer::tokenize(input)
        .map_err(|e| format!("Failed to tokenize scene file: {:?}", e))?;
    // strip comments
    let filtered_tokens = tokens
        .0
        .into_iter()
        .filter(|x| *x != lexer::Tokens::COMMENT)
        .collect::<Vec<_>>();
    let api = RealApi::default();
    api.init()?;
    let res = parser::parse(&filtered_tokens[..], &api)
        .map_err(|e| format!("Failed to parse scene file: {:?}", e))?;

    println!("Scene parsed: {:?} -- {:?}", res.0, res.1);
    Ok(())
}

#[test]
fn test_parse_scene() {
    let scene = r##"
LookAt 0 0 5 0 0 0 0 1 0
Camera "perspective" "float fov" [50]


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

    assert!(parse_scene(scene).is_ok());
}
