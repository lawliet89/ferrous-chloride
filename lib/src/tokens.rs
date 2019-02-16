//! Tokens and literals

pub mod strings;

use std::str::FromStr;

use nom::types::CompleteStr;
use nom::{alt, complete, do_parse, map, map_res, named, one_of, opt, recognize, tag};

// Parse Literal Values
// NUMBER  // 12345
// 	FLOAT   // 123.45
// 	BOOL    // true,false
// 	STRING  // "abc"
// HEREDOC // <<FOO\nbar\nFOO

/// Parsed Integer Literal
#[allow(dead_code)]
struct Integer<'a> {
    pub sign: Option<char>,
    pub digits: CompleteStr<'a>,
}

impl<'a> Integer<'a> {
    #[allow(dead_code)]
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
            >>  digits: recognize!(nom::digit)
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::ResultUtils;

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
}
