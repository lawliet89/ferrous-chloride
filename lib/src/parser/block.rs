//! Block structure
//!
//! Blocks create a child body annotated by a type and optional labels.
//!
//! ```ebnf
//! Block        = Identifier (StringLit|Identifier)* "{" Newline Body "}" Newline;
//! OneLineBlock = Identifier (StringLit|Identifier)* "{" (Identifier "=" Expression)? "}" Newline;
//! ```
use std::borrow::{Borrow, Cow};

use nom::types::CompleteStr;
use nom::{alt, call, char, do_parse, many0, named, peek, recognize, tag, terminated, IResult};

use crate::parser::body::Body;
use crate::parser::identifier::{identifier, Identifier};
use crate::parser::string::{string_literal, StringLiteral};
use crate::parser::whitespace::newline;

/// HCL Block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block<'a> {
    pub r#type: Identifier<'a>,
    pub labels: Vec<BlockLabel<'a>>,
    pub body: Body<'a>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BlockLabel<'a> {
    StringLiteral(StringLiteral),
    Identifier(Identifier<'a>),
}

impl<'a, S> PartialEq<S> for BlockLabel<'a>
where
    S: AsRef<str>,
{
    fn eq(&self, other: &S) -> bool {
        match self {
            BlockLabel::StringLiteral(lit) => lit.eq(other.as_ref()),
            BlockLabel::Identifier(ident) => ident.eq(other.as_ref()),
        }
    }
}

impl<'a> Borrow<str> for BlockLabel<'a> {
    fn borrow(&self) -> &str {
        match self {
            BlockLabel::StringLiteral(ref lit) => lit,
            BlockLabel::Identifier(ref ident) => ident,
        }
    }
}

impl<'a> From<&'a str> for BlockLabel<'a> {
    fn from(s: &'a str) -> Self {
        BlockLabel::Identifier(Cow::Borrowed(s))
    }
}

named!(
    pub block_label(CompleteStr) -> BlockLabel,
    alt!(
        call!(identifier) =>
            { |ident| BlockLabel::Identifier(ident) }
        | call!(string_literal) =>
            { |s| BlockLabel::StringLiteral(s) }
    )
);

named!(
    pub block_labels(CompleteStr) -> Vec<BlockLabel>,
    many0!(
        inline_whitespace!(block_label)
    )
);

// named!(
//     pub one_line_block_body(CompleteStr) -> Option<
// )

// named!(
//     pub one_line_block(CompleteStr) -> Block,
//     inline_whitespace!(
//         do_parse!(
//             block_type: call!(identifier)
//             >> labels: call!(block_labels)
//             >> tag!("{")
//             >>
//         )
//     )
// );

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::ResultUtilsString;

    #[test]
    fn block_label_is_parsed_successfully() {
        let test_cases = [
            ("foobar", BlockLabel::from("foobar")),
            (
                r#""foobar""#,
                BlockLabel::StringLiteral(From::from("foobar")),
            ),
        ];

        for (input, expected_output) in &test_cases {
            let output = block_label(CompleteStr(input)).unwrap_output();
            assert_eq!(output, *expected_output);
        }
    }

    #[test]
    fn block_labels_are_parsed_successfully() {
        let test_cases = [
            ("", vec![]),
            ("foobar", vec![BlockLabel::from("foobar")]),
            (
                "foo bar baz",
                vec![
                    BlockLabel::from("foo"),
                    BlockLabel::from("bar"),
                    BlockLabel::from("baz"),
                ],
            ),
            (
                r#""foobar""#,
                vec![BlockLabel::StringLiteral(From::from("foobar"))],
            ),
            (
                "foo \"bar\" baz",
                vec![
                    BlockLabel::from("foo"),
                    BlockLabel::StringLiteral(From::from("bar")),
                    BlockLabel::from("baz"),
                ],
            ),
        ];

        for (input, expected_output) in &test_cases {
            let output = block_labels(CompleteStr(input)).unwrap_output();
            assert_eq!(output, *expected_output);
        }
    }
}
