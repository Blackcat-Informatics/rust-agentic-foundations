// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The body of a document: the text after the closing frontmatter fence.

use crate::span::Span;

/// The body of a document.
///
/// The body is kept byte for byte. [`Body::span`] locates it in the text of
/// the document it belongs to. A body is built by [`crate::Document::parse`]
/// or [`crate::Document::new`] only.
#[derive(Debug, Clone)]
pub struct Body {
    text: String,
    span: Span,
}

impl Body {
    /// A body holding `text`. Its span is `0..len` until a document places it.
    pub(crate) fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let span = Span {
            lines: 1..1 + line_count(&text),
            bytes: 0..text.len(),
        };
        Self { text, span }
    }

    /// The body text, exactly as written.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Where the body sits in the whole text of its document.
    ///
    /// For a parsed document the coordinates are those of the text that was
    /// parsed. For a document built with [`crate::Document::new`] they are
    /// those of the text [`crate::Document::render`] returns. The rendering of
    /// a parsed document whose keys were reordered can differ in length from
    /// the text it was parsed from, and its body then sits elsewhere.
    #[must_use]
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// This body placed at `first_line` and byte `start` of a document text.
    pub(crate) fn placed(mut self, first_line: usize, start: usize) -> Self {
        self.span = Span {
            lines: first_line..first_line + line_count(&self.text),
            bytes: start..start + self.text.len(),
        };
        self
    }
}

/// The number of lines in `text`: a final line without a line ending counts.
fn line_count(text: &str) -> usize {
    let newlines = text.bytes().filter(|b| *b == b'\n').count();
    if text.is_empty() || text.ends_with('\n') {
        newlines
    } else {
        newlines + 1
    }
}

impl PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl Eq for Body {}
