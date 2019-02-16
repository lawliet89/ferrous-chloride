// https://www.reddit.com/r/rust/comments/8rpzjd/parsing_string_literals_in_nom/
// https://github.com/Geal/nom/issues/787

use nom::types::CompleteStr;
use nom::{alt, digit, escaped_transform, named, tag, take_while1, take_while_m_n};

fn not_escape(c: char) -> bool {
    c != '\\'
}

fn bytes_to_string_safe(i: Vec<u8>) -> String {
    String::from_utf8_lossy(&i).into_owned()
}

fn octal_to_string(s: &str) -> CompleteStr {
    CompleteStr("")
}

/// Unescape characters according to the reference https://en.cppreference.com/w/cpp/language/escape
/// Source: https://github.com/hashicorp/hcl/blob/ef8a98b0bbce4a65b5aa4c368430a80ddc533168/hcl/scanner/scanner.go#L513
named!(unescape(CompleteStr) -> CompleteStr,
        alt!(
            // Control Chracters
            tag!("a")  => { |_| CompleteStr("\x07") }
            | tag!("b")  => { |_| CompleteStr("\x08") }
            | tag!("f")  => { |_| CompleteStr("\x0c") }
            | tag!("n") => { |_| CompleteStr("\n") }
            | tag!("r")  => { |_| CompleteStr("\r") }
            | tag!("t")  => { |_| CompleteStr("\t") }
            | tag!("v")  => { |_| CompleteStr("\x0b") }
            | tag!("\\") => { |_| CompleteStr("\\") }
            | tag!("\"") => { |_| CompleteStr("\"") }
            | tag!("?") => { |_| CompleteStr("?") }
            // // Octal, at most 3
            // | take_while_m_n!(1, 3, digit) => { |s| octal_to_string(s) }
        )
);

named!(
    string_content(CompleteStr) -> String,
    escaped_transform!(
        take_while1!(not_escape),
        '\\',
        unescape
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::ResultUtils;

    #[test]
    fn unescaping_works_correctly() {
        let test_cases = [
            (r#"a"#, "\x07"),
            (r#"b"#, "\x08"),
            (r#"f"#, "\x0c"),
            (r#"n"#, "\n"),
            (r#"r"#, "\r"),
            (r#"t"#, "\t"),
            (r#"v"#, "\x0b"),
            (r#"\"#, "\\"),
            (r#"""#, "\""),
            ("?", "?"),
        ];

        for (input, expected) in test_cases.iter() {
            assert_eq!(
                unescape(CompleteStr(input)).unwrap_output(),
                CompleteStr(expected)
            );
        }
    }

    #[test]
    fn simple_string_content_are_parsed_correctly() {
        assert_eq!(
            string_content(CompleteStr("abcd")).unwrap_output(),
            "abcd".to_string()
        );
    }

    #[test]
    fn escaped_string_content_are_parsed_correctly() {
        let test_cases = [
            (r#"ab\"cd"#, r#"ab"cd"#),
            (r#"ab \\ cd"#, r#"ab \ cd"#),
            (r#"ab \n cd"#, "ab \n cd"),
            (r#"ab \? cd"#, "ab ? cd"),
        ];

        for (input, expected) in test_cases.iter() {
            assert_eq!(
                string_content(CompleteStr(input)).unwrap_output(),
                expected.to_string()
            );
        }
    }
}
