// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! PROPOSED blocks and generated blocks.

use std::sync::Arc;

use crate::body::blocks::{Line, Origin, error_at};
use crate::body::heading::span_of;
use crate::error::{ErrorKind, ParseError};
use crate::span::Span;

const PROPOSED_OPEN: &str = "> [!PROPOSED change:";
pub(crate) const GENERATED_OPEN: &str = "<!-- raf:generated:";
const GENERATED_CLOSE: &str = " -->";
pub(crate) const GENERATED_END: &str = "end";

/// A block quote whose first line is the marker
/// `> [!PROPOSED change:<slug>]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedBlock {
    slug: String,
    span: Span,
    marker: Span,
    content: String,
    section: Option<Arc<str>>,
}

impl ProposedBlock {
    /// The slug of the change that proposes the text.
    #[must_use]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// The marker line and the quote lines that follow it, with their line
    /// endings, in whole-text coordinates.
    #[must_use]
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// The marker line alone, with its line ending.
    #[must_use]
    pub fn marker(&self) -> &Span {
        &self.marker
    }

    /// The quoted text after the marker line, with the `>` and the space after
    /// it removed from each line and the lines joined by `\n`.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// The slug of the second-level section that holds the block, such as
    /// `contract`, or `None` before the first second-level heading.
    #[must_use]
    pub fn section(&self) -> Option<&str> {
        self.section.as_deref()
    }
}

/// A block delimited by `<!-- raf:generated:<kind> -->` and
/// `<!-- raf:generated:end -->`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedBlock {
    kind: String,
    outer: Span,
    content: Span,
}

impl GeneratedBlock {
    /// The kind named by the opening marker.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Both marker lines and everything between them, in whole-text
    /// coordinates.
    #[must_use]
    pub fn outer(&self) -> &Span {
        &self.outer
    }

    /// The lines between the markers, in whole-text coordinates. Replacing
    /// this span rewrites the block and keeps its markers.
    #[must_use]
    pub fn content(&self) -> &Span {
        &self.content
    }
}

/// True when the line reads as a PROPOSED marker however it is spelled: the
/// quote marks, list bullets and spaces before `[!` are skipped and the word
/// is compared without regard to case.
fn looks_like_proposed(trimmed: &str) -> bool {
    trimmed
        .trim_start_matches(['>', ' ', '\t', '-', '*'])
        .to_ascii_lowercase()
        .starts_with("[!proposed")
}

/// A lowercase kebab-case slug.
pub(crate) fn is_kebab(slug: &str) -> bool {
    !slug.is_empty()
        && slug.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

/// What the marker scan found.
pub(crate) struct Markers {
    pub(crate) proposed: Vec<ProposedBlock>,
    pub(crate) generated: Vec<GeneratedBlock>,
    /// For each line, whether it belongs to a PROPOSED block.
    pub(crate) in_proposed: Vec<bool>,
    /// For each line, whether it lies between generated markers.
    pub(crate) in_generated: Vec<bool>,
}

/// The unquoted text of a quote line: the `>` and one following space gone.
fn unquote(line: &str) -> &str {
    let rest = if let Some(rest) = line.trim_start().strip_prefix('>') {
        rest
    } else {
        line
    };
    if let Some(rest) = rest.strip_prefix(' ') {
        rest
    } else {
        rest
    }
}

/// Scan the lines outside fences for markers. `section_of` gives the slug of
/// the second-level section that holds each line.
pub(crate) fn scan_markers(
    body: &str,
    lines: &[Line],
    origin: Origin,
    section_of: &[Option<Arc<str>>],
) -> Result<Markers, ParseError> {
    let mut found = Markers {
        proposed: Vec::new(),
        generated: Vec::new(),
        in_proposed: vec![false; lines.len()],
        in_generated: vec![false; lines.len()],
    };
    let mut open: Option<(usize, String)> = None;
    for (index, line) in lines.iter().enumerate() {
        if line.fenced {
            continue;
        }
        let raw = &body[line.start..line.end];
        let trimmed = raw.trim();
        let column = raw.len() - raw.trim_start().len();
        let fail = |kind| error_at(body, lines, origin, index, column, kind);
        if looks_like_proposed(trimmed) {
            let slug = trimmed
                .strip_prefix(PROPOSED_OPEN)
                .and_then(|rest| rest.strip_suffix(']'))
                .filter(|slug| is_kebab(slug));
            let Some(slug) = slug else {
                return Err(fail(ErrorKind::MalformedProposed));
            };
            let mut last = index + 1;
            while last < lines.len()
                && !lines[last].fenced
                && body[lines[last].start..lines[last].end]
                    .trim_start()
                    .starts_with('>')
                && !looks_like_proposed(body[lines[last].start..lines[last].end].trim())
            {
                last += 1;
            }
            let content = (index + 1..last)
                .map(|at| unquote(&body[lines[at].start..lines[at].end]))
                .collect::<Vec<_>>()
                .join("\n");
            for flag in &mut found.in_proposed[index..last] {
                *flag = true;
            }
            found.proposed.push(ProposedBlock {
                slug: slug.to_string(),
                span: span_of(lines, body.len(), index, last, origin),
                marker: span_of(lines, body.len(), index, index + 1, origin),
                content,
                section: section_of[index].clone(),
            });
        }
        if trimmed.starts_with(GENERATED_OPEN) {
            let kind = trimmed
                .strip_prefix(GENERATED_OPEN)
                .and_then(|rest| rest.strip_suffix(GENERATED_CLOSE));
            match (kind, open.take()) {
                (Some(GENERATED_END), Some((first, kind))) => {
                    for flag in &mut found.in_generated[first + 1..index] {
                        *flag = true;
                    }
                    found.generated.push(GeneratedBlock {
                        kind,
                        outer: span_of(lines, body.len(), first, index + 1, origin),
                        content: span_of(lines, body.len(), first + 1, index, origin),
                    });
                }
                (Some(GENERATED_END), None) => {
                    return Err(fail(ErrorKind::OrphanGeneratedEnd));
                }
                (Some(kind), None) if is_kebab(kind) => {
                    open = Some((index, kind.to_string()));
                }
                (Some(kind), Some((_, outer))) if is_kebab(kind) => {
                    return Err(fail(ErrorKind::NestedGenerated { outer }));
                }
                (Some(_) | None, _) => return Err(fail(ErrorKind::MalformedGenerated)),
            }
        }
    }
    if let Some((first, kind)) = open {
        return Err(error_at(
            body,
            lines,
            origin,
            first,
            0,
            ErrorKind::UnclosedGenerated { kind },
        ));
    }
    Ok(found)
}
