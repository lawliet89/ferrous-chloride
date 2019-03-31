use nom::types::CompleteStr;
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::forward_to_deserialize_any;
use serde::Deserialize;

use super::{Compat, Error};

pub struct Deserializer<'de> {
    input: CompleteStr<'de>,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer {
            input: CompleteStr(input),
        }
    }

    fn parse_bool(&mut self) -> Result<bool, Error> {
        let (remaining, output) = crate::literals::boolean(self.input)?;
        self.input = remaining;
        Ok(output)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Compat;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Custom("Not implemented".to_string()))?
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T, Compat>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_boolean() {
        let mut deserializer = Deserializer::from_str("true");
        let deserialized: bool = bool::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, true);

        let mut deserializer = Deserializer::from_str("false");
        let deserialized: bool = bool::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, false);
    }

}
