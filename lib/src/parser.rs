use crate::value;
use crate::{AsOwned, Error, MergeBehaviour};

use nom::types::CompleteStr;
use nom::{call, exact, named};

named!(
    pub body(CompleteStr) -> Body,
    exact!(call!(value::map_values))
);

/// Parse a HCL string into a [`Body`] which is close to an abstract syntax tree of the
/// HCL string.
///
/// You can opt to merge the parsed body after parsing. The behaviour of merging is determined by
/// the [`MergeBehaviour`] enum.
pub fn parse_str(input: &str, merge: Option<MergeBehaviour>) -> Result<Body, Error> {
    let (remaining_input, unmerged) =
        body(CompleteStr(input)).map_err(|e| Error::from_err_str(&e))?;

    if !remaining_input.is_empty() {
        Err(Error::Bug(format!(
            r#"Input was not completely parsed:
Input: {},
Remaining: {}
"#,
            input, remaining_input
        )))?
    }

    let pairs = match merge {
        None => unmerged,
        Some(MergeBehaviour::Error) => unmerged.merge()?,
        Some(_) => unimplemented!("Not implemented yet"),
    };

    Ok(pairs)
}

/// Parse a HCL string from a IO stream reader
///
/// The entire IO stream has to be buffered in memory first before parsing can occur.
///
/// When reading from a source against which short reads are not efficient, such as a
/// [`File`](std::fs::File), you will want to apply your own buffering because the library
/// will not buffer the input. See [`std::io::BufReader`].
pub fn parse_reader<R: std::io::Read>(
    mut reader: R,
    merge: Option<MergeBehaviour>,
) -> Result<Body<'static>, Error> {
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;

    // FIXME: Can we do better? We are allocating twice. Once for reading into a buffer
    // and second time calling `as_owned`.
    Ok(parse_str(&buffer, merge)?.as_owned())
}

/// Parse a HCL string from a slice of bytes
pub fn parse_slice(bytes: &[u8], merge: Option<MergeBehaviour>) -> Result<Body, Error> {
    let input = std::str::from_utf8(bytes)?;
    parse_str(input, merge)
}

/// A HCL document body
///
/// ```ebnf
/// ConfigFile   = Body;
/// Body         = (Attribute | Block | OneLineBlock)*;
/// Attribute    = Identifier "=" Expression Newline;
/// Block        = Identifier (StringLit|Identifier)* "{" Newline Body "}" Newline;
/// OneLineBlock = Identifier (StringLit|Identifier)* "{" (Identifier "=" Expression)? "}" Newline;
/// ```
pub type Body<'a> = value::MapValues<'a>;

#[cfg(test)]
mod tests {
    use super::*;

    use crate::fixtures;
    use crate::Mergeable;

    #[test]
    fn strings_are_parsed_correctly_unmerged() {
        for string in fixtures::ALL {
            let parsed = parse_str(string, None).unwrap();
            assert!(parsed.is_unmerged());
        }
    }

    #[test]
    fn strings_are_parsed_correctly_merged() {
        for string in fixtures::ALL {
            let parsed = parse_str(string, Some(MergeBehaviour::Error)).unwrap();
            assert!(parsed.is_merged());
        }
    }
}
