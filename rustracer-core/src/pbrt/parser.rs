use std::ops::RangeFrom;

use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::space1,
    combinator::{map, map_res, value},
    error::{Error, ErrorKind},
    multi::many1,
    sequence::{delimited, pair, tuple},
    Finish, IResult,
};

use super::lexer::{Token, Tokens};
use crate::api::{Api, Array, ParamListEntry, ParamType};
use crate::paramset::ParamSet;

pub fn parse<'input, 'api, A: Api>(
    input: Tokens<'input>,
    api: &'api A,
) -> IResult<Tokens<'input>, ()> {
    let (rest, _) = many1(alt((
        map_res(token(Token::ATTRIBUTEBEGIN), |_| api.attribute_begin()),
        map_res(token(Token::ATTRIBUTEEND), |_| api.attribute_end()),
        map_res(token(Token::TRANSFORMBEGIN), |_| api.transform_begin()),
        map_res(token(Token::TRANSFORMEND), |_| api.transform_end()),
        map_res(pair(token(Token::OBJECTBEGIN), string_), |(_, name)| {
            api.object_begin(name)
        }),
        map_res(token(Token::OBJECTEND), |_| api.object_end()),
    )))(input)?;

    Ok((rest, ()))
}

/*
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

fn param_list(input: Tokens<'_>) -> IResult<Tokens<'_>, ParamSet> {
    map(many1(param_list_entry), |x| {
        let mut ps = ParamSet::default();
        ps.init(x);
        ps
    })(input)
}

fn param_type(input: &str) -> IResult<&str, ParamType> {
    alt((
        value(ParamType::Int, tag("integer")),
        value(ParamType::Bool, tag("bool")),
        value(ParamType::Float, tag("float")),
        value(ParamType::Point2, tag("point2")),
        value(ParamType::Vector2, tag("vector2")),
        value(ParamType::Point3, tag("point3")),
        value(ParamType::Vector3, tag("vector3")),
        value(ParamType::Point3, tag("point")),
        value(ParamType::Vector3, tag("vector")),
        value(ParamType::Normal, tag("normal")),
        value(ParamType::Rgb, tag("color")),
        value(ParamType::Rgb, tag("rgb")),
        value(ParamType::Xyz, tag("xyz")),
        value(ParamType::Blackbody, tag("blackbody")),
        value(ParamType::Spectrum, tag("spectrum")),
        value(ParamType::String, tag("string")),
        value(ParamType::Texture, tag("texture")),
    ))(input)
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

fn param_list_entry_header(input: Tokens<'_>) -> IResult<Tokens<'_>, (ParamType, String)> {
    let (rest, s) = string_(input)?;

    let (name, (param_type, _)) = match tuple((param_type, space1))(&s).finish() {
        Ok(res) => res,
        Err(e) => return Err(nom::Err::Error(Error::new(rest, e.code))),
    };

    Ok((rest, (param_type, name.to_string())))
}

fn param_list_entry(input: Tokens<'_>) -> IResult<Tokens<'_>, ParamListEntry> {
    map(
        tuple((param_list_entry_header, array)),
        |(header, array)| ParamListEntry::new(header.0, header.1, array),
    )(input)
}

fn array(input: Tokens<'_>) -> IResult<Tokens<'_>, Array> {
    alt((
        map(string_array, Array::StrArray),
        map(num_array, Array::NumArray),
    ))(input)
}

fn string_array(input: Tokens<'_>) -> IResult<Tokens<'_>, Vec<String>> {
    alt((
        delimited(token(Token::LBRACK), many1(string_), token(Token::RBRACK)),
        map(string_, |x| vec![x]),
    ))(input)
}

fn num_array(input: Tokens<'_>) -> IResult<Tokens<'_>, Vec<f32>> {
    alt((
        delimited(token(Token::LBRACK), many1(num), token(Token::RBRACK)),
        map(num, |x| vec![x]),
    ))(input)
}

fn num(input: Tokens<'_>) -> IResult<Tokens<'_>, f32> {
    let (i, ret) = take(1usize)(dbg!(input))?;

    match ret[0] {
        Token::NUMBER(n) => Ok((i, n)),
        _ => Err(nom::Err::Error(Error::new(i, ErrorKind::Digit))),
    }
}

fn string_(input: Tokens<'_>) -> IResult<Tokens<'_>, String> {
    let (i, ret) = take(1usize)(input)?;
    match ret[0] {
        Token::STR(ref s) => Ok((i, s.clone())),
        _ => Err(nom::Err::Error(Error::new(i, ErrorKind::Tag))),
    }
}

fn token<'a, I>(t: Token) -> impl Fn(I) -> IResult<I, Token>
where
    I: nom::Slice<RangeFrom<usize>> + nom::InputIter<Item = &'a Token>,
{
    move |i: I| match (i).iter_elements().next().map(|tt| {
        let b = *tt == t;
        (&t, b)
    }) {
        Some((t, true)) => Ok((i.slice(1..), t.clone())),
        _ => Err(nom::Err::Error(Error::new(i, nom::error::ErrorKind::Char))),
    }
}

// #[test]
// fn test_parse() {
//     let tokens = vec![Tokens::ATTRIBUTEBEGIN, Tokens::ATTRIBUTEEND];
//     let api = ::api::RealApi::default();
//     api.init().unwrap();
//     parse(&tokens[..], &api).unwrap();
// }

#[cfg(test)]
mod tests {
    use nom::{error::ParseError, Parser};

    use super::*;

    fn test_parse<'a, O, E>(input: &'a [Token], mut p: impl Parser<Tokens<'a>, O, E>, v: O)
    where
        O: std::fmt::Debug + PartialEq,
        E: ParseError<Tokens<'a>> + std::fmt::Debug,
    {
        assert_eq!(p.parse(Tokens::new(input)).unwrap().1, v);
    }

    #[test]
    fn test_array() {
        let p = vec![
            Token::LBRACK,
            Token::NUMBER(50.0),
            Token::NUMBER(12.0),
            Token::RBRACK,
        ];

        test_parse(&p, array, Array::NumArray(vec![50.0, 12.0]))
    }

    #[test]
    fn test_num_array() {
        let p = vec![Token::LBRACK, Token::NUMBER(50.0), Token::RBRACK];

        test_parse(&p, num_array, vec![50.0])
    }

    #[test]
    fn test_param_list_entry() {
        let p = vec![
            Token::STR("float fov".to_owned()),
            Token::LBRACK,
            Token::NUMBER(50.0),
            Token::RBRACK,
        ];

        param_list_entry(Tokens::new(&p[..])).unwrap();
    }

    #[test]
    fn test_param_type() {
        let p = "float";

        let res = param_type(&p).unwrap();
        assert_eq!(res, ("", ParamType::Float));

        let q = "floatxxx";
        let res = param_type(&q).unwrap();
        assert_eq!(res, ("xxx", ParamType::Float));
    }

    #[test]
    fn test_param_list_entry_header() {
        let p = vec![Token::STR("float fov".to_owned())];

        test_parse(
            &p,
            param_list_entry_header,
            (ParamType::Float, "fov".to_string()),
        );
    }
}
