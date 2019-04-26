//! Deserializer Implementation
//!
//! This module contains the types and trait implementation to allow deserialization from a HCL
//! string to Rust types that you can usually disregard. To find out more
//! about _using_ them, head to [`serde` documentation](https://serde.rs/).
pub mod block;
pub mod body;
pub mod expression;
pub mod object;

use std::borrow::Cow;

use nom::types::CompleteStr;
use serde::de::{self, IntoDeserializer, Visitor};
use serde::{forward_to_deserialize_any, Deserialize, Serialize};

use crate::parser;

pub use self::error::*;

mod error {
    use std::fmt::Display;
    use std::ops::Deref;

    use failure::{self, Fail};

    /// Error type for deserialization
    #[derive(Debug, Fail)]
    pub enum Error {
        #[fail(display = "HCL parse error: {}", _0)]
        ParseError(#[cause] crate::Error),

        #[fail(display = "Error parsing integer: {}", _0)]
        ParseIntError(#[cause] std::num::ParseIntError),

        #[fail(display = "Error parsing float: {}", _0)]
        ParseFloatError(#[cause] std::num::ParseFloatError),

        #[fail(display = "Input was not completely consumed during deserialization.")]
        TrailingCharacters,

        #[fail(display = "Overflow when trying to convert to {}", _0)]
        Overflow(&'static str),

        #[fail(
            display = "Invalid tuple length. Expected {}, got {}",
            expected, actual
        )]
        InvalidTupleLength { expected: usize, actual: usize },

        #[fail(display = "Object has duplicate key {}", _0)]
        ObjectDuplicateKey(String),

        #[fail(display = "Body has duplicate key {}", _0)]
        BodyDuplicateKey(String),

        #[fail(display = "{}", _0)]
        Custom(String),
    }

    impl From<crate::Error> for Error {
        fn from(e: crate::Error) -> Self {
            Error::ParseError(e)
        }
    }

    impl<I> From<nom::Err<I, u32>> for Error
    where
        I: nom::AsBytes + AsRef<str> + std::fmt::Debug,
    {
        fn from(e: nom::Err<I, u32>) -> Self {
            let parse_error = crate::Error::from_err_str(&e);
            From::from(parse_error)
        }
    }

    impl From<std::num::ParseIntError> for Error {
        fn from(e: std::num::ParseIntError) -> Self {
            Error::ParseIntError(e)
        }
    }

    impl From<std::num::ParseFloatError> for Error {
        fn from(e: std::num::ParseFloatError) -> Self {
            Error::ParseFloatError(e)
        }
    }

    #[derive(Debug)]
    pub struct Compat(pub failure::Compat<Error>);

    impl Deref for Compat {
        type Target = failure::Compat<Error>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl serde::de::Error for Compat {
        fn custom<T>(msg: T) -> Self
        where
            T: Display,
        {
            From::from(Error::Custom(msg.to_string()))
        }
    }

    impl Display for Compat {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            self.0.fmt(f)
        }
    }

    impl std::error::Error for Compat {}

    impl From<Error> for Compat {
        fn from(e: Error) -> Self {
            Compat(e.compat())
        }
    }

    impl From<crate::Error> for Compat {
        fn from(e: crate::Error) -> Self {
            From::from(Error::ParseError(e))
        }
    }

    impl From<Compat> for Error {
        fn from(e: Compat) -> Self {
            e.0.into_inner()
        }
    }

}

pub struct Deserializer<'de> {
    input: CompleteStr<'de>,
}

macro_rules! parse_number {
    ($name:ident, $target:ty) => {
        #[allow(clippy::cast_lossless)]
        fn $name(&mut self) -> Result<$target, Error> {
            Ok(self.parse_number()?.parse()?)
        }
    }
}

fn deserialize_string<'de, V>(string: Cow<'de, str>, visitor: V) -> Result<V::Value, Compat>
where
    V: Visitor<'de>,
{
    match string {
        Cow::Borrowed(string) => visitor.visit_borrowed_str(string),
        Cow::Owned(string) => visitor.visit_string(string),
    }
}

fn deserialize_number<'de, V>(
    number: parser::number::Number<'de>,
    visitor: V,
) -> Result<V::Value, Compat>
where
    V: Visitor<'de>,
{
    if number.is_float() {
        visitor.visit_f64(number.as_f64().map_err(Error::ParseFloatError)?)
    } else if number.is_signed() {
        visitor.visit_i64(number.as_i64().map_err(Error::ParseIntError)?)
    } else {
        visitor.visit_u64(number.as_u64().map_err(Error::ParseIntError)?)
    }
}

fn deserialize_tuple<'de, V>(
    tuple: parser::tuple::Tuple<'de>,
    visitor: V,
    check_length: Option<usize>,
) -> Result<V::Value, Compat>
where
    V: Visitor<'de>,
{
    if let Some(len) = check_length {
        if tuple.len() != len {
            Err(Error::InvalidTupleLength {
                expected: len,
                actual: tuple.len(),
            })?;
        }
    }

    visitor.visit_seq(tuple.into_deserializer())
}

fn deserialize_object<'de, V>(
    object: parser::object::Object<'de>,
    visitor: V,
) -> Result<V::Value, Compat>
where
    V: Visitor<'de>,
{
    visitor.visit_map(object::ObjectMapAccess::new(object))
}

impl<'de> Deserializer<'de> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &'de str) -> Self {
        Deserializer {
            input: CompleteStr(input),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    fn parse_bool(&mut self) -> Result<bool, Error> {
        let (remaining, output) = parser::boolean::boolean(self.input)?;
        self.input = remaining;
        Ok(output)
    }

    fn parse_number(&mut self) -> Result<parser::number::Number, Error> {
        let (remaining, output) = parser::number::number(self.input)?;
        self.input = remaining;
        Ok(output)
    }

    parse_number!(parse_i8, i8);
    parse_number!(parse_i16, i16);
    parse_number!(parse_i32, i32);
    parse_number!(parse_i64, i64);
    parse_number!(parse_i128, i128);
    parse_number!(parse_u8, u8);
    parse_number!(parse_u16, u16);
    parse_number!(parse_u32, u32);
    parse_number!(parse_u64, u64);
    parse_number!(parse_u128, u128);
    parse_number!(parse_f32, f32);
    parse_number!(parse_f64, f64);

    fn parse_string(&mut self) -> Result<Cow<'de, str>, Error> {
        let (remaining, output) = parser::string::string(self.input)?;
        self.input = remaining;
        Ok(output)
    }

    fn parse_null(&mut self) -> Result<(), Error> {
        let (remaining, ()) = parser::null::null(self.input)?;
        self.input = remaining;
        Ok(())
    }

    fn parse_list(&mut self) -> Result<parser::tuple::Tuple<'de>, Error> {
        let (remaining, list) = parser::tuple::tuple(self.input)?;
        self.input = remaining;
        Ok(list)
    }

    fn parse_object_identifier(
        &mut self,
    ) -> Result<parser::object::ObjectElementIdentifier<'de>, Error> {
        let (remaining, ident) = parser::object::object_element_identifier(self.input)?;
        self.input = remaining;
        Ok(ident)
    }

    fn parse_object(&mut self) -> Result<parser::object::Object<'de>, Error> {
        let (remaining, object) = parser::object::object(self.input)?;
        self.input = remaining;
        Ok(object)
    }

    fn parse_expression(&mut self) -> Result<parser::expression::Expression<'de>, Error> {
        let (remaining, expr) = parser::expression::expression(self.input)?;
        self.input = remaining;
        Ok(expr)
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
        // This is an expensive procedure!
        let expression = self.parse_expression();
        if let Ok(expr) = expression {
            return expr.deserialize_any(visitor);
        }

        unimplemented!("Unknown");
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

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        deserialize_string(self.parse_string()?, visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
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
        match self.parse_null() {
            Ok(()) => visitor.visit_none(),
            Err(_) => visitor.visit_some(self),
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

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let list = self.parse_list()?;
        deserialize_tuple(list, visitor, None)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let list = self.parse_list()?;
        deserialize_tuple(list, visitor, Some(len))
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

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.parse_object_identifier()? {
            parser::object::ObjectElementIdentifier::Identifier(string) => {
                deserialize_string(string, visitor)
            }
            _ => unimplemented!("Not supported"),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // FIXME: Can be an object or block...
        let object = self.parse_object()?;
        deserialize_object(object, visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // FIXME: Can be an object or block...
        self.deserialize_map(visitor)
    }

    forward_to_deserialize_any! {
        enum
    }
}

/// Deserialize a type `T` from a provided HCL String
///
/// ```rust
/// # use ferrous_chloride::serde::from_str;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, PartialEq, Debug)]
/// struct DeserializeMe {
///     name: String,
///     allow: bool,
///     index: usize,
///     list: Vec<String>,
///     nothing: Option<f64>,
/// }
///
/// let input = r#"
/// name = "second"
/// allow = false
/// index = 1
/// list = ["foo", "bar", "baz"]"#;
///
/// let deserialized: DeserializeMe = from_str(input).unwrap();
/// ```
pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

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
    fn deserialize_borrowed_string() {
        let input = r#"<<EOF
something
    EOF
"#;
        let expected = "something";
        let mut deserializer = Deserializer::from_str(input);

        let deserialized: &str = Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    #[should_panic(expected = "expected a borrowed string")]
    fn deserialize_borrowed_string_errors_when_invalid() {
        let input = r#""impossible!\n""#;
        let mut deserializer = Deserializer::from_str(input);
        let _: &str = Deserialize::deserialize(&mut deserializer).unwrap();
    }

    #[test]
    fn deserialize_char() {
        let mut deserializer = Deserializer::from_str("\"c\"");
        let deserialized = char::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, 'c');
    }

    #[test]
    #[should_panic(expected = "expected a character")]
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
    #[should_panic(expected = "expected u8")]
    fn deserialize_bytes_errors_on_invalid_entries() {
        let mut deserializer = Deserializer::from_str("[1, false]");
        let _ = ByteBuf::deserialize(&mut deserializer).unwrap();
    }

    #[test]
    #[should_panic(expected = "expected u8")]
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

    #[test]
    fn deserialize_newtype_struct() {
        #[derive(Deserialize)]
        struct Newtype(pub bool);

        let mut deserializer = Deserializer::from_str("true");
        let newtype = Newtype::deserialize(&mut deserializer).unwrap();
        assert!(newtype.0);
    }

    #[test]
    fn deserialize_list_of_scalars() {
        use std::collections::HashSet;

        let mut deserializer = Deserializer::from_str("[1, 2, 3, 4, 5]");
        let deserialized: Vec<u32> = Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, &[1, 2, 3, 4, 5]);

        let mut deserializer = Deserializer::from_str("[null, null, null, null]");
        let deserialized: Vec<()> = Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, &[(), (), (), ()]);

        let mut deserializer = Deserializer::from_str("[true, false]");
        let deserialized: HashSet<bool> = Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, [true, false].iter().cloned().collect());

        let mut deserializer = Deserializer::from_str("[[1, 2, 9], [3, 4, 5]]");
        let deserialized: Vec<Vec<u32>> = Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, &[&[1, 2, 9], &[3, 4, 5]]);

        // let mut deserializer = Deserializer::from_str("[1, \"string\", true, null, 5.2]");
        // let deserialized: Vec<value::Value> = Deserialize::deserialize(&mut deserializer).unwrap();
        // assert_eq!(deserialized, &[1, 2, 3, 4, 5]);
    }

    // TODO: Deserialize more complicated nested things

    #[test]
    fn deserialize_tuples_of_scalars() {
        let mut deserializer = Deserializer::from_str("[1, 2, 3]");
        let deserialized: (u32, i32, i16) = Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, (1, 2, 3));

        let mut deserializer = Deserializer::from_str("[1, true, null]");
        let deserialized: (f32, bool, ()) = Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, (1., true, ()));
    }

    #[test]
    #[should_panic(expected = "InvalidTupleLength { expected: 3, actual: 4 }")]
    fn deserialize_tuples_errors_on_invalid_length() {
        let mut deserializer = Deserializer::from_str("[1, 2, 3, 4]");
        let _: (u32, i32, i16) = Deserialize::deserialize(&mut deserializer).unwrap();
    }

    #[test]
    fn deserialize_tuple_structs_of_scalars() {
        #[derive(Deserialize, Eq, PartialEq, Debug)]
        struct TupleOne(u32, i32, i16);

        let mut deserializer = Deserializer::from_str("[1, 2, 3]");
        let deserialized: TupleOne = Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, TupleOne(1, 2, 3));

        #[derive(Deserialize, PartialEq, Debug)]
        struct TupleTwo(f32, bool, String);

        let mut deserializer = Deserializer::from_str("[1, true, \"null\"]");
        let deserialized: TupleTwo = Deserialize::deserialize(&mut deserializer).unwrap();
        assert_eq!(deserialized, TupleTwo(1., true, "null".to_string()));
    }

    #[test]
    fn deserialize_simple_maps() {
        let input = r#"{
    test = "foo"
    bar  = "baz"
}"#;
        let mut deserializer = Deserializer::from_str(input);
        let deserialized: HashMap<String, String> =
            Deserialize::deserialize(&mut deserializer).unwrap();

        let expected: HashMap<_, _> = [("test", "foo"), ("bar", "baz")]
            .iter()
            .cloned()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        assert_eq!(deserialized, expected);
    }

    #[test]
    fn deserialize_simple_structs() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct DeserializeMe {
            name: String,
            allow: bool,
            index: usize,
            list: Vec<String>,
            nothing: Option<f64>,
        }

        let input = r#"
name = "second"
allow = false
index = 1
list = ["foo", "bar", "baz"]}
"#;
        let mut deserializer = Deserializer::from_str(input);
        let deserialized: DeserializeMe = Deserialize::deserialize(&mut deserializer).unwrap();

        let expected = DeserializeMe {
            name: "second".to_string(),
            allow: false,
            index: 1,
            list: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
            nothing: None,
        };

        assert_eq!(expected, deserialized);
    }

    //     #[test]
    //     fn deserialize_nested_structs() {
    //         #[derive(Deserialize, PartialEq, Debug)]
    //         struct SecurityGroup {
    //             name: String,
    //             allow: Allow,
    //         }

    //         #[derive(Deserialize, PartialEq, Debug)]
    //         struct Allow {
    //             name: String,
    //             cidrs: Vec<String>,
    //         }

    //         let input = r#"
    //   name = "second"

    //   allow {
    //     name = "all"
    //     cidrs = ["0.0.0.0/0"]
    //   }
    // "#;
    //         let mut deserializer = Deserializer::from_str(input);
    //         let deserialized: SecurityGroup = Deserialize::deserialize(&mut deserializer).unwrap();
    //     }
}
