use std::collections::{hash_map, HashMap};
use std::vec;

use serde::de::{self, Deserializer, IntoDeserializer, Visitor};
use serde::forward_to_deserialize_any;

use crate::parser::block::{BlockBody, BlockLabel};
use crate::parser::body::Body;
use crate::serde::de::body::Deserializer as BodyDeserializer;
use crate::serde::de::Compat;

fn deserialize_body_seq<'de, V>(bodies: Vec<Body<'de>>, visitor: V) -> Result<V::Value, Compat>
where
    V: Visitor<'de>,
{
    visitor.visit_seq(
        bodies
            .into_iter()
            .map(BodyDeserializer::new)
            .collect::<Vec<_>>()
            .into_deserializer(),
    )
}

fn deserialize_map<'de, V>(body: Body<'de>, visitor: V) -> Result<V::Value, Compat>
where
    V: Visitor<'de>,
{
    BodyDeserializer::new(body).deserialize_map(visitor)
}

/// Possible states of `BlockBody`:
/// - Empty: Single Body => Deserialize Map/Struct
/// - Empty: Multiple Bodies => Seq
/// - Labels: Zero labels => Logic error! Treat like Empty variannt
/// - Labels: Zero empty => Single label: enum/struct with labels fields
///                      => Multiple labels: Seq of above
/// - Labels: Non-zero empty => Seq of structs with label fields
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
                    deserialize_map(bodies.remove(0), visitor)
                } else {
                    deserialize_body_seq(bodies, visitor)
                }
            }
            BlockBody::Labels { mut empty, labels } => {
                if labels.is_empty() {
                    // This should be impossible but we handle it anyway
                    return if empty.len() == 1 {
                        // Deseriaize the single block body as a map/struct
                        deserialize_map(empty.remove(0), visitor)
                    } else {
                        deserialize_body_seq(empty, visitor)
                    };
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
            BlockBody::Body(bodies) => deserialize_body_seq(bodies, visitor),
            BlockBody::Labels { mut empty, labels } => {
                if labels.is_empty() {
                    // This should be impossible but we handle it anyway
                    return if empty.len() == 1 {
                        // Deseriaize the single block body as a map/struct
                        deserialize_map(empty.remove(0), visitor)
                    } else {
                        deserialize_body_seq(empty, visitor)
                    };
                }
                unimplemented!("not yet")
            }
        }
    }

    // Tuple
    // map - mapaccess `"labels" = rest`
    // struct
    // enum
    // identifier = probably enum

    // Many of these types cannot be deserialized from BlockBody
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[derive(Debug)]
pub struct LabelsSeqAccess<'de> {
    empty: vec::IntoIter<Body<'de>>,
    labels: hash_map::IntoIter<BlockLabel<'de>, BlockBody<'de>>,
}

impl<'de> LabelsSeqAccess<'de> {
    pub fn new(empty: Vec<Body<'de>>, labels: HashMap<BlockLabel<'de>, BlockBody<'de>>) -> Self {
        Self {
            empty: empty.into_iter(),
            labels: labels.into_iter(),
        }
    }
}
