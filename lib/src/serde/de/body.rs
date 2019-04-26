use std::borrow::Cow;
use std::collections::HashSet;
use std::vec;

use serde::de::{self, DeserializeSeed, IntoDeserializer, Visitor};
use serde::forward_to_deserialize_any;
use serde::Deserialize;

use crate::parser::block;
use crate::parser::body::{Body, BodyElement};
use crate::parser::expression::Expression;
use crate::parser::identifier::Identifier;
use crate::serde::de::{Compat, Error};

#[derive(Clone, Debug)]
pub enum BodyValue<'de> {
    Expression(Expression<'de>),
    Block(block::BlockBody<'de>),
}

/// Deserializer for a HCL configuration file.
///
/// In HCL, a [`ConfigFile`](crate::parser::ConfigFile) is essentially a
/// [`Body`] which is a sequence of
/// [`Attribute`s](crate::parser::Attribute) or [`Block`s](block::Block).
///
/// The semantics of HCL processing requires that the entire file be parsed at once before we can
/// process the structural elements such as blocks. This deserializer thus expects to be passed
/// a full HCL configuration file.
///
/// To deserialize HCL expressions into types like `&str`,
/// use [`Expression::parse`]
/// to parse the HCL expression, and then use the parsed [`Expression`] to deserialize.
#[derive(Clone, Debug)]
pub struct Deserializer<'de> {
    body: Body<'de>,
}

impl<'de> Deserializer<'de> {
    pub fn new(body: Body<'de>) -> Self {
        Self { body }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &'de str) -> Result<Self, Error> {
        let body = crate::parser::parse_str(s)?;
        Ok(Self::new(body))
    }
}

impl<'de> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Compat;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Best guess is to treat the visitor as a map
        visitor.visit_map(MapAccess::new(self.body))
    }

    // Option
    // unit
    // unit struct
    // newtype struct
    // seq
    // tuple
    // tuple struct
    // struct
    // enum

    // These types are not possible to deserialize from a Body
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[derive(Clone, Debug)]
pub struct MapAccess<'de> {
    elements: vec::IntoIter<(Identifier<'de>, BodyValue<'de>)>,
    /// MapAccess users have to call `next_key_seed` before `next_value_seed`
    /// So we store the value extracted after calling `next_key_seed`
    value: Option<BodyValue<'de>>,
    /// Set of keys we have seen before
    seen: HashSet<Cow<'de, str>>,
}

impl<'de> MapAccess<'de> {
    pub fn new(body: Body<'de>) -> Self {
        Self {
            elements: build_map_acces_iter(body),
            value: None,
            seen: Default::default(),
        }
    }
}

pub(crate) fn build_map_acces_iter<'de>(
    body: Body<'de>,
) -> vec::IntoIter<(Identifier<'de>, BodyValue<'de>)> {
    let (attributes, blocks): (Vec<_>, Vec<_>) =
        body.into_iter().partition(BodyElement::is_attribute);
    let attributes = attributes
        .into_iter()
        .map(BodyElement::unwrap_attribute)
        .map(|(ident, expr)| (ident, BodyValue::Expression(expr)));
    let blocks = block::Blocks::new(blocks.into_iter().map(BodyElement::unwrap_block))
        .into_iter()
        .map(|(ident, bodies)| (ident, BodyValue::Block(bodies)));

    let elements: Vec<_> = attributes.chain(blocks).collect();
    elements.into_iter()
}

impl<'de> de::MapAccess<'de> for MapAccess<'de> {
    type Error = Compat;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let next = self.elements.next();
        if next.is_none() {
            return Ok(None);
        }

        match next.expect("to be some") {
            (ident, expr @ BodyValue::Expression(_)) => {
                if !self.seen.insert(ident.clone()) {
                    Err(Error::BodyDuplicateKey(ident.to_string()))?;
                }
                self.value = Some(expr);
                seed.deserialize(ident.into_deserializer()).map(Some)
            }
            (block_type, blk @ BodyValue::Block(_)) => {
                // If this key has been seen before, we have a bug
                assert!(
                    self.seen.insert(block_type.clone()),
                    "bug in block merging code"
                );
                self.value = Some(blk);
                seed.deserialize(block_type.into_deserializer()).map(Some)
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.value.take().expect("to be some") {
            BodyValue::Expression(expr) => seed.deserialize(expr),
            BodyValue::Block(blk) => seed.deserialize(blk),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        // Vector size hint always returns a value
        let (lower, _) = self.elements.size_hint();
        Some(lower)
    }
}

/// Deserialize a type `T` from a provided HCL String
///
/// ```rust
/// # use ferrous_chloride::serde::de::body::from_str;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, PartialEq, Debug)]
/// struct DeserializeMe {
///     name: String,
///     allow: bool,
///     index: usize,
///     list: Vec<String>,
///     nothing: Option<f64>,
/// }
///
/// let input = r#"
/// name = "second"
/// allow = false
/// index = 1
/// list = ["foo", "bar", "baz"]"#;
///
/// let deserialized: DeserializeMe = from_str(input).unwrap();
/// ```
pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let deserializer = Deserializer::from_str(s)?;
    Ok(T::deserialize(deserializer)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_simple_structs() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct DeserializeMe {
            name: String,
            allow: bool,
            index: usize,
            list: Vec<String>,
            nothing: Option<f64>,
        }

        let input = r#"
name = "second"
allow = false
index = 1
list = ["foo", "bar", "baz"]
"#;
        let deserialized = from_str(input).unwrap();

        let expected = DeserializeMe {
            name: "second".to_string(),
            allow: false,
            index: 1,
            list: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
            nothing: None,
        };

        assert_eq!(expected, deserialized);
    }

    //     #[test]
    //     fn deserialize_nested_structs() {
    //         #[derive(Deserialize, PartialEq, Debug)]
    //         struct SecurityGroup {
    //             name: String,
    //             allow: Allow,
    //         }

    //         #[derive(Deserialize, PartialEq, Debug)]
    //         struct Allow {
    //             name: String,
    //             cidrs: Vec<String>,
    //         }

    //         let input = r#"
    //   name = "second"

    //   allow {
    //     name = "all"
    //     cidrs = ["0.0.0.0/0"]
    //   }
    // "#;
    //         let mut deserializer = Deserializer::from_str(input);
    //         let deserialized: SecurityGroup = Deserialize::deserialize(&mut deserializer).unwrap();
    //     }
}
