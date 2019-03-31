use std::fmt::{Debug, Display};
use std::string::ToString;

use failure_derive::Fail;
use nom::verbose_errors::Context;
use nom::ErrorKind;

use crate::OneOrMany;

/// Error type for parsing
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Invalid Unicode Code Points \\{}", _0)]
    InvalidUnicodeCodePoint(String),
    #[fail(display = "Invalid Number {}", _0)]
    InvalidNumber(String),
    #[fail(display = "Bytes contain invalid unicode: {:#?}", _0)]
    InvalidUnicode(Vec<u8>),
    #[fail(display = "Generic Parse Error {}", _0)]
    ParseError(String),
    #[fail(
        display = "Variant {} does not allow multiple values with the same key {}",
        variant, key
    )]
    IllegalMultipleEntries { key: String, variant: &'static str },
    #[fail(
        display = "Error merging key {} into `Value`: existing value of variant {} \
                   cannot be merged with variant {}",
        key, existing_variant, incoming_variant
    )]
    ErrorMergingKeys {
        key: String,
        existing_variant: &'static str,
        incoming_variant: &'static str,
    },
    #[fail(
        display = "Expected type {} to be of variant {} but got {} instead",
        enum_type, expected, actual
    )]
    UnexpectedVariant {
        enum_type: &'static str,
        expected: &'static str,
        actual: &'static str,
    },
    #[fail(display = "IO Error: {}", _0)]
    IOError(#[cause] std::io::Error),
    #[fail(display = "Bytes to be parsed is invalid UTF-8: {}", _0)]
    InvalidUnicodeToParse(#[cause] std::str::Utf8Error),
    #[fail(
        display = "Possible bug with the library encountered: {}; Please report to \
                   https://github.com/lawliet89/ferrous-chloride/issues",
        _0
    )]
    Bug(String),
}

impl Error {
    /// "Unknown" generic error
    fn new_generic<E: Display>(err: E) -> Self {
        Error::ParseError(format!("{:#}", err))
    }

    /// Convert a Nom Err into something useful
    pub fn from_err_bytes<I>(err: &nom::Err<I>) -> Self
    where
        I: nom::AsBytes + Debug,
    {
        Self::from_err(err, |s| {
            std::str::from_utf8(s.as_bytes())
                .ok()
                .map(ToString::to_string)
        })
    }

    /// Convert a Nom Err into something useful
    pub fn from_err_str<I>(err: &nom::Err<I>) -> Self
    where
        I: nom::AsBytes + AsRef<str> + Debug,
    {
        Self::from_err(err, |s| Some(s.as_ref().to_string()))
    }

    /// Convert a Nom Err into something useful
    fn from_err<I, F>(err: &nom::Err<I>, convert_fn: F) -> Self
    where
        I: nom::AsBytes + std::fmt::Debug,
        F: Fn(&I) -> Option<String>,
    {
        match err {
            nom::Err::Failure(ref context) => match Error::from_context(context, convert_fn) {
                Some(e) => e,
                None => Error::ParseError(format!("{:#}", err)),
            },
            err => Self::new_generic(err),
        }
    }

    /// Convert to a Custom Nom Error
    pub fn make_custom_error<I, F>(err: nom::Err<I>, convert_fn: F) -> nom::Err<I, Error>
    where
        I: nom::AsBytes + std::fmt::Debug,
        F: Fn(&I) -> Option<String>,
    {
        // let custom_error = Self::from_err(err, convert_fn);

        match err {
            nom::Err::Incomplete(needed) => nom::Err::Incomplete(needed),
            nom::Err::Error(context) => nom::Err::Error(Self::convert_context(context, convert_fn)),
            nom::Err::Failure(context) => {
                nom::Err::Failure(Self::convert_context(context, convert_fn))
            }
        }
    }

    pub fn make_custom_err_str<I>(err: nom::Err<I>) -> nom::Err<I, Error>
    where
        I: nom::AsBytes + AsRef<str> + Debug,
    {
        Self::make_custom_error(err, |s| Some(s.as_ref().to_string()))
    }

    pub fn make_custom_err_bytes<I>(err: nom::Err<I>) -> nom::Err<I, Error>
    where
        I: nom::AsBytes + Debug,
    {
        Self::make_custom_error(err, |s| {
            std::str::from_utf8(s.as_bytes())
                .ok()
                .map(ToString::to_string)
        })
    }

    /// Convert a Nom context into something more useful
    fn from_context<I, F>(context: &Context<I>, convert_fn: F) -> Option<Self>
    where
        F: Fn(&I) -> Option<String>,
        I: nom::AsBytes,
    {
        match context {
            Context::Code(input, ErrorKind::Custom(code)) => {
                Self::from_input_and_code(input, *code, convert_fn)
            }
            Context::List(list) => {
                if let Some((input, ErrorKind::Custom(code))) = list.last() {
                    Self::from_input_and_code(input, *code, convert_fn)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Convert Context with custom Error Kind
    fn convert_context<I, F>(context: Context<I>, convert_fn: F) -> Context<I, Error>
    where
        F: Fn(&I) -> Option<String>,
        I: nom::AsBytes,
    {
        let custom_error = match context {
            Context::Code(input, ErrorKind::Custom(code)) => {
                let error = nom::ErrorKind::Custom(
                    Self::from_input_and_code(&input, code, convert_fn)
                        .unwrap_or_else(|| Error::new_generic("UNKNOWN")),
                );
                OneOrMany::One((input, error))
            }
            Context::List(list) => OneOrMany::Many(
                list.into_iter()
                    .map(|(input, error_kind)| {
                        let error = match error_kind {
                            ErrorKind::Custom(code) => {
                                Self::from_input_and_code(&input, code, &convert_fn)
                                    .unwrap_or_else(|| Error::new_generic("UNKNOWN"))
                            }
                            other => Error::new_generic(other.description()),
                        };

                        (input, nom::ErrorKind::Custom(error))
                    })
                    .collect(),
            ),
            Context::Code(input, other) => OneOrMany::One((
                input,
                nom::ErrorKind::Custom(Self::new_generic(other.description())),
            )),
        };

        match custom_error {
            OneOrMany::One((input, error)) => Context::Code(input, error),
            OneOrMany::Many(list) => Context::List(list),
        }
    }

    fn from_input_and_code<I, F>(input: &I, code: u32, convert_fn: F) -> Option<Self>
    where
        F: Fn(&I) -> Option<String>,
        I: nom::AsBytes,
    {
        let kind = InternalKind::from_u32(code);
        if let Some(kind) = kind {
            match kind {
                InternalKind::InvalidUnicodeCodePoint => Some(Error::InvalidUnicodeCodePoint(
                    convert_fn(input).unwrap_or_else(|| "UNKNOWN".to_string()),
                )),
                InternalKind::InvalidUnicode => {
                    Some(Error::InvalidUnicode(input.as_bytes().to_vec()))
                }
                InternalKind::InvalidNumber => Some(Error::InvalidNumber(
                    convert_fn(input).unwrap_or_else(|| "UNKNOWN".to_string()),
                )),
            }
        } else {
            None
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Error::InvalidUnicodeToParse(e)
    }
}

// From https://serde.rs/enum-number.html
macro_rules! enum_number {
    ($name:ident { $($variant:ident = $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl $name {
            /// Convert from integer to enum
            pub fn from_u32(value: u32) -> Option<Self> {
                // Rust does not come with a simple way of converting a
                // number to an enum, so use a big `match`.
                match value {
                    $( $value => Some($name::$variant), )*
                    _ => None,
                }
            }
        }

        impl From<$name> for u32 {
            fn from(kind: $name) -> Self {
                kind as Self
            }
        }
    }
}

// Custom ErrorKind
enum_number!(InternalKind {
    InvalidUnicodeCodePoint = 0,
    InvalidUnicode = 1,
    InvalidNumber = 2,
});

impl From<std::str::Utf8Error> for InternalKind {
    fn from(_: std::str::Utf8Error) -> Self {
        InternalKind::InvalidUnicode
    }
}

impl From<std::num::ParseIntError> for InternalKind {
    fn from(_: std::num::ParseIntError) -> Self {
        InternalKind::InvalidNumber
    }
}

impl From<std::num::ParseFloatError> for InternalKind {
    fn from(_: std::num::ParseFloatError) -> Self {
        InternalKind::InvalidNumber
    }
}
