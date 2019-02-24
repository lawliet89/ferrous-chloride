use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::FromIterator;

use crate::literals;

use nom::dbg;
use nom::{
    alt, alt_complete, call, char, complete, do_parse, many0, many1, map, named, opt, preceded,
    separated_list, tag, terminated, ws,
};

use nom::types::CompleteStr;

pub type MapValues<'a> = HashMap<literals::Key<'a>, Value<'a>>;

#[derive(Debug, PartialEq, Clone)]
/// Value in HCL
pub enum Value<'a> {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    List(Vec<Value<'a>>),
    Map(Vec<Map<'a>>),
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
impl_from_value!(Map, Vec<Map<'a>>);

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

impl<'a> From<Map<'a>> for Value<'a> {
    fn from(map: Map<'a>) -> Self {
        Value::Map(vec![map])
    }
}

impl<'a> From<MapValues<'a>> for Value<'a> {
    fn from(values: MapValues<'a>) -> Self {
        Value::from(Map::from(values))
    }
}

// https://github.com/Geal/nom/blob/master/tests/json.rs
#[derive(Debug, PartialEq, Clone)]
pub struct Map<'a> {
    pub keys: Vec<String>,
    pub values: MapValues<'a>,
}

impl<'a> Map<'a> {
    pub fn new<S, K, V>(keys: &[S], values: &'a [(K, V)]) -> Self
    where
        S: AsRef<str>,
        K: AsRef<str>,
        V: Into<Value<'a>> + Clone,
    {
        Map {
            keys: keys.iter().map(|s| s.as_ref().to_string()).collect(),
            values: values
                .iter()
                .map(|(k, v)| {
                    (
                        literals::Key::Identifier(Cow::Borrowed(k.as_ref())),
                        Into::into(v),
                    )
                })
                .collect(),
        }
    }

    pub fn new_direct(keys: Vec<String>, values: MapValues<'a>) -> Self {
        Map { keys, values }
    }
}

impl<'a> From<MapValues<'a>> for Map<'a> {
    fn from(values: MapValues<'a>) -> Self {
        Map {
            keys: vec![],
            values,
        }
    }
}

// From https://github.com/Geal/nom/issues/14#issuecomment-158788226
// whitespace! Must not be captured after `]`!
named!(
    pub list(CompleteStr) -> Vec<Value>,
    preceded!(
        whitespace!(char!('[')),
        terminated!(
            whitespace!(
                separated_list!(
                    char!(','),
                    single_value
                )
            ),
            terminated!(
                whitespace!(opt!(char!(','))),
                char!(']')
            )
        )
    )
);

named!(
    pub single_value(CompleteStr) -> Value,
    alt_complete!(
        call!(literals::number) => { |v| From::from(v) }
        | call!(literals::boolean) => { |v| Value::Boolean(v) }
        | literals::string => { |v| Value::String(v) }
        | list => { |v| Value::List(v) }
    )
);

/// Parse values of the form "key" = ... | ["..."] | {...}
named!(
    pub key_value(CompleteStr) -> (literals::Key, Value),
    space_tab!(
        alt!(
                do_parse!(
                    key: call!(literals::key)
                    >> char!('=')
                    >> value: call!(single_value)
                    >> (key, value)
                )
                | do_parse!(
                    identifier: call!(literals::identifier)
                    >> complete!(opt!(char!('=')))
                    >> whitespace!(char!('{'))
                    >> values: whitespace!(call!(map_values))
                    >> char!('}')
                    >> (literals::Key::Identifier(Cow::Borrowed(identifier)), Value::from(values))
                )
                | do_parse!(
                    identifier: call!(literals::identifier)
                    >> keys: many0!(literals::quoted_single_line_string)
                    >> whitespace!(char!('{'))
                    >> values: whitespace!(call!(map_values))
                    >> char!('}')
                    >> (literals::Key::Identifier(Cow::Borrowed(identifier)), Value::from(Map::new_direct(keys, values)))
                )
        )
    )
);

named!(
    pub map_values(CompleteStr) -> MapValues,
    do_parse!(
        values: many0!(
                    terminated!(
                        call!(key_value),
                        alt!(
                            whitespace!(tag!(","))
                            | map!(many1!(nom::eol), |_| CompleteStr(""))
                        )
                    )
                )
        >> (values.into_iter().collect())
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::ResultUtilsString;

    #[test]
    fn list_values_are_parsed_successfully() {
        let test_cases = [
            (r#"[]"#, vec![]),
            (r#"[1,]"#, vec![Value::from(1)]),
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
            (
                r#"[
                        true,
                        false,
                        123,
                        -123.456,
                        "testing",                        [
                            "inside voice!",
                            "lol"
                        ],
                    ]"#,
                vec![
                    Value::from(true),
                    Value::from(false),
                    Value::from(123),
                    Value::from(-123.456),
                    Value::from("testing"),
                    [Value::from("inside voice!"), Value::from("lol")]
                        .into_iter()
                        .collect(),
                ],
            ),
        ];

        for (input, expected_value) in test_cases.into_iter() {
            println!("Testing {}", input);
            let actual_value = list(CompleteStr(input)).unwrap_output();
            assert_eq!(actual_value, *expected_value);
        }
    }

    #[test]
    fn single_values_are_parsed_successfully() {
        let test_cases = [
            (r#"123"#, Value::Integer(123)),
            ("123", Value::Integer(123)),
            ("123", Value::Integer(123)),
            ("true", Value::Boolean(true)),
            ("123.456", Value::Float(123.456)),
            ("123", Value::Integer(123)),
            (r#""foobar""#, Value::String("foobar".to_string())),
            (
                r#"<<EOF
new
line
EOF
"#,
                Value::String("new\nline".to_string()),
            ),
            (
                r#"[true, false, 123, -123.456, "foobar"]"#,
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
        ];

        for (input, expected_value) in test_cases.into_iter() {
            println!("Testing {}", input);
            let actual_value = single_value(CompleteStr(input)).unwrap_output();
            assert_eq!(actual_value, *expected_value);
        }
    }

    #[test]
    fn key_value_pairs_are_parsed_successfully() {
        let test_cases = [
            ("test = 123", ("test", Value::Integer(123))),
            ("test = 123", ("test", Value::Integer(123))),
            ("test = true", ("test", Value::Boolean(true))),
            ("test = 123.456", ("test", Value::Float(123.456))),
            ("   test   =   123  ", ("test", Value::Integer(123))), // Random spaces
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
            (r#"test = [],"#, ("test", Value::List(vec![]))),
            (
                r#"test = [1,]"#,
                ("test", [Value::from(1)].into_iter().collect()),
            ),
            (
                r#"test = [true, false, 123, -123.456, "foobar"],"#,
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
            let (actual_key, actual_value) = key_value(CompleteStr(input)).unwrap_output();
            assert_eq!(actual_key.unwrap(), *expected_key);
            assert_eq!(actual_value, *expected_value);
        }
    }

    #[test]
    fn maps_are_parsed_correctly() {
        let test_cases = [
            (
                r#"test {
foo = "bar"
}"#,
                (
                    "test",
                    Value::from(Map::new::<&str, _, _>(&[], &[("foo", "bar")])),
                ),
            ),
            (
                r#"test = {
foo = "bar"


}"#,
                (
                    "test",
                    Value::from(Map::new::<&str, _, _>(&[], &[("foo", "bar")])),
                ),
            ),
            (
                r#"test "one" "two" {
foo = "bar"
}"#,
                (
                    "test",
                    Value::from(Map::new(&["one", "two"], &[("foo", "bar")])),
                ),
            ),
        ];

        for (input, (expected_key, expected_value)) in test_cases.into_iter() {
            println!("Testing {}", input);
            let (actual_key, actual_value) = key_value(CompleteStr(input)).unwrap_output();
            assert_eq!(actual_key.unwrap(), *expected_key);
            assert_eq!(actual_value, *expected_value);
        }
    }

    #[test]
    fn empty_map_values_are_parsed_correctly() {
        let hcl = include_str!("../fixtures/empty.hcl");
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();

        assert_eq!(0, parsed.len());
    }

    #[test]
    fn single_map_values_are_parsed_correctly() {
        let hcl = include_str!("../fixtures/single.hcl");
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();

        assert_eq!(1, parsed.len());
        assert_eq!(parsed["foo"], Value::from("bar"));
    }

    #[test]
    fn scalar_map_values_are_parsed_correctly() {
        let hcl = include_str!("../fixtures/scalar.hcl");
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();

        let expected: HashMap<_, _> = vec![
            ("test_unsigned_int", Value::from(123)),
            ("test_signed_int", Value::from(-123)),
            ("test_float", Value::from(-1.23)),
            ("bool_true", Value::from(true)),
            ("bool_false", Value::from(false)),
            ("comma_separed", Value::from("oh my, a rebel!")),
            ("string", Value::from("Hello World!")),
            ("long_string", Value::from("hihi\nanother line!")),
            ("string_escaped", Value::from("\" Hello World!")),
        ]
        .into_iter()
        .collect();

        assert_eq!(expected.iter().len(), parsed.iter().len());
        for (expected_key, expected_value) in expected {
            println!("Checking {}", expected_key);
            let actual_value = &parsed[expected_key];
            assert_eq!(*actual_value, expected_value);
        }
    }

    #[test]
    fn list_map_values_are_parsed_correctly() {
        let hcl = include_str!("../fixtures/list.hcl");
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();

        let expected: HashMap<_, _> = vec![
            (
                "list",
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
            (
                "list_multi",
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
            (
                "list_in_list",
                [
                    [Value::from("test"), Value::from("foobar")]
                        .into_iter()
                        .collect(),
                    Value::from(1),
                    Value::from(2),
                    Value::from(-3),
                ]
                .into_iter()
                .collect(),
            ),
        ]
        .into_iter()
        .collect();

        assert_eq!(expected.iter().len(), parsed.iter().len());
        for (expected_key, expected_value) in expected {
            println!("Checking {}", expected_key);
            let actual_value = &parsed[expected_key];
            assert_eq!(*actual_value, expected_value);
        }
    }
}
