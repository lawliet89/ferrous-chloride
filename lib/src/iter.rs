//! Iterator Types and implementations for data structures
//! In general, you should not need to instantiate these types manually
//!
//! This module also containts the Iterator related trait implementations

use std::hash::Hash;

use crate::KeyValuePairs;

pub enum OneOrManyIterator<'a, T> {
    One(std::iter::Once<&'a T>),
    Many(std::slice::Iter<'a, T>),
}

impl<'a, T> std::iter::IntoIterator for &'a crate::OneOrMany<T> {
    type Item = &'a T;
    type IntoIter = OneOrManyIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
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

pub enum OneOrManyIntoIterator<T> {
    One(std::iter::Once<T>),
    Many(std::vec::IntoIter<T>),
}

impl<T> std::iter::IntoIterator for crate::OneOrMany<T> {
    type Item = T;
    type IntoIter = OneOrManyIntoIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<T> Iterator for OneOrManyIntoIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OneOrManyIntoIterator::One(iter) => iter.next(),
            OneOrManyIntoIterator::Many(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            OneOrManyIntoIterator::One(iter) => iter.size_hint(),
            OneOrManyIntoIterator::Many(iter) => iter.size_hint(),
        }
    }
}

impl<T> ExactSizeIterator for OneOrManyIntoIterator<T> {}

impl<K, V> std::iter::Extend<(K, V)> for KeyValuePairs<K, V>
where
    K: Hash + Eq,
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
    K: Hash + Eq,
{
    type Item = (&'a K, &'a V);
    type IntoIter = KeyValuePairsIterator<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K, V> std::iter::IntoIterator for KeyValuePairs<K, V>
where
    K: Hash + Eq,
{
    type Item = (K, V);
    type IntoIter = KeyValuePairsIntoIterator<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a, K: 'a, V: 'a, Q> std::ops::Index<&'a Q> for KeyValuePairs<K, V>
where
    K: Hash + Eq + std::borrow::Borrow<Q>,
    Q: Eq + Hash + ?Sized,
{
    type Output = V;

    /// # Warning
    /// If the variant is unmerged, this operation will __only__ return the first matching key it
    /// sees. A `Vec`'s order might not be stable.
    fn index(&self, key: &Q) -> &V {
        self.get_single(key).expect("no entry found for key")
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
