use serde::de::{self, Visitor};
use serde::forward_to_deserialize_any;

use crate::parser::expression::Expression;
use crate::parser::number::Number;
use crate::serde::de::{deserialize_string, Compat};

/// Copy the implementation of [`nom::recognize_float`] to check which visitor method to use
fn deserialize_number<'de, V>(number: Number<'de>, visitor: V) -> Result<V::Value, Compat>
where
    V: Visitor<'de>,
{
    unimplemented!("Not yet")
}

impl<'de> de::Deserializer<'de> for Expression<'de> {
    type Error = Compat;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        use Expression::*;
        match self {
            Null => visitor.visit_unit(),
            Number(number) => deserialize_number(number, visitor),
            Boolean(boolean) => visitor.visit_bool(boolean),
            String(string) => deserialize_string(string, visitor),
            Tuple(_tuple) => unimplemented!("Not yet"),
            Object(_object) => unimplemented!("Not yet"),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {}
}
