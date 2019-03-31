use nom::types::CompleteStr;
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::forward_to_deserialize_any;
use serde::Deserialize;

use super::{Compat, Error};
use crate::literals;

pub struct Deserializer<'de> {
    input: CompleteStr<'de>,
}

macro_rules! parse_integer {
    ($name:ident, $target:ty) => {
        fn $name(&mut self) -> Result<$target, Error> {
            match self.parse_number()? {
                literals::Number::Float(_) => Err(Error::UnexpectedFloat)?,
                literals::Number::Integer(u) => {
                    let min = <$target>::min_value() as i128;
                    let max = <$target>::max_value() as i128;
                    if u < min || u > max {
                        Err(Error::Overflow(stringify!($target)))
                    } else {
                        Ok(u as $target)
                    }
                }
            }
        }
    }
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer {
            input: CompleteStr(input),
        }
    }

    fn parse_bool(&mut self) -> Result<bool, Error> {
        let (remaining, output) = literals::boolean(self.input)?;
        self.input = remaining;
        Ok(output)
    }

    fn parse_number(&mut self) -> Result<literals::Number, Error> {
        let (remaining, output) = literals::number(self.input)?;
        self.input = remaining;
        Ok(output)
    }

    parse_integer!(parse_i8, i8);
    parse_integer!(parse_i16, i16);
    parse_integer!(parse_i32, i32);
    parse_integer!(parse_i64, i64);
    parse_integer!(parse_u8, u8);
    parse_integer!(parse_u16, u16);
    parse_integer!(parse_u32, u32);
    parse_integer!(parse_u64, u64);
    parse_integer!(parse_u128, u128);

    fn parse_i128(&mut self) -> Result<i128, Error> {
        match self.parse_number()? {
            literals::Number::Float(_) => Err(Error::UnexpectedFloat)?,
            literals::Number::Integer(u) => Ok(u),
        }
    }

    /// Possibly Lossy
    fn parse_f32(&mut self) -> Result<f32, Error> {
        match self.parse_number()? {
            literals::Number::Integer(i) => Ok(i as f32),
            literals::Number::Float(f) => Ok(f as f32),
        }
    }

    fn parse_f64(&mut self) -> Result<f64, Error> {
        match self.parse_number()? {
            literals::Number::Integer(i) => Ok(i as f64),
            literals::Number::Float(f) => Ok(f),
        }
    }
}

macro_rules! deserialize_scalars {
    ($name:ident, $visit:ident, $parse:ident) => {
        fn $name<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            visitor.$visit(self.$parse()?)
        }
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
        char str
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    deserialize_scalars!(deserialize_bool, visit_bool, parse_bool);
    deserialize_scalars!(deserialize_i8, visit_i8, parse_i8);
    deserialize_scalars!(deserialize_i16, visit_i16, parse_i16);
    deserialize_scalars!(deserialize_i32, visit_i32, parse_i32);
    deserialize_scalars!(deserialize_i64, visit_i64, parse_i64);
    deserialize_scalars!(deserialize_i128, visit_i128, parse_i128);
    deserialize_scalars!(deserialize_u8, visit_u8, parse_u8);
    deserialize_scalars!(deserialize_u16, visit_u16, parse_u16);
    deserialize_scalars!(deserialize_u32, visit_u32, parse_u32);
    deserialize_scalars!(deserialize_u64, visit_u64, parse_u64);
    deserialize_scalars!(deserialize_u128, visit_u128, parse_u128);
    deserialize_scalars!(deserialize_f32, visit_f32, parse_f32);
    deserialize_scalars!(deserialize_f64, visit_f64, parse_f64);

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
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

    #[test]
    fn deserialize_integer() {
        let mut deserializer = Deserializer::from_str("12345");
        let deserialized: u32 = u32::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, 12345);

        let mut deserializer = Deserializer::from_str("-12345");
        let deserialized: i32 = i32::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, -12345);
    }

    #[test]
    fn deserialize_float() {
        let mut deserializer = Deserializer::from_str("12345");
        let deserialized: f64 = f64::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, 12345.);

        let mut deserializer = Deserializer::from_str("-12345.12");
        let deserialized: f32 = f32::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, -12345.12);
    }
}
