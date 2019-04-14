use serde::de::{self, IntoDeserializer, Visitor};
use serde::forward_to_deserialize_any;

use crate::parser::expression::Expression;
use crate::serde::de::{deserialize_number, deserialize_string, deserialize_tuple, Compat};

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
            Tuple(tuple) => deserialize_tuple(tuple, visitor),
            Object(_object) => unimplemented!("Not yet"),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> IntoDeserializer<'de, Compat> for Expression<'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::borrow::Cow;

    #[test]
    fn deserialize_boolean() {
        let expression = Expression::from(true);
        let deserialized = bool::deserialize(expression).unwrap();
        assert_eq!(deserialized, true);

        let expression = Expression::from(false);
        let deserialized = bool::deserialize(expression).unwrap();
        assert_eq!(deserialized, false);
    }

    #[test]
    fn deserialize_integer() {
        let expression = Expression::from(12345);
        let deserialized = u32::deserialize(expression).unwrap();
        assert_eq!(deserialized, 12345);

        let expression = Expression::from(-12345);
        let deserialized = i32::deserialize(expression).unwrap();
        assert_eq!(deserialized, -12345);
    }

    #[test]
    #[should_panic(expected = "expected u8")]
    fn deserialize_integer_checks_for_overflow() {
        let expression = Expression::from(12345);
        let _ = u8::deserialize(expression).unwrap();
    }

    #[test]
    #[allow(clippy::float_cmp)] // Don't be a pedant!
    fn deserialize_float() {
        let expression = Expression::from(12345);
        let deserialized = f64::deserialize(expression).unwrap();
        assert_eq!(deserialized, 12345.);

        let expression = Expression::from(-12345.12);
        let deserialized = f32::deserialize(expression).unwrap();
        assert_eq!(deserialized, -12345.12);
    }

    #[test]
    fn deserialize_string() {
        let expression = Expression::String(Cow::Owned("foobar".to_string()));
        let deserialized = String::deserialize(expression).unwrap();
        assert_eq!(deserialized, "foobar");
    }

    #[test]
    fn deserialize_borrowed_string() {
        let string = "foobar".to_string();
        let expression = Expression::String(Cow::Borrowed(&string));
        let deserialized: &str = Deserialize::deserialize(expression).unwrap();
        assert_eq!(deserialized, string);
    }

    #[test]
    #[should_panic(expected = "expected a borrowed string")]
    fn deserialize_borrowed_string_errors_when_invalid() {
        let deserializer = Expression::String(Cow::Owned("input".to_string()));
        let _: &str = Deserialize::deserialize(deserializer).unwrap();
    }

    #[test]
    fn deserialize_char() {
        let deserializer = Expression::from("c");
        let deserialized = char::deserialize(deserializer).unwrap();
        assert_eq!(deserialized, 'c');
    }

    #[test]
    #[should_panic(expected = "expected a character")]
    fn deserialize_char_should_error_on_strings() {
        let deserializer = Expression::from("this will fail");
        let _ = char::deserialize(deserializer).unwrap();
    }

    #[test]
    fn deserialize_bytes() {
        use serde_bytes::ByteBuf;

        let string = b"hello world";
        let deserializer =
            Expression::Tuple(string.to_vec().into_iter().map(Expression::from).collect());
        let deserialized = ByteBuf::deserialize(deserializer).unwrap();

        let actual: &[u8] = deserialized.as_ref();
        assert_eq!(actual, string);
    }

    #[test]
    #[should_panic(expected = "expected u8")]
    fn deserialize_bytes_errors_on_invalid_entries() {
        use serde_bytes::ByteBuf;

        let deserializer = Expression::Tuple(vec![Expression::from(false), Expression::from("hi")]);
        let _ = ByteBuf::deserialize(deserializer).unwrap();
    }
}
