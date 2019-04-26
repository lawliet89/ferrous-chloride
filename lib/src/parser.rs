pub(crate) mod literals;

#[macro_use]
pub mod whitespace;
pub mod attribute;
pub mod block;
pub mod body;
pub mod boolean;
pub mod expression;
pub mod identifier;
pub mod null;
pub mod number;
pub mod object;
pub mod string;
pub mod tuple;

#[doc(inline)]
pub use attribute::Attribute;
#[doc(inline)]
pub use block::{Block, Blocks};
#[doc(inline)]
pub use body::Body;
#[doc(inline)]
pub use expression::Expression;

use std::borrow::Cow;

use crate::value::{self, MapValues, Value};
use crate::{AsOwned, Error};
use literals::Key;
use whitespace::newline;

use nom::types::CompleteStr;
use nom::{
    alt, alt_complete, call, char, complete, do_parse, eof, exact, named, named_attr, opt,
    preceded, tag, terminated,
};

/// A HCL document body
///
/// ```ebnf
/// ConfigFile   = Body;
/// Body         = (Attribute | Block | OneLineBlock)*;
/// Attribute    = Identifier "=" Expression Newline;
/// Block        = Identifier (StringLit|Identifier)* "{" Newline Body "}" Newline;
/// OneLineBlock = Identifier (StringLit|Identifier)* "{" (Identifier "=" Expression)? "}" Newline;
/// ```
pub(crate) type OldBody<'a> = value::MapValues<'a>;

named!(
    list_begin(CompleteStr) -> char,
    char!('[')
);

named!(
    list_separator(CompleteStr) -> char,
    char!(',')
);

// From https://github.com/Geal/nom/issues/14#issuecomment-158788226
// whitespace! Must not be captured after `]`!
named!(
    pub(crate) list(CompleteStr) -> Vec<Value>,
    preceded!(
        list_begin,
        terminated!(
            whitespace!(
                separated_list!(
                    list_separator,
                    single_value
                )
            ),
            terminated!(
                whitespace!(opt!(list_separator)),
                char!(']')
            )
        )
    )
);

named!(
    pub(crate) single_value(CompleteStr) -> Value,
    alt_complete!(
        call!(null::null) => { |_| Value::Null }
        | call!(literals::number) => { |v| From::from(v) }
        | call!(boolean::boolean) => { |v| Value::Boolean(v) }
        | string::string => { |v: Cow<str>| Value::String(v.to_string()) }
        | list => { |v| Value::List(v) }
        | map_expression => { |m| Value::Object(vec![m]) }
    )
);

named!(
    pub(crate) map_expression(CompleteStr) -> MapValues,
    do_parse!(
        whitespace!(char!('{'))
        >> values: whitespace!(call!(map_values))
        >> char!('}')
        >> (values)
    )
);

// Parse single key value pair in the form of
// `"key" = ... | ["..."] | {...}`
named!(
    pub(crate) attribute(CompleteStr) -> (Key, Value),
    inline_whitespace!(
        alt!(
            do_parse!(
                key: call!(literals::key)
                >> char!('=')
                >> value: call!(single_value)
                >> (key, value)
            )
            | do_parse!(
                identifier: call!(identifier::identifier)
                >> complete!(opt!(char!('=')))
                >> values: call!(map_expression)
                >> (Key::Identifier(identifier), Value::from(values))
            )
            | do_parse!(
                identifier: call!(identifier::identifier)
                >> keys: many0!(string::string_literal)
                >> values: call!(map_expression)
                >> (Key::Identifier(identifier), Value::Block(vec![(keys, values)].into_iter().collect()))
            )
        )
    )
);

named!(
    pub(crate) map_values(CompleteStr) -> MapValues,
    do_parse!(
        values: whitespace!(
            many0!(
                terminated!(
                    call!(attribute),
                    alt!(
                        whitespace!(tag!(","))
                        | call!(newline) => { |_| CompleteStr("") }
                        | eof!()
                    )
                )
            )
        )
        >> (values.into_iter().collect())
    )
);

named!(
    pub(crate) body(CompleteStr) -> OldBody,
    exact!(call!(map_values))
);

/// A HCL Configuration File
///
/// ```ebnf
/// ConfigFile = Body;
/// ```
///
/// See the [HCL specification](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md)
/// for more information on the file format.
pub type ConfigFile<'a> = self::body::Body<'a>;

named_attr!(
    #[doc = r#"Parse a HCL Configuration file.

This is a low level parsing function that will return a [`nom::IResult`]. You should use the
functions in the [`parser`](crate::parser) module instead.

When parsing the file, all input must be consumed or a parse error will be raised.

See the [HCL specification](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md)
for more information on the file format.
"#],
    pub config_file(CompleteStr) -> ConfigFile,
    exact!(call!(self::body::body))
);

/// Parse a HCL string into a [`ConfigFile`] which is close to an abstract syntax tree of the
/// HCL string.
pub fn parse_str(input: &str) -> Result<ConfigFile, Error> {
    let (remaining_input, body) =
        config_file(CompleteStr(input)).map_err(|e| Error::from_err_str(&e))?;

    if !remaining_input.is_empty() {
        Err(Error::Bug(format!(
            r#"Input was not completely parsed:
Input: {},
Remaining: {}
"#,
            input, remaining_input
        )))?
    }

    Ok(body)
}

/// Parse a HCL string from a IO stream reader
///
/// The entire IO stream has to be buffered in memory first before parsing can occur.
///
/// When reading from a source against which short reads are not efficient, such as a
/// [`File`](std::fs::File), you will want to apply your own buffering because the library
/// will not buffer the input. See [`std::io::BufReader`].
pub fn parse_reader<R: std::io::Read>(mut reader: R) -> Result<ConfigFile<'static>, Error> {
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;

    let parsed = parse_str(&buffer)?;
    Ok(parsed.as_owned())
}

/// Parse a HCL string from a slice of bytes
pub fn parse_slice(bytes: &[u8]) -> Result<ConfigFile, Error> {
    let input = std::str::from_utf8(bytes)?;
    parse_str(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use crate::fixtures;
    use crate::utils::{assert_list_eq, ResultUtilsString};
    use crate::value::Block;
    use crate::{Mergeable, ScalarLength};

    #[test]
    fn strings_are_parsed_correctly() {
        for string in fixtures::ALL {
            let _ = parse_str(string).unwrap();
        }
    }

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

        for (input, expected_value) in test_cases.iter() {
            println!("Testing {}", input);
            let actual_value = list(CompleteStr(input)).unwrap_output();
            assert_eq!(actual_value, *expected_value);
        }
    }

    #[test]
    fn single_values_are_parsed_successfully() {
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
            let (remaining, actual_value) = single_value(CompleteStr(input)).unwrap();
            assert_eq!(&remaining.0, expected_remaining);
            assert_eq!(actual_value, *expected_value);
        }
    }

    #[test]
    fn map_expressions_are_parsed_correctly() {
        let test_cases = [
            (
                r#"{
foo = "bar"
}"#,
                MapValues::new_unmerged(vec![(From::from("foo"), Value::from("bar"))]),
            ),
            (
                r#"{
foo = "bar"


}"#,
                MapValues::new_unmerged(vec![(From::from("foo"), Value::from("bar"))]),
            ),
            (
                r#"{
            foo = "bar"
            }"#,
                MapValues::new_unmerged(vec![(From::from("foo"), Value::from("bar"))]),
            ),
        ];

        for (input, expected_value) in test_cases.iter() {
            println!("Testing {}", input);
            let actual_value = map_expression(CompleteStr(input)).unwrap_output();
            assert_eq!(actual_value, *expected_value);
        }
    }

    #[test]
    fn attribute_pairs_are_parsed_successfully() {
        let test_cases = [
            ("test = 123", ("test", Value::Integer(123)), ""),
            ("test = 123", ("test", Value::Integer(123)), ""),
            ("test = true", ("test", Value::Boolean(true)), ""),
            ("test = 123.456", ("test", Value::Float(123.456)), ""),
            ("   test   =   123  ", ("test", Value::Integer(123)), ""), // Random spaces
            (
                r#""a/b/c" = "foobar","#,
                ("a/b/c", Value::String("foobar".to_string())),
                ",",
            ),
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
            let (remaining, (actual_key, actual_value)) = attribute(CompleteStr(input)).unwrap();
            assert_eq!(&remaining.0, expected_remaining);
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
                    Value::new_block(
                        &["one", "two"],
                        vec![(From::from("foo"), Value::from("bar"))],
                    ),
                ),
            ),
        ];

        for (input, (expected_key, expected_value)) in test_cases.iter() {
            println!("Testing {}", input);
            let (actual_key, actual_value) = attribute(CompleteStr(input)).unwrap_output();
            assert_eq!(actual_key.unwrap(), *expected_key);
            assert_eq!(actual_value, *expected_value);
        }
    }

    // map_values

    #[test]
    fn empty_map_values_are_parsed_correctly() {
        let hcl = "";
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();

        assert_eq!(0, parsed.len());
    }

    #[test]
    fn non_terminating_new_lines_are_parsed_correctly() {
        let hcl = fixtures::NO_NEWLINE_EOF;
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();

        assert_eq!(1, parsed.len());
        assert_eq!(parsed["test"], Value::from(true));
    }

    #[test]
    fn single_map_values_are_parsed_correctly() {
        let hcl = fixtures::SINGLE;
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();

        assert_eq!(1, parsed.len());
        assert_eq!(parsed["foo"], Value::from("bar"));
    }

    #[test]
    fn scalar_map_values_are_parsed_correctly() {
        let hcl = fixtures::SCALAR;
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();

        let expected: HashMap<_, _> = vec![
            ("test_unsigned_int", Value::from(123)),
            ("test_signed_int", Value::from(-123)),
            ("test_float", Value::from(-1.23)),
            ("bool_true", Value::from(true)),
            ("bool_false", Value::from(false)),
            ("string", Value::from("Hello World!")),
            ("long_string", Value::from("hihi\nanother line!")),
            ("string_escaped", Value::from("\" Hello World!")),
        ]
        .into_iter()
        .collect();

        assert_eq!(expected.len(), parsed.len());
        for (expected_key, expected_value) in expected {
            println!("Checking {}", expected_key);
            let actual_value = &parsed[expected_key];
            assert_eq!(*actual_value, expected_value);
        }
    }

    #[test]
    fn list_map_values_are_parsed_correctly() {
        let hcl = fixtures::LIST;
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
            (
                "object_in_list",
                Value::new_list(vec![
                    Value::new_map(vec![vec![(Key::new_identifier("test"), Value::from(123))]]),
                    Value::new_map(vec![vec![(Key::new_identifier("foo"), Value::from("bar"))]]),
                    Value::new_map(vec![vec![(Key::new_identifier("baz"), Value::from(false))]]),
                ]),
            ),
        ]
        .into_iter()
        .collect();

        assert_eq!(expected.len(), parsed.len());
        for (expected_key, expected_value) in expected {
            println!("Checking {}", expected_key);
            let actual_value = &parsed[expected_key];
            assert_eq!(*actual_value, expected_value);
        }
    }

    #[test]
    fn multiple_maps_are_parsed_correctly() {
        let hcl = fixtures::BLOCK;
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();
        println!("{:#?}", parsed);
        assert!(parsed.is_unmerged());

        assert_eq!(parsed.len(), 5); // unmerged values
        assert_eq!(parsed.len_scalar(), 19);

        // simple_map
        let simple_map = parsed.get("simple_map").unwrap().unwrap_many();
        assert_eq!(simple_map.len(), 2);

        let expected_simple_maps = vec![
            vec![MapValues::new_unmerged(vec![
                (Key::new_identifier("foo"), Value::from("bar")),
                (Key::new_identifier("bar"), Value::from("baz")),
                (Key::new_identifier("index"), Value::from(1)),
            ])],
            vec![MapValues::new_unmerged(vec![
                (Key::new_identifier("foo"), Value::from("bar")),
                (Key::new_identifier("bar"), Value::from("baz")),
                (Key::new_identifier("index"), Value::from(0)),
            ])],
        ];
        let actual_simple_map: Vec<_> = simple_map
            .into_iter()
            .map(|v| v.borrow_map().expect("to be a map"))
            .collect();
        assert_list_eq!(expected_simple_maps, actual_simple_map);

        // resource
        let resources = parsed.get("resource").unwrap().unwrap_many();
        assert_eq!(resources.len(), 3);
        let resources: Vec<_> = resources
            .into_iter()
            .map(|v| v.borrow_block().expect("to be a block"))
            .collect();

        let expected_resources = vec![
            Block::new_unmerged(vec![(
                vec!["security/group", "foobar"],
                MapValues::new_unmerged(vec![
                    (Key::new_identifier("name"), Value::from("foobar")),
                    (
                        Key::new_identifier("allow"),
                        Value::Object(vec![MapValues::new_unmerged(vec![
                            (Key::new_identifier("name"), Value::from("localhost")),
                            (
                                Key::new_identifier("cidrs"),
                                vec![Value::from("127.0.0.1/32")].into_iter().collect(),
                            ),
                        ])]),
                    ),
                    (
                        Key::new_identifier("allow"),
                        Value::Object(vec![MapValues::new_unmerged(vec![
                            (Key::new_identifier("name"), Value::from("lan")),
                            (
                                Key::new_identifier("cidrs"),
                                vec![Value::from("192.168.0.0/16")].into_iter().collect(),
                            ),
                        ])]),
                    ),
                    (
                        Key::new_identifier("deny"),
                        Value::Object(vec![MapValues::new_unmerged(vec![
                            (Key::new_identifier("name"), Value::from("internet")),
                            (
                                Key::new_identifier("cidrs"),
                                vec![Value::from("0.0.0.0/0")].into_iter().collect(),
                            ),
                        ])]),
                    ),
                ]),
            )]),
            Block::new_unmerged(vec![(
                vec!["security/group", "second"],
                MapValues::new_unmerged(vec![
                    (Key::new_identifier("name"), Value::from("second")),
                    (
                        Key::new_identifier("allow"),
                        Value::Object(vec![MapValues::new_unmerged(vec![
                            (Key::new_identifier("name"), Value::from("all")),
                            (
                                Key::new_identifier("cidrs"),
                                vec![Value::from("0.0.0.0/0")].into_iter().collect(),
                            ),
                        ])]),
                    ),
                ]),
            )]),
            Block::new_unmerged(vec![(
                vec!["instance", "an_instance"],
                MapValues::new_unmerged(vec![
                    (Key::new_identifier("name"), Value::from("an_instance")),
                    (Key::new_identifier("image"), Value::from("ubuntu:18.04")),
                    (
                        Key::new_identifier("user"),
                        Value::Block(Block::new_unmerged(vec![(
                            vec!["test"],
                            MapValues::new_unmerged(vec![(
                                Key::new_identifier("root"),
                                Value::from(true),
                            )]),
                        )])),
                    ),
                ]),
            )]),
        ];
        assert_list_eq(expected_resources, resources);
    }

    // TODO: Tests for merging

    #[test]
    fn maps_are_merged_correctly() {
        let hcl = fixtures::BLOCK;
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();
        assert!(parsed.is_unmerged());

        let parsed = parsed.merge().unwrap();
        println!("{:#?}", parsed);
        assert!(parsed.is_merged());

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed.len_scalar(), 19);

        // simple_map
        let simple_map = parsed.get("simple_map").unwrap().unwrap_one();
        assert_eq!(simple_map.len(), 2);

        let expected_simple_maps = vec![
            MapValues::new_merged(vec![
                (Key::new_identifier("foo"), Value::from("bar")),
                (Key::new_identifier("bar"), Value::from("baz")),
                (Key::new_identifier("index"), Value::from(1)),
            ])
            .unwrap(),
            MapValues::new_merged(vec![
                (Key::new_identifier("foo"), Value::from("bar")),
                (Key::new_identifier("bar"), Value::from("baz")),
                (Key::new_identifier("index"), Value::from(0)),
            ])
            .unwrap(),
        ];
        let simple_maps = simple_map.unwrap_borrow_map();
        println!("{:#?}", simple_maps);
        assert!(simple_maps.iter().eq(&expected_simple_maps));

        // resource
        let resource = parsed.get("resource").unwrap().unwrap_one();
        assert_eq!(resource.len(), 3);
        let resource = resource.unwrap_borrow_block();

        let expected_resources = Block::new_merged(vec![
            (
                vec!["security/group", "foobar"],
                MapValues::new_merged(vec![
                    (Key::new_identifier("name"), Value::from("foobar")),
                    (
                        Key::new_identifier("allow"),
                        Value::Object(vec![MapValues::new_merged(vec![
                            (Key::new_identifier("name"), Value::from("localhost")),
                            (
                                Key::new_identifier("cidrs"),
                                vec![Value::from("127.0.0.1/32")].into_iter().collect(),
                            ),
                        ])
                        .unwrap()]),
                    ),
                    (
                        Key::new_identifier("allow"),
                        Value::Object(vec![MapValues::new_merged(vec![
                            (Key::new_identifier("name"), Value::from("lan")),
                            (
                                Key::new_identifier("cidrs"),
                                vec![Value::from("192.168.0.0/16")].into_iter().collect(),
                            ),
                        ])
                        .unwrap()]),
                    ),
                    (
                        Key::new_identifier("deny"),
                        Value::Object(vec![MapValues::new_merged(vec![
                            (Key::new_identifier("name"), Value::from("internet")),
                            (
                                Key::new_identifier("cidrs"),
                                vec![Value::from("0.0.0.0/0")].into_iter().collect(),
                            ),
                        ])
                        .unwrap()]),
                    ),
                ])
                .unwrap(),
            ),
            (
                vec!["security/group", "second"],
                MapValues::new_merged(vec![
                    (Key::new_identifier("name"), Value::from("second")),
                    (
                        Key::new_identifier("allow"),
                        Value::Object(vec![MapValues::new_merged(vec![
                            (Key::new_identifier("name"), Value::from("all")),
                            (
                                Key::new_identifier("cidrs"),
                                vec![Value::from("0.0.0.0/0")].into_iter().collect(),
                            ),
                        ])
                        .unwrap()]),
                    ),
                ])
                .unwrap(),
            ),
            (
                vec!["instance", "an_instance"],
                MapValues::new_merged(vec![
                    (Key::new_identifier("name"), Value::from("an_instance")),
                    (Key::new_identifier("image"), Value::from("ubuntu:18.04")),
                    (
                        Key::new_identifier("user"),
                        Value::Block(
                            Block::new_merged(vec![(
                                vec!["test"],
                                MapValues::new_merged(vec![(
                                    Key::new_identifier("root"),
                                    Value::from(true),
                                )])
                                .unwrap(),
                            )])
                            .unwrap(),
                        ),
                    ),
                ])
                .unwrap(),
            ),
        ])
        .unwrap();
        assert_eq!(&expected_resources, resource);
    }
}
