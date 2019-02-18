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

/*
/// Given a parser `F` that takes in a `CompleteByteSlice`, wrap your inputs which are
/// `& [u8]`, call parser `F` and then unwrap it afterwards.
pub(crate) fn wrap_bytes<'a, F, O>(parser: F) -> impl FnOnce(&'a [u8]) -> IResult<&'a [u8], O>
where
    F: Fn(CompleteByteSlice<'a>) -> IResult<CompleteByteSlice<'a>, O>,
{
    move |input| bytes::unwrap_bytes(parser, input)
}

mod bytes {
    use super::*;

    pub(super) fn unwrap_bytes<'a, F, O>(parser: F, input: &'a [u8]) -> IResult<&'a [u8], O>
    where
        F: Fn(CompleteByteSlice<'a>) -> IResult<CompleteByteSlice<'a>, O>,
    {
        let input = CompleteByteSlice(input);

        match parser(input) {
            Ok((i, o)) => Ok((i.0, o)),
            Err(e) => Err(match e {
                nom::Err::Incomplete(n) => nom::Err::Incomplete(n),
                nom::Err::Failure(c) => nom::Err::Failure(context_unwrap_complete_bytes(c)),
                nom::Err::Error(c) => nom::Err::Error(context_unwrap_complete_bytes(c)),
            }),
        }
    }

    fn context_unwrap_complete_bytes<'a>(
        context: Context<CompleteByteSlice<'a>>,
    ) -> Context<&'a [u8]> {
        match context {
            Context::Code(i, e) => Context::Code(i.0, e),
            Context::List(list) => Context::List(list.into_iter().map(|(i, e)| (i.0, e)).collect()),
        }
    }

}
*/

/// Given a parser `F` that takes in a `CompleteStr`, wrap your inputs which are
/// `&str`, call parser `F` and then unwrap it afterwards.
pub(crate) fn wrap_str<'a, F, O>(parser: F) -> impl FnOnce(&'a str) -> IResult<&'a str, O>
where
    F: Fn(CompleteStr<'a>) -> IResult<CompleteStr<'a>, O>,
{
    move |input| r#str::unwrap_str(parser, input)
}

mod r#str {
    use super::*;

    pub(super) fn unwrap_str<'a, F, O>(parser: F, input: &'a str) -> IResult<&'a str, O>
    where
        F: Fn(CompleteStr<'a>) -> IResult<CompleteStr<'a>, O>,
    {
        let input = CompleteStr(input);

        match parser(input) {
            Ok((i, o)) => Ok((i.0, o)),
            Err(e) => Err(match e {
                nom::Err::Incomplete(n) => nom::Err::Incomplete(n),
                nom::Err::Failure(c) => nom::Err::Failure(context_unwrap_complete_str(c)),
                nom::Err::Error(c) => nom::Err::Error(context_unwrap_complete_str(c)),
            }),
        }
    }

    fn context_unwrap_complete_str<'a>(context: Context<CompleteStr<'a>>) -> Context<&'a str> {
        match context {
            Context::Code(i, e) => Context::Code(i.0, e),
            Context::List(list) => Context::List(list.into_iter().map(|(i, e)| (i.0, e)).collect()),
        }
    }

}
