// https://www.reddit.com/r/rust/comments/8rpzjd/parsing_string_literals_in_nom/
// https://github.com/Geal/nom/issues/787

use std::str;

use crate::errors::InternalKind;
use log::debug;
use nom::is_hex_digit;
use nom::types::CompleteByteSlice;
use nom::ErrorKind;
use nom::{
    alt, call, complete, delimited, do_parse, escaped_transform, many_till, map_res, named,
    named_args, opt, preceded, return_error, tag, take_while1, take_while_m_n,
};

fn not_single_line_string_illegal_byte(c: u8) -> bool {
    let c = c as char;
    let test = c != '\\' && c != '"' && c != '\n' && c != '\r';
    debug!("Checking valid string character {:?}: {:?}", c, test);
    test
}

fn octal_to_bytes(s: &[u8]) -> Result<Vec<u8>, InternalKind> {
    use std::char;

    let octal =
        u32::from_str_radix(str::from_utf8(s)?, 8).expect("Parser to have caught invalid inputs");
    let c = char::from_u32(octal).ok_or_else(|| InternalKind::InvalidUnicodeCodePoint)?;

    let mut buffer: Vec<u8> = vec![0; c.len_utf8()];
    c.encode_utf8(buffer.as_mut_slice());
    Ok(buffer)
}

fn hex_to_bytes(s: &[u8]) -> Result<Vec<u8>, InternalKind> {
    use std::char;

    let byte =
        u32::from_str_radix(str::from_utf8(s)?, 16).expect("Parser to have caught invalid inputs");
    let c = char::from_u32(byte).ok_or_else(|| InternalKind::InvalidUnicodeCodePoint)?;

    let mut buffer: Vec<u8> = vec![0; c.len_utf8()];
    c.encode_utf8(buffer.as_mut_slice());
    Ok(buffer)
}

/// Unescape characters according to the reference https://en.cppreference.com/w/cpp/language/escape
/// Source: https://github.com/hashicorp/hcl/blob/ef8a98b0bbce4a65b5aa4c368430a80ddc533168/hcl/scanner/scanner.go#L513
/// Unicode References: https://en.wikipedia.org/wiki/List_of_Unicode_characters
// TODO: Issues with variable length alt https://docs.rs/nom/4.2.0/nom/macro.alt.html#behaviour-of-alt
named!(unescape_bytes(&[u8]) -> Vec<u8>,
        alt!(
            // Control Chracters
            tag!("a")  => { |_| b"\x07".to_vec() }
            | tag!("b")  => { |_| b"\x08".to_vec() }
            | tag!("f")  => { |_| b"\x0c".to_vec() }
            | tag!("n") => { |_| b"\n".to_vec() }
            | tag!("r")  => { |_| b"\r".to_vec() }
            | tag!("t")  => { |_| b"\t".to_vec() }
            | tag!("v")  => { |_| b"\x0b".to_vec() }
            | tag!("\\") => { |_| b"\\".to_vec() }
            | tag!("\"") => { |_| b"\"".to_vec() }
            | tag!("?") => { |_| b"?".to_vec() }
            | map_res!(complete!(take_while_m_n!(1, 3, nom::is_oct_digit)), |s| octal_to_bytes(s))
            | hex_to_unicode_bytes
        )
);

named!(hex_to_unicode_bytes(&[u8]) -> Vec<u8>,
    return_error!(
        ErrorKind::Custom(InternalKind::InvalidUnicodeCodePoint as u32),
        alt!(
            // Technically the C++ spec allows characters of arbitrary length but the HashiCorp
            // Go implementation only scans up to two.
            map_res!(preceded!(tag!("x"), take_while_m_n!(1, 2, is_hex_digit)), |s| hex_to_bytes(s))
            | map_res!(preceded!(tag!("u"), take_while_m_n!(1, 4, is_hex_digit)), |s| hex_to_bytes(s))
            // The official unicode code points only go up to 6 digits
            | map_res!(preceded!(tag!("U"), take_while_m_n!(1, 8, is_hex_digit)), |s| hex_to_bytes(s))
        )
    )
);

/// Contents of a single line string
named!(
    single_line_string_content_bytes(&[u8]) -> Vec<u8>,
    escaped_transform!(
        take_while1!(not_single_line_string_illegal_byte),
        '\\',
        unescape_bytes
    )
);

named!(
    single_line_string_bytes(&[u8]) -> Vec<u8>,
    delimited!(
        tag!("\""),
        single_line_string_content_bytes,
        tag!("\"")
    )
);

/// Heredoc marker
#[derive(Debug, Eq, PartialEq)]
struct HereDoc<'a> {
    identifier: &'a [u8],
    indented: bool,
}

named!(heredoc_begin(&[u8]) -> HereDoc,
    do_parse!(
        tag!("<<")
        >> indented: opt!(complete!(tag!("-")))
        >> identifier: call!(nom::alphanumeric1)
        >> call!(nom::eol)
        >> (HereDoc {
                identifier,
                indented: indented == Some(b"-")
           })
    )
);

named_args!(
    heredoc_end<'a>(identifier: &'a HereDoc<'a>)<()>,
    do_parse!(
        call!(nom::eol)
        >> call!(nom::multispace0)
        >> tag!(identifier.identifier)
        >> call!(nom::eol)
        >> ()
    )
);

named!(
    heredoc_string(&[u8]) -> String,
    do_parse!(
        identifier: call!(heredoc_begin)
        >> strings: many_till!(call!(nom::anychar), call!(heredoc_end, &identifier))
        >> (strings.0.into_iter().collect())
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
            (r#"uD000"#, "\u{D000}"), // Unicode up to 4 bytes
            (r#"U29000"#, "\u{29000}"), // Unicode up to 8 bytes... but max unicode is only up to 6
        ];

        for (input, expected) in test_cases.iter() {
            let mut actual = unescape_bytes(input.as_bytes());
            // while actual.

            assert_eq!(actual.unwrap_output(), expected.as_bytes());
        }
    }

    // #[test]
    // #[should_panic(expected = "Invalid Unicode Code Points \\UD800")]
    // fn unescaping_invalid_unicode_errors() {
    //     unescape(CompleteStr("UD800")).unwrap_output();
    // }

    // #[test]
    // fn simple_string_content_are_parsed_correctly() {
    //     assert_eq!(
    //         single_line_string_content(CompleteStr("abcd")).unwrap_output(),
    //         "abcd".to_string()
    //     );
    // }

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
                single_line_string_content_bytes(input.as_bytes()).unwrap_output(),
                expected.as_bytes()
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
            (
                r#""ab \xff \251 \uD000 \U29000""#,
                "ab ÿ © \u{D000} \u{29000}",
            ),
        ];

        for (input, expected) in test_cases.iter() {
            assert_eq!(
                single_line_string_bytes(input.as_bytes()).unwrap_output(),
                expected.as_bytes()
            );
        }
    }

    #[test]
    fn heredoc_identifier_is_parsed_correctly() {
        let test_cases = [
            (
                "<<EOF\n",
                HereDoc {
                    identifier: b"EOF",
                    indented: false,
                },
            ),
            (
                "<<-EOH\n",
                HereDoc {
                    identifier: b"EOH",
                    indented: true,
                },
            ),
        ];

        for (input, expected) in test_cases.iter() {
            let (_, actual) = heredoc_begin(input.as_bytes()).unwrap();
            assert_eq!(&actual, expected);
        }
    }

    #[test]
    fn heredoc_end_is_parsed_correctly() {
        let test_cases = [
            (
                "\nEOF\n",
                HereDoc {
                    identifier: b"EOF",
                    indented: false,
                },
            ),
            (
                "\n    EOH\n",
                HereDoc {
                    identifier: b"EOH",
                    indented: true,
                },
            ),
        ];

        for (input, identifier) in test_cases.iter() {
            let _ = heredoc_end(input.as_bytes(), &identifier).unwrap();
        }
    }

    #[test]
    fn heredoc_strings_are_pased_correctly() {
        let test_cases = [
            (
                r#"<<EOF
something
EOF
"#,
                "something",
            ),
            (
                r#"<<EOH
something
with
new lines
and quotes "
                    EOH
"#,
                r#"something
with
new lines
and quotes ""#,
            ),
        ];

        for (input, expected) in test_cases.iter() {
            assert_eq!(
                heredoc_string(input.as_bytes()).unwrap().1,
                expected.to_string()
            );
        }
    }
}
