// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The prose view of a body.

/// One line of prose: outside fenced code and HTML comments, with code spans
/// removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProseLine {
    pub(crate) line: usize,
    pub(crate) text: String,
    pub(crate) section: Option<String>,
    pub(crate) in_proposed: bool,
    pub(crate) in_generated: bool,
}

impl ProseLine {
    /// The 1-based line in the whole text.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }

    /// The text of the line with HTML comments removed and every code span
    /// replaced by one space.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The slug of the second-level section that holds the line, or `None`
    /// before the first second-level heading.
    #[must_use]
    pub fn section(&self) -> Option<&str> {
        self.section.as_deref()
    }

    /// Whether the line belongs to a PROPOSED block.
    #[must_use]
    pub fn in_proposed(&self) -> bool {
        self.in_proposed
    }

    /// Whether the line lies between the markers of a generated block.
    #[must_use]
    pub fn in_generated(&self) -> bool {
        self.in_generated
    }
}
