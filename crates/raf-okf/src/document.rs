// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A parsed document and its deterministic rendering.

use crate::body::{Body, Origin};
use crate::error::{ConstructError, ParseError, SpliceError};
use crate::frontmatter::reader::{LineEnding, read_head};
use crate::frontmatter::value::{Frontmatter, ListItem, Scalar, Value};
use crate::frontmatter::writer::write_frontmatter;
use crate::limits::MAX_TEXT_BYTES;
use crate::profile::DocType;
use crate::span::Span;

/// A Markdown document with YAML frontmatter.
///
/// The top-level `tags` list is held in sorted order, which is the order the
/// writer emits, so a document equals the reading of its own rendering. Items
/// keep their attached comments.
///
/// Two documents are equal when their frontmatter mappings are equal as maps
/// (values and attached comments included, key order aside), their comments
/// after the last key are equal, and their bodies hold the same bytes. The
/// line ending of the fences and a missing final newline are layout, not
/// content, and do not take part.
#[derive(Debug, Clone)]
pub struct Document {
    frontmatter: Frontmatter,
    body: Body,
    eol: LineEnding,
    closing_newline: bool,
}

impl Document {
    /// Read a document from `text`.
    ///
    /// The text starts with a `---` fence line, holds a block mapping in the
    /// frontmatter subset, and closes it with a `---` line; the rest is the
    /// body. The reader refuses what it does not take, with the line and
    /// column of the offence, and never panics or allocates beyond the size of
    /// the parsed result.
    ///
    /// # Errors
    ///
    /// A [`ParseError`] naming the line, the column or byte offset, and the
    /// rule broken: a text over the size bound, a missing or unclosed fence,
    /// a frontmatter that is not a block mapping, or a value outside the
    /// subset (flow mapping, duplicate key, anchor, alias, tag, merge key,
    /// block scalar, unknown escape, nesting over the bound).
    pub fn parse(text: &str) -> Result<Self, ParseError> {
        let mut head = read_head(text)?;
        sort_tags(&mut head.frontmatter);
        let origin = Origin::after_fences(&head.fences);
        let body = Body::parse_at(&text[origin.byte..], origin)?;
        Ok(Self {
            frontmatter: head.frontmatter,
            body,
            eol: head.fences.eol,
            closing_newline: head.fences.closing_newline,
        })
    }

    /// A document of `frontmatter` and `body`, written with `\n` line endings.
    /// The top-level `tags` list is put in sorted order, as in a parsed
    /// document. The body span is in the coordinates of [`Self::render`].
    ///
    /// # Errors
    ///
    /// [`ConstructError::TooLarge`] when the rendered text would exceed the
    /// size bound the reader enforces, and [`ConstructError::Body`] when the
    /// body does not read (a malformed marker, nesting over the bound).
    pub fn new(frontmatter: Frontmatter, body: impl Into<String>) -> Result<Self, ConstructError> {
        Self::new_with_head(frontmatter, &body.into()).map(|(document, _, _)| document)
    }

    /// Construct the document with its rendered head and whether tags were reordered.
    pub(crate) fn new_with_head(
        mut frontmatter: Frontmatter,
        body: &str,
    ) -> Result<(Self, String, bool), ConstructError> {
        let tags_reordered = sort_tags(&mut frontmatter);
        let mut document = Self {
            frontmatter,
            body: Body::parse_at("", Origin::START).map_err(ConstructError::Body)?,
            eol: LineEnding::Lf,
            closing_newline: true,
        };
        let head = document.render_head();
        let origin = body_origin(&head);
        let len = origin.byte + body.len();
        if len > MAX_TEXT_BYTES {
            return Err(ConstructError::TooLarge {
                len,
                max: MAX_TEXT_BYTES,
            });
        }
        // Read the body where it sits in the rendered text, so that an error
        // carries the line and offset of that text.
        document.body = Body::parse_at(body, origin).map_err(ConstructError::Body)?;
        Ok((document, head, tags_reordered))
    }

    /// Read a reconstructed head and install its parsed source positions.
    ///
    /// Return false if rendering changes the frontmatter or the body boundary.
    pub(crate) fn read_rendered_head(&mut self, mut rendered: String) -> Result<bool, ParseError> {
        let mut head = read_head(&rendered)?;
        let origin = Origin::after_fences(&head.fences);
        let span = self.body.span();
        if origin.byte != span.bytes.start || origin.line != span.lines.start {
            // An early closing fence makes part of the rendered head into
            // Markdown. Preserve a parse error in that shifted body.
            rendered.push_str(self.body.text());
            Body::parse_at(&rendered[origin.byte..], origin)?;
            return Ok(false);
        }
        sort_tags(&mut head.frontmatter);
        if head.frontmatter != self.frontmatter {
            return Ok(false);
        }
        self.frontmatter = head.frontmatter;
        Ok(true)
    }

    /// Replace byte ranges of the body in one pass; see
    /// [`Body::replace_spans`]. The spans are those the body model returned.
    /// After a successful edit the body model is in the coordinates of the
    /// text [`Self::render`] returns.
    ///
    /// # Errors
    ///
    /// The [`SpliceError`] of [`Body::replace_spans`]; the document is
    /// unchanged on an error.
    pub fn replace_spans(&mut self, edits: &[(Span, &str)]) -> Result<(), SpliceError> {
        let mut body = self.body.clone();
        body.replace_spans(edits)?;
        // A body that follows a closing fence with no line ending needs one.
        let closing_newline = self.closing_newline || !body.text().is_empty();
        let moved = Self {
            body,
            closing_newline,
            ..self.clone()
        };
        let placed = moved.place_body().map_err(SpliceError::Reparse)?;
        self.body = placed;
        self.closing_newline = closing_newline;
        Ok(())
    }

    /// The body read again at the place it has in the rendered text.
    fn place_body(&self) -> Result<Body, ParseError> {
        self.body.placed(body_origin(&self.render_head()))
    }

    /// The frontmatter.
    #[must_use]
    pub fn frontmatter(&self) -> &Frontmatter {
        &self.frontmatter
    }

    /// The body.
    #[must_use]
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// The document type named by the `type` key, when it is one of the
    /// profile's types.
    #[must_use]
    pub fn doc_type(&self) -> Option<DocType> {
        self.frontmatter
            .map()
            .get("type")
            .and_then(|entry| entry.value().as_scalar())
            .and_then(|scalar| DocType::from_name(scalar.text()))
    }

    /// Write the document as text.
    ///
    /// The output is deterministic. The keys of a known document type come
    /// first in the order of its schema table and the other keys follow in
    /// input order; a document of an unknown type keeps input order. The
    /// `tags` list is written sorted. Comments stay with the key they were
    /// attached to, and the comments after the last key stay last. Flow lists
    /// are written as `[a, b]`. The body is written byte for byte.
    /// Rendering the output of `render` gives the same text.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = self.render_head();
        out.push_str(self.body.text());
        out
    }

    /// The text up to the body: both fences and the frontmatter.
    fn render_head(&self) -> String {
        let eol = self.eol.as_str();
        let mut out = String::new();
        out.push_str("---");
        out.push_str(eol);
        write_frontmatter(&mut out, &self.frontmatter, self.eol);
        out.push_str("---");
        if self.closing_newline {
            out.push_str(eol);
        }
        out
    }
}

/// Where the body starts in the rendered text.
fn body_origin(head: &str) -> Origin {
    Origin {
        byte: head.len(),
        line: 1 + head.matches('\n').count() + usize::from(!head.ends_with('\n')),
    }
}

/// Put the top-level `tags` list in stable text order and report a reordering.
fn sort_tags(frontmatter: &mut Frontmatter) -> bool {
    let Some(entry) = frontmatter
        .map
        .entries
        .iter_mut()
        .find(|entry| entry.key == "tags")
    else {
        return false;
    };
    if let Value::List(list) = &mut entry.value
        && !list
            .items
            .is_sorted_by(|left, right| tag_text(left) <= tag_text(right))
    {
        list.items
            .sort_by(|left, right| tag_text(left).cmp(&tag_text(right)));
        return true;
    }
    false
}

fn tag_text(item: &ListItem) -> Option<&str> {
    item.value.as_scalar().map(Scalar::text)
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.frontmatter == other.frontmatter && self.body == other.body
    }
}

impl Eq for Document {}
