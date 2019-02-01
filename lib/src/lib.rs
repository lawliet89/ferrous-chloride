pub mod errors;

use nom::{complete, map, map_res, named, one_of, opt};

pub use errors::Error;

// Parse Literal Values
// NUMBER  // 12345
// 	FLOAT   // 123.45
// 	BOOL    // true,false
// 	STRING  // "abc"
// HEREDOC // <<FOO\nbar\nFOO

#[derive(Debug, Copy, Clone)]
pub enum NumericSign {
    Positive,
    Negative,
}

impl NumericSign {
    pub fn from_option(input: Option<char>) -> Result<Self, Error> {
        match input {
            None | Some('+') => Ok(NumericSign::Positive),
            Some('-') => Ok(NumericSign::Negative),
            Some(others @ _) => Err(Error::UnknownNumericSign(others)),
        }
    }
}

named!(numeric_sign(&[u8]) -> NumericSign,
    map_res!(
        opt!(complete!(one_of!("+-"))),
        |s| NumericSign::from_option(s)
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_sign() {
        dbg!(numeric_sign("+".as_bytes()));
        dbg!(numeric_sign("-".as_bytes()));
        dbg!(numeric_sign("".as_bytes()));
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
