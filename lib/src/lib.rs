#[macro_use]
mod macros;
mod errors;
mod utils;

mod constants;
pub mod literals;
pub mod value;

pub use constants::*;
pub use errors::Error;
pub use value::Value;

use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
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

    pub fn iter(&self) -> OneOrManyIterator<T> {
        match self {
            OneOrMany::One(value) => OneOrManyIterator::One(std::iter::once(value)),
            OneOrMany::Many(vec) => OneOrManyIterator::Many(vec.iter()),
        }
    }
}

impl<'a, T> std::iter::IntoIterator for &'a OneOrMany<T> {
    type Item = &'a T;
    type IntoIter = OneOrManyIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub enum OneOrManyIterator<'a, T> {
    One(std::iter::Once<&'a T>),
    Many(std::slice::Iter<'a, T>),
}

impl<'a, T> Iterator for OneOrManyIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OneOrManyIterator::One(iter) => iter.next(),
            OneOrManyIterator::Many(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            OneOrManyIterator::One(iter) => iter.size_hint(),
            OneOrManyIterator::Many(iter) => iter.size_hint(),
        }
    }
}

impl<'a, T> ExactSizeIterator for OneOrManyIterator<'a, T> {}

#[derive(Debug, PartialEq, Clone)]
pub enum KeyValuePairs<K, V>
where
    K: std::hash::Hash + Eq,
{
    Merged(HashMap<K, V>),
    Unmerged(Vec<(K, V)>),
}

impl<K, V> KeyValuePairs<K, V>
where
    K: std::hash::Hash + Eq,
{
    pub fn is_merged(&self) -> bool {
        if let KeyValuePairs::Merged(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_unmerged(&self) -> bool {
        !self.is_merged()
    }

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

    pub fn iter(&self) -> KeyValuePairsIterator<K, V> {
        match self {
            KeyValuePairs::Merged(hashmap) => KeyValuePairsIterator::Merged(hashmap.iter()),
            KeyValuePairs::Unmerged(vec) => KeyValuePairsIterator::Unmerged(vec.iter()),
        }
    }

    pub fn into_iter(self) -> KeyValuePairsIntoIterator<K, V> {
        match self {
            KeyValuePairs::Merged(hashmap) => {
                KeyValuePairsIntoIterator::Merged(hashmap.into_iter())
            }
            KeyValuePairs::Unmerged(vec) => KeyValuePairsIntoIterator::Unmerged(vec.into_iter()),
        }
    }
}

impl<K, V> std::iter::Extend<(K, V)> for KeyValuePairs<K, V>
where
    K: std::hash::Hash + Eq,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (K, V)>,
    {
        match self {
            KeyValuePairs::Unmerged(vec) => vec.extend(iter),
            KeyValuePairs::Merged(hashmap) => hashmap.extend(iter),
        }
    }
}

impl<'a, K: 'a, V: 'a> std::iter::IntoIterator for &'a KeyValuePairs<K, V>
where
    K: std::hash::Hash + Eq,
{
    type Item = (&'a K, &'a V);
    type IntoIter = KeyValuePairsIterator<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K, V> std::iter::IntoIterator for KeyValuePairs<K, V>
where
    K: std::hash::Hash + Eq,
{
    type Item = (K, V);
    type IntoIter = KeyValuePairsIntoIterator<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a, K: 'a, V: 'a, Q: ?Sized> std::ops::Index<&'a Q> for KeyValuePairs<K, V>
where
    K: std::hash::Hash + Eq + std::borrow::Borrow<Q>,
    Q: Eq + std::hash::Hash,
{
    type Output = V;

    /// # Warning:
    /// If the variant is unmerged, this operation will __only__ return the first matching key it
    /// sees. A `Vec`'s order might not be stable.
    fn index(&self, key: &Q) -> &V {
        match self {
            KeyValuePairs::Merged(hashmap) => hashmap.index(key),
            KeyValuePairs::Unmerged(vec) => {
                let (_, v) = vec
                    .iter()
                    .find(|(k, _)| key.eq(k.borrow()))
                    .expect("no entry found for key");
                v
            }
        }
    }
}

pub enum KeyValuePairsIterator<'a, K: 'a, V: 'a> {
    Merged(std::collections::hash_map::Iter<'a, K, V>),
    Unmerged(std::slice::Iter<'a, (K, V)>),
}

impl<'a, K: 'a, V: 'a> Iterator for KeyValuePairsIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            KeyValuePairsIterator::Merged(iter) => iter.next(),
            KeyValuePairsIterator::Unmerged(iter) => match iter.next() {
                None => None,
                Some((k, v)) => Some((k, v)),
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            KeyValuePairsIterator::Merged(iter) => iter.size_hint(),
            KeyValuePairsIterator::Unmerged(iter) => iter.size_hint(),
        }
    }
}

impl<'a, K: 'a, V: 'a> ExactSizeIterator for KeyValuePairsIterator<'a, K, V> {}

pub enum KeyValuePairsIntoIterator<K, V> {
    Merged(std::collections::hash_map::IntoIter<K, V>),
    Unmerged(std::vec::IntoIter<(K, V)>),
}

impl<K, V> Iterator for KeyValuePairsIntoIterator<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            KeyValuePairsIntoIterator::Merged(iter) => iter.next(),
            KeyValuePairsIntoIterator::Unmerged(iter) => match iter.next() {
                None => None,
                Some((k, v)) => Some((k, v)),
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            KeyValuePairsIntoIterator::Merged(iter) => iter.size_hint(),
            KeyValuePairsIntoIterator::Unmerged(iter) => iter.size_hint(),
        }
    }
}

impl<K, V> ExactSizeIterator for KeyValuePairsIntoIterator<K, V> {}
