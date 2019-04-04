//! Object
//!
//! An Object is part of `CollectionValue`
//!
//! [Reference](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#collection-values)
//!
//! ```ebnf
//! object = "{" (
//!     (objectelem ("," objectelem)* ","?)?
//! ) "}";
//! objectelem = (Identifier | Expression) "=" Expression;
//! ```
use std::borrow::Cow;

use super::expression::Expression;

// TODO: Dealing with expressions and ambiguity. See reference
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ObjectIdentifier<'a> {
    Identifier(Cow<'a, str>),
    // Expression(Expression<'a>),
}
