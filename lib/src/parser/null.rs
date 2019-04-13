use nom::types::CompleteStr;
use nom::{do_parse, named_attr, tag};

named_attr!(
    #[doc = r#"Parses the literal `null` as
              [`()`](https://doc.rust-lang.org/std/primitive.unit.html)"#],
    pub null(CompleteStr) -> (),
    do_parse!(
        tag!("null")
        >> (())
    )
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::ResultUtilsString;
    #[test]
    fn parses_for_null() {
        null(CompleteStr("null")).unwrap_output();
    }

    #[test]
    fn errors_for_non_null() {
        use nom;
        match null(CompleteStr("true")) {
            Ok(_) => panic!("Should not succeed"),
            Err(nom::Err::Error(nom::verbose_errors::Context::Code(_, nom::ErrorKind::Tag))) => {}
            Err(e) => panic!("Unexpected error {:#?}", e),
        }
    }
}
