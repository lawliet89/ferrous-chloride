// https://www.reddit.com/r/rust/comments/8rpzjd/parsing_string_literals_in_nom/
// https://github.com/Geal/nom/issues/787

use log::debug;
use nom::map;
use nom::types::CompleteStr;
use nom::ErrorKind;
use nom::{
    alt, call, complete, delimited, do_parse, escaped_transform, map_res, named, opt, preceded,
    return_error, tag, take_while1, take_while_m_n,
};

use crate::errors::InternalKind;

fn not_single_line_string_illegal(c: char) -> bool {
    let test = c != '\\' && c != '"' && c != '\n' && c != '\r';
    debug!("Checking valid string character {:?}: {:?}", c, test);
    test
}

fn octal_to_string(s: CompleteStr) -> String {
    use std::char;

    let octal = u32::from_str_radix(s.as_ref(), 8).expect("Parser to have caught invalid inputs");
    char::from_u32(octal)
        .map(|c| c.to_string())
        .expect("To never fail between 0 and 511.")
}

fn hex_to_string(s: CompleteStr) -> Result<String, InternalKind> {
    use std::char;

    let byte = u32::from_str_radix(s.as_ref(), 16).expect("Parser to have caught invalid inputs");
    char::from_u32(byte)
        .map(|c| c.to_string())
        .ok_or_else(|| InternalKind::InvalidUnicode)
}

fn is_octal(s: char) -> bool {
    s.len_utf8() == 1 && nom::is_oct_digit(s as u8)
}

fn is_hex(s: char) -> bool {
    s.len_utf8() == 1 && nom::is_hex_digit(s as u8)
}

/// Unescape characters according to the reference https://en.cppreference.com/w/cpp/language/escape
/// Source: https://github.com/hashicorp/hcl/blob/ef8a98b0bbce4a65b5aa4c368430a80ddc533168/hcl/scanner/scanner.go#L513
/// Unicode References: https://en.wikipedia.org/wiki/List_of_Unicode_characters
// TODO: Issues with variable length alt https://docs.rs/nom/4.2.0/nom/macro.alt.html#behaviour-of-alt
named!(unescape(CompleteStr) -> String,
        alt!(
            // Control Chracters
            tag!("a")  => { |_| "\x07".to_string() }
            | tag!("b")  => { |_| "\x08".to_string() }
            | tag!("f")  => { |_| "\x0c".to_string() }
            | tag!("n") => { |_| "\n".to_string() }
            | tag!("r")  => { |_| "\r".to_string() }
            | tag!("t")  => { |_| "\t".to_string() }
            | tag!("v")  => { |_| "\x0b".to_string() }
            | tag!("\\") => { |_| "\\".to_string() }
            | tag!("\"") => { |_| "\"".to_string() }
            | tag!("?") => { |_| "?".to_string() }
            | complete!(take_while_m_n!(1, 3, is_octal)) => { |s| octal_to_string(s) }
            // Technically the C++ spec allows characters of arbitrary length but the HashiCorp
            // Go implementation only scans up to two.
            | hex_to_unicode
        )
);

named!(hex_to_unicode(CompleteStr) -> String,
    return_error!(
        ErrorKind::Custom(InternalKind::InvalidUnicode as u32),
        alt!(
            // Technically the C++ spec allows characters of arbitrary length but the HashiCorp
            // Go implementation only scans up to two.
            map_res!(preceded!(tag!("x"), take_while_m_n!(1, 2, is_hex)), |s| hex_to_string(s))
            | map_res!(preceded!(tag!("u"), take_while_m_n!(1, 4, is_hex)), |s| hex_to_string(s))
            // The official unicode code points only go up to 6 digits, but the HashiCorp implementation
            // parses till 8
            | map_res!(preceded!(tag!("U"), take_while_m_n!(1, 6, is_hex)), |s| hex_to_string(s))
        )
    )
);

/// Contents of a single line string
named!(
    pub single_line_string_content(CompleteStr) -> String,
    escaped_transform!(
        take_while1!(not_single_line_string_illegal),
        // nom::alpha,
        '\\',
        unescape
    )
);

named!(
    single_line_string(CompleteStr) -> String,
    map!(
        delimited!(
            tag!("\""),
            single_line_string_content,
            tag!("\"")
        ),
        |s| s.to_string()
    )
);

/// Heredoc marker
#[derive(Debug, Eq, PartialEq)]
struct HereDoc {
    identifier: String,
    indented: bool,
}

named!(heredoc_begin(&[u8]) -> HereDoc,
    do_parse!(
        tag!("<<")
        >> indented: opt!(complete!(tag!("-")))
        >> identifier: call!(nom::alphanumeric1)
        >> call!(nom::eol)
        >> (HereDoc {
                identifier: String::from_utf8_lossy(identifier).into_owned(),
                indented: indented == Some(b"-")
           })
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
            (r#"xff"#, "ÿ"), // Hex
            (r#"251"#, "©"), // Octal
            (r#"uD000"#, "\u{D000}"),
            (r#"U29000"#, "\u{29000}"),
        ];

        for (input, expected) in test_cases.iter() {
            assert_eq!(unescape(CompleteStr(input)).unwrap_output(), *expected);
        }
    }

    #[test]
    #[should_panic(expected = "Invalid Unicode Code Points \\UD800")]
    fn unescaping_invalid_unicode_errors() {
        unescape(CompleteStr("UD800")).unwrap_output();
    }

    #[test]
    fn simple_string_content_are_parsed_correctly() {
        assert_eq!(
            single_line_string_content(CompleteStr("abcd")).unwrap_output(),
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
                single_line_string_content(CompleteStr(input)).unwrap_output(),
                expected.to_string()
            );
        }
    }

    #[test]
    fn single_line_string_literals_are_parsed_correctly() {
        let test_cases = [
            (r#""ab\"cd""#, r#"ab"cd"#),
            (r#""ab \\ cd""#, r#"ab \ cd"#),
            (r#""ab \n cd""#, "ab \n cd"),
            (r#""ab \? cd""#, "ab ? cd"),
        ];

        for (input, expected) in test_cases.iter() {
            assert_eq!(
                single_line_string(CompleteStr(input)).unwrap_output(),
                expected.to_string()
            );
        }
    }

    #[test]
    fn heredoc_identifier_is_parsed_correctly() {
        let test_cases = [
            (
                "<<EOF\n",
                HereDoc {
                    identifier: "EOF".to_string(),
                    indented: false,
                },
            ),
            (
                "<<-EOH\n",
                HereDoc {
                    identifier: "EOH".to_string(),
                    indented: true,
                },
            ),
        ];

        for (input, expected) in test_cases.iter() {
            let (_, actual) = heredoc_begin(input.as_bytes()).unwrap();
            assert_eq!(&actual, expected);
        }
    }
}
