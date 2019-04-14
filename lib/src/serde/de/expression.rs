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
            Tuple(tuple) => deserialize_tuple(tuple, visitor, None),
            Object(_object) => unimplemented!("Not yet"),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Expression::Null => visitor.visit_none(),
            other => visitor.visit_some(other),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Expression::Tuple(tuple) => deserialize_tuple(tuple, visitor, Some(len)),
            other => other.deserialize_any(visitor),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq
        map struct enum identifier ignored_any
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
    fn deserializes_unit() {
        let deserializer = Expression::Null;
        Deserialize::deserialize(deserializer).unwrap()
    }

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

    #[test]
    #[should_panic(expected = "expected u8")]
    fn deserialize_bytes_errors_on_overflow() {
        use serde_bytes::ByteBuf;

        let deserializer = Expression::Tuple(vec![Expression::from(1), Expression::from(999)]);
        let _ = ByteBuf::deserialize(deserializer).unwrap();
    }

    #[test]
    fn deserialize_option() {
        let deserializer = Expression::Null;
        let deserialized: Option<u32> = Deserialize::deserialize(deserializer).unwrap();
        assert_eq!(deserialized, None);

        let deserializer = Expression::from(42);
        let deserialized: Option<u32> = Deserialize::deserialize(deserializer).unwrap();
        assert_eq!(deserialized, Some(42));
    }

    #[test]
    fn deserialize_unit_struct() {
        #[derive(Deserialize)]
        struct Unit;

        let deserializer = Expression::Null;
        let _unit = Unit::deserialize(deserializer).unwrap();
    }

    #[test]
    fn deserialize_newtype_struct() {
        #[derive(Deserialize)]
        struct Newtype(pub bool);

        let deserializer = Expression::from(true);
        let newtype = Newtype::deserialize(deserializer).unwrap();
        assert!(newtype.0);
    }

    #[test]
    fn deserialize_list_of_scalars() {
        use std::collections::HashSet;

        let deserializer = Expression::Tuple(
            vec![1, 2, 3, 4, 5]
                .into_iter()
                .map(Expression::from)
                .collect(),
        );
        let deserialized: Vec<u32> = Deserialize::deserialize(deserializer).unwrap();
        assert_eq!(deserialized, &[1, 2, 3, 4, 5]);

        let deserializer =
            Expression::Tuple(vec![(); 4].into_iter().map(Expression::from).collect());
        let deserialized: Vec<()> = Deserialize::deserialize(deserializer).unwrap();
        assert_eq!(deserialized, &[(), (), (), ()]);

        let deserializer = Expression::Tuple(
            vec![true, false]
                .into_iter()
                .map(Expression::from)
                .collect(),
        );
        let deserialized: HashSet<bool> = Deserialize::deserialize(deserializer).unwrap();
        assert_eq!(deserialized, [true, false].iter().cloned().collect());

        let deserializer = Expression::Tuple(
            vec![
                Expression::Tuple(vec![1, 2, 9].into_iter().map(Expression::from).collect()),
                Expression::Tuple(vec![3, 4, 5].into_iter().map(Expression::from).collect()),
            ]
            .into_iter()
            .map(Expression::from)
            .collect(),
        );
        let deserialized: Vec<Vec<u32>> = Deserialize::deserialize(deserializer).unwrap();
        assert_eq!(deserialized, &[&[1, 2, 9], &[3, 4, 5]]);

        // let mut deserializer = Deserializer::from_str("[1, \"string\", true, null, 5.2]");
        // let deserialized: Vec<value::Value> = Deserialize::deserialize(&mut deserializer).unwrap();
        // assert_eq!(deserialized, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn deserialize_tuples_of_scalars() {
        let deserializer =
            Expression::Tuple(vec![1, 2, 3].into_iter().map(Expression::from).collect());
        let deserialized: (u32, i32, i16) = Deserialize::deserialize(deserializer).unwrap();
        assert_eq!(deserialized, (1, 2, 3));

        let deserializer = Expression::Tuple(vec![
            Expression::from(1),
            Expression::from(true),
            Expression::Null,
        ]);
        let deserialized: (f32, bool, ()) = Deserialize::deserialize(deserializer).unwrap();
        assert_eq!(deserialized, (1., true, ()));
    }

    #[test]
    #[should_panic(expected = "InvalidTupleLength { expected: 3, actual: 4 }")]
    fn deserialize_tuples_errors_on_invalid_length() {
        let deserializer =
            Expression::Tuple(vec![1, 2, 3, 4].into_iter().map(Expression::from).collect());
        let _: (u32, i32, i16) = Deserialize::deserialize(deserializer).unwrap();
    }

    #[test]
    fn deserialize_tuple_structs_of_scalars() {
        #[derive(Deserialize, Eq, PartialEq, Debug)]
        struct TupleOne(u32, i32, i16);

        let deserializer =
            Expression::Tuple(vec![1, 2, 3].into_iter().map(Expression::from).collect());
        let deserialized: TupleOne = Deserialize::deserialize(deserializer).unwrap();
        assert_eq!(deserialized, TupleOne(1, 2, 3));

        #[derive(Deserialize, PartialEq, Debug)]
        struct TupleTwo<'a>(f32, bool, &'a str);

        let deserializer = Expression::Tuple(vec![
            Expression::from(1),
            Expression::from(true),
            Expression::from("null"),
        ]);
        let deserialized: TupleTwo = Deserialize::deserialize(deserializer).unwrap();
        assert_eq!(deserialized, TupleTwo(1., true, "null"));
    }
}
