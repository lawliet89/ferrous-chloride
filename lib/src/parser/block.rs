//! Block structure
//!
//! Blocks create a child body annotated by a type and optional labels.
//!
//! ```ebnf
//! Block        = Identifier (StringLit|Identifier)* "{" Newline Body "}" Newline;
//! OneLineBlock = Identifier (StringLit|Identifier)* "{" (Identifier "=" Expression)? "}" Newline;
//! ```
use std::borrow::{Borrow, Cow};

use crate::parser::body::Body;
use crate::parser::identifier::Identifier;
use crate::parser::string::StringLiteral;

/// HCL Block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block<'a> {
    pub r#type: Identifier<'a>,
    pub labels: Vec<BlockLabel<'a>>,
    pub body: Body<'a>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BlockLabel<'a> {
    StringLiteral(StringLiteral),
    Identifier(Identifier<'a>),
}

impl<'a, S> PartialEq<S> for BlockLabel<'a>
where
    S: AsRef<str>,
{
    fn eq(&self, other: &S) -> bool {
        match self {
            BlockLabel::StringLiteral(lit) => lit.eq(other.as_ref()),
            BlockLabel::Identifier(ident) => ident.eq(other.as_ref()),
        }
    }
}

impl<'a> Borrow<str> for BlockLabel<'a> {
    fn borrow(&self) -> &str {
        match self {
            BlockLabel::StringLiteral(ref lit) => lit,
            BlockLabel::Identifier(ref ident) => ident,
        }
    }
}

impl<'a> From<&'a str> for BlockLabel<'a> {
    fn from(s: &'a str) -> Self {
        BlockLabel::Identifier(Cow::Borrowed(s))
    }
}
