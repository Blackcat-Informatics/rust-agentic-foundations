// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The body of a document: the Markdown after the frontmatter, and the model
//! tools query instead of re-scanning the text.

mod blocks;
mod heading;
mod link;
mod marker;
mod prose;

use std::sync::Arc;

use crate::error::{ErrorKind, ParseError, SpliceError};
use crate::frontmatter::reader::locate_fences;
use crate::limits::MAX_TEXT_BYTES;
use crate::span::Span;

pub(crate) use blocks::Origin;
pub(crate) use blocks::remove_code_spans;
use blocks::{is_blank, remove_comments};
pub use heading::{Heading, Section};
pub use link::{Link, LinkError, LinkForm, LinkTarget, SplitLink};
pub use marker::{Claim, GeneratedBlock, ProposedBlock, is_kebab};
pub(crate) use marker::{GENERATED_END, GENERATED_OPEN};
pub use prose::ProseLine;

/// A fenced code block.
#[derive(Clone)]
pub struct FencedBlock {
    span: Span,
    info: String,
    /// The text of the whole body, shared with the [`Body`] and every other block.
    text: Arc<str>,
    /// The byte range of the content in `text`.
    content: std::ops::Range<usize>,
    closed: bool,
}

// The shared body text is left out of `Debug` and equality: it is the same
// text for every block of a body, so printing or comparing it per block would
// cost the size of the body each time, and two blocks with the same fences,
// content and position are the same block whatever text surrounds them.
impl std::fmt::Debug for FencedBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FencedBlock")
            .field("span", &self.span)
            .field("info", &self.info)
            .field("content", &self.content())
            .field("closed", &self.closed)
            .finish_non_exhaustive()
    }
}

impl PartialEq for FencedBlock {
    fn eq(&self, other: &Self) -> bool {
        self.span == other.span
            && self.info == other.info
            && self.closed == other.closed
            && self.content() == other.content()
    }
}

impl Eq for FencedBlock {}

impl FencedBlock {
    /// The fence lines and everything between them, in whole-text
    /// coordinates. A block that is never closed runs to the end of the body.
    #[must_use]
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// The info string after the opening fence.
    #[must_use]
    pub fn info(&self) -> &str {
        &self.info
    }

    /// The text between the fence lines, exactly as written: every line
    /// between them with its line ending, and nothing of the fence lines. A
    /// block with nothing between its fences has empty content, and a block
    /// that is never closed runs to the end of the body. Inside a block quote
    /// or a list item the quote and list prefixes of the lines are part of
    /// the content. The result borrows from the body text the block shares
    /// with its [`Body`], so reading it allocates nothing.
    ///
    /// ```rust
    /// use raf_okf::Body;
    ///
    /// let body = Body::parse("~~~rust\nlet x = 1;\n~~~\n").expect("a body");
    /// let block = &body.fenced_blocks()[0];
    /// assert_eq!(block.info(), "rust");
    /// assert_eq!(block.content(), "let x = 1;\n");
    /// ```
    #[must_use]
    pub fn content(&self) -> &str {
        &self.text[self.content.clone()]
    }

    /// Whether a closing fence line ends the block. A block that runs to the
    /// end of the body without one is not closed.
    ///
    /// ```rust
    /// use raf_okf::Body;
    ///
    /// let closed = Body::parse("```\ncode\n```\n").expect("a body");
    /// assert!(closed.fenced_blocks()[0].is_closed());
    /// let open = Body::parse("```\ncode\n").expect("a body");
    /// assert!(!open.fenced_blocks()[0].is_closed());
    /// assert_eq!(open.fenced_blocks()[0].content(), "code\n");
    /// ```
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

#[derive(Debug, Clone)]
struct Model {
    headings: Vec<Heading>,
    sections: Vec<Section>,
    links: Vec<Link>,
    split_links: Vec<SplitLink>,
    fences: Vec<FencedBlock>,
    generated: Vec<GeneratedBlock>,
    proposed: Vec<ProposedBlock>,
    claims: Vec<Claim>,
    prose: Vec<ProseLine>,
}

/// The body of a document.
///
/// The body is kept byte for byte, and its model is read from it once.
/// Every [`Span`] the model returns is in the coordinates of the whole text:
/// the text that was parsed, or for a body inside a document that was built
/// from parts, the text [`crate::Document::render`] returns.
#[derive(Debug, Clone)]
pub struct Body {
    text: Arc<str>,
    span: Span,
    origin: Origin,
    model: Model,
}

impl Body {
    /// Read the body model of `text`, which is a Markdown file with or without
    /// frontmatter. When `text` starts with a `---` fence line that a later
    /// `---` line closes, the frontmatter is skipped (it is read by
    /// [`crate::Document::parse`]) and the body is what follows; otherwise the
    /// whole text is the body.
    ///
    /// # Errors
    ///
    /// A [`ParseError`] with the line of the offence: a text over the size
    /// bound, a malformed PROPOSED marker, a malformed, nested, unclosed or
    /// orphaned generated marker, a malformed or unattached claim marker, a
    /// `<!-- raf:` comment that is no marker, block quotes and list items
    /// nested over 64 levels, or a link destination or title over 4096 bytes.
    pub fn parse(text: &str) -> Result<Self, ParseError> {
        let (start, line) = match locate_fences(text) {
            Ok(fences) => (fences.body_start, fences.fence_line + 1),
            Err(error)
                if matches!(
                    error.kind(),
                    ErrorKind::MissingFence | ErrorKind::UnclosedFence
                ) =>
            {
                (0, 1)
            }
            Err(error) => return Err(error),
        };
        Self::parse_at(&text[start..], Origin { byte: start, line })
    }

    /// Read the model of a body that starts at `origin` in its whole text.
    pub(crate) fn parse_at(body: &str, origin: Origin) -> Result<Self, ParseError> {
        if origin.byte + body.len() > MAX_TEXT_BYTES {
            return Err(ParseError::at_offset(
                ErrorKind::TooLarge {
                    len: origin.byte + body.len(),
                    max: MAX_TEXT_BYTES,
                },
                origin.line,
                MAX_TEXT_BYTES,
            ));
        }
        let scan = blocks::scan(body, origin)?;
        let lines = &scan.lines;
        let mut visible: Vec<String> = Vec::with_capacity(lines.len());
        let mut in_comment = false;
        for line in lines {
            if line.fenced {
                visible.push(String::new());
            } else {
                visible.push(remove_comments(
                    &body[line.start..line.end],
                    &mut in_comment,
                ));
            }
        }
        let (headings, sections) = heading::read_headings(lines, &visible, body.len(), origin);
        // One shared slug per section: a line holds a pointer to it, so the
        // memory of the line table does not grow with the heading's length.
        let mut section_of: Vec<Option<Arc<str>>> = vec![None; lines.len()];
        for section in &sections {
            let first = section.line() - origin.line;
            let end = section.content().lines.end - origin.line;
            for slot in &mut section_of[first..end] {
                *slot = Some(section.shared_slug());
            }
        }
        let markers = marker::scan_markers(body, lines, &visible, origin, &section_of)?;
        let views: Vec<link::LineView<'_>> = lines
            .iter()
            .zip(&visible)
            .map(|(line, visible)| link::LineView {
                fenced: line.fenced,
                blank: is_blank(&body[line.start..line.end]),
                heading: !line.fenced && heading::atx(visible).is_some(),
                visible,
            })
            .collect();
        let found =
            link::scan_links(&views, origin.line, remove_code_spans).map_err(|too_long| {
                ParseError::at(
                    ErrorKind::LinkPartTooLong {
                        max: link::MAX_LINK_PART,
                    },
                    too_long.line,
                    1,
                    origin.byte,
                )
            })?;
        let mut prose = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            if line.fenced {
                continue;
            }
            let text = remove_code_spans(&visible[index]);
            if text.trim().is_empty() {
                continue;
            }
            prose.push(ProseLine {
                line: origin.line + index,
                text,
                section: section_of[index].clone(),
                in_proposed: markers.in_proposed[index],
                in_generated: markers.in_generated[index],
            });
        }
        let text: Arc<str> = Arc::from(body);
        let fences = scan
            .fences
            .iter()
            .map(|block| fenced_block(&text, lines, block, origin))
            .collect();
        Ok(Self {
            text,
            span: Span {
                lines: origin.line..origin.line + lines.len(),
                bytes: origin.byte..origin.byte + body.len(),
            },
            origin,
            model: Model {
                headings,
                sections,
                links: found.links,
                split_links: found.split,
                fences,
                generated: markers.generated,
                proposed: markers.proposed,
                claims: markers.claims,
                prose,
            },
        })
    }

    /// The body text, exactly as written.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Where the body sits in the whole text.
    ///
    /// For a parsed document the coordinates are those of the text that was
    /// parsed. For a document built with [`crate::Document::new`], and for a
    /// document after [`crate::Document::replace_spans`], they are those of
    /// the text [`crate::Document::render`] returns.
    #[must_use]
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// Every heading outside fenced code, levels 1 to 6, in order.
    #[must_use]
    pub fn headings(&self) -> &[Heading] {
        &self.model.headings
    }

    /// Whether a heading has this anchor, repeats numbered.
    #[must_use]
    pub fn has_anchor(&self, anchor: &str) -> bool {
        self.model.headings.iter().any(|h| h.anchor() == anchor)
    }

    /// The second-level sections, in order.
    #[must_use]
    pub fn sections(&self) -> &[Section] {
        &self.model.sections
    }

    /// The content of a section of this body, as text: the part of
    /// [`Self::text`] its span covers. `None` when the section does not belong
    /// to this body.
    #[must_use]
    pub fn section_text(&self, section: &Section) -> Option<&str> {
        let base = self.span.bytes.start;
        let bytes = &section.content().bytes;
        self.text
            .get(bytes.start.checked_sub(base)?..bytes.end.checked_sub(base)?)
    }

    /// The links outside fenced code, code spans and HTML comments, in line
    /// order. A link split across lines is not here; see
    /// [`Self::split_links`].
    #[must_use]
    pub fn links(&self) -> &[Link] {
        &self.model.links
    }

    /// The inline links and images whose text or parentheses span a line
    /// break, outside fenced code and code spans.
    #[must_use]
    pub fn split_links(&self) -> &[SplitLink] {
        &self.model.split_links
    }

    /// The fenced code blocks.
    #[must_use]
    pub fn fenced_blocks(&self) -> &[FencedBlock] {
        &self.model.fences
    }

    /// The generated blocks, in order.
    #[must_use]
    pub fn generated_blocks(&self) -> &[GeneratedBlock] {
        &self.model.generated
    }

    /// The PROPOSED blocks, in order.
    #[must_use]
    pub fn proposed_blocks(&self) -> &[ProposedBlock] {
        &self.model.proposed
    }

    /// The claim markers outside fenced code, code spans and PROPOSED blocks,
    /// in line order, each with the statement it supports. A claim inside a
    /// PROPOSED block is not a claim until the block is resolved.
    #[must_use]
    pub fn claims(&self) -> &[Claim] {
        &self.model.claims
    }

    /// The prose lines: outside fenced code and HTML comments, with code
    /// spans removed, each with its section and block membership.
    #[must_use]
    pub fn prose_lines(&self) -> &[ProseLine] {
        &self.model.prose
    }

    /// The placeholder tokens in the prose lines, as the 1-based line in the
    /// whole text and the token, in line order. This is the rule
    /// [`crate::Profile::validate`] applies to a body: a token is matched
    /// without regard to case, spacing or dots between letters, and fenced
    /// code, code spans and HTML comments are not read.
    #[must_use]
    pub fn placeholder_lines(&self) -> Vec<(usize, &'static str)> {
        let mut found = Vec::new();
        for line in &self.model.prose {
            for token in crate::profile::placeholders(line.text()) {
                found.push((line.line(), token));
            }
        }
        found
    }

    /// Replace byte ranges of the body in one pass and read the model again.
    ///
    /// The spans are in whole-text coordinates, as the model returns them;
    /// only their byte ranges count. They must not overlap. The change is
    /// all or nothing: on an error the body is as it was.
    ///
    /// # Errors
    ///
    /// [`SpliceError::OutOfBody`] for a range outside the body,
    /// [`SpliceError::NotCharBoundary`] for an offset inside a character,
    /// [`SpliceError::Overlap`] for two ranges that share bytes, and
    /// [`SpliceError::Reparse`] when the edited text does not read as a body.
    pub fn replace_spans(&mut self, edits: &[(Span, &str)]) -> Result<(), SpliceError> {
        let mut order: Vec<&(Span, &str)> = edits.iter().collect();
        order.sort_by_key(|(span, _)| (span.bytes.start, span.bytes.end));
        let base = self.span.bytes.start;
        let mut text = String::with_capacity(self.text.len());
        let mut copied = 0;
        let mut previous_end = base;
        for (span, replacement) in order {
            let (start, end) = (span.bytes.start, span.bytes.end);
            if start > end || start < base || end > self.span.bytes.end {
                return Err(SpliceError::OutOfBody { start, end });
            }
            for offset in [start, end] {
                if !self.text.is_char_boundary(offset - base) {
                    return Err(SpliceError::NotCharBoundary { offset });
                }
            }
            if start < previous_end {
                return Err(SpliceError::Overlap { offset: start });
            }
            text.push_str(&self.text[copied..start - base]);
            text.push_str(replacement);
            copied = end - base;
            previous_end = end;
        }
        text.push_str(&self.text[copied..]);
        let rebuilt = Self::parse_at(&text, self.origin).map_err(SpliceError::Reparse)?;
        *self = rebuilt;
        Ok(())
    }

    /// This body read again as if it started at `origin`.
    pub(crate) fn placed(&self, origin: Origin) -> Result<Self, ParseError> {
        Self::parse_at(&self.text, origin)
    }
}

/// The model of one fenced code block of `body`.
fn fenced_block(
    text: &Arc<str>,
    lines: &[blocks::Line],
    block: &blocks::FenceBlock,
    origin: Origin,
) -> FencedBlock {
    let body_len = text.len();
    let from = lines.get(block.first + 1).map_or(body_len, |l| l.start);
    let to = if block.closed {
        lines[block.last].start
    } else {
        body_len
    };
    FencedBlock {
        span: heading::span_of(lines, body_len, block.first, block.last + 1, origin),
        info: block.info.clone(),
        text: Arc::clone(text),
        content: from..to,
        closed: block.closed,
    }
}

impl PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl Eq for Body {}
