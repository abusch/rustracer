mod lexer;
mod parser;

pub fn parse_scene(scene: &str) {
    // TODO handle errors
    let tokens = lexer::tokenize(scene).unwrap().0;
    // strip comments
    let mut filtered_tokens =
        tokens.into_iter().filter(|x| *x != lexer::Tokens::COMMENT).collect::<Vec<_>>();
    let res = parser::parse(&filtered_tokens[..]).unwrap();

    println!("Scene parsed: {:?} -- {:?}", res.0, res.1);
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

    parse_scene(scene);
}
