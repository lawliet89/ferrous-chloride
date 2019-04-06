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
use std::borrow::{Borrow, Cow};

use nom::types::CompleteStr;
use nom::{alt, call, char, do_parse, named, peek, recognize, tag, terminated, IResult};

use super::expression::{expression, Expression};
use crate::parser::literals::{identifier, newline};
use crate::HashMap;

// TODO: Dealing with expressions and ambiguity. See reference
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ObjectElementIdentifier<'a> {
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

impl<'a, S> PartialEq<S> for ObjectElementIdentifier<'a>
where
    S: AsRef<str>,
{
    fn eq(&self, other: &S) -> bool {
        match self {
            ObjectElementIdentifier::Identifier(ident) => ident.eq(other.as_ref()),
            ObjectElementIdentifier::Expression(_) => false,
        }
    }
}

impl<'a> Borrow<str> for ObjectElementIdentifier<'a> {
    fn borrow(&self) -> &str {
        match self {
            ObjectElementIdentifier::Identifier(ref ident) => ident,
            ObjectElementIdentifier::Expression(ref expr) => expr,
        }
    }
}

impl<'a> From<&'a str> for ObjectElementIdentifier<'a> {
    fn from(s: &'a str) -> Self {
        ObjectElementIdentifier::Identifier(Cow::Borrowed(s))
    }
}

pub type ObjectElement<'a> = (ObjectElementIdentifier<'a>, Expression<'a>);

pub type Object<'a> = HashMap<ObjectElementIdentifier<'a>, Expression<'a>>;

// Cannot use `named!` because the compiler cannot determine the lifetime
pub fn object_element_identifier<'a>(
    input: CompleteStr<'a>,
) -> IResult<CompleteStr<'a>, ObjectElementIdentifier<'a>, u32> {
    alt!(
        input,
        call!(identifier) =>
            { |ident| ObjectElementIdentifier::Identifier(Cow::Borrowed(ident)) }
        | recognize!(call!(expression)) =>
            { |expr: CompleteStr<'a>| ObjectElementIdentifier::Expression(Cow::Borrowed(expr.0)) }
    )
}

named!(
    pub object_element(CompleteStr) -> ObjectElement,
    inline_whitespace!(
        do_parse!(
            identifier: call!(object_element_identifier)
            >> char!('=')
            >> expression: call!(expression)
            >> (identifier, expression)
        )
    )
);

named!(
    pub object_begin(CompleteStr) -> char,
    char!('{')
);

named!(
    pub object_end(CompleteStr) -> char,
    char!('}')
);

named!(
    pub object_separator(CompleteStr) -> CompleteStr,
    alt!(
        tag!(",")
        | call!(newline) => {|_| CompleteStr("") }
        | peek!(object_end) => {|_| CompleteStr("") }
    )
);

named!(
    pub object_body(CompleteStr) -> HashMap<ObjectElementIdentifier, Expression>,
    do_parse!(
        values: whitespace!(
            many0!(
                terminated!(
                    call!(object_element),
                    call!(object_separator)
                )
            )
        )
        >> (values.into_iter().collect())
    )
);

named!(
    pub object(CompleteStr) -> Object,
    do_parse!(
        whitespace!(call!(object_begin))
        >> values: whitespace!(call!(object_body))
        >> call!(object_end)
        >> (values)
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::fixtures;
    use crate::utils::ResultUtilsString;

    #[test]
    fn object_element_identifiers_are_parsed_correctly() {
        let test_cases = [
            (
                "foobar",
                ObjectElementIdentifier::Identifier(Cow::Borrowed("foobar")),
            ),
            (
                "true",
                ObjectElementIdentifier::Identifier(Cow::Borrowed("true")),
            ),
            (
                "(true)",
                ObjectElementIdentifier::Expression(Cow::Borrowed("(true)")),
            ),
            (
                "(1234)",
                ObjectElementIdentifier::Expression(Cow::Borrowed("(1234)")),
            ),
        ];

        for (input, expected_output) in &test_cases {
            let output = object_element_identifier(CompleteStr(input)).unwrap_output();
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
                "test /* test */ = 123",
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

    #[test]
    fn empty_object_body_is_parsed_correctly() {
        let hcl = "";
        let parsed = object_body(CompleteStr(hcl)).unwrap_output();

        assert_eq!(0, parsed.len());
    }

    #[test]
    fn empty_object_is_parsed_correctly() {
        let hcl = "{}";
        let parsed = object(CompleteStr(hcl)).unwrap_output();

        assert_eq!(0, parsed.len());
    }

    #[test]
    fn non_terminating_new_lines_object_bodies_are_parsed_correctly() {
        let hcl = "test = true,";
        let parsed = object_body(CompleteStr(hcl)).unwrap_output();

        assert_eq!(1, parsed.len());
        assert_eq!(parsed["test"], Expression::from(true));
    }

    #[test]
    fn single_line_object_is_parsed_correctly() {
        let hcl = "{ test = true }";
        let parsed = object(CompleteStr(hcl)).unwrap_output();

        assert_eq!(1, parsed.len());
        assert_eq!(parsed["test"], Expression::from(true));
    }

    #[test]
    fn single_object_body_are_parsed_correctly() {
        let hcl = "foo = \"bar\"\n";
        let parsed = object_body(CompleteStr(hcl)).unwrap_output();

        assert_eq!(1, parsed.len());
        assert_eq!(parsed["foo"], Expression::from("bar"));
    }

    #[test]
    fn single_object_is_parsed_correctly() {
        let hcl = "{\nfoo = \"bar\"\n}";
        let parsed = object(CompleteStr(hcl)).unwrap_output();

        assert_eq!(1, parsed.len());
        assert_eq!(parsed["foo"], Expression::from("bar"));
    }

    #[test]
    fn multiple_elements_in_body_are_parsed_correctly() {
        let hcl = r#"foo = "bar"
bar = "baz"
true = false
"#;
        let parsed = object_body(CompleteStr(hcl)).unwrap_output();

        assert_eq!(3, parsed.len());
        assert_eq!(parsed["foo"], Expression::from("bar"));
        assert_eq!(parsed["bar"], Expression::from("baz"));
        assert_eq!(parsed["true"], Expression::from(false));
    }

    #[test]
    fn multiple_elements_in_object_is_parsed_correctly() {
        let hcl = r#"{
foo = "bar"
bar = "baz"
true = false}"#;
        let parsed = object(CompleteStr(hcl)).unwrap_output();

        assert_eq!(3, parsed.len());
        assert_eq!(parsed["foo"], Expression::from("bar"));
        assert_eq!(parsed["bar"], Expression::from("baz"));
        assert_eq!(parsed["true"], Expression::from(false));
    }

    #[test]
    fn multiple_elements_in_body_with_trailing_comma_are_parsed_correctly() {
        let hcl = r#"foo = "bar"
bar = "baz"
true = false,"#;
        let parsed = object_body(CompleteStr(hcl)).unwrap_output();

        assert_eq!(3, parsed.len());
        assert_eq!(parsed["foo"], Expression::from("bar"));
        assert_eq!(parsed["bar"], Expression::from("baz"));
        assert_eq!(parsed["true"], Expression::from(false));
    }

    #[test]
    fn multiple_elements_in_object_with_trailing_comma_is_parsed_correctly() {
        let hcl = r#"{
foo = "bar"
bar = "baz"
true = false,}"#;
        let parsed = object(CompleteStr(hcl)).unwrap_output();

        assert_eq!(3, parsed.len());
        assert_eq!(parsed["foo"], Expression::from("bar"));
        assert_eq!(parsed["bar"], Expression::from("baz"));
        assert_eq!(parsed["true"], Expression::from(false));
    }

    #[test]
    fn multiple_elements_in_body_with_trailing_newline_are_parsed_correctly() {
        let hcl = r#"foo = "bar"
bar = "baz"
true = false
# Hi
"#;
        let parsed = object_body(CompleteStr(hcl)).unwrap_output();

        assert_eq!(3, parsed.len());
        assert_eq!(parsed["foo"], Expression::from("bar"));
        assert_eq!(parsed["bar"], Expression::from("baz"));
        assert_eq!(parsed["true"], Expression::from(false));
    }

    #[test]
    fn multiple_elements_in_object_with_trailing_newline_is_parsed_correctly() {
        let hcl = r#"{
foo = "bar"
bar = "baz"
true = false
# Hi
}"#;
        let parsed = object(CompleteStr(hcl)).unwrap_output();

        assert_eq!(3, parsed.len());
        assert_eq!(parsed["foo"], Expression::from("bar"));
        assert_eq!(parsed["bar"], Expression::from("baz"));
        assert_eq!(parsed["true"], Expression::from(false));
    }

    #[test]
    fn scalar_object_body_are_parsed_correctly() {
        let hcl = r#"test_unsigned_int = 123
test_signed_int /*inline comment */ = -123 # Another comment
test_float = -1.23

bool_true = true
bool_false = false

string = "Hello World!"
comma_separated = "I'm special!",

long_string = <<EOF
hihi
another line!
EOF
string_escaped = "\" Hello World!"
"#;
        let parsed = object_body(CompleteStr(hcl)).unwrap_output();

        let expected: HashMap<_, _> = vec![
            ("test_unsigned_int", Expression::from(123)),
            ("test_signed_int", Expression::from(-123)),
            ("test_float", Expression::from(-1.23)),
            ("bool_true", Expression::from(true)),
            ("bool_false", Expression::from(false)),
            ("string", Expression::from("Hello World!")),
            ("comma_separated", Expression::from("I'm special!")),
            ("long_string", Expression::from("hihi\nanother line!")),
            ("string_escaped", Expression::from("\" Hello World!")),
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
    fn scalar_object_is_parsed_correctly() {
        let hcl = r#"{
    test_unsigned_int = 123
    test_signed_int /*inline comment */ = -123 # Another comment
    test_float = -1.23

    bool_true = true
    bool_false = false

    string = "Hello World!"
    comma_separated = "I'm special!",

    long_string = <<EOF
hihi
another line!
EOF
    string_escaped = "\" Hello World!"
}
"#;
        let (remaining, parsed) = object(CompleteStr(hcl)).unwrap();
        assert_eq!("\n", remaining.0);

        let expected: HashMap<_, _> = vec![
            ("test_unsigned_int", Expression::from(123)),
            ("test_signed_int", Expression::from(-123)),
            ("test_float", Expression::from(-1.23)),
            ("bool_true", Expression::from(true)),
            ("bool_false", Expression::from(false)),
            ("string", Expression::from("Hello World!")),
            ("comma_separated", Expression::from("I'm special!")),
            ("long_string", Expression::from("hihi\nanother line!")),
            ("string_escaped", Expression::from("\" Hello World!")),
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
    fn list_object_body_are_parsed_correctly() {
        let hcl = fixtures::LIST;
        let parsed = object_body(CompleteStr(hcl)).unwrap_output();

        let expected: HashMap<_, _> = vec![
            (
                "list",
                Expression::new_tuple(vec![
                    Expression::from(true),
                    Expression::from(false),
                    Expression::from(123),
                    Expression::from(-123.456),
                    Expression::from("foobar"),
                ]),
            ),
            (
                "list_multi",
                Expression::new_tuple(vec![
                    Expression::from(true),
                    Expression::from(false),
                    Expression::from(123),
                    Expression::from(-123.456),
                    Expression::from("foobar"),
                ]),
            ),
            (
                "list_in_list",
                Expression::new_tuple(vec![
                    Expression::new_tuple(vec![
                        Expression::from("test"),
                        Expression::from("foobar"),
                    ]),
                    Expression::from(1),
                    Expression::from(2),
                    Expression::from(-3),
                ]),
            ),
            (
                "object_in_list",
                Expression::new_tuple(vec![
                    Expression::new_object(vec![("test", Expression::from(123))]),
                    Expression::new_object(vec![("foo", Expression::from("bar"))]),
                    Expression::new_object(vec![("baz", Expression::from(false))]),
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
    fn list_object_is_parsed_correctly() {
        let hcl = ["{", fixtures::LIST, "}"].join("");
        let parsed = object(CompleteStr(&hcl)).unwrap_output();

        let expected: HashMap<_, _> = vec![
            (
                "list",
                Expression::new_tuple(vec![
                    Expression::from(true),
                    Expression::from(false),
                    Expression::from(123),
                    Expression::from(-123.456),
                    Expression::from("foobar"),
                ]),
            ),
            (
                "list_multi",
                Expression::new_tuple(vec![
                    Expression::from(true),
                    Expression::from(false),
                    Expression::from(123),
                    Expression::from(-123.456),
                    Expression::from("foobar"),
                ]),
            ),
            (
                "list_in_list",
                Expression::new_tuple(vec![
                    Expression::new_tuple(vec![
                        Expression::from("test"),
                        Expression::from("foobar"),
                    ]),
                    Expression::from(1),
                    Expression::from(2),
                    Expression::from(-3),
                ]),
            ),
            (
                "object_in_list",
                Expression::new_tuple(vec![
                    Expression::new_object(vec![("test", Expression::from(123))]),
                    Expression::new_object(vec![("foo", Expression::from("bar"))]),
                    Expression::new_object(vec![("baz", Expression::from(false))]),
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
    fn nested_object_is_parsed_correctly() {
        let hcl = r#"{
    test_unsigned_int = 123
    true = false

    nested = {
        false = true
        oh_no = "reality is broken!"
    },
},
"#;
        let (remaining, parsed) = object(CompleteStr(hcl)).unwrap();
        assert_eq!(",\n", remaining.0);

        let expected: HashMap<ObjectElementIdentifier, _> = vec![
            (From::from("test_unsigned_int"), Expression::from(123)),
            (From::from("true"), Expression::from(false)),
            (
                From::from("nested"),
                Expression::new_object(vec![
                    ("false", Expression::from(true)),
                    ("oh_no", Expression::from("reality is broken!")),
                ]),
            ),
        ]
        .into_iter()
        .collect();

        assert_eq!(expected, parsed);
    }
}
