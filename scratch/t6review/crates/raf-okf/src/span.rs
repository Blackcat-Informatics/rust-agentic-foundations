// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A region of a document text.

use std::ops::Range;

/// A region of the whole document text, in lines and in bytes.
///
/// The coordinates belong to one text: the text a document was parsed from,
/// or the text `render` returns for a document built from parts.
///
/// Both ranges are half open. Line numbers are 1-based, so the span of an
/// empty region that starts on line `n` is `n..n`. Byte offsets count from the
/// first byte of the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// The lines the region covers.
    pub lines: Range<usize>,
    /// The bytes the region covers.
    pub bytes: Range<usize>,
}
