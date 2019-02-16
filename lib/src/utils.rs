use std::fmt::{Debug, Display};

use nom::IResult;

pub trait ResultUtils<O> {
    /// Unwraps the Output from `IResult`
    ///
    /// # Panics
    ///
    /// Panics if there is an error
    fn unwrap_output(self) -> O;
}

impl<I, O> ResultUtils<O> for IResult<I, O, u32>
where
    I: Display + Debug,
{
    fn unwrap_output(self) -> O {
        match self {
            Err(e) => {
                let e = crate::Error::from(e);
                panic!("{:#}", e)
            }
            Ok((_, output)) => output,
        }
    }
}
