use failure_derive::Fail;

/// Error type for this library
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Invalid Unicode Code Points {}", _0)]
    InvalidUnicode(String),
}

/// Custom ErrorKind
#[derive(Debug)]
pub enum ErrorKind {
    InvalidUnicode,
}

impl From<ErrorKind> for u32 {
    fn from(kind: ErrorKind) -> Self {
        kind as Self
    }
}
