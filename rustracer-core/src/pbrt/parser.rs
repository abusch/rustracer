use anyhow::*;
use combine::char::{spaces, string};
use combine::primitives::Error;
use combine::{
    between, choice, eof, many, many1, r#try, satisfy_map, token, value, ParseError, Parser, Stream,
};
use log::info;

use super::lexer::Tokens;
use crate::api::{Api, Array, ParamListEntry, ParamType};
use crate::paramset::ParamSet;

pub fn parse<I: Stream<Item = Tokens>, A: Api>(
    input: I,
    api: &A,
) -> ::std::result::Result<(Vec<()>, I), ParseError<I>> {
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

    let parsers = many1::<Vec<_>, _>(choice!(
        r#try(accelerator),
        r#try(attribute_begin),
        r#try(attribute_end),
        r#try(transform_begin),
        r#try(transform_end),
        r#try(object_begin),
        r#try(object_end),
        r#try(object_instance),
        r#try(world_begin),
        r#try(world_end),
        r#try(look_at),
        r#try(coordinate_system),
        r#try(coord_sys_transform),
        r#try(camera),
        r#try(film),
        r#try(filter),
        r#try(include),
        r#try(integrator),
        r#try(arealightsource),
        r#try(lightsource),
        r#try(material),
        r#try(texture),
        r#try(make_named_material),
        r#try(named_material),
        r#try(sampler),
        r#try(shape),
        r#try(reverse_orientation),
        r#try(scale),
        r#try(rotate),
        r#try(translate),
        r#try(concat_transform),
        r#try(transform)
    ));
    (parsers, eof()).map(|(res, _)| res).parse(input)
}

fn param_list<'a, I: Stream<Item = Tokens> + 'a>(
) -> Box<dyn Parser<Input = I, Output = ParamSet> + 'a> {
    many(param_list_entry())
        .map(|x| {
            let mut ps = ParamSet::default();
            ps.init(x);
            ps
        })
        .boxed()
}

fn param_type<'a, I: Stream<Item = char> + 'a>(
) -> Box<dyn Parser<Input = I, Output = ParamType> + 'a> {
    choice!(
        r#try(string("integer").with(value(ParamType::Int))),
        r#try(string("bool").with(value(ParamType::Bool))),
        r#try(string("float").with(value(ParamType::Float))),
        r#try(string("point2").with(value(ParamType::Point2))),
        r#try(string("vector2").with(value(ParamType::Vector2))),
        r#try(string("point3").with(value(ParamType::Point3))),
        r#try(string("vector3").with(value(ParamType::Vector3))),
        r#try(string("point").with(value(ParamType::Point3))),
        r#try(string("vector").with(value(ParamType::Vector3))),
        r#try(string("normal").with(value(ParamType::Normal))),
        r#try(string("color").with(value(ParamType::Rgb))),
        r#try(string("rgb").with(value(ParamType::Rgb))),
        r#try(string("xyz").with(value(ParamType::Xyz))),
        r#try(string("blackbody").with(value(ParamType::Blackbody))),
        r#try(string("spectrum").with(value(ParamType::Spectrum))),
        r#try(string("string").with(value(ParamType::String))),
        r#try(string("texture").with(value(ParamType::Texture)))
    )
    .boxed()
}

fn param_list_entry_header<'a, I: Stream<Item = Tokens> + 'a>(
) -> Box<dyn Parser<Input = I, Output = (ParamType, String)> + 'a> {
    string_()
        .and_then(|s| match param_type().skip(spaces()).parse(&s[..]) {
            Ok((t, n)) => Ok((t, n.to_owned())),
            Err(error) => Err(Error::Message(
                format!("Invalid param list entry: {}", error).into(),
            )),
        })
        .boxed()
}

fn param_list_entry<'a, I: Stream<Item = Tokens> + 'a>(
) -> Box<dyn Parser<Input = I, Output = ParamListEntry> + 'a> {
    (param_list_entry_header(), array())
        .map(|(header, array)| ParamListEntry::new(header.0, header.1, array))
        .boxed()
}

fn array<'a, I: Stream<Item = Tokens> + 'a>() -> Box<dyn Parser<Input = I, Output = Array> + 'a> {
    choice!(
        r#try(string_array().map(Array::StrArray)),
        r#try(num_array().map(Array::NumArray))
    )
    .boxed()
}

fn string_array<'a, I: Stream<Item = Tokens> + 'a>(
) -> Box<dyn Parser<Input = I, Output = Vec<String>> + 'a> {
    choice!(
        r#try(between(
            token(Tokens::LBRACK),
            token(Tokens::RBRACK),
            many1::<Vec<_>, _>(string_())
        )),
        r#try(string_().map(|x| vec![x]))
    )
    .boxed()
}

fn num_array<'a, I: Stream<Item = Tokens> + 'a>(
) -> Box<dyn Parser<Input = I, Output = Vec<f32>> + 'a> {
    choice!(
        r#try(between(
            token(Tokens::LBRACK),
            token(Tokens::RBRACK),
            many1::<Vec<_>, _>(num())
        )),
        r#try(num().map(|x| vec![x]))
    )
    .boxed()
}

fn num<'a, I: Stream<Item = Tokens> + 'a>() -> Box<dyn Parser<Input = I, Output = f32> + 'a> {
    satisfy_map(|t| match t {
        Tokens::NUMBER(n) => Some(n),
        _ => None,
    })
    .boxed()
}

fn string_<'a, I: Stream<Item = Tokens> + 'a>() -> Box<dyn Parser<Input = I, Output = String> + 'a>
{
    satisfy_map(|t| match t {
        Tokens::STR(s) => Some(s),
        _ => None,
    })
    .boxed()
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
