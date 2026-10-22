// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The work of validating a document grows with its size, not with the
//! product of two of its lists.
//!
//! The work is counted, not timed. Where a count is a function of the input
//! (one `affects` entry judged per entry, one limit looked up per limit, the
//! `changes` set built once) the test pins it exactly, on documents at the
//! size cap. Where it is not (the generated blocks looked at for the sections
//! of a body, the lines tested for a fence) the test builds the document at
//! one, two and three units, with both lists growing together, and asserts
//! that the count is nonzero and that the third unit adds no more than the
//! second did, which needs no constant to be tuned: a step that does work per
//! pair makes each size add more than the one before.
//!
//! A count sees the steps that are counted. A loop added somewhere else is not
//! seen by it.

use std::fmt::Write as _;

use crate::body::Body;
use crate::work::{Work, measure, measure_kind};
use crate::{Document, MAX_TEXT_BYTES, Profile, RafDocument, Rule};

/// The size of the smallest document of a growth test, in sections or lines.
const UNIT: usize = 1_000;

/// The most the kind of work a growth test asserts on may count while one
/// document is judged. A document that is judged with work per pair passes it
/// at once instead of running to the end.
const CEILING: usize = 64 * UNIT;

fn parse(text: &str) -> Document {
    Document::parse(text).expect("the probe is a document")
}

/// How many entries of `size` bytes each fit in `room` bytes.
fn entries_that_fit(room: usize, size: impl Fn(usize) -> usize) -> usize {
    let mut used = 0;
    let mut count = 0;
    loop {
        let next = size(count);
        if used + next > room {
            return count;
        }
        used += next;
        count += 1;
    }
}

/// A change record with `count` capabilities in `changes` and one `affects`
/// entry for each.
fn change_with(count: usize) -> String {
    let mut text = String::from(
        "---\ntype: Change\nid: change:probe\ntitle: Probe\ndescription: Probe.\nchanges:\n",
    );
    (0..count).for_each(|n| writeln!(text, "  - cap:n{n}").expect("write"));
    text.push_str("affects:\n");
    (0..count).for_each(|n| writeln!(text, "  - cap:n{n}#contract").expect("write"));
    text.push_str("contract_effect: none\nitems: []\n---\n\n## Summary\n\nProbe.\n");
    text
}

/// A change record at the size cap, and the number of capabilities in it.
fn change_at_the_cap() -> (String, usize) {
    let fixed = change_with(0).len();
    let count = entries_that_fit(MAX_TEXT_BYTES - fixed, |n| {
        "  - cap:n\n".len() + "  - cap:n#contract\n".len() + 2 * n.to_string().len()
    });
    (change_with(count), count)
}

const CAPABILITY_HEAD: &str = "---\ntype: Capability\nid: cap:probe\ntitle: Probe\n\
    description: Probe.\nstate: designed\nbindings:\n  code: []\n  tests: []\n  \
    examples: []\ndepends: []\n---\n\n## Purpose\n\n";
const BLOCK: &str = "<!-- raf:generated:list -->\n<!-- raf:generated:end -->\n";

/// A capability at the size cap: generated blocks first, then as many sections
/// as fit. Returns the text, the number of blocks and the number of sections.
fn capability_at_the_cap() -> (String, usize, usize) {
    let blocks = 10_000;
    let mut text = String::from(CAPABILITY_HEAD);
    (0..blocks).for_each(|_| text.push_str(BLOCK));
    let section = "## Purpose\n";
    let mut sections = 0;
    while text.len() + section.len() <= MAX_TEXT_BYTES {
        text.push_str(section);
        sections += 1;
    }
    (text, blocks, sections)
}

/// A capability of `sections` sections that each hold `per_section` generated
/// blocks.
fn capability_with(sections: usize, per_section: usize) -> String {
    let mut text = String::from(CAPABILITY_HEAD);
    for _ in 0..sections {
        for _ in 0..per_section {
            text.push_str(BLOCK);
        }
        text.push_str("\n## Purpose\n\n");
    }
    text
}

/// An evidence record with `count` values, each with its limit, the limits in
/// the reverse order of the values.
fn evidence_with(count: usize) -> String {
    let mut text = String::from(
        "---\ntype: Evidence\nid: evidence:probe\nsubject: command\ninputs:\n  \
         - kind: symbol\n    name: probe::item\n    digest: \
         9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08\n\
         verdict: pass\nby: agent\nfor: [cap:probe]\ncommand: \"cargo bench\"\nvalues:\n",
    );
    (0..count).for_each(|n| writeln!(text, "  m{n}: 1").expect("write"));
    text.push_str("limits:\n");
    (0..count)
        .rev()
        .for_each(|n| writeln!(text, "  m{n}: 2").expect("write"));
    text.push_str("---\n");
    text
}

/// An evidence record at the size cap, and the number of measures in it.
fn evidence_at_the_cap() -> (String, usize) {
    let fixed = evidence_with(0).len();
    let count = entries_that_fit(MAX_TEXT_BYTES - fixed, |n| {
        "  m: 1\n".len() + "  m: 2\n".len() + 2 * n.to_string().len()
    });
    (evidence_with(count), count)
}

/// The counts of `kind` for the documents `build` makes at one, two and three
/// units, and the asserted growth: the count is nonzero and the third unit
/// adds no more than the second.
fn assert_linear(kind: Work, build: impl Fn(usize) -> String, run: impl Fn(&str)) {
    let counts = [1, 2, 3].map(|units| {
        let text = build(units * UNIT);
        let ((), tally) = measure_kind(kind, CEILING, || run(&text));
        tally.get(kind)
    });
    let [one, two, three] = counts;
    assert!(one > 0, "{kind:?} counted nothing: {counts:?}");
    assert!(
        // The same as three - two <= two - one, without a subtraction below zero.
        three + one <= 2 * two,
        "{kind:?} grows faster than its input: {counts:?}"
    );
}

#[test]
fn a_change_at_the_cap_judges_each_affects_entry_once_against_one_set() {
    let (text, count) = change_at_the_cap();
    assert!(text.len() <= MAX_TEXT_BYTES && text.len() > MAX_TEXT_BYTES - 64);
    assert!(count > 20_000, "the probe holds only {count} entries");
    let document = parse(&text);
    // Nothing counted may pass the number of entries: a set built for each
    // entry passes it on the third.
    let (result, tally) = measure(count, || {
        Profile.validate(&document, "work/changes/probe.md")
    });
    let Ok(RafDocument::Change(change)) = result else {
        panic!("the probe is a valid change: {result:?}");
    };
    assert_eq!(change.changes.len(), count);
    assert_eq!(change.affects.len(), count);
    assert_eq!(tally.get(Work::ChangeIdsSet), count);
    assert_eq!(tally.get(Work::AffectsEntries), count);
    assert_eq!(tally.get(Work::ListedLookups), count);
}

#[test]
fn a_capability_at_the_cap_looks_at_each_block_once() {
    let (text, blocks, sections) = capability_at_the_cap();
    assert!(text.len() <= MAX_TEXT_BYTES && text.len() > MAX_TEXT_BYTES - 16);
    assert!(
        sections > 30_000,
        "the probe holds only {sections} sections"
    );
    let document = parse(&text);
    // The blocks all sit in the first section, so each is looked at there and
    // by no later section.
    let (result, tally) = measure_kind(Work::BlocksVisited, blocks, || {
        Profile.validate(&document, "docs/capabilities/probe.md")
    });
    let violations = result.expect_err("repeated and missing sections are violations");
    assert!(
        violations
            .iter()
            .any(|violation| violation.rule() == &Rule::SectionDuplicated),
        "{} violations, none for the duplicated section",
        violations.len()
    );
    assert_eq!(tally.get(Work::BlocksVisited), blocks);
}

#[test]
fn the_blocks_looked_at_grow_with_sections_and_blocks_together() {
    assert_linear(
        Work::BlocksVisited,
        |sections| capability_with(sections, 2),
        |text| {
            let document = parse(text);
            let violations = Profile
                .validate(&document, "docs/capabilities/probe.md")
                .expect_err("repeated sections are a violation");
            assert!(
                violations
                    .iter()
                    .any(|violation| violation.rule() == &Rule::SectionDuplicated)
            );
        },
    );
}

#[test]
fn an_evidence_record_at_the_cap_indexes_each_value_once_and_looks_up_each_limit_once() {
    let (text, count) = evidence_at_the_cap();
    assert!(text.len() <= MAX_TEXT_BYTES && text.len() > MAX_TEXT_BYTES - 64);
    assert!(count > 40_000, "the probe holds only {count} measures");
    let document = parse(&text);
    // A table built for each limit passes the number of measures on the
    // second limit.
    let (result, tally) = measure(count, || {
        Profile.validate(&document, "work/evidence/probe.md")
    });
    let Ok(RafDocument::Evidence(evidence)) = result else {
        panic!("the probe is a valid evidence record: {result:?}");
    };
    assert_eq!(evidence.values.len(), count);
    assert_eq!(evidence.limits.len(), count);
    assert_eq!(tally.get(Work::MeasuresIndexed), count);
    assert_eq!(tally.get(Work::LimitLookups), count);
}

/// A page of `lines` lines in which a fence opens on the first and never
/// closes.
fn unterminated_fence(lines: usize) -> String {
    let mut text = String::from("```\n");
    (1..lines).for_each(|_| text.push_str("code\n"));
    text
}

/// A page of `lines` lines of prose.
fn prose(lines: usize) -> String {
    "line\n".repeat(lines)
}

/// A page of fenced blocks of two lines each, `lines` lines in all.
fn closed_fences(lines: usize) -> String {
    "```\n```\n".repeat(lines / 2)
}

fn parse_body(text: &str) {
    Body::parse(text).expect("the probe is a body");
}

#[test]
fn the_lines_tested_for_a_fence_grow_with_the_length_of_the_page() {
    assert_linear(Work::FenceScans, unterminated_fence, parse_body);
    assert_linear(Work::FenceScans, prose, parse_body);
    assert_linear(Work::FenceScans, closed_fences, parse_body);
}

#[test]
fn an_unterminated_fence_is_tested_once_for_each_line_it_swallows() {
    let lines = 5 * UNIT;
    let text = unterminated_fence(lines);
    let ((), tally) = measure_kind(Work::FenceScans, lines + 1, || parse_body(&text));
    // The opening line is tested for opening, every line after it for closing.
    assert_eq!(tally.get(Work::FenceScans), lines);
}
