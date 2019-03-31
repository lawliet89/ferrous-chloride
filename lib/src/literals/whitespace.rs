//! Whitespace and comments related
use nom::types::CompleteStr;
use nom::{
    alt_complete, call, delimited, eat_separator, eol, many0, named, tag, take_until, take_while,
};

fn not_eol(c: char) -> bool {
    c != '\r' && c != '\n'
}

named!(pub inline_whitespace(CompleteStr) -> CompleteStr,
    eat_separator!(" \t")
);

named!(pub whitespace(CompleteStr) -> Vec<CompleteStr>,
    many0!(
        alt_complete!(
            delimited!(tag!("#"), take_while!(not_eol), call!(eol))
            | delimited!(tag!("//"), take_while!(not_eol), call!(eol))
            | delimited!(tag!("/*"), take_until!("*/"), tag!("*/"))
            | eat_separator!(" \t\r\n")
        )
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

    use nom::{eof, is_alphanumeric, tag, take, take_while1};

    named!(inline_whitespace_test<CompleteStr, (CompleteStr, CompleteStr) >,
        inline_whitespace!(tuple!(take!(3), tag!("de")))
    );

    named!(whitespace_test<CompleteStr, Vec<CompleteStr>>,
        whitespace!(
            do_parse!(
                list: whitespace!(
                    separated_list!(
                         whitespace!(tag!("|")),
                         whitespace!(take_while1!(|c| is_alphanumeric(c as u8)))
                    )
                )
                >> whitespace!(eof!())
                >> (list)
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
            ("  \t\r\n", vec!["  \t\r\n"]),
            ("# Test Comment\r\n", vec![" Test Comment"]),
            ("// Test Comment\n", vec![" Test Comment"]),
            ("/* Test Comment One liner */", vec![" Test Comment One liner "]),
            (
                "/* Test Comment \nmultiple\r\n liner */",
                vec![" Test Comment \nmultiple\r\n liner "],
            ),
            (
                r#"// Comment One
# Comment Two
/* I am the last */
"#,
                vec![" Comment One", " Comment Two", " I am the last ", "\n"]
            )
        ];

        for (input, expected) in test_cases.iter() {
            let actual = whitespace(CompleteStr(input)).unwrap_output();
            let actual: Vec<_> = actual.into_iter().map(|s| s.0).collect();
            assert_eq!(actual, *expected, "Input: {}", input);
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
            ("foobar |  again # Test Comment\n", vec!["foobar", "again"]),
            (
                "foo | bar | baz // Test Comment\n",
                vec!["foo", "bar", "baz"],
            ),
            (
                "foo | bar | /* Test Comment One liner */ baz // Test Comment\n",
                vec!["foo", "bar", "baz"],
            ),
            (
                "foo | bar | /* Test Comment \nmultiple\r\n liner */ baz // Test Comment\n",
                vec!["foo", "bar", "baz"],
            ),
        ];

        for (input, expected) in test_cases.iter() {
            println!("Testing \"{}\"", input);
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
