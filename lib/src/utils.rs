use std::fmt::Debug;

use nom::types::CompleteStr;
use nom::verbose_errors::Context;
use nom::IResult;

pub(crate) trait ResultUtils<O> {
    /// Unwraps the Output from `IResult`
    ///
    /// # Panics
    ///
    /// Panics if there is an error
    fn unwrap_output(self) -> O;
}

/// Duplicated trait because there is no specialisation!
pub(crate) trait ResultUtilsString<O> {
    /// Unwraps the Output from `IResult`
    ///
    /// # Panics
    ///
    /// Panics if there is an error
    fn unwrap_output(self) -> O;
}

impl<I, O> ResultUtils<O> for IResult<I, O>
where
    I: nom::AsBytes + Debug,
{
    fn unwrap_output(self) -> O {
        match self {
            Err(e) => {
                let e = crate::Error::from_err_bytes(e);
                panic!("{:#}", e)
            }
            Ok((_, output)) => output,
        }
    }
}

impl<I, O> ResultUtilsString<O> for IResult<I, O>
where
    I: AsRef<str> + std::fmt::Debug,
{
    fn unwrap_output(self) -> O {
        match self {
            Err(e) => {
                let e = crate::Error::from_err_str(e);
                panic!("{:#}", e)
            }
            Ok((_, output)) => output,
        }
    }
}

/// Recognizes at least 1 character while a predicate holds true
pub fn while_predicate1<T, F>(input: T, predicate: F) -> nom::IResult<T, T>
where
    F: Fn(char) -> bool,
    T: nom::InputTakeAtPosition,
    <T as nom::InputTakeAtPosition>::Item: nom::AsChar,
{
    use nom::AsChar;

    input.split_at_position1(
        |item| !predicate(item.as_char()),
        nom::ErrorKind::AlphaNumeric,
    )
}
