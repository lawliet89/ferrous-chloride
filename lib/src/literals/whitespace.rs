//! Whitespace and comments related
use nom::types::CompleteStr;
use nom::{alt_complete, call, delimited, eat_separator, eol, named, tag, take_until, take_while};

fn not_eol(c: char) -> bool {
    c != '\r' && c != '\n'
}

named!(pub inline_whitespace(CompleteStr) -> CompleteStr,
    eat_separator!(" \t")
);

named!(pub whitespace(CompleteStr) -> CompleteStr,
    alt_complete!(
        delimited!(tag!("#"), take_while!(not_eol), call!(eol))
        | delimited!(tag!("//"), take_while!(not_eol), call!(eol))
        | delimited!(tag!("/*"), take_until!("*/"), tag!("*/"))
        | eat_separator!(" \t\r\n")
    )
);

#[macro_export]
macro_rules! inline_whitespace (
  ($i:expr, $($args:tt)*) => (
    {
      use nom::{Convert, Err};
      use nom::sep;

      use crate::literals::inline_whitespace;

      match sep!($i, inline_whitespace, $($args)*) {
        Err(e) => Err(e),
        Ok((i1,o))    => {
          match inline_whitespace(i1) {
            Err(e) => Err(Err::convert(e)),
            Ok((i2,_)) => Ok((i2, o))
          }
        }
      }
    }
  )
);

// TODO: Handle comments
#[macro_export]
macro_rules! whitespace (
  ($i:expr, $($args:tt)*) => (
    {
      use nom::{Convert, Err};
      use nom::sep;

      use crate::literals::whitespace;

      match sep!($i, whitespace, $($args)*) {
        Err(e) => Err(e),
        Ok((i1,o))    => {
          match whitespace(i1) {
            Err(e) => Err(Err::convert(e)),
            Ok((i2,_)) => Ok((i2, o))
          }
        }
      }
    }
  )
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::ResultUtilsString;

    use nom::{tag, take};

    named!(inline_whitespace_test<CompleteStr, (CompleteStr, CompleteStr) >,
        inline_whitespace!(tuple!(take!(3), tag!("de")))
    );

    named!(whitespace_test<CompleteStr, Vec<CompleteStr>>,
        whitespace!(
            separated_list!(
                tag!("|"),
                call!(crate::utils::while_predicate1,
                      |c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
            )
        )
    );

    #[test]
    fn inline_whitespace_are_ignored() {
        assert_eq!(
            inline_whitespace_test(CompleteStr(" \t abc de fg")),
            Ok((CompleteStr("fg"), (CompleteStr("abc"), CompleteStr("de"))))
        );
    }

    #[test]
    fn whitespace_finds_comments() {
        let test_cases = [
            ("  \t\r\n", "  \t\r\n"),
            ("# Test Comment\r\n", " Test Comment"),
            ("// Test Comment\n", " Test Comment"),
            ("/* Test Comment One liner */", " Test Comment One liner "),
            (
                "/* Test Comment \nmultiple\r\n liner */",
                " Test Comment \nmultiple\r\n liner ",
            ),
        ];

        for (input, expected) in test_cases.iter() {
            let actual = whitespace(CompleteStr(input)).unwrap_output();
            assert_eq!(actual.0, *expected, "Input: {}", input);
        }
    }

    #[test]
    fn whitespace_are_ignored() {
        let test_cases = [
            (
                "foo | bar \t | baz | \r\n more",
                vec!["foo", "bar", "baz", "more"],
            ),
            ("# Test Comment\n", vec![]),
            ("// Test Comment\n", vec![]),
            ("/* Test Comment One liner */", vec![]),
            ("foobar # Test Comment\n", vec!["foobar"]),
            (
                "foo | bar | baz // Test Comment\n",
                vec!["foo", "bar", "baz"],
            ),
            (
                "foo | bar | /* Test Comment One liner */ baz // Test Comment",
                vec!["foo", "bar", "baz"],
            ),
            (
                "foo | bar | /* Test Comment \nmultiple\r\n liner */ baz // Test Comment",
                vec!["foo", "bar", "baz"],
            ),
        ];

        for (input, expected) in test_cases.iter() {
            let actual = whitespace_test(CompleteStr(input)).unwrap_output();
            assert_eq!(
                actual.iter().map(|s| s.0).collect::<Vec<_>>(),
                *expected,
                "Input: {}",
                input
            );
        }
    }
}
