// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The cost of `Profile::validate` on documents at the size cap: it grows with
//! the size of the document, not with the product of two of its lists.

use std::time::{Duration, Instant};

use raf_okf::{Document, MAX_TEXT_BYTES, Profile, RafDocument, Rule};

/// What a validation of a document at the cap may take. A linear pass takes a
/// few milliseconds; the product of two lists of this size took half a second.
/// The bound is checked in a release build only: a debug build runs an order
/// of magnitude slower and a wall-clock bound there would be noise. `make
/// test-release` runs this file in release.
const RELEASE_BOUND: Duration = Duration::from_millis(100);

/// A change record at the size cap: as many `changes` entries as fit, and as
/// many `affects` entries, one per change.
fn change_at_the_cap() -> (String, usize) {
    let head = "---\ntype: Change\nid: change:probe\ntitle: Probe\ndescription: Probe.\n";
    let tail = "contract_effect: none\nitems: []\n---\n\n## Summary\n\nProbe.\n";
    // One `changes` line and one `affects` line per capability.
    let entry = |n: usize| {
        (
            format!("  - cap:n{n}\n"),
            format!("  - cap:n{n}#contract\n"),
        )
    };
    let mut used = head.len() + tail.len() + "changes:\naffects:\n".len();
    let mut count = 0;
    loop {
        let (change, affect) = entry(count);
        if used + change.len() + affect.len() > MAX_TEXT_BYTES {
            break;
        }
        used += change.len() + affect.len();
        count += 1;
    }
    let mut text = String::with_capacity(MAX_TEXT_BYTES);
    text.push_str(head);
    text.push_str("changes:\n");
    (0..count).for_each(|n| text.push_str(&entry(n).0));
    text.push_str("affects:\n");
    (0..count).for_each(|n| text.push_str(&entry(n).1));
    text.push_str(tail);
    (text, count)
}

/// A capability at the size cap: generated blocks first, then as many
/// sections as fit.
fn capability_at_the_cap() -> (String, usize) {
    let head = "---\ntype: Capability\nid: cap:probe\ntitle: Probe\ndescription: Probe.\n\
                state: designed\nbindings:\n  code: []\n  tests: []\n  examples: []\n\
                depends: []\n---\n\n## Purpose\n\n";
    let block = "<!-- raf:generated:list -->\n<!-- raf:generated:end -->\n";
    let section = "## Purpose\n";
    let blocks = 10_000;
    let mut text = String::from(head);
    for _ in 0..blocks {
        text.push_str(block);
    }
    let mut sections = 0;
    while text.len() + section.len() <= MAX_TEXT_BYTES {
        text.push_str(section);
        sections += 1;
    }
    (text, sections)
}

fn assert_within_bound(elapsed: Duration, what: &str) {
    if !cfg!(debug_assertions) {
        assert!(
            elapsed < RELEASE_BOUND,
            "validating {what} took {elapsed:?}, the bound is {RELEASE_BOUND:?}"
        );
    }
}

fn timed(text: &str, path: &str) -> (Duration, Result<RafDocument, Vec<raf_okf::Violation>>) {
    let document = Document::parse(text).expect("the probe is a document");
    let started = Instant::now();
    let result = Profile.validate(&document, path);
    (started.elapsed(), result)
}

#[test]
fn a_change_at_the_cap_with_maximal_affects_and_changes_validates_within_the_bound() {
    let (text, count) = change_at_the_cap();
    assert!(text.len() <= MAX_TEXT_BYTES && text.len() > MAX_TEXT_BYTES - 64);
    assert!(count > 20_000, "the probe holds only {count} entries");
    let (elapsed, result) = timed(&text, "work/changes/probe.md");
    let Ok(RafDocument::Change(change)) = result else {
        panic!("the probe is a valid change: {result:?}");
    };
    assert_eq!(change.changes.len(), count);
    assert_eq!(change.affects.len(), count);
    assert_within_bound(elapsed, &format!("{count} changes and affects"));
}

#[test]
fn a_capability_at_the_cap_with_many_sections_and_blocks_validates_within_the_bound() {
    let (text, sections) = capability_at_the_cap();
    assert!(text.len() <= MAX_TEXT_BYTES && text.len() > MAX_TEXT_BYTES - 16);
    assert!(
        sections > 30_000,
        "the probe holds only {sections} sections"
    );
    let (elapsed, result) = timed(&text, "docs/capabilities/probe.md");
    let violations = result.expect_err("repeated and missing sections are violations");
    assert!(
        violations
            .iter()
            .any(|violation| violation.rule() == &Rule::SectionDuplicated),
        "{} violations, none for the duplicated section",
        violations.len()
    );
    assert_within_bound(elapsed, &format!("{sections} sections and 10000 blocks"));
}

/// An evidence record at the size cap: as many `values` as fit, each with its
/// limit, so judging the limits against the values is the product of two
/// lists if it is done by search.
fn evidence_at_the_cap() -> (String, usize) {
    let head = "---\ntype: Evidence\nid: evidence:probe\nsubject: command\ninputs:\n  \
                - kind: symbol\n    name: probe::item\n    digest: \
                9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08\n\
                verdict: pass\nby: agent\nfor: [cap:probe]\ncommand: \"cargo bench\"\n";
    let tail = "---\n";
    let entry = |n: usize| (format!("  m{n}: 1\n"), format!("  m{n}: 2\n"));
    let mut used = head.len() + tail.len() + "values:\nlimits:\n".len();
    let mut count = 0;
    loop {
        let (value, limit) = entry(count);
        if used + value.len() + limit.len() > MAX_TEXT_BYTES {
            break;
        }
        used += value.len() + limit.len();
        count += 1;
    }
    let mut text = String::with_capacity(MAX_TEXT_BYTES);
    text.push_str(head);
    text.push_str("values:\n");
    (0..count).for_each(|n| text.push_str(&entry(n).0));
    text.push_str("limits:\n");
    // The limits in the reverse order of the values: a search from the front
    // would walk the whole list for each.
    (0..count).rev().for_each(|n| text.push_str(&entry(n).1));
    text.push_str(tail);
    (text, count)
}

#[test]
fn an_evidence_record_at_the_cap_with_maximal_values_and_limits_validates_within_the_bound() {
    let (text, count) = evidence_at_the_cap();
    assert!(text.len() <= MAX_TEXT_BYTES && text.len() > MAX_TEXT_BYTES - 64);
    assert!(count > 40_000, "the probe holds only {count} measures");
    let (elapsed, result) = timed(&text, "work/evidence/probe.md");
    let Ok(RafDocument::Evidence(evidence)) = result else {
        panic!("the probe is a valid evidence record: {result:?}");
    };
    assert_eq!(evidence.values.len(), count);
    assert_eq!(evidence.limits.len(), count);
    assert_within_bound(elapsed, &format!("{count} values and limits"));
}
