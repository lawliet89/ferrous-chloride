use nom::types::CompleteStr;
use nom::{alt_complete, call, named};

use super::literals;
use super::{list, map_expression};
use crate::value::Value;

// FIXME: For now
pub type Expression<'a> = Value<'a>;

named!(
    pub expression(CompleteStr) -> Value,
    alt_complete!(
        call!(literals::null) => { |_| Value::Null }
        | call!(literals::number) => { |v| From::from(v) }
        | call!(literals::boolean) => { |v| Value::Boolean(v) }
        | literals::string => { |v| Value::String(v) }
        | list => { |v| Value::List(v) }
        | map_expression => { |m| Value::Object(vec![m]) }
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parser::literals::Key;

    #[test]
    fn expressions_are_parsed_successfully() {
        let test_cases = [
            ("null", Value::Null, ""),
            (r#"123"#, Value::Integer(123), ""),
            ("123", Value::Integer(123), ""),
            ("123", Value::Integer(123), ""),
            ("true", Value::Boolean(true), ""),
            ("123.456", Value::Float(123.456), ""),
            ("123", Value::Integer(123), ""),
            (r#""foobar""#, Value::String("foobar".to_string()), ""),
            (
                r#"<<EOF
new
line
EOF
"#,
                Value::String("new\nline".to_string()),
                "\n",
            ),
            (
                r#"[true, false, 123, -123.456, "foobar"]"#,
                Value::new_list(vec![
                    Value::from(true),
                    Value::from(false),
                    Value::from(123),
                    Value::from(-123.456),
                    Value::from("foobar"),
                ]),
                "",
            ),
            (
                r#"{
        test = 123
}"#,
                Value::new_map(vec![vec![(Key::new_identifier("test"), Value::from(123))]]),
                "",
            ),
        ];

        for (input, expected_value, expected_remaining) in test_cases.iter() {
            println!("Testing {}", input);
            let (remaining, actual_value) = expression(CompleteStr(input)).unwrap();
            assert_eq!(&remaining.0, expected_remaining);
            assert_eq!(actual_value, *expected_value);
        }
    }

}