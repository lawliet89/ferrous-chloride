//! String and Template
//!
//! - [Template Expression](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#template-expressions)
//! - [Template](https://github.com/hashicorp/hcl2/blob/master/hcl/hclsyntax/spec.md#templates)
//!

use std::borrow::Cow;
use std::str;

use crate::errors::InternalKind;
use log::debug;
use nom::types::CompleteStr;
use nom::ErrorKind;
use nom::{
    alt, call, complete, delimited, do_parse, escaped_transform, many_till, map, map_res, named,
    named_args, opt, peek, preceded, return_error, tag, take_while1, take_while_m_n,
};


/// The StringLit production permits the escape sequences discussed for quoted template expressions
/// as above, but does not permit template interpolation or directive sequences.
pub type StringLiteral = String;

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn is_oct_digit(c: char) -> bool {
    c.is_digit(8)
}

fn legal_string_literal_character(c: char) -> bool {
    let test = c != '\\' && c != '"';
    debug!("Checking valid string character {:?}: {:?}", c, test);
    test
}

fn legal_string_literal_single_line_character(c: char) -> bool {
    let test = c != '\\' && c != '"' && c != '\r' && c != '\n';
    debug!("Checking valid string character {:?}: {:?}", c, test);
    test
}

fn octal_to_string(s: &str) -> Result<String, InternalKind> {
    use std::char;

    let octal = u32::from_str_radix(s, 8).expect("Parser to have caught invalid inputs");
    Ok(char::from_u32(octal)
        .ok_or_else(|| InternalKind::InvalidUnicodeCodePoint)?
        .to_string())
}

fn hex_to_string(s: &str) -> Result<String, InternalKind> {
    let byte = u32::from_str_radix(s, 16).expect("Parser to have caught invalid inputs");
    Ok(std::char::from_u32(byte)
        .ok_or_else(|| InternalKind::InvalidUnicodeCodePoint)?
        .to_string())
}

// Unescape characters according to the reference https://en.cppreference.com/w/cpp/language/escape
// Source: https://github.com/hashicorp/hcl/blob/ef8a98b0bbce4a65b5aa4c368430a80ddc533168/hcl/scanner/scanner.go#L513
// Unicode References: https://en.wikipedia.org/wiki/List_of_Unicode_characters
// TODO: Issues with variable length alt https://docs.rs/nom/4.2.0/nom/macro.alt.html#behaviour-of-alt
named!(unescape(CompleteStr) -> Cow<str>,
    alt!(
        // Control Chracters
        tag!("a")  => { |_| Cow::Borrowed("\x07") }
        | tag!("b")  => { |_| Cow::Borrowed("\x08") }
        | tag!("f")  => { |_| Cow::Borrowed("\x0c") }
        | tag!("n") => { |_| Cow::Borrowed("\n") }
        | tag!("r")  => { |_| Cow::Borrowed("\r") }
        | tag!("t")  => { |_| Cow::Borrowed("\t") }
        | tag!("v")  => { |_| Cow::Borrowed("\x0b") }
        | tag!("\\") => { |_| Cow::Borrowed("\\") }
        | tag!("\"") => { |_| Cow::Borrowed("\"") }
        | tag!("?") => { |_| Cow::Borrowed("?") }
        | map!(map_res!(complete!(take_while_m_n!(1, 3, is_oct_digit)), |s: CompleteStr| octal_to_string(s.0)), Cow::Owned)
        | hex_to_unicode
    )
);

named!(hex_to_unicode(CompleteStr) -> Cow<str>,
    return_error!(
        ErrorKind::Custom(InternalKind::InvalidUnicodeCodePoint as u32),
        map!(
            alt!(
                // Technically the C++ spec allows characters of arbitrary length but the HashiCorp
                // Go implementation only scans up to two.
                map_res!(preceded!(tag!("x"), take_while_m_n!(1, 2, is_hex_digit)), |s: CompleteStr| hex_to_string(s.0))
                | map_res!(preceded!(tag!("u"), take_while_m_n!(1, 4, is_hex_digit)), |s: CompleteStr| hex_to_string(s.0))
                // The official unicode code points only go up to 6 digits
                | map_res!(preceded!(tag!("U"), take_while_m_n!(1, 8, is_hex_digit)), |s: CompleteStr| hex_to_string(s.0))
            ),
            Cow::Owned
        )
    )
);

// Contents of a single line string
named!(
    multiline_string_content(CompleteStr) -> String,
    escaped_transform!(
        take_while1!(legal_string_literal_character),
        '\\',
        unescape
    )
);

named!(
    quoted_string(CompleteStr) -> String,
    delimited!(
        tag!("\""),
        call!(multiline_string_content),
        tag!("\"")
    )
);

named!(
    pub string_literal_content(CompleteStr) -> StringLiteral,
    escaped_transform!(
        take_while1!(legal_string_literal_single_line_character),
        '\\',
        unescape
    )
);

named!(
    pub string_literal(CompleteStr) -> StringLiteral,
    delimited!(
        tag!("\""),
        call!(string_literal_content),
        tag!("\"")
    )
);

/// Heredoc marker
#[derive(Debug, Eq, PartialEq)]
pub struct HereDoc<'a> {
    pub identifier: CompleteStr<'a>,
    pub indented: bool,
}

// Start of heredoc identifier. Must end with an EOL
// EOL is not consumed
named!(
    pub heredoc_begin(CompleteStr) -> HereDoc,
    do_parse!(
        tag!("<<")
        >> indented: opt!(complete!(tag!("-")))
        >> identifier: call!(crate::utils::while_predicate1, |c| c.is_alphanumeric() || c == '_')
        >> peek!(call!(nom::eol))
        >> (HereDoc {
                identifier,
                indented: indented == Some(CompleteStr("-"))
           })
    )
);

// End of heredoc. Must end with an EOL
// EOL is not consumed
named_args!(
    pub heredoc_end<'a>(identifier: &'_ HereDoc<'_>)<CompleteStr<'a>, ()>,
    do_parse!(
        call!(nom::eol)
        >> call!(nom::multispace0)
        >> tag!(identifier.identifier.0)
        >> peek!(call!(nom::eol))
        >> ()
    )
);

// Parse a Heredoc string
named!(
    pub heredoc_string(CompleteStr) -> String,
    do_parse!(
        identifier: call!(heredoc_begin)
        >> strings: alt!(
            call!(heredoc_end, &identifier) => {|()| vec![] }
            | do_parse!(
                call!(nom::eol)
                >> content: many_till!(call!(nom::anychar), call!(heredoc_end, &identifier))
                >> (content.0)
            )
        )
        >> (strings.into_iter().collect())
    )
);

named!(
    pub string(CompleteStr) -> String,
    alt!(
        quoted_string
        | heredoc_string
    )
);

// TODO:
// - Interpolation `${test("...")}`
// - Unindent heredoc: https://github.com/hashicorp/hcl/blob/65a6292f0157eff210d03ed1bf6c59b190b8b906/hcl/token/token.go#L174

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::*;

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
            (r#"xff"#, "ÿ"),           // Hex
            (r#"251"#, "©"),           // Octal
            (r#"uD000"#, "\u{D000}"),   // Unicode up to 4 bytes
            (r#"U29000"#, "\u{29000}"), // Unicode up to 8 bytes... but max unicode is only up to 6
        ];

        for (input, expected) in test_cases.iter() {
            println!("Testing {}", input);
            let actual = unescape(CompleteStr(input)).map(|(i, o)| (i, o.into_owned()));
            assert_eq!(ResultUtilsString::unwrap_output(actual), *expected);
        }
    }

    #[test]
    #[should_panic(expected = "Invalid Unicode Code Points \\UD800")]
    fn unescaping_invalid_unicode_errors() {
        let actual = unescape(CompleteStr("UD800")).map(|(i, o)| (i, o.into_owned()));
        ResultUtilsString::unwrap_output(actual);
    }

    #[test]
    fn string_content_are_parsed_correctly() {
        let test_cases = [
            ("", ""),
            (r#"abcd"#, r#"abcd"#),
            (r#"ab\"cd"#, r#"ab"cd"#),
            (r#"ab \\ cd"#, r#"ab \ cd"#),
            (r#"ab \n cd"#, "ab \n cd"),
            (r#"ab \? cd"#, "ab ? cd"),
            (
                r#"ab \xff \251 \uD000 \U29000"#,
                "ab ÿ © \u{D000} \u{29000}",
            ),
            ("ab\ncd", "ab\ncd"),
        ];

        for (input, expected) in test_cases.iter() {
            println!("Testing {}", input);
            let actual = multiline_string_content(CompleteStr(input));
            assert_eq!(
                ResultUtilsString::unwrap_output(actual.map(|s| s.to_owned())),
                *expected
            );
        }
    }

    #[test]
    fn quoted_string_literals_are_parsed_correctly() {
        let test_cases = [
            (r#""""#, ""),
            (r#""abcd""#, r#"abcd"#),
            (r#""ab\"cd""#, r#"ab"cd"#),
            (r#""ab \\ cd""#, r#"ab \ cd"#),
            (r#""ab \n cd""#, "ab \n cd"),
            (r#""ab \? cd""#, "ab ? cd"),
            (
                r#""ab \xff \251 \uD000 \U29000""#,
                "ab ÿ © \u{D000} \u{29000}",
            ),
            ("\"ab\ncd\"", "ab\ncd"),
        ];

        for (input, expected) in test_cases.iter() {
            println!("Testing {}", input);
            assert_eq!(
                ResultUtilsString::unwrap_output(quoted_string(CompleteStr(input))),
                *expected
            );
        }
    }

    #[test]
    fn heredoc_identifier_is_parsed_correctly() {
        let test_cases = [
            (
                "<<EOF\n",
                HereDoc {
                    identifier: CompleteStr("EOF"),
                    indented: false,
                },
                "\n",
            ),
            (
                "<<-EOH\n",
                HereDoc {
                    identifier: CompleteStr("EOH"),
                    indented: true,
                },
                "\n",
            ),
            (
                "<<藏_\r\n",
                HereDoc {
                    identifier: CompleteStr("藏_"),
                    indented: false,
                },
                "\r\n",
            ),
        ];

        for (input, expected, expected_remaining) in test_cases.iter() {
            println!("Testing {}", input);
            let (remaining, actual) = heredoc_begin(CompleteStr(input)).unwrap();
            assert_eq!(&remaining.0, expected_remaining);
            assert_eq!(&actual, expected);
        }
    }

    #[test]
    fn heredoc_end_is_parsed_correctly() {
        let test_cases = [
            (
                "\nEOF\n",
                HereDoc {
                    identifier: CompleteStr("EOF"),
                    indented: false,
                },
                "\n",
            ),
            (
                "\n    EOH\n",
                HereDoc {
                    identifier: CompleteStr("EOH"),
                    indented: true,
                },
                "\n",
            ),
            (
                "\r\nEOF\r\n",
                HereDoc {
                    identifier: CompleteStr("EOF"),
                    indented: false,
                },
                "\r\n",
            ),
        ];

        for (input, identifier, expected_remaining) in test_cases.iter() {
            println!("Testing {}", input);
            let (remaining, ()) = heredoc_end(CompleteStr(input), &identifier).unwrap();
            assert_eq!(
                &remaining.0, expected_remaining,
                "Input: {}; Remaining: {}",
                input, remaining
            );
        }
    }

    #[test]
    fn heredoc_strings_are_pased_correctly() {
        let test_cases = [
            (
                r#"<<EOF
EOF
"#,
                "",
            ),
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
            println!("Testing {}", input);
            let (remaining, actual) = heredoc_string(CompleteStr(input)).unwrap();
            assert_eq!(remaining.0, "\n");
            assert_eq!(actual, expected.to_string());
        }
    }

    #[test]
    fn strings_are_parsed_correctly() {
        let test_cases = [
            (r#""""#, "", ""),
            (r#""abcd""#, r#"abcd"#, ""),
            (r#""ab\"cd""#, r#"ab"cd"#, ""),
            (r#""ab \\ cd""#, r#"ab \ cd"#, ""),
            (r#""ab \n cd""#, "ab \n cd", ""),
            (r#""ab \? cd""#, "ab ? cd", ""),
            (
                r#"<<EOF
    EOF
"#,
                "",
                "\n",
            ),
            (
                r#""ab \xff \251 \uD000 \U29000""#,
                "ab ÿ © \u{D000} \u{29000}",
                "",
            ),
            (
                r#"<<EOF
something
    EOF
"#,
                "something",
                "\n",
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
                "\n",
            ),
        ];

        for (input, expected, expected_remaining) in test_cases.iter() {
            println!("Testing {}", input);
            let (remaining, actual) = string(CompleteStr(input)).unwrap();
            assert_eq!(&remaining.0, expected_remaining);
            assert_eq!(&actual, expected, "Input: {}", input);
        }
    }
}
