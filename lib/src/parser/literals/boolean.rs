use nom::types::CompleteStr;
use nom::{alt, map, named, tag};

// Parse a boolean literal
named!(pub boolean(CompleteStr) -> bool,
    map!(
        alt!(
            tag!("true")
            | tag!("false")
        ),
        |s| s.as_ref() == "true"    // Can only ever be "true" or "false"
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
