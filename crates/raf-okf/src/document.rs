// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A parsed document and its deterministic rendering.

use crate::body::Body;
use crate::error::{ConstructError, ParseError};
use crate::frontmatter::reader::{LineEnding, read_head};
use crate::frontmatter::value::{Frontmatter, ListItem, Value};
use crate::frontmatter::writer::write_frontmatter;
use crate::limits::MAX_TEXT_BYTES;
use crate::profile::DocType;

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
        let body = Body::new(&text[head.body_start..]).placed(head.body_line, head.body_start);
        Ok(Self {
            frontmatter: head.frontmatter,
            body,
            eol: head.eol,
            closing_newline: head.closing_newline,
        })
    }

    /// A document of `frontmatter` and `body`, written with `\n` line endings.
    /// The top-level `tags` list is put in sorted order, as in a parsed
    /// document. The body span is in the coordinates of [`Self::render`].
    ///
    /// # Errors
    ///
    /// [`ConstructError::TooLarge`] when the rendered text would exceed the
    /// size bound the reader enforces.
    pub fn new(frontmatter: Frontmatter, body: impl Into<String>) -> Result<Self, ConstructError> {
        let body = Body::new(body);
        let mut frontmatter = frontmatter;
        sort_tags(&mut frontmatter);
        let mut document = Self {
            frontmatter,
            body: Body::new(""),
            eol: LineEnding::Lf,
            closing_newline: true,
        };
        let head_len = document.render_head().len();
        let first_line = 3 + document.frontmatter_line_count();
        document.body = body.placed(first_line, head_len);
        let len = head_len + document.body.text().len();
        if len > MAX_TEXT_BYTES {
            return Err(ConstructError::TooLarge {
                len,
                max: MAX_TEXT_BYTES,
            });
        }
        Ok(document)
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

    /// The number of lines the frontmatter takes when written.
    fn frontmatter_line_count(&self) -> usize {
        let mut out = String::new();
        write_frontmatter(&mut out, &self.frontmatter, self.eol);
        out.matches('\n').count()
    }
}

/// Put the top-level `tags` list in sorted order. The sort is stable and
/// orders scalars by their text.
fn sort_tags(frontmatter: &mut Frontmatter) {
    let Some(entry) = frontmatter
        .map
        .entries
        .iter_mut()
        .find(|entry| entry.key == "tags")
    else {
        return;
    };
    if let Value::List(list) = &mut entry.value {
        list.items.sort_by(|left, right| {
            let text = |item: &ListItem| item.value.as_scalar().map(|s| s.text.clone());
            text(left).cmp(&text(right))
        });
    }
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.frontmatter == other.frontmatter && self.body == other.body
    }
}

impl Eq for Document {}
