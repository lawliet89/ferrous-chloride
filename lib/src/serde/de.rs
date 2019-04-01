use nom::types::CompleteStr;
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::forward_to_deserialize_any;
use serde::Deserialize;

use super::{Compat, Error};
use crate::literals;
use crate::value;

pub struct Deserializer<'de> {
    input: CompleteStr<'de>,
}

macro_rules! parse_integer {
    ($name:ident, $target:ty) => {
        #[allow(clippy::cast_lossless)]
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
    #[allow(clippy::should_implement_trait)]
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

    fn parse_string(&mut self) -> Result<String, Error> {
        let (remaining, output) = literals::string(self.input)?;
        self.input = remaining;
        Ok(output)
    }

    fn parse_bytes(&mut self) -> Result<Vec<u8>, Error> {
        let (remaining, list) = value::list(self.input)?;
        self.input = remaining;
        // Check that we are all numbers and fits within u8
        let numbers = list
            .into_iter()
            .map(|value| {
                value.integer().map_err(Error::from).and_then(|integer| {
                    #[allow(clippy::cast_lossless)]
                    let min = u8::min_value() as i128;
                    #[allow(clippy::cast_lossless)]
                    let max = u8::max_value() as i128;

                    if integer < min || integer > max {
                        Err(Error::Overflow(stringify!(u8)))
                    } else {
                        Ok(integer as u8)
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(numbers)
    }

    fn parse_null(&mut self) -> Result<(), Error> {
        let (remaining, ()) = literals::null(self.input)?;
        self.input = remaining;
        Ok(())
    }

    fn peek(&mut self) -> Result<value::Value, Error> {
        let (remaining, peek) = value::peek(self.input)?;
        self.input = remaining;
        Ok(peek)
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
        use value::Value::*;
        match self.peek()? {
            Null => self.deserialize_unit(visitor),
            Boolean(_) => self.deserialize_bool(visitor),
            Integer(_) => self.deserialize_i128(visitor),
            Float(_) => self.deserialize_f64(visitor),
            String(_) => self.deserialize_string(visitor),
            List(_) => self.deserialize_seq(visitor),
            Map(_) => self.deserialize_map(visitor),
            Block(_) => self.deserialize_map(visitor),
        }
    }

    forward_to_deserialize_any! {
        newtype_struct seq tuple
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

    // TODO: Borrowed string?
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.parse_string()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let parsed = self.parse_string()?;
        let parsed = if parsed.len() != 1 {
            Err(Error::ExpectedCharacterGotString(parsed))?
        } else {
            parsed
        };
        let character = parsed.chars().next().expect("to have one character");
        visitor.visit_char(character)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.parse_bytes()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_null()?;
        visitor.visit_unit()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.peek()? {
            value::Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
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

    use serde::Deserialize;
    use serde_bytes::ByteBuf;

    #[test]
    fn deserialize_boolean() {
        let mut deserializer = Deserializer::from_str("true");
        let deserialized = bool::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, true);

        let mut deserializer = Deserializer::from_str("false");
        let deserialized = bool::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, false);
    }

    #[test]
    fn deserialize_integer() {
        let mut deserializer = Deserializer::from_str("12345");
        let deserialized = u32::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, 12345);

        let mut deserializer = Deserializer::from_str("-12345");
        let deserialized = i32::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, -12345);
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn deserialize_integer_checks_for_overflow() {
        let mut deserializer = Deserializer::from_str("12345");
        let _ = u8::deserialize(&mut deserializer).unwrap();
    }

    #[test]
    #[allow(clippy::float_cmp)] // Don't be a pedant!
    fn deserialize_float() {
        let mut deserializer = Deserializer::from_str("12345");
        let deserialized = f64::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, 12345.);

        let mut deserializer = Deserializer::from_str("-12345.12");
        let deserialized = f32::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, -12345.12);
    }

    #[test]
    fn deserialize_string() {
        let test_cases = [
            (r#""""#, ""),
            (r#""abcd""#, r#"abcd"#),
            (r#""ab\"cd""#, r#"ab"cd"#),
            (r#""ab \\ cd""#, r#"ab \ cd"#),
            (r#""ab \n cd""#, "ab \n cd"),
            (r#""ab \? cd""#, "ab ? cd"),
            (
                r#"<<EOF
    EOF
"#,
                "",
            ),
            (
                r#""ab \xff \251 \uD000 \U29000""#,
                "ab ÿ © \u{D000} \u{29000}",
            ),
            (
                r#"<<EOF
something
    EOF
"#,
                "something",
            ),
            (
                r#"<<EOH
something
with
new lines
and quotes "
                        EOH
"#,
                r#"something
with
new lines
and quotes ""#,
            ),
        ];

        for (input, expected) in test_cases.iter() {
            println!("Testing {}", input);
            let mut deserializer = Deserializer::from_str(input);
            let deserialized = String::deserialize(&mut deserializer).unwrap();
            assert_eq!(&deserialized, expected);
        }
    }

    #[test]
    fn deserialize_char() {
        let mut deserializer = Deserializer::from_str("\"c\"");
        let deserialized = char::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, 'c');
    }

    #[test]
    #[should_panic(expected = "ExpectedCharacterGotString")]
    fn deserialize_char_should_error_on_strings() {
        let mut deserializer = Deserializer::from_str("\"foobar\"");
        let _ = char::deserialize(&mut deserializer).unwrap();
    }

    #[test]
    fn deserialize_bytes() {
        let string = b"hello world";
        let byte_string = format!("{:?}", string);

        let mut deserializer = Deserializer::from_str(&byte_string);
        let deserialized = ByteBuf::deserialize(&mut deserializer).unwrap();

        let actual: &[u8] = deserialized.as_ref();
        assert_eq!(actual, string);
    }

    #[test]
    #[should_panic(expected = "UnexpectedVariant")]
    fn deserialize_bytes_errors_on_invalid_entries() {
        let mut deserializer = Deserializer::from_str("[1, false]");
        let _ = ByteBuf::deserialize(&mut deserializer).unwrap();
    }

    #[test]
    #[should_panic(expected = "Overflow")]
    fn deserialize_bytes_errors_on_overflow() {
        let mut deserializer = Deserializer::from_str("[1, 999]");
        let _ = ByteBuf::deserialize(&mut deserializer).unwrap();
    }

    #[test]
    fn deserializes_unit() {
        let mut deserializer = Deserializer::from_str("null");
        Deserialize::deserialize(&mut deserializer).unwrap()
    }

    #[test]
    fn deserialize_option() {
        let mut deserializer = Deserializer::from_str("null");
        let deserialized: Option<u32> = Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, None);

        let mut deserializer = Deserializer::from_str("42");
        let deserialized: Option<u32> = Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, Some(42));
    }

    #[test]
    fn deserialize_unit_struct() {
        #[derive(Deserialize)]
        struct Unit;

        let mut deserializer = Deserializer::from_str("null");
        let _unit = Unit::deserialize(&mut deserializer).unwrap();
    }
}
