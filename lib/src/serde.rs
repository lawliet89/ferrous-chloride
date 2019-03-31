pub mod de;

use failure::{self, Fail};
use std::fmt::Display;
use std::ops::Deref;

/// Error type for serialization or deserialization
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "HCL parse error: {}", _0)]
    ParseError(#[cause] crate::Error),
    #[fail(display = "Input was not completely consumed during deserialization.")]
    TrailingCharacters,
    #[fail(display = "Expected an integer but found a float.")]
    UnexpectedFloat,
    #[fail(display = "Overflow when trying to convert to {}", _0)]
    Overflow(&'static str),
    #[fail(display = "{}", _0)]
    Custom(String),
}

impl From<crate::Error> for Error {
    fn from(e: crate::Error) -> Self {
        Error::ParseError(e)
    }
}

impl<I> From<nom::Err<I, u32>> for Error
where
    I: nom::AsBytes + AsRef<str> + std::fmt::Debug,
{
    fn from(e: nom::Err<I, u32>) -> Self {
        let parse_error = crate::Error::from_err_str(&e);
        From::from(parse_error)
    }
}

#[derive(Debug)]
pub struct Compat(pub failure::Compat<Error>);

impl Deref for Compat {
    type Target = failure::Compat<Error>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl serde::de::Error for Compat {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        From::from(Error::Custom(msg.to_string()))
    }
}

impl Display for Compat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl std::error::Error for Compat {}

impl From<Error> for Compat {
    fn from(e: Error) -> Self {
        Compat(e.compat())
    }
}
