use failure_derive::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Parser error: {}", _0)]
    ParserError(#[cause] ferrous_chloride::Error),
    #[fail(display = "IO Error: {}", _0)]
    IOError(#[cause] std::io::Error),
    #[fail(display = "Unknown command: {}", _0)]
    UnknownCommand(String),
}

impl From<ferrous_chloride::Error> for Error {
    fn from(e: ferrous_chloride::Error) -> Self {
        Error::ParserError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e)
    }
}
