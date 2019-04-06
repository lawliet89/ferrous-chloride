//! Object
//!
//! An Object is part of `CollectionValue`
//!
//! [Reference](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#collection-values)
//!
//! ```ebnf
//! object = "{" (
//!     (objectelem ("," objectelem)* ","?)?
//! ) "}";
//! objectelem = (Identifier | Expression) "=" Expression;
//! ```
use std::borrow::Cow;

use nom::types::CompleteStr;
use nom::{
    alt, alt_complete, call, char, complete, do_parse, eof, exact, named, opt, peek, preceded,
    recognize, tag, terminated, IResult,
};

use super::attribute::attribute;
use super::expression::{expression, Expression};
use crate::parser::literals::{identifier, newline};
use crate::HashMap;

// TODO: Dealing with expressions and ambiguity. See reference
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ElementIdentifier<'a> {
    /// A literal attribute name
    Identifier(Cow<'a, str>),
    /// An expression that must evaluate to a string
    ///
    /// The HCL [specification](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#collection-values)
    /// allows Object Element identifiers to be expressions, but the
    /// [HCL Syntax-Agnostic Information Model](https://github.com/hashicorp/hcl2/blob/master/hcl/spec.md#structural-types)
    /// states that "\[object\] attribute names are always strings".
    ///
    /// This variant preserves an Expression used as the identifier as an unparsed tokens.
    /// Users are expected to parse and process the expression in a manner that is appropriate for
    /// the semantics for their application.
    Expression(Cow<'a, str>),
}

impl<'a, S> PartialEq<S> for ElementIdentifier<'a>
where
    S: AsRef<str>,
{
    fn eq(&self, other: &S) -> bool {
        match self {
            ElementIdentifier::Identifier(ident) => ident.eq(other.as_ref()),
            ElementIdentifier::Expression(_) => false,
        }
    }
}

pub type ObjectElement<'a> = (ElementIdentifier<'a>, Expression<'a>);

pub type Object<'a> = HashMap<ElementIdentifier<'a>, Expression<'a>>;

// Cannot use `named!` because the compiler cannot determine the lifetime
pub fn element_identifier<'a>(
    input: CompleteStr<'a>,
) -> IResult<CompleteStr<'a>, ElementIdentifier<'a>, u32> {
    alt!(
        input,
        call!(identifier) =>
            { |ident| ElementIdentifier::Identifier(Cow::Borrowed(ident)) }
        | recognize!(call!(expression)) =>
            { |expr: CompleteStr<'a>| ElementIdentifier::Expression(Cow::Borrowed(expr.0)) }
    )
}

named!(
    pub object_element(CompleteStr) -> ObjectElement,
    inline_whitespace!(
        do_parse!(
            identifier: call!(element_identifier)
            >> char!('=')
            >> expression: call!(expression)
            >> (identifier, expression)
        )
    )
);

// named!(
//     pub object_elements(CompleteStr) -> Vec<ObjectElement>,
//     do_parse!(
//         values: whitespace!(
//             many0!(
//                 terminated!(
//                     call!(attribute),
//                     alt!(
//                         whitespace!(tag!(","))
//                         | call!(newline) => { |_| CompleteStr("") }
//                         | eof!()
//                     )
//                 )
//             )
//         )
//         >> (values.into_iter().collect())
//     )
// );

// // named!(
// //     pub object(CompleteStr) -> Object,
// //     do_parse!(
// //         whitespace!(char!('{'))
// //         >> values: whitespace!(call!(map_values))
// //         >> char!('}')
// //         >> (values)
// //     )
// // );

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::ResultUtilsString;

    #[test]
    fn element_identifiers_are_parsed_correctly() {
        let test_cases = [
            (
                "foobar",
                ElementIdentifier::Identifier(Cow::Borrowed("foobar")),
            ),
            ("true", ElementIdentifier::Identifier(Cow::Borrowed("true"))),
            (
                "(true)",
                ElementIdentifier::Expression(Cow::Borrowed("(true)")),
            ),
            (
                "(1234)",
                ElementIdentifier::Expression(Cow::Borrowed("(1234)")),
            ),
        ];

        for (input, expected_output) in &test_cases {
            let output = element_identifier(CompleteStr(input)).unwrap_output();
            assert_eq!(output, *expected_output);
        }
    }

    #[test]
    fn element_objects_are_parsed_successfully() {
        let test_cases = [
            (
                "test = 123",
                ("test", Expression::Number(From::from(123))),
                "",
            ),
            (
                "test = 123",
                ("test", Expression::Number(From::from(123))),
                "",
            ),
            ("test = true", ("test", Expression::Boolean(true)), ""),
            (
                "test = 123.456",
                ("test", Expression::Number(From::from(123.456))),
                "",
            ),
            (
                "   test   =   123  ",
                ("test", Expression::Number(From::from(123))),
                "",
            ), // Random spaces
            (
                r#"test = <<EOF
new
line
EOF
"#,
                ("test", Expression::String("new\nline".to_string())),
                "\n",
            ),
            (r#"test = [],"#, ("test", Expression::Tuple(vec![])), ","),
            (
                r#"test = [1,]"#,
                ("test", Expression::new_tuple(vec![From::from(1)])),
                "",
            ),
            (
                r#"test = [true, false, 123, -123.456, "foobar"],"#,
                (
                    "test",
                    Expression::new_tuple(vec![
                        From::from(true),
                        From::from(false),
                        From::from(123),
                        From::from(-123.456),
                        From::from("foobar"),
                    ]),
                ),
                ",",
            ),
        ];

        for (input, (expected_key, expected_value), expected_remaining) in test_cases.iter() {
            println!("Testing {}", input);
            let (remaining, (actual_identifier, actual_expression)) =
                object_element(CompleteStr(input)).unwrap();
            assert_eq!(&remaining.0, expected_remaining);
            assert_eq!(actual_identifier, *expected_key);
            assert_eq!(actual_expression, *expected_value);
        }
    }
}
