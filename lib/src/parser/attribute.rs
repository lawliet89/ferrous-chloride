use std::borrow::Cow;

use nom::types::CompleteStr;
use nom::{call, char, named};

use super::expression::{expression, Expression};
use super::literals;
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
            identifier: call!(literals::identifier)
            >> char!('=')
            >> expression: call!(expression)
            >> (Cow::Borrowed(identifier), expression)
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Value;

    #[test]
    fn attribute_pairs_are_parsed_successfully() {
        let test_cases = [
            ("test = 123", ("test", Value::Integer(123)), ""),
            ("test = 123", ("test", Value::Integer(123)), ""),
            ("test = true", ("test", Value::Boolean(true)), ""),
            ("test = 123.456", ("test", Value::Float(123.456)), ""),
            ("   test   =   123  ", ("test", Value::Integer(123)), ""), // Random spaces
            (
                r#"test = <<EOF
new
line
EOF
"#,
                ("test", Value::String("new\nline".to_string())),
                "\n",
            ),
            (r#"test = [],"#, ("test", Value::List(vec![])), ","),
            (
                r#"test = [1,]"#,
                ("test", Value::new_list(vec![Value::from(1)])),
                "",
            ),
            (
                r#"test = [true, false, 123, -123.456, "foobar"],"#,
                (
                    "test",
                    Value::new_list(vec![
                        Value::from(true),
                        Value::from(false),
                        Value::from(123),
                        Value::from(-123.456),
                        Value::from("foobar"),
                    ]),
                ),
                ",",
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
