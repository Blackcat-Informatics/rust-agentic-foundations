// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The generated navigation file of the capability directory.

use crate::body::{Body, GENERATED_END, GENERATED_OPEN, GeneratedBlock, Origin, is_kebab};
use crate::error::{ErrorKind, ParseError, SpliceError};
use crate::span::Span;

/// The repository path of the capability index.
const CAPABILITY_INDEX: &str = "docs/capabilities/index.md";

/// True for the path of the capability index, `docs/capabilities/index.md`,
/// relative to the repository root. The file is never read as a Capability
/// document.
#[must_use]
pub fn is_capability_index(path: &str) -> bool {
    path == CAPABILITY_INDEX || path.strip_prefix("./") == Some(CAPABILITY_INDEX)
}

/// A generated index: an optional SPDX header comment, then generated blocks
/// and nothing else but blank lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Index {
    body: Body,
}

/// True for a line that opens with a real SPDX tag that has a value.
fn is_spdx_tag(text: &str) -> bool {
    ["SPDX-FileCopyrightText:", "SPDX-License-Identifier:"]
        .iter()
        .any(|tag| text.strip_prefix(tag).is_some_and(|v| !v.trim().is_empty()))
}

impl Index {
    /// Read an index.
    ///
    /// Apart from a leading HTML comment of SPDX lines (one line, or several
    /// between `<!--` and `-->` lines, each tag with a value), every non-blank
    /// line lies inside a generated block, and each block marker starts its
    /// own line with no indentation.
    ///
    /// # Errors
    ///
    /// A [`ParseError`] with the line: prose outside a block, a header comment
    /// that holds more than SPDX tags or sits after a block, a malformed,
    /// indented or misplaced marker, a block that is never closed, or an
    /// index with no block at all.
    pub fn parse(text: &str) -> Result<Self, ParseError> {
        check_index(text)?;
        let body = Body::parse_at(text, Origin::START)?;
        Ok(Self { body })
    }

    /// The text of the index.
    #[must_use]
    pub fn render(&self) -> String {
        self.body.text().to_string()
    }

    /// The generated blocks, in order.
    #[must_use]
    pub fn generated_blocks(&self) -> &[GeneratedBlock] {
        self.body.generated_blocks()
    }

    /// The body model of the index.
    #[must_use]
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// Replace byte ranges of the index in one pass; see
    /// [`Body::replace_spans`]. The result must be an index again.
    ///
    /// # Errors
    ///
    /// The [`SpliceError`] of [`Body::replace_spans`], with
    /// [`SpliceError::Reparse`] also when the edited text is not an index.
    /// The index is unchanged on an error.
    pub fn replace_spans(&mut self, edits: &[(Span, &str)]) -> Result<(), SpliceError> {
        let mut body = self.body.clone();
        body.replace_spans(edits)?;
        check_index(body.text()).map_err(SpliceError::Reparse)?;
        self.body = body;
        Ok(())
    }
}

fn check_index(text: &str) -> Result<(), ParseError> {
    let mut in_header: Option<(usize, usize)> = None;
    let mut header_done = false;
    let mut open: Option<(usize, usize, String)> = None;
    let mut blocks = 0_usize;
    let mut offset = 0;
    for (index, raw) in text.split_inclusive('\n').enumerate() {
        let number = index + 1;
        let start = offset;
        offset += raw.len();
        let line = line_text(raw);
        let fail = |kind| ParseError::at(kind, number, 1, start);
        let trimmed = line.trim();
        if in_header.is_some() {
            if trimmed == "-->" {
                in_header = None;
            } else if !is_spdx_tag(trimmed) {
                return Err(fail(ErrorKind::IndexHeader));
            }
            continue;
        }
        let spdx_line = trimmed
            .strip_prefix("<!--")
            .and_then(|rest| rest.strip_suffix("-->"))
            .is_some_and(|inner| is_spdx_tag(inner.trim()));
        if trimmed.is_empty() || (open.is_none() && blocks == 0 && spdx_line) {
            continue;
        }
        if open.is_none() && blocks == 0 && !header_done && line == "<!--" {
            in_header = Some((number, start));
            header_done = true;
            continue;
        }
        if line.starts_with(GENERATED_OPEN) {
            let kind = line
                .strip_prefix(GENERATED_OPEN)
                .and_then(|rest| rest.strip_suffix(" -->"));
            match (kind, open.is_some()) {
                (Some(GENERATED_END), true) => open = None,
                (Some(kind), false) if kind != GENERATED_END && is_kebab(kind) => {
                    open = Some((number, start, kind.to_string()));
                    blocks += 1;
                }
                (Some(_) | None, _) => return Err(fail(ErrorKind::IndexMarker)),
            }
        } else if open.is_none() {
            let kind = if trimmed.starts_with("<!--") {
                ErrorKind::IndexHeader
            } else {
                ErrorKind::IndexText
            };
            return Err(fail(kind));
        }
    }
    if let Some((number, start)) = in_header {
        return Err(ParseError::at(ErrorKind::IndexHeader, number, 1, start));
    }
    if let Some((number, start, kind)) = open {
        return Err(ParseError::at(
            ErrorKind::UnclosedGenerated { kind },
            number,
            1,
            start,
        ));
    }
    if blocks == 0 {
        return Err(ParseError::at(ErrorKind::IndexEmpty, 1, 1, 0));
    }
    Ok(())
}

/// The line without its line ending, `\n` or `\r\n`.
fn line_text(raw: &str) -> &str {
    let Some(line) = raw.strip_suffix('\n') else {
        return raw;
    };
    if let Some(line) = line.strip_suffix('\r') {
        line
    } else {
        line
    }
}
