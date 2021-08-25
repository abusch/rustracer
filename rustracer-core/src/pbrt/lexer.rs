use std::{
    fmt,
    iter::Enumerate,
    ops::{Index, Range, RangeFrom},
    slice::Iter,
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alphanumeric1, char, line_ending, multispace0, none_of, not_line_ending,
    },
    combinator::{all_consuming, map, map_res, value},
    multi::{many0, many1},
    number::complete::float,
    sequence::{delimited, preceded, terminated},
    IResult, InputIter, InputLength, InputTake, Needed, Slice,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
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

impl Token {
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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct Tokens<'a> {
    tokens: &'a [Token],
    start: usize,
    end: usize,
}

impl<'a> Tokens<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            start: 0,
            end: tokens.len(),
        }
    }
}

impl<'a> Index<usize> for Tokens<'a> {
    type Output = Token;

    fn index(&self, index: usize) -> &Self::Output {
        &self.tokens[index]
    }
}

impl<'a> InputTake for Tokens<'a> {
    fn take(&self, count: usize) -> Self {
        Tokens::new(&self.tokens[0..count])
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (first, second) = self.tokens.split_at(count);
        (Tokens::new(second), Tokens::new(first))
    }
}

impl<'a> InputLength for Tokens<'a> {
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

impl<'a> InputIter for Tokens<'a> {
    type Item = &'a Token;

    type Iter = Enumerate<Iter<'a, Token>>;

    type IterElem = Iter<'a, Token>;

    fn iter_indices(&self) -> Self::Iter {
        self.tokens.iter().enumerate()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.tokens.iter()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.tokens.iter().position(predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.tokens.len() >= count {
            Ok(count)
        } else {
            Err(Needed::Unknown)
        }
    }
}

impl<'a> Slice<Range<usize>> for Tokens<'a> {
    fn slice(&self, range: Range<usize>) -> Self {
        {
            let start = self.start + range.start;
            let end = self.start + range.end;
            let tokens: &'a [Token] = &self.tokens[range];
            Self { tokens, start, end }
        }
    }
}

impl<'a> Slice<RangeFrom<usize>> for Tokens<'a> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        self.slice(range.start..self.end - self.start)
    }
}

pub fn tokenize(input: &str) -> IResult<&str, Vec<Token>> {
    all_consuming(terminated(
        many1(alt((
            preceded(multispace0, keyword),
            preceded(multispace0, float_parser),
            preceded(multispace0, string_parser),
            preceded(multispace0, comment_parser),
        ))),
        multispace0,
    ))(input)
}

pub fn keyword(input: &str) -> IResult<&str, Token> {
    map_res(
        preceded(multispace0, alt((alphanumeric1, tag("["), tag("]")))),
        |s| match s {
            "Accelerator" => Ok(Token::ACCELERATOR),
            "ActiveTransform" => Ok(Token::ACTIVETRANSFORM),
            "All" => Ok(Token::ALL),
            "AreaLightSource" => Ok(Token::AREALIGHTSOURCE),
            "AttributeBegin" => Ok(Token::ATTRIBUTEBEGIN),
            "AttributeEnd" => Ok(Token::ATTRIBUTEEND),
            "Camera" => Ok(Token::CAMERA),
            "ConcatTransform" => Ok(Token::CONCATTRANSFORM),
            "CoordinateSystem" => Ok(Token::COORDINATESYSTEM),
            "CoordSysTransform" => Ok(Token::COORDSYSTRANSFORM),
            "EndTime" => Ok(Token::ENDTIME),
            "Film" => Ok(Token::FILM),
            "Identity" => Ok(Token::IDENTITY),
            "Include" => Ok(Token::INCLUDE),
            "LightSource" => Ok(Token::LIGHTSOURCE),
            "LookAt" => Ok(Token::LOOKAT),
            "MakeNamedMedium" => Ok(Token::MAKENAMEDMEDIUM),
            "MakeNamedMaterial" => Ok(Token::MAKENAMEDMATERIAL),
            "Material" => Ok(Token::MATERIAL),
            "MediumInterface" => Ok(Token::MEDIUMINTERFACE),
            "NamedMaterial" => Ok(Token::NAMEDMATERIAL),
            "ObjectBegin" => Ok(Token::OBJECTBEGIN),
            "ObjectEnd" => Ok(Token::OBJECTEND),
            "ObjectInstance" => Ok(Token::OBJECTINSTANCE),
            "PixelFilter" => Ok(Token::PIXELFILTER),
            "ReverseOrientation" => Ok(Token::REVERSEORIENTATION),
            "Rotate" => Ok(Token::ROTATE),
            "Sampler" => Ok(Token::SAMPLER),
            "Scale" => Ok(Token::SCALE),
            "Shape" => Ok(Token::SHAPE),
            "StartTime" => Ok(Token::STARTTIME),
            "Integrator" => Ok(Token::INTEGRATOR),
            "Texture" => Ok(Token::TEXTURE),
            "TransformBegin" => Ok(Token::TRANSFORMBEGIN),
            "TransformEnd" => Ok(Token::TRANSFORMEND),
            "TransformTimes" => Ok(Token::TRANSFORMTIMES),
            "Transform" => Ok(Token::TRANSFORM),
            "Translate" => Ok(Token::TRANSLATE),
            "WorldBegin" => Ok(Token::WORLDBEGIN),
            "WorldEnd" => Ok(Token::WORLDEND),
            "[" => Ok(Token::LBRACK),
            "]" => Ok(Token::RBRACK),
            _ => Err(format!("Invalid keyword found: {}", s)),
        },
    )(input)
}

pub fn float_parser(input: &str) -> IResult<&str, Token> {
    map(float, Token::NUMBER)(input)
}

pub fn string_parser(input: &str) -> IResult<&str, Token> {
    map(delimited(char('"'), many0(none_of("\"")), char('"')), |s| {
        Token::STR(s.into_iter().collect())
    })(input)
}

pub fn comment_parser(input: &str) -> IResult<&str, Token> {
    value(
        Token::COMMENT,
        delimited(char('#'), not_line_ending, line_ending),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let scene = r##"
LookAt 0 0 5 0 0 0 0 1 0
Camera "perspective" "float fov" [50]


Film "image" "integer xresolution" [800] "integer yresolution" [600]
    "string filename" "test-whitted.tga"

Integrator "whitted"

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
        let (rest, _tokens) = tokenize(scene).unwrap();
        assert_eq!(rest, "");
    }

    #[test]
    fn test_float_parser() {
        let s = "-1.23e2";
        let result = float_parser(s);
        assert_eq!(result, Ok(("", Token::NUMBER(-123.0))))
    }

    #[test]
    fn test_string_parser() {
        let s = "\"this is a string\"";
        let result = string_parser(s);
        assert_eq!(result, Ok(("", Token::STR("this is a string".to_string()))))
    }

    #[test]
    fn test_keyword_parser() {
        let s = "Accelerator";
        assert_eq!(keyword(&s), Ok(("", Token::ACCELERATOR)));

        let s = "[";
        assert_eq!(keyword(&s), Ok(("", Token::LBRACK)));
    }

    #[test]
    fn test_comment_parser() {
        let s = "#foo\n";
        assert_eq!(comment_parser(&s), Ok(("", Token::COMMENT)));
    }
}
