use failure_derive::Fail;

/// Error type for this library
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Unknown Numeric Sign: {}", _0)]
    UnknownNumericSign(char),
}
