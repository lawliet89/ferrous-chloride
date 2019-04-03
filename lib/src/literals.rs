//! Tokens and literals
#[macro_use]
pub mod whitespace;

pub mod boolean;
pub mod identifier;
pub mod key;
pub mod number;
pub mod string;

#[doc(inline)]
pub use boolean::boolean;
#[doc(inline)]
pub use identifier::identifier;
#[doc(inline)]
pub use key::{key, Key};
#[doc(inline)]
pub use number::{number, Number};
#[doc(inline)]
pub use string::{quoted_single_line_string, string};
#[doc(inline)]
pub use whitespace::{inline_whitespace, newline, whitespace};

use nom::types::CompleteStr;
use nom::{do_parse, named, tag};

named!(
    pub null(CompleteStr) -> (),
    do_parse!(
        tag!("null")
        >> (())
    )
);

#[test]
fn parses_for_null() {
    use crate::utils::test::ResultUtilsString;
    null(CompleteStr("null")).unwrap_output();
}
