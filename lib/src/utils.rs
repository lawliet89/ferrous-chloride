use nom::types::{CompleteByteSlice, CompleteStr, Input};
use std::ops::RangeFull;

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

pub trait SafeIndexing: nom::Slice<RangeFull> + Sized {
    type Iter: Iterator<Item = usize>;

    /// Returns an iterator that will only ever yield "safe" indices
    /// that do not cause panics when slicing
    ///
    /// For example, in a `str`, this iterator should never yield an index
    /// in the middle of a single unicode codepoint
    fn safe_indices(&self) -> Self::Iter;
}

fn char_index_take_index((index, _char): (usize, char)) -> usize {
    index
}

type TakeCharIndexFn = fn((usize, char)) -> usize;

impl<'a> SafeIndexing for &'a str
where
    &'a str: nom::Slice<RangeFull>,
{
    type Iter = std::iter::Map<std::str::CharIndices<'a>, TakeCharIndexFn>;

    fn safe_indices(&self) -> Self::Iter {
        self.char_indices().map(char_index_take_index)
    }
}

impl<'a> SafeIndexing for CompleteStr<'a>
where
    CompleteStr<'a>: nom::Slice<RangeFull>,
{
    type Iter = std::iter::Map<std::str::CharIndices<'a>, TakeCharIndexFn>;

    fn safe_indices(&self) -> Self::Iter {
        self.char_indices().map(char_index_take_index)
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn add_one(i: &usize) -> Option<usize> {
    Some(i + 1)
}

type AddOneFn = fn(&usize) -> Option<usize>;

impl<'a, T> SafeIndexing for &'a [T]
where
    &'a [T]: nom::Slice<RangeFull>,
{
    type Iter = std::iter::Successors<usize, AddOneFn>;

    fn safe_indices(&self) -> Self::Iter {
        std::iter::successors(Some(0), add_one)
    }
}

impl<'a> SafeIndexing for CompleteByteSlice<'a>
where
    CompleteByteSlice<'a>: nom::Slice<RangeFull>,
{
    type Iter = std::iter::Successors<usize, AddOneFn>;

    fn safe_indices(&self) -> Self::Iter {
        std::iter::successors(Some(0), add_one)
    }
}

impl<T> SafeIndexing for Input<T>
where
    Input<T>: nom::Slice<RangeFull>,
    T: nom::Slice<RangeFull>,
{
    type Iter = std::iter::Successors<usize, AddOneFn>;

    fn safe_indices(&self) -> Self::Iter {
        std::iter::successors(Some(0), add_one)
    }
}

// From https://github.com/Geal/nom/issues/709#issuecomment-475958529
/// Take bytes until the child parser succeeds
///
/// `take_till_match!(I -> IResult<I, O>) => I -> IResult<I, (I, O)>`
///
/// ```rust
/// # use ferrous_chloride::take_till_match;
/// use nom::{alt, named, tag};
///
/// named!(test<&str, (&str, &str)>, take_till_match!(alt!(tag!("John") | tag!("Amanda"))));
///
/// assert_eq!(test("Hello, Amanda and Tim"), Ok((" and Tim", ("Hello, ", "Amanda"))));
/// ```
#[macro_export]
macro_rules! take_till_match(
  (__impl $i:expr, $submac2:ident!( $($args2:tt)* )) => (
    {
      use nom::{Needed, need_more_err, ErrorKind};
      use nom::{InputTake, Slice};

      use $crate::utils::SafeIndexing;

      let mut ret = None;
      let input = $i;
      for index in input.safe_indices() {
        let slice = input.slice(index..);
        match $submac2!(slice, $($args2)*) {
            Ok((i, o)) => {
                let (_, start) = input.take_split(index);
                 ret = Some(Ok((i, (start, o))));
            },
            Err(_e1) => {},
        }
      }
      match ret {
          Some(ret) => ret,
          None => need_more_err(input, Needed::Size(0), ErrorKind::TakeUntil::<u32>)
      }
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
    fn strings_indices_are_returned_correctly() {
        let s = "Löwe 老虎 Léopard";
        let indices: Vec<_> = s.safe_indices().collect();

        let expected_indices = [
            0,  // L
            1,  // ö
            3,  // w
            4,  // e
            5,  // ` `
            6,  // 老
            9,  // 虎
            12, // ` `
            13, // L
            14, // é
            16, // o
            17, // p
            18, // a
            19, // r
            20, // d
        ];
        assert!(indices
            .iter()
            .zip(&expected_indices)
            .all(|(actual, expected)| actual == expected),)
    }
}
