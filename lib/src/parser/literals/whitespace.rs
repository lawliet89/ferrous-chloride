//! Whitespace and comments related
//!
//! [Reference](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#comments-and-whitespace)
//!
//! Comments and Whitespace are recognized as lexical elements but are ignored
//! except as described below.
//!
//! Whitespace is defined as a sequence of zero or more space characters
//! (U+0020). Newline sequences (either U+000A or U+000D followed by U+000A)
//! are _not_ considered whitespace but are ignored as such in certain contexts.
//!
//! Horizontal tab characters (U+0009) are not considered to be whitespace and
//! are not valid within HCL native syntax.
//!
//! Comments serve as program documentation and come in two forms:
//!
//! - _Line comments_ start with either the `//` or `#` sequences and end with
//!   the next newline sequence. A line comments is considered equivalent to a
//!   newline sequence.
//!
//! - _Inline comments_ start with the `/*` sequence and end with the `*/`
//!   sequence, and may have any characters within except the ending sequence.
//!   An inline comments is considered equivalent to a whitespace sequence.
//!
//! Comments and whitespace cannot begin within within other comments, or within
//! template literals except inside an interpolation sequence or template directive.
use nom::types::CompleteStr;
use nom::{
    alt_complete, call, delimited, do_parse, eat_separator, eol, many0, many1, named, tag,
    take_until, take_while,
};

fn not_eol(c: char) -> bool {
    c != '\r' && c != '\n'
}

named!(
    pub inline_comment(CompleteStr) -> CompleteStr,
    delimited!(tag!("/*"), take_until!("*/"), tag!("*/"))
);

named!(
    pub hash_comment(CompleteStr) -> CompleteStr,
    delimited!(tag!("#"), take_while!(not_eol), call!(eol))
);

named!(
    pub slash_comment(CompleteStr) -> CompleteStr,
    delimited!(tag!("//"), take_while!(not_eol), call!(eol))
);

named!(
    pub line_comment(CompleteStr) -> CompleteStr,
    alt_complete!(
        hash_comment | hash_comment
    )
);

named!(pub inline_whitespace(CompleteStr) -> Vec<CompleteStr>,
    many0!(
        alt_complete!(
            inline_comment
            | eat_separator!(" \t")
        )
    )
);

named!(pub whitespace(CompleteStr) -> Vec<CompleteStr>,
    many0!(
        alt_complete!(
            hash_comment
            | slash_comment
            | inline_comment
            | eat_separator!(" \t\r\n")
        )
    )
);

named!(
    pub newline(CompleteStr) -> Vec<CompleteStr>,
    many1!(
        alt_complete!(
            hash_comment
            | slash_comment
            | do_parse!(
                comment: inline_comment
                >> call!(eol)
                >> (comment)
            )
            | call!(eol)
        )
    )
);

#[macro_export]
macro_rules! inline_whitespace (
  ($i:expr, $($args:tt)*) => (
    {
      use nom::{Convert, Err};
      use nom::sep;

      use crate::parser::literals::inline_whitespace;

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

      use crate::parser::literals::whitespace;

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
            (
                "/* Test Comment One liner */",
                vec![" Test Comment One liner "],
            ),
            (
                "/* Test Comment \nmultiple\r\n liner */",
                vec![" Test Comment \nmultiple\r\n liner "],
            ),
            (
                r#"// Comment One
# Comment Two
/* I am the last */
"#,
                vec![" Comment One", " Comment Two", " I am the last ", "\n"],
            ),
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
