//! Tokens and literals

pub mod strings;

pub use strings::string;

use std::borrow::Cow;
use std::ops::Deref;
use std::str::FromStr;

use nom::types::CompleteStr;
use nom::{
    alt, alt_complete, call, complete, do_parse, map, map_res, named, one_of, opt, recognize, tag,
    verify,
};

/// Parsed Integer Literal
struct Integer<'a> {
    pub sign: Option<char>,
    pub digits: CompleteStr<'a>,
}

impl<'a> Integer<'a> {
    pub(crate) fn to_integer<T>(&self) -> Result<T, std::num::ParseIntError>
    where
        T: FromStr<Err = std::num::ParseIntError>,
    {
        match self.sign {
            Some(sign) => T::from_str(format!("{}{}", sign, &self.digits).as_str()),
            None => T::from_str(&self.digits),
        }
    }
}

/// Parse an interger literal
named!(pub integer(CompleteStr) -> i64,
    map_res!(
        do_parse!(
            sign: opt!(complete!(one_of!("+-")))
            >> digits: recognize!(nom::digit)
            >> (Integer { sign, digits })
        ),
        |integer: Integer| integer.to_integer::<i64>()
    )
);

/// Parse a float literal
named!(pub float(CompleteStr) -> f64, complete!(nom::double));

/// Parse a boolean literal
named!(pub boolean(CompleteStr) -> bool,
    map!(
        alt!(
            tag!("true")
            | tag!("false")
        ),
        |s| s.as_ref() == "true"    // Can only ever be "true" or "false"
    )
);

/// Parse an identifier
named!(pub identifier(CompleteStr) -> &str,
    do_parse!(
        identifier: verify!(
            call!(crate::utils::while_predicate1, |c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.'),
            |s: CompleteStr| {
                let first = s.chars().nth(0);
                match first {
                    None => false,
                    Some(c) => c.is_alphabetic() || c == '_'
                }
            }
        )
        >> (identifier.0)
    )
);

/// A "key" in a map
#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub enum Key<'a> {
    Identifier(Cow<'a, str>),
    String(Cow<'a, str>),
}

impl<'a> Key<'a> {
    pub fn unwrap(self) -> Cow<'a, str> {
        match self {
            Key::Identifier(s) => s,
            Key::String(s) => s,
        }
    }
}

impl<'a> Deref for Key<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Key::Identifier(s) => s.deref(),
            Key::String(s) => s.deref(),
        }
    }
}

/// Parse a "key" for a map
named!(
    pub key(&str) -> Key,
    alt_complete!(
        call!(crate::utils::wrap_str(identifier)) => { |s| Key::Identifier(Cow::Borrowed(s)) }
        | string => { |s| Key::String(Cow::Owned(s)) }
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::ResultUtilsString;

    #[test]
    fn integers_are_parsed_correctly() {
        assert_eq!(integer(CompleteStr("12345")).unwrap_output(), 12345);
        assert_eq!(integer(CompleteStr("+12345")).unwrap_output(), 12345);
        assert_eq!(integer(CompleteStr("-12345")).unwrap_output(), -12345);
    }

    #[test]
    fn floats_are_parsed_correctly() {
        assert_eq!(float(CompleteStr("12.34")).unwrap_output(), 12.34);
        assert_eq!(float(CompleteStr("+12.34")).unwrap_output(), 12.34);
        assert_eq!(float(CompleteStr("-12.34")).unwrap_output(), -12.34);
    }

    #[test]
    fn booleans_are_parsed_correctly() {
        assert_eq!(boolean(CompleteStr("true")).unwrap_output(), true);
        assert_eq!(boolean(CompleteStr("false")).unwrap_output(), false);
    }

    #[test]
    fn identifiers_are_parsed_correctly() {
        let test_cases = [
            ("abcd123", "abcd123"),
            ("_abc", "_abc"),
            ("藏_①", "藏_①"),
        ];

        for (input, expected) in test_cases.into_iter() {
            println!("Testing {}", input);
            assert_eq!(identifier(CompleteStr(input)).unwrap_output(), *expected);
        }
    }

    #[test]
    fn incorrect_identifiers_are_not_accepted() {
        let test_cases = ["1abc", "①_is_some_number"];

        for input in test_cases.into_iter() {
            println!("Testing {}", input);
            assert!(identifier(CompleteStr(input)).is_err());
        }
    }

    #[test]
    fn keys_are_parsed_correctly() {
        let test_cases = [
            ("abcd123", Key::Identifier(From::from("abcd123"))),
            ("_abc", Key::Identifier(From::from("_abc"))),
            ("藏_①", Key::Identifier(From::from("藏_①"))),
            (r#""123""#, Key::String(From::from("123"))),
            (r#""a/b/c""#, Key::String(From::from("a/b/c"))),
        ];

        for (input, expected) in test_cases.into_iter() {
            println!("Testing {}", input);
            assert_eq!(key(input).unwrap_output(), *expected);
        }
    }
}
