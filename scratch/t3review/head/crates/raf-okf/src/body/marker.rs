// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! PROPOSED blocks, generated blocks, claim markers and tested-by markers.

use std::collections::HashSet;
use std::sync::Arc;

use crate::body::blocks::{Line, Origin, error_at, opens_list_item, remove_code_spans};
use crate::body::heading::{atx, span_of};
use crate::error::{ErrorKind, ParseError};
use crate::span::Span;

const PROPOSED_OPEN: &str = "> [!PROPOSED change:";
pub(crate) const GENERATED_OPEN: &str = "<!-- raf:generated:";
const GENERATED_CLOSE: &str = " -->";
pub(crate) const GENERATED_END: &str = "end";
const CLAIM_OPEN: &str = "<!-- raf:claim ";
const CLAIM_CLOSE: &str = " -->";
/// The attribute of a claim marker that names its evidence.
const CLAIM_EVIDENCE: &str = "evidence";
const TESTED_BY_OPEN: &str = "<!-- raf:tested-by ";
const TESTED_BY_CLOSE: &str = " -->";

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

/// A claim marker, `<!-- raf:claim evidence=<slug>[,<slug>...] -->`, and the
/// statement it is attached to: the paragraph or list item directly above it.
///
/// ```rust
/// use raf_okf::Body;
///
/// let text = "## Contract\n\nParsing stays under 64 MiB.\n<!-- raf:claim evidence=memory-bound -->\n";
/// let body = Body::parse(text).expect("a body");
/// let claim = &body.claims()[0];
/// assert_eq!(claim.evidence(), ["memory-bound"]);
/// assert_eq!(claim.line(), 4);
/// assert_eq!(claim.section(), Some("contract"));
/// assert_eq!(&text[claim.statement().bytes.clone()], "Parsing stays under 64 MiB.\n");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    evidence: Vec<String>,
    marker: Span,
    statement: Span,
    section: Option<Arc<str>>,
}

impl Claim {
    /// The 1-based line of the marker in the whole text.
    #[must_use]
    pub fn line(&self) -> usize {
        self.marker.lines.start
    }

    /// The slugs of the evidence records the statement rests on, in marker
    /// order, each once. A slug `<slug>` names the record
    /// `evidence:<slug>`.
    #[must_use]
    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }

    /// The marker line, with its line ending.
    #[must_use]
    pub fn marker(&self) -> &Span {
        &self.marker
    }

    /// The lines of the paragraph or list item the marker supports, with
    /// their line endings, ending before the marker.
    #[must_use]
    pub fn statement(&self) -> &Span {
        &self.statement
    }

    /// The slug of the second-level section that holds the marker, or `None`
    /// before the first second-level heading.
    #[must_use]
    pub fn section(&self) -> Option<&str> {
        self.section.as_deref()
    }
}

/// A tested-by marker, `<!-- raf:tested-by <path>[,<path>...] -->`, and the
/// statement it maps to the tests it names: the paragraph or list item
/// directly above it, the statement of the claim marker directly above it,
/// or the heading directly above it.
///
/// ```rust
/// use raf_okf::Body;
///
/// let text = "## Contract\n\n### Chunked input\n<!-- raf:tested-by demo::test_crate::it::chunks -->\n";
/// let body = Body::parse(text).expect("a body");
/// let marker = &body.tested_by()[0];
/// assert_eq!(marker.paths(), ["demo::test_crate::it::chunks"]);
/// assert_eq!(marker.line(), 4);
/// assert_eq!(marker.section(), Some("contract"));
/// assert_eq!(&text[marker.statement().bytes.clone()], "### Chunked input\n");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestedBy {
    paths: Vec<String>,
    marker: Span,
    statement: Span,
    section: Option<Arc<str>>,
}

impl TestedBy {
    /// The 1-based line of the marker in the whole text.
    #[must_use]
    pub fn line(&self) -> usize {
        self.marker.lines.start
    }

    /// The paths of the tests and examples the marker names, in marker
    /// order, each once.
    #[must_use]
    pub fn paths(&self) -> &[String] {
        &self.paths
    }

    /// The marker line, with its line ending.
    #[must_use]
    pub fn marker(&self) -> &Span {
        &self.marker
    }

    /// The lines of the statement the marker maps, with their line endings:
    /// the paragraph or list item directly above it, the statement of the
    /// claim marker directly above it, or the heading line directly above it.
    #[must_use]
    pub fn statement(&self) -> &Span {
        &self.statement
    }

    /// The slug of the second-level section that holds the marker, or `None`
    /// before the first second-level heading.
    #[must_use]
    pub fn section(&self) -> Option<&str> {
        self.section.as_deref()
    }
}

/// The family of a `<!-- raf:` comment that is not a generated marker, by
/// the word after `raf:`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    Claim,
    TestedBy,
    Other,
}

/// A marker the scan read on one line.
enum Found {
    Claim(Claim),
    TestedBy(TestedBy),
}

/// The family of a comment by its first text after `<!--`, trimmed: `raf:`
/// in any letter case and then a word other than `generated:`, or `None`.
fn family_of(text: &str) -> Option<Family> {
    let word = text
        .get(..4)
        .filter(|prefix| prefix.eq_ignore_ascii_case("raf:"))
        .map(|_| text[4..].to_ascii_lowercase())?;
    if word.starts_with("generated:") {
        None
    } else if word.starts_with("claim") {
        Some(Family::Claim)
    } else if word.starts_with("tested-by") {
        Some(Family::TestedBy)
    } else {
        Some(Family::Other)
    }
}

/// The family of the first `<!-- raf:` comment of a line outside its code
/// spans, in any letter case and with any spacing after `<!--`, that is not
/// a `raf:generated:` comment, or `None` when the line has none.
fn raf_comment(raw: &str) -> Option<Family> {
    let visible = remove_code_spans(raw);
    visible
        .match_indices("<!--")
        .find_map(|(at, _)| family_of(visible[at + 4..].trim_start()))
}

/// The family of a comment that `<!--` opens at the end of line `index`
/// outside its code spans, with nothing after it on that line, read from
/// the first non-blank text of the lines below it as [`raf_comment`] reads
/// the text after `<!--` on one line. `None` when the line ends otherwise,
/// or when that text is fenced code, which is not read.
fn split_raf_comment(body: &str, lines: &[Line], index: usize) -> Option<Family> {
    let raw = &body[lines[index].start..lines[index].end];
    if !remove_code_spans(raw).trim_end().ends_with("<!--") {
        return None;
    }
    let next = lines[index + 1..]
        .iter()
        .find(|line| !body[line.start..line.end].trim().is_empty())?;
    if next.fenced {
        return None;
    }
    family_of(body[next.start..next.end].trim_start())
}

/// The evidence slugs of a claim marker line, trimmed, or `None` when the
/// line is not exactly a marker: the attributes are single-space separated
/// `key=value` pairs from a closed set, each once, and `evidence` is a comma
/// list of distinct kebab-case slugs.
fn parse_claim(trimmed: &str) -> Option<Vec<String>> {
    let attributes = trimmed
        .strip_prefix(CLAIM_OPEN)?
        .strip_suffix(CLAIM_CLOSE)?;
    let mut evidence: Option<Vec<String>> = None;
    for attribute in attributes.split(' ') {
        let (key, value) = attribute.split_once('=')?;
        if key != CLAIM_EVIDENCE || evidence.is_some() {
            return None;
        }
        let mut seen = HashSet::new();
        let slugs: Option<Vec<String>> = value
            .split(',')
            .map(|slug| (is_kebab(slug) && seen.insert(slug)).then(|| slug.to_string()))
            .collect();
        evidence = Some(slugs?);
    }
    evidence
}

/// The paths of a tested-by marker line, trimmed, or `None` when the line is
/// not exactly a marker: one space after the word, then a comma list with no
/// space of distinct paths, each matching the `<item-path>` pattern of the
/// completion gate, and one space before `-->`.
fn parse_tested_by(trimmed: &str) -> Option<Vec<String>> {
    let list = trimmed
        .strip_prefix(TESTED_BY_OPEN)?
        .strip_suffix(TESTED_BY_CLOSE)?;
    let mut seen = HashSet::new();
    split_entries(list)
        .into_iter()
        .map(|path| (is_item_path(path) && seen.insert(path)).then(|| path.to_string()))
        .collect()
}

/// The entries of a comma list, split at each `,` outside a generic argument
/// list, so that `Pair<A,B>::get` is one entry.
fn split_entries(list: &str) -> Vec<&str> {
    let mut entries = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (at, byte) in list.bytes().enumerate() {
        match byte {
            b'<' => depth += 1,
            b'>' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                entries.push(&list[start..at]);
                start = at + 1;
            }
            _ => {}
        }
    }
    entries.push(&list[start..]);
    entries
}

/// Whether `text` matches `<item-path>`: segments `[A-Za-z_][A-Za-z0-9_]*`
/// joined by `::`, each with an optional generic argument list
/// `<[A-Za-z0-9_,':&]*>`, as the completion gate reads a bound path.
fn is_item_path(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut at = 0;
    loop {
        if !bytes
            .get(at)
            .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
        {
            return false;
        }
        at += 1;
        while bytes
            .get(at)
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
        {
            at += 1;
        }
        if bytes.get(at) == Some(&b'<') {
            at += 1;
            while bytes.get(at).is_some_and(|b| {
                b.is_ascii_alphanumeric() || matches!(*b, b'_' | b',' | b'\'' | b':' | b'&')
            }) {
                at += 1;
            }
            if bytes.get(at) != Some(&b'>') {
                return false;
            }
            at += 1;
        }
        if at == bytes.len() {
            return true;
        }
        if bytes[at..].starts_with(b"::") {
            at += 2;
        } else {
            return false;
        }
    }
}

/// The index of the first line of the statement a claim marker on line
/// `index` supports: the run of lines directly above it with visible text,
/// up to a blank line, a heading, a fence, a PROPOSED block or a marker, and
/// no further up than the line that opens the nearest list item. `None` when
/// the line directly above is no part of a statement.
fn statement_start(
    body: &str,
    lines: &[Line],
    visible: &[String],
    in_proposed: &[bool],
    index: usize,
) -> Option<usize> {
    let mut first = None;
    let mut at = index;
    while let Some(above) = at.checked_sub(1) {
        let text = &visible[above];
        if lines[above].fenced
            || in_proposed[above]
            || text.trim().is_empty()
            || atx(text).is_some()
        {
            break;
        }
        first = Some(above);
        if opens_list_item(&body[lines[above].start..lines[above].end]) {
            break;
        }
        at = above;
    }
    first
}

/// The marker of line `index`, outside a PROPOSED block and not a generated
/// marker line: `None` when the line holds no `<!-- raf:` comment, and the
/// error of the comment when it is not a well-formed, attached claim or
/// tested-by marker.
fn raf_marker(
    body: &str,
    lines: &[Line],
    visible: &[String],
    found: &Markers,
    origin: Origin,
    section_of: &[Option<Arc<str>>],
    index: usize,
) -> Result<Option<Found>, ErrorKind> {
    let raw = &body[lines[index].start..lines[index].end];
    if !raw.contains("<!--") {
        return Ok(None);
    }
    // A generated marker starts its line, where the scan reads it; a
    // `raf:generated:` comment anywhere else is text. A marker is one line,
    // so a comment that opens at the end of the line and goes on with a
    // `raf:` word below is a marker error, never a plain comment.
    match raf_comment(raw).or_else(|| split_raf_comment(body, lines, index)) {
        None => Ok(None),
        Some(Family::Other) => Err(ErrorKind::UnknownMarker),
        Some(Family::Claim) => {
            let evidence = parse_claim(raw.trim()).ok_or(ErrorKind::MalformedClaim)?;
            let first = statement_start(body, lines, visible, &found.in_proposed, index)
                .ok_or(ErrorKind::UnattachedClaim)?;
            Ok(Some(Found::Claim(Claim {
                evidence,
                marker: span_of(lines, body.len(), index, index + 1, origin),
                statement: span_of(lines, body.len(), first, index, origin),
                section: section_of[index].clone(),
            })))
        }
        Some(Family::TestedBy) => {
            let paths = parse_tested_by(raw.trim()).ok_or(ErrorKind::MalformedTestedBy)?;
            let statement = tested_statement(body, lines, visible, found, origin, index)
                .ok_or(ErrorKind::UnattachedTestedBy)?;
            Ok(Some(Found::TestedBy(TestedBy {
                paths,
                marker: span_of(lines, body.len(), index, index + 1, origin),
                statement,
                section: section_of[index].clone(),
            })))
        }
    }
}

/// The statement a tested-by marker on line `index` maps: the statement of
/// the claim marker on the line directly above it, the heading line directly
/// above it, or the run of lines a claim marker on its line would support.
/// `None` when there is none, and when a claim marker follows directly: the
/// claim marker goes between the statement and the tested-by marker.
fn tested_statement(
    body: &str,
    lines: &[Line],
    visible: &[String],
    found: &Markers,
    origin: Origin,
    index: usize,
) -> Option<Span> {
    let next = lines.get(index + 1).filter(|line| !line.fenced);
    if next.is_some_and(|line| raf_comment(&body[line.start..line.end]) == Some(Family::Claim)) {
        return None;
    }
    let above = index.checked_sub(1)?;
    if let Some(claim) = found
        .claims
        .last()
        .filter(|claim| claim.line() == origin.line + above)
    {
        return Some(claim.statement.clone());
    }
    if !lines[above].fenced && !found.in_proposed[above] && atx(&visible[above]).is_some() {
        return Some(span_of(lines, body.len(), above, index, origin));
    }
    let first = statement_start(body, lines, visible, &found.in_proposed, index)?;
    Some(span_of(lines, body.len(), first, index, origin))
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

/// Whether `slug` is a lowercase kebab-case slug: one or more parts of ASCII
/// lowercase letters and digits, joined by single `-`, with no `-` at either
/// end. It is the one slug pattern of the profile: the slug of a document id
/// and of a PROPOSED marker, and the kind of a generated marker.
///
/// ```rust
/// use raf_okf::is_kebab;
///
/// assert!(is_kebab("incremental-parsing"));
/// assert!(is_kebab("2fa"));
/// assert!(!is_kebab("-rf"));
/// assert!(!is_kebab("Upper"));
/// assert!(!is_kebab("a--b"));
/// assert!(!is_kebab("../x"));
/// assert!(!is_kebab(""));
/// ```
#[must_use]
pub fn is_kebab(slug: &str) -> bool {
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
    pub(crate) claims: Vec<Claim>,
    pub(crate) tested_by: Vec<TestedBy>,
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

/// Scan the lines outside fences for markers. `visible` is each line with
/// its HTML comments removed, and `section_of` gives the slug of the
/// second-level section that holds each line.
///
/// A `<!-- raf:` comment outside code spans and PROPOSED blocks, in any
/// letter case and spacing, is a marker or an error: a generated marker
/// starts its line (a `raf:generated:` comment elsewhere in a line is text),
/// a claim marker and a tested-by marker are each their whole line, and any
/// other `raf:` comment is an unknown marker, so a misspelt marker is never
/// read as a plain comment. A marker is one line: a comment whose first text
/// after `<!--` is on a later line and is such a `raf:` comment is the same
/// error.
pub(crate) fn scan_markers(
    body: &str,
    lines: &[Line],
    visible: &[String],
    origin: Origin,
    section_of: &[Option<Arc<str>>],
) -> Result<Markers, ParseError> {
    let mut found = Markers {
        proposed: Vec::new(),
        generated: Vec::new(),
        claims: Vec::new(),
        tested_by: Vec::new(),
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
        } else if !found.in_proposed[index] {
            match raf_marker(body, lines, visible, &found, origin, section_of, index)
                .map_err(fail)?
            {
                Some(Found::Claim(claim)) => found.claims.push(claim),
                Some(Found::TestedBy(marker)) => found.tested_by.push(marker),
                None => {}
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
