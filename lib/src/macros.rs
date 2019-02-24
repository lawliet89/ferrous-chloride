// Eat whitespace without "\r" or "\n"
// See https://docs.rs/nom/4.2.0/nom/whitespace/index.html
use nom::{eat_separator, named};

named!(pub(crate) space(&str) -> &str, eat_separator!(" \t"));

#[macro_export]
macro_rules! whitespace (
  ($i:expr, $($args:tt)*) => (
    {
      use nom::{Convert, Err};
      use nom::sep;

      use crate::macros::space;

      match sep!($i, space, $($args)*) {
        Err(e) => Err(e),
        Ok((i1,o))    => {
          match space(i1) {
            Err(e) => Err(Err::convert(e)),
            Ok((i2,_)) => Ok((i2, o))
          }
        }
      }
    }
  )
);