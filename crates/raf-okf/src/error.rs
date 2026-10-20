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
    /// Block quotes and list items are nested deeper than the bound.
    BlockNesting {
        /// The bound in levels.
        max: usize,
    },
    /// A line reads as a PROPOSED marker in some spelling but is not the
    /// marker `> [!PROPOSED change:<slug>]`.
    MalformedProposed,
    /// A line starts a generated marker but is not `<!-- raf:generated:<kind> -->`
    /// with a lowercase kebab-case kind, or the end marker.
    MalformedGenerated,
    /// A generated block opens inside another one.
    NestedGenerated {
        /// The kind of the block that is still open.
        outer: String,
    },
    /// A generated end marker with no block open.
    OrphanGeneratedEnd,
    /// A generated block that is never closed.
    UnclosedGenerated {
        /// The kind of the block.
        kind: String,
    },
    /// A comment that begins a claim marker in some spelling but is not
    /// `<!-- raf:claim evidence=<slug>[,<slug>...] -->` alone on its line.
    MalformedClaim,
    /// A claim marker that does not directly follow the paragraph or list
    /// item it supports: it comes first in the body, or after a blank line, a
    /// heading, a fence, a marker or a PROPOSED block.
    UnattachedClaim,
    /// A comment that begins a tested-by marker in some spelling but is not
    /// `<!-- raf:tested-by <path>[,<path>...] -->` alone on its line, with
    /// distinct paths of the `<item-path>` pattern.
    MalformedTestedBy,
    /// A tested-by marker that follows no statement: it comes first in the
    /// body, or after a blank line, a fence, a comment or a PROPOSED block,
    /// or it comes directly before a claim marker, whose place is between the
    /// statement and the tested-by marker.
    UnattachedTestedBy,
    /// A `<!-- raf:` comment that is neither a generated marker, a claim
    /// marker nor a tested-by marker.
    UnknownMarker,
    /// A link destination or title is longer than the bound.
    LinkPartTooLong {
        /// The bound in bytes.
        max: usize,
    },
    /// The header comment of an index holds more than SPDX tags, a tag has no
    /// value, or the comment sits after a block.
    IndexHeader,
    /// An index holds text outside its generated blocks.
    IndexText,
    /// A marker of an index is malformed, indented, or not at the start of
    /// its line.
    IndexMarker,
    /// An index holds no generated block.
    IndexEmpty,
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
            Self::BlockNesting { max } => {
                write!(f, "block quotes and list items are nested deeper than {max} levels")
            }
            Self::MalformedProposed => f.write_str(
                "malformed PROPOSED marker; write exactly `> [!PROPOSED change:<slug>]`",
            ),
            Self::MalformedGenerated => f.write_str(
                "malformed generated marker; write `<!-- raf:generated:<kind> -->` with a lowercase kebab-case kind, or `<!-- raf:generated:end -->`",
            ),
            Self::NestedGenerated { outer } => {
                write!(f, "a generated block opens inside the open block `{outer}`")
            }
            Self::OrphanGeneratedEnd => {
                f.write_str("generated end marker without a start")
            }
            Self::UnclosedGenerated { kind } => {
                write!(f, "the generated block `{kind}` is never closed")
            }
            Self::MalformedClaim => f.write_str(
                "malformed claim marker; write exactly `<!-- raf:claim evidence=<slug>[,<slug>...] -->` on its own line",
            ),
            Self::UnattachedClaim => f.write_str(
                "a claim marker follows directly the paragraph or list item it supports, with no blank line between",
            ),
            Self::MalformedTestedBy => f.write_str(
                "malformed tested-by marker; write exactly `<!-- raf:tested-by <path>[,<path>...] -->` on its own line, the paths distinct and separated by `,` with no space",
            ),
            Self::UnattachedTestedBy => f.write_str(
                "a tested-by marker follows directly the paragraph, list item, heading or claim marker of the statement it maps, with no blank line between, and comes after the claim marker",
            ),
            Self::UnknownMarker => f.write_str(
                "unknown `raf:` marker; the markers are `raf:generated:<kind>`, `raf:claim` and `raf:tested-by`",
            ),
            Self::LinkPartTooLong { max } => {
                write!(f, "a link destination or title is longer than {max} bytes")
            }
            Self::IndexHeader => f.write_str(
                "the header comment must hold only SPDX tags with a value, before the first block",
            ),
            Self::IndexText => f.write_str("text outside a generated block"),
            Self::IndexMarker => f.write_str(
                "malformed or misplaced marker; a marker starts its own line with no indentation",
            ),
            Self::IndexEmpty => f.write_str("the index holds no generated block"),
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
    /// The body does not read: a malformed marker or nesting over the bound.
    /// The line and offset are in the text the document renders to.
    Body(ParseError),
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
            Self::Body(error) => write!(f, "the body does not read: {error}"),
        }
    }
}

impl Error for ConstructError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::Body(error) = self {
            Some(error)
        } else {
            None
        }
    }
}

/// Why a batch of span edits cannot be applied to a body.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SpliceError {
    /// A span reaches outside the body.
    OutOfBody {
        /// Start of the offered span, in whole-text bytes.
        start: usize,
        /// End of the offered span, in whole-text bytes.
        end: usize,
    },
    /// A span starts or ends inside a character.
    NotCharBoundary {
        /// The offending offset, in whole-text bytes.
        offset: usize,
    },
    /// Two spans share bytes.
    Overlap {
        /// The start of the later span, in whole-text bytes.
        offset: usize,
    },
    /// The edited text does not read as a body.
    Reparse(ParseError),
}

impl fmt::Display for SpliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBody { start, end } => {
                write!(f, "span {start}..{end} reaches outside the body")
            }
            Self::NotCharBoundary { offset } => {
                write!(f, "offset {offset} is inside a character")
            }
            Self::Overlap { offset } => {
                write!(f, "the span at offset {offset} overlaps the one before it")
            }
            Self::Reparse(error) => write!(f, "the edited body does not parse: {error}"),
        }
    }
}

impl Error for SpliceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Reparse(error) => Some(error),
            Self::OutOfBody { .. } | Self::NotCharBoundary { .. } | Self::Overlap { .. } => None,
        }
    }
}

/// The kind of a reader error, without its data.
///
/// One variant per variant of [`ErrorKind`]. The set is closed on purpose: a
/// `match` over `ErrorTag` is exhaustive, so an error kind added to
/// [`ErrorKind`] adds a variant here and breaks every match that must name them
/// all. [`ErrorKind::ALL`] lists every tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ErrorTag {
    /// [`ErrorKind::MissingFence`].
    MissingFence,
    /// [`ErrorKind::UnclosedFence`].
    UnclosedFence,
    /// [`ErrorKind::TooLarge`].
    TooLarge,
    /// [`ErrorKind::MixedLineEndings`].
    MixedLineEndings,
    /// [`ErrorKind::ControlCharacter`].
    ControlCharacter,
    /// [`ErrorKind::TrailingWhitespace`].
    TrailingWhitespace,
    /// [`ErrorKind::Indentation`].
    Indentation,
    /// [`ErrorKind::Spacing`].
    Spacing,
    /// [`ErrorKind::NotAMapping`].
    NotAMapping,
    /// [`ErrorKind::ExpectedKey`].
    ExpectedKey,
    /// [`ErrorKind::ExpectedItem`].
    ExpectedItem,
    /// [`ErrorKind::InvalidKey`].
    InvalidKey,
    /// [`ErrorKind::DuplicateKey`].
    DuplicateKey,
    /// [`ErrorKind::MergeKey`].
    MergeKey,
    /// [`ErrorKind::Anchor`].
    Anchor,
    /// [`ErrorKind::Alias`].
    Alias,
    /// [`ErrorKind::Tag`].
    Tag,
    /// [`ErrorKind::BlockScalar`].
    BlockScalar,
    /// [`ErrorKind::IndicatorCharacter`].
    IndicatorCharacter,
    /// [`ErrorKind::FlowMapping`].
    FlowMapping,
    /// [`ErrorKind::NestedFlow`].
    NestedFlow,
    /// [`ErrorKind::NestedSequence`].
    NestedSequence,
    /// [`ErrorKind::UnterminatedFlow`].
    UnterminatedFlow,
    /// [`ErrorKind::EmptyFlowEntry`].
    EmptyFlowEntry,
    /// [`ErrorKind::TextAfterFlow`].
    TextAfterFlow,
    /// [`ErrorKind::UnterminatedQuote`].
    UnterminatedQuote,
    /// [`ErrorKind::TextAfterQuote`].
    TextAfterQuote,
    /// [`ErrorKind::UnknownEscape`].
    UnknownEscape,
    /// [`ErrorKind::PlainColonSpace`].
    PlainColonSpace,
    /// [`ErrorKind::EmptyItem`].
    EmptyItem,
    /// [`ErrorKind::MixedList`].
    MixedList,
    /// [`ErrorKind::DepthExceeded`].
    DepthExceeded,
    /// [`ErrorKind::BlockNesting`].
    BlockNesting,
    /// [`ErrorKind::MalformedProposed`].
    MalformedProposed,
    /// [`ErrorKind::MalformedGenerated`].
    MalformedGenerated,
    /// [`ErrorKind::NestedGenerated`].
    NestedGenerated,
    /// [`ErrorKind::OrphanGeneratedEnd`].
    OrphanGeneratedEnd,
    /// [`ErrorKind::UnclosedGenerated`].
    UnclosedGenerated,
    /// [`ErrorKind::MalformedClaim`].
    MalformedClaim,
    /// [`ErrorKind::UnattachedClaim`].
    UnattachedClaim,
    /// [`ErrorKind::MalformedTestedBy`].
    MalformedTestedBy,
    /// [`ErrorKind::UnattachedTestedBy`].
    UnattachedTestedBy,
    /// [`ErrorKind::UnknownMarker`].
    UnknownMarker,
    /// [`ErrorKind::LinkPartTooLong`].
    LinkPartTooLong,
    /// [`ErrorKind::IndexHeader`].
    IndexHeader,
    /// [`ErrorKind::IndexText`].
    IndexText,
    /// [`ErrorKind::IndexMarker`].
    IndexMarker,
    /// [`ErrorKind::IndexEmpty`].
    IndexEmpty,
}

impl ErrorTag {
    /// The name of the tag: the variant in lowercase kebab-case.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::MissingFence => "missing-fence",
            Self::UnclosedFence => "unclosed-fence",
            Self::TooLarge => "too-large",
            Self::MixedLineEndings => "mixed-line-endings",
            Self::ControlCharacter => "control-character",
            Self::TrailingWhitespace => "trailing-whitespace",
            Self::Indentation => "indentation",
            Self::Spacing => "spacing",
            Self::NotAMapping => "not-a-mapping",
            Self::ExpectedKey => "expected-key",
            Self::ExpectedItem => "expected-item",
            Self::InvalidKey => "invalid-key",
            Self::DuplicateKey => "duplicate-key",
            Self::MergeKey => "merge-key",
            Self::Anchor => "anchor",
            Self::Alias => "alias",
            Self::Tag => "tag",
            Self::BlockScalar => "block-scalar",
            Self::IndicatorCharacter => "indicator-character",
            Self::FlowMapping => "flow-mapping",
            Self::NestedFlow => "nested-flow",
            Self::NestedSequence => "nested-sequence",
            Self::UnterminatedFlow => "unterminated-flow",
            Self::EmptyFlowEntry => "empty-flow-entry",
            Self::TextAfterFlow => "text-after-flow",
            Self::UnterminatedQuote => "unterminated-quote",
            Self::TextAfterQuote => "text-after-quote",
            Self::UnknownEscape => "unknown-escape",
            Self::PlainColonSpace => "plain-colon-space",
            Self::EmptyItem => "empty-item",
            Self::MixedList => "mixed-list",
            Self::DepthExceeded => "depth-exceeded",
            Self::BlockNesting => "block-nesting",
            Self::MalformedProposed => "malformed-proposed",
            Self::MalformedGenerated => "malformed-generated",
            Self::NestedGenerated => "nested-generated",
            Self::OrphanGeneratedEnd => "orphan-generated-end",
            Self::UnclosedGenerated => "unclosed-generated",
            Self::MalformedClaim => "malformed-claim",
            Self::UnattachedClaim => "unattached-claim",
            Self::MalformedTestedBy => "malformed-tested-by",
            Self::UnattachedTestedBy => "unattached-tested-by",
            Self::UnknownMarker => "unknown-marker",
            Self::LinkPartTooLong => "link-part-too-long",
            Self::IndexHeader => "index-header",
            Self::IndexText => "index-text",
            Self::IndexMarker => "index-marker",
            Self::IndexEmpty => "index-empty",
        }
    }
}

impl fmt::Display for ErrorTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl ErrorKind {
    /// Every tag, in the order of the variants of [`ErrorKind`].
    pub const ALL: [ErrorTag; 48] = [
        ErrorTag::MissingFence,
        ErrorTag::UnclosedFence,
        ErrorTag::TooLarge,
        ErrorTag::MixedLineEndings,
        ErrorTag::ControlCharacter,
        ErrorTag::TrailingWhitespace,
        ErrorTag::Indentation,
        ErrorTag::Spacing,
        ErrorTag::NotAMapping,
        ErrorTag::ExpectedKey,
        ErrorTag::ExpectedItem,
        ErrorTag::InvalidKey,
        ErrorTag::DuplicateKey,
        ErrorTag::MergeKey,
        ErrorTag::Anchor,
        ErrorTag::Alias,
        ErrorTag::Tag,
        ErrorTag::BlockScalar,
        ErrorTag::IndicatorCharacter,
        ErrorTag::FlowMapping,
        ErrorTag::NestedFlow,
        ErrorTag::NestedSequence,
        ErrorTag::UnterminatedFlow,
        ErrorTag::EmptyFlowEntry,
        ErrorTag::TextAfterFlow,
        ErrorTag::UnterminatedQuote,
        ErrorTag::TextAfterQuote,
        ErrorTag::UnknownEscape,
        ErrorTag::PlainColonSpace,
        ErrorTag::EmptyItem,
        ErrorTag::MixedList,
        ErrorTag::DepthExceeded,
        ErrorTag::BlockNesting,
        ErrorTag::MalformedProposed,
        ErrorTag::MalformedGenerated,
        ErrorTag::NestedGenerated,
        ErrorTag::OrphanGeneratedEnd,
        ErrorTag::UnclosedGenerated,
        ErrorTag::MalformedClaim,
        ErrorTag::UnattachedClaim,
        ErrorTag::MalformedTestedBy,
        ErrorTag::UnattachedTestedBy,
        ErrorTag::UnknownMarker,
        ErrorTag::LinkPartTooLong,
        ErrorTag::IndexHeader,
        ErrorTag::IndexText,
        ErrorTag::IndexMarker,
        ErrorTag::IndexEmpty,
    ];

    /// The tag of this value, which drops its data.
    #[must_use]
    pub fn tag(&self) -> ErrorTag {
        match self {
            Self::MissingFence => ErrorTag::MissingFence,
            Self::UnclosedFence => ErrorTag::UnclosedFence,
            Self::TooLarge { .. } => ErrorTag::TooLarge,
            Self::MixedLineEndings => ErrorTag::MixedLineEndings,
            Self::ControlCharacter => ErrorTag::ControlCharacter,
            Self::TrailingWhitespace => ErrorTag::TrailingWhitespace,
            Self::Indentation => ErrorTag::Indentation,
            Self::Spacing => ErrorTag::Spacing,
            Self::NotAMapping => ErrorTag::NotAMapping,
            Self::ExpectedKey => ErrorTag::ExpectedKey,
            Self::ExpectedItem => ErrorTag::ExpectedItem,
            Self::InvalidKey { .. } => ErrorTag::InvalidKey,
            Self::DuplicateKey { .. } => ErrorTag::DuplicateKey,
            Self::MergeKey => ErrorTag::MergeKey,
            Self::Anchor { .. } => ErrorTag::Anchor,
            Self::Alias { .. } => ErrorTag::Alias,
            Self::Tag { .. } => ErrorTag::Tag,
            Self::BlockScalar { .. } => ErrorTag::BlockScalar,
            Self::IndicatorCharacter { .. } => ErrorTag::IndicatorCharacter,
            Self::FlowMapping { .. } => ErrorTag::FlowMapping,
            Self::NestedFlow => ErrorTag::NestedFlow,
            Self::NestedSequence => ErrorTag::NestedSequence,
            Self::UnterminatedFlow => ErrorTag::UnterminatedFlow,
            Self::EmptyFlowEntry => ErrorTag::EmptyFlowEntry,
            Self::TextAfterFlow => ErrorTag::TextAfterFlow,
            Self::UnterminatedQuote => ErrorTag::UnterminatedQuote,
            Self::TextAfterQuote => ErrorTag::TextAfterQuote,
            Self::UnknownEscape { .. } => ErrorTag::UnknownEscape,
            Self::PlainColonSpace => ErrorTag::PlainColonSpace,
            Self::EmptyItem => ErrorTag::EmptyItem,
            Self::MixedList => ErrorTag::MixedList,
            Self::DepthExceeded { .. } => ErrorTag::DepthExceeded,
            Self::BlockNesting { .. } => ErrorTag::BlockNesting,
            Self::MalformedProposed => ErrorTag::MalformedProposed,
            Self::MalformedGenerated => ErrorTag::MalformedGenerated,
            Self::NestedGenerated { .. } => ErrorTag::NestedGenerated,
            Self::OrphanGeneratedEnd => ErrorTag::OrphanGeneratedEnd,
            Self::UnclosedGenerated { .. } => ErrorTag::UnclosedGenerated,
            Self::MalformedClaim => ErrorTag::MalformedClaim,
            Self::UnattachedClaim => ErrorTag::UnattachedClaim,
            Self::MalformedTestedBy => ErrorTag::MalformedTestedBy,
            Self::UnattachedTestedBy => ErrorTag::UnattachedTestedBy,
            Self::UnknownMarker => ErrorTag::UnknownMarker,
            Self::LinkPartTooLong { .. } => ErrorTag::LinkPartTooLong,
            Self::IndexHeader => ErrorTag::IndexHeader,
            Self::IndexText => ErrorTag::IndexText,
            Self::IndexMarker => ErrorTag::IndexMarker,
            Self::IndexEmpty => ErrorTag::IndexEmpty,
        }
    }
}

#[cfg(test)]
mod tag_tests {
    use super::{ErrorKind, ErrorTag};

    /// A kind of each tag. The match names every tag, so a tag added to
    /// `ErrorTag` stops this from compiling until it has a sample.
    fn sample(tag: ErrorTag) -> ErrorKind {
        match tag {
            ErrorTag::MissingFence => ErrorKind::MissingFence,
            ErrorTag::UnclosedFence => ErrorKind::UnclosedFence,
            ErrorTag::TooLarge => ErrorKind::TooLarge { len: 0, max: 0 },
            ErrorTag::MixedLineEndings => ErrorKind::MixedLineEndings,
            ErrorTag::ControlCharacter => ErrorKind::ControlCharacter,
            ErrorTag::TrailingWhitespace => ErrorKind::TrailingWhitespace,
            ErrorTag::Indentation => ErrorKind::Indentation,
            ErrorTag::Spacing => ErrorKind::Spacing,
            ErrorTag::NotAMapping => ErrorKind::NotAMapping,
            ErrorTag::ExpectedKey => ErrorKind::ExpectedKey,
            ErrorTag::ExpectedItem => ErrorKind::ExpectedItem,
            ErrorTag::InvalidKey => ErrorKind::InvalidKey { key: String::new() },
            ErrorTag::DuplicateKey => ErrorKind::DuplicateKey { key: String::new() },
            ErrorTag::MergeKey => ErrorKind::MergeKey,
            ErrorTag::Anchor => ErrorKind::Anchor {
                name: String::new(),
            },
            ErrorTag::Alias => ErrorKind::Alias {
                name: String::new(),
            },
            ErrorTag::Tag => ErrorKind::Tag {
                name: String::new(),
            },
            ErrorTag::BlockScalar => ErrorKind::BlockScalar {
                header: String::new(),
            },
            ErrorTag::IndicatorCharacter => ErrorKind::IndicatorCharacter { indicator: '&' },
            ErrorTag::FlowMapping => ErrorKind::FlowMapping { key: String::new() },
            ErrorTag::NestedFlow => ErrorKind::NestedFlow,
            ErrorTag::NestedSequence => ErrorKind::NestedSequence,
            ErrorTag::UnterminatedFlow => ErrorKind::UnterminatedFlow,
            ErrorTag::EmptyFlowEntry => ErrorKind::EmptyFlowEntry,
            ErrorTag::TextAfterFlow => ErrorKind::TextAfterFlow,
            ErrorTag::UnterminatedQuote => ErrorKind::UnterminatedQuote,
            ErrorTag::TextAfterQuote => ErrorKind::TextAfterQuote,
            ErrorTag::UnknownEscape => ErrorKind::UnknownEscape { escape: 'q' },
            ErrorTag::PlainColonSpace => ErrorKind::PlainColonSpace,
            ErrorTag::EmptyItem => ErrorKind::EmptyItem,
            ErrorTag::MixedList => ErrorKind::MixedList,
            ErrorTag::DepthExceeded => ErrorKind::DepthExceeded { max: 0 },
            ErrorTag::BlockNesting => ErrorKind::BlockNesting { max: 0 },
            ErrorTag::MalformedProposed => ErrorKind::MalformedProposed,
            ErrorTag::MalformedGenerated => ErrorKind::MalformedGenerated,
            ErrorTag::NestedGenerated => ErrorKind::NestedGenerated {
                outer: String::new(),
            },
            ErrorTag::OrphanGeneratedEnd => ErrorKind::OrphanGeneratedEnd,
            ErrorTag::UnclosedGenerated => ErrorKind::UnclosedGenerated {
                kind: String::new(),
            },
            ErrorTag::MalformedClaim => ErrorKind::MalformedClaim,
            ErrorTag::UnattachedClaim => ErrorKind::UnattachedClaim,
            ErrorTag::MalformedTestedBy => ErrorKind::MalformedTestedBy,
            ErrorTag::UnattachedTestedBy => ErrorKind::UnattachedTestedBy,
            ErrorTag::UnknownMarker => ErrorKind::UnknownMarker,
            ErrorTag::LinkPartTooLong => ErrorKind::LinkPartTooLong { max: 0 },
            ErrorTag::IndexHeader => ErrorKind::IndexHeader,
            ErrorTag::IndexText => ErrorKind::IndexText,
            ErrorTag::IndexMarker => ErrorKind::IndexMarker,
            ErrorTag::IndexEmpty => ErrorKind::IndexEmpty,
        }
    }

    /// The position of a kind among the variants of `ErrorKind`. The match
    /// names every variant, so a variant added to `ErrorKind` stops this from
    /// compiling until it is placed here and listed in `ErrorKind::ALL`.
    fn position(kind: &ErrorKind) -> usize {
        match kind {
            ErrorKind::MissingFence => 0,
            ErrorKind::UnclosedFence => 1,
            ErrorKind::TooLarge { .. } => 2,
            ErrorKind::MixedLineEndings => 3,
            ErrorKind::ControlCharacter => 4,
            ErrorKind::TrailingWhitespace => 5,
            ErrorKind::Indentation => 6,
            ErrorKind::Spacing => 7,
            ErrorKind::NotAMapping => 8,
            ErrorKind::ExpectedKey => 9,
            ErrorKind::ExpectedItem => 10,
            ErrorKind::InvalidKey { .. } => 11,
            ErrorKind::DuplicateKey { .. } => 12,
            ErrorKind::MergeKey => 13,
            ErrorKind::Anchor { .. } => 14,
            ErrorKind::Alias { .. } => 15,
            ErrorKind::Tag { .. } => 16,
            ErrorKind::BlockScalar { .. } => 17,
            ErrorKind::IndicatorCharacter { .. } => 18,
            ErrorKind::FlowMapping { .. } => 19,
            ErrorKind::NestedFlow => 20,
            ErrorKind::NestedSequence => 21,
            ErrorKind::UnterminatedFlow => 22,
            ErrorKind::EmptyFlowEntry => 23,
            ErrorKind::TextAfterFlow => 24,
            ErrorKind::UnterminatedQuote => 25,
            ErrorKind::TextAfterQuote => 26,
            ErrorKind::UnknownEscape { .. } => 27,
            ErrorKind::PlainColonSpace => 28,
            ErrorKind::EmptyItem => 29,
            ErrorKind::MixedList => 30,
            ErrorKind::DepthExceeded { .. } => 31,
            ErrorKind::BlockNesting { .. } => 32,
            ErrorKind::MalformedProposed => 33,
            ErrorKind::MalformedGenerated => 34,
            ErrorKind::NestedGenerated { .. } => 35,
            ErrorKind::OrphanGeneratedEnd => 36,
            ErrorKind::UnclosedGenerated { .. } => 37,
            ErrorKind::MalformedClaim => 38,
            ErrorKind::UnattachedClaim => 39,
            ErrorKind::MalformedTestedBy => 40,
            ErrorKind::UnattachedTestedBy => 41,
            ErrorKind::UnknownMarker => 42,
            ErrorKind::LinkPartTooLong { .. } => 43,
            ErrorKind::IndexHeader => 44,
            ErrorKind::IndexText => 45,
            ErrorKind::IndexMarker => 46,
            ErrorKind::IndexEmpty => 47,
        }
    }

    /// How many variants `ErrorKind` has.
    const VARIANTS: usize = 48;

    #[test]
    fn all_lists_every_error_kind_once_in_variant_order() {
        assert_eq!(ErrorKind::ALL.len(), VARIANTS);
        for (index, tag) in ErrorKind::ALL.into_iter().enumerate() {
            let kind = sample(tag);
            assert_eq!(kind.tag(), tag, "{tag}");
            assert_eq!(
                position(&kind),
                index,
                "{tag} is out of order in ErrorKind::ALL"
            );
        }
    }

    #[test]
    fn names_are_unique_lowercase_kebab_case() {
        let mut names: Vec<&str> = ErrorKind::ALL.iter().map(|tag| tag.name()).collect();
        for name in &names {
            assert!(
                !name.is_empty()
                    && name.split('-').all(
                        |part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase())
                    ),
                "{name}"
            );
        }
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), VARIANTS);
    }
}
