use nom::types::CompleteStr;
use nom::{do_parse, named, tag};

named!(
    pub null(CompleteStr) -> (),
    do_parse!(
        tag!("null")
        >> (())
    )
);

#[cfg(test)]
#[test]
fn parses_for_null() {
    use crate::utils::ResultUtilsString;
    null(CompleteStr("null")).unwrap_output();
}
