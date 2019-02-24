use std::collections::HashMap;

use crate::literals;

use nom::{
    alt, alt_complete, call, char, delimited, do_parse, many0, map, named, opt, preceded,
    separated_list, tag, terminated, ws,
};

use nom::dbg;

pub type Map<'a> = HashMap<literals::Key<'a>, Value<'a>>;

#[derive(Debug, PartialEq, Clone)]
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

macro_rules! impl_from_value (
    ($variant: ident, $type: ty) => (
        impl<'a> From<$type> for Value<'a> {
            fn from(v: $type) -> Self {
                Value::$variant(v)
            }
        }
    )
);

impl<'a, 'b, T> From<&'b T> for Value<'a>
where
    T: Into<Value<'a>> + Clone,
{
    fn from(v: &'b T) -> Value<'a> {
        Into::into(v.clone())
    }
}

impl<'a, A> std::iter::FromIterator<A> for Value<'a>
where
    A: Into<Value<'a>>,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: std::iter::IntoIterator<Item = A>,
    {
        Value::List(iter.into_iter().map(|v| Into::into(v)).collect())
    }
}

impl_from_value!(Integer, i64);
impl_from_value!(Float, f64);
impl_from_value!(Boolean, bool);
impl_from_value!(String, String);
impl_from_value!(Stanza, Stanza<'a>);
impl_from_value!(Map, Map<'a>);

/// Special Snowflake treatment for &str and friends
impl<'a, 'b> From<&'b str> for Value<'a> {
    fn from(s: &'b str) -> Self {
        Value::String(s.to_string())
    }
}

impl<'a> From<Option<Vec<Value<'a>>>> for Value<'a> {
    fn from(l: Option<Vec<Value<'a>>>) -> Self {
        match l {
            None => Value::List(vec![]),
            Some(v) => Value::List(v),
        }
    }
}

// https://github.com/Geal/nom/blob/master/tests/json.rs
#[derive(Debug, PartialEq, Clone)]
pub struct Stanza<'a> {
    pub keys: Vec<String>,
    pub values: Map<'a>,
}

// From https://github.com/Geal/nom/issues/14#issuecomment-158788226
named!(
    pub list(&str) -> Vec<Value>,
    ws!(
        preceded!(
            char!('['),
            terminated!(
                separated_list!(
                    dbg!(char!(',')),
                    value
                ),
                terminated!(
                    opt!(char!(',')),
                    char!(']')
                )
            )
        )
    )
);

// named!(
//     pub list(&str) -> Vec<Value>,
//     ws!(
//         do_parse!(
//             char!('[')
//             >> first: call!(value)
//             >> rest: opt!(many0!(
//                 ws!(
//                     do_parse!(
//                         dbg!(char!(','))
//                         >> value: call!(value)
//                         >> (value)
//                     )
//                 )
//             ))
//             >> opt!(char!(','))
//             >> char!(']')
//             // >> (first, rest)
//             >> (rest.unwrap())
//         )
//     )
// );

named!(
    pub value(&str) -> Value,
    alt_complete!(
        call!(crate::utils::wrap_str(literals::number)) => { |v| From::from(v) }
        | call!(crate::utils::wrap_str(literals::boolean)) => { |v| Value::Boolean(v) }
        | literals::string => { |v| Value::String(v) }
        | list => { |v| Value::List(v) }
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
    fn scalar_values_are_parsed_successfully() {
        let test_cases = [
            (r#"123"#, Value::Integer(123)), // Comma separated
            ("123", Value::Integer(123)),    // New line
            ("123", Value::Integer(123)),    // Windows New line
            ("true", Value::Boolean(true)),
            ("123.456", Value::Float(123.456)),
            ("123", Value::Integer(123)), // Random spaces
            (r#""foobar""#, Value::String("foobar".to_string())),
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
    fn list_values_are_parsed_successfully() {
        let test_cases = [
            // (r#"[]"#, vec![]),
            (
                r#"[true, false, 123, -123.456, "foobar"]"#,
                vec![
                    Value::from(true),
                    Value::from(false),
                    Value::from(123),
                    Value::from(-123.456),
                    Value::from("foobar"),
                ],
            ),
        ];

        for (input, expected_value) in test_cases.into_iter() {
            println!("Testing {}", input);
            let actual_value = list(input).unwrap_output();

            println!("{:#?}", actual_value);

            // assert_eq!(actual_value, *expected_value);
        }

        //         let input = r#"[
        // true,
        // false,
        // 123,
        // -123.456,
        // "testing",
        // [
        //     "inside voice!",
        //     "lol"
        // ]
        // ]"#;
        //         let expected = [
        //             Value::from(true),
        //             Value::from(false),
        //             Value::from(123),
        //             Value::from(-123.456),
        //             Value::from("test"),
        //             [Value::from("inside voice!"), Value::from("lol")]
        //                 .into_iter()
        //                 .collect(),
        //         ];

        //         let parsed = list(input).unwrap_output();
        //         assert_eq!(parsed, expected);
    }

    #[test]
    fn scalar_key_value_pairs_are_parsed_successfully() {
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
            (
                r#"[true, false, 123, -123.456, "foobar"]"#,
                (
                    "test",
                    [
                        Value::from(true),
                        Value::from(false),
                        Value::from(123),
                        Value::from(-123.456),
                        Value::from("foobar"),
                    ]
                    .into_iter()
                    .collect(),
                ),
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
