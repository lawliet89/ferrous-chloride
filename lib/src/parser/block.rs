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
use nom::{alt, call, many0, named, opt, tag};

use crate::parser::attribute::{attribute, Attribute};
use crate::parser::body::{body, Body};
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

impl<'a> Block<'a> {
    pub fn new(r#type: Identifier<'a>, labels: Vec<BlockLabel<'a>>, body: Body<'a>) -> Self {
        Self {
            r#type,
            labels,
            body,
        }
    }

    pub fn new_one_line(
        r#type: Identifier<'a>,
        labels: Vec<BlockLabel<'a>>,
        attribute: Option<Attribute<'a>>,
    ) -> Self {
        let body = match attribute {
            None => Body::new_unmerged(vec![]),
            Some((ident, expr)) => Body::new_unmerged(vec![(ident, From::from(expr))]),
        };

        Self {
            r#type,
            labels,
            body,
        }
    }

    pub fn merge(mut self) -> Result<Self, crate::Error> {
        self.body = self.body.merge()?;
        Ok(self)
    }
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

named!(
    pub one_line_block_body(CompleteStr) -> Option<Attribute>,
    opt!(attribute)
);

named!(
    pub one_line_block(CompleteStr) -> Block,
    inline_whitespace!(
        do_parse!(
            block_type: call!(identifier)
            >> labels: call!(block_labels)
            >> tag!("{")
            >> attribute: call!(one_line_block_body)
            >> tag!("}")
            >> (Block::new_one_line(block_type, labels, attribute))
        )
    )
);

named!(
    pub block(CompleteStr) -> Block,
    inline_whitespace!(
        do_parse!(
            block_type: call!(identifier)
            >> labels: call!(block_labels)
            >> tag!("{")
            >> newline
            >> body: call!(body)
            >> tag!("}")
            >> (Block::new(block_type, labels, body))
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parser::expression::Expression;
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

    #[test]
    fn single_line_block_body_is_parsed_correctly() {
        let test_cases = [
            ("", None),
            ("foo = true", Some((From::from("foo"), From::from(true)))),
        ];

        for (input, expected_output) in &test_cases {
            let output = one_line_block_body(CompleteStr(input)).unwrap_output();
            assert_eq!(output, *expected_output);
        }
    }

    #[test]
    fn single_line_block_is_parsed_correctly() {
        let test_cases = [
            (
                "test {}",
                Block::new_one_line(From::from("test"), vec![], None),
            ),
            (
                "test foo bar baz {}",
                Block::new_one_line(
                    From::from("test"),
                    vec![
                        BlockLabel::from("foo"),
                        BlockLabel::from("bar"),
                        BlockLabel::from("baz"),
                    ],
                    None,
                ),
            ),
            (
                "test foo \"bar\" baz { foo = 123 }",
                Block::new_one_line(
                    From::from("test"),
                    vec![
                        BlockLabel::from("foo"),
                        BlockLabel::StringLiteral(From::from("bar")),
                        BlockLabel::from("baz"),
                    ],
                    Some((From::from("foo"), From::from(123))),
                ),
            ),
        ];

        for (input, expected_output) in &test_cases {
            let output = one_line_block(CompleteStr(input)).unwrap_output();
            assert_eq!(output, *expected_output);
        }
    }

    #[test]
    fn block_is_parsed_correctly() {
        let hcl = r#"simple_map "foo" bar {
  foo   = "bar"
  bar   = "baz"
  index = 0
}"#;
        let block = block(CompleteStr(hcl)).unwrap_output();

        let expected = Block::new(
            From::from("simple_map"),
            vec![
                BlockLabel::StringLiteral(From::from("foo")),
                BlockLabel::from("bar"),
            ],
            Body::new_unmerged(vec![
                (From::from("foo"), From::from(Expression::from("bar"))),
                (From::from("bar"), From::from(Expression::from("baz"))),
                (From::from("index"), From::from(Expression::from(0))),
            ]),
        );

        assert_eq!(block, expected);
    }
}
