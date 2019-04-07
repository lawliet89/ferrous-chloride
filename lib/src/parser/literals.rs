//! Tokens and literals
#[macro_use]
pub mod whitespace;

pub mod key;
pub mod number;

#[doc(inline)]
pub use key::{key, Key};
#[doc(inline)]
pub use number::{number, Number};
#[doc(inline)]
pub use whitespace::{inline_whitespace, newline, whitespace};
