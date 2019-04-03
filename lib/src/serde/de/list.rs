use serde::de::{DeserializeSeed, SeqAccess};

use super::Compat;
use crate::value;

pub struct ListAccess<'a> {
    // List is reversed!
    list: value::List<'a>,
}

impl<'a> ListAccess<'a> {
    pub(crate) fn new(mut list: value::List<'a>) -> Self {
        list.reverse();
        Self { list }
    }
}

impl<'de, 'a> SeqAccess<'de> for ListAccess<'a> {
    type Error = Compat;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        // Check if there are no more elements.
        if self.list.is_empty() {
            return Ok(None);
        }

        // Deserialize an array element.
        let item = self.list.pop().expect("to not be empty");
        seed.deserialize(item).map(Some)
    }
}
