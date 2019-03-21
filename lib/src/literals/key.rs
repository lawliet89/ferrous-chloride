use std::borrow::{Borrow, Cow};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use nom::types::CompleteStr;
use nom::{alt_complete, call, named};

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

/// Parse a "key" for a map
named!(
    pub key(CompleteStr) -> Key,
    alt_complete!(
        call!(super::identifier) => { |s| Key::Identifier(Cow::Borrowed(s)) }
        | super::string::quoted_single_line_string => { |s| Key::String(Cow::Owned(s)) }
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::ResultUtilsString;

    #[test]
    fn keys_are_parsed_correctly() {
        let test_cases = [
            ("abcd123", Key::Identifier(From::from("abcd123"))),
            ("_abc", Key::Identifier(From::from("_abc"))),
            ("藏_①", Key::Identifier(From::from("藏_①"))),
            (r#""123""#, Key::String(From::from("123"))),
            (r#""a/b/c""#, Key::String(From::from("a/b/c"))),
        ];

        for (input, expected) in test_cases.iter() {
            println!("Testing {}", input);
            assert_eq!(key(CompleteStr(input)).unwrap_output(), *expected);
        }
    }
}
