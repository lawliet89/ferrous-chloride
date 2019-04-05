//! Expressions
//!
//! [Reference](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#expressions)

use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use nom::types::CompleteStr;
use nom::{alt_complete, call, named, IResult};

use super::literals;
use super::number::{number, Number};
use super::tuple::tuple;
use super::{list, map_expression};
use crate::constants::*;
use crate::value::Value;
use crate::Error;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expression<'a> {
    pub expression: ExpressionType<'a>,
    pub tokens: Cow<'a, str>,
}

impl<'a> Expression<'a> {
    pub fn merge(self) -> Result<Self, Error> {
        Ok(Self {
            expression: self.expression.merge()?,
            tokens: self.tokens,
        })
    }
}

impl<'a> Hash for Expression<'a> {
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

impl<'a> ExpressionType<'a> {
    pub fn merge(self) -> Result<Self, Error> {
        match self {
            no_op @ ExpressionType::Null
            | no_op @ ExpressionType::Number(_)
            | no_op @ ExpressionType::Boolean(_)
            | no_op @ ExpressionType::String(_) => Ok(no_op),
            // Value::List(list) => Ok(Value::List(
            //     list.into_iter()
            //         .map(Value::merge)
            //         .collect::<Result<_, _>>()?,
            // )),
            // Value::Object(maps) => Ok(Value::Object(
            //     maps.into_iter()
            //         .map(MapValues::merge)
            //         .collect::<Result<_, _>>()?,
            // )),
            // Value::Block(block) => {
            //     let unmerged: Block = block
            //         .into_iter()
            //         .map(|(key, value)| Ok((key, value.merge()?)))
            //         .collect::<Result<_, Error>>()?;
            //     let merged = Block::new_merged(unmerged)?;
            //     Ok(Value::Block(merged))
            // }
        }
    }

    pub fn variant_name(&self) -> &'static str {
        match self {
            ExpressionType::Null => NULL,
            ExpressionType::Number(_) => NUMBER,
            ExpressionType::Boolean(_) => BOOLEAN,
            ExpressionType::String(_) => STRING,
            // ExpressionType::List(_) => LIST,
            // ExpressionType::Object(_) => OBJECT,
            // ExpressionType::Block(_) => BLOCK,
        }
    }
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

impl<'a> From<&'a str> for ExpressionType<'a> {
    fn from(s: &'a str) -> Self {
        ExpressionType::String(s.to_string())
    }
}

impl<'a> FromStr for ExpressionType<'a> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ExpressionType::String(s.to_string()))
    }
}

// named!(
//     pub expression(CompleteStr) -> Expression,
//     alt_complete!(
//         // LiteralValue -> "null"
//         call!(literals::null) => { |_| Value::Null }
//         // LiteralValue -> NumericLit
//         | call!(literals::number) => { |v| From::from(v) }
//         // LiteralValue -> "true" | "false"
//         | call!(literals::boolean) => { |v| Value::Boolean(v) }
//         // TemplateExpr
//         // https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#template-expressions
//         | literals::string => { |v| Value::String(v) }
//         // CollectionValue -> tuple
//         | tuple => { |v| Value::List(v) }
//         // CollectionValue -> object
//         | map_expression => { |m| Value::Object(vec![m]) }
//         // VariableExpr
//         // FunctionCall
//         // ForExpr
//         // ExprTerm Index
//         // ExprTerm GetAttr
//         // ExprTerm Splat
//         // "(" Expression ")"
//     )
// );

named!(
    pub expression_type(CompleteStr) -> ExpressionType,
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

pub fn expression(input: CompleteStr) -> IResult<CompleteStr, Expression, u32> {
    expression_type(input).map(|(input, output)| {
        (
            input,
            Expression {
                expression: output,
                tokens: Cow::Borrowed(input.0),
            },
        )
    })
}

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
