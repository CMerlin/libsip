use nom::{
    IResult,
    branch::alt,
    error::ParseError,
    combinator::{opt, map_res},
    bytes::complete::take_while,
    character::{
        *,
        complete::char
    }
};

use crate::{parse::*, uri::parse_uri, Uri};

use std::{collections::HashMap, fmt};

/// Header Value for Named Headers,
/// e.g. From, To, Contact
#[derive(Debug, PartialEq, Clone)]
pub struct NamedHeader {
    pub display_name: Option<String>,
    pub uri: Uri,
    pub params: HashMap<String, String>,
}

impl NamedHeader {
    pub fn new(uri: Uri) -> NamedHeader {
        NamedHeader {
            display_name: None,
            params: HashMap::new(),
            uri,
        }
    }

    pub fn name<S: Into<String>>(mut self, name: S) -> NamedHeader {
        self.display_name = Some(name.into());
        self
    }
}

impl fmt::Display for NamedHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = &self.display_name {
            if name.contains(' ') {
                write!(f, "\"{}\" <{}>", name, self.uri)?;
            } else if name.is_empty() {
                write!(f, "\"\" <{}>", self.uri)?;
            } else {
                write!(f, "{} <{}>", name, self.uri)?;
            }
        } else {
            write!(f, "{}", self.uri)?;
        }
        for (key, value) in (&self.params).iter() {
            write!(f, ";{}={}", key, value)?;
        }
        Ok(())
    }
}

/// Parse a single NamedHeader param value.
pub fn parse_named_field_param<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], (String, String), E> {
    let (input, _) = char(';')(input)?;
    let (input, key) = map_res(take_while(is_alphabetic), slice_to_string::<E>)(input)?;
    let (input, _) = char('=')(input)?;
    let (input, value) = map_res(take_while(is_alphanumeric), slice_to_string::<E>)(input)?;
    Ok((input, (key, value)))
}

/// Parse the name part of the NamedHeader.
pub fn parse_name<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], String, E> {
    Ok(
        alt::<_, _, E, _>((parse_quoted_string::<E>, parse_unquoted_string::<E>))(input)?
    )
}

/// Parse a stream of text that is not quoted. This will stop
/// at the first ' ' char the input contains.
pub fn parse_unquoted_string<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], String, E> {
    let (input, string_data) = map_res(take_while(is_alphabetic), slice_to_string::<E>)(input)?;
    let (input, _) = char(' ')(input)?;
    Ok((input, string_data))
}

/// Parse a single NamedHeader value.
pub fn parse_named_field_value<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], (Option<String>, Uri), E> {
    let (input, name) = opt(parse_name)(input)?;
    let (input, _) = opt(take_while(is_space))(input)?;
    let (input, _) = opt(char('<'))(input)?;
    let (input, value) = parse_uri(input)?;
    let (input, _) = opt(char('>'))(input)?;
    Ok((input, (name, value)))
}

/// Parse as many valid named field params as the input contains.
pub fn parse_named_field_params<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> ParserResult<HashMap<String, String>, E> {
    let mut map = HashMap::new();
    let mut input = input;
    while let Ok((data, (key, value))) = parse_named_field_param::<E>(input) {
        map.insert(key, value);
        input = data;
    }
    Ok((input, map))
}
