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

impl<O> ResultUtils<O> for IResult<&[u8], O, u32> {
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

impl<I, O> ResultUtilsString<O> for IResult<I, O, u32>
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
