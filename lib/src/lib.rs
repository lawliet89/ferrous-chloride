mod errors;
pub mod tokens;
pub mod utils;

pub use errors::Error;

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
/// Value in HCL
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    r#String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

// https://github.com/Geal/nom/blob/master/tests/json.rs
