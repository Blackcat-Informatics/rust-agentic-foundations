// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Coordinate projection of native authoring lines.

use crate::error::{ErrorKind, ParseError};

/// Where a body starts in the whole text of its document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Origin {
    /// Byte offset of the first byte of the body.
    pub(crate) byte: usize,
    /// 1-based line number of the first line of the body.
    pub(crate) line: usize,
}

impl Origin {
    /// The origin of a text that is not part of a longer one.
    pub(crate) const START: Self = Self { byte: 0, line: 1 };
}

/// One line of the body. Offsets are local to the body text.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Line {
    /// Offset of the first byte.
    pub(crate) start: usize,
    /// Offset after the last byte of the text, before any line ending.
    pub(crate) end: usize,
    /// The line is a fence line or sits inside a fenced code block.
    pub(crate) fenced: bool,
    pub(crate) heading: bool,
    pub(crate) list_item: bool,
}

/// Project an original byte to its physical line without rescanning text.
pub(crate) fn line_of(lines: &[Line], byte: usize) -> usize {
    lines
        .partition_point(|line| line.start <= byte)
        .saturating_sub(1)
}

/// The error for a failure at byte offset within a physical line.
pub(crate) fn error_at(
    body: &str,
    lines: &[Line],
    origin: Origin,
    index: usize,
    column_byte: usize,
    kind: ErrorKind,
) -> ParseError {
    let line = &lines[index];
    let local = line.start + column_byte;
    let column = body[line.start..local].chars().count() + 1;
    ParseError::at(kind, origin.line + index, column, origin.byte + local)
}

/// Native spans are offsets into the admitted, at most one MiB input.
pub(super) fn offset(byte: u64) -> usize {
    // INVARIANT: native authoring observations derive every endpoint from its
    // borrowed source; that source already fits usize and the document bound.
    usize::try_from(byte).expect("INVARIANT: native span is within the admitted source")
}

/// Project coordinates and native heading/list observations without reading grammar.
pub(super) fn project(native: &purrdf_markdown::AuthoringDocument<'_>) -> Vec<Line> {
    let mut lines: Vec<Line> = native
        .lines()
        .iter()
        .map(|line| Line {
            start: offset(line.source.start),
            end: offset(line.source.end),
            fenced: line.fenced,
            heading: false,
            list_item: false,
        })
        .collect();
    for heading in native.headings() {
        let index = line_of(&lines, offset(heading.source.start));
        lines[index].heading = true;
    }
    for block in native.blocks() {
        if matches!(
            block.kind,
            purrdf_markdown::syntax::BlockKind::ListItem { .. }
        ) {
            // Native block ranges index the physical line arena.
            lines[block.lines.start].list_item = true;
        }
    }
    lines
}
