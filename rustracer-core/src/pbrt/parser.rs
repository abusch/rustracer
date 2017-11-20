use combine::{eof, value, satisfy_map, token, between, many, many1, try, Parser, Stream,
              ParseError};
use combine::char::{string, spaces};
use combine::primitives::Error;
use failure::*;

use api::{Api, ParamType, ParamListEntry, Array};
use paramset::ParamSet;
use super::lexer::Tokens;

pub fn parse<I: Stream<Item = Tokens>, A: Api>
    (input: I,
     api: &A)
     -> ::std::result::Result<(Vec<()>, I), ParseError<I>> {
    // TODO remove all the error conversions once https://github.com/brson/error-chain/issues/134 is fixed
    let accelerator = (token(Tokens::ACCELERATOR), string_(), param_list())
        .and_then(|(_, typ, mut params)| {
                      api.accelerator(typ, &mut params)
                          .map_err(|e| e.compat())
                  });
    let attribute_begin =
        token(Tokens::ATTRIBUTEBEGIN).and_then(|_| api.attribute_begin().map_err(|e| e.compat()));
    let attribute_end =
        token(Tokens::ATTRIBUTEEND).and_then(|_| api.attribute_end().map_err(|e| e.compat()));
    let transform_begin =
        token(Tokens::TRANSFORMBEGIN).and_then(|_| api.transform_begin().map_err(|e| e.compat()));
    let transform_end =
        token(Tokens::TRANSFORMEND).and_then(|_| api.transform_end().map_err(|e| e.compat()));
    let world_begin =
        token(Tokens::WORLDBEGIN).and_then(|_| api.world_begin().map_err(|e| e.compat()));
    let world_end = token(Tokens::WORLDEND).and_then(|_| api.world_end().map_err(|e| e.compat()));
    let look_at =
        (token(Tokens::LOOKAT), num(), num(), num(), num(), num(), num(), num(), num(), num())
            .and_then(|(_, ex, ey, ez, lx, ly, lz, ux, uy, uz)| {
                          api.look_at(ex, ey, ez, lx, ly, lz, ux, uy, uz)
                              .map_err(|e| e.compat())
                      });
    let coord_sys_transform =
        (token(Tokens::COORDSYSTRANSFORM), string_()).and_then(|(_, name)| {
                                                                   api.coord_sys_transform(name)
                                                                       .map_err(|e| e.compat())
                                                               });
    let camera =
        (token(Tokens::CAMERA), string_(), param_list()).and_then(|(_, name, mut params)| {
                                                                      api.camera(name, &mut params)
                                                                          .map_err(|e| {
                                                                                       e.compat()
                                                                                   })
                                                                  });
    let film =
        (token(Tokens::FILM), string_(), param_list()).and_then(|(_, name, mut params)| {
                                                                    api.film(name, &mut params)
                                                                        .map_err(|e| e.compat())
                                                                });
    let integrator = (token(Tokens::INTEGRATOR), string_(), param_list())
        .and_then(|(_, name, mut params)| {
                      api.integrator(name, &mut params).map_err(|e| e.compat())
                  });
    let arealightsource = (token(Tokens::AREALIGHTSOURCE), string_(), param_list())
        .and_then(|(_, name, mut params)| {
                      api.arealightsource(name, &mut params)
                          .map_err(|e| e.compat())
                  });
    let lightsource = (token(Tokens::LIGHTSOURCE), string_(), param_list())
        .and_then(|(_, name, mut params)| {
                      api.lightsource(name, &mut params)
                          .map_err(|e| e.compat())
                  });
    let material =
        (token(Tokens::MATERIAL), string_(), param_list()).and_then(|(_, name, mut params)| {
                                                                        api.material(name,
                                                                                     &mut params)
                                                                            .map_err(|e| {
                                                                                         e.compat()
                                                                                     })
                                                                    });
    let make_named_material = (token(Tokens::MAKENAMEDMATERIAL), string_(), param_list())
        .and_then(|(_, name, mut params)| {
                      api.make_named_material(name, &mut params)
                          .map_err(|e| e.compat())
                  });
    let named_material =
        (token(Tokens::NAMEDMATERIAL), string_()).and_then(|(_, name)| {
                                                               api.named_material(name)
                                                                   .map_err(|e| e.compat())
                                                           });
    let sampler = (token(Tokens::SAMPLER), string_(), param_list()).and_then(|(_,
                                                                               name,
                                                                               mut params)| {
                                                                                 api.sampler(name,
                                                                                   &mut params)
            .map_err(|e| e.compat())
                                                                             });
    let shape =
        (token(Tokens::SHAPE), string_(), param_list()).and_then(|(_, name, mut params)| {
                                                                     api.shape(name, &mut params)
                                                                         .map_err(|e| {
                                                                                      e.compat()
                                                                                  })
                                                                 });

    let reverse_orientation =
        token(Tokens::REVERSEORIENTATION).and_then(|_| {
                                                       api.reverse_orientation()
            .map_err(|e| e.compat())
                                                   });
    let filter =
        (token(Tokens::PIXELFILTER), string_(), param_list())
            .and_then(|(_, name, mut params)| api.pixel_filter(name, &mut params)
            .map_err(|e| e.compat())
                      );
    let scale =
        (token(Tokens::SCALE), num(), num(), num()).and_then(|(_, sx, sy, sz)| {
                                                                 api.scale(sx, sy, sz)
                                                                     .map_err(|e| e.compat())
                                                             });
    let rotate =
        (token(Tokens::ROTATE), num(), num(), num(), num()).and_then(|(_, angle, dx, dy, dz)| {
            api.rotate(angle, dx, dy, dz).map_err(|e| e.compat())
        });

    let texture = (token(Tokens::TEXTURE), string_(), string_(), string_(), param_list())
        .and_then(|(_, name, typ, texname, mut params)| {
                      api.texture(name, typ, texname, &mut params)
                          .map_err(|e| e.compat())
                  });
    let transform = (token::<I>(Tokens::TRANSFORM), num_array()).and_then(|(_, nums)| {
        api.transform(nums[0],
                       nums[1],
                       nums[2],
                       nums[3],
                       nums[4],
                       nums[5],
                       nums[6],
                       nums[7],
                       nums[8],
                       nums[9],
                       nums[10],
                       nums[11],
                       nums[12],
                       nums[13],
                       nums[14],
                       nums[15])
            .map_err(|e| e.compat())
    });

    let translate =
        (token(Tokens::TRANSLATE), num(), num(), num()).and_then(|(_, dx, dy, dz)| {
                                                                     api.translate(dx, dy, dz)
                                                                         .map_err(|e| {
                                                                                      e.compat()
                                                                                  })
                                                                 });

    let parsers = many1::<Vec<_>, _>(choice!(try(accelerator),
                                             try(attribute_begin),
                                             try(attribute_end),
                                             try(transform_begin),
                                             try(transform_end),
                                             try(world_begin),
                                             try(world_end),
                                             try(look_at),
                                             try(coord_sys_transform),
                                             try(camera),
                                             try(film),
                                             try(filter),
                                             try(integrator),
                                             try(arealightsource),
                                             try(lightsource),
                                             try(material),
                                             try(texture),
                                             try(make_named_material),
                                             try(named_material),
                                             try(sampler),
                                             try(shape),
                                             try(reverse_orientation),
                                             try(scale),
                                             try(rotate),
                                             try(translate),
                                             try(transform)));
    (parsers, eof()).map(|(res, _)| res).parse(input)
}



fn param_list<'a, I: Stream<Item = Tokens> + 'a>
    ()
    -> Box<Parser<Input = I, Output = ParamSet> + 'a>
{
    many(param_list_entry())
        .map(|x| {
                 let mut ps = ParamSet::default();
                 ps.init(x);
                 ps
             })
        .boxed()
}

fn param_type<'a, I: Stream<Item = char> + 'a>
    ()
    -> Box<Parser<Input = I, Output = ParamType> + 'a>
{
    choice!(try(string("integer").with(value(ParamType::Int))),
            try(string("bool").with(value(ParamType::Bool))),
            try(string("float").with(value(ParamType::Float))),
            try(string("point2").with(value(ParamType::Point2))),
            try(string("vector2").with(value(ParamType::Vector2))),
            try(string("point3").with(value(ParamType::Point3))),
            try(string("vector3").with(value(ParamType::Vector3))),
            try(string("point").with(value(ParamType::Point3))),
            try(string("vector").with(value(ParamType::Vector3))),
            try(string("normal").with(value(ParamType::Normal))),
            try(string("color").with(value(ParamType::Rgb))),
            try(string("rgb").with(value(ParamType::Rgb))),
            try(string("xyz").with(value(ParamType::Xyz))),
            try(string("blackbody").with(value(ParamType::Blackbody))),
            try(string("spectrum").with(value(ParamType::Spectrum))),
            try(string("string").with(value(ParamType::String))),
            try(string("texture").with(value(ParamType::Texture))))
            .boxed()
}

fn param_list_entry_header<'a, I: Stream<Item = Tokens> + 'a>
    ()
    -> Box<Parser<Input = I, Output = (ParamType, String)> + 'a>
{
    string_()
        .and_then(|s| match param_type().skip(spaces()).parse(&s[..]) {
                      Ok((t, n)) => Ok((t, n.to_owned())),
                      Err(error) => {
                          Err(Error::Message(format!("Invalid param list entry: {}", error).into()))
                      }
                  })
        .boxed()
}

fn param_list_entry<'a, I: Stream<Item = Tokens> + 'a>
    ()
    -> Box<Parser<Input = I, Output = ParamListEntry> + 'a>
{
    (param_list_entry_header(), array())
        .map(|(header, array)| ParamListEntry::new(header.0, header.1, array))
        .boxed()
}

fn array<'a, I: Stream<Item = Tokens> + 'a>() -> Box<Parser<Input = I, Output = Array> + 'a> {
    choice!(try(string_array().map(Array::StrArray)),
            try(num_array().map(Array::NumArray)))
            .boxed()
}

fn string_array<'a, I: Stream<Item = Tokens> + 'a>
    ()
    -> Box<Parser<Input = I, Output = Vec<String>> + 'a>
{
    choice!(try(between(token(Tokens::LBRACK),
                        token(Tokens::RBRACK),
                        many1::<Vec<_>, _>(string_()))),
            try(string_().map(|x| vec![x])))
            .boxed()
}

fn num_array<'a, I: Stream<Item = Tokens> + 'a>
    ()
    -> Box<Parser<Input = I, Output = Vec<f32>> + 'a>
{
    choice!(try(between(token(Tokens::LBRACK),
                        token(Tokens::RBRACK),
                        many1::<Vec<_>, _>(num()))),
            try(num().map(|x| vec![x])))
            .boxed()
}

fn num<'a, I: Stream<Item = Tokens> + 'a>() -> Box<Parser<Input = I, Output = f32> + 'a> {
    satisfy_map(|t| match t {
                    Tokens::NUMBER(n) => Some(n),
                    _ => None,
                })
            .boxed()
}

fn string_<'a, I: Stream<Item = Tokens> + 'a>() -> Box<Parser<Input = I, Output = String> + 'a> {
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
    let p = vec![Tokens::LBRACK,
                 Tokens::NUMBER(50.0),
                 Tokens::NUMBER(12.0),
                 Tokens::RBRACK];
    let foo = vec![];

    assert_eq!(array().parse(&p[..]),
               Ok((Array::NumArray(vec![50.0, 12.0]), &foo[..])));
}

#[test]
fn test_num_array() {
    let p = vec![Tokens::LBRACK, Tokens::NUMBER(50.0), Tokens::RBRACK];
    let foo = vec![];

    assert_eq!(num_array().parse(&p[..]), Ok((vec![50.0], &foo[..])));
}

#[test]
fn test_param_list_entry() {
    let p = vec![Tokens::STR("float fov".to_owned()),
                 Tokens::LBRACK,
                 Tokens::NUMBER(50.0),
                 Tokens::RBRACK];

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

    assert_eq!(param_list_entry_header().parse(&p[..]),
               Ok(((ParamType::Float, "fov".to_owned()), &foo[..])));
}
