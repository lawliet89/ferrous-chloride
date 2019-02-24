use std::collections::HashMap;

use crate::literals;

use nom::{alt, call, char, delimited, named, separated_list, tag, terminated, ws};

pub type Map<'a> = HashMap<literals::Key<'a>, Value<'a>>;

#[derive(Debug, PartialEq)]
/// Value in HCL
pub enum Value<'a> {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    List(Vec<Value<'a>>),
    Stanza(Stanza<'a>),
    Map(Map<'a>),
}

// https://github.com/Geal/nom/blob/master/tests/json.rs
#[derive(Debug, PartialEq)]
pub struct Stanza<'a> {
    pub keys: Vec<String>,
    pub values: Map<'a>,
}

// named!(
//     pub list(&str) -> Vec<Value>,
//     ws!(
//         delimited!(
//             char!('['],
//             separated_list!(char!(','), value),
//         )
//     )
// )

named!(
    pub value(&str) -> Value,
    alt!(
        call!(crate::utils::wrap_str(literals::number)) => { |v| From::from(v) }
        | call!(crate::utils::wrap_str(literals::boolean)) => { |v| Value::Boolean(v) }
        | literals::string => { |v| Value::String(v) }
    )
);

/// Parse values of the form "key" = ... | ["..."] | {...}
named!(
    pub key_value(&str) -> (literals::Key, Value),
    terminated!(
        whitespace!(
            do_parse!(
                key: call!(literals::key)
                >> char!('=')
                >> value: call!(value)
                >> (key, value)
            )
        ),
        alt!(
            tag!(",")
            | call!(nom::eol)
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::ResultUtilsString;

    #[test]
    fn values_are_parsed_successfully() {
        let test_cases = [
            (r#"123"#, Value::Integer(123)), // Comma separated
            ("123", Value::Integer(123)),   // New line
            ("123", Value::Integer(123)), // Windows New line
            ("true", Value::Boolean(true)),
            ("123.456", Value::Float(123.456)),
            ("123", Value::Integer(123)), // Random spaces
            (
                r#""foobar""#,
                Value::String("foobar".to_string()),
            ),
            (
                r#"<<EOF
new
line
EOF
"#,
                Value::String("new\nline".to_string()),
            ),
        ];

        for (input, expected_value) in test_cases.into_iter() {
            println!("Testing {}", input);
            let actual_value = value(input).unwrap_output();
            assert_eq!(actual_value, *expected_value);
        }
    }

    #[test]
    fn key_value_pairs_are_parsed_successfully() {
        let test_cases = [
            (r#"test = 123,"#, ("test", Value::Integer(123))), // Comma separated
            ("test = 123\n", ("test", Value::Integer(123))),   // New line
            ("test = 123\r\n", ("test", Value::Integer(123))), // Windows New line
            ("test = true\n", ("test", Value::Boolean(true))),
            ("test = 123.456\n", ("test", Value::Float(123.456))),
            ("   test   =   123  \n", ("test", Value::Integer(123))), // Random spaces
            (
                r#""a/b/c" = "foobar","#,
                ("a/b/c", Value::String("foobar".to_string())),
            ),
            (
                r#"test = <<EOF
new
line
EOF
"#,
                ("test", Value::String("new\nline".to_string())),
            ),
        ];

        for (input, (expected_key, expected_value)) in test_cases.into_iter() {
            println!("Testing {}", input);
            let (actual_key, actual_value) = key_value(input).unwrap_output();
            assert_eq!(actual_key.unwrap(), *expected_key);
            assert_eq!(actual_value, *expected_value);
        }
    }
}
