use std::borrow::Cow;
use std::ops::Deref;

use nom::{alt_complete, call, named};

/// A "key" in a map
#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub enum Key<'a> {
    Identifier(Cow<'a, str>),
    String(Cow<'a, str>),
}

impl<'a> Key<'a> {
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

/// Parse a "key" for a map
named!(
    pub key(&str) -> Key,
    alt_complete!(
        call!(crate::utils::wrap_str(super::identifier)) => { |s| Key::Identifier(Cow::Borrowed(s)) }
        | super::string => { |s| Key::String(Cow::Owned(s)) }
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

        for (input, expected) in test_cases.into_iter() {
            println!("Testing {}", input);
            assert_eq!(key(input).unwrap_output(), *expected);
        }
    }
}
