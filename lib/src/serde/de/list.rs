use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::de::Deserializer;
use serde::forward_to_deserialize_any;

use crate::value;
use super::Compat;

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
        use value::Value::*;

        // FIXME: Is this OK?
        let item = self.list.pop().expect("to not be empty");
        match item {
            Null => visitor.visit_unit(),
            Integer(integer) => visitor.visit_i64(integer),
            Float(float) => visitor.visit_f64(float),
            Boolean(boolean) => visitor.visit_bool(boolean),
            String(string) => visitor.visit_string(string),
            List(list) => unimplemented!("Not yet"),
            Map(map) => unimplemented!("Not yet"),
            Block(block) => unimplemented!("Not yet"),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
