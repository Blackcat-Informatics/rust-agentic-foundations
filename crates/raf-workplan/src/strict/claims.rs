// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Claim coverage: the claims of the Contract and the Constraints of a
//! delivered capability, the tested-by markers that cover them, the findings
//! of the complete policy for a claim that no marker covers and for a Contract that
//! is one long claim, and the report of every claim. See the documentation of
//! [`crate::strict`].

use std::collections::BTreeSet;
use std::ops::Range;

use raf_okf::{Body, Capability, CapabilitySection, Location, Section, State};

use super::{MIN_SECTION_WORDS, is_thematic_break, list_item};
use crate::debt::Owed;
use crate::finding::kind;
use crate::obligation::tense::word_spans;
use crate::{Finding, Loaded, Workplan};

/// How many characters of the lead or the item a finding quotes.
const LEAD_CHARS: usize = 60;

/// How many hexadecimal digits of the digest the key of a Constraints unit
/// holds.
const DIGEST_DIGITS: usize = 16;

/// One claim of a delivered capability, the tests its markers name, and where
/// it starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimCoverage {
    capability: String,
    section: CapabilitySection,
    claim: String,
    line: usize,
    tests: Vec<String>,
}

impl ClaimCoverage {
    /// The id of the capability, `cap:<slug>`.
    #[must_use]
    pub fn capability(&self) -> &str {
        &self.capability
    }

    /// The section that holds the claim: Contract or Constraints.
    #[must_use]
    pub fn section(&self) -> CapabilitySection {
        self.section
    }

    /// The key of the claim within its section: the kebab slug of a bold lead
    /// or a heading, `opening`, or `c-` and 16 hexadecimal digits of the
    /// digest of a Constraints unit, with `-2`, `-3` and on for a repeat.
    #[must_use]
    pub fn claim(&self) -> &str {
        &self.claim
    }

    /// The whole key of the claim, `cap:<slug>#<section>/<claim>`, the
    /// subject a debt row of the claim names.
    #[must_use]
    pub fn key(&self) -> String {
        format!("{}#{}/{}", self.capability, self.section.slug(), self.claim)
    }

    /// The 1-based line of the first line of the claim in its document.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }

    /// The paths the tested-by markers of the claim name, in document order,
    /// each once; empty for a claim that no marker covers.
    #[must_use]
    pub fn tests(&self) -> &[String] {
        &self.tests
    }

    /// Whether a tested-by marker covers the claim.
    #[must_use]
    pub fn is_covered(&self) -> bool {
        !self.tests.is_empty()
    }
}

/// Every claim of every delivered capability, as [`Workplan::claims_report`]
/// gives it: sorted by capability, by the slug of the section and by line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClaimsReport {
    claims: Vec<ClaimCoverage>,
}

impl ClaimsReport {
    /// The claims, in the order of the report.
    #[must_use]
    pub fn claims(&self) -> &[ClaimCoverage] {
        &self.claims
    }

    /// The claims of the capability `id` alone, in the same order.
    #[must_use]
    pub fn capability(&self, id: &str) -> Self {
        Self {
            claims: self
                .claims
                .iter()
                .filter(|claim| claim.capability == id)
                .cloned()
                .collect(),
        }
    }

    /// Whether every claim names a test.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.claims.iter().all(ClaimCoverage::is_covered)
    }
}

impl Workplan<'_> {
    /// Every claim of every loaded `delivered` capability: each clause of its
    /// Contract and each unit of its Constraints, with its key, its first line
    /// and the paths its tested-by markers name, under the complete policy. It
    /// is the report of `cargo workplan coverage claims`, and it writes
    /// nothing.
    ///
    /// ```rust
    /// use raf_source::{MemoryTree, Workspace};
    /// use raf_workplan::Workplan;
    ///
    /// let source = Workspace::load(
    ///     &MemoryTree::from([
    ///         ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///         ("src/lib.rs", "pub fn run() {}\n"),
    ///         ("tests/it.rs", "#[test]\nfn case() {}\n"),
    ///     ]),
    ///     &["."],
    /// )?;
    /// let section = |name: &str, text: &str| format!("## {name}\n\n{text}\n\n");
    /// let document = format!(
    ///     "---\ntype: Capability\nid: cap:demo\ntitle: Demo\ndescription: Runs.\n\
    ///      state: delivered\nbindings:\n  code: [demo::run]\n  tests: [demo::test_crate::it::case]\n  \
    ///      examples: []\ndepends: []\n---\n\n# Demo\n\n{}{}{}{}{}{}{}",
    ///     section("Purpose", "Runs."),
    ///     section(
    ///         "Contract",
    ///         "**Running.** It runs.\n<!-- raf:tested-by demo::test_crate::it::case -->\n\n\
    ///          **Stopping.** It stops.",
    ///     ),
    ///     section("Usage", "Call it."),
    ///     section("Examples", "None."),
    ///     section("Constraints", "- One at a time."),
    ///     section("Explanation", "Simple."),
    ///     section("Verification", "Tested."),
    /// );
    /// let tree = MemoryTree::from([("docs/capabilities/demo.md", document.as_str())]);
    /// let report = Workplan::load(&tree, &source).claims_report();
    /// let claims: Vec<(String, &[String])> = report
    ///     .claims()
    ///     .iter()
    ///     .map(|claim| (claim.key(), claim.tests()))
    ///     .collect();
    /// assert_eq!(claims[1].0, "cap:demo#contract/running");
    /// assert_eq!(claims[1].1, ["demo::test_crate::it::case"]);
    /// assert_eq!(claims[2].0, "cap:demo#contract/stopping");
    /// assert!(claims[2].1.is_empty());
    /// assert!(!report.is_complete());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn claims_report(&self) -> ClaimsReport {
        let mut claims: Vec<ClaimCoverage> = self
            .capabilities()
            .iter()
            .filter(|loaded| loaded.value().state == State::Delivered)
            .flat_map(|loaded| {
                let id = &loaded.value().header.id;
                units(loaded.body())
                    .units
                    .into_iter()
                    .map(|unit| ClaimCoverage {
                        capability: id.clone(),
                        section: unit.section,
                        claim: unit.key,
                        line: unit.line,
                        tests: unit.tests,
                    })
            })
            .collect();
        claims.sort_by(|a, b| {
            (&a.capability, a.section.slug(), a.line).cmp(&(
                &b.capability,
                b.section.slug(),
                b.line,
            ))
        });
        ClaimsReport { claims }
    }
}

/// The findings of claim coverage for each delivered capability, in the order
/// of the load: `strict-claim-unit-collapsed` for a Contract with no clause
/// start and more than [`MIN_SECTION_WORDS`] words, then
/// `strict-claim-uncovered` for each claim no marker covers, in document
/// order.
pub(super) fn check(plan: &Workplan<'_>) -> Vec<Owed> {
    let mut out = Vec::new();
    for loaded in plan.capabilities() {
        if loaded.value().state == State::Delivered {
            judge(loaded, &mut out);
        }
    }
    out
}

/// The claim findings of one delivered capability.
fn judge(loaded: &Loaded<Capability>, out: &mut Vec<Owed>) {
    let path = loaded.path();
    let id = &loaded.value().header.id;
    let found = units(loaded.body());
    if let Some(words) = found.collapsed {
        out.push(Owed::held(
            Finding::new(
                kind::STRICT_CLAIM_UNIT_COLLAPSED,
                path,
                Location::Section(CapabilitySection::Contract.heading().to_string()),
                format!(
                    "the Contract holds {words} words of prose and no clause start, more than {MIN_SECTION_WORDS}, so it is one claim that one marker covers"
                ),
                "start each guarantee of the Contract with a paragraph that opens with a bold lead (`**The guarantee.**`) or with a `###` heading, so that each clause is a claim of its own with its own tested-by marker",
            ),
            format!("{id}#{}", CapabilitySection::Contract.slug()),
        ));
    }
    for unit in found.units.iter().filter(|unit| unit.tests.is_empty()) {
        let key = format!("{id}#{}/{}", unit.section.slug(), unit.key);
        out.push(Owed::held(
            Finding::new(
                kind::STRICT_CLAIM_UNCOVERED,
                path,
                Location::Line(unit.line),
                format!("the claim `{key}` (\"{}\") names no test", unit.lead),
                "put `<!-- raf:tested-by <path>[,<path>...] -->` after the statement, naming the bound tests or examples that show it, or move reasoning that no test can show into the Explanation",
            ),
            key,
        ));
    }
}

/// One claim unit of a body.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Unit {
    section: CapabilitySection,
    key: String,
    /// The first line of the unit, 1-based in the whole text.
    line: usize,
    /// The lines of the unit, 1-based in the whole text, end exclusive.
    lines: Range<usize>,
    /// The first [`LEAD_CHARS`] characters of its lead or item.
    lead: String,
    tests: Vec<String>,
}

/// The claim units of a body and whether its Contract collapsed.
#[derive(Debug, Default)]
struct Units {
    units: Vec<Unit>,
    /// The words of a Contract with no clause start, when they are more than
    /// [`MIN_SECTION_WORDS`].
    collapsed: Option<usize>,
}

/// What a line of a section is to the unit rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Block {
    /// Authored Markdown.
    Text,
    /// A line of fenced code, its fences included.
    Fenced,
    /// A line of a PROPOSED block or of a generated block, its markers
    /// included: not authored prose of the current contract.
    Hidden,
}

/// One line of a section.
struct Row<'b> {
    /// 1-based in the whole text.
    line: usize,
    raw: &'b str,
    /// The line with each comment that opens and closes on it removed.
    text: String,
    block: Block,
}

impl Row<'_> {
    /// Blank, or only comments.
    fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }

    fn indent(&self) -> usize {
        self.raw.len() - self.raw.trim_start_matches([' ', '\t']).len()
    }
}

/// The claim units of the Contract and the Constraints of `body`, each with
/// the paths of the markers that cover it.
fn units(body: &Body) -> Units {
    let mut found = Units::default();
    for section in body.sections() {
        let rows = rows(body, section);
        match CapabilitySection::from_slug(section.slug()) {
            Some(CapabilitySection::Contract) => {
                let (units, collapsed) = contract(body, section, &rows);
                found.units.extend(units);
                found.collapsed = found.collapsed.or(collapsed);
            }
            Some(CapabilitySection::Constraints) => found.units.extend(constraints(&rows)),
            _ => {}
        }
    }
    number_repeats(&mut found.units);
    cover(body, &mut found.units);
    found
}

/// The lines of a section below its heading.
fn rows<'b>(body: &'b Body, section: &Section) -> Vec<Row<'b>> {
    let Some(text) = body.section_text(section) else {
        return Vec::new();
    };
    let first = section.content().lines.start;
    let within = |lines: &Range<usize>, line: usize| lines.contains(&line);
    text.split_inclusive('\n')
        .enumerate()
        .map(|(index, raw)| {
            let line = first + index;
            let raw = raw.trim_end_matches(['\n', '\r']);
            let hidden = body
                .proposed_blocks()
                .iter()
                .any(|block| within(&block.span().lines, line))
                || body
                    .generated_blocks()
                    .iter()
                    .any(|block| within(&block.outer().lines, line));
            let fenced = body
                .fenced_blocks()
                .iter()
                .any(|block| within(&block.span().lines, line));
            let block = if hidden {
                Block::Hidden
            } else if fenced {
                Block::Fenced
            } else {
                Block::Text
            };
            Row {
                line,
                raw,
                text: without_comments(raw),
                block,
            }
        })
        .collect()
}

/// `raw` with each comment that opens and closes on the line removed.
fn without_comments(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(open) = rest.find("<!--") {
        let Some(close) = rest[open + 4..].find("-->") else {
            break;
        };
        out.push_str(&rest[..open]);
        rest = &rest[open + 4 + close + 3..];
    }
    out.push_str(rest);
    out
}

/// The clause starts and the opening of the Contract, and its words when it
/// has no clause start and is longer than [`MIN_SECTION_WORDS`].
fn contract(body: &Body, section: &Section, rows: &[Row<'_>]) -> (Vec<Unit>, Option<usize>) {
    let end = section.content().lines.end;
    // (line, text, whether it starts a clause): every heading ends a
    // paragraph, and a `###` or `####` heading starts a clause.
    let headings: Vec<(usize, &str, bool)> = body
        .headings()
        .iter()
        .map(|heading| {
            (
                heading.line(),
                heading.text(),
                matches!(heading.level(), 3 | 4),
            )
        })
        .collect();
    // (index of the row, key base, lead)
    let mut starts: Vec<(usize, String, String)> = Vec::new();
    let mut boundary = true;
    for (index, row) in rows.iter().enumerate() {
        if row.block != Block::Text {
            boundary = true;
            continue;
        }
        if let Some((_, text, clause)) = headings.iter().find(|(line, ..)| *line == row.line) {
            if *clause {
                starts.push((index, kebab(text), lead(text)));
            }
            boundary = true;
            continue;
        }
        if row.is_empty() {
            boundary = true;
            continue;
        }
        if boundary && let Some(bold) = bold_lead(row.raw) {
            starts.push((index, kebab(bold), lead(&paragraph(rows, index))));
        }
        boundary = false;
    }
    let content = |range: Range<usize>| {
        rows[range]
            .iter()
            .position(|row| row.block == Block::Text && !row.is_empty())
    };
    let mut units = Vec::new();
    let first_start = starts.first().map_or(rows.len(), |(index, _, _)| *index);
    if let Some(at) = content(0..first_start) {
        units.push(Unit {
            section: CapabilitySection::Contract,
            key: "opening".to_string(),
            line: rows[at].line,
            lines: rows[at].line
                ..starts
                    .first()
                    .map_or(end, |(index, _, _)| rows[*index].line),
            lead: lead(&paragraph(rows, at)),
            tests: Vec::new(),
        });
    }
    for (position, (index, key, text)) in starts.iter().enumerate() {
        let until = starts
            .get(position + 1)
            .map_or(end, |(next, _, _)| rows[*next].line);
        units.push(Unit {
            section: CapabilitySection::Contract,
            key: key.clone(),
            line: rows[*index].line,
            lines: rows[*index].line..until,
            lead: text.clone(),
            tests: Vec::new(),
        });
    }
    let collapsed = if starts.is_empty() {
        let words = contract_words(body, section);
        (words > MIN_SECTION_WORDS).then_some(words)
    } else {
        None
    };
    (units, collapsed)
}

/// The words of prose of the Contract, counted as the section rule of strict
/// mode counts them.
fn contract_words(body: &Body, section: &Section) -> usize {
    body.prose_lines()
        .iter()
        .filter(|line| {
            line.section() == Some(section.slug())
                && line.line() != section.line()
                && !line.in_proposed()
                && !line.in_generated()
        })
        .map(|line| word_spans(line.text()).len())
        .sum()
}

/// The bold run a paragraph line opens with, `**The lead.**`, or `None`. The
/// line is not indented, so a paragraph of a list item is never a clause.
fn bold_lead(raw: &str) -> Option<&str> {
    let rest = raw.strip_prefix("**")?;
    let close = rest.find("**")?;
    let lead = rest[..close].trim();
    (!lead.is_empty()).then_some(lead)
}

/// The text of the paragraph that starts at row `index`: its lines up to a
/// blank line, a comment line or another block.
fn paragraph(rows: &[Row<'_>], index: usize) -> String {
    rows[index..]
        .iter()
        .take_while(|row| row.block == Block::Text && !row.is_empty())
        .map(|row| row.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// What a Constraints unit is while it is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// A top-level list item and the column its content starts at.
    Item(usize),
    Paragraph,
    Table,
}

/// The units of the Constraints: each top-level list item, and each
/// paragraph or table outside a list. A blank line or a comment line ends a
/// paragraph or a table, after which a list item goes on only with the lines
/// indented to its content; a heading, a fence and a hidden block end every
/// unit, and a thematic break and an item with nothing after its marker end
/// the open unit and open none.
fn constraints(rows: &[Row<'_>]) -> Vec<Unit> {
    let mut units = Vec::new();
    // (shape, first row, last row with text, text)
    let mut open: Option<(Shape, usize, usize, Vec<String>)> = None;
    // A blank line, or a comment line, since the last line of text: a list
    // item continues after one only with lines indented to its content.
    let mut after_blank = false;
    let close = |open: &mut Option<(Shape, usize, usize, Vec<String>)>, units: &mut Vec<Unit>| {
        if let Some((_, first, last, text)) = open.take() {
            units.push(constraint_unit(rows, first, last, &text));
        }
    };
    for (index, row) in rows.iter().enumerate() {
        let in_item = matches!(open, Some((Shape::Item(_), ..)));
        match row.block {
            Block::Hidden => {
                close(&mut open, &mut units);
                after_blank = false;
                continue;
            }
            Block::Fenced => {
                // A fence indented under a list item belongs to the item;
                // any other fence ends the unit and is no unit of its own.
                if in_item && row.indent() > 0 {
                    if let Some((_, _, last, _)) = open.as_mut() {
                        *last = index;
                    }
                } else {
                    close(&mut open, &mut units);
                }
                after_blank = false;
                continue;
            }
            Block::Text => {}
        }
        if row.raw.trim().is_empty() {
            after_blank = true;
            if !in_item {
                close(&mut open, &mut units);
            }
            continue;
        }
        if row.is_empty() {
            // A comment line, such as a marker, ends a paragraph or a table;
            // a list item goes on with the lines indented to its content.
            after_blank = true;
            if !in_item {
                close(&mut open, &mut units);
            }
            continue;
        }
        let trimmed = row.raw.trim_start();
        if trimmed.starts_with('#') && is_heading(trimmed) {
            close(&mut open, &mut units);
            after_blank = false;
            continue;
        }
        // A thematic break and a list item with nothing after its marker
        // state nothing: each ends the open unit and opens none.
        let item = list_item(row.raw);
        if is_thematic_break(row.raw) || item.is_some_and(|item| item.content.trim().is_empty()) {
            close(&mut open, &mut units);
            after_blank = false;
            continue;
        }
        let item = item.map(|item| item.column);
        let starts_item = match (item, open.as_ref()) {
            (Some(_), Some((Shape::Item(column), ..))) => row.indent() < *column,
            (Some(_), _) => true,
            (None, _) => false,
        };
        let continues = match open.as_ref() {
            Some((Shape::Item(column), ..)) => !after_blank || row.indent() >= *column,
            Some((Shape::Paragraph | Shape::Table, ..)) => !after_blank,
            None => false,
        };
        if starts_item {
            close(&mut open, &mut units);
            let column = item.unwrap_or_default();
            open = Some((Shape::Item(column), index, index, vec![row.text.clone()]));
        } else if continues {
            if let Some((_, _, last, text)) = open.as_mut() {
                *last = index;
                text.push(row.text.clone());
            }
        } else {
            close(&mut open, &mut units);
            let shape = if trimmed.starts_with('|') {
                Shape::Table
            } else {
                Shape::Paragraph
            };
            open = Some((shape, index, index, vec![row.text.clone()]));
        }
        after_blank = false;
    }
    close(&mut open, &mut units);
    units
}

/// Whether a line, without its indentation, is an ATX heading.
fn is_heading(trimmed: &str) -> bool {
    let marks = trimmed.bytes().take_while(|b| *b == b'#').count();
    (1..=6).contains(&marks) && trimmed[marks..].starts_with([' ', '\t'])
}

/// The unit of the Constraints from row `first` to row `last`, its key the
/// digest of its text with each run of whitespace read as one space.
fn constraint_unit(rows: &[Row<'_>], first: usize, last: usize, text: &[String]) -> Unit {
    let folded = fold(&text.join(" "));
    let hash = purrdf_core::content_store::ContentDigest::of(folded.as_bytes());
    let mut key = String::from("c-");
    purrdf_hash::hex::encode_into(&hash.as_bytes()[..DIGEST_DIGITS / 2], &mut key);
    let item = folded
        .strip_prefix(['-', '*', '+'])
        .map_or(folded.as_str(), str::trim_start);
    Unit {
        section: CapabilitySection::Constraints,
        key,
        line: rows[first].line,
        lines: rows[first].line..rows[last].line + 1,
        lead: lead(item),
        tests: Vec::new(),
    }
}

/// `text` with each run of whitespace read as one space, trimmed.
fn fold(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The first [`LEAD_CHARS`] characters of `text`, folded, without the `**` of
/// bold runs and without a space they end in.
fn lead(text: &str) -> String {
    let cut: String = fold(&text.replace("**", ""))
        .chars()
        .take(LEAD_CHARS)
        .collect();
    cut.trim_end().to_string()
}

/// The kebab slug of a lead or a heading: its letters and digits in lowercase,
/// each other run of characters one `-`, none at either end; `clause` for a
/// text with no letter or digit.
fn kebab(text: &str) -> String {
    let mut out = String::new();
    let mut gap = false;
    for c in text.chars() {
        if c.is_alphanumeric() {
            if gap && !out.is_empty() {
                out.push('-');
            }
            gap = false;
            out.extend(c.to_lowercase());
        } else {
            gap = true;
        }
    }
    if out.is_empty() {
        "clause".to_string()
    } else {
        out
    }
}

/// Number the repeats of a key within its section, in document order: a key
/// that an earlier unit of the section has takes the first of `-2`, `-3` and on
/// that is no key a unit of the section has or takes, so the keys of a section
/// are distinct whatever its leads are (`**Foo.**`, `**Foo.**` and `**Foo 2.**`
/// give `foo`, `foo-3` and `foo-2`).
fn number_repeats(units: &mut [Unit]) {
    let mut taken: BTreeSet<(&'static str, String)> = units
        .iter()
        .map(|unit| (unit.section.slug(), unit.key.clone()))
        .collect();
    let mut seen: BTreeSet<(&'static str, String)> = BTreeSet::new();
    for unit in units {
        let section = unit.section.slug();
        if seen.insert((section, unit.key.clone())) {
            continue;
        }
        // Fewer keys are taken than the numbers tried, so one is free.
        let key = (2..=taken.len() + 2)
            .map(|count| format!("{}-{count}", unit.key))
            .find(|key| !taken.contains(&(section, key.clone())))
            .unwrap_or_default();
        taken.insert((section, key.clone()));
        seen.insert((section, key.clone()));
        unit.key = key;
    }
}

/// The paths each unit's markers name: a Contract clause holds the markers on
/// its lines, and a Constraints unit the markers whose statement starts in it.
fn cover(body: &Body, units: &mut [Unit]) {
    for marker in body.tested_by() {
        let section = marker.section().and_then(CapabilitySection::from_slug);
        let line = match section {
            Some(CapabilitySection::Contract) => marker.line(),
            Some(CapabilitySection::Constraints) => marker.statement().lines.start,
            _ => continue,
        };
        if let Some(unit) = units
            .iter_mut()
            .find(|unit| Some(unit.section) == section && unit.lines.contains(&line))
        {
            for path in marker.paths() {
                if !unit.tests.contains(path) {
                    unit.tests.push(path.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::list_item;
    use super::{Body, kebab, lead, units};

    /// The column the content of the list item `raw` opens starts at.
    fn list_content(raw: &str) -> Option<usize> {
        list_item(raw).map(|item| item.column)
    }

    /// The section, key, first line and tests of each unit of `text`.
    fn read(text: &str) -> Vec<(&'static str, String, usize, Vec<String>)> {
        let body = Body::parse(text).expect("a body");
        units(&body)
            .units
            .into_iter()
            .map(|unit| (unit.section.slug(), unit.key, unit.line, unit.tests))
            .collect()
    }

    #[test]
    fn a_key_is_the_kebab_slug_of_its_lead_and_a_lead_is_sixty_characters() {
        assert_eq!(kebab("The public surface."), "the-public-surface");
        assert_eq!(
            kebab("`Gate::strict` and the floor"),
            "gate-strict-and-the-floor"
        );
        assert_eq!(kebab("..."), "clause");
        let long =
            "**Chunked input.** The parser accepts input in arbitrary chunks and keeps its state.";
        assert_eq!(
            lead(long),
            "Chunked input. The parser accepts input in arbitrary chunks"
        );
        assert_eq!(lead(&long.replace("chunks", "chunked")).chars().count(), 60);
    }

    #[test]
    fn a_list_item_opens_at_its_marker_and_its_content_starts_after_the_space() {
        assert_eq!(list_content("- An item."), Some(2));
        assert_eq!(list_content("  * Nested."), Some(4));
        assert_eq!(list_content("10. Tenth."), Some(4));
        assert_eq!(list_content("-no space"), None);
        assert_eq!(list_content("**Bold.**"), None);
        assert_eq!(list_content("1234567890. Too long."), None);
    }

    #[test]
    fn a_unit_ends_on_the_line_before_the_next_unit() {
        let text = "## Contract\n\n**One.** First.\n\nMore of one.\n<!-- raf:tested-by a::b -->\n**Two.** Second.\n";
        assert_eq!(
            read(text),
            [
                ("contract", "one".to_string(), 3, vec!["a::b".to_string()]),
                ("contract", "two".to_string(), 7, vec![]),
            ]
        );
    }
}
