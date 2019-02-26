use std::borrow::{Borrow, Cow};
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::Deref;

use crate::literals::{self, Key};
use crate::Error;

use nom::{
    alt, alt_complete, call, char, complete, do_parse, many0, many1, map, named, opt, preceded,
    separated_list, tag, terminated, ws,
};

use nom::types::CompleteStr;

pub type Stanza<'a> = HashMap<Vec<String>, MapValues<'a>>;

#[derive(Debug, PartialEq, Clone)]
/// Value in HCL
pub enum Value<'a> {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    List(Vec<Value<'a>>),
    Map(Vec<MapValues<'a>>),
    Stanza(Stanza<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct MapValues<'a>(pub HashMap<Key<'a>, Value<'a>>);

impl<'a> Value<'a> {
    pub fn new_list<T>(iterator: T) -> Self
    where
        T: IntoIterator<Item = Value<'a>>,
    {
        Value::List(iterator.into_iter().collect())
    }

    pub fn new_map<I, T>(iterator: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoIterator<Item = (Key<'a>, Value<'a>)>,
    {
        Value::Map(
            iterator
                .into_iter()
                .map(|iter| iter.into_iter().collect())
                .collect(),
        )
    }

    pub fn new_single_map<T>(iterator: T) -> Self
    where
        T: IntoIterator<Item = (Key<'a>, Value<'a>)>,
    {
        Value::Map(vec![iterator.into_iter().collect()])
    }

    pub fn new_stanza<S, T>(keys: &'a [S], iterator: T) -> Self
    where
        S: AsRef<str>,
        T: IntoIterator<Item = (Key<'a>, Value<'a>)>,
    {
        let keys: Vec<String> = keys.into_iter().map(|s| s.as_ref().to_string()).collect();
        let map: MapValues = iterator.into_iter().collect();
        let stanza: Stanza = [(keys, map)].into_iter().cloned().collect();
        Value::Stanza(stanza)
    }

    pub fn variant_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "Integer",
            Value::Float(_) => "Float",
            Value::Boolean(_) => "Boolean",
            Value::String(_) => "String",
            Value::List(_) => "List",
            Value::Map(_) => "Map",
            Value::Stanza(_) => "Stanza",
        }
    }
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

impl_from_value!(Integer, i64);
impl_from_value!(Float, f64);
impl_from_value!(Boolean, bool);
impl_from_value!(String, String);
impl_from_value!(Map, Vec<MapValues<'a>>);
impl_from_value!(Stanza, Stanza<'a>);

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

impl<'a> From<MapValues<'a>> for Value<'a> {
    fn from(values: MapValues<'a>) -> Self {
        Value::from(vec![values])
    }
}

impl<'a> MapValues<'a> {
    pub fn new_from_key_value_pairs<T>(iter: T) -> Result<Self, Error>
    where
        T: IntoIterator<Item = (Key<'a>, Value<'a>)>,
    {
        use std::collections::hash_map::Entry;

        let mut map = HashMap::new();
        for (key, mut value) in iter {
            match map.entry(key) {
                Entry::Vacant(vacant) => {
                    vacant.insert(value.into());
                }
                Entry::Occupied(mut occupied) => {
                    let key = occupied.key().to_string();
                    match occupied.get_mut() {
                        illegal @ Value::Integer(_)
                        | illegal @ Value::Float(_)
                        | illegal @ Value::Boolean(_)
                        | illegal @ Value::String(_)
                        | illegal @ Value::List(_) => Err(Error::IllegalMultipleEntries {
                            key,
                            variant: illegal.variant_name(),
                        })?,
                        Value::Map(ref mut map) => {
                            // Check that the incoming value is also a Map
                            if let Value::Map(ref mut incoming) = value {
                                map.append(incoming);
                            } else {
                                Err(Error::ErrorMergingKeys {
                                    key,
                                    existing_variant: "Map",
                                    incoming_variant: value.variant_name(),
                                })?;
                            }
                        }
                        Value::Stanza(ref mut stanza) => {
                            let value = value.into();
                            // Check that the incoming value is also a Stanza
                            if let Value::Stanza(incoming) = value {
                                stanza.extend(incoming);
                            } else {
                                Err(Error::ErrorMergingKeys {
                                    key,
                                    existing_variant: "Stanza",
                                    incoming_variant: value.variant_name(),
                                })?;
                            }
                        }
                    };
                }
            };
        }
        Ok(MapValues(map))
    }
}

impl<'a> Deref for MapValues<'a> {
    type Target = HashMap<Key<'a>, Value<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> FromIterator<(Key<'a>, Value<'a>)> for MapValues<'a> {
    /// Can panic if merging fails
    fn from_iter<T: IntoIterator<Item = (Key<'a>, Value<'a>)>>(iter: T) -> Self {
        Self::new_from_key_value_pairs(iter).unwrap()
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
    pub key_value(CompleteStr) -> (Key, Value),
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
                    >> (Key::Identifier(Cow::Borrowed(identifier)), Value::from(values))
                )
                | do_parse!(
                    identifier: call!(literals::identifier)
                    >> keys: many0!(literals::quoted_single_line_string)
                    >> whitespace!(char!('{'))
                    >> values: whitespace!(call!(map_values))
                    >> char!('}')
                    >> (Key::Identifier(Cow::Borrowed(identifier)), Value::Stanza(vec![(keys, values)].into_iter().collect()))
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
        >> (values.into_iter().collect()) // FIXME: This can panic!
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
                    Value::new_list(vec![Value::from("inside voice!"), Value::from("lol")]),
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
                Value::new_list(vec![
                    Value::from(true),
                    Value::from(false),
                    Value::from(123),
                    Value::from(-123.456),
                    Value::from("foobar"),
                ]),
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
                ("test", Value::new_list(vec![Value::from(1)])),
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
                    Value::new_single_map(vec![(From::from("foo"), Value::from("bar"))]),
                ),
            ),
            (
                r#"test = {
foo = "bar"


}"#,
                (
                    "test",
                    Value::new_single_map(vec![(From::from("foo"), Value::from("bar"))]),
                ),
            ),
            (
                r#"test "one" "two" {
            foo = "bar"
            }"#,
                (
                    "test",
                    Value::new_stanza(
                        &["one", "two"],
                        vec![(From::from("foo"), Value::from("bar"))],
                    ),
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
                Value::new_list(vec![
                    Value::from(true),
                    Value::from(false),
                    Value::from(123),
                    Value::from(-123.456),
                    Value::from("foobar"),
                ]),
            ),
            (
                "list_multi",
                Value::new_list(vec![
                    Value::from(true),
                    Value::from(false),
                    Value::from(123),
                    Value::from(-123.456),
                    Value::from("foobar"),
                ]),
            ),
            (
                "list_in_list",
                Value::new_list(vec![
                    Value::new_list(vec![Value::from("test"), Value::from("foobar")]),
                    Value::from(1),
                    Value::from(2),
                    Value::from(-3),
                ]),
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

    // MapValues(
    //     {
    //         Identifier(
    //             "simple_map"
    //         ): Map(
    //             [
    //                 MapValues(
    //                     {
    //                         Identifier(
    //                             "bar"
    //                         ): String(
    //                             "baz"
    //                         ),
    //                         Identifier(
    //                             "foo"
    //                         ): String(
    //                             "bar"
    //                         )
    //                     }
    //                 ),
    //                 MapValues(
    //                     {
    //                         Identifier(
    //                             "bar"
    //                         ): String(
    //                             "baz"
    //                         ),
    //                         Identifier(
    //                             "foo"
    //                         ): String(
    //                             "bar"
    //                         )
    //                     }
    //                 )
    //             ]
    //         ),
    //         Identifier(
    //             "resource"
    //         ): Stanza(
    //             {
    //                 [
    //                     "security/group",
    //                     "second"
    //                 ]: MapValues(
    //                     {
    //                         Identifier(
    //                             "allow"
    //                         ): Map(
    //                             [
    //                                 MapValues(
    //                                     {
    //                                         Identifier(
    //                                             "name"
    //                                         ): String(
    //                                             "all"
    //                                         ),
    //                                         Identifier(
    //                                             "cidrs"
    //                                         ): List(
    //                                             [
    //                                                 String(
    //                                                     "0.0.0.0/0"
    //                                                 )
    //                                             ]
    //                                         )
    //                                     }
    //                                 )
    //                             ]
    //                         )
    //                     }
    //                 ),
    //                 [
    //                     "security/group",
    //                     "foobar"
    //                 ]: MapValues(
    //                     {
    //                         Identifier(
    //                             "deny"
    //                         ): Map(
    //                             [
    //                                 MapValues(
    //                                     {
    //                                         Identifier(
    //                                             "name"
    //                                         ): String(
    //                                             "internet"
    //                                         ),
    //                                         Identifier(
    //                                             "cidrs"
    //                                         ): List(
    //                                             [
    //                                                 String(
    //                                                     "0.0.0.0/0"
    //                                                 )
    //                                             ]
    //                                         )
    //                                     }
    //                                 )
    //                             ]
    //                         ),
    //                         Identifier(
    //                             "allow"
    //                         ): Map(
    //                             [
    //                                 MapValues(
    //                                     {
    //                                         Identifier(
    //                                             "name"
    //                                         ): String(
    //                                             "localhost"
    //                                         ),
    //                                         Identifier(
    //                                             "cidrs"
    //                                         ): List(
    //                                             [
    //                                                 String(
    //                                                     "127.0.0.1/32"
    //                                                 )
    //                                             ]
    //                                         )
    //                                     }
    //                                 ),
    //                                 MapValues(
    //                                     {
    //                                         Identifier(
    //                                             "cidrs"
    //                                         ): List(
    //                                             [
    //                                                 String(
    //                                                     "192.168.0.0/16"
    //                                                 )
    //                                             ]
    //                                         ),
    //                                         Identifier(
    //                                             "name"
    //                                         ): String(
    //                                             "lan"
    //                                         )
    //                                     }
    //                                 )
    //                             ]
    //                         )
    //                     }
    //                 )
    //             }
    //         )
    //     }
    // )
    #[test]
    fn multiple_maps_are_parsed_correctly() {
        let hcl = include_str!("../fixtures/map.hcl");
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();

        println!("{:#?}", parsed);

        let expected: HashMap<_, _> = vec![(
            "simple_map",
            Value::new_single_map(vec![
                (From::from("foo"), Value::from("bar")),
                (From::from("bar"), Value::from("baz")),
            ]),
        )]
        .into_iter()
        .collect();

        // assert_eq!(expected.iter().len(), parsed.iter().len());
        for (expected_key, expected_value) in expected {
            println!("Checking {}", expected_key);
            let actual_value = &parsed[expected_key];
            assert_eq!(*actual_value, expected_value);
        }
    }
}
