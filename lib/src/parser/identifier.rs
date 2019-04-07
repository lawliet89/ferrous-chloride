//! Identifiers
//!
//! [Reference](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#identifiers)
//!
//! Identifiers name entities such as blocks, attributes and expression variables.
//! Identifiers are interpreted as per [UAX #31][uax31] Section 2. Specifically,
//! their syntax is defined in terms of the `ID_Start` and `ID_Continue`
//! character properties as follows:
//!
//! ```ebnf
//! Identifier = ID_Start (ID_Continue | '-')*;
//! ```
//!
//! The Unicode specification provides the normative requirements for identifier
//! parsing. Non-normatively, the spirit of this specification is that `ID_Start`
//! consists of Unicode letter and certain unambiguous punctuation tokens, while
//! `ID_Continue` augments that set with Unicode digits, combining marks, etc.
//!
//! The dash character `-` is additionally allowed in identifiers, even though
//! that is not part of the unicode `ID_Continue` definition. This is to allow
//! attribute names and block type names to contain dashes, although underscores
//! as word separators are considered the idiomatic usage.
//!
//! [uax31]: http://unicode.org/reports/tr31/ "Unicode Identifier and Pattern Syntax"

use nom::types::CompleteStr;
use nom::{call, do_parse, named_attr, verify};
use unic_ucd_ident::{is_id_continue, is_id_start};

// Parse an identifier
named_attr!(#[allow(clippy::block_in_if_condition_stmt)], pub identifier(CompleteStr) -> &str,
    do_parse!(
        identifier: verify!(
            call!(crate::utils::while_predicate1, |c| is_id_continue(c) || c == '-'),
            |s: CompleteStr| {
                let first = s.chars().nth(0);
                match first {
                    None => false,
                    // FIXME: ID_START doesn't allow underscores. But I think HCL does?
                    Some(c) => is_id_start(c) || c == '_'
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
            ("abcd_123", "abcd_123"),
            ("abcd-123", "abcd-123"),
            ("_abc", "_abc"),
            ("゛藏_a", "゛藏_a"),
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
