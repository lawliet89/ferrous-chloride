use nom::types::{CompleteByteSlice, CompleteStr, Input};
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
    /// and/or end of a UTF-8 code point sequence
    /// (see [`str::is_char_boundary`](https://doc.rust-lang.org/std/primitive.str.html#method.is_char_boundary)).
    fn is_slice_boundary(&self, index: usize) -> bool;

    /// Safe slicing. The start and the end of the range (if bounded) must be
    /// at a boundary
    ///
    /// If they are unsafe, then the implementation will return `None`.
    fn safe_slice(&self, range: R) -> Option<Self> {
        if !match range.start_bound() {
            Bound::Included(start) => self.is_slice_boundary(*start),
            // No such Range exist
            Bound::Excluded(start) => self.is_slice_boundary(start + 1),
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

impl<'a, R> SliceBoundary<R> for &'a str
where
    R: RangeBounds<usize>,
    &'a str: nom::Slice<R>,
{
    fn is_slice_boundary(&self, index: usize) -> bool {
        self.is_char_boundary(index)
    }
}

impl<'a, R> SliceBoundary<R> for CompleteStr<'a>
where
    R: RangeBounds<usize>,
    CompleteStr<'a>: nom::Slice<R>,
{
    fn is_slice_boundary(&self, index: usize) -> bool {
        self.is_char_boundary(index)
    }
}

impl<'a, R, T> SliceBoundary<R> for &'a [T]
where
    R: RangeBounds<usize>,
    &'a [T]: nom::Slice<R>,
{
    fn is_slice_boundary(&self, _index: usize) -> bool {
        // Always true for arbitrary slices
        true
    }
}

impl<R, T> SliceBoundary<R> for Input<T>
where
    R: RangeBounds<usize>,
    Input<T>: nom::Slice<R>,
    T: nom::Slice<R>,
{
    fn is_slice_boundary(&self, _index: usize) -> bool {
        // Always true for arbitrary slices
        true
    }
}

impl<'a, R> SliceBoundary<R> for CompleteByteSlice<'a>
where
    R: RangeBounds<usize>,
    CompleteByteSlice<'a>: nom::Slice<R>,
{
    fn is_slice_boundary(&self, _index: usize) -> bool {
        // Always true for arbitrary bytes
        true
    }
}

// From https://github.com/Geal/nom/issues/709#issuecomment-475958529
// `take_till_match!(alt!(tag!("John") | tag!("Amanda")))`
// Running that on `"Hello, Amanda"` gives `Ok(("Amanda", "Hello, "))
#[macro_export]
macro_rules! take_till_match(
  (__impl $i:expr, $submac2:ident!( $($args2:tt)* )) => (
    {
      use nom::{Needed, need_more_err, ErrorKind};
      use nom::InputTake;

      use $crate::utils::SliceBoundary;

      let ret;
      let input = $i;
      let mut index = 0;

      loop {
        let slice = input.safe_slice(index..);

        match slice {
            None => {
                index += 1;
                if index >= input.len() {
                // XXX: this error is dramatically wrong
                    ret = need_more_err(input, Needed::Size(0), ErrorKind::TakeUntil::<u32>);
                    break;
                }
                else {
                    continue;
                }
            },
            Some(slice) => {
                match $submac2!(slice, $($args2)*) {
                    Ok((_i, _o)) => {
                        ret = Ok(input.take_split(index));
                        break;
                    },
                    Err(_e1)    => {
                        if index >= input.len() {
                            // XXX: this error is dramatically wrong
                            ret = need_more_err(input, Needed::Size(0), ErrorKind::TakeUntil::<u32>);
                            break;
                        } else {
                            index += 1;
                        }
                    },
                }
            }
        }
      }

      ret
    }
  );
  ($i:expr, $submac2:ident!( $($args2:tt)* )) => (
    take_till_match!(__impl $i, $submac2!($($args2)*));
  );
  ($i:expr, $g:expr) => (
    take_till_match!(__impl $i, call!($g));
  );
  ($i:expr, $submac2:ident!( $($args2:tt)* )) => (
    take_till_match!(__impl $i, $submac2!($($args2)*));
  );
  ($i:expr, $g: expr) => (
    take_till_match!(__impl $i, call!($g));
  );
);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_are_sliced_safely() {
        let s = "Löwe 老虎 Léopard";
        assert_eq!(Some(s), s.safe_slice(..));
        assert_eq!(Some("Löwe "), s.safe_slice(0..6));
        assert_eq!(Some("Löwe "), s.safe_slice(..6));
        assert_eq!(Some("老"), s.safe_slice(6..9));
        assert_eq!(Some("老虎 Léopard"), s.safe_slice(6..));

        // "老" is bytes 6 to 8
        assert_eq!(None, s.safe_slice(6..8));
        assert_eq!(None, s.safe_slice(7..));
        assert_eq!(None, s.safe_slice(..7));
    }
}
