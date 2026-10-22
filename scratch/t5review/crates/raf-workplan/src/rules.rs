// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The rules between a capability, its PROPOSED blocks and the change records.
//!
//! The rules that read one document's frontmatter or text belong to `raf-okf`
//! and reach `check` through its findings: a PROPOSED block outside a
//! capability or outside a section, a delivered capability without code and
//! test bindings, a designed one with bindings, an empty section. The rules
//! here are the ones `raf-okf` does not have: a block names a change record
//! that lists the capability and affects the section, and a state fixes which
//! blocks and which open records may exist. Three of the state rules read the
//! capability alone (considered, designed and delivered against its blocks);
//! the open-record rules and the block rules read the records too.

use std::collections::BTreeSet;

use raf_okf::{Capability, CapabilitySection, Change, Location, State};

use crate::finding::kind;
use crate::{Finding, Loaded};

/// The findings of the proposal and state rules over the loaded documents.
///
/// `present` holds the ids of the documents that exist as files, loaded or
/// not. A block that names a change whose record is there but broken is
/// explained by the findings of that record, not reported as missing.
pub(crate) fn check(
    capabilities: &[Loaded<Capability>],
    changes: &[Loaded<Change>],
    present: &BTreeSet<String>,
    unvalidated_blocks: &[(String, String, usize)],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    // A record that is there and did not load may list any capability, so no
    // capability is reported as unlisted while one exists: its own findings
    // say what is wrong, as for a block that names it.
    let broken_record = present.iter().any(|id| {
        id.starts_with("change:") && !changes.iter().any(|change| change.value().header.id == *id)
    });
    for (file, slug, line) in unvalidated_blocks {
        let change_id = format!("change:{slug}");
        if !present.contains(&change_id) {
            findings.push(missing_change(file, slug, Location::Line(*line)));
        }
    }
    for cap in capabilities {
        let id = cap.value().header.id.as_str();
        let listing: Vec<&Loaded<Change>> = changes
            .iter()
            .filter(|change| change.value().changes.iter().any(|listed| listed == id))
            .collect();
        let blocks = cap.body().proposed_blocks();
        for block in blocks {
            let at = Location::Line(block.marker().lines.start);
            let change_id = format!("change:{}", block.slug());
            let Some(change) = changes
                .iter()
                .find(|change| change.value().header.id == change_id)
            else {
                if !present.contains(&change_id) {
                    findings.push(missing_change(cap.path(), block.slug(), at));
                }
                continue;
            };
            if !listing.iter().any(|listed| std::ptr::eq(*listed, change)) {
                findings.push(Finding::new(
                    kind::PROPOSED_CHANGE_NOT_LISTING,
                    cap.path(),
                    at,
                    format!(
                        "the PROPOSED change `{}` does not list {id} in its `changes`",
                        block.slug()
                    ),
                    "list the capability in the `changes` of the change record, or move the block to a change that lists it",
                ));
                continue;
            }
            let section = block.section().and_then(CapabilitySection::from_slug);
            let affected = section.is_some_and(|section| {
                change
                    .value()
                    .affects
                    .iter()
                    .any(|entry| entry.capability == id && entry.section == section)
            });
            if !affected {
                let named = block.section().unwrap_or("(none)");
                findings.push(Finding::new(
                    kind::PROPOSED_SECTION_NOT_AFFECTED,
                    cap.path(),
                    at,
                    format!(
                        "the PROPOSED block in section `{named}` needs `{id}#{named}` in the `affects` of change `{}`",
                        change.value().header.slug
                    ),
                    "add the entry to the `affects` of the change record, or move the block to a section the change affects",
                ));
            }
        }
        state_rules(cap, blocks.len(), &listing, broken_record, &mut findings);
    }
    findings
}

fn missing_change(file: &str, slug: &str, at: Location) -> Finding {
    Finding::new(
        kind::PROPOSED_MISSING_CHANGE,
        file,
        at,
        format!(
            "the PROPOSED block names the change `{slug}`, but work/changes/{slug}.md does not exist"
        ),
        "create the change record, or correct the slug in the marker",
    )
}

/// What the state of a capability fixes about its PROPOSED blocks and the
/// change records that list it.
fn state_rules(
    cap: &Loaded<Capability>,
    blocks: usize,
    listing: &[&Loaded<Change>],
    broken_record: bool,
    findings: &mut Vec<Finding>,
) {
    let state = cap.value().state;
    let at = || Location::Key("state".to_string());
    let mut push = |kind: &'static str, message: String, fix: &str| {
        findings.push(Finding::new(kind, cap.path(), at(), message, fix));
    };
    match state {
        State::Considered if blocks == 0 => push(
            kind::STATE_CONSIDERED_NO_PROPOSAL,
            "state considered requires its candidate contract in PROPOSED blocks".to_string(),
            "write the candidate contract in a PROPOSED block, or move the state to designed",
        ),
        State::Designed if blocks > 0 => push(
            kind::STATE_DESIGNED_PROPOSAL,
            "state designed must have no PROPOSED blocks".to_string(),
            "state the contract plainly and remove the blocks, or move the state to considered or implementing",
        ),
        State::Delivered => {
            if blocks > 0 {
                push(
                    kind::STATE_DELIVERED_PROPOSAL,
                    "state delivered must have no PROPOSED blocks".to_string(),
                    "reconcile each block into the contract and remove it, or move the state to implementing",
                );
            }
            for change in listing {
                push(
                    kind::STATE_DELIVERED_OPEN_RECORD,
                    format!(
                        "state delivered must not be listed by the open change record {}",
                        change.path()
                    ),
                    "remove the capability from the record, or move the state to implementing",
                );
            }
        }
        State::Considered | State::Designed | State::Implementing => {}
    }
    if state != State::Delivered && listing.is_empty() && !broken_record {
        push(
            kind::NO_OPEN_RECORD,
            format!(
                "state {} requires an open change record that lists {}",
                state.name(),
                cap.value().header.id
            ),
            "list the capability in the `changes` of an open change record, or move the state to delivered",
        );
    }
}
