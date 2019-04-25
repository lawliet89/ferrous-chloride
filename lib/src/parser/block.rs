//! Block structure
//!
//! Blocks create a child body annotated by a type and optional labels.
//!
//! ```ebnf
//! Block        = Identifier (StringLit|Identifier)* "{" Newline Body "}" Newline;
//! OneLineBlock = Identifier (StringLit|Identifier)* "{" (Identifier "=" Expression)? "}" Newline;
//! ```
use std::borrow::{Borrow, Cow};
use std::collections::hash_map::{self, Entry};
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::iter::{Extend, FromIterator};

use itertools::Itertools;
use nom::types::CompleteStr;
use nom::{alt, call, many0, named, opt, tag};

use crate::parser::attribute::{attribute, Attribute};
use crate::parser::body::{body, Body};
use crate::parser::identifier::{identifier, Identifier};
use crate::parser::string::{string_literal, StringLiteral};
use crate::parser::whitespace::newline;

/// HCL Block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block<'a> {
    pub r#type: Identifier<'a>,
    pub labels: Vec<BlockLabel<'a>>,
    pub body: Body<'a>,
}

impl<'a> Block<'a> {
    pub fn new(r#type: Identifier<'a>, labels: Vec<BlockLabel<'a>>, body: Body<'a>) -> Self {
        Self {
            r#type,
            labels,
            body,
        }
    }

    pub fn new_one_line(
        r#type: Identifier<'a>,
        labels: Vec<BlockLabel<'a>>,
        attribute: Option<Attribute<'a>>,
    ) -> Self {
        let body = match attribute {
            None => vec![],
            Some(attr) => vec![From::from(attr)],
        };

        Self {
            r#type,
            labels,
            body,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockLabel<'a> {
    StringLiteral(StringLiteral),
    Identifier(Identifier<'a>),
}

impl<'a> BlockLabel<'a> {
    pub fn as_str(&self) -> &str {
        self.borrow()
    }

    pub fn as_cow(&self) -> Cow<'a, str> {
        match self {
            BlockLabel::StringLiteral(literal) => Cow::Owned(literal.clone()),
            BlockLabel::Identifier(ident) => ident.clone(),
        }
    }
}

impl<'a> Hash for BlockLabel<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl<'a, S> PartialEq<S> for BlockLabel<'a>
where
    S: AsRef<str>,
{
    fn eq(&self, other: &S) -> bool {
        match self {
            BlockLabel::StringLiteral(lit) => lit.eq(other.as_ref()),
            BlockLabel::Identifier(ident) => ident.eq(other.as_ref()),
        }
    }
}

impl<'a> Borrow<str> for BlockLabel<'a> {
    fn borrow(&self) -> &str {
        match self {
            BlockLabel::StringLiteral(ref lit) => lit,
            BlockLabel::Identifier(ref ident) => ident,
        }
    }
}

impl<'a> From<&'a str> for BlockLabel<'a> {
    fn from(s: &'a str) -> Self {
        BlockLabel::Identifier(Cow::Borrowed(s))
    }
}

impl<'a> crate::AsOwned for BlockLabel<'a> {
    type Output = BlockLabel<'static>;

    fn as_owned(&self) -> Self::Output {
        match self {
            BlockLabel::StringLiteral(string) => BlockLabel::StringLiteral(string.clone()),
            BlockLabel::Identifier(ident) => BlockLabel::Identifier(Cow::Owned(ident.as_owned())),
        }
    }
}

impl<'a> crate::AsOwned for Block<'a> {
    type Output = Block<'static>;

    fn as_owned(&self) -> Self::Output {
        Self::Output {
            r#type: Cow::Owned(self.r#type.as_owned()),
            labels: self.labels.as_owned(),
            body: self.body.as_owned(),
        }
    }
}

named!(
    pub block_label(CompleteStr) -> BlockLabel,
    alt!(
        call!(identifier) =>
            { |ident| BlockLabel::Identifier(ident) }
        | call!(string_literal) =>
            { |s| BlockLabel::StringLiteral(s) }
    )
);

named!(
    pub block_labels(CompleteStr) -> Vec<BlockLabel>,
    many0!(
        inline_whitespace!(block_label)
    )
);

named!(
    pub one_line_block_body(CompleteStr) -> Option<Attribute>,
    opt!(attribute)
);

named!(
    pub one_line_block(CompleteStr) -> Block,
    inline_whitespace!(
        do_parse!(
            block_type: call!(identifier)
            >> labels: call!(block_labels)
            >> tag!("{")
            >> attribute: call!(one_line_block_body)
            >> tag!("}")
            >> (Block::new_one_line(block_type, labels, attribute))
        )
    )
);

named!(
    pub block(CompleteStr) -> Block,
    inline_whitespace!(
        do_parse!(
            block_type: call!(identifier)
            >> labels: call!(block_labels)
            >> tag!("{")
            >> newline
            >> body: call!(body)
            >> tag!("}")
            >> (Block::new(block_type, labels, body))
        )
    )
);

/// Blocks in a body indexed by their type and labels
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocks<'a> {
    blocks: HashMap<Identifier<'a>, BlockBody<'a>>,
}

impl<'a> Blocks<'a> {
    pub fn new<T>(blocks: T) -> Self
    where
        T: IntoIterator<Item = Block<'a>>,
    {
        let mut hashmap = HashMap::new();
        for (block_type, blocks) in blocks
            .into_iter()
            .group_by(|block| block.r#type.clone())
            .into_iter()
        {
            let blocks = BlockBody::from_iter(blocks.map(|block| (block.labels, block.body)));
            hashmap.insert(block_type, blocks);
        }
        Self { blocks: hashmap }
    }

    pub fn append(&mut self, block: Block<'a>) {
        match self.blocks.entry(block.r#type) {
            Entry::Vacant(vacant) => {
                let mut body = BlockBody::default();
                body.append(block.labels, block.body);
                vacant.insert(body);
            }
            Entry::Occupied(mut occupied) => {
                occupied.get_mut().append(block.labels, block.body);
            }
        }
    }

    pub fn get<S1, S2>(&self, block_type: S1, labels: &[S2]) -> Option<&BlockBody<'a>>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        match self.blocks.get(block_type.as_ref()) {
            None => None,
            Some(body) => body.get(labels),
        }
    }

    pub fn get_mut<S1, S2>(&mut self, block_type: S1, labels: &[S2]) -> Option<&mut BlockBody<'a>>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        match self.blocks.get_mut(block_type.as_ref()) {
            None => None,
            Some(body) => body.get_mut(labels),
        }
    }

    /// Get an iterator over the types of blocks
    pub fn iter(&self) -> hash_map::Iter<Identifier<'a>, BlockBody<'a>> {
        self.blocks.iter()
    }

    pub fn iter_mut(&mut self) -> hash_map::IterMut<Identifier<'a>, BlockBody<'a>> {
        self.blocks.iter_mut()
    }

    /// Returns a flattened iterator, yielding a three-tuple of the block type, block labels and the
    /// body of a block
    pub fn flat_iter<'b>(
        &'b self,
    ) -> impl Iterator<Item = (&'a str, VecDeque<&'a str>, &'b Body<'a>)> + 'b
    where
        'b: 'a,
    {
        self.blocks
            .iter()
            .map(|(block_type, blocks)| {
                let block_type: &str = block_type.borrow();
                blocks
                    .flat_iter()
                    .map(move |(labels, bodies)| (block_type, labels, bodies))
            })
            .flatten()
    }

    /// Returns a flattened mutable iterator, yielding a three-tuple of the block type, block
    /// labels and the body of a block.
    ///
    /// Only the body is mutable
    pub fn flat_iter_mut<'b>(
        &'b mut self,
    ) -> impl Iterator<Item = (&'a str, VecDeque<&'a str>, &'b mut Body<'a>)> + 'b
    where
        'b: 'a,
    {
        self.blocks
            .iter_mut()
            .map(|(block_type, blocks)| {
                let block_type: &str = block_type.borrow();
                blocks
                    .flat_iter_mut()
                    .map(move |(labels, bodies)| (block_type, labels, bodies))
            })
            .flatten()
    }

    /// Consumes self and return  a flattened iterator, yielding a three-tuple of the block type,
    /// block labels and the body of a block
    pub fn flat_into_iter(
        self,
    ) -> impl Iterator<Item = (Cow<'a, str>, VecDeque<Cow<'a, str>>, Body<'a>)> + 'a {
        self.blocks
            .into_iter()
            .map(|(block_type, blocks)| {
                blocks
                    .flat_into_iter()
                    .map(move |(labels, bodies)| (block_type.clone(), labels, bodies))
            })
            .flatten()
    }

    /// Top level length
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Whether the top level is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Total number of blocks recursively
    pub fn len_blocks(&self) -> usize {
        self.blocks
            .iter()
            .fold(0, |acc, (_, bodies)| bodies.len_blocks() + acc)
    }
}

impl<'a> IntoIterator for Blocks<'a> {
    type Item = (Identifier<'a>, BlockBody<'a>);
    type IntoIter = hash_map::IntoIter<Identifier<'a>, BlockBody<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.blocks.into_iter()
    }
}

impl<'a, 'b> IntoIterator for &'b Blocks<'a> {
    type Item = (&'b Identifier<'a>, &'b BlockBody<'a>);
    type IntoIter = hash_map::Iter<'b, Identifier<'a>, BlockBody<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'b> IntoIterator for &'b mut Blocks<'a> {
    type Item = (&'b Identifier<'a>, &'b mut BlockBody<'a>);
    type IntoIter = hash_map::IterMut<'b, Identifier<'a>, BlockBody<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a> FromIterator<Block<'a>> for Blocks<'a> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Block<'a>>,
    {
        Self::new(iter)
    }
}

impl<'a> Extend<Block<'a>> for Blocks<'a> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = Block<'a>>,
    {
        for block in iter {
            self.append(block)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockBody<'a> {
    Body(Vec<Body<'a>>),
    Labels {
        empty: Vec<Body<'a>>,
        labels: HashMap<BlockLabel<'a>, BlockBody<'a>>,
    },
}

impl<'a> BlockBody<'a> {
    pub fn append(&mut self, mut labels: Vec<BlockLabel<'a>>, body: Body<'a>) {
        match self {
            BlockBody::Body(ref mut bodies) => {
                if labels.is_empty() {
                    bodies.push(body);
                } else {
                    self.body_to_labels(labels, body);
                }
            }
            BlockBody::Labels {
                ref mut empty,
                labels: ref mut hashmap,
            } => {
                if labels.is_empty() {
                    empty.push(body);
                } else {
                    let label = labels.drain(0..1).next().expect("to be some");
                    match hashmap.entry(label) {
                        Entry::Vacant(vacant) => {
                            vacant.insert(BlockBody::Body(vec![body]));
                        }
                        Entry::Occupied(mut occupied) => {
                            occupied.get_mut().append(labels, body);
                        }
                    }
                };
            }
        }
    }

    pub fn get<S>(&self, labels: &[S]) -> Option<&Self>
    where
        S: AsRef<str>,
    {
        match labels.split_first() {
            None => Some(self),
            Some((first, rest)) => match self {
                BlockBody::Body(_) => None,
                BlockBody::Labels {
                    labels: ref hashmap,
                    ..
                } => match hashmap.get(first.as_ref()) {
                    None => None,
                    Some(inner) => inner.get(rest),
                },
            },
        }
    }

    pub fn get_mut<S>(&mut self, labels: &[S]) -> Option<&mut Self>
    where
        S: AsRef<str>,
    {
        match labels.split_first() {
            None => Some(self),
            Some((first, rest)) => match self {
                BlockBody::Body(_) => None,
                BlockBody::Labels {
                    labels: ref mut hashmap,
                    ..
                } => match hashmap.get_mut(first.as_ref()) {
                    None => None,
                    Some(inner) => inner.get_mut(rest),
                },
            },
        }
    }

    pub fn flat_iter<'b>(
        &'b self,
    ) -> Box<dyn Iterator<Item = (VecDeque<&'a str>, &'b Body<'a>)> + 'b>
    where
        'b: 'a,
    {
        match self {
            BlockBody::Body(ref bodies) => {
                Box::new(bodies.iter().map(|body| (VecDeque::new(), body)))
            }
            BlockBody::Labels {
                ref empty,
                ref labels,
            } => {
                let mut iterators: Vec<Box<dyn Iterator<Item = _>>> =
                    vec![Box::new(empty.iter().map(|body| (VecDeque::new(), body)))];
                for (label, nested) in labels.iter() {
                    let nested_iter = nested.flat_iter().map(move |(mut labels, body)| {
                        labels.push_front(label.as_str());
                        (labels, body)
                    });
                    iterators.push(Box::new(nested_iter));
                }
                Box::new(iterators.into_iter().flatten())
            }
        }
    }

    pub fn flat_iter_mut<'b>(
        &'b mut self,
    ) -> Box<dyn Iterator<Item = (VecDeque<&'a str>, &'b mut Body<'a>)> + 'b>
    where
        'b: 'a,
    {
        match self {
            BlockBody::Body(ref mut bodies) => {
                Box::new(bodies.iter_mut().map(|body| (VecDeque::new(), body)))
            }
            BlockBody::Labels {
                ref mut empty,
                ref mut labels,
            } => {
                let mut iterators: Vec<Box<dyn Iterator<Item = _>>> = vec![Box::new(
                    empty.iter_mut().map(|body| (VecDeque::new(), body)),
                )];
                for (label, nested) in labels.iter_mut() {
                    let nested_iter = nested.flat_iter_mut().map(move |(mut labels, body)| {
                        labels.push_front(label.as_str());
                        (labels, body)
                    });
                    iterators.push(Box::new(nested_iter));
                }
                Box::new(iterators.into_iter().flatten())
            }
        }
    }

    pub fn flat_into_iter(
        self,
    ) -> Box<dyn Iterator<Item = (VecDeque<Cow<'a, str>>, Body<'a>)> + 'a> {
        match self {
            BlockBody::Body(bodies) => {
                Box::new(bodies.into_iter().map(|body| (VecDeque::new(), body)))
            }
            BlockBody::Labels { empty, labels } => {
                let mut iterators: Vec<Box<dyn Iterator<Item = _>>> = vec![Box::new(
                    empty.into_iter().map(|body| (VecDeque::new(), body)),
                )];
                for (label, nested) in labels.into_iter() {
                    let nested_iter = nested.flat_into_iter().map(move |(mut labels, body)| {
                        labels.push_front(label.as_cow());
                        (labels, body)
                    });
                    iterators.push(Box::new(nested_iter));
                }
                Box::new(iterators.into_iter().flatten())
            }
        }
    }

    pub fn len_blocks(&self) -> usize {
        match self {
            BlockBody::Body(ref bodies) => bodies.len(),
            BlockBody::Labels {
                ref empty,
                ref labels,
            } => {
                empty.len()
                    + labels
                        .iter()
                        .fold(0, |acc, (_, bodies)| bodies.len_blocks() + acc)
            }
        }
    }

    /// Indicates that there are further labels for the block
    pub fn has_further_labels(&self) -> bool {
        if let BlockBody::Labels { .. } = self {
            true
        } else {
            false
        }
    }

    /// Borrow the bodies with no labels
    pub fn get_empty(&self) -> &[Body<'a>] {
        match self {
            BlockBody::Body(ref bodies) => bodies,
            BlockBody::Labels { ref empty, .. } => empty,
        }
    }

    /// Borrow the bodies with additional labels
    pub fn get_labels(&self) -> Option<&HashMap<BlockLabel<'a>, BlockBody<'a>>> {
        match self {
            BlockBody::Body(_) => None,
            BlockBody::Labels { ref labels, .. } => Some(labels),
        }
    }

    /// In place transmute of Body to Labels
    ///
    /// Must only be called when `labels` are not empty and the enum is of Body type
    /// Otherwise, this functon will panic
    fn body_to_labels(&mut self, mut labels: Vec<BlockLabel<'a>>, body: Body<'a>) {
        take_mut::take(self, move |current| {
            if let BlockBody::Body(bodies) = current {
                let label = labels.drain(0..1).next().expect("to be some");

                let mut hashmap = HashMap::new();
                let mut new_body = BlockBody::default();
                new_body.append(labels, body);
                hashmap.insert(label, new_body);
                BlockBody::Labels {
                    empty: bodies,
                    labels: hashmap,
                }
            } else {
                panic!("Unexpected enum variant")
            }
        });
    }
}

impl<'a> FromIterator<(Vec<BlockLabel<'a>>, Body<'a>)> for BlockBody<'a> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Vec<BlockLabel<'a>>, Body<'a>)>,
    {
        let mut results = Self::default();
        results.extend(iter);
        results
    }
}

impl<'a> Extend<(Vec<BlockLabel<'a>>, Body<'a>)> for BlockBody<'a> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (Vec<BlockLabel<'a>>, Body<'a>)>,
    {
        for (labels, body) in iter {
            self.append(labels, body);
        }
    }
}

impl<'a> Default for BlockBody<'a> {
    fn default() -> Self {
        BlockBody::Body(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parser::body::BodyElement;
    use crate::parser::expression::Expression;
    use crate::utils::ResultUtilsString;

    #[test]
    fn block_label_is_parsed_successfully() {
        let test_cases = [
            ("foobar", BlockLabel::from("foobar")),
            (
                r#""foobar""#,
                BlockLabel::StringLiteral(From::from("foobar")),
            ),
        ];

        for (input, expected_output) in &test_cases {
            let output = block_label(CompleteStr(input)).unwrap_output();
            assert_eq!(output, *expected_output);
        }
    }

    #[test]
    fn block_labels_are_parsed_successfully() {
        let test_cases = [
            ("", vec![]),
            ("foobar", vec![BlockLabel::from("foobar")]),
            (
                "foo bar baz",
                vec![
                    BlockLabel::from("foo"),
                    BlockLabel::from("bar"),
                    BlockLabel::from("baz"),
                ],
            ),
            (
                r#""foobar""#,
                vec![BlockLabel::StringLiteral(From::from("foobar"))],
            ),
            (
                "foo \"bar\" baz",
                vec![
                    BlockLabel::from("foo"),
                    BlockLabel::StringLiteral(From::from("bar")),
                    BlockLabel::from("baz"),
                ],
            ),
        ];

        for (input, expected_output) in &test_cases {
            let output = block_labels(CompleteStr(input)).unwrap_output();
            assert_eq!(output, *expected_output);
        }
    }

    #[test]
    fn single_line_block_body_is_parsed_correctly() {
        let test_cases = [
            ("", None),
            ("foo = true", Some((From::from("foo"), From::from(true)))),
        ];

        for (input, expected_output) in &test_cases {
            let output = one_line_block_body(CompleteStr(input)).unwrap_output();
            assert_eq!(output, *expected_output);
        }
    }

    #[test]
    fn single_line_block_is_parsed_correctly() {
        let test_cases = [
            (
                "test {}",
                Block::new_one_line(From::from("test"), vec![], None),
            ),
            (
                "test { foo = 123 }",
                Block::new_one_line(
                    From::from("test"),
                    vec![],
                    Some((From::from("foo"), From::from(123))),
                ),
            ),
            (
                "test foo bar baz {}",
                Block::new_one_line(
                    From::from("test"),
                    vec![
                        BlockLabel::from("foo"),
                        BlockLabel::from("bar"),
                        BlockLabel::from("baz"),
                    ],
                    None,
                ),
            ),
            (
                "test foo \"bar\" baz { foo = 123 }",
                Block::new_one_line(
                    From::from("test"),
                    vec![
                        BlockLabel::from("foo"),
                        BlockLabel::StringLiteral(From::from("bar")),
                        BlockLabel::from("baz"),
                    ],
                    Some((From::from("foo"), From::from(123))),
                ),
            ),
        ];

        for (input, expected_output) in &test_cases {
            let output = one_line_block(CompleteStr(input)).unwrap_output();
            assert_eq!(output, *expected_output);
        }
    }

    #[test]
    fn block_is_parsed_correctly() {
        let hcl = r#"simple_map "foo" bar {
  foo   = "bar"
  bar   = "baz"
  index = 0
}"#;
        let block = block(CompleteStr(hcl)).unwrap_output();

        let expected = Block::new(
            From::from("simple_map"),
            vec![
                BlockLabel::StringLiteral(From::from("foo")),
                BlockLabel::from("bar"),
            ],
            vec![
                From::from((From::from("foo"), Expression::from("bar"))),
                From::from((From::from("bar"), Expression::from("baz"))),
                From::from((From::from("index"), Expression::from(0))),
            ],
        );

        assert_eq!(block, expected);
    }

    #[test]
    fn nested_block_is_parsed_correctly() {
        let hcl = r#"resource "security/group" foobar {
  name = "foobar" # Comment

  allow {
    name = "localhost" // Seems pointless
    cidrs = ["127.0.0.1/32"]
  }

  allow {
    name = "lan" /* Is this all our LAN CIDR? */
    cidrs = ["192.168.0.0/16"]
  }

  deny {
    # Now this is pointless
    name = "internet"
    cidrs = ["0.0.0.0/0"]
  }
}"#;

        let block = block(CompleteStr(hcl)).unwrap_output();

        let expected = Block::new(
            From::from("resource"),
            vec![
                BlockLabel::StringLiteral(From::from("security/group")),
                BlockLabel::from("foobar"),
            ],
            vec![
                From::from((From::from("name"), Expression::from("foobar"))),
                BodyElement::Block(Block::new(
                    From::from("allow"),
                    vec![],
                    vec![
                        From::from((From::from("name"), Expression::from("localhost"))),
                        From::from((
                            From::from("cidrs"),
                            Expression::from(vec![From::from("127.0.0.1/32")]),
                        )),
                    ],
                )),
                BodyElement::Block(Block::new(
                    From::from("allow"),
                    vec![],
                    vec![
                        From::from((From::from("name"), Expression::from("lan"))),
                        From::from((
                            From::from("cidrs"),
                            Expression::from(vec![From::from("192.168.0.0/16")]),
                        )),
                    ],
                )),
                BodyElement::Block(Block::new(
                    From::from("deny"),
                    vec![],
                    vec![
                        From::from((From::from("name"), Expression::from("internet"))),
                        From::from((
                            From::from("cidrs"),
                            Expression::from(vec![From::from("0.0.0.0/0")]),
                        )),
                    ],
                )),
            ],
        );

        assert_eq!(block, expected);
    }

    fn repeat_blocks(n: usize) -> Blocks<'static> {
        let hcl: Vec<_> = std::iter::repeat("test { foo = 123 }").take(n).collect();
        let parsed: Vec<_> = hcl
            .iter()
            .map(|hcl| one_line_block(CompleteStr(hcl)).unwrap_output())
            .collect();
        Blocks::new(parsed)
    }

    #[test]
    fn blocks_with_no_labels_are_constructed_correctly() {
        const N: usize = 10;

        let blocks = repeat_blocks(N);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks.len_blocks(), N);

        let test = blocks.get::<_, &str>("test", &[]).unwrap();
        assert!(!test.has_further_labels())
    }

    #[test]
    fn multiple_blocks_with_no_labels_are_constructed_correctly() {
        const N: usize = 10;

        let mut counter = 0;
        let hcl: Vec<_> = std::iter::from_fn(move || {
            let hcl = format!("test_{} {{ foo = 123 }}", counter);
            counter += 1;
            Some(hcl)
        })
        .take(N)
        .collect();

        let parsed: Vec<_> = hcl
            .iter()
            .map(|hcl| one_line_block(CompleteStr(hcl)).unwrap_output())
            .collect();
        let blocks = Blocks::new(parsed);

        assert_eq!(blocks.len(), N);
        assert_eq!(blocks.len_blocks(), N);

        assert!(blocks
            .iter()
            .all(|(_type, bodies)| !bodies.has_further_labels()))
    }

    #[test]
    fn appending_block_with_labels_transforms_correctly() {
        const N: usize = 10;

        let mut blocks = repeat_blocks(N);
        let additional_block = std::iter::repeat(
            one_line_block(CompleteStr(r#"test "foo" { foo = true } "#)).unwrap_output(),
        );
        blocks.extend(additional_block.take(N));

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks.len_blocks(), N + N);

        let test = blocks.get::<_, &str>("test", &[]).unwrap();
        assert!(test.has_further_labels());

        let empty = test.get_empty();
        assert_eq!(empty.len(), N);

        let labels = test.get_labels().expect("to be some");
        assert_eq!(labels.len(), 1);

        #[allow(clippy::blacklisted_name)]
        let foo = labels.get("foo").expect("to be some");
        assert!(!foo.has_further_labels());
        assert_eq!(foo.len_blocks(), N);
    }

    #[test]
    fn appending_block_with_multiple_labels_transforms_correctly() {
        const N: usize = 10;

        let mut blocks = repeat_blocks(N);

        let mut counter = 0;
        let additional_block_hcl: Vec<_> = std::iter::from_fn(move || {
            let labels = std::iter::repeat("\"foobar\"")
                .take(counter % 2 + 1)
                .join(" ");
            let hcl = format!("test_{} {} {{ foo = true }}", counter, labels);
            counter += 1;
            Some(hcl)
        })
        .take(N)
        .collect();
        let additional_block = additional_block_hcl
            .iter()
            .map(|hcl| one_line_block(CompleteStr(&hcl)).unwrap_output());
        blocks.extend(additional_block);

        assert_eq!(blocks.len(), 1 + N);
        assert_eq!(blocks.len_blocks(), N + N);

        let test = blocks.get::<_, &str>("test", &[]).unwrap();
        assert!(!test.has_further_labels());

        let test_0 = blocks.get("test_0", &["foobar"]).unwrap();
        assert!(!test_0.has_further_labels());

        let test_1 = blocks.get("test_1", &["foobar"]).unwrap();
        assert!(test_1.has_further_labels());

        let test_1 = blocks.get("test_1", &["foobar", "foobar"]).unwrap();
        assert!(!test_1.has_further_labels());

        // Test the flattened iterator implementation
        for (block_type, labels, _body) in blocks.flat_iter() {
            if block_type == "test" {
                assert_eq!(0, labels.len());
            } else {
                assert!(!labels.is_empty());
            }
        }
    }
}
