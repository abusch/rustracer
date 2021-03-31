use anyhow::{Result, format_err};
use nom::{IResult, branch::alt, bytes::complete::tag, combinator::map_res, multi::many1, sequence::pair};
use log::info;

use super::lexer::Tokens;
use crate::api::{Api, Array, ParamListEntry, ParamType};
use crate::paramset::ParamSet;

pub fn parse<'input, 'api, A: Api>(input: &'input [Tokens], api: &'api A) -> IResult<&'input [Tokens], ()> {
    many1(
        alt((
            map_res(tag(Tokens::ATTRIBUTEBEGIN), |_| api.attribute_begin()),
            map_res(tag(Tokens::ATTRIBUTEEND), |_| api.attribute_end()),
            map_res(tag(Tokens::TRANSFORMBEGIN), |_| api.transform_begin()),
            map_res(tag(Tokens::TRANSFORMEND), |_| api.transform_end()),
            map_res(pair(tag(Tokens::OBJECTBEGIN), string_), |(_, name)| api.object_begin(name)),
            map_res(tag(Tokens::OBJECTEND), |_| api.object_end()),
        ))
    )(input)
}

/**
pub fn parse<I, A: Api>(
    input: I,
    api: &A,
) -> Result<(Vec<()>, I), easy::ParseError<I>>
where
  I: Stream<Token = Tokens, Error = easy::ParseError<I>>,
  I::Position: Default,
  I::Range: PartialEq,
  I::Error: ParseError<
    I::Token,
    I::Range,
    I::Position,
    StreamError = Error<I::Token, I::Range>
  >
{
    let accelerator =
        (token(Tokens::ACCELERATOR), string_(), param_list()).and_then(|(_, typ, params)| {
            api.accelerator(typ, &params)
                .map_err(|e| Error::Other(e.into()))
        });
    let attribute_begin = token(Tokens::ATTRIBUTEBEGIN)
        .and_then(|_| api.attribute_begin().map_err(|e| Error::Other(e.into())));
    let attribute_end = token(Tokens::ATTRIBUTEEND)
        .and_then(|_| api.attribute_end().map_err(|e| Error::Other(e.into())));
    let transform_begin = token(Tokens::TRANSFORMBEGIN)
        .and_then(|_| api.transform_begin().map_err(|e| Error::Other(e.into())));
    let transform_end = token(Tokens::TRANSFORMEND)
        .and_then(|_| api.transform_end().map_err(|e| Error::Other(e.into())));
    let object_begin = (token(Tokens::OBJECTBEGIN), string_())
        .and_then(|(_, name)| api.object_begin(name).map_err(|e| Error::Other(e.into())));
    let object_end =
        token(Tokens::OBJECTEND).and_then(|_| api.object_end().map_err(|e| Error::Other(e.into())));
    let object_instance = (token(Tokens::OBJECTINSTANCE), string_()).and_then(|(_, name)| {
        api.object_instance(name)
            .map_err(|e| Error::Other(e.into()))
    });
    let world_begin = token(Tokens::WORLDBEGIN)
        .and_then(|_| api.world_begin().map_err(|e| Error::Other(e.into())));
    let world_end =
        token(Tokens::WORLDEND).and_then(|_| api.world_end().map_err(|e| Error::Other(e.into())));
    let look_at = (
        token(Tokens::LOOKAT),
        num(),
        num(),
        num(),
        num(),
        num(),
        num(),
        num(),
        num(),
        num(),
    )
        .and_then(|(_, ex, ey, ez, lx, ly, lz, ux, uy, uz)| {
            api.look_at(ex, ey, ez, lx, ly, lz, ux, uy, uz)
                .map_err(|e| Error::Other(e.into()))
        });
    let coordinate_system = (token(Tokens::COORDINATESYSTEM), string_()).and_then(|(_, name)| {
        api.coordinate_system(name)
            .map_err(|e| Error::Other(e.into()))
    });
    let coord_sys_transform =
        (token(Tokens::COORDSYSTRANSFORM), string_()).and_then(|(_, name)| {
            api.coord_sys_transform(name)
                .map_err(|e| Error::Other(e.into()))
        });
    let camera = (token(Tokens::CAMERA), string_(), param_list()).and_then(|(_, name, params)| {
        api.camera(name, &params)
            .map_err(|e| Error::Other(e.into()))
    });
    let film = (token(Tokens::FILM), string_(), param_list())
        .and_then(|(_, name, params)| api.film(name, &params).map_err(|e| Error::Other(e.into())));
    let include = (token(Tokens::INCLUDE), string_()).and_then(|(_, name)| {
        info!("Parsing included file: {}", name);
        super::tokenize_file(&name)
            .and_then(|tokens| {
                parse(&tokens[..], api)
                    .map(|_| ())
                    .map_err(|e| format_err!("Failed to parse included file: {:?}", e))
            })
            .map_err(|e| Error::Other(e.into()))
    });
    let integrator =
        (token(Tokens::INTEGRATOR), string_(), param_list()).and_then(|(_, name, params)| {
            api.integrator(name, &params)
                .map_err(|e| Error::Other(e.into()))
        });
    let arealightsource =
        (token(Tokens::AREALIGHTSOURCE), string_(), param_list()).and_then(|(_, name, params)| {
            api.arealightsource(name, &params)
                .map_err(|e| Error::Other(e.into()))
        });
    let lightsource =
        (token(Tokens::LIGHTSOURCE), string_(), param_list()).and_then(|(_, name, params)| {
            api.lightsource(name, &params)
                .map_err(|e| Error::Other(e.into()))
        });
    let material =
        (token(Tokens::MATERIAL), string_(), param_list()).and_then(|(_, name, params)| {
            api.material(name, &params)
                .map_err(|e| Error::Other(e.into()))
        });
    let make_named_material = (token(Tokens::MAKENAMEDMATERIAL), string_(), param_list()).and_then(
        |(_, name, params)| {
            api.make_named_material(name, &params)
                .map_err(|e| Error::Other(e.into()))
        },
    );
    let named_material = (token(Tokens::NAMEDMATERIAL), string_())
        .and_then(|(_, name)| api.named_material(name).map_err(|e| Error::Other(e.into())));
    let sampler =
        (token(Tokens::SAMPLER), string_(), param_list()).and_then(|(_, name, params)| {
            api.sampler(name, &params)
                .map_err(|e| Error::Other(e.into()))
        });
    let shape = (token(Tokens::SHAPE), string_(), param_list())
        .and_then(|(_, name, params)| api.shape(name, &params).map_err(|e| Error::Other(e.into())));

    let reverse_orientation = token(Tokens::REVERSEORIENTATION).and_then(|_| {
        api.reverse_orientation()
            .map_err(|e| Error::Other(e.into()))
    });
    let filter =
        (token(Tokens::PIXELFILTER), string_(), param_list()).and_then(|(_, name, params)| {
            api.pixel_filter(name, &params)
                .map_err(|e| Error::Other(e.into()))
        });
    let scale = (token(Tokens::SCALE), num(), num(), num())
        .and_then(|(_, sx, sy, sz)| api.scale(sx, sy, sz).map_err(|e| Error::Other(e.into())));
    let rotate =
        (token(Tokens::ROTATE), num(), num(), num(), num()).and_then(|(_, angle, dx, dy, dz)| {
            api.rotate(angle, dx, dy, dz)
                .map_err(|e| Error::Other(e.into()))
        });

    let texture = (
        token(Tokens::TEXTURE),
        string_(),
        string_(),
        string_(),
        param_list(),
    )
        .and_then(|(_, name, typ, texname, params)| {
            api.texture(name, typ, texname, &params)
                .map_err(|e| Error::Other(e.into()))
        });
    let concat_transform =
        (token::<I>(Tokens::CONCATTRANSFORM), num_array()).and_then(|(_, nums)| {
            api.concat_transform(
                nums[0], nums[1], nums[2], nums[3], nums[4], nums[5], nums[6], nums[7], nums[8],
                nums[9], nums[10], nums[11], nums[12], nums[13], nums[14], nums[15],
            )
            .map_err(|e| Error::Other(e.into()))
        });

    let transform = (token::<I>(Tokens::TRANSFORM), num_array()).and_then(|(_, nums)| {
        api.transform(
            nums[0], nums[1], nums[2], nums[3], nums[4], nums[5], nums[6], nums[7], nums[8],
            nums[9], nums[10], nums[11], nums[12], nums[13], nums[14], nums[15],
        )
        .map_err(|e| Error::Other(e.into()))
    });

    let translate = (token(Tokens::TRANSLATE), num(), num(), num()).and_then(|(_, dx, dy, dz)| {
        api.translate(dx, dy, dz)
            .map_err(|e| Error::Other(e.into()))
    });

    let parsers = many1::<Vec<_>, _, _>(choice!(
        attempt(accelerator),
        attempt(attribute_begin),
        attempt(attribute_end),
        attempt(transform_begin),
        attempt(transform_end),
        attempt(object_begin),
        attempt(object_end),
        attempt(object_instance),
        attempt(world_begin),
        attempt(world_end),
        attempt(look_at),
        attempt(coordinate_system),
        attempt(coord_sys_transform),
        attempt(camera),
        attempt(film),
        attempt(filter),
        attempt(include),
        attempt(integrator),
        attempt(arealightsource),
        attempt(lightsource),
        attempt(material),
        attempt(texture),
        attempt(make_named_material),
        attempt(named_material),
        attempt(sampler),
        attempt(shape),
        attempt(reverse_orientation),
        attempt(scale),
        attempt(rotate),
        attempt(translate),
        attempt(concat_transform),
        attempt(transform)
    ));
    (parsers, eof()).map(|(res, _)| res).parse(input)
}
*/

fn param_list<I>(
) -> impl Parser<I, Output = ParamSet>
where
  I: Stream<Token = Tokens>,
  I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    many(param_list_entry())
        .map(|x| {
            let mut ps = ParamSet::default();
            ps.init(x);
            ps
        })
}

fn param_type<I>(
) -> impl Parser<I, Output = ParamType>
where
  I: Stream<Token = char>,
  I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice!(
        attempt(string("integer").with(value(ParamType::Int))),
        attempt(string("bool").with(value(ParamType::Bool))),
        attempt(string("float").with(value(ParamType::Float))),
        attempt(string("point2").with(value(ParamType::Point2))),
        attempt(string("vector2").with(value(ParamType::Vector2))),
        attempt(string("point3").with(value(ParamType::Point3))),
        attempt(string("vector3").with(value(ParamType::Vector3))),
        attempt(string("point").with(value(ParamType::Point3))),
        attempt(string("vector").with(value(ParamType::Vector3))),
        attempt(string("normal").with(value(ParamType::Normal))),
        attempt(string("color").with(value(ParamType::Rgb))),
        attempt(string("rgb").with(value(ParamType::Rgb))),
        attempt(string("xyz").with(value(ParamType::Xyz))),
        attempt(string("blackbody").with(value(ParamType::Blackbody))),
        attempt(string("spectrum").with(value(ParamType::Spectrum))),
        attempt(string("string").with(value(ParamType::String))),
        attempt(string("texture").with(value(ParamType::Texture)))
    )
}

fn get_param_type(t: &str) -> Option<ParamType> {
  match t {
    "integer" => Some(ParamType::Int),
    "bool" => Some(ParamType::Bool),
    "float" => Some(ParamType::Float),
    "point2" => Some(ParamType::Point2),
    "vector2" => Some(ParamType::Vector2),
    "point3" => Some(ParamType::Point3),
    "vector3" => Some(ParamType::Vector3),
    "point" => Some(ParamType::Point3),
    "vector" => Some(ParamType::Vector3),
    "normal" => Some(ParamType::Normal),
    "color" => Some(ParamType::Rgb),
    "rgb" => Some(ParamType::Rgb),
    "xyz" => Some(ParamType::Xyz),
    "blackbody" => Some(ParamType::Blackbody),
    "spectrum" => Some(ParamType::Spectrum),
    "string" => Some(ParamType::String),
    "texture" => Some(ParamType::Texture),
    _ => None,
  }
}

fn param_list_entry_header<I>(
) -> impl Parser<I, Output = (ParamType, String)>
where
  I: Stream<Token = Tokens>,
  I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    string_().and_then(|s| get_param_type(s).ok_or_else(||  )

        .and_then(|s| match param_type().skip(spaces()).parse(&s[..]) {
            Ok((t, n)) => Ok((t, n.to_owned())),
            Err(error) => Err(easy::Error::Message(
                format!("Invalid param list entry: {}", error).into(),
            )),
        })
}

fn param_list_entry<I>(
) -> impl Parser<I, Output = ParamListEntry>
where
  I: Stream<Token = Tokens>,
  I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (param_list_entry_header(), array())
        .map(|(header, array)| ParamListEntry::new(header.0, header.1, array))
}

fn array<I>() -> impl Parser<I, Output = Array>
where
  I: Stream<Token = Tokens>,
  I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice!(
        attempt(string_array().map(Array::StrArray)),
        attempt(num_array().map(Array::NumArray))
    )
}

fn string_array<I>(
) -> impl Parser<I, Output = Vec<String>>
where
  I: Stream<Token = Tokens>,
  I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice!(
        attempt(between(
            token(Tokens::LBRACK),
            token(Tokens::RBRACK),
            many1::<Vec<_>, _, _>(string_())
        )),
        attempt(string_().map(|x| vec![x]))
    )
}

fn num_array<I>(
) -> impl Parser<I, Output = Vec<f32>>
where
  I: Stream<Token = Tokens>,
  I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice!(
        attempt(between(
            token(Tokens::LBRACK),
            token(Tokens::RBRACK),
            many1::<Vec<_>, _, _>(num())
        )),
        attempt(num().map(|x| vec![x]))
    )
}

fn num<I>() -> impl Parser<I, Output = f32>
where
  I: Stream<Token = Tokens>,
  I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    satisfy_map(|t| match t {
        Tokens::NUMBER(n) => Some(n),
        _ => None,
    })
}

fn string_(input: &[Tokens]) -> IResult<&[Tokens], ()> {
    satisfy_map(|t| match t {
        Tokens::STR(s) => Some(s),
        _ => None,
    })
}

// #[test]
// fn test_parse() {
//     let tokens = vec![Tokens::ATTRIBUTEBEGIN, Tokens::ATTRIBUTEEND];
//     let api = ::api::RealApi::default();
//     api.init().unwrap();
//     parse(&tokens[..], &api).unwrap();
// }

#[test]
fn test_array() {
    let p = vec![
        Tokens::LBRACK,
        Tokens::NUMBER(50.0),
        Tokens::NUMBER(12.0),
        Tokens::RBRACK,
    ];
    let foo = vec![];

    assert_eq!(
        array().parse(&p[..]),
        Ok((Array::NumArray(vec![50.0, 12.0]), &foo[..]))
    );
}

#[test]
fn test_num_array() {
    let p = vec![Tokens::LBRACK, Tokens::NUMBER(50.0), Tokens::RBRACK];
    let foo = vec![];

    assert_eq!(num_array().parse(&p[..]), Ok((vec![50.0], &foo[..])));
}

#[test]
fn test_param_list_entry() {
    let p = vec![
        Tokens::STR("float fov".to_owned()),
        Tokens::LBRACK,
        Tokens::NUMBER(50.0),
        Tokens::RBRACK,
    ];

    param_list_entry().parse(&p[..]).unwrap();
}

#[test]
fn test_param_type() {
    let p = "float";
    assert_eq!(param_type().parse(&p[..]), Ok((ParamType::Float, "")));

    let q = "floatxxx";
    assert_eq!(param_type().parse(&q[..]), Ok((ParamType::Float, "xxx")));
}

#[test]
fn test_param_list_entry_header() {
    let p = vec![Tokens::STR("float fov".to_owned())];
    let foo = vec![];

    assert_eq!(
        param_list_entry_header().parse(&p[..]),
        Ok(((ParamType::Float, "fov".to_owned()), &foo[..]))
    );
}
