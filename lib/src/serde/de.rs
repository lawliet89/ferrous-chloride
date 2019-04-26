//! Deserializer Implementation
//!
//! This module contains the types and trait implementation to allow deserialization from a HCL
//! string to Rust types that you can usually disregard. To find out more
//! about _using_ them, head to [`serde` documentation](https://serde.rs/).
pub mod block;
pub mod body;
pub mod expression;
pub mod object;

#[doc(inline)]
pub use self::error::*;
#[doc(inline)]
pub use body::{from_str, Deserializer};

use std::borrow::Cow;
use serde::de::{IntoDeserializer, Visitor};
use crate::parser;

mod error {
    use std::fmt::Display;
    use std::ops::Deref;

    use failure::{self, Fail};

    /// Error type for deserialization
    #[derive(Debug, Fail)]
    pub enum Error {
        #[fail(display = "HCL parse error: {}", _0)]
        ParseError(#[cause] crate::Error),

        #[fail(display = "Error parsing integer: {}", _0)]
        ParseIntError(#[cause] std::num::ParseIntError),

        #[fail(display = "Error parsing float: {}", _0)]
        ParseFloatError(#[cause] std::num::ParseFloatError),

        #[fail(display = "Input was not completely consumed during deserialization.")]
        TrailingCharacters,

        #[fail(display = "Overflow when trying to convert to {}", _0)]
        Overflow(&'static str),

        #[fail(
            display = "Invalid tuple length. Expected {}, got {}",
            expected, actual
        )]
        InvalidTupleLength { expected: usize, actual: usize },

        #[fail(display = "Object has duplicate key {}", _0)]
        ObjectDuplicateKey(String),

        #[fail(display = "Body has duplicate key {}", _0)]
        BodyDuplicateKey(String),

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

    impl From<std::num::ParseIntError> for Error {
        fn from(e: std::num::ParseIntError) -> Self {
            Error::ParseIntError(e)
        }
    }

    impl From<std::num::ParseFloatError> for Error {
        fn from(e: std::num::ParseFloatError) -> Self {
            Error::ParseFloatError(e)
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

    impl From<crate::Error> for Compat {
        fn from(e: crate::Error) -> Self {
            From::from(Error::ParseError(e))
        }
    }

    impl From<Compat> for Error {
        fn from(e: Compat) -> Self {
            e.0.into_inner()
        }
    }
}

fn deserialize_string<'de, V>(string: Cow<'de, str>, visitor: V) -> Result<V::Value, Compat>
where
    V: Visitor<'de>,
{
    match string {
        Cow::Borrowed(string) => visitor.visit_borrowed_str(string),
        Cow::Owned(string) => visitor.visit_string(string),
    }
}

fn deserialize_number<'de, V>(
    number: parser::number::Number<'de>,
    visitor: V,
) -> Result<V::Value, Compat>
where
    V: Visitor<'de>,
{
    if number.is_float() {
        visitor.visit_f64(number.as_f64().map_err(Error::ParseFloatError)?)
    } else if number.is_signed() {
        visitor.visit_i64(number.as_i64().map_err(Error::ParseIntError)?)
    } else {
        visitor.visit_u64(number.as_u64().map_err(Error::ParseIntError)?)
    }
}

fn deserialize_tuple<'de, V>(
    tuple: parser::tuple::Tuple<'de>,
    visitor: V,
    check_length: Option<usize>,
) -> Result<V::Value, Compat>
where
    V: Visitor<'de>,
{
    if let Some(len) = check_length {
        if tuple.len() != len {
            Err(Error::InvalidTupleLength {
                expected: len,
                actual: tuple.len(),
            })?;
        }
    }

    visitor.visit_seq(tuple.into_deserializer())
}

fn deserialize_object<'de, V>(
    object: parser::object::Object<'de>,
    visitor: V,
) -> Result<V::Value, Compat>
where
    V: Visitor<'de>,
{
    visitor.visit_map(object::ObjectMapAccess::new(object))
}
