use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::string::ToString;

use crate::constants::*;
use crate::literals::{self, Key};
use crate::{Error, KeyValuePairs, ScalarLength};

use nom::types::CompleteStr;
use nom::{
    alt, alt_complete, call, char, complete, do_parse, many0, many1, map, named, opt, preceded,
    tag, terminated, ws,
};

#[derive(Debug, PartialEq, Clone)]
/// Value in HCL
pub enum Value<'a> {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    List(List<'a>),
    Map(Map<'a>),
    Block(Block<'a>),
}

pub type Block<'a> = KeyValuePairs<Vec<String>, MapValues<'a>>;

pub type Map<'a> = Vec<MapValues<'a>>;

pub type MapValues<'a> = KeyValuePairs<Key<'a>, Value<'a>>;

pub type List<'a> = Vec<Value<'a>>;

impl<'a> Value<'a> {
    pub fn new_list<T>(iterator: T) -> Self
    where
        T: IntoIterator<Item = Value<'a>>,
    {
        Value::List(iterator.into_iter().collect())
    }

    pub fn new_map<I, T>(iterator: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoIterator<Item = (Key<'a>, Value<'a>)>,
    {
        Value::Map(
            iterator
                .into_iter()
                .map(|iter| iter.into_iter().collect())
                .collect(),
        )
    }

    pub fn new_single_map<T>(iterator: T) -> Self
    where
        T: IntoIterator<Item = (Key<'a>, Value<'a>)>,
    {
        Value::Map(vec![iterator.into_iter().collect()])
    }

    pub fn new_block<S, T>(keys: &'a [S], iterator: T) -> Self
    where
        S: AsRef<str>,
        T: IntoIterator<Item = (Key<'a>, Value<'a>)>,
    {
        let keys: Vec<String> = keys.iter().map(|s| s.as_ref().to_string()).collect();
        let map: MapValues = iterator.into_iter().collect();
        let block: Block = [(keys, map)].iter().cloned().collect();
        Value::Block(block)
    }

    pub fn variant_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => INTEGER,
            Value::Float(_) => FLOAT,
            Value::Boolean(_) => BOOLEAN,
            Value::String(_) => STRING,
            Value::List(_) => LIST,
            Value::Map(_) => MAP,
            Value::Block(_) => BLOCK,
        }
    }

    pub fn is_scalar(&self) -> bool {
        match self {
            Value::Integer(_) | Value::Float(_) | Value::Boolean(_) | Value::String(_) => true,
            _ => false,
        }
    }

    pub fn is_aggregate(&self) -> bool {
        !self.is_scalar()
    }

    /// "Top" level length
    pub fn len(&self) -> usize {
        if self.is_scalar() {
            1
        } else {
            match self {
                Value::List(vector) => vector.len(),
                Value::Map(vectors) => vectors.len(),
                Value::Block(block) => block.len(),
                _ => unreachable!("Impossible to reach this. This is a bug."),
            }
        }
    }

    /// Whether Value is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn integer(&self) -> Result<i64, Error> {
        if let Value::Integer(i) = self {
            Ok(*i)
        } else {
            Err(Error::UnexpectedVariant {
                enum_type: VALUE,
                expected: INTEGER,
                actual: self.variant_name(),
            })
        }
    }

    /// # Panics
    /// Panics if the variant is not an integer
    pub fn unwrap_integer(&self) -> i64 {
        self.integer().unwrap()
    }

    pub fn float(&self) -> Result<f64, Error> {
        if let Value::Float(f) = self {
            Ok(*f)
        } else {
            Err(Error::UnexpectedVariant {
                enum_type: VALUE,
                expected: FLOAT,
                actual: self.variant_name(),
            })
        }
    }

    /// # Panics
    /// Panics if the variant is not a float
    pub fn unwrap_float(&self) -> f64 {
        self.float().unwrap()
    }

    pub fn boolean(&self) -> Result<bool, Error> {
        if let Value::Boolean(v) = self {
            Ok(*v)
        } else {
            Err(Error::UnexpectedVariant {
                enum_type: VALUE,
                expected: BOOLEAN,
                actual: self.variant_name(),
            })
        }
    }

    /// # Panics
    /// Panics if the variant is not a boolean
    pub fn unwrap_boolean(&self) -> bool {
        self.boolean().unwrap()
    }

    pub fn borrow_str(&self) -> Result<&str, Error> {
        if let Value::String(v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedVariant {
                enum_type: VALUE,
                expected: STRING,
                actual: self.variant_name(),
            })
        }
    }

    /// # Panics
    /// Panics if the variant is not a string
    pub fn unwrap_borrow_str(&self) -> &str {
        self.borrow_str().unwrap()
    }

    pub fn borrow_string_mut(&mut self) -> Result<&mut String, Error> {
        if let Value::String(ref mut v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedVariant {
                enum_type: VALUE,
                expected: STRING,
                actual: self.variant_name(),
            })
        }
    }

    /// # Panics
    /// Panics if the variant is not a string
    pub fn unwrap_borrow_string_mut(&mut self) -> &mut String {
        self.borrow_string_mut().unwrap()
    }

    pub fn string(self) -> Result<String, (Error, Self)> {
        if let Value::String(v) = self {
            Ok(v)
        } else {
            Err((
                Error::UnexpectedVariant {
                    enum_type: VALUE,
                    expected: STRING,
                    actual: self.variant_name(),
                },
                self,
            ))
        }
    }

    /// # Panics
    /// Panics if the variant is not a string
    pub fn unwrap_string(self) -> String {
        self.string().unwrap()
    }

    pub fn borrow_list(&self) -> Result<&List<'a>, Error> {
        if let Value::List(v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedVariant {
                enum_type: VALUE,
                expected: LIST,
                actual: self.variant_name(),
            })
        }
    }

    /// # Panics
    /// Panics if the variant is not a string
    pub fn unwrap_borrow_list(&self) -> &List<'_> {
        self.borrow_list().unwrap()
    }

    pub fn borrow_list_mut(&mut self) -> Result<&mut List<'a>, Error> {
        if let Value::List(ref mut v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedVariant {
                enum_type: VALUE,
                expected: LIST,
                actual: self.variant_name(),
            })
        }
    }

    /// # Panics
    /// Panics if the variant is not a list
    pub fn unwrap_borrow_list_mut(&mut self) -> &mut List<'a> {
        self.borrow_list_mut().unwrap()
    }

    pub fn list(self) -> Result<List<'a>, (Error, Self)> {
        if let Value::List(v) = self {
            Ok(v)
        } else {
            Err((
                Error::UnexpectedVariant {
                    enum_type: VALUE,
                    expected: LIST,
                    actual: self.variant_name(),
                },
                self,
            ))
        }
    }

    /// # Panics
    /// Panics if the variant is not a list
    pub fn unwrap_list(self) -> List<'a> {
        self.list().unwrap()
    }

    pub fn borrow_map(&self) -> Result<&Map<'a>, Error> {
        if let Value::Map(v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedVariant {
                enum_type: VALUE,
                expected: MAP,
                actual: self.variant_name(),
            })
        }
    }

    /// # Panics
    /// Panics if the variant is not a string
    pub fn unwrap_borrow_map(&self) -> &Map<'_> {
        self.borrow_map().unwrap()
    }

    pub fn borrow_map_mut(&mut self) -> Result<&mut Map<'a>, Error> {
        if let Value::Map(ref mut v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedVariant {
                enum_type: VALUE,
                expected: MAP,
                actual: self.variant_name(),
            })
        }
    }

    /// # Panics
    /// Panics if the variant is not a map
    pub fn unwrap_borrow_map_mut(&mut self) -> &mut Map<'a> {
        self.borrow_map_mut().unwrap()
    }

    pub fn map(self) -> Result<Map<'a>, (Error, Self)> {
        if let Value::Map(v) = self {
            Ok(v)
        } else {
            Err((
                Error::UnexpectedVariant {
                    enum_type: VALUE,
                    expected: MAP,
                    actual: self.variant_name(),
                },
                self,
            ))
        }
    }

    /// # Panics
    /// Panics if the variant is not a map
    pub fn unwrap_map(self) -> Map<'a> {
        self.map().unwrap()
    }

    pub fn borrow_block(&self) -> Result<&Block<'a>, Error> {
        if let Value::Block(v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedVariant {
                enum_type: VALUE,
                expected: BLOCK,
                actual: self.variant_name(),
            })
        }
    }

    /// # Panics
    /// Panics if the variant is not a string
    pub fn unwrap_borrow_block(&self) -> &Block<'_> {
        self.borrow_block().unwrap()
    }

    pub fn borrow_block_mut(&mut self) -> Result<&mut Block<'a>, Error> {
        if let Value::Block(ref mut v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedVariant {
                enum_type: VALUE,
                expected: BLOCK,
                actual: self.variant_name(),
            })
        }
    }

    /// # Panics
    /// Panics if the variant is not a block
    pub fn unwrap_borrow_block_mut(&mut self) -> &mut Block<'a> {
        self.borrow_block_mut().unwrap()
    }

    pub fn block(self) -> Result<Block<'a>, (Error, Self)> {
        if let Value::Block(v) = self {
            Ok(v)
        } else {
            Err((
                Error::UnexpectedVariant {
                    enum_type: VALUE,
                    expected: BLOCK,
                    actual: self.variant_name(),
                },
                self,
            ))
        }
    }

    /// # Panics
    /// Panics if the variant is not a block
    pub fn unwrap_block(self) -> Block<'a> {
        self.block().unwrap()
    }

    /// Recursively merge value
    pub fn merge(self) -> Result<Self, Error> {
        match self {
            no_op @ Value::Integer(_)
            | no_op @ Value::Float(_)
            | no_op @ Value::Boolean(_)
            | no_op @ Value::String(_) => Ok(no_op),
            Value::List(list) => Ok(Value::List(
                list.into_iter()
                    .map(Value::merge)
                    .collect::<Result<_, _>>()?,
            )),
            Value::Map(maps) => Ok(Value::Map(
                maps.into_iter()
                    .map(MapValues::merge)
                    .collect::<Result<_, _>>()?,
            )),
            Value::Block(block) => Ok(Value::Block(
                block
                    .into_iter()
                    .map(|(key, value)| Ok((key, value.merge()?)))
                    .collect::<Result<_, _>>()?,
            )),
        }
    }
}

impl<'a> ScalarLength for Value<'a> {
    fn len_scalar(&self) -> usize {
        if self.is_scalar() {
            1
        } else {
            match self {
                Value::List(vector) => vector.len_scalar(),
                Value::Map(vectors) => vectors.len_scalar(),
                Value::Block(block) => block.len_scalar(),
                _ => unreachable!("Impossible to reach this. This is a bug."),
            }
        }
    }
}

impl<'a> crate::Mergeable for Value<'a> {
    fn is_merged(&self) -> bool {
        if self.is_scalar() {
            true
        } else {
            match self {
                Value::List(vector) => vector.is_merged(),
                Value::Map(vectors) => vectors.is_merged(),
                Value::Block(block) => block.is_merged(),
                _ => unreachable!("Impossible to reach this. This is a bug."),
            }
        }
    }
}

macro_rules! impl_from_value (
    ($variant: ident, $type: ty) => (
        impl<'a> From<$type> for Value<'a> {
            fn from(v: $type) -> Self {
                Value::$variant(v)
            }
        }
    )
);

impl<'a, 'b, T> From<&'b T> for Value<'a>
where
    T: Into<Value<'a>> + Clone,
{
    fn from(v: &'b T) -> Value<'a> {
        Into::into(v.clone())
    }
}

impl_from_value!(Integer, i64);
impl_from_value!(Float, f64);
impl_from_value!(Boolean, bool);
impl_from_value!(String, String);
impl_from_value!(Map, Vec<MapValues<'a>>);
impl_from_value!(Block, Block<'a>);

/// Special Snowflake treatment for &str and friends
impl<'a, 'b> From<&'b str> for Value<'a> {
    fn from(s: &'b str) -> Self {
        Value::String(s.to_string())
    }
}

impl<'a> From<Option<Vec<Value<'a>>>> for Value<'a> {
    fn from(l: Option<Vec<Value<'a>>>) -> Self {
        match l {
            None => Value::List(vec![]),
            Some(v) => Value::List(v),
        }
    }
}

impl<'a> From<MapValues<'a>> for Value<'a> {
    fn from(values: MapValues<'a>) -> Self {
        Value::from(vec![values])
    }
}

impl<'a> FromIterator<Value<'a>> for Value<'a> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Value<'a>>,
    {
        let list = iter.into_iter().collect();
        Value::List(list)
    }
}

impl<'a> Block<'a> {
    // TODO: Customise behaviour wrt duplicate block keys
    pub fn new_merged<T, K, S>(iter: T) -> Result<Self, Error>
    where
        T: IntoIterator<Item = (K, MapValues<'a>)>,
        K: IntoIterator<Item = S>,
        S: ToString,
    {
        let mut merged = HashMap::new();
        for (key, value) in iter {
            let _ = merged.insert(
                key.into_iter().map(|s| s.to_string()).collect(),
                value.merge()?,
            );
        }
        Ok(KeyValuePairs::Merged(merged))
    }

    pub fn new_unmerged<T, K, S>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, MapValues<'a>)>,
        K: IntoIterator<Item = S>,
        S: ToString,
    {
        KeyValuePairs::Unmerged(
            iter.into_iter()
                .map(|(keys, value)| (keys.into_iter().map(|s| s.to_string()).collect(), value))
                .collect(),
        )
    }

    /// Borrow the keys as `Vec<&str>` for more ergonomic indexing.
    ///
    /// # Usage
    ///
    /// ```rust
    ///
    /// ```
    ///
    /// # Motivation
    /// A Block is implemented as [`KeyValuePairs`] with `Vec<String>` as keys.
    /// Behind the scenes, a merged [`KeyValuePairs`] is backed by a [`HashMap`].
    ///
    /// Retrieving a key from a [`HashMap`] involves using the [`HashMap::get`] method
    /// which specifies that to lookup a key of type `K`, you may use any type `Q` that
    /// implements [`std::borrow::Borrow`]`<K>`.
    ///
    /// Since `Vec<T>` only implements `Borrow<[T]>`, a `Vec<String>` only implements
    /// `Borrow<[String]>`.
    ///
    /// The implication is that we cannot lookup the `HashMap<Vec<String>, Value>` with a list of
    /// `&str`.
    ///
    /// Consider the following:
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use std::borrow::Borrow;
    ///
    /// let mut hashmap: HashMap<Vec<String>, usize> = HashMap::new();
    /// let _ = hashmap.insert(vec!["a".to_string(), "b".to_string()], 123);
    ///
    /// // Let's try to retrieve the value
    /// // The following won't compile
    /// // let _ = hashmap.get(&["a", "b"]);
    ///
    /// // We have to use this...
    /// let _ = hashmap.get::<[String]>(&["a".to_string(), "b".to_string()]);
    /// ```
    ///
    /// As you can see, this is not ergonomic at all.
    ///
    /// Fundamentally, this is because it's not zero-cost to convert a `Vec<Stirng>` into a
    /// `&[&str]`. See this [question](https://stackoverflow.com/q/41179659/602002) on
    /// StackOverflow.
    ///
    /// # Alternatives
    ///
    /// The unstable [raw entry API](https://github.com/rust-lang/rust/issues/56167) might help with
    /// this in the future.
    pub fn borrow_keys(&self) -> KeyValuePairs<Vec<&str>, &MapValues<'a>> {
        match self {
            KeyValuePairs::Merged(hashmap) => KeyValuePairs::Merged(
                hashmap
                    .iter()
                    .map(|(k, v)| (k.iter().map(|s| s.as_str()).collect(), v))
                    .collect(),
            ),
            KeyValuePairs::Unmerged(vec) => KeyValuePairs::Unmerged(
                vec.iter()
                    .map(|(k, v)| (k.iter().map(|s| s.as_str()).collect(), v))
                    .collect(),
            ),
        }
    }
}

impl<'a, K, S> FromIterator<(K, MapValues<'a>)> for Block<'a>
where
    K: IntoIterator<Item = S>,
    S: ToString,
{
    fn from_iter<T: IntoIterator<Item = (K, MapValues<'a>)>>(iter: T) -> Self {
        Self::new_unmerged(iter)
    }
}

impl<'a> MapValues<'a> {
    // TODO: Customise merging behaviour wrt duplicate keys
    pub fn new_merged<T>(iter: T) -> Result<Self, Error>
    where
        T: IntoIterator<Item = (Key<'a>, Value<'a>)>,
    {
        use std::collections::hash_map::Entry;

        let mut map = HashMap::new();
        for (key, value) in iter {
            let mut value = value.merge()?;
            match map.entry(key) {
                Entry::Vacant(vacant) => {
                    vacant.insert(value);
                }
                Entry::Occupied(mut occupied) => {
                    let key = occupied.key().to_string();
                    match occupied.get_mut() {
                        illegal @ Value::Integer(_)
                        | illegal @ Value::Float(_)
                        | illegal @ Value::Boolean(_)
                        | illegal @ Value::String(_)
                        | illegal @ Value::List(_) => Err(Error::IllegalMultipleEntries {
                            key,
                            variant: illegal.variant_name(),
                        })?,
                        Value::Map(ref mut map) => {
                            // Check that the incoming value is also a Map
                            if let Value::Map(ref mut incoming) = value {
                                map.append(incoming);
                            } else {
                                Err(Error::ErrorMergingKeys {
                                    key,
                                    existing_variant: MAP,
                                    incoming_variant: value.variant_name(),
                                })?;
                            }
                        }
                        Value::Block(ref mut block) => {
                            let value = value;
                            // Check that the incoming value is also a Block
                            if let Value::Block(incoming) = value {
                                block.extend(incoming);
                            } else {
                                Err(Error::ErrorMergingKeys {
                                    key,
                                    existing_variant: BLOCK,
                                    incoming_variant: value.variant_name(),
                                })?;
                            }
                        }
                    };
                }
            };
        }
        Ok(KeyValuePairs::Merged(map))
    }

    pub fn new_unmerged<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Key<'a>, Value<'a>)>,
    {
        KeyValuePairs::Unmerged(iter.into_iter().collect())
    }

    pub fn merge(self) -> Result<Self, Error> {
        if let KeyValuePairs::Unmerged(vec) = self {
            Self::new_merged(vec.into_iter())
        } else {
            Ok(self)
        }
    }

    pub fn as_merged(&self) -> Result<Self, Error> {
        if let KeyValuePairs::Unmerged(vec) = self {
            Self::new_merged(vec.iter().cloned())
        } else {
            Ok(self.clone())
        }
    }

    pub fn unmerge(self) -> Self {
        if let KeyValuePairs::Merged(hashmap) = self {
            Self::new_unmerged(hashmap.into_iter())
        } else {
            self
        }
    }

    pub fn as_unmerged(&self) -> Self {
        if let KeyValuePairs::Merged(hashmap) = self {
            Self::new_unmerged(
                hashmap
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone())),
            )
        } else {
            self.clone()
        }
    }
}

impl<'a> FromIterator<(Key<'a>, Value<'a>)> for MapValues<'a> {
    fn from_iter<T: IntoIterator<Item = (Key<'a>, Value<'a>)>>(iter: T) -> Self {
        Self::new_unmerged(iter)
    }
}

// From https://github.com/Geal/nom/issues/14#issuecomment-158788226
// whitespace! Must not be captured after `]`!
named!(
    pub list(CompleteStr) -> Vec<Value>,
    preceded!(
        whitespace!(char!('[')),
        terminated!(
            whitespace!(
                separated_list!(
                    char!(','),
                    single_value
                )
            ),
            terminated!(
                whitespace!(opt!(char!(','))),
                char!(']')
            )
        )
    )
);

named!(
    pub single_value(CompleteStr) -> Value,
    alt_complete!(
        call!(literals::number) => { |v| From::from(v) }
        | call!(literals::boolean) => { |v| Value::Boolean(v) }
        | literals::string => { |v| Value::String(v) }
        | list => { |v| Value::List(v) }
    )
);

/// Parse single key value pair in the form of
/// `"key" = ... | ["..."] | {...}`
named!(
    pub key_value(CompleteStr) -> (Key, Value),
    space_tab!(
        alt!(
                do_parse!(
                    key: call!(literals::key)
                    >> char!('=')
                    >> value: call!(single_value)
                    >> (key, value)
                )
                | do_parse!(
                    identifier: call!(literals::identifier)
                    >> complete!(opt!(char!('=')))
                    >> whitespace!(char!('{'))
                    >> values: whitespace!(call!(map_values))
                    >> char!('}')
                    >> (Key::Identifier(Cow::Borrowed(identifier)), Value::from(values))
                )
                | do_parse!(
                    identifier: call!(literals::identifier)
                    >> keys: many0!(literals::quoted_single_line_string)
                    >> whitespace!(char!('{'))
                    >> values: whitespace!(call!(map_values))
                    >> char!('}')
                    >> (Key::Identifier(Cow::Borrowed(identifier)), Value::Block(vec![(keys, values)].into_iter().collect()))
                )
        )
    )
);

named!(
    pub map_values(CompleteStr) -> MapValues,
    do_parse!(
        values: many0!(
                    terminated!(
                        call!(key_value),
                        alt!(
                            whitespace!(tag!(","))
                            | map!(many1!(nom::eol), |_| CompleteStr(""))
                        )
                    )
                )
        >> (values.into_iter().collect())
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Mergeable;
    use crate::utils::{assert_list_eq, ResultUtilsString};

    #[test]
    fn list_values_are_parsed_successfully() {
        let test_cases = [
            (r#"[]"#, vec![]),
            (r#"[1,]"#, vec![Value::from(1)]),
            (
                r#"[true, false, 123, -123.456, "foobar"]"#,
                vec![
                    Value::from(true),
                    Value::from(false),
                    Value::from(123),
                    Value::from(-123.456),
                    Value::from("foobar"),
                ],
            ),
            (
                r#"[
                        true,
                        false,
                        123,
                        -123.456,
                        "testing",                        [
                            "inside voice!",
                            "lol"
                        ],
                    ]"#,
                vec![
                    Value::from(true),
                    Value::from(false),
                    Value::from(123),
                    Value::from(-123.456),
                    Value::from("testing"),
                    Value::new_list(vec![Value::from("inside voice!"), Value::from("lol")]),
                ],
            ),
        ];

        for (input, expected_value) in test_cases.iter() {
            println!("Testing {}", input);
            let actual_value = list(CompleteStr(input)).unwrap_output();
            assert_eq!(actual_value, *expected_value);
        }
    }

    #[test]
    fn single_values_are_parsed_successfully() {
        let test_cases = [
            (r#"123"#, Value::Integer(123)),
            ("123", Value::Integer(123)),
            ("123", Value::Integer(123)),
            ("true", Value::Boolean(true)),
            ("123.456", Value::Float(123.456)),
            ("123", Value::Integer(123)),
            (r#""foobar""#, Value::String("foobar".to_string())),
            (
                r#"<<EOF
new
line
EOF
"#,
                Value::String("new\nline".to_string()),
            ),
            (
                r#"[true, false, 123, -123.456, "foobar"]"#,
                Value::new_list(vec![
                    Value::from(true),
                    Value::from(false),
                    Value::from(123),
                    Value::from(-123.456),
                    Value::from("foobar"),
                ]),
            ),
        ];

        for (input, expected_value) in test_cases.iter() {
            println!("Testing {}", input);
            let actual_value = single_value(CompleteStr(input)).unwrap_output();
            assert_eq!(actual_value, *expected_value);
        }
    }

    #[test]
    fn key_value_pairs_are_parsed_successfully() {
        let test_cases = [
            ("test = 123", ("test", Value::Integer(123))),
            ("test = 123", ("test", Value::Integer(123))),
            ("test = true", ("test", Value::Boolean(true))),
            ("test = 123.456", ("test", Value::Float(123.456))),
            ("   test   =   123  ", ("test", Value::Integer(123))), // Random spaces
            (
                r#""a/b/c" = "foobar","#,
                ("a/b/c", Value::String("foobar".to_string())),
            ),
            (
                r#"test = <<EOF
new
line
EOF
"#,
                ("test", Value::String("new\nline".to_string())),
            ),
            (r#"test = [],"#, ("test", Value::List(vec![]))),
            (
                r#"test = [1,]"#,
                ("test", Value::new_list(vec![Value::from(1)])),
            ),
            (
                r#"test = [true, false, 123, -123.456, "foobar"],"#,
                (
                    "test",
                    Value::new_list(vec![
                        Value::from(true),
                        Value::from(false),
                        Value::from(123),
                        Value::from(-123.456),
                        Value::from("foobar"),
                    ]),
                ),
            ),
        ];

        for (input, (expected_key, expected_value)) in test_cases.iter() {
            println!("Testing {}", input);
            let (actual_key, actual_value) = key_value(CompleteStr(input)).unwrap_output();
            assert_eq!(actual_key.unwrap(), *expected_key);
            assert_eq!(actual_value, *expected_value);
        }
    }

    #[test]
    fn maps_are_parsed_correctly() {
        let test_cases = [
            (
                r#"test {
foo = "bar"
}"#,
                (
                    "test",
                    Value::new_single_map(vec![(From::from("foo"), Value::from("bar"))]),
                ),
            ),
            (
                r#"test = {
foo = "bar"


}"#,
                (
                    "test",
                    Value::new_single_map(vec![(From::from("foo"), Value::from("bar"))]),
                ),
            ),
            (
                r#"test "one" "two" {
            foo = "bar"
            }"#,
                (
                    "test",
                    Value::new_block(
                        &["one", "two"],
                        vec![(From::from("foo"), Value::from("bar"))],
                    ),
                ),
            ),
        ];

        for (input, (expected_key, expected_value)) in test_cases.iter() {
            println!("Testing {}", input);
            let (actual_key, actual_value) = key_value(CompleteStr(input)).unwrap_output();
            assert_eq!(actual_key.unwrap(), *expected_key);
            assert_eq!(actual_value, *expected_value);
        }
    }

    #[test]
    fn empty_map_values_are_parsed_correctly() {
        let hcl = include_str!("../fixtures/empty.hcl");
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();

        assert_eq!(0, parsed.len());
    }

    #[test]
    fn single_map_values_are_parsed_correctly() {
        let hcl = include_str!("../fixtures/single.hcl");
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();

        assert_eq!(1, parsed.len());
        assert_eq!(parsed["foo"], Value::from("bar"));
    }

    #[test]
    fn scalar_map_values_are_parsed_correctly() {
        let hcl = include_str!("../fixtures/scalar.hcl");
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();

        let expected: HashMap<_, _> = vec![
            ("test_unsigned_int", Value::from(123)),
            ("test_signed_int", Value::from(-123)),
            ("test_float", Value::from(-1.23)),
            ("bool_true", Value::from(true)),
            ("bool_false", Value::from(false)),
            ("comma_separed", Value::from("oh my, a rebel!")),
            ("string", Value::from("Hello World!")),
            ("long_string", Value::from("hihi\nanother line!")),
            ("string_escaped", Value::from("\" Hello World!")),
        ]
        .into_iter()
        .collect();

        assert_eq!(expected.len(), parsed.len());
        for (expected_key, expected_value) in expected {
            println!("Checking {}", expected_key);
            let actual_value = &parsed[expected_key];
            assert_eq!(*actual_value, expected_value);
        }
    }

    #[test]
    fn list_map_values_are_parsed_correctly() {
        let hcl = include_str!("../fixtures/list.hcl");
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();

        let expected: HashMap<_, _> = vec![
            (
                "list",
                Value::new_list(vec![
                    Value::from(true),
                    Value::from(false),
                    Value::from(123),
                    Value::from(-123.456),
                    Value::from("foobar"),
                ]),
            ),
            (
                "list_multi",
                Value::new_list(vec![
                    Value::from(true),
                    Value::from(false),
                    Value::from(123),
                    Value::from(-123.456),
                    Value::from("foobar"),
                ]),
            ),
            (
                "list_in_list",
                Value::new_list(vec![
                    Value::new_list(vec![Value::from("test"), Value::from("foobar")]),
                    Value::from(1),
                    Value::from(2),
                    Value::from(-3),
                ]),
            ),
        ]
        .into_iter()
        .collect();

        assert_eq!(expected.len(), parsed.len());
        for (expected_key, expected_value) in expected {
            println!("Checking {}", expected_key);
            let actual_value = &parsed[expected_key];
            assert_eq!(*actual_value, expected_value);
        }
    }

    #[test]
    fn multiple_maps_are_parsed_correctly() {
        let hcl = include_str!("../fixtures/map.hcl");
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();
        assert!(parsed.is_unmerged());

        assert_eq!(parsed.len(), 5); // unmerged values
        assert_eq!(parsed.len_scalar(), 19);

        // simple_map
        let simple_map = parsed.get("simple_map").unwrap().unwrap_many();
        assert_eq!(simple_map.len(), 2);

        let expected_simple_maps = vec![
            vec![MapValues::new_unmerged(vec![
                (Key::new_identifier("foo"), Value::from("bar")),
                (Key::new_identifier("bar"), Value::from("baz")),
                (Key::new_identifier("index"), Value::from(1)),
            ])],
            vec![MapValues::new_unmerged(vec![
                (Key::new_identifier("foo"), Value::from("bar")),
                (Key::new_identifier("bar"), Value::from("baz")),
                (Key::new_identifier("index"), Value::from(0)),
            ])],
        ];
        let actual_simple_map: Vec<_> = simple_map
            .into_iter()
            .map(|v| v.borrow_map().expect("to be a map"))
            .collect();
        assert_list_eq!(expected_simple_maps, actual_simple_map);

        // resource
        let resources = parsed.get("resource").unwrap().unwrap_many();
        assert_eq!(resources.len(), 3);
        let resources: Vec<_> = resources
            .into_iter()
            .map(|v| v.borrow_block().expect("to be a block"))
            .collect();

        let expected_resources = vec![
            Block::new_unmerged(vec![(
                vec!["security/group", "foobar"],
                MapValues::new_unmerged(vec![
                    (Key::new_identifier("name"), Value::from("foobar")),
                    (
                        Key::new_identifier("allow"),
                        Value::Map(vec![MapValues::new_unmerged(vec![
                            (Key::new_identifier("name"), Value::from("localhost")),
                            (
                                Key::new_identifier("cidrs"),
                                vec![Value::from("127.0.0.1/32")].into_iter().collect(),
                            ),
                        ])]),
                    ),
                    (
                        Key::new_identifier("allow"),
                        Value::Map(vec![MapValues::new_unmerged(vec![
                            (Key::new_identifier("name"), Value::from("lan")),
                            (
                                Key::new_identifier("cidrs"),
                                vec![Value::from("192.168.0.0/16")].into_iter().collect(),
                            ),
                        ])]),
                    ),
                    (
                        Key::new_identifier("deny"),
                        Value::Map(vec![MapValues::new_unmerged(vec![
                            (Key::new_identifier("name"), Value::from("internet")),
                            (
                                Key::new_identifier("cidrs"),
                                vec![Value::from("0.0.0.0/0")].into_iter().collect(),
                            ),
                        ])]),
                    ),
                ]),
            )]),
            Block::new_unmerged(vec![(
                vec!["security/group", "second"],
                MapValues::new_unmerged(vec![
                    (Key::new_identifier("name"), Value::from("second")),
                    (
                        Key::new_identifier("allow"),
                        Value::Map(vec![MapValues::new_unmerged(vec![
                            (Key::new_identifier("name"), Value::from("all")),
                            (
                                Key::new_identifier("cidrs"),
                                vec![Value::from("0.0.0.0/0")].into_iter().collect(),
                            ),
                        ])]),
                    ),
                ]),
            )]),
            Block::new_unmerged(vec![(
                vec!["instance", "an_instance"],
                MapValues::new_unmerged(vec![
                    (Key::new_identifier("name"), Value::from("an_instance")),
                    (Key::new_identifier("image"), Value::from("ubuntu:18.04")),
                    (
                        Key::new_identifier("user"),
                        Value::Block(Block::new_unmerged(vec![(
                            vec!["test"],
                            MapValues::new_unmerged(vec![(
                                Key::new_identifier("root"),
                                Value::from(true),
                            )]),
                        )])),
                    ),
                ]),
            )]),
        ];
        assert_list_eq(expected_resources, resources);
    }

    // TODO: Tests for merging

    #[test]
    fn maps_are_merged_correctly() {
        let hcl = include_str!("../fixtures/map.hcl");
        let parsed = map_values(CompleteStr(hcl)).unwrap_output();
        assert!(parsed.is_unmerged());
        let parsed = parsed.merge().unwrap();
        assert!(parsed.is_merged());
        println!("{:#?}", parsed);

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed.len_scalar(), 19);

        // simple_map
        let simple_map = parsed.get("simple_map").unwrap().unwrap_one();
        assert_eq!(simple_map.len(), 2);

        let expected_simple_maps = vec![
            MapValues::new_merged(vec![
                (Key::new_identifier("foo"), Value::from("bar")),
                (Key::new_identifier("bar"), Value::from("baz")),
                (Key::new_identifier("index"), Value::from(1)),
            ])
            .unwrap(),
            MapValues::new_merged(vec![
                (Key::new_identifier("foo"), Value::from("bar")),
                (Key::new_identifier("bar"), Value::from("baz")),
                (Key::new_identifier("index"), Value::from(0)),
            ])
            .unwrap(),
        ];
        let simple_maps = simple_map.unwrap_borrow_map();
        println!("{:#?}", simple_maps);
        assert!(simple_maps.iter().eq(&expected_simple_maps));

        // resource
        let resource = parsed.get("resource").unwrap().unwrap_one();
        assert_eq!(resource.len(), 3);
        let resource = resource.unwrap_borrow_block();

        let sg_foobar = resource
            .borrow_keys()
            .get::<[&str]>(&["security/group", "foobar"])
            .unwrap()
            .unwrap_one();

        // let sg_foobar = resource[&["security/group".to_string(), "foobar".to_string()]];
    }
}
