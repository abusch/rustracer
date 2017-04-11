use combine::{satisfy, skip_many, none_of, token, between, optional, many, many1, choice, try,
              Parser, Stream, ParseResult};
use combine::char::{string, spaces, digit, char};

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
    SAMPLE,
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

pub fn tokenize<I: Stream<Item = char>>(input: I) -> ParseResult<Vec<Tokens>, I> {

    // parsers for keywords
    let mut parsers = vec![token_parser("Accelerator", Tokens::ACCELERATOR),
                           token_parser("ActiveTransform", Tokens::ACTIVETRANSFORM),
                           token_parser("All", Tokens::ALL),
                           token_parser("AreaLightSource", Tokens::AREALIGHTSOURCE),
                           token_parser("AttributeBegin", Tokens::ATTRIBUTEBEGIN),
                           token_parser("AttributeEnd", Tokens::ATTRIBUTEEND),
                           token_parser("Camera", Tokens::CAMERA),
                           token_parser("ConcatTransform", Tokens::CONCATTRANSFORM),
                           token_parser("CoordinateSystem", Tokens::COORDINATESYSTEM),
                           token_parser("CoordSysTransform", Tokens::COORDSYSTRANSFORM),
                           token_parser("EndTime", Tokens::ENDTIME),
                           token_parser("Film", Tokens::FILM),
                           token_parser("Identity", Tokens::IDENTITY),
                           token_parser("Include", Tokens::INCLUDE),
                           token_parser("LightSource", Tokens::LIGHTSOURCE),
                           token_parser("LookAt", Tokens::LOOKAT),
                           token_parser("MakeNamedMedium", Tokens::MAKENAMEDMEDIUM),
                           token_parser("MakeNamedMaterial", Tokens::MAKENAMEDMATERIAL),
                           token_parser("Material", Tokens::MATERIAL),
                           token_parser("MediumInterface", Tokens::MEDIUMINTERFACE),
                           token_parser("NamedMaterial", Tokens::NAMEDMATERIAL),
                           token_parser("ObjectBegin", Tokens::OBJECTBEGIN),
                           token_parser("ObjectEnd", Tokens::OBJECTEND),
                           token_parser("ObjectInstance", Tokens::OBJECTINSTANCE),
                           token_parser("PixelFilter", Tokens::PIXELFILTER),
                           token_parser("ReverseOrientation", Tokens::REVERSEORIENTATION),
                           token_parser("Rotate", Tokens::ROTATE),
                           token_parser("Sample", Tokens::SAMPLE),
                           token_parser("Scale", Tokens::SCALE),
                           token_parser("Shape", Tokens::SHAPE),
                           token_parser("StartTime", Tokens::STARTTIME),
                           token_parser("Integrator", Tokens::INTEGRATOR),
                           token_parser("Texture", Tokens::TEXTURE),
                           token_parser("TransformBegin", Tokens::TRANSFORMBEGIN),
                           token_parser("TransformEnd", Tokens::TRANSFORMEND),
                           token_parser("TransformTimes", Tokens::TRANSFORMTIMES),
                           token_parser("Transform", Tokens::TRANSFORM),
                           token_parser("Translate", Tokens::TRANSLATE),
                           token_parser("WorldBegin", Tokens::WORLDBEGIN),
                           token_parser("WorldEnd", Tokens::WORLDEND),
                           token_parser("[", Tokens::LBRACK),
                           token_parser("]", Tokens::RBRACK)]
            .into_iter()
            .map(|parser| try(parser))
            .collect::<Vec<_>>();

    // Add parsers from num, strings, etc...
    parsers.push(try(float_parser()));
    parsers.push(try(string_parser()));
    parsers.push(try(comment_parser()));

    spaces()
        .with(many::<Vec<_>, _>(choice(parsers)))
        .parse_stream(input)
}

fn token_parser<'a, I: Stream<Item = char> + 'a>
    (s: &'static str,
     t: Tokens)
     -> Box<Parser<Input = I, Output = Tokens> + 'a> {
    string(s).skip(spaces()).map(move |_| t.clone()).boxed()
}

fn float_parser<'a, I: Stream<Item = char> + 'a>
    ()
    -> Box<Parser<Input = I, Output = Tokens> + 'a>
{

    (optional(char('-').or(char('+'))),
     many1::<Vec<_>, _>(digit()),
     optional(char('.').with(many::<Vec<_>, _>(digit()))),
     optional(char('e').or(char('E')).with(many1::<Vec<_>, _>(digit()))))
            .skip(spaces())
            .and_then(|(sign, int_part, frac_part, mantissa)| {
                let mut buf = String::new();
                if let Some(s) = sign {
                    buf.push(s);
                }
                for c in int_part {
                    buf.push(c);
                }
                if let Some(frac) = frac_part {
                    buf.push('.');
                    for c in frac {
                        buf.push(c);
                    }
                }
                if let Some(mant) = mantissa {
                    buf.push('e');
                    for c in mant {
                        buf.push(c);
                    }
                }

                buf.parse::<f32>().map(Tokens::NUMBER)
            })
            .boxed()
}

fn string_parser<'a, I: Stream<Item = char> + 'a>
    ()
    -> Box<Parser<Input = I, Output = Tokens> + 'a>
{
    between(token('"'),
            token('"'),
            many::<Vec<_>, _>(none_of("\"".chars())))
            .skip(spaces())
            .map(|chars| {
                     let mut buf = String::new();
                     for c in chars {
                         buf.push(c);
                     }
                     Tokens::STR(buf)
                 })
            .boxed()
}

fn comment_parser<'a, I: Stream<Item = char> + 'a>
    ()
    -> Box<Parser<Input = I, Output = Tokens> + 'a>
{
    token('#')
        .and(skip_many(satisfy(|c| c != '\n')))
        .skip(spaces())
        .map(|_| Tokens::COMMENT)
        .boxed()
}

#[test]
fn test_tokenize() {
    use combine::primitives::Consumed;
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
    let (tokens, rest) = result.unwrap();
    assert_eq!(rest, Consumed::Consumed(""));
}

#[test]
fn test_string_parser() {
    let s = "\"this is a string\"";
    let result = string_parser().parse(s);
    assert_eq!(result,
               Ok((Tokens::STR("this is a string".to_string()), "")))
}
