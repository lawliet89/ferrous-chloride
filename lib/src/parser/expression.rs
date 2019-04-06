//! Expressions
//!
//! [Reference](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#expressions)

use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::str::FromStr;

use nom::types::CompleteStr;
use nom::{alt_complete, call, do_parse, named, tag, IResult};

use super::literals;
use super::number::{number, Number};
use super::tuple::{tuple, Tuple};
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
pub enum Expression<'a> {
    /// LiteralValue -> "null"
    Null,
    Number(Number<'a>),
    Boolean(bool),
    String(String),
    Tuple(Tuple<'a>),
}

impl<'a> Expression<'a> {
    pub fn new_tuple<T>(iterator: T) -> Self
    where
        T: IntoIterator<Item = Expression<'a>>,
    {
        Expression::Tuple(iterator.into_iter().collect())
    }

    /// # [Reference](https://github.com/hashicorp/hcl2/blob/master/hcl/spec.md#schema-driven-processing)
    ///
    /// Within a schema, it is an error to request the same attribute name twice or to request a
    /// block type whose name is also an attribute name. While this can in principle be supported
    /// in some syntaxes, in other syntaxes the attribute and block namespaces are combined and so
    /// an an attribute cannot coexist with a block whose type name is identical to the attribute
    /// name.
    pub fn merge(self) -> Result<Self, Error> {
        match self {
            no_op @ Expression::Null
            | no_op @ Expression::Number(_)
            | no_op @ Expression::Boolean(_)
            | no_op @ Expression::String(_) => Ok(no_op),
            Expression::Tuple(tuple) => Ok(Expression::Tuple(
                tuple
                    .into_iter()
                    .map(Self::merge)
                    .collect::<Result<_, Error>>()?,
            )),
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
            Expression::Null => NULL,
            Expression::Number(_) => NUMBER,
            Expression::Boolean(_) => BOOLEAN,
            Expression::String(_) => STRING,
            Expression::Tuple(_) => TUPLE,
            // Expression::Object(_) => OBJECT,
            // Expression::Block(_) => BLOCK,
        }
    }
}

impl<'a> FromIterator<Expression<'a>> for Expression<'a> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Expression<'a>>,
    {
        Expression::new_tuple(iter)
    }
}

macro_rules! impl_from_expr_type (
    ($variant: ident, $type: ty) => (
        impl<'a> From<$type> for Expression<'a> {
            fn from(v: $type) -> Self {
                Expression::$variant(From::from(v))
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
impl_from_expr_type!(Tuple, Vec<Expression<'a>>);

impl<'a> From<&'a str> for Expression<'a> {
    fn from(s: &'a str) -> Self {
        Expression::String(s.to_string())
    }
}

impl<'a> FromStr for Expression<'a> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Expression::String(s.to_string()))
    }
}

// "(" Expression ")"
named!(
    pub bracket_expression(CompleteStr) -> Expression,
    do_parse!(
        whitespace!(tag!("("))
        >> expr: whitespace!(call!(expression))
        >> tag!(")")
        >> (expr)
    )
);

named!(
    pub expression(CompleteStr) -> Expression,
    alt_complete!(
        // LiteralValue -> "null"
        call!(literals::null) => { |_| Expression::Null }
        // LiteralValue -> NumericLit
        | call!(number) => { |v| From::from(v) }
        // LiteralValue -> "true" | "false"
        | call!(literals::boolean) => { |v| From::from(v) }
        // TemplateExpr
        // https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#template-expressions
        | literals::string => { |v| From::from(v) }
        // CollectionValue -> tuple
        | tuple => { |v| From::from(v) }
        // CollectionValue -> object
        // | map_expression => { |m| Value::Object(vec![m]) }
        // VariableExpr
        // FunctionCall
        // ForExpr
        // ExprTerm Index
        // ExprTerm GetAttr
        // ExprTerm Splat
        // "(" Expression ")"
        | call!(bracket_expression)
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parser::literals::Key;

    #[test]
    fn bracket_expression_parses_correctly() {
        let test_cases = [
            ("(null)", Expression::Null, ""),
            (r#"(123)"#, Expression::from(123), ""),
            ("((123))", Expression::from(123), ""),
            ("(((123)))", Expression::from(123), ""),
            ("(true)", Expression::Boolean(true), ""),
            ("123.456", Expression::from(123.456), ""),
            ("123", Expression::from(123), ""),
            (r#""foobar""#, Expression::String("foobar".to_string()), ""),
            (
                r#"
(
<<EOF
new
line
EOF
)
"#,
                Expression::String("new\nline".to_string()),
                "\n",
            ),
        ];

        for (input, expected_value, expected_remaining) in test_cases.iter() {
            println!("Testing {}", input);
            let (remaining, actual_value) = expression(CompleteStr(input)).unwrap();
            assert_eq!(&remaining.0, expected_remaining);
            assert_eq!(actual_value, *expected_value);
        }
    }

    #[test]
    fn expressions_are_parsed_successfully() {
        let test_cases = [
            ("null", Expression::Null, ""),
            (r#"123"#, Expression::from(123), ""),
            ("123", Expression::from(123), ""),
            ("123", Expression::from(123), ""),
            ("true", Expression::Boolean(true), ""),
            ("123.456", Expression::from(123.456), ""),
            ("123", Expression::from(123), ""),
            (r#""foobar""#, Expression::String("foobar".to_string()), ""),
            (
                r#"<<EOF
new
line
EOF
"#,
                Expression::String("new\nline".to_string()),
                "\n",
            ),
            (
                r#"[true, false, 123, -123.456, "foobar"]"#,
                Expression::new_tuple(vec![
                    Expression::from(true),
                    Expression::from(false),
                    Expression::from(123),
                    Expression::from(-123.456),
                    Expression::from("foobar"),
                ]),
                "",
            ),
            //             (
            //                 r#"{
            //         test = 123
            // }"#,
            //                 Expression::new_map(vec![vec![(
            //                     Key::new_identifier("test"),
            //                     Expression::from(123),
            //                 )]]),
            //                 "",
            //             ),
        ];

        for (input, expected_value, expected_remaining) in test_cases.iter() {
            println!("Testing {}", input);
            let (remaining, actual_value) = expression(CompleteStr(input)).unwrap();
            assert_eq!(&remaining.0, expected_remaining);
            assert_eq!(actual_value, *expected_value);
        }
    }

}
