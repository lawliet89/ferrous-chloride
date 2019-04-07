use std::borrow::{Borrow, Cow};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use nom::types::CompleteStr;
use nom::{alt_complete, call, named};

#[cfg(feature = "serde")]
pub use self::serde::*;

/// A "key" in a map
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Key<'a> {
    Identifier(Cow<'a, str>),
    String(Cow<'a, str>),
}

impl<'a> Key<'a> {
    pub fn new_identifier(s: &'a str) -> Self {
        Key::Identifier(Cow::Borrowed(s))
    }

    pub fn new_identifier_owned(s: String) -> Self {
        Key::Identifier(Cow::Owned(s))
    }

    pub fn new_string(s: &'a str) -> Self {
        Key::String(Cow::Borrowed(s))
    }

    pub fn new_string_owned(s: String) -> Self {
        Key::String(Cow::Owned(s))
    }

    pub fn unwrap(self) -> Cow<'a, str> {
        match self {
            Key::Identifier(s) => s,
            Key::String(s) => s,
        }
    }

    /// Returns key where nothing is borrowed
    pub fn as_owned(&self) -> Key<'static> {
        match self {
            Key::Identifier(s) => Key::Identifier(Cow::Owned(s.to_string())),
            Key::String(s) => Key::String(Cow::Owned(s.to_string())),
        }
    }
}

impl<'a> crate::AsOwned for Key<'a> {
    type Output = Key<'static>;

    fn as_owned(&self) -> Key<'static> {
        match self {
            Key::Identifier(s) => Key::Identifier(Cow::Owned(s.to_string())),
            Key::String(s) => Key::String(Cow::Owned(s.to_string())),
        }
    }
}

impl<'a> Deref for Key<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Key::Identifier(s) => s.deref(),
            Key::String(s) => s.deref(),
        }
    }
}

impl<'a> From<&'a str> for Key<'a> {
    fn from(s: &'a str) -> Self {
        Key::Identifier(Cow::Borrowed(s))
    }
}

impl<'a> From<String> for Key<'a> {
    fn from(s: String) -> Self {
        Key::Identifier(Cow::Owned(s))
    }
}

impl<'a> Borrow<str> for Key<'a> {
    fn borrow(&self) -> &str {
        self.deref()
    }
}

impl<'a> Hash for Key<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

// Parse a "key" for a map
named!(
    pub key(CompleteStr) -> Key,
    alt_complete!(
        call!(crate::parser::identifier::identifier) => { |s| Key::Identifier(s) }
        | crate::parser::string::string_literal => { |s| Key::String(Cow::Owned(s)) }
    )
);

#[cfg(feature = "serde")]
mod serde {
    use ::serde::de::{Deserializer, Visitor};
    use ::serde::forward_to_deserialize_any;

    use super::*;
    use crate::serde::de::Compat;

    impl<'de, 'a> Deserializer<'de> for Key<'a> {
        type Error = Compat;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            match self {
                Key::Identifier(cow) | Key::String(cow) => visitor.visit_str(&cow),
            }
        }

        forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::ResultUtilsString;

    #[test]
    fn keys_are_parsed_correctly() {
        let test_cases = [
            ("abcd123", Key::Identifier(From::from("abcd123"))),
            ("_abc", Key::Identifier(From::from("_abc"))),
            ("゛藏_a", Key::Identifier(From::from("゛藏_a"))),
            (r#""123""#, Key::String(From::from("123"))),
            (r#""a/b/c""#, Key::String(From::from("a/b/c"))),
        ];

        for (input, expected) in test_cases.iter() {
            println!("Testing {}", input);
            assert_eq!(key(CompleteStr(input)).unwrap_output(), *expected);
        }
    }
}
