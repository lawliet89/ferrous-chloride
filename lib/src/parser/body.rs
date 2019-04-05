//! HCL Body
//!
//! [Reference](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#structural-elements)
use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::FromIterator;

use nom::types::CompleteStr;
use nom::{alt, call, do_parse, eof, named, tag, terminated};

use super::attribute::attribute;
use super::expression::{Expression, ExpressionType};
use super::literals::newline;
use crate::constants::*;
use crate::{Error, KeyValuePairs, Value};

/// A HCL document body
///
/// ```ebnf
/// ConfigFile   = Body;
/// Body         = (Attribute | Block | OneLineBlock)*;
/// Attribute    = Identifier "=" Expression Newline;
/// Block        = Identifier (StringLit|Identifier)* "{" Newline Body "}" Newline;
/// OneLineBlock = Identifier (StringLit|Identifier)* "{" (Identifier "=" Expression)? "}" Newline;
/// ```
pub type Body<'a> = KeyValuePairs<Cow<'a, str>, Expression<'a>>;

impl<'a> Body<'a> {
    // TODO: Customise merging behaviour wrt duplicate keys
    pub fn new_merged<T>(iter: T) -> Result<Self, Error>
    where
        T: IntoIterator<Item = (Cow<'a, str>, Expression<'a>)>,
    {
        use std::collections::hash_map::Entry;

        let mut map = HashMap::new();
        for (key, value) in iter {
            let mut value = value.merge()?;
            match map.entry(key) {
                Entry::Vacant(vacant) => {
                    vacant.insert(value);
                }
                Entry::Occupied(mut occupied) => {
                    let key = occupied.key().to_string();
                    let Expression {
                        ref mut expression, tokens: ref mut _tokens
                    } = occupied.get_mut();
                    match expression {
                        illegal @ ExpressionType::Null
                        | illegal @ ExpressionType::Number(_)
                        | illegal @ ExpressionType::Boolean(_)
                        | illegal @ ExpressionType::String(_)
                        | illegal @ ExpressionType::Tuple(_) => Err(Error::IllegalMultipleEntries {
                            key,
                            variant: illegal.variant_name(),
                        })?,
                        // Value::Object(ref mut map) => {
                        //     // Check that the incoming value is also a Object
                        //     if let Value::Object(ref mut incoming) = value {
                        //         map.append(incoming);
                        //     } else {
                        //         Err(Error::ErrorMergingKeys {
                        //             key,
                        //             existing_variant: OBJECT,
                        //             incoming_variant: value.variant_name(),
                        //         })?;
                        //     }
                        // }
                        // Value::Block(ref mut block) => {
                        //     let value = value;
                        //     // Check that the incoming value is also a Block
                        //     if let Value::Block(incoming) = value {
                        //         block.extend(incoming);
                        //     } else {
                        //         Err(Error::ErrorMergingKeys {
                        //             key,
                        //             existing_variant: BLOCK,
                        //             incoming_variant: value.variant_name(),
                        //         })?;
                        //     }
                        // }
                    };
                }
            };
        }
        Ok(KeyValuePairs::Merged(map))
    }

    pub fn new_unmerged<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Cow<'a, str>, Expression<'a>)>,
    {
        KeyValuePairs::Unmerged(iter.into_iter().collect())
    }

    pub fn merge(self) -> Result<Self, Error> {
        if let KeyValuePairs::Unmerged(vec) = self {
            Self::new_merged(vec.into_iter())
        } else {
            Ok(self)
        }
    }

    pub fn as_merged(&self) -> Result<Self, Error> {
        if let KeyValuePairs::Unmerged(vec) = self {
            Self::new_merged(vec.iter().cloned())
        } else {
            Ok(self.clone())
        }
    }

    pub fn unmerge(self) -> Self {
        if let KeyValuePairs::Merged(hashmap) = self {
            Self::new_unmerged(hashmap.into_iter())
        } else {
            self
        }
    }

    pub fn as_unmerged(&self) -> Self {
        if let KeyValuePairs::Merged(hashmap) = self {
            Self::new_unmerged(
                hashmap
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone())),
            )
        } else {
            self.clone()
        }
    }
}

impl<'a> FromIterator<(Cow<'a, str>, Expression<'a>)> for Body<'a> {
    fn from_iter<T: IntoIterator<Item = (Cow<'a, str>, Expression<'a>)>>(iter: T) -> Self {
        Self::new_unmerged(iter)
    }
}

named!(
    pub body(CompleteStr) -> Body,
    do_parse!(
        values: whitespace!(
            many0!(
                terminated!(
                    call!(attribute),
                    alt!(
                        call!(newline) => { |_| CompleteStr("") }
                        | eof!()
                    )
                )
            )
        )
        >> (values.into_iter().collect())
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use crate::fixtures;
    use crate::parser::literals::Key;
    use crate::utils::{assert_list_eq, ResultUtilsString};
    use crate::value::Block;
    use crate::value::MapValues;
    use crate::{Mergeable, ScalarLength};

    #[test]
    fn empty_body_is_parsed_correctly() {
        let hcl = "";
        let parsed = body(CompleteStr(hcl)).unwrap_output();

        assert_eq!(0, parsed.len());
    }

    #[test]
    fn non_terminating_new_lines_are_parsed_correctly() {
        let hcl = fixtures::NO_NEWLINE_EOF;
        let parsed = body(CompleteStr(hcl)).unwrap_output();

        assert_eq!(1, parsed.len());
        assert_eq!(parsed["test"], Value::from(true));
    }

    #[test]
    fn single_attribute_body_is_parsed_correctly() {
        let hcl = fixtures::SINGLE;
        let parsed = body(CompleteStr(hcl)).unwrap_output();

        assert_eq!(1, parsed.len());
        assert_eq!(parsed["foo"], Value::from("bar"));
    }

    #[test]
    fn scalar_attributes_are_parsed_correctly() {
        let hcl = fixtures::SCALAR;
        let parsed = body(CompleteStr(hcl)).unwrap_output();

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
    #[ignore]
    fn list_in_body_are_parsed_correctly() {
        let hcl = fixtures::LIST;
        let parsed = body(CompleteStr(hcl)).unwrap_output();

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
                "map_in_list",
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

    // TODO: Test merging
}
