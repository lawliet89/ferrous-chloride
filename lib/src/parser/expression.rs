//! Expressions
//!
//! [Reference](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#expressions)

use std::borrow::Cow;

use nom::types::CompleteStr;
use nom::{alt_complete, call, named};

use super::literals;
use super::tuple::tuple;
use super::{list, map_expression};
use super::number::Number;
use crate::value::Value;

// FIXME: For now
/// An Expression
///
/// ```enbf
/// Expression = (
///     ExprTerm |
///     Operation |  # Not supported
///     Conditional # Not supported
/// );
///
/// ExprTerm = (
///     LiteralValue |
///     CollectionValue |
///     TemplateExpr |
///     VariableExpr |
///     FunctionCall |
///     ForExpr |
///     ExprTerm Index |
///     ExprTerm GetAttr |
///     ExprTerm Splat |
///     "(" Expression ")"
/// );
///
/// LiteralValue = (
///   NumericLit |
///   "true" |
///   "false" |
///   "null"
/// );
/// ```
///
/// - Numeric literals represent values of type number.
pub type Expression<'a> = Value<'a>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionWip<'a> {
    expression: ExpressionType<'a>,
    tokens: Cow<'a, str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionType<'a> {
    /// LiteralValue -> "null"
    Null,
    Number(Number<'a>),
    Boolean(bool),
    String(String),
    // Tuple(List),
}

named!(
    pub expression(CompleteStr) -> Expression,
    alt_complete!(
        // LiteralValue -> "null"
        call!(literals::null) => { |_| Value::Null }
        // LiteralValue -> NumericLit
        | call!(literals::number) => { |v| From::from(v) }
        // LiteralValue -> "true" | "false"
        | call!(literals::boolean) => { |v| Value::Boolean(v) }
        // TemplateExpr
        // https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#template-expressions
        | literals::string => { |v| Value::String(v) }
        // CollectionValue -> tuple
        | tuple => { |v| Value::List(v) }
        // CollectionValue -> object
        | map_expression => { |m| Value::Object(vec![m]) }
        // VariableExpr
        // FunctionCall
        // ForExpr
        // ExprTerm Index
        // ExprTerm GetAttr
        // ExprTerm Splat
        // "(" Expression ")"
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parser::literals::Key;

    #[test]
    fn expressions_are_parsed_successfully() {
        let test_cases = [
            ("null", Value::Null, ""),
            (r#"123"#, Value::Integer(123), ""),
            ("123", Value::Integer(123), ""),
            ("123", Value::Integer(123), ""),
            ("true", Value::Boolean(true), ""),
            ("123.456", Value::Float(123.456), ""),
            ("123", Value::Integer(123), ""),
            (r#""foobar""#, Value::String("foobar".to_string()), ""),
            (
                r#"<<EOF
new
line
EOF
"#,
                Value::String("new\nline".to_string()),
                "\n",
            ),
            (
                r#"[true, false, 123, -123.456, "foobar"]"#,
                Value::new_list(vec![
                    Value::from(true),
                    Value::from(false),
                    Value::from(123),
                    Value::from(-123.456),
                    Value::from("foobar"),
                ]),
                "",
            ),
            (
                r#"{
        test = 123
}"#,
                Value::new_map(vec![vec![(Key::new_identifier("test"), Value::from(123))]]),
                "",
            ),
        ];

        for (input, expected_value, expected_remaining) in test_cases.iter() {
            println!("Testing {}", input);
            let (remaining, actual_value) = expression(CompleteStr(input)).unwrap();
            assert_eq!(&remaining.0, expected_remaining);
            assert_eq!(actual_value, *expected_value);
        }
    }

}
