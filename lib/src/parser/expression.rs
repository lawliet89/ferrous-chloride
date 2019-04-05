//! Expressions
//!
//! [Reference](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#expressions)

use std::borrow::Cow;
use std::hash::{Hash, Hasher};

use nom::types::CompleteStr;
use nom::{alt_complete, call, named};

use super::literals;
use super::number::{number, Number};
use super::tuple::tuple;
use super::{list, map_expression};
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

impl<'a> Hash for ExpressionWip<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.tokens.as_bytes())
    }
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

macro_rules! impl_from_expr_type (
    ($variant: ident, $type: ty) => (
        impl<'a> From<$type> for ExpressionType<'a> {
            fn from(v: $type) -> Self {
                ExpressionType::$variant(From::from(v))
            }
        }
    )
);

impl_from_expr_type!(Number, Number<'a>);
impl_from_expr_type!(Number, u8);
impl_from_expr_type!(Number, u16);
impl_from_expr_type!(Number, u32);
impl_from_expr_type!(Number, u64);
impl_from_expr_type!(Number, u128);
impl_from_expr_type!(Number, i8);
impl_from_expr_type!(Number, i16);
impl_from_expr_type!(Number, i32);
impl_from_expr_type!(Number, i64);
impl_from_expr_type!(Number, i128);
impl_from_expr_type!(Number, f32);
impl_from_expr_type!(Number, f64);
impl_from_expr_type!(Boolean, bool);
impl_from_expr_type!(String, String);

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

named!(
    pub expression_wip(CompleteStr) -> ExpressionType,
    alt_complete!(
        // LiteralValue -> "null"
        call!(literals::null) => { |_| ExpressionType::Null }
        // LiteralValue -> NumericLit
        | call!(number) => { |v| From::from(v) }
        // LiteralValue -> "true" | "false"
        | call!(literals::boolean) => { |v| From::from(v) }
        // TemplateExpr
        // https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#template-expressions
        | literals::string => { |v| From::from(v) }
        // CollectionValue -> tuple
        // | tuple => { |v| Value::List(v) }
        // CollectionValue -> object
        // | map_expression => { |m| Value::Object(vec![m]) }
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
