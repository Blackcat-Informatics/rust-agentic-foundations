// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The generated blocks: what each kind holds, where it may sit, and which
//! blocks of a repository are stale.
//!
//! One pass, [`analyze`], reads the loaded documents and the index. For each
//! block it renders the expected content with the renderer of its kind and
//! compares it with the block. The findings of `check` and the edits of
//! `sync` both come from that pass, so they cannot disagree.
//!
//! # Cell text
//!
//! Text that comes from a document and is written into a table cell (a
//! description, a path, a gap) is escaped by one rule, in [`cell`]:
//!
//! * a backslash becomes `\\`, a pipe `\|`, a bracket `\[` or `\]`;
//! * `<` becomes `&lt;`, so text can never open an HTML comment or a marker;
//! * each line break (`\r\n`, `\n` or `\r`) becomes one space.
//!
//! # Line endings
//!
//! A block is written in the line ending of the body it sits in, taken from
//! the first line break of that body. A body that mixes `\r\n` and `\n` is
//! refused (`mixed-line-endings`) and nothing in it is rewritten.

use std::fmt::Write as _;

use raf_okf::{Body, CapabilitySection, Index, Location, Span, State};

use crate::bindings::ExampleKind;
use crate::finding::kind;
use crate::graph::{BindingKind, ChangesEdge, DependsEdge, Graph};
use crate::model::IndexSlot;
use crate::roadmap::Roadmap;
use crate::{Finding, Workplan};

/// The kind of the verification matrix.
const MATRIX: &str = "verification-matrix";
/// The kind of the relationship table.
const RELATIONSHIPS: &str = "relationships";
/// The kind of the capability catalogue.
const CATALOGUE: &str = "catalogue";

/// The kinds `raf-workplan` owns.
pub(crate) const KINDS: [&str; 3] = [MATRIX, RELATIONSHIPS, CATALOGUE];

/// The path of the capability index.
pub(crate) const INDEX_PATH: &str = "docs/capabilities/index.md";

/// The marker that opens a block of `kind`.
pub(crate) fn open_marker(kind: &str) -> String {
    format!("<!-- raf:generated:{kind} -->")
}

/// The marker that closes a block.
pub(crate) const END_MARKER: &str = "<!-- raf:generated:end -->";

/// A line ending.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Eol {
    Lf,
    CrLf,
}

impl Eol {
    fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
        }
    }
}

/// Where a block is found.
#[derive(Clone, Copy)]
enum Role<'a> {
    /// A capability document of this id.
    Capability(&'a str),
    /// A change record or an architecture page, which take no block.
    Other,
    /// `docs/capabilities/index.md`.
    Index,
}

/// What the pass found.
#[derive(Default)]
pub(crate) struct Analysis {
    /// The findings that stop a rewrite: sync refuses.
    pub(crate) refusals: Vec<Finding>,
    /// The findings that a rewrite repairs: `index-missing`, `generated-stale`.
    pub(crate) repairs: Vec<Finding>,
    /// The new text of every stale file, by path, sorted by path.
    pub(crate) edits: Vec<(String, String)>,
    /// The catalogue content, with `\n` line endings, when the index is
    /// missing and has to be created.
    pub(crate) new_index: Option<String>,
    /// The files whose rewritten text does not read back (for instance because
    /// it would pass the size bound), with the reason.
    pub(crate) failures: Vec<(String, String)>,
    /// The finding of an index that is there and cannot be used. `check`
    /// already reports it among the findings of the load.
    pub(crate) invalid_index: Option<Finding>,
}

impl Analysis {
    /// The findings that belong to `check`: refusals, then repairs.
    pub(crate) fn into_findings(self) -> Vec<Finding> {
        let mut all = self.refusals;
        all.extend(self.repairs);
        all
    }
}

/// Judge every generated block of the loaded documents.
pub(crate) fn analyze(plan: &Workplan<'_>) -> Analysis {
    let roadmap = plan.status();
    let inputs = Inputs {
        graph: plan.graph(),
        roadmap: &roadmap,
    };
    let mut out = Analysis::default();
    for cap in plan.capabilities() {
        let role = Role::Capability(&cap.value().header.id);
        let found = file(&mut out, &inputs, cap.path(), cap.body(), role);
        apply(&mut out, cap.path(), found, |edits| {
            let mut body = cap.body().clone();
            body.replace_spans(edits)
                .map_err(|error| error.to_string())?;
            Ok(format!("{}{}", cap.head(), body.text()))
        });
    }
    // A change record or an architecture page takes no block, so every block
    // in one is a refusal and there is nothing to splice.
    for change in plan.changes() {
        file(&mut out, &inputs, change.path(), change.body(), Role::Other);
    }
    for page in plan.architecture() {
        file(&mut out, &inputs, page.path(), page.body(), Role::Other);
    }
    match &plan.index {
        IndexSlot::Absent => {
            if plan.has_capability_files() {
                out.repairs.push(Finding::new(
                    kind::INDEX_MISSING,
                    INDEX_PATH,
                    Location::Document,
                    "the capability catalogue `docs/capabilities/index.md` does not exist",
                    "run the sync operation (`Workplan::sync`, `make docs-sync`), which creates it",
                ));
                let content = catalogue(&roadmap);
                out.new_index = Some(lines_to_text(&content, Eol::Lf));
            }
        }
        IndexSlot::Invalid(finding) => out.invalid_index = Some(finding.clone()),
        IndexSlot::Valid(index) => {
            let found = file(&mut out, &inputs, INDEX_PATH, index.body(), Role::Index);
            apply(&mut out, INDEX_PATH, found, |edits| {
                edited_index(index, edits)
            });
        }
    }
    out.edits.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// The index with `edits` applied, as text.
fn edited_index(index: &Index, edits: &[(Span, &str)]) -> Result<String, String> {
    let mut index = index.clone();
    index
        .replace_spans(edits)
        .map_err(|error| error.to_string())?;
    Ok(index.render())
}

/// Turn the splices found in a file into its new text, when there are any and
/// the file has no refusal; a text that does not read back is a failure.
fn apply(
    out: &mut Analysis,
    path: &str,
    found: Option<Vec<(Span, String)>>,
    splice: impl FnOnce(&[(Span, &str)]) -> Result<String, String>,
) {
    let Some(found) = found.filter(|found| !found.is_empty()) else {
        return;
    };
    let edits: Vec<(Span, &str)> = found
        .iter()
        .map(|(span, text)| (span.clone(), text.as_str()))
        .collect();
    match splice(&edits) {
        Ok(text) => out.edits.push((path.to_string(), text)),
        Err(reason) => out.failures.push((path.to_string(), reason)),
    }
}

/// What the renderers read.
struct Inputs<'a> {
    graph: &'a Graph,
    roadmap: &'a Roadmap,
}

/// Judge the blocks of one file. Returns the splices that bring its stale
/// blocks up to date, or `None` when the file is refused.
fn file(
    out: &mut Analysis,
    inputs: &Inputs<'_>,
    path: &str,
    body: &Body,
    role: Role<'_>,
) -> Option<Vec<(Span, String)>> {
    let blocks = body.generated_blocks();
    if blocks.is_empty() {
        return None;
    }
    let (eol, mixed) = line_ending(body.text());
    if let Some(offset) = mixed {
        out.refusals.push(Finding::new(
            kind::MIXED_LINE_ENDINGS,
            path,
            Location::Line(body.span().lines.start + offset),
            "the body mixes CRLF and LF line endings, so the generated blocks cannot be written in one",
            "use one line ending throughout the body; the first line break of the body sets it",
        ));
        return None;
    }
    let base = body.span().bytes.start;
    let mut refused = false;
    let mut splices = Vec::new();
    for block in blocks {
        let line = block.outer().lines.start;
        let name = block.kind();
        if !KINDS.contains(&name) {
            refused = true;
            out.refusals.push(Finding::new(
                kind::GENERATED_UNKNOWN_KIND,
                path,
                Location::Line(line),
                format!(
                    "the generated block `{name}` is not a kind raf-workplan writes ({})",
                    KINDS.join(", ")
                ),
                "remove the block, or correct its kind",
            ));
            continue;
        }
        if let Some(problem) = misplaced(name, role, block.outer(), body) {
            refused = true;
            out.refusals.push(Finding::new(
                kind::GENERATED_MISPLACED,
                path,
                Location::Line(line),
                format!("the generated block `{name}` {problem}"),
                "move the block to a place that takes it, or remove it",
            ));
            continue;
        }
        let expected = lines_to_text(&render(name, role, inputs), eol);
        let content = &block.content().bytes;
        let current = &body.text()[content.start - base..content.end - base];
        if current != expected {
            out.repairs.push(Finding::new(
                kind::GENERATED_STALE,
                path,
                Location::Line(line),
                format!("the generated block `{name}` in {path} differs from what sync writes"),
                "run the sync operation (`Workplan::sync`, `make docs-sync`) to rewrite it; never edit inside a block",
            ));
            splices.push((block.content().clone(), expected));
        }
    }
    if refused { None } else { Some(splices) }
}

/// Why a block of `name` may not sit where it is, or `None` when it may.
fn misplaced(name: &str, role: Role<'_>, outer: &Span, body: &Body) -> Option<&'static str> {
    match (name, role) {
        (CATALOGUE, Role::Index) => None,
        (CATALOGUE, _) => Some("belongs in docs/capabilities/index.md only"),
        (_, Role::Index) => Some("belongs in a capability document, not the index"),
        (_, Role::Other) => Some("belongs in a capability document only"),
        (_, Role::Capability(_)) => {
            let inside = body.sections().iter().any(|section| {
                let content = &section.content().bytes;
                content.start <= outer.bytes.start && outer.bytes.end <= content.end
            });
            (!inside)
                .then_some("belongs inside a section of the document, below a second-level heading")
        }
    }
}

/// The line ending of a body, from its first line break, and the offset (in
/// lines from the start of the body) of the first line break that differs from
/// it. A body with no line break is `\n`.
fn line_ending(text: &str) -> (Eol, Option<usize>) {
    let mut first = None;
    for (offset, raw) in text.split_inclusive('\n').enumerate() {
        let Some(line) = raw.strip_suffix('\n') else {
            continue;
        };
        let eol = if line.ends_with('\r') {
            Eol::CrLf
        } else {
            Eol::Lf
        };
        match first {
            None => first = Some(eol),
            Some(seen) if seen != eol => return (seen, Some(offset)),
            Some(_) => {}
        }
    }
    (first.unwrap_or(Eol::Lf), None)
}

/// The lines, each ended with `eol`.
fn lines_to_text(lines: &[String], eol: Eol) -> String {
    let mut out = String::new();
    for line in lines {
        out.push_str(line);
        out.push_str(eol.as_str());
    }
    out
}

/// The lines a block of `name` holds in a file of `role`.
fn render(name: &str, role: Role<'_>, inputs: &Inputs<'_>) -> Vec<String> {
    match (name, role) {
        (MATRIX, Role::Capability(id)) => matrix(inputs.graph, id),
        (RELATIONSHIPS, Role::Capability(id)) => relationships(inputs.graph, id),
        _ => catalogue(inputs.roadmap),
    }
}

/// The text of a table cell: see the module documentation for the rule.
pub(crate) fn cell(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => out.push_str("\\\\"),
            '|' => out.push_str("\\|"),
            '[' => out.push_str("\\["),
            ']' => out.push_str("\\]"),
            '<' => out.push_str("&lt;"),
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push(' ');
            }
            '\n' => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

/// `text` as a code span in a table cell: escaped like any cell text, in a
/// backtick fence longer than any backtick run inside, padded with a space
/// when the text starts or ends with a backtick (`CommonMark`).
pub(crate) fn code_span(text: &str) -> String {
    let mut longest = 0;
    let mut run = 0;
    for c in text.chars() {
        run = if c == '`' { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    let fence = "`".repeat(longest + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{fence}{pad}{}{pad}{fence}", cell(text))
}

/// The slug of an id: the part after the prefix.
fn slug_of(id: &str) -> &str {
    id.split_once(':').map_or(id, |(_, slug)| slug)
}

/// The `verification-matrix` block: one row per `tests` and `examples` entry.
fn matrix(graph: &Graph, id: &str) -> Vec<String> {
    let mut rows: Vec<(String, &'static str, String)> = Vec::new();
    for binding in graph.bindings_of(id) {
        let label = match binding.kind() {
            BindingKind::Code => continue,
            BindingKind::Test => "test",
            BindingKind::Example => {
                let doctest = binding
                    .items()
                    .iter()
                    .filter_map(|item| binding.example_kind(*item))
                    .any(|how| how == ExampleKind::Doctest);
                // An example that does not resolve is `binding-unresolved`; its
                // row reads as the common kind.
                if doctest { "doctest" } else { "example target" }
            }
        };
        let sections: Vec<String> = binding
            .sections()
            .iter()
            .map(|section| section_link(*section))
            .collect();
        rows.push((binding.path().to_string(), label, sections.join(", ")));
    }
    rows.sort_by(|a, b| (&a.0, a.1).cmp(&(&b.0, b.1)));
    let mut lines = vec![
        "| Item | Kind | Section |".to_string(),
        "| --- | --- | --- |".to_string(),
    ];
    for (path, label, sections) in rows {
        lines.push(format!("| {} | {label} | {sections} |", code_span(&path)));
    }
    lines
}

/// A link to a section of the document it is in.
fn section_link(section: CapabilitySection) -> String {
    format!("[{}](#{})", section.heading(), section.slug())
}

/// The `relationships` block: what the capability depends on, what depends on
/// it, and the open changes that list it.
fn relationships(graph: &Graph, id: &str) -> Vec<String> {
    let capability_link = |other: &str| {
        let slug = slug_of(other);
        format!("[{slug}](./{slug}.md)")
    };
    let mut depends: Vec<&str> = graph
        .depends()
        .iter()
        .filter(|edge| edge.from() == id)
        .map(DependsEdge::to)
        .collect();
    let mut depended: Vec<&str> = graph
        .depends()
        .iter()
        .filter(|edge| edge.to() == id)
        .map(DependsEdge::from)
        .collect();
    let mut changed: Vec<&str> = graph
        .changes_edges()
        .iter()
        .filter(|edge| edge.capability() == id)
        .map(ChangesEdge::change)
        .collect();
    for list in [&mut depends, &mut depended, &mut changed] {
        list.sort_by_key(|other| slug_of(other));
    }
    let mut lines = vec![
        "| Relation | Capability |".to_string(),
        "| --- | --- |".to_string(),
    ];
    for other in depends {
        lines.push(format!("| Depends on | {} |", capability_link(other)));
    }
    for other in depended {
        lines.push(format!("| Depended on by | {} |", capability_link(other)));
    }
    for change in changed {
        let slug = slug_of(change);
        lines.push(format!(
            "| Changed by | [{slug}](../../work/changes/{slug}.md) |"
        ));
    }
    lines
}

/// The `catalogue` block: one table per state that has a capability.
fn catalogue(roadmap: &Roadmap) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for (state, entries) in roadmap.groups() {
        if entries.is_empty() {
            continue;
        }
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(format!("## {}", heading_of(state)));
        lines.push(String::new());
        lines.push("| Capability | Description | Gaps |".to_string());
        lines.push("| --- | --- | --- |".to_string());
        for entry in entries {
            let gaps: Vec<String> = entry.gaps().iter().map(ToString::to_string).collect();
            let mut row = String::new();
            // Writing to a `String` cannot fail.
            let _ = write!(
                row,
                "| [{slug}](./{target}) | {description} | {gaps} |",
                slug = cell(entry.slug()),
                target = entry.link_target(),
                description = cell(entry.description()),
                gaps = cell(&gaps.join("; ")),
            );
            lines.push(row);
        }
    }
    lines
}

/// The heading of a state group: its name with a capital letter.
fn heading_of(state: State) -> String {
    let name = state.name();
    let mut chars = name.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_ascii_uppercase().to_string() + chars.as_str()
    })
}

/// The text of a new index: the two SPDX comment lines, a blank line and the
/// `catalogue` block, with `\n` line endings.
pub(crate) fn new_index_text(copyright: &str, licence: &str, content: &str) -> String {
    // The tag names are joined here so that this source does not carry them
    // as a licence header of its own.
    let copyright_tag = concat!("SPDX-", "FileCopyrightText:");
    let licence_tag = concat!("SPDX-", "License-Identifier:");
    format!(
        "<!-- {copyright_tag} {copyright} -->\n<!-- {licence_tag} {licence} -->\n\n{}\n{content}{END_MARKER}\n",
        open_marker(CATALOGUE)
    )
}

#[cfg(test)]
mod tests {
    use super::{Eol, cell, code_span, heading_of, line_ending};
    use raf_okf::State;

    #[test]
    fn cell_text_is_escaped_by_the_documented_rule() {
        assert_eq!(cell("plain text"), "plain text");
        assert_eq!(cell("a|b"), "a\\|b");
        assert_eq!(cell("back\\slash"), "back\\\\slash");
        assert_eq!(cell("[x](y)"), "\\[x\\](y)");
        assert_eq!(cell("<!-- x -->"), "&lt;!-- x -->");
        assert_eq!(cell("one\ntwo\r\nthree\rfour"), "one two three four");
        assert_eq!(cell("\r\n\n"), "  ");
        assert_eq!(cell("é ü"), "é ü");
    }

    #[test]
    fn the_line_ending_is_the_first_break_and_a_change_is_mixed() {
        assert!(line_ending("") == (Eol::Lf, None));
        assert!(line_ending("no break") == (Eol::Lf, None));
        assert!(line_ending("a\nb\n") == (Eol::Lf, None));
        assert!(line_ending("a\r\nb\r\nc") == (Eol::CrLf, None));
        assert!(line_ending("a\nb\r\nc\n") == (Eol::Lf, Some(1)));
        assert!(line_ending("a\r\nb\nc\r\n") == (Eol::CrLf, Some(1)));
    }

    #[test]
    fn a_code_span_fence_is_longer_than_any_backtick_run_inside() {
        assert_eq!(code_span("a::b"), "`a::b`");
        assert_eq!(code_span("a`b"), "``a`b``");
        assert_eq!(code_span("`edge"), "`` `edge ``");
        assert_eq!(code_span("two``ticks"), "```two``ticks```");
        assert_eq!(code_span("p|q"), "`p\\|q`");
    }

    #[test]
    fn a_state_heading_has_a_capital() {
        assert_eq!(heading_of(State::Delivered), "Delivered");
        assert_eq!(heading_of(State::Considered), "Considered");
    }
}
