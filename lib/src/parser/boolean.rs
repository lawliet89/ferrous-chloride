use nom::types::CompleteStr;
use nom::{alt, named, tag};

// Parse a boolean literal
named!(pub boolean(CompleteStr) -> bool,
    alt!(
        tag!("true") => {|_| true}
        | tag!("false") => {|_| false}
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::ResultUtilsString;

    #[test]
    fn booleans_are_parsed_correctly() {
        assert_eq!(boolean(CompleteStr("true")).unwrap_output(), true);
        assert_eq!(boolean(CompleteStr("false")).unwrap_output(), false);
    }
}
