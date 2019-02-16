use std::fmt::Debug;

use nom::IResult;

pub trait ResultUtils<O> {
    /// Unwraps the Output from `IResult`
    ///
    /// # Panics
    ///
    /// Panics if there is an error
    fn unwrap_output(self) -> O;

    /// Unwraps the Output from `IResult`
    ///
    /// # Panics
    ///
    /// Panics if there is an error and in a compact manner
    fn unwrap_output_compact(self) -> O;
}

impl<I, O, E> ResultUtils<O> for IResult<I, O, E>
where
    I: Debug,
    E: Debug,
{
    fn unwrap_output(self) -> O {
        match self {
            Err(e) => panic!("{:#?}", e),
            Ok((_, output)) => output,
        }
    }

    fn unwrap_output_compact(self) -> O {
        match self {
            Err(e) => panic!("{:#}", e),
            Ok((_, output)) => output,
        }
    }
}
