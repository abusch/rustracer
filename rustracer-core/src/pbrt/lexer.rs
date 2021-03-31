use std::fmt;

use nom::{IResult, branch::alt, character::complete::{
        alphanumeric1, char, line_ending, multispace0, none_of, not_line_ending,
    }, combinator::{map, map_res, value}, multi::{many0, many1}, number::complete::float, sequence::{delimited, preceded}};

#[derive(Clone, Debug, PartialEq)]
pub enum Tokens {
    ACCELERATOR,
    ACTIVETRANSFORM,
    ALL,
    AREALIGHTSOURCE,
    ATTRIBUTEBEGIN,
    ATTRIBUTEEND,
    CAMERA,
    CONCATTRANSFORM,
    COORDINATESYSTEM,
    COORDSYSTRANSFORM,
    ENDTIME,
    FILM,
    IDENTITY,
    INCLUDE,
    LIGHTSOURCE,
    LOOKAT,
    MAKENAMEDMEDIUM,
    MAKENAMEDMATERIAL,
    MATERIAL,
    MEDIUMINTERFACE,
    NAMEDMATERIAL,
    OBJECTBEGIN,
    OBJECTEND,
    OBJECTINSTANCE,
    PIXELFILTER,
    REVERSEORIENTATION,
    ROTATE,
    SAMPLER,
    SCALE,
    SHAPE,
    STARTTIME,
    INTEGRATOR,
    TEXTURE,
    TRANSFORMBEGIN,
    TRANSFORMEND,
    TRANSFORMTIMES,
    TRANSFORM,
    TRANSLATE,
    WORLDBEGIN,
    WORLDEND,
    STR(String),
    NUMBER(f32),
    LBRACK,
    RBRACK,
    COMMENT,
}

impl Tokens {

    pub fn as_str(&self) -> Option<&String> {
        if let Self::STR(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_number(&self) -> Option<&f32> {
        if let Self::NUMBER(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl fmt::Display for Tokens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn tokenize(input: &str) -> IResult<&str, Vec<Tokens>> {
    many1(
        alt((
            preceded(multispace0, keyword),
            preceded(multispace0, float_parser),
            preceded(multispace0, string_parser),
            preceded(multispace0, comment_parser),
        ))
    )(input)
}

pub fn keyword(input: &str) -> IResult<&str, Tokens> {
    map_res(preceded(multispace0, alphanumeric1), |s| match s {
        "Accelerator" => Ok(Tokens::ACCELERATOR),
        "ActiveTransform" => Ok(Tokens::ACTIVETRANSFORM),
        "All" => Ok(Tokens::ALL),
        "AreaLightSource" => Ok(Tokens::AREALIGHTSOURCE),
        "AttributeBegin" => Ok(Tokens::ATTRIBUTEBEGIN),
        "AttributeEnd" => Ok(Tokens::ATTRIBUTEEND),
        "Camera" => Ok(Tokens::CAMERA),
        "ConcatTransform" => Ok(Tokens::CONCATTRANSFORM),
        "CoordinateSystem" => Ok(Tokens::COORDINATESYSTEM),
        "CoordSysTransform" => Ok(Tokens::COORDSYSTRANSFORM),
        "EndTime" => Ok(Tokens::ENDTIME),
        "Film" => Ok(Tokens::FILM),
        "Identity" => Ok(Tokens::IDENTITY),
        "Include" => Ok(Tokens::INCLUDE),
        "LightSource" => Ok(Tokens::LIGHTSOURCE),
        "LookAt" => Ok(Tokens::LOOKAT),
        "MakeNamedMedium" => Ok(Tokens::MAKENAMEDMEDIUM),
        "MakeNamedMaterial" => Ok(Tokens::MAKENAMEDMATERIAL),
        "Material" => Ok(Tokens::MATERIAL),
        "MediumInterface" => Ok(Tokens::MEDIUMINTERFACE),
        "NamedMaterial" => Ok(Tokens::NAMEDMATERIAL),
        "ObjectBegin" => Ok(Tokens::OBJECTBEGIN),
        "ObjectEnd" => Ok(Tokens::OBJECTEND),
        "ObjectInstance" => Ok(Tokens::OBJECTINSTANCE),
        "PixelFilter" => Ok(Tokens::PIXELFILTER),
        "ReverseOrientation" => Ok(Tokens::REVERSEORIENTATION),
        "Rotate" => Ok(Tokens::ROTATE),
        "Sampler" => Ok(Tokens::SAMPLER),
        "Scale" => Ok(Tokens::SCALE),
        "Shape" => Ok(Tokens::SHAPE),
        "StartTime" => Ok(Tokens::STARTTIME),
        "Integrator" => Ok(Tokens::INTEGRATOR),
        "Texture" => Ok(Tokens::TEXTURE),
        "TransformBegin" => Ok(Tokens::TRANSFORMBEGIN),
        "TransformEnd" => Ok(Tokens::TRANSFORMEND),
        "TransformTimes" => Ok(Tokens::TRANSFORMTIMES),
        "Transform" => Ok(Tokens::TRANSFORM),
        "Translate" => Ok(Tokens::TRANSLATE),
        "WorldBegin" => Ok(Tokens::WORLDBEGIN),
        "WorldEnd" => Ok(Tokens::WORLDEND),
        "[" => Ok(Tokens::LBRACK),
        "]" => Ok(Tokens::RBRACK),
        _ => Err(format!("Invalid keyword found: {}", s)),
    })(input)
}

pub fn float_parser(input: &str) -> IResult<&str, Tokens> {
    map(float, Tokens::NUMBER)(input)
}

pub fn string_parser(input: &str) -> IResult<&str, Tokens> {
    map(delimited(char('"'), many0(none_of("\"")), char('"')), |s| Tokens::STR(s.into_iter().collect()))(input)
}

pub fn comment_parser(input: &str) -> IResult<&str, Tokens> {
    value(
        Tokens::COMMENT,
        delimited(char('#'), many0(not_line_ending), line_ending),
    )(input)
}

#[test]
fn test_tokenize() {
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
    let result = tokenize(scene);
    assert!(result.is_ok());
    let (rest, _tokens) = result.unwrap();
    assert_eq!(rest, "");
}

#[test]
fn test_string_parser() {
    let s = "\"this is a string\"";
    let result = string_parser(s);
    assert_eq!(
        result,
        Ok(("", Tokens::STR("this is a string".to_string())))
    )
}
