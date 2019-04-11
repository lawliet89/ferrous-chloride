use std::ops::{Bound, RangeBounds};

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

pub trait SliceBoundary<R>: nom::Slice<R> + Sized
where
    R: RangeBounds<usize>,
{
    /// Indicate if an `index` is the start and/or end of some boundary
    ///
    /// For example, in an implementation for a `&str`, this might be start
    /// and/or end of a UTF-8 code point sequence (see [`str::is_char_boundary`]).
    fn is_slice_boundary(&self, index: usize) -> bool;

    /// Safe slicing. The start and the end of the range (if bounded) must be
    /// at a boundary
    ///
    /// If they are unsafe, then the implementation will return `None`.
    fn safe_slice(&self, range: R) -> Option<Self> {
        if !match range.start_bound() {
            Bound::Included(start) => self.is_slice_boundary(start - 1),
            Bound::Excluded(start) => self.is_slice_boundary(*start),
            Bound::Unbounded => true,
        } {
            return None;
        }

        if !match range.end_bound() {
            Bound::Included(end) => self.is_slice_boundary(end + 1),
            Bound::Excluded(end) => self.is_slice_boundary(*end),
            Bound::Unbounded => true,
        } {
            return None;
        }

        Some(self.slice(range))
    }
}

#[cfg(test)]
mod test_utils {
    use nom::{IResult, InputLength};
    use std::borrow::Borrow;
    use std::fmt::Debug;

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
        I: nom::AsBytes + Debug + InputLength,
    {
        fn unwrap_output(self) -> O {
            match self {
                Err(e) => {
                    let e = crate::Error::from_err_bytes(&e);
                    panic!("{:#}", e)
                }
                Ok((remaining, output)) => {
                    assert!(remaining.input_len() == 0, "Remaining: {:#?}", remaining);
                    output
                }
            }
        }
    }

    impl<I, O> ResultUtilsString<O> for IResult<I, O>
    where
        I: nom::AsBytes + AsRef<str> + std::fmt::Debug + InputLength,
    {
        fn unwrap_output(self) -> O {
            match self {
                Err(e) => {
                    let e = crate::Error::from_err_str(&e);
                    panic!("{:#}", e)
                }
                Ok((remaining, output)) => {
                    assert!(
                        remaining.input_len() == 0,
                        "Remaining: {}",
                        remaining.as_ref()
                    );
                    output
                }
            }
        }
    }

    pub(crate) fn assert_list_eq<B1, B2, T1, T2, L1, L2>(left: L1, right: L2)
    where
        B1: Borrow<T1>,
        B2: Borrow<T2>,
        T1: PartialEq<T2>,
        T2: PartialEq<T1>,
        L1: IntoIterator<Item = B1> + Debug,
        L2: IntoIterator<Item = B2> + Debug,
    {
        println!(
            r#"Checking `(left == right)`
  left: `{:#?}`,
  right: `{:#?}`"#,
            left, right
        );

        let equal = left
            .into_iter()
            .zip(right)
            .all(|(left, right)| left.borrow().eq(right.borrow()));
        if !equal {
            panic!("left != right");
        }
    }
}

#[cfg(test)]
pub(crate) use test_utils::*;
