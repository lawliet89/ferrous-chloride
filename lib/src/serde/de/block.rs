use std::vec::IntoIter;

use serde::de::{self, Visitor};
use serde::forward_to_deserialize_any;

use crate::parser::block::BlockBody;
use crate::parser::body::Body;
use crate::serde::de::Compat;

impl<'de> de::Deserializer<'de> for BlockBody<'de> {
    type Error = Compat;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Make a "best guess" of how to deserialize the block
        match self {
            BlockBody::Body(mut bodies) => {
                if bodies.len() == 1 {
                    // Deseriaize the single block body as a map/struct
                    visitor.visit_map(crate::serde::de::body::MapAccess::new(bodies.remove(0)))
                } else {
                    visitor.visit_seq(SeqAccess {
                        iterator: bodies.into_iter(),
                    })
                }
            }
            BlockBody::Labels { mut empty, labels } => {
                if empty.len() == 1 && labels.is_empty() {
                    // This should be impossible but we handle it anyway
                    return visitor
                        .visit_map(crate::serde::de::body::MapAccess::new(empty.remove(0)));
                }
                unimplemented!("not yet")
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.len_blocks() == 0 {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            BlockBody::Body(bodies) => visitor.visit_seq(SeqAccess {
                iterator: bodies.into_iter(),
            }),
            BlockBody::Labels { empty, labels } => unimplemented!("not yet"),
        }
    }
    // Tuple
    // map
    // struct
    // enum

    // Many of these types cannot be deserialized from BlockBody
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

/// Deserialize a sequence of blocks with no labels
pub struct SeqAccess<'de> {
    pub(crate) iterator: IntoIter<Body<'de>>,
}

impl<'de> de::SeqAccess<'de> for SeqAccess<'de> {
    type Error = Compat;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.iterator.next() {
            None => Ok(None),
            Some(body) => seed
                .deserialize(crate::serde::de::body::Deserializer { body })
                .map(Some),
        }
    }
}
