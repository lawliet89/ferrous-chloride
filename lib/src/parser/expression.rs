//! Expressions
//!
//! [Reference](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#expressions)

use std::borrow::Cow;
use std::iter::FromIterator;

use nom::types::CompleteStr;
use nom::{alt_complete, call, do_parse, named, tag};

use crate::constants::*;
use crate::parser::boolean::boolean;
use crate::parser::null::null;
use crate::parser::number::{number, Number};
use crate::parser::object::{object, Object, ObjectElementIdentifier};
use crate::parser::string::string;
use crate::parser::tuple::{tuple, Tuple};
use crate::Error;

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
    /// A `null` HCL expression, expressed literally
    Null,
    /// An arbitrary precision number
    Number(Number<'a>),
    /// A boolean value, expressed as the literals `true` or `false`
    Boolean(bool),
    /// A HCL string
    String(Cow<'a, str>),
    /// A HCL tuple (list)
    Tuple(Tuple<'a>),
    /// A HCL object (map)
    Object(Object<'a>),
}

impl<'a> Expression<'a> {
    /// Parse a string as a HCL expression
    ///
    /// The string is expected to be fully consumed during parsing or an eror will be returned.
    ///
    /// In general, this method should not be used. Prefer to use
    /// [`parse_str`](crate::parser::parse_str) to parse a HCL configuration file instead.
    pub fn parse(s: &'a str) -> Result<Self, Error> {
        let (remaining, expr) = expression(CompleteStr(s)).map_err(|e| Error::from_err_str(&e))?;;
        if !remaining.is_empty() {
            return Err(Error::UnexpectedRemainingInput(remaining.to_string()));
        }
        Ok(expr)
    }

    /// Convenience method to create a new Tuple Expression variant from an iterator of Expressions
    pub fn new_tuple<T>(iterator: T) -> Self
    where
        T: IntoIterator<Item = Expression<'a>>,
    {
        Expression::Tuple(iterator.into_iter().collect())
    }

    /// Convenient method to create a new Object Expression variant from an iterator of
    /// Key-value pairs.
    pub fn new_object<T, I>(iterator: T) -> Self
    where
        T: IntoIterator<Item = (I, Expression<'a>)>,
        I: Into<ObjectElementIdentifier<'a>> + 'a,
    {
        Expression::Object(iterator.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    /// Get the name of the Expression variant as a string.
    pub fn variant_name(&self) -> &'static str {
        match self {
            Expression::Null => NULL,
            Expression::Number(_) => NUMBER,
            Expression::Boolean(_) => BOOLEAN,
            Expression::String(_) => STRING,
            Expression::Tuple(_) => TUPLE,
            Expression::Object(_) => OBJECT,
        }
    }
}

impl<'a> crate::AsOwned for Expression<'a> {
    type Output = Expression<'static>;

    fn as_owned(&self) -> Self::Output {
        match self {
            Expression::Null => Expression::Null,
            Expression::Number(number) => Expression::Number(number.as_owned()),
            Expression::Boolean(boolean) => Expression::Boolean(*boolean),
            Expression::String(string) => Expression::String(Cow::Owned(string.to_string())),
            Expression::Tuple(tup) => Expression::Tuple(tup.as_owned()),
            Expression::Object(obj) => Expression::Object(obj.as_owned()),
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
impl_from_expr_type!(String, Cow<'a, str>);
impl_from_expr_type!(Tuple, Vec<Expression<'a>>);

impl<'a> From<()> for Expression<'a> {
    fn from(_: ()) -> Self {
        Expression::Null
    }
}

impl<'a> From<&'a str> for Expression<'a> {
    fn from(s: &'a str) -> Self {
        Expression::String(Cow::Borrowed(s))
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
        call!(null) => { |_| Expression::Null }
        // LiteralValue -> NumericLit
        | call!(number) => { |v| From::from(v) }
        // LiteralValue -> "true" | "false"
        | call!(boolean) => { |v| From::from(v) }
        // TemplateExpr
        // https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#template-expressions
        | string => { |v| From::from(v) }
        // CollectionValue -> tuple
        | tuple => { |v| From::from(v) }
        // CollectionValue -> object
        | object => { |obj| Expression::Object(obj) }
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
            (r#""foobar""#, Expression::from("foobar"), ""),
            (
                r#"
(
<<EOF
new
line
EOF
)
"#,
                Expression::from("new\nline"),
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
            (r#""foobar""#, Expression::from("foobar"), ""),
            (
                r#"<<EOF
new
line
EOF
"#,
                Expression::from("new\nline"),
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
            (
                r#"{
                    test = 123
            }"#,
                Expression::new_object(vec![("test", Expression::from(123))]),
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
