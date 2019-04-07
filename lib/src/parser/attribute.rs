use std::borrow::Cow;

use nom::types::CompleteStr;
use nom::{call, char, named};

use crate::parser::expression::{expression, Expression};
use crate::parser::identifier::identifier;

/// A HCL Attribute
///
/// ```ebnf
/// Attribute = Identifier "=" Expression Newline;
/// ```
pub type Attribute<'a> = (Cow<'a, str>, Expression<'a>);

named!(
    pub attribute(CompleteStr) -> Attribute,
    inline_whitespace!(
        do_parse!(
            identifier: call!(identifier)
            >> char!('=')
            >> expression: call!(expression)
            >> (Cow::Borrowed(identifier), expression)
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attribute_pairs_are_parsed_successfully() {
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
            (
                r#"list_multi = [ # hmm
    true, # Comment
    false, // Test
    123, /* Comment */
    # "non-existent",
    /* Comment */ -123.456,
    "foobar",
]
"#,
                (
                    "list_multi",
                    Expression::new_tuple(vec![
                        From::from(true),
                        From::from(false),
                        From::from(123),
                        From::from(-123.456),
                        From::from("foobar"),
                    ]),
                ),
                "\n",
            ),
            (
                r#"list_in_list = [ // This should work
    [/* Inline start */ "test", "foobar"],
    1,
    2,
    -3,
]
"#,
                (
                    "list_in_list",
                    Expression::new_tuple(vec![
                        Expression::new_tuple(vec![From::from("test"), From::from("foobar")]),
                        From::from(1),
                        From::from(2),
                        From::from(-3),
                    ]),
                ),
                "\n",
            ),
            (
                r#"object_in_list = [ /* This too! */
    {
        test = 123
    },
    {
        foo = "bar"
    },
    {
        baz = false,
    },
]
"#,
                (
                    "object_in_list",
                    Expression::new_tuple(vec![
                        Expression::new_object(vec![("test", Expression::from(123))]),
                        Expression::new_object(vec![("foo", Expression::from("bar"))]),
                        Expression::new_object(vec![("baz", Expression::from(false))]),
                    ]),
                ),
                "\n",
            ),
        ];

        for (input, (expected_key, expected_value), expected_remaining) in test_cases.iter() {
            println!("Testing {}", input);
            let (remaining, (actual_identifier, actual_expression)) =
                attribute(CompleteStr(input)).unwrap();
            assert_eq!(&remaining.0, expected_remaining);
            assert_eq!(actual_identifier, *expected_key);
            assert_eq!(actual_expression, *expected_value);
        }
    }
}
