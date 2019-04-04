use std::collections::hash_map::IntoIter;

use serde::de::{self, DeserializeSeed};

use super::{Compat, Error};
use crate::parser::literals::Key;
use crate::value::{MapValues, Value};

#[derive(Debug)]
pub struct MapAccess<'a> {
    iterator: IntoIter<Key<'a>, Value<'a>>,
    // MapAccess users have to call `next_key_seed` before `next_value_seed`
    // So we store the value extracted after calling `next_key_seed`
    value: Option<Value<'a>>,
}

impl<'a> MapAccess<'a> {
    pub(crate) fn new(map: MapValues<'a>) -> Result<Self, Error> {
        Ok(Self {
            iterator: map.merge()?.unwrap_merged().into_iter(),
            value: None,
        })
    }
}

impl<'de, 'a> de::MapAccess<'de> for MapAccess<'a> {
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

        self.value = Some(value);
        seed.deserialize(key).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self.value.take().expect("to be some");
        seed.deserialize(value)
    }
}
