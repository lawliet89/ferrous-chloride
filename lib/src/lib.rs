#[macro_use]
mod macros;
mod errors;
mod utils;

mod constants;
pub mod iter;
pub mod literals;
pub mod value;

pub use constants::*;
pub use errors::Error;
pub use value::Value;

use std::collections::HashMap;

/// Either a single value, or many values
///
/// This is a utility type to make some implementation easier.
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

    pub fn iter(&self) -> iter::OneOrManyIterator<T> {
        match self {
            OneOrMany::One(value) => iter::OneOrManyIterator::One(std::iter::once(value)),
            OneOrMany::Many(vec) => iter::OneOrManyIterator::Many(vec.iter()),
        }
    }

    pub fn into_iter(self) -> iter::OneOrManyIntoIterator<T> {
        match self {
            OneOrMany::One(value) => iter::OneOrManyIntoIterator::One(std::iter::once(value)),
            OneOrMany::Many(vec) => iter::OneOrManyIntoIterator::Many(vec.into_iter()),
        }
    }
}

/// A set of `(Key, Value)` pairs which can exist in a merged or unmerged variant
///
/// A merged variant can only have unique keys, where the unmerged variant may have duplicate keys
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

    pub fn iter(&self) -> iter::KeyValuePairsIterator<K, V> {
        match self {
            KeyValuePairs::Merged(hashmap) => iter::KeyValuePairsIterator::Merged(hashmap.iter()),
            KeyValuePairs::Unmerged(vec) => iter::KeyValuePairsIterator::Unmerged(vec.iter()),
        }
    }

    pub fn into_iter(self) -> iter::KeyValuePairsIntoIterator<K, V> {
        match self {
            KeyValuePairs::Merged(hashmap) => {
                iter::KeyValuePairsIntoIterator::Merged(hashmap.into_iter())
            }
            KeyValuePairs::Unmerged(vec) => iter::KeyValuePairsIntoIterator::Unmerged(vec.into_iter()),
        }
    }
}
