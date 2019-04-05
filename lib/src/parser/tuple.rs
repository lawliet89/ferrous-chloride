//! Tuple
//!
//! Part of `CollectionValue`
//!
//! ```enbf
//! tuple = "[" (
//!     (Expression ("," Expression)* ","?)?
//! ) "]";
//! ```
//!
//! [Reference](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#collection-values)
use nom::types::CompleteStr;
use nom::{char, named, opt, preceded, terminated};

use super::expression::{expression, Expression};

pub type Tuple<'a> = Vec<Expression<'a>>;

named!(
    tuple_begin(CompleteStr) -> char,
    char!('[')
);

named!(
    tuple_separator(CompleteStr) -> char,
    char!(',')
);

// From https://github.com/Geal/nom/issues/14#issuecomment-158788226
// whitespace! Must not be captured after `]`!

// TODO: Deal with for syntax ambiguity when implementing later
named!(
    pub tuple(CompleteStr) -> Tuple,
    preceded!(
        tuple_begin,
        terminated!(
            whitespace!(
                separated_list!(
                    tuple_separator,
                    expression
                )
            ),
            terminated!(
                whitespace!(opt!(tuple_separator)),
                char!(']')
            )
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::HashMap;

    use crate::fixtures;
    use crate::utils::{assert_list_eq, ResultUtilsString};
    use crate::value::{Block, Value};
    use crate::{Mergeable, ScalarLength};

    #[test]
    fn simple_tuples_are_parsed_successfully() {
        let test_cases = [
            (r#"[]"#, vec![]),
            (r#"[1,]"#, vec![Expression::from(1)]),
            (
                r#"[true, false, 123, -123.456, "foobar"]"#,
                vec![
                    Expression::from(true),
                    Expression::from(false),
                    Expression::from(123),
                    Expression::from(-123.456),
                    Expression::from("foobar"),
                ],
            ),
            (
                r#"[
                        true,
                        false,
                        123,
                        -123.456,
                        "testing",                        [
                            "inside voice!",
                            "lol"
                        ],
                    ]"#,
                vec![
                    Expression::from(true),
                    Expression::from(false),
                    Expression::from(123),
                    Expression::from(-123.456),
                    Expression::from("testing"),
                    Expression::new_tuple(vec![
                        Expression::from("inside voice!"),
                        Expression::from("lol"),
                    ]),
                ],
            ),
        ];

        for (input, expected_value) in test_cases.iter() {
            println!("Testing {}", input);
            let actual_value = tuple(CompleteStr(input)).unwrap_output();
            assert_eq!(actual_value, *expected_value);
        }
    }
}
