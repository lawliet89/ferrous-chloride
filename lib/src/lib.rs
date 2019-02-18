mod errors;
pub mod literals;
mod utils;

pub use errors::Error;
use nom::{alt_complete, call, char, named, separated_pair, ws};

use std::collections::HashMap;

pub type Map<'a> = HashMap<literals::Key<'a>, Value<'a>>;

#[derive(Debug, PartialEq)]
/// Value in HCL
pub enum Value<'a> {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(Vec<Value<'a>>),
    Stanza(Stanza<'a>),
    Map(Map<'a>),
}

// https://github.com/Geal/nom/blob/master/tests/json.rs
#[derive(Debug, PartialEq)]
pub struct Stanza<'a> {
    pub keys: Vec<String>,
    pub values: Map<'a>,
}

/// Parse values of the form "key" = "..." | ["..."] | {...}
named!(
    pub key_value(&str) -> (literals::Key, Value),
    ws!(
        separated_pair!(
            literals::key,
            char!('='),
            alt_complete!(
                call!(crate::utils::wrap_str(literals::integer)) => { |v| Value::Integer(v) }
                | call!(crate::utils::wrap_str(literals::float)) => { |v| Value::Float(v) }
            )
        )
    )
);


#[cfg(test)]
mod tests {
    use super::*;

    use utils::ResultUtilsString;

    #[test]
    fn key_value_pairs_are_parsed_successfully() {
        let test_cases = [
            (r#"test = 123"#, ("test", Value::Integer(123))),
        ];

        for (input, (expected_key, expected_value)) in test_cases.into_iter() {
            println!("Testing {}", input);
            let (actual_key, actual_value) = key_value(input).unwrap_output();
            assert_eq!(actual_key.unwrap(), *expected_key);
            assert_eq!(actual_value, *expected_value);
        }
    }
}
