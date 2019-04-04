use std::borrow::Cow;

use super::expression::Expression;
/// A HCL Attribute
///
/// ```ebnf
/// Attribute = Identifier "=" Expression Newline;
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Attribute<'a> {
    pub identifier: Cow<'a, str>,
    pub expression: Expression<'a>,
}
