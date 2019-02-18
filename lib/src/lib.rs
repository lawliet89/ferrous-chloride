mod errors;
pub mod literals;
mod utils;

pub use errors::Error;
use nom::{alt, named, ws};

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
/// Value in HCL
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(Vec<Value>),
    Stanza(Stanza),
    Object(HashMap<String, Value>),
}

// https://github.com/Geal/nom/blob/master/tests/json.rs
#[derive(Debug, PartialEq)]
pub struct Stanza {
    pub keys: Vec<String>,
    pub values: HashMap<String, Value>,
}

// Parse values of the form "key" = "..." | ["..."] | {...}
