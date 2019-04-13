#[macro_use]
mod macros;
mod errors;

pub mod constants;
pub mod iter;
#[macro_use]
pub mod utils;
#[macro_use]
pub mod parser;
pub mod value;

#[cfg(feature = "serde")]
pub mod serde;

#[cfg(feature = "serde")]
#[doc(inline)]
pub use crate::serde::from_str;
#[doc(inline)]
pub use errors::Error;
#[doc(inline)]
pub use parser::{parse_reader, parse_slice, parse_str};
#[doc(inline)]
pub use value::Value;

pub use nom;

use std::borrow::Cow;
use std::collections::HashMap as StdHashMap;
use std::hash::{BuildHasher, Hash};

pub(crate) type HashMap<K, V> = StdHashMap<K, V, hashbrown::hash_map::DefaultHashBuilder>;

/// Has scalar length
pub trait ScalarLength {
    /// Recursively count the number of scalars
    fn len_scalar(&self) -> usize;

    fn is_empty_scalar(&self) -> bool {
        self.len_scalar() == 0
    }
}

/// Type is mergeable
pub trait Mergeable {
    /// Recursively checks that self is merged
    ///
    /// This method should return true if all values recursively are merged.
    ///
    /// Note that this method might not be the complement of `is_unmerged`.
    fn is_merged(&self) -> bool;

    /// Recursively checks that self is unmerged
    ///
    /// This method should return true if all values recursively are unmerged.
    ///
    /// Note that this method might not be the complement of `is_merged`.
    fn is_unmerged(&self) -> bool {
        !self.is_merged()
    }
}

/// Type which has borrowed content which is able to be turned into an Owned version
///
/// In other words, this type should have some lifetime `'a` that can be turned into `'static`.
pub trait AsOwned {
    type Output: 'static;

    /// Returns a variant of `Self` where nothing is borrowed.
    fn as_owned(&self) -> Self::Output;
}

/// Either a single value, or many values
///
/// This is a utility type to make some implementation easier.
#[derive(Debug, PartialEq, Clone)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

/// Merge behaviour when parsing HCL Documents
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum MergeBehaviour {
    /// Error on duplicate identifiers in a map or duplicate labels between block with the same
    /// identifier
    Error,
    /// Take the first value seen on duplicate identifiers in a map or duplicate labels
    /// between block with the same identifier
    ///
    /// __Unimplemented__
    TakeFirst,
    /// Take the last value seen on duplicate identifiers in a map or duplicate labels
    /// between block with the same identifier
    ///
    /// __Unimplemented__
    TakeLast,
}

impl<T> OneOrMany<T> {
    pub fn len(&self) -> usize {
        match self {
            OneOrMany::One(_) => 1,
            OneOrMany::Many(vec) => vec.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_one(&self) -> bool {
        if let OneOrMany::One(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_many(&self) -> bool {
        !self.is_one()
    }

    pub fn iter(&self) -> iter::OneOrManyIterator<T> {
        match self {
            OneOrMany::One(value) => iter::OneOrManyIterator::One(std::iter::once(value)),
            OneOrMany::Many(vec) => iter::OneOrManyIterator::Many(vec.iter()),
        }
    }

    pub fn unwrap_one(self) -> T {
        if let OneOrMany::One(one) = self {
            one
        } else {
            panic!("unwrapping a many")
        }
    }

    pub fn unwrap_many(self) -> Vec<T> {
        if let OneOrMany::Many(many) = self {
            many
        } else {
            panic!("unwrapping a one")
        }
    }
}

/// A set of `(Key, Value)` pairs which can exist in a merged or unmerged variant
///
/// A merged variant can only have unique keys, where the unmerged variant may have duplicate keys
#[derive(Debug, PartialEq, Clone, Eq)]
pub enum KeyValuePairs<K, V>
where
    K: Hash + Eq,
{
    Merged(HashMap<K, V>),
    Unmerged(Vec<(K, V)>),
}

impl<K, V> KeyValuePairs<K, V>
where
    K: Hash + Eq,
{
    pub fn len(&self) -> usize {
        match self {
            KeyValuePairs::Merged(hashmap) => hashmap.len(),
            KeyValuePairs::Unmerged(vec) => vec.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn unwrap_merged(self) -> HashMap<K, V> {
        match self {
            KeyValuePairs::Merged(hashmap) => hashmap,
            KeyValuePairs::Unmerged(_) => panic!("Unwrapping an unmerged KeyValuePairs"),
        }
    }

    pub fn unwrap_unmerged(self) -> Vec<(K, V)> {
        match self {
            KeyValuePairs::Merged(_) => panic!("Unwrapping an unmerged KeyValuePairs"),
            KeyValuePairs::Unmerged(vec) => vec,
        }
    }

    pub fn iter(&self) -> iter::KeyValuePairsIterator<K, V> {
        match self {
            KeyValuePairs::Merged(hashmap) => iter::KeyValuePairsIterator::Merged(hashmap.iter()),
            KeyValuePairs::Unmerged(vec) => iter::KeyValuePairsIterator::Unmerged(vec.iter()),
        }
    }

    pub fn keys(&self) -> iter::KeyIterator<K, V> {
        match self {
            KeyValuePairs::Merged(hashmap) => iter::KeyIterator::Merged(hashmap.keys()),
            KeyValuePairs::Unmerged(vec) => {
                iter::KeyIterator::Unmerged(Box::new(vec.iter().map(|(k, _)| k)))
            }
        }
    }

    pub fn values(&self) -> iter::ValueIterator<K, V> {
        match self {
            KeyValuePairs::Merged(hashmap) => iter::ValueIterator::Merged(hashmap.values()),
            KeyValuePairs::Unmerged(vec) => {
                iter::ValueIterator::Unmerged(Box::new(vec.iter().map(|(_, v)| v)))
            }
        }
    }

    /// Get a single value with the specified key.
    ///
    /// # Warning
    /// If the variant is unmerged, this operation will __only__ return the first matching key it
    /// sees. A `Vec`'s order might not be stable.
    pub fn get_single<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Eq + Hash,
    {
        match self {
            KeyValuePairs::Merged(hashmap) => hashmap.get(key),
            KeyValuePairs::Unmerged(vec) => {
                vec.iter().find(|(k, _)| key.eq(k.borrow())).map(|(_, v)| v)
            }
        }
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<OneOrMany<&V>>
    where
        K: std::borrow::Borrow<Q>,
        Q: Eq + Hash,
    {
        match self {
            KeyValuePairs::Merged(hashmap) => hashmap.get(key).map(OneOrMany::One),
            KeyValuePairs::Unmerged(vec) => {
                let values: Vec<_> = vec
                    .iter()
                    .filter(|(k, _)| key.eq(k.borrow()))
                    .map(|(_, v)| v)
                    .collect();

                if values.is_empty() {
                    None
                } else {
                    Some(OneOrMany::Many(values))
                }
            }
        }
    }
}

impl<K, V> ScalarLength for KeyValuePairs<K, V>
where
    K: Hash + Eq,
    V: ScalarLength,
{
    fn len_scalar(&self) -> usize {
        match self {
            KeyValuePairs::Unmerged(vector) => {
                vector.iter().fold(0, |acc, (_, v)| acc + v.len_scalar())
            }
            KeyValuePairs::Merged(hashmap) => {
                hashmap.iter().fold(0, |acc, (_, v)| acc + v.len_scalar())
            }
        }
    }
}

impl<T> ScalarLength for &T
where
    T: ScalarLength,
{
    fn len_scalar(&self) -> usize {
        T::len_scalar(self)
    }
}

impl<T> ScalarLength for Vec<T>
where
    T: ScalarLength,
{
    fn len_scalar(&self) -> usize {
        self.iter().fold(0, |acc, v| acc + v.len_scalar())
    }
}

// Can't do. Orphan rules: https://www.reddit.com/r/rust/comments/b56p8i/_/ejc1syk/

// impl<T, I> ScalarLength for T
// where
//     T: IntoIterator<Item = I>,
//     I: ScalarLength,
// {
//     fn len_scalar(&self) -> usize {
//         self.into_iter().fold(0, |acc, v| acc + v.len_scalar())
//     }
// }

impl<K, V, S> ScalarLength for StdHashMap<K, V, S>
where
    K: Eq + Hash,
    V: ScalarLength,
    S: BuildHasher,
{
    fn len_scalar(&self) -> usize {
        self.iter().fold(0, |acc, (_, v)| acc + v.len_scalar())
    }
}

macro_rules! array_impls {
    ($($N:expr)+) => {
        $(
            impl<T> ScalarLength for [T; $N]
                where T: ScalarLength
            {
                fn len_scalar(&self) -> usize {
                    self.iter().fold(0, |acc, v| acc + v.len_scalar())
                }
            }
        )+
    }
}

array_impls! {
     0  1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32
}

impl<T> Mergeable for OneOrMany<T>
where
    T: Mergeable,
{
    fn is_merged(&self) -> bool {
        match self {
            OneOrMany::One(inner) => inner.is_merged(),
            OneOrMany::Many(vector) => vector.iter().all(T::is_merged),
        }
    }

    fn is_unmerged(&self) -> bool {
        match self {
            OneOrMany::One(inner) => inner.is_unmerged(),
            OneOrMany::Many(vector) => vector.iter().all(T::is_unmerged),
        }
    }
}

impl<K, V> Mergeable for KeyValuePairs<K, V>
where
    K: Hash + Eq,
    V: Mergeable,
{
    fn is_merged(&self) -> bool {
        match self {
            KeyValuePairs::Merged(hashmap) => hashmap.is_merged(),
            KeyValuePairs::Unmerged(_) => false,
        }
    }

    fn is_unmerged(&self) -> bool {
        match self {
            KeyValuePairs::Merged(_) => false,
            KeyValuePairs::Unmerged(vec) => vec.is_unmerged(),
        }
    }
}

impl<T> Mergeable for Vec<T>
where
    T: Mergeable,
{
    fn is_merged(&self) -> bool {
        self.iter().all(T::is_merged)
    }

    fn is_unmerged(&self) -> bool {
        self.iter().all(T::is_unmerged)
    }
}

impl<K, V, S> Mergeable for StdHashMap<K, V, S>
where
    K: Hash + Eq,
    V: Mergeable,
    S: BuildHasher,
{
    fn is_merged(&self) -> bool {
        self.iter().all(|(_, v)| v.is_merged())
    }

    fn is_unmerged(&self) -> bool {
        self.iter().all(|(_, v)| v.is_unmerged())
    }
}

impl<T1, T2> Mergeable for (T1, T2)
where
    T2: Mergeable,
{
    fn is_merged(&self) -> bool {
        self.1.is_merged()
    }

    fn is_unmerged(&self) -> bool {
        self.1.is_unmerged()
    }
}

impl<T> Mergeable for &T
where
    T: Mergeable,
{
    fn is_merged(&self) -> bool {
        T::is_merged(self)
    }

    fn is_unmerged(&self) -> bool {
        T::is_unmerged(self)
    }
}

impl<K, V, KO, VO> AsOwned for KeyValuePairs<K, V>
where
    K: Hash + Eq + AsOwned<Output = KO>,
    V: AsOwned<Output = VO>,
    KO: Hash + Eq + 'static,
    VO: 'static,
{
    type Output = KeyValuePairs<KO, VO>;

    fn as_owned(&self) -> Self::Output {
        match self {
            KeyValuePairs::Merged(hashmap) => KeyValuePairs::Merged(hashmap.as_owned()),
            KeyValuePairs::Unmerged(vec) => KeyValuePairs::Unmerged(vec.as_owned()),
        }
    }
}

impl<T, O> AsOwned for &T
where
    T: AsOwned<Output = O>,
    O: 'static,
{
    type Output = O;

    fn as_owned(&self) -> Self::Output {
        T::as_owned(self)
    }
}

impl<K, V, KO, VO> AsOwned for (K, V)
where
    K: AsOwned<Output = KO>,
    V: AsOwned<Output = VO>,
    KO: 'static,
    VO: 'static,
{
    type Output = (KO, VO);

    fn as_owned(&self) -> Self::Output {
        (self.0.as_owned(), self.1.as_owned())
    }
}

impl<T, O> AsOwned for Vec<T>
where
    T: AsOwned<Output = O>,
    O: 'static,
{
    type Output = Vec<O>;

    fn as_owned(&self) -> Self::Output {
        self.iter().map(AsOwned::as_owned).collect()
    }
}

impl<K, V, S, KO, VO> AsOwned for StdHashMap<K, V, S>
where
    K: Hash + Eq + AsOwned<Output = KO>,
    V: AsOwned<Output = VO>,
    S: BuildHasher + Default + 'static,
    KO: Hash + Eq + 'static,
    VO: 'static,
{
    type Output = StdHashMap<KO, VO, S>;

    fn as_owned(&self) -> Self::Output {
        self.iter().map(|pair| pair.as_owned()).collect()
    }
}

impl AsOwned for String {
    type Output = String;
    fn as_owned(&self) -> Self::Output {
        self.clone()
    }
}

impl AsOwned for str {
    type Output = String;
    fn as_owned(&self) -> Self::Output {
        self.to_string()
    }
}

impl<'a, B> AsOwned for Cow<'a, B>
where
    B: ToOwned + AsOwned,
    <B as AsOwned>::Output: Clone,
{
    type Output = Cow<'static, <B as AsOwned>::Output>;

    fn as_owned(&self) -> Self::Output {
        use std::ops::Deref;
        Cow::Owned(self.deref().as_owned())
    }
}

impl<T> AsOwned for Option<T>
where
    T: AsOwned,
    <T as AsOwned>::Output: 'static,
{
    type Output = Option<<T as AsOwned>::Output>;

    fn as_owned(&self) -> Self::Output {
        match self {
            None => None,
            Some(t) => Some(t.as_owned()),
        }
    }
}

impl Default for MergeBehaviour {
    fn default() -> Self {
        MergeBehaviour::Error
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    pub static ALL: &[&str] = &[
        BLOCK,
        LIST,
        NO_NEWLINE_EOF,
        SCALAR,
        SIMPLE_BLOCK,
        SINGLE,
        STRINGS,
    ];

    pub static BLOCK: &str = include_str!("../fixtures/block.hcl");
    pub static LIST: &str = include_str!("../fixtures/list.hcl");
    pub static NO_NEWLINE_EOF: &str = include_str!("../fixtures/no_newline_terminating.hcl");
    pub static SCALAR: &str = include_str!("../fixtures/scalar.hcl");
    pub static SIMPLE_BLOCK: &str = include_str!("../fixtures/simple_block.hcl");
    pub static SINGLE: &str = include_str!("../fixtures/single.hcl");
    pub static STRINGS: &str = include_str!("../fixtures/strings.hcl");
}
