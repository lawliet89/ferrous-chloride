use std::borrow::Cow;
use std::collections::HashSet;

use serde::de::{DeserializeSeed, IntoDeserializer, MapAccess};

use crate::parser::expression::Expression;
use crate::parser::object::{Object, ObjectElementIdentifier};
use crate::serde::de::{Compat, Error};

#[derive(Debug)]
pub struct ObjectMapAccess<'de> {
    iterator: std::vec::IntoIter<(ObjectElementIdentifier<'de>, Expression<'de>)>,
    /// MapAccess users have to call `next_key_seed` before `next_value_seed`
    /// So we store the value extracted after calling `next_key_seed`
    expression: Option<Expression<'de>>,
    /// Set of keys we have seen before
    seen_keys: HashSet<Cow<'de, str>>,
}

impl<'de> ObjectMapAccess<'de> {
    pub fn new(object: Object<'de>) -> Self {
        Self {
            iterator: object.into_iter(),
            expression: Default::default(),
            seen_keys: Default::default(),
        }
    }
}

impl<'de> MapAccess<'de> for ObjectMapAccess<'de> {
    type Error = Compat;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let next = self.iterator.next();

        let (key, value) = match next {
            None => return Ok(None),
            Some((key, value)) => (key, value),
        };
        let key = key.as_str();

        if !self.seen_keys.insert(key.clone()) {
            Err(Error::ObjectDuplicateKey(key.to_string()))?;
        }

        self.expression = Some(value);
        seed.deserialize(key.into_deserializer()).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let expression = self.expression.take().expect("to be some");
        seed.deserialize(expression)
    }

    fn size_hint(&self) -> Option<usize> {
        // Vector size hint always returns a value
        let (lower, _) = self.iterator.size_hint();
        Some(lower)
    }
}
