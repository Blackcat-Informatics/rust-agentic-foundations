// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The content rules of strict mode for a delivered capability: an example,
//! the verification matrix, the words of each section, a Usage that names a
//! bound example and a Constraints section that lists a limitation. See the
//! documentation of [`crate::strict`].

use raf_okf::{Body, Capability, CapabilitySection, Location, Section, State};

use super::MIN_SECTION_WORDS;
use crate::finding::kind;
use crate::graph::BindingKind;
use crate::obligation::tense::word_spans;
use crate::work::{Work, add};
use crate::{Finding, Loaded, Workplan};

/// The kind of the generated block the Verification section holds.
const MATRIX: &str = "verification-matrix";

/// The findings of the content rules, capability by capability in the order
/// of the load.
pub(super) fn check(plan: &Workplan<'_>) -> Vec<Finding> {
    let mut out = Vec::new();
    for loaded in plan.capabilities() {
        if loaded.value().state == State::Delivered {
            add(Work::StrictUnits, 1);
            judge(plan, loaded, &mut out);
        }
    }
    out
}

/// The content rules of one delivered capability.
fn judge(plan: &Workplan<'_>, loaded: &Loaded<Capability>, out: &mut Vec<Finding>) {
    let path = loaded.path();
    let capability = loaded.value();
    let body = loaded.body();
    if capability.bindings.examples.is_empty() {
        out.push(Finding::new(
            kind::STRICT_EXAMPLE_MISSING,
            path,
            Location::Key("bindings".to_string()),
            "the delivered capability binds no entry in bindings.examples",
            "bind at least one example target or runnable documentation test in `bindings.examples`; strict mode asks every delivered capability for a usage that runs",
        ));
    }
    if !holds_matrix(body) {
        out.push(Finding::new(
            kind::STRICT_MATRIX_MISSING,
            path,
            section(CapabilitySection::Verification),
            "the Verification section holds no verification-matrix generated block",
            "add `<!-- raf:generated:verification-matrix -->` and `<!-- raf:generated:end -->` to the Verification section and run `cargo workplan docs sync`",
        ));
    }
    for (heading, words) in section_words(body) {
        if words < MIN_SECTION_WORDS {
            out.push(Finding::new(
                kind::STRICT_SECTION_THIN,
                path,
                Location::Section(heading.to_string()),
                format!(
                    "the section holds {words} words of prose, fewer than {MIN_SECTION_WORDS}"
                ),
                format!(
                    "write at least {MIN_SECTION_WORDS} words of prose saying what the section covers; a section of a delivered capability is read on its own, and a generated block, a code span and fenced code do not count as prose"
                ),
            ));
        }
    }
    let usage_bound = plan
        .graph()
        .bindings_of(&capability.header.id)
        .filter(|binding| binding.kind() == BindingKind::Example)
        .any(|binding| binding.sections().contains(&CapabilitySection::Usage));
    if !usage_bound {
        out.push(Finding::new(
            kind::STRICT_USAGE_UNBOUND,
            path,
            section(CapabilitySection::Usage),
            "no path of bindings.examples is named in the section",
            "name a path of bindings.examples in a code span in the Usage section, so the verification matrix lists Usage for an example that verify runs",
        ));
    }
    if !lists_an_item(body, CapabilitySection::Constraints) {
        out.push(Finding::new(
            kind::STRICT_LIMITATIONS_MISSING,
            path,
            section(CapabilitySection::Constraints),
            "the Constraints section holds no list item",
            "state each limitation of the capability as a list item of Constraints",
        ));
    }
}

/// The location of one of the seven sections.
fn section(section: CapabilitySection) -> Location {
    Location::Section(section.heading().to_string())
}

/// The second-level section of `body` that is `which`.
fn find(body: &Body, which: CapabilitySection) -> Option<&Section> {
    body.sections()
        .iter()
        .find(|section| section.slug() == which.slug())
}

/// Whether the Verification section holds a `verification-matrix` block.
fn holds_matrix(body: &Body) -> bool {
    let Some(verification) = find(body, CapabilitySection::Verification) else {
        return false;
    };
    let lines = &verification.content().lines;
    body.generated_blocks()
        .iter()
        .any(|block| block.kind() == MATRIX && lines.contains(&block.outer().lines.start))
}

/// The prose lines of one section: below its heading, outside PROPOSED and
/// generated blocks.
fn prose<'b>(body: &'b Body, section: &'b Section) -> impl Iterator<Item = &'b str> {
    body.prose_lines()
        .iter()
        .filter(move |line| {
            line.section() == Some(section.slug())
                && line.line() != section.line()
                && !line.in_proposed()
                && !line.in_generated()
        })
        .map(raf_okf::ProseLine::text)
}

/// The heading and the number of words of prose of each of the seven
/// sections, in document order.
fn section_words(body: &Body) -> Vec<(&str, usize)> {
    body.sections()
        .iter()
        .filter(|section| CapabilitySection::from_slug(section.slug()).is_some())
        .map(|section| {
            let words = prose(body, section)
                .map(|text| word_spans(text).len())
                .sum();
            (section.heading(), words)
        })
        .collect()
}

/// Whether the section `which` holds a list item: a prose line that starts,
/// after its indentation, with `-`, `*` or `+` and a space, or with digits,
/// `.` or `)` and a space.
fn lists_an_item(body: &Body, which: CapabilitySection) -> bool {
    let Some(section) = find(body, which) else {
        return false;
    };
    prose(body, section).any(|text| is_list_item(text.trim_start()))
}

/// Whether a line, without its indentation, opens a list item.
fn is_list_item(text: &str) -> bool {
    if let Some(rest) = text.strip_prefix(['-', '*', '+']) {
        return rest.starts_with(' ');
    }
    let digits = text.bytes().take_while(u8::is_ascii_digit).count();
    digits > 0 && {
        let rest = &text[digits..];
        rest.strip_prefix(['.', ')'])
            .is_some_and(|after| after.starts_with(' '))
    }
}
