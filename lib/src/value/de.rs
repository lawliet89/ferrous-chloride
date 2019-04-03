use serde::de::{Deserializer, Visitor};
use serde::forward_to_deserialize_any;

use crate::serde::de::{self, Compat};
use crate::Value;

impl<'de, 'a> Deserializer<'de> for Value<'a> {
    type Error = Compat;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        use crate::Value::*;

        match self {
            Null => visitor.visit_unit(),
            Integer(integer) => visitor.visit_i64(integer),
            Float(float) => visitor.visit_f64(float),
            Boolean(boolean) => visitor.visit_bool(boolean),
            String(string) => visitor.visit_string(string),
            List(list) => visitor.visit_seq(de::list::ListAccess::new(list)),
            // Map(map) => visitor.visit_map(de::map::MapAccess::new(map)?),
            Map(_map) => unimplemented!("Not yet"),
            Block(_block) => unimplemented!("Not yet"),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
