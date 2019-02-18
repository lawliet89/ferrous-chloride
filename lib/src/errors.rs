use std::fmt::Debug;

use failure_derive::Fail;
use nom::verbose_errors::Context;
use nom::ErrorKind;

/// Error type for this library
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Invalid Unicode Code Points \\{}", _0)]
    InvalidUnicodeCodePoint(String),
    #[fail(display = "Bytes contain invalid unicode: {:#?}", _0)]
    InvalidUnicode(Vec<u8>),
    #[fail(display = "Generic Parse Error {}", _0)]
    ParseError(String),
}

impl Error {
    /// Convert a Nom Err into something useful
    pub fn from_err_bytes<I>(err: nom::Err<I>) -> Self
    where
        I: AsRef<[u8]> + Debug,
    {
        Self::from_err(err, |s| {
            std::str::from_utf8(s.as_ref()).ok().map(|s| s.to_string())
        })
    }

    /// Convert a Nom Err into something useful
    pub fn from_err_str<I>(err: nom::Err<I>) -> Self
    where
        I: AsRef<str> + Debug,
    {
        Self::from_err(err, |s| Some(s.as_ref().to_string()))
    }

    /// Convert a Nom Err into something useful
    fn from_err<I, F>(err: nom::Err<I>, convert_fn: F) -> Self
    where
        I: std::fmt::Debug,
        F: Fn(&I) -> Option<String>,
    {
        match err {
            nom::Err::Failure(ref context) => match Error::from_context(context, convert_fn) {
                Some(e) => e,
                None => Error::ParseError(format!("{:#}", err)),
            },
            err => Error::ParseError(format!("{:#}", err)),
        }
    }

    // Convert a Nom context into something more useful
    fn from_context<I, F>(context: &Context<I>, convert_fn: F) -> Option<Self>
    where
        F: Fn(&I) -> Option<String>,
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

    fn from_input_and_code<I, F>(input: &I, code: u32, convert_fn: F) -> Option<Self>
    where
        F: Fn(&I) -> Option<String>,
    {
        let kind = InternalKind::from_u32(code);
        if let Some(kind) = kind {
            match kind {
                InternalKind::InvalidUnicodeCodePoint => Some(Error::InvalidUnicodeCodePoint(
                    convert_fn(input).unwrap_or_else(|| "UNKNOWN".to_string()),
                )),
                InternalKind::InvalidUnicode => None, // TODO!
                InternalKind::InvalidInteger => None, // TODO!
            }
        } else {
            None
        }
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

/// Custom ErrorKind
enum_number!(InternalKind {
    InvalidUnicodeCodePoint = 0,
    InvalidUnicode = 1,
    InvalidInteger = 2,
});

impl From<std::str::Utf8Error> for InternalKind {
    fn from(_: std::str::Utf8Error) -> Self {
        InternalKind::InvalidUnicode
    }
}

impl From<std::num::ParseIntError> for InternalKind {
    fn from(_: std::num::ParseIntError) -> Self {
        InternalKind::InvalidInteger
    }
}
