//! HCL Body
//!
//! [Reference](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#structural-elements)
use nom::types::CompleteStr;
use nom::{alt, call, do_parse, eof, named_attr, terminated};

use crate::parser::attribute::{attribute, Attribute};
use crate::parser::block::{block, one_line_block, Block};
use crate::parser::whitespace::newline;

/// A HCL document body
///
/// ```ebnf
/// ConfigFile   = Body;
/// Body         = (Attribute | Block | OneLineBlock)*;
/// Attribute    = Identifier "=" Expression Newline;
/// Block        = Identifier (StringLit|Identifier)* "{" Newline Body "}" Newline;
/// OneLineBlock = Identifier (StringLit|Identifier)* "{" (Identifier "=" Expression)? "}" Newline;
/// ```
// TODO: Change this into a vec of Body Element. Remove merging semantics
pub type Body<'a> = Vec<BodyElement<'a>>;

/// An element of `Body`
///
/// ```ebnf
/// Attribute | Block | OneLineBlock
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BodyElement<'a> {
    Attribute(Attribute<'a>),
    Block(Block<'a>),
}

impl<'a> From<Attribute<'a>> for BodyElement<'a> {
    fn from(attr: Attribute<'a>) -> Self {
        BodyElement::Attribute(attr)
    }
}

impl<'a> From<Block<'a>> for BodyElement<'a> {
    fn from(blk: Block<'a>) -> Self {
        BodyElement::Block(blk)
    }
}

named_attr!(
    #[doc = r#"Parses a `Body` element

```ebnf
Attribute | Block | OneLineBlock
```
"#],
    pub body_element(CompleteStr) -> BodyElement,
    alt!(
        attribute => { |attr| BodyElement::Attribute(attr) }
        | one_line_block => { |blk| BodyElement::Block(blk) }
        | block => { |blk| BodyElement::Block(blk) }
    )
);

named_attr!(
    #[doc = r#"Parses a `Body`

```ebnf
Body = (Attribute | Block | OneLineBlock)*;
```
"#],
    pub body(CompleteStr) -> Body,
    do_parse!(
        values: whitespace!(
            many0!(
                terminated!(
                    call!(body_element),
                    alt!(
                        call!(newline) => { |_| CompleteStr("") }
                        | eof!()
                    )
                )
            )
        )
        >> (values.into_iter().collect())
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::fixtures;
    use crate::parser::expression::Expression;
    use crate::utils::ResultUtilsString;

    #[test]
    fn empty_body_is_parsed_correctly() {
        let hcl = "";
        let parsed = body(CompleteStr(hcl)).unwrap_output();

        assert_eq!(0, parsed.len());
    }

    #[test]
    fn non_terminating_new_lines_are_parsed_correctly() {
        let hcl = fixtures::NO_NEWLINE_EOF;
        let parsed = body(CompleteStr(hcl)).unwrap_output();

        assert_eq!(
            parsed,
            vec![From::from((From::from("test"), From::from(true)))]
        );
    }

    #[test]
    fn single_attribute_body_is_parsed_correctly() {
        let hcl = fixtures::SINGLE;
        let parsed = body(CompleteStr(hcl)).unwrap_output();

        assert_eq!(
            parsed,
            vec![From::from((From::from("foo"), From::from("bar")))]
        );
    }

    #[test]
    fn scalar_attributes_are_parsed_correctly() {
        let hcl = fixtures::SCALAR;
        let parsed = body(CompleteStr(hcl)).unwrap_output();

        let expected = vec![
            BodyElement::from((From::from("test_unsigned_int"), Expression::from(123))),
            BodyElement::from((From::from("test_signed_int"), Expression::from(-123))),
            BodyElement::from((From::from("test_float"), Expression::from(-1.23))),
            BodyElement::from((From::from("bool_true"), Expression::from(true))),
            BodyElement::from((From::from("bool_false"), Expression::from(false))),
            BodyElement::from((From::from("string"), Expression::from("Hello World!"))),
            BodyElement::from((
                From::from("long_string"),
                Expression::from("hihi\nanother line!"),
            )),
            BodyElement::from((
                From::from("string_escaped"),
                Expression::from("\" Hello World!"),
            )),
        ];

        assert_eq!(expected, parsed);
    }

    #[test]
    fn list_in_body_are_parsed_correctly() {
        let hcl = fixtures::LIST;
        let parsed = body(CompleteStr(hcl)).unwrap_output();

        let expected = vec![
            BodyElement::from((
                From::from("list"),
                Expression::new_tuple(vec![
                    From::from(true),
                    From::from(false),
                    From::from(123),
                    From::from(-123.456),
                    From::from("foobar"),
                ]),
            )),
            BodyElement::from((
                From::from("list_multi"),
                Expression::new_tuple(vec![
                    From::from(true),
                    From::from(false),
                    From::from(123),
                    From::from(-123.456),
                    From::from("foobar"),
                ]),
            )),
            BodyElement::from((
                From::from("list_in_list"),
                Expression::new_tuple(vec![
                    Expression::new_tuple(vec![From::from("test"), From::from("foobar")]),
                    From::from(1),
                    From::from(2),
                    From::from(-3),
                ]),
            )),
            BodyElement::from((
                From::from("object_in_list"),
                Expression::new_tuple(vec![
                    Expression::new_object(vec![("test", Expression::from(123))]),
                    Expression::new_object(vec![("foo", Expression::from("bar"))]),
                    Expression::new_object(vec![("baz", Expression::from(false))]),
                ]),
            )),
        ];

        assert_eq!(expected, parsed);
    }

    #[test]
    fn fixture_block_is_parsed_correctly() {
        use crate::parser::block::BlockLabel;

        let hcl = fixtures::BLOCK;
        let parsed = body(CompleteStr(hcl)).unwrap_output();

        let expected = vec![
            From::from(Block::new(
                From::from("simple_map"),
                vec![],
                vec![
                    From::from((From::from("foo"), Expression::from("bar"))),
                    From::from((From::from("bar"), Expression::from("baz"))),
                    From::from((From::from("index"), Expression::from(1))),
                ],
            )),
            From::from(Block::new(
                From::from("simple_map"),
                vec![],
                vec![
                    From::from((From::from("foo"), Expression::from("bar"))),
                    From::from((From::from("bar"), Expression::from("baz"))),
                    From::from((From::from("index"), Expression::from(0))),
                ],
            )),
            From::from(Block::new(
                From::from("resource"),
                vec![
                    BlockLabel::StringLiteral(From::from("security/group")),
                    BlockLabel::StringLiteral(From::from("foobar")),
                ],
                vec![
                    From::from((From::from("name"), Expression::from("foobar"))),
                    BodyElement::Block(Block::new(
                        From::from("allow"),
                        vec![],
                        vec![
                            From::from((From::from("name"), Expression::from("localhost"))),
                            From::from((
                                From::from("cidrs"),
                                Expression::from(vec![From::from("127.0.0.1/32")]),
                            )),
                        ],
                    )),
                    BodyElement::Block(Block::new(
                        From::from("allow"),
                        vec![],
                        vec![
                            From::from((From::from("name"), Expression::from("lan"))),
                            From::from((
                                From::from("cidrs"),
                                Expression::from(vec![From::from("192.168.0.0/16")]),
                            )),
                        ],
                    )),
                    BodyElement::Block(Block::new(
                        From::from("deny"),
                        vec![],
                        vec![
                            From::from((From::from("name"), Expression::from("internet"))),
                            From::from((
                                From::from("cidrs"),
                                Expression::from(vec![From::from("0.0.0.0/0")]),
                            )),
                        ],
                    )),
                ],
            )),
            From::from(Block::new(
                From::from("resource"),
                vec![
                    BlockLabel::StringLiteral(From::from("security/group")),
                    BlockLabel::StringLiteral(From::from("second")),
                ],
                vec![
                    From::from((From::from("name"), Expression::from("second"))),
                    BodyElement::Block(Block::new(
                        From::from("allow"),
                        vec![],
                        vec![
                            From::from((From::from("name"), Expression::from("all"))),
                            From::from((
                                From::from("cidrs"),
                                Expression::from(vec![From::from("0.0.0.0/0")]),
                            )),
                        ],
                    )),
                ],
            )),
            From::from(Block::new(
                From::from("resource"),
                vec![
                    BlockLabel::StringLiteral(From::from("instance")),
                    BlockLabel::StringLiteral(From::from("an_instance")),
                ],
                vec![
                    From::from((From::from("name"), Expression::from("an_instance"))),
                    From::from((From::from("image"), Expression::from("ubuntu:18.04"))),
                    BodyElement::Block(Block::new(
                        From::from("user"),
                        vec![BlockLabel::StringLiteral(From::from("test"))],
                        vec![From::from((From::from("root"), Expression::from(true)))],
                    )),
                ],
            )),
        ];

        assert_eq!(parsed, expected);
    }
}
