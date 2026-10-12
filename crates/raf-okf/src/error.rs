// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The typed errors of the reader and of the value constructors.

use std::error::Error;
use std::fmt;

/// What went wrong while reading a document.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Line 1 is not a `---` fence.
    MissingFence,
    /// No closing `---` fence line follows the opening fence.
    UnclosedFence,
    /// The document text exceeds the size bound.
    TooLarge {
        /// Size of the offered text in bytes.
        len: usize,
        /// The bound in bytes.
        max: usize,
    },
    /// Frontmatter lines do not all end the way the opening fence line ends.
    MixedLineEndings,
    /// A control character appears inside a frontmatter line.
    ControlCharacter,
    /// A frontmatter line ends in spaces that no value or comment explains.
    TrailingWhitespace,
    /// Indentation is odd, deeper than a block allows, or not a nesting step.
    Indentation,
    /// A value is separated from its key by more than one space.
    Spacing,
    /// The frontmatter is not a block mapping.
    NotAMapping,
    /// A line is not `key: value` or `key:`.
    ExpectedKey,
    /// A block list holds a line that does not start with `- `.
    ExpectedItem,
    /// A key holds characters outside `A-Z a-z 0-9 _ -`.
    InvalidKey {
        /// The offending key text.
        key: String,
    },
    /// A key occurs twice in one mapping.
    DuplicateKey {
        /// The repeated key.
        key: String,
    },
    /// A `<<` merge key.
    MergeKey,
    /// An anchor (`&name`).
    Anchor {
        /// The anchor name, without the `&`.
        name: String,
    },
    /// An alias (`*name`).
    Alias {
        /// The alias name, without the `*`.
        name: String,
    },
    /// A tag (`!name`).
    Tag {
        /// The tag text, without the `!`.
        name: String,
    },
    /// A block scalar header such as `|` or `>-`.
    BlockScalar {
        /// The header text.
        header: String,
    },
    /// A plain value begins with a YAML indicator character.
    IndicatorCharacter {
        /// The indicator character.
        indicator: char,
    },
    /// A flow mapping (`{...}`) value.
    FlowMapping {
        /// The key whose value is the flow mapping.
        key: String,
    },
    /// A flow sequence inside a flow sequence or a list item.
    NestedFlow,
    /// A block list item that is itself a block list.
    NestedSequence,
    /// A flow sequence without its closing `]`.
    UnterminatedFlow,
    /// A flow sequence with an empty entry.
    EmptyFlowEntry,
    /// Text follows the closing `]` of a flow sequence.
    TextAfterFlow,
    /// A quoted value without its closing quote.
    UnterminatedQuote,
    /// Text follows the closing quote of a quoted value.
    TextAfterQuote,
    /// A backslash escape outside the shared escape set.
    UnknownEscape {
        /// The character after the backslash.
        escape: char,
    },
    /// A plain value that contains `: ` or ends in `:`.
    PlainColonSpace,
    /// A block list item with no value.
    EmptyItem,
    /// A block list mixes scalar items and mapping items.
    MixedList,
    /// Nesting is deeper than the bound.
    DepthExceeded {
        /// The bound in levels.
        max: usize,
    },
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFence => f.write_str("the document does not start with a `---` fence line"),
            Self::UnclosedFence => f.write_str("the frontmatter is not closed by a `---` line"),
            Self::TooLarge { len, max } => {
                write!(f, "the document is {len} bytes, over the {max} byte bound")
            }
            Self::MixedLineEndings => f.write_str(
                "the line ending differs from the one the opening fence uses; use one line ending",
            ),
            Self::ControlCharacter => f.write_str("a control character appears in the frontmatter"),
            Self::TrailingWhitespace => f.write_str("the line ends in spaces"),
            Self::Indentation => f.write_str(
                "indentation must be a multiple of two spaces and open a block only under a key with no value",
            ),
            Self::Spacing => f.write_str("exactly one space separates a key from its value"),
            Self::NotAMapping => f.write_str("the frontmatter is not a block mapping"),
            Self::ExpectedKey => f.write_str("expected `key: value` or `key:`"),
            Self::ExpectedItem => f.write_str("expected a `- ` list item"),
            Self::InvalidKey { key } => write!(
                f,
                "key `{key}` holds characters outside letters, digits, `_` and `-`"
            ),
            Self::DuplicateKey { key } => write!(f, "duplicate key `{key}`"),
            Self::MergeKey => f.write_str("merge key `<<` is not accepted"),
            Self::Anchor { name } => write!(f, "anchor `&{name}` is not accepted"),
            Self::Alias { name } => write!(f, "alias `*{name}` is not accepted"),
            Self::Tag { name } => write!(f, "tag `!{name}` is not accepted"),
            Self::BlockScalar { header } => {
                write!(f, "block scalar header `{header}` is not accepted")
            }
            Self::IndicatorCharacter { indicator } => write!(
                f,
                "a plain value cannot start with the YAML indicator character `{indicator}`; quote it"
            ),
            Self::FlowMapping { key } => write!(
                f,
                "the value of `{key}` is a flow mapping, which is not accepted; write a block mapping"
            ),
            Self::NestedFlow => f.write_str("a flow sequence holds only scalars and is not a list item"),
            Self::NestedSequence => f.write_str("a list item cannot be a list"),
            Self::UnterminatedFlow => f.write_str("flow sequence is not closed by `]`"),
            Self::EmptyFlowEntry => f.write_str("flow sequence has an empty entry"),
            Self::TextAfterFlow => f.write_str("text after the closing `]`"),
            Self::UnterminatedQuote => f.write_str("unterminated quoted value"),
            Self::TextAfterQuote => f.write_str("text after the closing quote"),
            Self::UnknownEscape { escape } => write!(
                f,
                "unknown escape `\\{escape}`; the escapes are `\\\\`, `\\\"`, `\\n`, `\\r`, `\\t` and `\\0`"
            ),
            Self::PlainColonSpace => {
                f.write_str("an unquoted value contains `: ` or ends in `:`; quote it")
            }
            Self::EmptyItem => f.write_str("a list item has no value"),
            Self::MixedList => f.write_str("a list mixes scalar items and mapping items"),
            Self::DepthExceeded { max } => write!(f, "nesting is deeper than {max} levels"),
        }
    }
}

/// A positioned reader error.
///
/// `line` is the 1-based line of the whole document text. `column` is the
/// 1-based character column when the error points inside a line, and `offset`
/// is the byte offset into the text when the error points at one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    kind: ErrorKind,
    line: usize,
    column: Option<usize>,
    offset: Option<usize>,
}

impl ParseError {
    pub(crate) fn at(kind: ErrorKind, line: usize, column: usize, offset: usize) -> Self {
        Self {
            kind,
            line,
            column: Some(column),
            offset: Some(offset),
        }
    }

    /// An error that points at a byte offset but not at a column.
    pub(crate) fn at_offset(kind: ErrorKind, line: usize, offset: usize) -> Self {
        Self {
            kind,
            line,
            column: None,
            offset: Some(offset),
        }
    }

    /// What went wrong.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// The 1-based line the error points at.
    pub fn line(&self) -> usize {
        self.line
    }

    /// The 1-based character column, when the error points inside a line.
    pub fn column(&self) -> Option<usize> {
        self.column
    }

    /// The byte offset into the text, when the error points at one.
    pub fn offset(&self) -> Option<usize> {
        self.offset
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.column {
            Some(column) => write!(f, "line {}, column {column}: {}", self.line, self.kind),
            None => write!(f, "line {}: {}", self.line, self.kind),
        }
    }
}

impl Error for ParseError {}

/// Why a value or document cannot be built from its parts.
///
/// The constructors refuse every value the reader would not read back, so a
/// document built from parts renders text that parses to an equal document.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConstructError {
    /// A scalar holds a control character that has no escape, so no style can
    /// write it.
    Unrepresentable {
        /// The character.
        character: char,
    },
    /// A key is empty or holds characters outside `A-Z a-z 0-9 _ -`.
    InvalidKey {
        /// The offending key text.
        key: String,
    },
    /// A key occurs twice in one mapping.
    DuplicateKey {
        /// The repeated key.
        key: String,
    },
    /// A mapping has no entries; the reader reads `key:` as an empty scalar.
    EmptyMap,
    /// A block list has no items; write a flow list `[]` instead.
    EmptyBlockList,
    /// A list item is an empty unquoted scalar.
    EmptyListItem,
    /// A flow list holds an unquoted scalar with a character the flow syntax
    /// reserves, or one the reader would read as something else.
    NotFlowSafe {
        /// The scalar text.
        text: String,
    },
    /// A block list mixes scalar items and mapping items.
    MixedList,
    /// A list item is itself a list.
    NestedSequence,
    /// Nesting is deeper than the bound.
    DepthExceeded {
        /// The bound in levels.
        max: usize,
    },
    /// The rendered text exceeds the size bound.
    TooLarge {
        /// Size of the rendered text in bytes.
        len: usize,
        /// The bound in bytes.
        max: usize,
    },
}

impl fmt::Display for ConstructError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unrepresentable { character } => write!(
                f,
                "the control character U+{:04X} has no escape, so no scalar style can write it",
                u32::from(*character)
            ),
            Self::InvalidKey { key } => write!(
                f,
                "key `{key}` is empty or holds characters outside letters, digits, `_` and `-`"
            ),
            Self::DuplicateKey { key } => write!(f, "duplicate key `{key}`"),
            Self::EmptyMap => f.write_str("a mapping needs at least one entry"),
            Self::EmptyBlockList => {
                f.write_str("a block list needs at least one item; use a flow list for none")
            }
            Self::EmptyListItem => f.write_str("a list item has no value"),
            Self::NotFlowSafe { text } => write!(
                f,
                "the unquoted scalar `{text}` cannot sit inside a flow list; quote it"
            ),
            Self::MixedList => f.write_str("a list mixes scalar items and mapping items"),
            Self::NestedSequence => f.write_str("a list item cannot be a list"),
            Self::DepthExceeded { max } => write!(f, "nesting is deeper than {max} levels"),
            Self::TooLarge { len, max } => {
                write!(f, "the document is {len} bytes, over the {max} byte bound")
            }
        }
    }
}

impl Error for ConstructError {}
