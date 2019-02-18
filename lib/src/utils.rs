use std::fmt::Debug;

use nom::types::CompleteByteSlice;
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
                nom::Err::Failure(c) => {
                    nom::Err::Failure(context_unwrap_complete_bytes(c))
                }
                nom::Err::Error(c) => {
                    nom::Err::Error(context_unwrap_complete_bytes(c))
                }
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
