use std::collections::HashMap;
use std::iter::FromIterator;
use std::string::ToString;

use nom::types::CompleteStr;

use crate::constants::*;
use crate::parser::literals::Key;
use crate::MergeBehaviour;
use crate::{AsOwned, Error, KeyValuePairs, ScalarLength};

#[derive(Debug, PartialEq, Clone)]
/// Value in HCL
pub enum Value<'a> {
    Null,
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    List(List<'a>),
    Object(Object<'a>),
    Block(Block<'a>),
}

// TODO: Make Value Generic over type of
// Merged/unmerged object

pub type Body<'a> = MapValues<'a>;

/// Contains a list of HCL Blocks sharing the same identifier with one or more labels
/// differentiating each Block from each other.
pub type Block<'a> = KeyValuePairs<Vec<String>, MapValues<'a>>;

pub type Object<'a> = Vec<MapValues<'a>>;

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
        Value::Object(
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
        Value::Object(vec![iterator.into_iter().collect()])
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
            Value::Null => NULL,
            Value::Integer(_) => INTEGER,
            Value::Float(_) => FLOAT,
            Value::Boolean(_) => BOOLEAN,
            Value::String(_) => STRING,
            Value::List(_) => LIST,
            Value::Object(_) => OBJECT,
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
                Value::Object(vectors) => vectors.len(),
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

    pub fn borrow_map(&self) -> Result<&Object<'a>, Error> {
        if let Value::Object(v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedVariant {
                enum_type: VALUE,
                expected: OBJECT,
                actual: self.variant_name(),
            })
        }
    }

    /// # Panics
    /// Panics if the variant is not a string
    pub fn unwrap_borrow_map(&self) -> &Object<'_> {
        self.borrow_map().unwrap()
    }

    pub fn borrow_map_mut(&mut self) -> Result<&mut Object<'a>, Error> {
        if let Value::Object(ref mut v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedVariant {
                enum_type: VALUE,
                expected: OBJECT,
                actual: self.variant_name(),
            })
        }
    }

    /// # Panics
    /// Panics if the variant is not a map
    pub fn unwrap_borrow_map_mut(&mut self) -> &mut Object<'a> {
        self.borrow_map_mut().unwrap()
    }

    pub fn map(self) -> Result<Object<'a>, (Error, Self)> {
        if let Value::Object(v) = self {
            Ok(v)
        } else {
            Err((
                Error::UnexpectedVariant {
                    enum_type: VALUE,
                    expected: OBJECT,
                    actual: self.variant_name(),
                },
                self,
            ))
        }
    }

    /// # Panics
    /// Panics if the variant is not a map
    pub fn unwrap_map(self) -> Object<'a> {
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
            no_op @ Value::Null
            | no_op @ Value::Integer(_)
            | no_op @ Value::Float(_)
            | no_op @ Value::Boolean(_)
            | no_op @ Value::String(_) => Ok(no_op),
            Value::List(list) => Ok(Value::List(
                list.into_iter()
                    .map(Value::merge)
                    .collect::<Result<_, _>>()?,
            )),
            Value::Object(maps) => Ok(Value::Object(
                maps.into_iter()
                    .map(MapValues::merge)
                    .collect::<Result<_, _>>()?,
            )),
            Value::Block(block) => {
                let unmerged: Block = block
                    .into_iter()
                    .map(|(key, value)| Ok((key, value.merge()?)))
                    .collect::<Result<_, Error>>()?;
                let merged = Block::new_merged(unmerged)?;
                Ok(Value::Block(merged))
            }
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Value::Null => true,
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        match self {
            Value::Integer(_) => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Value::Float(_) => true,
            _ => false,
        }
    }

    pub fn is_boolean(&self) -> bool {
        match self {
            Value::Boolean(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Value::String(_) => true,
            _ => false,
        }
    }

    pub fn is_list(&self) -> bool {
        match self {
            Value::List(_) => true,
            _ => false,
        }
    }

    pub fn is_map(&self) -> bool {
        match self {
            Value::Object(_) => true,
            _ => false,
        }
    }

    pub fn is_block(&self) -> bool {
        match self {
            Value::Block(_) => true,
            _ => false,
        }
    }

    pub fn is_body(&self) -> bool {
        self.is_map()
    }
}

impl<'a> ScalarLength for Value<'a> {
    fn len_scalar(&self) -> usize {
        if self.is_scalar() {
            1
        } else {
            match self {
                Value::List(vector) => vector.len_scalar(),
                Value::Object(vectors) => vectors.len_scalar(),
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
                Value::Object(vectors) => vectors.is_merged(),
                Value::Block(block) => block.is_merged(),
                _ => unreachable!("Impossible to reach this. This is a bug."),
            }
        }
    }

    fn is_unmerged(&self) -> bool {
        if self.is_scalar() {
            true
        } else {
            match self {
                Value::List(vector) => vector.is_unmerged(),
                Value::Object(vectors) => vectors.is_unmerged(),
                Value::Block(block) => block.is_unmerged(),
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
impl_from_value!(Object, Vec<MapValues<'a>>);
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

impl<'a> AsOwned for Value<'a> {
    type Output = Value<'static>;

    fn as_owned(&self) -> Self::Output {
        match self {
            Value::Null => Value::Null,
            Value::Integer(i) => Value::Integer(*i),
            Value::Float(f) => Value::Float(*f),
            Value::Boolean(b) => Value::Boolean(*b),
            Value::String(ref string) => Value::String(string.clone()),
            Value::List(ref vec) => Value::List(vec.as_owned()),
            Value::Object(ref map) => Value::Object(map.as_owned()),
            Value::Block(ref block) => Value::Block(block.as_owned()),
        }
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
        let mut merged = HashMap::default();
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

    /// Borrow the keys as `Vec<&str>` for more ergonomic indexing.
    ///
    /// # Usage
    ///
    /// ```ignore
    /// use ferrous_chloride::parser::literals::Key;
    /// use ferrous_chloride::value::*;
    ///
    /// let block = Block::new_unmerged(vec![(
    ///     vec!["instance", "an_instance"],
    ///     MapValues::new_unmerged(vec![
    ///         (Key::new_identifier("name"), Value::from("an_instance")),
    ///         (Key::new_identifier("image"), Value::from("ubuntu:18.04")),
    ///         (
    ///             Key::new_identifier("user"),
    ///             Value::Block(Block::new_unmerged(vec![(
    ///                 vec!["test"],
    ///                 MapValues::new_unmerged(vec![(
    ///                     Key::new_identifier("root"),
    ///                     Value::from(true),
    ///                 )]),
    ///             )])),
    ///         ),
    ///     ]),
    /// )]);
    /// let block = block.merge().unwrap();
    /// let instance = block
    ///     .borrow_keys()
    ///     .get::<[&str]>(&["instance", "an_instance"])
    ///     .unwrap()
    ///     .unwrap_one();
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
    /// let mut hashmap: HashMap<Vec<String>, usize> = HashMap::default();
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
                    .map(|(k, v)| (k.iter().map(String::as_str).collect(), v))
                    .collect(),
            ),
            KeyValuePairs::Unmerged(vec) => KeyValuePairs::Unmerged(
                vec.iter()
                    .map(|(k, v)| (k.iter().map(String::as_str).collect(), v))
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

        let mut map = HashMap::default();
        for (key, value) in iter {
            let mut value = value.merge()?;
            match map.entry(key) {
                Entry::Vacant(vacant) => {
                    vacant.insert(value);
                }
                Entry::Occupied(mut occupied) => {
                    let key = occupied.key().to_string();
                    match occupied.get_mut() {
                        illegal @ Value::Null
                        | illegal @ Value::Integer(_)
                        | illegal @ Value::Float(_)
                        | illegal @ Value::Boolean(_)
                        | illegal @ Value::String(_)
                        | illegal @ Value::List(_) => {
                            return Err(Error::IllegalMultipleEntries {
                                key,
                                variant: illegal.variant_name(),
                            })
                        }
                        Value::Object(ref mut map) => {
                            // Check that the incoming value is also a Object
                            if let Value::Object(ref mut incoming) = value {
                                map.append(incoming);
                            } else {
                                return Err(Error::ErrorMergingKeys {
                                    key,
                                    existing_variant: OBJECT,
                                    incoming_variant: value.variant_name(),
                                });
                            }
                        }
                        Value::Block(ref mut block) => {
                            let value = value;
                            // Check that the incoming value is also a Block
                            if let Value::Block(incoming) = value {
                                block.extend(incoming);
                            } else {
                                return Err(Error::ErrorMergingKeys {
                                    key,
                                    existing_variant: BLOCK,
                                    incoming_variant: value.variant_name(),
                                });
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

/// Parse a HCL string into a [`Body`] which is close to an abstract syntax tree of the
/// HCL string.
///
/// You can opt to merge the parsed body after parsing. The behaviour of merging is determined by
/// the [`MergeBehaviour`] enum.
pub fn from_str(input: &str, merge: Option<MergeBehaviour>) -> Result<Body, Error> {
    let (remaining_input, unmerged) =
        crate::parser::body(CompleteStr(input)).map_err(|e| Error::from_err_str(&e))?;

    if !remaining_input.is_empty() {
        return Err(Error::Bug(format!(
            r#"Input was not completely parsed:
Input: {},
Remaining: {}
"#,
            input, remaining_input
        )));
    }

    let pairs = match merge {
        None => unmerged,
        Some(MergeBehaviour::Error) => unmerged.merge()?,
        Some(_) => unimplemented!("Not implemented yet"),
    };

    Ok(pairs)
}

/// Parse a HCL string from a IO stream reader
///
/// The entire IO stream has to be buffered in memory first before parsing can occur.
///
/// When reading from a source against which short reads are not efficient, such as a
/// [`File`](std::fs::File), you will want to apply your own buffering because the library
/// will not buffer the input. See [`std::io::BufReader`].
pub fn from_reader<R: std::io::Read>(
    mut reader: R,
    merge: Option<MergeBehaviour>,
) -> Result<Body<'static>, Error> {
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;

    // FIXME: Can we do better? We are allocating twice. Once for reading into a buffer
    // and second time calling `as_owned`.
    Ok(from_str(&buffer, merge)?.as_owned())
}

/// Parse a HCL string from a slice of bytes
pub fn from_slice(bytes: &[u8], merge: Option<MergeBehaviour>) -> Result<Body, Error> {
    let input = std::str::from_utf8(bytes)?;
    from_str(input, merge)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;
    use crate::Mergeable;

    #[test]
    fn strings_are_parsed_correctly_unmerged() {
        for string in fixtures::ALL {
            let parsed = from_str(string, None).unwrap();
            assert!(parsed.is_unmerged());
        }
    }

    #[test]
    fn strings_are_parsed_correctly_merged() {
        for string in fixtures::ALL {
            let parsed = from_str(string, Some(MergeBehaviour::Error)).unwrap();
            assert!(parsed.is_merged());
        }
    }
}
