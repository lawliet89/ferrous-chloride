use std::str::FromStr;

use nom::recognize_float;
use nom::types::CompleteStr;
use nom::{call, flat_map, named, parse_to};

/// Parsed Number
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

impl<'a> From<Number> for crate::Value<'a> {
    fn from(number: Number) -> Self {
        use crate::Value;

        match number {
            Number::Integer(i) => Value::Integer(i),
            Number::Float(f) => Value::Float(f),
        }
    }
}

impl FromStr for Number {
    type Err = crate::errors::InternalKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // XXX: Can we do better?
        // Try to parse integer, failing that float, failing that we give up
        match s.parse() {
            Ok(i) => Ok(Number::Integer(i)),
            Err(_) => match s.parse() {
                Ok(f) => Ok(Number::Float(f)),
                Err(e) => Err(e)?,
            },
        }
    }
}

impl From<i64> for Number {
    fn from(i: i64) -> Self {
        Number::Integer(i)
    }
}

impl From<f64> for Number {
    fn from(f: f64) -> Self {
        Number::Float(f)
    }
}

/// Parse Number
named!(pub number(CompleteStr) -> Number,
    flat_map!(call!(recognize_float), parse_to!(Number))
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::ResultUtilsString;

    #[test]
    fn integers_are_parsed_correctly() {
        assert_eq!(
            number(CompleteStr("12345")).unwrap_output(),
            From::from(12345)
        );
        assert_eq!(
            number(CompleteStr("+12345")).unwrap_output(),
            From::from(12345)
        );
        assert_eq!(
            number(CompleteStr("-12345")).unwrap_output(),
            From::from(-12345)
        );
    }

    #[test]
    fn floats_are_parsed_correctly() {
        assert_eq!(
            number(CompleteStr("12.34")).unwrap_output(),
            From::from(12.34)
        );
        assert_eq!(
            number(CompleteStr("+12.34")).unwrap_output(),
            From::from(12.34)
        );
        assert_eq!(
            number(CompleteStr("-12.34")).unwrap_output(),
            From::from(-12.34)
        );
    }
}
