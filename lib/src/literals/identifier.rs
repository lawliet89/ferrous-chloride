use nom::types::CompleteStr;
use nom::{call, do_parse, named_attr, verify};

/// Parse an identifier
named_attr!(#[allow(clippy::block_in_if_condition_stmt)], pub identifier(CompleteStr) -> &str,
    do_parse!(
        identifier: verify!(
            call!(crate::utils::while_predicate1, |c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.'),
            |s: CompleteStr| {
                let first = s.chars().nth(0);
                match first {
                    None => false,
                    Some(c) => c.is_alphabetic() || c == '_'
                }
            }
        )
        >> (identifier.0)
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::ResultUtilsString;

    #[test]
    fn identifiers_are_parsed_correctly() {
        let test_cases = [
            ("abcd123", "abcd123"),
            ("_abc", "_abc"),
            ("藏_①", "藏_①"),
        ];

        for (input, expected) in test_cases.iter() {
            println!("Testing {}", input);
            assert_eq!(identifier(CompleteStr(input)).unwrap_output(), *expected);
        }
    }

    #[test]
    fn incorrect_identifiers_are_not_accepted() {
        let test_cases = ["1abc", "①_is_some_number"];

        for input in test_cases.iter() {
            println!("Testing {}", input);
            assert!(identifier(CompleteStr(input)).is_err());
        }
    }
}
