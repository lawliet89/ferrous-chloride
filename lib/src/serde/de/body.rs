use std::borrow::Cow;
use std::collections::HashSet;
use std::vec;

use serde::de::{self, DeserializeSeed, IntoDeserializer, Visitor};
use serde::forward_to_deserialize_any;

use crate::parser;
use crate::parser::attribute::Attribute;
use crate::parser::body::{Body, BodyElement};
use crate::parser::expression::Expression;
use crate::serde::de::{Compat, Error};

#[derive(Clone, Debug)]
pub enum BodyValue<'de> {
    Expression(Expression<'de>),
}

#[derive(Clone, Debug)]
pub struct Deserializer<'de> {
    body: Body<'de>,
}

impl<'de> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Compat;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!("Not yet!")
    }

    // These types are not possible to deserialize from a Body
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[derive(Clone, Debug)]
pub struct MapAccess<'de> {
    // attributes: vec::IntoIter<Attribute<'de>>,
    elements: vec::IntoIter<BodyElement<'de>>,
    /// MapAccess users have to call `next_key_seed` before `next_value_seed`
    /// So we store the value extracted after calling `next_key_seed`
    value: Option<BodyValue<'de>>,
    /// Set of keys we have seen before
    seen_attributes: HashSet<Cow<'de, str>>,
}

impl<'de> MapAccess<'de> {
    pub fn new(body: Body<'de>) -> Self {
        Self {
            elements: body.into_iter(),
            value: None,
            seen_attributes: Default::default(),
        }
    }
}

pub fn build_map_access<'de>(body: Body<'de>) {
    let (attributes, blocks): (Vec<_>, Vec<_>) =
        body.into_iter().partition(BodyElement::is_attribute);
}

impl<'de> de::MapAccess<'de> for MapAccess<'de> {
    type Error = Compat;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let next = self.elements.next();
        if let None = next {
            return Ok(None);
        }

        match next.expect("to be some") {
            BodyElement::Attribute((ident, expr)) => {
                if !self.seen_attributes.insert(ident.clone()) {
                    Err(Error::BodyDuplicateKey(ident.to_string()))?;
                }
                self.value = Some(BodyValue::Expression(expr));
                seed.deserialize(ident.into_deserializer()).map(Some)
            }
            BodyElement::Block(block) => unimplemented!("Not yet!"),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.value.take().expect("to be some") {
            BodyValue::Expression(expr) => seed.deserialize(expr),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        // Vector size hint always returns a value
        let (lower, _) = self.elements.size_hint();
        Some(lower)
    }
}
