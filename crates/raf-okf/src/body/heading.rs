// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Headings, their anchors, and second-level sections.

use std::collections::HashMap;

use crate::body::blocks::{Line, Origin};
use crate::span::Span;

/// An ATX heading outside fenced code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    level: usize,
    text: String,
    anchor: String,
    line: usize,
}

impl Heading {
    /// The level, 1 to 6.
    #[must_use]
    pub fn level(&self) -> usize {
        self.level
    }

    /// The heading text without the `#` marks and the closing marks.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The anchor a link uses: lowercase, spaces as hyphens, other
    /// punctuation dropped, and `-1`, `-2` ... added to a repeat.
    #[must_use]
    pub fn anchor(&self) -> &str {
        &self.anchor
    }

    /// The 1-based line of the heading in the whole text.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }
}

/// A second-level section: the heading and everything up to the next
/// second-level heading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    heading: String,
    slug: String,
    line: usize,
    content: Span,
}

impl Section {
    /// The heading text.
    #[must_use]
    pub fn heading(&self) -> &str {
        &self.heading
    }

    /// The heading text as an anchor, without the repeat suffix: `explanation`
    /// for `## Explanation`.
    #[must_use]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// The 1-based line of the heading in the whole text.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }

    /// The content: from the line after the heading to the line before the
    /// next second-level heading, or the end of the body. In whole-text
    /// coordinates.
    #[must_use]
    pub fn content(&self) -> &Span {
        &self.content
    }
}

/// The text of an ATX heading line and its level, or `None` when the line is
/// not one. Up to three spaces may indent the marks, and the marks are
/// followed by a space or tab.
pub(crate) fn atx(line: &str) -> Option<(usize, String)> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return None;
    }
    let rest = &line[indent..];
    let level = rest.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let after = &rest[level..];
    if !after.starts_with([' ', '\t']) {
        return None;
    }
    let text = after.trim();
    if text.is_empty() {
        return None;
    }
    let without_marks = text.trim_end_matches('#');
    let closed = without_marks.len() != text.len()
        && (without_marks.is_empty() || without_marks.ends_with([' ', '\t']));
    let text = if closed { without_marks.trim() } else { text };
    Some((level, text.to_string()))
}

/// The GitHub anchor of heading text: lowercase, spaces to hyphens, `-` and
/// `_` kept, every other non-alphanumeric character dropped.
pub(crate) fn github_anchor(heading: &str) -> String {
    let mut out = String::new();
    for c in heading.trim().chars() {
        if c == ' ' {
            out.push('-');
        } else if c == '-' || c == '_' {
            out.push(c);
        } else if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
        }
    }
    out
}

/// Number the repeats: the first use of an anchor keeps it, the next gets
/// `-1`, then `-2`.
pub(crate) struct Anchors {
    seen: HashMap<String, usize>,
}

impl Anchors {
    pub(crate) fn new() -> Self {
        Self {
            seen: HashMap::new(),
        }
    }

    pub(crate) fn next(&mut self, base: &str) -> String {
        let uses = self.seen.get(base).map_or(0, |count| *count);
        self.seen.insert(base.to_string(), uses + 1);
        if uses == 0 {
            base.to_string()
        } else {
            format!("{base}-{uses}")
        }
    }
}

/// The span of lines `first..end` (indexes into `lines`) in whole-text
/// coordinates.
pub(crate) fn span_of(
    lines: &[Line],
    body_len: usize,
    first: usize,
    end: usize,
    origin: Origin,
) -> Span {
    let byte_at = |index: usize| match lines.get(index) {
        Some(line) => line.start,
        None => body_len,
    };
    Span {
        lines: origin.line + first..origin.line + end,
        bytes: origin.byte + byte_at(first)..origin.byte + byte_at(end),
    }
}

/// Headings and sections from the comment-free text of each line.
pub(crate) fn read_headings(
    lines: &[Line],
    visible: &[String],
    body_len: usize,
    origin: Origin,
) -> (Vec<Heading>, Vec<Section>) {
    let mut headings = Vec::new();
    let mut found: Vec<(usize, usize)> = Vec::new();
    let mut anchors = Anchors::new();
    for (index, line) in lines.iter().enumerate() {
        if line.fenced {
            continue;
        }
        if let Some((level, text)) = atx(&visible[index]) {
            let anchor = anchors.next(&github_anchor(&text));
            headings.push(Heading {
                level,
                text,
                anchor,
                line: origin.line + index,
            });
            found.push((index, level));
        }
    }
    let second: Vec<(usize, &Heading)> = found
        .iter()
        .zip(headings.iter())
        .filter(|((_, level), _)| *level == 2)
        .map(|((index, _), heading)| (*index, heading))
        .collect();
    let mut sections = Vec::new();
    for (position, (index, heading)) in second.iter().enumerate() {
        let end = match second.get(position + 1) {
            Some((next, _)) => *next,
            None => lines.len(),
        };
        sections.push(Section {
            slug: github_anchor(&heading.text),
            heading: heading.text.clone(),
            line: heading.line,
            content: span_of(lines, body_len, index + 1, end, origin),
        });
    }
    (headings, sections)
}
