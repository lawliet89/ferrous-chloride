//! Serializer and Deserializer for HCL
//!
//! This module contains the necessary types and trait implementation to serialize and deserialize
//! HCL documents to and from Rust data structure.
//!
//! The sub-modules contain implementation details that you can usually disregard. To find out more
//! about _using_ them, head to [`serde` documentation](https://serde.rs/).
pub mod de;

#[doc(inline)]
pub use de::from_str;
