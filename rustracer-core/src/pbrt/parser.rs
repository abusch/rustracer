use std::ops::RangeFrom;

use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::space1,
    combinator::{all_consuming, map, map_res, value},
    error::{Error, ErrorKind},
    multi::{many0, many1},
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
    let accelerator = map_res(
        tuple((token(Token::ACCELERATOR), string_, param_list)),
        |(_, typ, params)| api.accelerator(typ, &params),
    );
    let attribute_begin = map_res(token(Token::ATTRIBUTEBEGIN), |_| api.attribute_begin());
    let attribute_end = map_res(token(Token::ATTRIBUTEEND), |_| api.attribute_end());
    let transform_begin = map_res(token(Token::TRANSFORMBEGIN), |_| api.transform_begin());
    let transform_end = map_res(token(Token::TRANSFORMEND), |_| api.transform_end());
    let object_begin = map_res(pair(token(Token::OBJECTBEGIN), string_), |(_, name)| {
        api.object_begin(name)
    });
    let object_end = map_res(token(Token::OBJECTEND), |_| api.object_end());
    let object_instance = map_res(pair(token(Token::OBJECTINSTANCE), string_), |(_, name)| {
        api.object_instance(name)
    });
    let world_begin = map_res(token(Token::WORLDBEGIN), |_| api.world_begin());
    let world_end = map_res(token(Token::WORLDEND), |_| api.world_end());
    let look_at = map_res(
        tuple((
            token(Token::LOOKAT),
            num,
            num,
            num,
            num,
            num,
            num,
            num,
            num,
            num,
        )),
        |(_, ex, ey, ez, lx, ly, lz, ux, uy, uz)| api.look_at(ex, ey, ez, lx, ly, lz, ux, uy, uz),
    );
    let coordinate_system = map_res(
        pair(token(Token::COORDINATESYSTEM), string_),
        |(_, name)| api.coordinate_system(name),
    );
    let coord_sys_transform = map_res(
        pair(token(Token::COORDSYSTRANSFORM), string_),
        |(_, name)| api.coord_sys_transform(name),
    );
    let camera = map_res(
        tuple((token(Token::CAMERA), string_, param_list)),
        |(_, typ, params)| api.camera(typ, &params),
    );
    let film = map_res(
        tuple((token(Token::FILM), string_, param_list)),
        |(_, typ, params)| api.film(typ, &params),
    );
    // TODO include
    /* let include = (token(Tokens::INCLUDE), string_()).and_then(|(_, name)| {
        info!("Parsing included file: {}", name);
        super::tokenize_file(&name)
            .and_then(|tokens| {
                parse(&tokens[..], api)
                    .map(|_| ())
                    .map_err(|e| format_err!("Failed to parse included file: {:?}", e))
            })
            .map_err(|e| Error::Other(e.into()))
    }); */
    let integrator = map_res(
        tuple((token(Token::INTEGRATOR), string_, param_list)),
        |(_, typ, params)| api.integrator(typ, &params),
    );
    let arealightsource = map_res(
        tuple((token(Token::AREALIGHTSOURCE), string_, param_list)),
        |(_, typ, params)| api.arealightsource(typ, &params),
    );
    let lightsource = map_res(
        tuple((token(Token::LIGHTSOURCE), string_, param_list)),
        |(_, typ, params)| api.lightsource(typ, &params),
    );
    let material = map_res(
        tuple((token(Token::MATERIAL), string_, param_list)),
        |(_, typ, params)| api.material(typ, &params),
    );
    let make_named_material = map_res(
        tuple((token(Token::MAKENAMEDMATERIAL), string_, param_list)),
        |(_, typ, params)| api.make_named_material(typ, &params),
    );
    let named_material = map_res(pair(token(Token::NAMEDMATERIAL), string_), |(_, name)| {
        api.named_material(name)
    });
    let sampler = map_res(
        tuple((token(Token::SAMPLER), string_, param_list)),
        |(_, typ, params)| api.sampler(typ, &params),
    );
    let shape = map_res(
        tuple((token(Token::SHAPE), string_, param_list)),
        |(_, typ, params)| api.shape(typ, &params),
    );
    let reverse_orientation = map_res(token(Token::REVERSEORIENTATION), |_| {
        api.reverse_orientation()
    });
    let filter = map_res(
        tuple((token(Token::PIXELFILTER), string_, param_list)),
        |(_, typ, params)| api.pixel_filter(typ, &params),
    );
    let scale = map_res(
        tuple((token(Token::SCALE), num, num, num)),
        |(_, sx, sy, sz)| api.scale(sx, sy, sz),
    );
    let rotate = map_res(
        tuple((token(Token::ROTATE), num, num, num, num)),
        |(_, angle, dx, dy, dz)| api.rotate(angle, dx, dy, dz),
    );
    let texture = map_res(
        tuple((token(Token::TEXTURE), string_, string_, string_, param_list)),
        |(_, name, typ, texname, params)| api.texture(name, typ, texname, &params),
    );
    let concat_transform = map_res(
        pair(token(Token::CONCATTRANSFORM), num_array),
        |(_, nums)| {
            api.concat_transform(
                nums[0], nums[1], nums[2], nums[3], nums[4], nums[5], nums[6], nums[7], nums[8],
                nums[9], nums[10], nums[11], nums[12], nums[13], nums[14], nums[15],
            )
        },
    );
    let transform = map_res(pair(token(Token::TRANSFORM), num_array), |(_, nums)| {
        api.transform(
            nums[0], nums[1], nums[2], nums[3], nums[4], nums[5], nums[6], nums[7], nums[8],
            nums[9], nums[10], nums[11], nums[12], nums[13], nums[14], nums[15],
        )
    });
    let translate = map_res(
        tuple((token(Token::TRANSLATE), num, num, num)),
        |(_, dx, dy, dz)| api.translate(dx, dy, dz),
    );

    let (rest, _) = all_consuming(many1(alt((
        accelerator,
        attribute_begin,
        attribute_end,
        transform_begin,
        transform_end,
        object_begin,
        object_end,
        object_instance,
        world_begin,
        world_end,
        look_at,
        coordinate_system,
        coord_sys_transform,
        camera,
        film,
        integrator,
        arealightsource,
        lightsource,
        material,
        make_named_material,
        alt((
            named_material,
            sampler,
            shape,
            reverse_orientation,
            filter,
            scale,
            rotate,
            texture,
            concat_transform,
            transform,
            translate,
        )),
    ))))(input)?;

    Ok((rest, ()))
}

fn param_list(input: Tokens<'_>) -> IResult<Tokens<'_>, ParamSet> {
    map(many0(param_list_entry), |x| {
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
    let (i, ret) = take(1usize)(input)?;

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
