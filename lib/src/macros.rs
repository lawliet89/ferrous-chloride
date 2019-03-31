/// `map_err_str(IResult<I, O, u32>) -> IResult<I, O, Error>`
///
/// Converts a standard [`IResult<I, O, u32>`](nom::IResult) to
/// `IResult<I, O, `[`Error`](crate::Error)`>`
///
/// `I` __must__ implement [`nom::AsBytes`] `+` [`AsRef`]`<`[`str`]`> + `[`Debug`](std::fmt::Debug)
#[macro_export]
macro_rules! map_err_str (
  ($i:expr, $submac:ident!( $($args:tt)* )) => (
    {
      use crate::Error;
      ($submac!($i, $($args)*)).map_err(Error::make_custom_err_str)
    }
  )
);

/// `map_err_str(IResult<I, O, u32>) -> IResult<I, O, Error>`
///
/// Converts a standard [`IResult<I, O, u32>`](nom::IResult) to
/// `IResult<I, O, `[`Error`](crate::Error)`>`
///
/// `I` __must__ implement [`nom::AsBytes`] ` + `[`Debug`](std::fmt::Debug)
#[macro_export]
macro_rules! map_err (
  ($i:expr, $submac:ident!( $($args:tt)* )) => (
    {
      use crate::Error;
      ($submac!($i, $($args)*)).map_err(Error::make_custom_err_bytes)
    }
  )
);

#[cfg(test)]
#[macro_export]
macro_rules! assert_list_eq {
    ($left:expr, $right:expr) => {{
      match (&$left, &$right) {
        (left_val, right_val) => {
          let equal = (left_val)
              .iter()
              .zip(right_val)
              .all(|(left, right)| left.eq(*right));
          if !equal {
              panic!(
                  r#"assertion failed: `(left == right)`
  left: `{:#?}`,
  right: `{:#?}`"#,
                  left_val, right_val
              )
          }
        }
      }
    }};
    ($left:expr, $right:expr,) => {{
        assert_list_eq!($left, $right)
    }};
    ($left:expr, $right:expr, $($arg:tt)+) => {{
      match (&$left, &$right) {
        (left_val, right_val) => {
          let equal = (left_val)
              .iter()
              .zip(right_val)
              .all(|(left, right)| left.eq(*right));
          if !equal {
              panic!(
                  r#"assertion failed: `(left == right)`
  left: `{:#?}`,
  right: `{:#?}: {}`"#,
                  left_val, right_val,
                  format_args!($($arg)+))
          }
        }
      }
    }};
}
