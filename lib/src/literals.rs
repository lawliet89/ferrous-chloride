use std::str::FromStr;

use nom::types::CompleteStr;
use nom::{complete, do_parse, map_res, named, one_of, opt, recognize};

pub use crate::errors::Error;

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

named!(integer(CompleteStr) -> i64,
    map_res!(
        do_parse!(
            sign: opt!(complete!(one_of!("+-")))
            >>  digits: recognize!(nom::digit)
            >> (Integer { sign, digits })
        ),
        |integer: Integer| integer.to_integer::<i64>()
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
}
