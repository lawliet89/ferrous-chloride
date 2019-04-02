use serde::de::{Deserialize, DeserializeSeed, Deserializer, SeqAccess, Visitor};
use serde::forward_to_deserialize_any;

use super::Compat;
use crate::value;

pub struct ListAccess<'a> {
    // List is reversed!
    list: value::List<'a>,
}

impl<'de, 'a> ListAccess<'a> {
    pub(crate) fn new(mut list: value::List<'a>) -> Self {
        list.reverse();
        Self { list }
    }
}

impl<'de, 'a> SeqAccess<'de> for ListAccess<'a> {
    type Error = Compat;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        // Check if there are no more elements.
        if self.list.is_empty() {
            return Ok(None);
        }

        // Deserialize an array element.
        seed.deserialize(&mut *self).map(Some)
    }
}

impl<'de, 'a> Deserializer<'de> for &mut ListAccess<'a> {
    type Error = Compat;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // FIXME: Is this OK?
        let item = self.list.pop().expect("to not be empty");
        item.deserialize_any(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
