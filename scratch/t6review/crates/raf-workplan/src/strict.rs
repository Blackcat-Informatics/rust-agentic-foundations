// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Strict mode: the harder rules a repository is judged by when
//! [`Workplan::strictness`] is [`Strictness::Strict`](crate::Strictness).
//!
//! [`Workplan::check`] judges them after every rule of the document model, and
//! only under strict; the load judges none of them, so [`Workplan::findings`]
//! is the same under either strictness and a load under `default` does no work
//! for them. The rules of this module are the only ones that raise a finding
//! under strict that `default` does not raise.
//!
//! # Examples, the matrix and the sections
//!
//! For each loaded `delivered` capability:
//!
//! * `strict-example-missing` at the key `bindings`: `bindings.examples` is
//!   empty. That a bound example runs is proven where the default rules prove
//!   it, by `verify` and condition 5 of the completion gate, which refuse a
//!   documentation test that is `no_run` or `compile_fail`.
//! * `strict-matrix-missing` at the section Verification: the Verification
//!   section holds no `verification-matrix` generated block.
//! * `strict-section-thin` at the section: one of the seven sections holds
//!   fewer than [`MIN_SECTION_WORDS`] words of prose. The prose of a section
//!   is its lines below its heading outside fenced code, HTML comments, code
//!   spans, PROPOSED blocks and generated blocks, cut into words as the tense
//!   scan of reconcile cuts them: runs of letters, digits and `_`, with the
//!   destination of a link left out.
//! * `strict-usage-unbound` at the section Usage: no path of
//!   `bindings.examples` substantiates Usage, that is, none is named in a code
//!   span of the Usage section outside an HTML comment, so the verification
//!   matrix lists Usage for no example that `verify` runs.
//! * `strict-limitations-missing` at the section Constraints: the Constraints
//!   section holds no list item with text or a code span after its marker; an
//!   empty item and a thematic break such as `* * *` are none.
//!
//! # Claims
//!
//! A claim is a unit of the authored prose of the Contract and the
//! Constraints of a delivered capability, read outside fenced code, PROPOSED
//! blocks and generated blocks. In the Contract it is a clause: it starts at a
//! paragraph whose first line opens with a bold run (`**The setting.**`) or
//! at a `###` or `####` heading outside fenced code, and runs to the next
//! clause start or the end of the section; text before the first clause start
//! is one clause, the opening, and the statement of a heading clause is its
//! heading line. In the Constraints it is each top-level list item and each
//! paragraph or table outside a list; a thematic break and a list item with
//! nothing after its marker are none. A blank line, a comment line, a heading of
//! any level, a fence and a hidden block each end a paragraph, and a list item
//! goes on after a blank or a comment line only with the lines indented to its
//! content.
//!
//! Its key, `cap:<slug>#<section>/<key>` ([`ClaimCoverage::key`]), takes the
//! kebab slug of the bold lead or the heading, `opening` for the opening, and
//! for a Constraints unit `c-` and the first 16 hexadecimal digits of the
//! SHA-256 digest of its text with its comments left out and each run of
//! whitespace read as one space; a key that repeats in its section takes the
//! first of `-2`, `-3` and on that no other unit of the section has or takes, in
//! document order, so the keys of a section are distinct.
//!
//! A Contract clause is covered by a tested-by marker on any of its lines, a
//! Constraints unit by a marker whose statement starts in it. Under strict:
//!
//! * `strict-claim-uncovered` at the first line of each claim that no marker
//!   covers, its message naming the key and the first 60 characters of its
//!   lead or item;
//! * `strict-claim-unit-collapsed` at the Contract when it has no clause start
//!   and holds more than [`MIN_SECTION_WORDS`] words, as the section rule counts
//!   them, whether or not a marker covers it.
//!
//! [`Workplan::claims_report`] lists every claim of every delivered capability
//! under either strictness. That a marker names only paths the capability
//! binds in `tests` or `examples` is a rule of the load under either
//! strictness, `claim-test-unbound`.
//!
//! # Explained items
//!
//! The code and the tests directions of two-way coverage
//! ([`crate::coverage`]) are enforced for every package that holds an item a
//! `code`, `tests` or `examples` binding of a loaded capability resolves to and
//! that the coverage record's `scope` does not name, with the kinds the
//! default rule gives a gap: `coverage-item-unbound`,
//! `coverage-reexport-unbound` and `coverage-test-unbound` at the file and line
//! of the gap, and `coverage-surface-unreadable` at the key `bindings` of the
//! capability whose binding reaches the package. The message names that
//! capability and the path it binds, the first binding of the graph that
//! reaches the package, as the reason strict mode enforces it. The exclusions
//! of the coverage record apply as they do to `scope`, and a repository whose
//! coverage record is missing or does not load has no gap of this rule, as it
//! has none of `scope`: the record's own finding fails the check. A package that no
//! binding reaches and `scope` does not name is not enforced, and while a
//! capability document does not load no gap is reported, as for `scope`. This
//! coverage is computed here: the default coverage check is the same under
//! either strictness.

mod claims;
mod content;
#[cfg(test)]
pub(crate) mod tests;

pub use claims::{ClaimCoverage, ClaimsReport};

use std::collections::{BTreeMap, BTreeSet};

use raf_okf::{DocType, Location};

use crate::coverage::{self, Computed};
use crate::finding::kind;
use crate::graph::{Binding, ScopeEdge};
use crate::work::{Work, add};
use crate::{Finding, Loaded, Workplan};

/// The fewest words of prose a section of a delivered capability holds under
/// strict mode. The number is part of the contract and is not configurable: a
/// repository that asks for less does not opt into strict mode.
pub const MIN_SECTION_WORDS: usize = 20;

/// The findings of the rules of strict mode, in order: the content rules of
/// each delivered capability in the order of the capabilities, then the claim
/// rules of each in the same order, then the coverage gaps of the packages
/// that strict mode alone enforces, then the classes of the packages whose
/// orphan lines are over their ceilings.
pub(crate) fn check(plan: &Workplan<'_>) -> Vec<Finding> {
    let mut out = content::check(plan);
    out.extend(claims::check(plan));
    out.extend(explained(plan));
    out.extend(coverage::lines::over_ceilings(plan));
    out
}

/// The gaps of two-way coverage of the packages a binding reaches and `scope`
/// does not name. Without a coverage record that loads there are none: the
/// record's absence, or its own findings, fail the check first.
fn explained(plan: &Workplan<'_>) -> Vec<Finding> {
    let Some(record) = plan.coverage() else {
        return Vec::new();
    };
    let reasons = reaching_bindings(plan);
    if reasons.is_empty() {
        return Vec::new();
    }
    add(Work::StrictUnits, reasons.len());
    let packages: BTreeSet<&str> = reasons.keys().copied().collect();
    let Computed { report, .. } =
        coverage::compute(plan.source(), plan.graph(), Some(record.value()), &packages);
    let mut out = Vec::new();
    for error in report.errors() {
        let Some(binding) = reasons.get(error.package()) else {
            continue;
        };
        out.push(Finding::new(
            kind::COVERAGE_SURFACE_UNREADABLE,
            capability_path(plan, binding.capability()),
            Location::Key("bindings".to_string()),
            format!(
                "`{}` binds `{}` of `{}`, which strict mode therefore enforces, and {error}",
                binding.capability(),
                binding.path(),
                error.package()
            ),
            "make every public name of the crate resolve to one item (the error names the file and line), or raise the item-bytes budget of the load; under strict mode a package that a binding reaches is not judged covered while its surface cannot be read",
        ));
    }
    // A capability document that did not load binds nothing in the graph, so
    // every item and test it binds would read as a gap, as for `scope`.
    let capability_unloaded = plan
        .unloaded()
        .iter()
        .any(|id| id.starts_with(DocType::Capability.id_prefix()));
    if capability_unloaded {
        return out;
    }
    for gap in report.code().iter().chain(report.tests()) {
        let Some(binding) = reasons.get(gap.package()) else {
            continue;
        };
        let enforced = format!(
            "strict mode enforces because `{}` binds `{}`",
            binding.capability(),
            binding.path()
        );
        out.push(coverage::gap_finding(gap, &enforced));
    }
    out
}

/// Each package outside `scope` that a binding of a loaded capability
/// reaches, with the first binding of the graph that reaches it.
fn reaching_bindings<'p>(plan: &'p Workplan<'_>) -> BTreeMap<&'p str, &'p Binding> {
    let source = plan.source();
    let scope: BTreeSet<&str> = plan
        .graph()
        .scope()
        .iter()
        .map(ScopeEdge::package)
        .collect();
    let mut reasons = BTreeMap::new();
    for binding in plan.graph().bindings() {
        for id in binding.items() {
            let Some(target) = source.item(*id).and_then(|item| source.target(item.target)) else {
                continue;
            };
            let package = target.package.as_str();
            if !scope.contains(package) {
                reasons.entry(package).or_insert(binding);
            }
        }
    }
    reasons
}

/// A line that opens a list item, as the content and the claim rules read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ListItem<'t> {
    /// The column its content starts at, its indentation of spaces included.
    column: usize,
    /// The text after the marker and its space, empty for an item with none.
    content: &'t str,
}

/// The list item `raw` opens: after an indentation of spaces, `-`, `*` or
/// `+`, or one to nine digits and `.` or `)`, then a space or the end of the
/// line. A thematic break such as `* * *` opens one too; [`is_thematic_break`]
/// tells it apart.
fn list_item(raw: &str) -> Option<ListItem<'_>> {
    let indent = raw.len() - raw.trim_start_matches(' ').len();
    let text = &raw[indent..];
    let width = if text.starts_with(['-', '*', '+']) {
        1
    } else {
        let digits = text.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 || digits > 9 || !text[digits..].starts_with(['.', ')']) {
            return None;
        }
        digits + 1
    };
    let after = &text[width..];
    let content = if after.is_empty() {
        after
    } else {
        after.strip_prefix(' ')?
    };
    Some(ListItem {
        column: indent + width + 1,
        content,
    })
}

/// Whether a line is a thematic break: three or more of one of `-`, `*` or `_`
/// and nothing else but spaces and tabs.
fn is_thematic_break(raw: &str) -> bool {
    let marks: Vec<char> = raw.chars().filter(|c| !matches!(c, ' ' | '\t')).collect();
    marks.len() >= 3
        && ['-', '*', '_']
            .iter()
            .any(|mark| marks.iter().all(|c| c == mark))
}

/// Whether `raw` opens a list item that states something: an item with text
/// or a code span after its marker, and not a thematic break.
fn states_an_item(raw: &str) -> bool {
    list_item(raw).is_some_and(|item| !item.content.trim().is_empty()) && !is_thematic_break(raw)
}

/// The path of the loaded capability document `id`; the id itself when no
/// loaded document has it, which a binding of the graph never lacks.
fn capability_path<'p>(plan: &'p Workplan<'_>, id: &'p str) -> &'p str {
    plan.capabilities()
        .iter()
        .find(|capability| capability.value().header.id == id)
        .map_or(id, Loaded::path)
}
