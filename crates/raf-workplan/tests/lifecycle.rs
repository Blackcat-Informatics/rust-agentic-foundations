// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `change new` and `change impact`: the record of a new change, the template
//! and the state guidance it prints, and what a change reaches.

mod support;

use std::path::Path;

use raf_okf::{CapabilitySection as S, Document, State};
use raf_workplan::lifecycle::{
    AffectsProblem, NewChange, NewChangeError, NewChangeRequest, StateAdvice,
};
use raf_workplan::{ChangeLookupError, Finding, SectionId, SyncOptions, Workplan};
use support::disk::{Scratch, changed, on_disk, put, read, snapshot, write_repo};
use support::synced::{self, ALPHA, BETA, DELTA, GAMMA, GROW};

/// The repository of the generated-block fixtures, written to a disk.
fn repo(tag: &str) -> Scratch {
    let scratch = Scratch::new(tag);
    write_repo(scratch.path(), &synced::base());
    scratch
}

fn create(root: &Path, request: &NewChangeRequest) -> Result<NewChange, NewChangeError> {
    on_disk(root, |plan| plan.new_change(root, request))
}

fn created(root: &Path, request: &NewChangeRequest) -> NewChange {
    create(root, request).unwrap_or_else(|e| panic!("`change new` should succeed: {e}"))
}

/// The findings of `check` that are about `file`.
fn findings_about(root: &Path, file: &str) -> Vec<Finding> {
    on_disk(root, |plan| {
        plan.check()
            .into_iter()
            .filter(|finding| finding.file == file)
            .collect()
    })
}

fn kinds(findings: &[Finding]) -> Vec<&'static str> {
    findings.iter().map(|finding| finding.kind).collect()
}

// --- the record --------------------------------------------------------------

const FRESH: &str = "work/changes/fresh-change.md";

/// The record `change new fresh-change --capability cap:beta` writes.
const FRESH_BETA: &str = "\
---
type: Change
id: change:fresh-change
title: Name this change
description: Say in one line what this change does to the capabilities it lists.
changes: [cap:beta]
affects:
  - cap:beta#contract
contract_effect: changes
items: []
---

## Summary

Describe what this change does and why, in a short paragraph.
";

#[test]
fn new_writes_one_file_with_the_keys_the_id_and_a_summary_and_it_loads_clean() {
    let scratch = repo("new-keys");
    let root = scratch.path();
    let before = snapshot(root);
    let request = NewChangeRequest::new("fresh-change").capability("cap:beta");
    let result = created(root, &request);

    // Exactly one file, and no other byte moved.
    assert_eq!(changed(&before, &snapshot(root)), [FRESH]);
    assert_eq!(result.path.as_str(), FRESH);
    assert_eq!(read(root, FRESH), FRESH_BETA);

    // The keys, the id and the Summary, read by the document reader.
    let document = Document::parse(&read(root, FRESH)).expect("reads");
    let keys: Vec<&str> = document
        .frontmatter()
        .map()
        .entries()
        .iter()
        .map(raf_okf::Entry::key)
        .collect();
    assert_eq!(
        keys,
        [
            "type",
            "id",
            "title",
            "description",
            "changes",
            "affects",
            "contract_effect",
            "items"
        ]
    );
    let summary = document
        .body()
        .sections()
        .iter()
        .find(|section| section.heading() == "Summary")
        .expect("a Summary section");
    let body = document.body();
    let start = summary.content().bytes.start - body.span().bytes.start;
    let end = summary.content().bytes.end - body.span().bytes.start;
    assert_eq!(
        body.text()[start..end].trim(),
        "Describe what this change does and why, in a short paragraph."
    );

    // The loaded record has the values and no finding.
    on_disk(root, |plan| {
        let record = plan.change("fresh-change").expect("loads").value();
        assert_eq!(record.header.id, "change:fresh-change");
        assert_eq!(record.header.slug, "fresh-change");
        assert_eq!(record.changes, ["cap:beta"]);
        assert_eq!(record.affects.len(), 1);
        assert_eq!(record.affects[0].capability, "cap:beta");
        assert_eq!(record.affects[0].section, S::Contract);
        assert_eq!(record.contract_effect, raf_okf::ContractEffect::Changes);
        assert_eq!(record.items.len(), 0);
        assert!(
            plan.check().iter().all(|finding| finding.file != FRESH),
            "{:#?}",
            plan.check()
        );
    });
}

#[test]
fn a_bad_slug_is_refused_before_anything_is_written_and_a_good_one_next_to_it_is_not() {
    let scratch = repo("new-slug");
    let root = scratch.path();
    let before = snapshot(root);
    for slug in [
        "Fresh",
        "fresh-Change",
        "-fresh",
        "--since=x",
        "-rf",
        "",
        "a/b",
        "../x",
        "..",
        ".",
        "a\\b",
        "a b",
        "fresh-",
        "fresh--change",
        "fresh_change",
        "fresh.md",
        "fresh\n",
    ] {
        let request = NewChangeRequest::new(slug).capability("cap:beta");
        match create(root, &request) {
            Err(NewChangeError::BadSlug { slug: refused }) => assert_eq!(refused, slug),
            other => panic!("`{slug:?}` should be a bad slug, got {other:?}"),
        }
        assert_eq!(snapshot(root), before, "{slug:?}: nothing is written");
    }
    // The neighbours: a digit first and a digit-only part are slugs.
    for slug in ["fresh", "2fa-login", "a-1-b"] {
        created(root, &NewChangeRequest::new(slug).capability("cap:beta"));
        assert!(root.join(format!("work/changes/{slug}.md")).is_file());
    }
}

#[test]
fn the_bad_slug_message_says_what_a_slug_is() {
    let scratch = repo("new-slug-text");
    let error = create(
        scratch.path(),
        &NewChangeRequest::new("--since=x").capability("cap:beta"),
    )
    .expect_err("refused");
    let text = error.to_string();
    assert!(text.contains("`--since=x` is not a change slug"), "{text}");
    assert!(text.contains("lowercase"), "{text}");
}

#[test]
fn an_existing_file_is_refused_and_left_byte_identical() {
    let scratch = repo("new-exists");
    let root = scratch.path();
    created(
        root,
        &NewChangeRequest::new("fresh-change").capability("cap:beta"),
    );
    // The record just written, a record of the repository, and files that are
    // not records at all.
    put(root, "work/changes/stray.md", "not a document\n");
    put(root, "work/changes/broken.md", "---\ntype: Change\n");
    let before = snapshot(root);
    for slug in ["fresh-change", "grow", "stray", "broken"] {
        let request = NewChangeRequest::new(slug).capability("cap:alpha");
        match create(root, &request) {
            Err(NewChangeError::Exists { path }) => {
                assert_eq!(path.as_str(), format!("work/changes/{slug}.md"));
            }
            other => panic!("`{slug}` should exist, got {other:?}"),
        }
        assert_eq!(snapshot(root), before, "{slug}: byte-identical afterwards");
    }
    // The neighbour: a slug with no file.
    created(
        root,
        &NewChangeRequest::new("other").capability("cap:alpha"),
    );
}

#[cfg(unix)]
#[test]
fn a_link_at_the_path_is_an_existing_file_and_is_not_followed() {
    use std::os::unix::fs::symlink;

    let scratch = repo("new-link");
    let root = scratch.path();
    let outside = Scratch::new("new-link-outside");
    // A dangling link, and a link to a file outside the repository.
    symlink(
        outside.path().join("nothing"),
        root.join("work/changes/dangling.md"),
    )
    .expect("links");
    put(outside.path(), "target.md", "outside\n");
    symlink(
        outside.path().join("target.md"),
        root.join("work/changes/aimed.md"),
    )
    .expect("links");
    let before = snapshot(root);
    for slug in ["dangling", "aimed"] {
        let request = NewChangeRequest::new(slug).capability("cap:beta");
        assert!(
            matches!(create(root, &request), Err(NewChangeError::Exists { .. })),
            "{slug}"
        );
        assert_eq!(snapshot(root), before, "{slug}");
    }
    assert_eq!(read(outside.path(), "target.md"), "outside\n");
    assert!(!outside.path().join("nothing").exists());
}

#[test]
fn a_capability_that_does_not_exist_is_refused() {
    let scratch = repo("new-missing");
    let root = scratch.path();
    let before = snapshot(root);
    for (capability, in_message) in [
        ("cap:ghost", "`cap:ghost` is not a capability"),
        ("beta", "capability ids read `cap:<name>`"),
        ("", "capability ids read `cap:<name>`"),
        ("cap:", "`cap:` is not a capability"),
        ("cap:../beta", "is not a capability"),
        ("change:grow", "capability ids read `cap:<name>`"),
    ] {
        let request = NewChangeRequest::new("fresh-change")
            .capability("cap:beta")
            .capability(capability);
        match create(root, &request) {
            Err(error @ NewChangeError::UnknownCapability { .. }) => {
                let NewChangeError::UnknownCapability {
                    capability: named,
                    known,
                } = &error
                else {
                    unreachable!()
                };
                assert_eq!(named, capability);
                assert_eq!(
                    known,
                    &["cap:alpha", "cap:beta", "cap:delta", "cap:gamma"],
                    "{capability}"
                );
                assert!(
                    error.to_string().contains(in_message),
                    "{capability}: {error}"
                );
            }
            other => panic!("`{capability}` should be unknown, got {other:?}"),
        }
        assert_eq!(snapshot(root), before, "{capability}: nothing is written");
    }
    // No capability at all.
    assert!(matches!(
        create(root, &NewChangeRequest::new("fresh-change")),
        Err(NewChangeError::NoCapability)
    ));
    assert_eq!(snapshot(root), before);
    // The neighbour.
    created(
        root,
        &NewChangeRequest::new("fresh-change").capability("cap:beta"),
    );
}

#[test]
fn a_capability_whose_document_did_not_load_is_named_and_never_skipped() {
    let scratch = repo("new-unloaded");
    let root = scratch.path();
    let text = read(root, BETA).replace("state: implementing", "state: finished");
    put(root, BETA, &text);
    let before = snapshot(root);
    match create(
        root,
        &NewChangeRequest::new("fresh-change").capability("cap:beta"),
    ) {
        Err(NewChangeError::CapabilityNotLoaded {
            capability,
            findings,
        }) => {
            assert_eq!(capability, "cap:beta");
            assert_ne!(findings.len(), 0);
            assert!(findings.iter().all(|finding| finding.file == BETA));
        }
        other => panic!("cap:beta did not load, got {other:?}"),
    }
    assert_eq!(snapshot(root), before);
    // Another capability still works.
    created(
        root,
        &NewChangeRequest::new("fresh-change").capability("cap:alpha"),
    );
}

#[test]
fn an_affects_entry_must_name_a_listed_capability_and_one_of_the_seven_sections() {
    let scratch = repo("new-affects");
    let root = scratch.path();
    let before = snapshot(root);
    for (entry, problem) in [
        ("cap:beta", AffectsProblem::Malformed),
        ("beta#contract", AffectsProblem::Malformed),
        ("#contract", AffectsProblem::Malformed),
        ("cap:#contract", AffectsProblem::Malformed),
        ("change:grow#contract", AffectsProblem::Malformed),
        (
            "cap:beta#nope",
            AffectsProblem::UnknownSection {
                section: "nope".to_string(),
            },
        ),
        (
            "cap:beta#Contract",
            AffectsProblem::UnknownSection {
                section: "Contract".to_string(),
            },
        ),
        (
            "cap:beta#",
            AffectsProblem::UnknownSection {
                section: String::new(),
            },
        ),
        // A capability that exists and is not listed.
        ("cap:alpha#contract", AffectsProblem::NotListed),
        // A capability that does not exist is not listed either.
        ("cap:ghost#contract", AffectsProblem::NotListed),
    ] {
        let request = NewChangeRequest::new("fresh-change")
            .capability("cap:beta")
            .affects("cap:beta#usage")
            .affects(entry);
        match create(root, &request) {
            Err(NewChangeError::BadAffects {
                entry: refused,
                problem: found,
            }) => {
                assert_eq!(refused, entry);
                assert_eq!(found, problem, "{entry}");
            }
            other => panic!("`{entry}` should be refused, got {other:?}"),
        }
        assert_eq!(snapshot(root), before, "{entry}: nothing is written");
    }
    // The neighbour: every section is accepted.
    let mut request = NewChangeRequest::new("fresh-change").capability("cap:beta");
    for section in S::ALL {
        request = request.affects(format!("cap:beta#{}", section.slug()));
    }
    let result = created(root, &request);
    assert_eq!(result.affects.len(), 7);
}

#[test]
fn several_capabilities_are_listed_in_the_order_given_without_repeats() {
    let scratch = repo("new-several");
    let root = scratch.path();
    let request = NewChangeRequest::new("fresh-change")
        .capability("cap:gamma")
        .capability("cap:alpha")
        .capability("cap:gamma")
        .capability("cap:beta");
    let result = created(root, &request);
    assert_eq!(result.capabilities, ["cap:gamma", "cap:alpha", "cap:beta"]);
    let text = read(root, FRESH);
    assert!(
        text.contains("changes: [cap:gamma, cap:alpha, cap:beta]\n"),
        "{text}"
    );
    assert!(
        text.contains(
            "affects:\n  - cap:gamma#contract\n  - cap:alpha#contract\n  - cap:beta#contract\n"
        ),
        "{text}"
    );
    on_disk(root, |plan| {
        let record = plan.change("fresh-change").expect("loads").value();
        assert_eq!(record.changes, ["cap:gamma", "cap:alpha", "cap:beta"]);
    });
    support::assert_clean(&findings_about(root, FRESH));
}

#[test]
fn the_default_affects_is_the_contract_of_each_listed_capability() {
    let scratch = repo("new-default");
    let root = scratch.path();
    let result = created(
        root,
        &NewChangeRequest::new("fresh-change")
            .capability("cap:alpha")
            .capability("cap:gamma"),
    );
    assert_eq!(
        result.affects,
        [
            SectionId::new("cap:alpha", S::Contract),
            SectionId::new("cap:gamma", S::Contract)
        ]
    );
    assert!(
        read(root, FRESH).contains("affects:\n  - cap:alpha#contract\n  - cap:gamma#contract\n")
    );
    // `impact` then reports exactly those sections.
    on_disk(root, |plan| {
        let impact = plan.impact("fresh-change").expect("impact");
        assert_eq!(impact.sections, result.affects);
        assert_eq!(impact.capabilities, ["cap:alpha", "cap:gamma"]);
    });
}

#[test]
fn explicit_affects_are_written_as_given_and_impact_reports_them_sorted() {
    let scratch = repo("new-explicit");
    let root = scratch.path();
    let result = created(
        root,
        &NewChangeRequest::new("fresh-change")
            .capability("cap:alpha")
            .capability("cap:beta")
            .affects("cap:beta#verification")
            .affects("cap:alpha#usage")
            .affects("cap:beta#purpose")
            .affects("cap:alpha#usage"),
    );
    // As given, without the repeat, and nothing added for a capability the
    // entries do not mention.
    assert_eq!(
        result.affects,
        [
            SectionId::new("cap:beta", S::Verification),
            SectionId::new("cap:alpha", S::Usage),
            SectionId::new("cap:beta", S::Purpose)
        ]
    );
    assert!(read(root, FRESH).contains(
        "affects:\n  - cap:beta#verification\n  - cap:alpha#usage\n  - cap:beta#purpose\n"
    ));
    assert!(!read(root, FRESH).contains("#contract"));
    support::assert_clean(&findings_about(root, FRESH));
    on_disk(root, |plan| {
        let impact = plan.impact("fresh-change").expect("impact");
        assert_eq!(
            impact.sections,
            [
                SectionId::new("cap:alpha", S::Usage),
                SectionId::new("cap:beta", S::Purpose),
                SectionId::new("cap:beta", S::Verification)
            ]
        );
    });
}

#[test]
fn a_slug_that_reads_as_a_tracker_reference_leaves_no_file() {
    // The record is read back through the loader and must have no finding.
    let scratch = repo("new-invalid");
    let root = scratch.path();
    let before = snapshot(root);
    // Spelled in two pieces so this file does not hold a tracker reference.
    let slug = ["g", "h-123"].concat();
    match create(
        root,
        &NewChangeRequest::new(slug.as_str()).capability("cap:beta"),
    ) {
        Err(NewChangeError::Invalid { path, findings }) => {
            assert_eq!(path.as_str(), format!("work/changes/{slug}.md"));
            assert_eq!(kinds(&findings), ["issue-reference"], "{findings:#?}");
        }
        other => panic!("the record should be refused, got {other:?}"),
    }
    assert_eq!(snapshot(root), before, "the refused record is removed");
    // The neighbour.
    created(
        root,
        &NewChangeRequest::new("gh-fix").capability("cap:beta"),
    );
}

#[test]
fn work_changes_is_created_when_the_repository_has_none() {
    let scratch = Scratch::new("new-mkdir");
    let root = scratch.path();
    let mut repo = synced::base();
    repo.remove(GROW);
    write_repo(root, &repo);
    assert!(!root.join("work").exists());
    let before = snapshot(root);
    created(
        root,
        &NewChangeRequest::new("fresh-change").capability("cap:beta"),
    );
    assert_eq!(changed(&before, &snapshot(root)), [FRESH]);
    assert_eq!(read(root, FRESH), FRESH_BETA);
}

#[test]
fn a_file_or_a_link_where_work_changes_belongs_is_refused() {
    let scratch = Scratch::new("new-refused-dir");
    let root = scratch.path();
    let mut repo = synced::base();
    repo.remove(GROW);
    write_repo(root, &repo);
    put(root, "work", "a file\n");
    let before = snapshot(root);
    let request = NewChangeRequest::new("fresh-change").capability("cap:beta");
    match create(root, &request) {
        Err(NewChangeError::Write { path, .. }) => assert_eq!(path, "work"),
        other => panic!("`work` is a file, got {other:?}"),
    }
    assert_eq!(snapshot(root), before);

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        std::fs::remove_file(root.join("work")).expect("removes");
        let outside = Scratch::new("new-refused-outside");
        std::fs::create_dir_all(root.join("work")).expect("creates");
        symlink(outside.path(), root.join("work/changes")).expect("links");
        match create(root, &request) {
            Err(NewChangeError::Write { path, .. }) => assert_eq!(path, "work/changes"),
            other => panic!("`work/changes` is a link, got {other:?}"),
        }
        assert_eq!(
            snapshot(outside.path()).len(),
            0,
            "nothing reached the target"
        );
    }
}

#[test]
fn the_guidance_follows_the_state_of_each_listed_capability() {
    assert_eq!(
        StateAdvice::for_state(State::Delivered),
        StateAdvice::SetImplementing
    );
    assert_eq!(
        StateAdvice::for_state(State::Designed),
        StateAdvice::SetImplementing
    );
    assert_eq!(StateAdvice::for_state(State::Considered), StateAdvice::Keep);
    assert_eq!(
        StateAdvice::for_state(State::Implementing),
        StateAdvice::Keep
    );

    let scratch = repo("new-guidance");
    let result = created(
        scratch.path(),
        &NewChangeRequest::new("fresh-change")
            .capability("cap:alpha")
            .capability("cap:beta")
            .capability("cap:gamma")
            .capability("cap:delta"),
    );
    let seen: Vec<(&str, State, StateAdvice)> = result
        .guidance
        .iter()
        .map(|entry| (entry.capability.as_str(), entry.state, entry.advice))
        .collect();
    assert_eq!(
        seen,
        [
            ("cap:alpha", State::Designed, StateAdvice::SetImplementing),
            ("cap:beta", State::Implementing, StateAdvice::Keep),
            ("cap:gamma", State::Delivered, StateAdvice::SetImplementing),
            ("cap:delta", State::Considered, StateAdvice::Keep),
        ]
    );
}

#[test]
fn the_printed_text_holds_the_template_and_the_guidance() {
    let scratch = repo("new-print");
    let result = created(
        scratch.path(),
        &NewChangeRequest::new("fresh-change")
            .capability("cap:alpha")
            .capability("cap:beta"),
    );
    assert_eq!(
        result.template,
        "> [!PROPOSED change:fresh-change]\n> State the proposed text here.\n"
    );
    assert!(result.sync_needed);
    assert_eq!(
        result.to_string(),
        "\
created work/changes/fresh-change.md
changes: cap:alpha, cap:beta
affects: cap:alpha#contract, cap:beta#contract

Place this block in a section named in `affects`, in the document of a listed capability, and replace its second line with the proposed text:

> [!PROPOSED change:fresh-change]
> State the proposed text here.

State of the listed capabilities:
  cap:alpha  designed  set `state: implementing` in the same commit as the block, since a PROPOSED block is a finding of `check` in this state (`state-designed-proposal`)
  cap:beta  implementing  no change of state is needed

The record makes generated blocks stale. Stage the record first, since a link to a file git does not track is a finding of `check`, then bring the blocks up to date:
  git add work/changes/fresh-change.md
  docs sync
"
    );
}

// --- the template in each state ----------------------------------------------

/// The repository with the record `fresh-change` for `capability`, the guidance
/// followed (the state set to `implementing` when it says so) and the template
/// placed in the Contract section of the capability.
fn with_template_placed(
    tag: &str,
    capability: &str,
    path: &str,
    follow: bool,
) -> (Scratch, NewChange) {
    let scratch = repo(tag);
    let root = scratch.path();
    let result = created(
        root,
        &NewChangeRequest::new("fresh-change").capability(capability),
    );
    let mut text = read(root, path);
    let anchor = format!("Text for Contract of {}.\n", &capability[4..]);
    assert!(text.contains(&anchor), "{path}");
    text = text.replacen(&anchor, &format!("{anchor}\n{}", result.template), 1);
    if follow && result.guidance[0].advice == StateAdvice::SetImplementing {
        let state = format!("state: {}\n", result.guidance[0].state.name());
        assert!(text.contains(&state));
        text = text.replacen(&state, "state: implementing\n", 1);
    }
    put(root, path, &text);
    (scratch, result)
}

/// Sync the generated blocks of the repository under `root` to the disk.
fn sync_to_disk(root: &Path) {
    let report = on_disk(root, |plan| {
        plan.sync(&SyncOptions::new("2026 Example", "CC-BY-4.0"))
    })
    .expect("syncs");
    Workplan::write(&report, root).expect("writes");
}

#[test]
fn the_template_in_a_capability_of_each_state_gives_one_block_and_a_clean_check() {
    for (capability, path, state) in [
        ("cap:delta", DELTA, State::Considered),
        ("cap:alpha", ALPHA, State::Designed),
        ("cap:beta", BETA, State::Implementing),
        ("cap:gamma", GAMMA, State::Delivered),
    ] {
        let (scratch, result) = with_template_placed("template", capability, path, true);
        let root = scratch.path();
        assert_eq!(result.guidance[0].state, state, "{capability}");
        on_disk(root, |plan| {
            // Exactly one block of the change in the graph.
            let blocks: Vec<_> = plan
                .graph()
                .proposals()
                .iter()
                .filter(|proposal| proposal.change() == "fresh-change")
                .collect();
            assert_eq!(blocks.len(), 1, "{capability}");
            assert_eq!(blocks[0].capability(), capability);
            assert_eq!(blocks[0].section(), Some(S::Contract));
            // The only finding left says the generated blocks are not synced.
            let findings = plan.check();
            assert!(
                findings.iter().all(|f| f.kind == "generated-stale"),
                "{capability}: {findings:#?}"
            );
            assert_eq!(
                !findings.is_empty(),
                result.sync_needed,
                "{capability}: `sync_needed` says whether a sync is owed"
            );
        });
        sync_to_disk(root);
        on_disk(root, |plan| {
            let findings = plan.check();
            assert!(findings.is_empty(), "{capability}: {findings:#?}");
            assert_eq!(
                plan.graph()
                    .proposals()
                    .iter()
                    .filter(|proposal| proposal.change() == "fresh-change")
                    .count(),
                1
            );
        });
    }
}

#[test]
fn the_template_in_the_wrong_state_is_the_finding_the_guidance_names() {
    // Designed: a PROPOSED block is a finding until the state is implementing.
    let (scratch, _) = with_template_placed("wrong-designed", "cap:alpha", ALPHA, false);
    let findings = findings_about(scratch.path(), ALPHA);
    assert_eq!(
        kinds(&findings),
        ["state-designed-proposal", "generated-stale"],
        "{findings:#?}"
    );

    // Delivered: the block, and the open record beside a delivered capability.
    let (scratch, _) = with_template_placed("wrong-delivered", "cap:gamma", GAMMA, false);
    let mut found = kinds(&findings_about(scratch.path(), GAMMA));
    found.sort_unstable();
    assert_eq!(
        found,
        [
            "generated-stale",
            "state-delivered-open-record",
            "state-delivered-proposal"
        ]
    );

    // The same two states with the guidance followed hold neither finding.
    for (capability, path) in [("cap:alpha", ALPHA), ("cap:gamma", GAMMA)] {
        let (scratch, _) = with_template_placed("followed", capability, path, true);
        let found = kinds(&findings_about(scratch.path(), path));
        assert!(
            found.iter().all(|kind| *kind == "generated-stale"),
            "{found:?}"
        );
    }
}

#[test]
fn the_record_beside_a_delivered_capability_is_a_finding_until_the_state_is_set() {
    let scratch = repo("new-delivered");
    let root = scratch.path();
    created(
        root,
        &NewChangeRequest::new("fresh-change").capability("cap:gamma"),
    );
    let found = kinds(&findings_about(root, GAMMA));
    assert!(found.contains(&"state-delivered-open-record"), "{found:?}");
    // A designed capability needs nothing for the record alone.
    created(
        root,
        &NewChangeRequest::new("another").capability("cap:alpha"),
    );
    assert!(
        findings_about(root, ALPHA)
            .iter()
            .all(|finding| finding.kind == "generated-stale")
    );
}

// --- impact ------------------------------------------------------------------

/// The repository for the impact fixtures: the generated-block repository with
/// links into the Usage section of `gamma` and the Purpose section of `beta`, a
/// binding of `beta` that does not resolve, and the change `reach`.
fn reach_repo(tag: &str) -> Scratch {
    let scratch = repo(tag);
    let root = scratch.path();
    let replace = |path: &str, from: &str, to: &str| {
        let text = read(root, path);
        assert!(text.contains(from), "{path} has no `{from}`");
        put(root, path, &text.replacen(from, to, 1));
    };
    replace(
        ALPHA,
        "Text for Constraints of alpha.\n",
        "Text for Constraints of alpha. See [usage](gamma.md#usage) and [the whole](gamma.md).\n",
    );
    replace(
        BETA,
        "Text for Explanation of beta.\n",
        "Text for Explanation of beta. See [usage](gamma.md#usage).\n",
    );
    replace(
        GAMMA,
        "Text for Examples of gamma.\n",
        "Text for Examples of gamma. See [purpose](beta.md#purpose).\n",
    );
    replace(BETA, "tests: []", "tests: [demo::test_crate::it::absent]");
    put(
        root,
        "work/changes/reach.md",
        "---\ntype: Change\nid: change:reach\ntitle: Reach\ndescription: Reaches things.\n\
         changes: [cap:gamma, cap:beta]\naffects:\n  - cap:gamma#usage\n  - cap:beta#purpose\n\
         contract_effect: changes\nitems: []\n---\n\n## Summary\n\nReaches.\n",
    );
    scratch
}

#[test]
fn impact_gives_the_capabilities_sections_bindings_dependants_and_linking_sections() {
    let scratch = reach_repo("impact");
    let root = scratch.path();
    let before = snapshot(root);
    let impact = on_disk(root, |plan| plan.impact("reach")).expect("impact");

    assert_eq!(impact.change, "reach");
    assert_eq!(impact.capabilities, ["cap:beta", "cap:gamma"]);
    assert_eq!(
        impact.sections,
        [
            SectionId::new("cap:beta", S::Purpose),
            SectionId::new("cap:gamma", S::Usage)
        ]
    );
    let bindings: Vec<(&str, &str, &str, bool)> = impact
        .bindings
        .iter()
        .map(|binding| {
            (
                binding.capability(),
                binding.kind().key(),
                binding.path(),
                binding.resolution().is_ok(),
            )
        })
        .collect();
    assert_eq!(
        bindings,
        [
            ("cap:beta", "code", "demo::bare", true),
            ("cap:beta", "tests", "demo::test_crate::it::absent", false),
            ("cap:gamma", "code", "demo::real", true),
            ("cap:gamma", "tests", "demo::test_crate::it::case", true),
            ("cap:gamma", "examples", "demo::example_crate::show", true),
            ("cap:gamma", "examples", "demo::real", true),
        ]
    );
    let dependants: Vec<(&str, &str)> = impact
        .dependants
        .iter()
        .map(|d| (d.capability.as_str(), d.depends_on.as_str()))
        .collect();
    assert_eq!(
        dependants,
        [
            ("cap:alpha", "cap:beta"),
            ("cap:alpha", "cap:gamma"),
            ("cap:beta", "cap:gamma")
        ]
    );
    let linking: Vec<(String, String)> = impact
        .linking
        .iter()
        .map(|l| (l.from.to_string(), l.target.to_string()))
        .collect();
    assert_eq!(
        linking,
        [
            (
                "cap:gamma#examples".to_string(),
                "cap:beta#purpose".to_string()
            ),
            (
                "cap:alpha#constraints".to_string(),
                "cap:gamma#usage".to_string()
            ),
            (
                "cap:beta#explanation".to_string(),
                "cap:gamma#usage".to_string()
            ),
            // The verification matrix of `gamma` links to the sections its
            // rows substantiate.
            (
                "cap:gamma#verification".to_string(),
                "cap:gamma#usage".to_string()
            ),
        ]
    );

    // Reading it wrote nothing.
    assert_eq!(snapshot(root), before);
}

#[test]
fn impact_prints_each_part_in_a_fixed_order() {
    let scratch = reach_repo("impact-text");
    let root = scratch.path();
    let text = on_disk(root, |plan| plan.impact("reach"))
        .expect("impact")
        .to_string();
    let reason = on_disk(root, |plan| {
        let impact = plan.impact("reach").expect("impact");
        impact
            .bindings
            .iter()
            .find_map(|binding| binding.resolution().err().map(ToString::to_string))
            .expect("one binding does not resolve")
    });
    assert_eq!(
        text,
        format!(
            "\
change:reach
capabilities:
  cap:beta
  cap:gamma
sections:
  cap:beta#purpose
  cap:gamma#usage
bindings:
  cap:beta  code  demo::bare  resolves
  cap:beta  tests  demo::test_crate::it::absent  does not resolve: {reason}
  cap:gamma  code  demo::real  resolves
  cap:gamma  tests  demo::test_crate::it::case  resolves
  cap:gamma  examples  demo::example_crate::show  resolves
  cap:gamma  examples  demo::real  resolves
dependants:
  cap:alpha depends on cap:beta
  cap:alpha depends on cap:gamma
  cap:beta depends on cap:gamma
linking sections:
  cap:gamma#examples links to cap:beta#purpose
  cap:alpha#constraints links to cap:gamma#usage
  cap:beta#explanation links to cap:gamma#usage
  cap:gamma#verification links to cap:gamma#usage
"
        )
    );
    // Two runs print the same text, and the prefix of the slug does not matter.
    let again = on_disk(root, |plan| plan.impact("change:reach")).expect("impact");
    assert_eq!(again.to_string(), text);
}

#[test]
fn impact_of_a_change_that_reaches_nothing_says_none() {
    let scratch = repo("impact-none");
    let root = scratch.path();
    // `delta` binds nothing, nothing depends on it and nothing links to it.
    created(
        root,
        &NewChangeRequest::new("fresh-change").capability("cap:delta"),
    );
    let impact = on_disk(root, |plan| plan.impact("fresh-change")).expect("impact");
    assert_eq!(
        impact.to_string(),
        "\
change:fresh-change
capabilities:
  cap:delta
sections:
  cap:delta#contract
bindings:
  none
dependants:
  none
linking sections:
  none
"
    );
}

#[test]
fn impact_of_the_repositorys_own_change_lists_what_the_record_says() {
    let scratch = repo("impact-grow");
    let impact = on_disk(scratch.path(), |plan| plan.impact("grow")).expect("impact");
    assert_eq!(impact.capabilities, ["cap:alpha", "cap:beta", "cap:delta"]);
    assert_eq!(impact.sections, [SectionId::new("cap:delta", S::Purpose)]);
    assert_eq!(
        impact
            .dependants
            .iter()
            .map(|d| (d.capability.as_str(), d.depends_on.as_str()))
            .collect::<Vec<_>>(),
        [("cap:alpha", "cap:beta")]
    );
}

#[test]
fn impact_tells_an_unknown_slug_from_a_record_that_did_not_load() {
    let scratch = reach_repo("impact-lookup");
    let root = scratch.path();
    put(
        root,
        "work/changes/broken.md",
        "---\ntype: Change\nid: change:broken\ntitle: Broken\ndescription: Does not load.\n\
         changes: [cap:beta]\naffects: [cap:beta#purpose]\ncontract_effect: sometimes\n\
         items: []\n---\n\n## Summary\n\nBroken.\n",
    );
    let before = snapshot(root);
    match on_disk(root, |plan| plan.impact("ghost")) {
        Err(ChangeLookupError::Unknown { slug, known }) => {
            assert_eq!(slug, "ghost");
            assert_eq!(known, ["grow", "reach"]);
        }
        other => panic!("`ghost` is unknown, got {other:?}"),
    }
    match on_disk(root, |plan| plan.impact("change:ghost")) {
        Err(ChangeLookupError::Unknown { slug, .. }) => assert_eq!(slug, "ghost"),
        other => panic!("`change:ghost` is unknown, got {other:?}"),
    }
    match on_disk(root, |plan| plan.impact("broken")) {
        Err(ChangeLookupError::NotLoaded { slug, findings }) => {
            assert_eq!(slug, "broken");
            assert_ne!(findings.len(), 0);
            assert!(
                findings
                    .iter()
                    .all(|finding| finding.file == "work/changes/broken.md")
            );
        }
        other => panic!("`broken` did not load, got {other:?}"),
    }
    // The neighbour.
    assert!(on_disk(root, |plan| plan.impact("reach")).is_ok());
    assert_eq!(snapshot(root), before, "no lookup wrote anything");
}

#[test]
fn impact_of_a_change_that_just_was_created_reads_the_record_from_the_disk() {
    let scratch = repo("impact-new");
    let root = scratch.path();
    created(
        root,
        &NewChangeRequest::new("fresh-change")
            .capability("cap:gamma")
            .affects("cap:gamma#examples")
            .affects("cap:gamma#usage"),
    );
    let before = snapshot(root);
    let impact = on_disk(root, |plan| plan.impact("fresh-change")).expect("impact");
    assert_eq!(
        impact.sections,
        [
            SectionId::new("cap:gamma", S::Usage),
            SectionId::new("cap:gamma", S::Examples)
        ]
    );
    assert_eq!(snapshot(root), before);
}

// --- the SPDX lines ----------------------------------------------------------

/// The tags of an SPDX header, in two pieces so this file does not carry them.
const COPYRIGHT_TAG: &str = concat!("SPDX-", "FileCopyrightText: ");
const LICENCE_TAG: &str = concat!("SPDX-", "License-Identifier: ");

const COPYRIGHT_TEXT: &str = "2026 Example Org <dev@example.org>";
const LICENCE_ID: &str = "CC-BY-4.0";

/// The leading comment lines of the first key of a record.
fn leading_comments(text: &str) -> Vec<String> {
    let document = Document::parse(text).expect("reads");
    document.frontmatter().map().entries()[0]
        .comments()
        .leading()
        .to_vec()
}

#[test]
fn both_spdx_fields_open_the_record_with_the_two_comment_lines_of_the_records_of_the_repository() {
    let scratch = repo("new-spdx");
    let root = scratch.path();
    let before = snapshot(root);
    let request = NewChangeRequest::new("fresh-change")
        .capability("cap:beta")
        .copyright(COPYRIGHT_TEXT)
        .licence(LICENCE_ID);
    assert_eq!(request.copyright.as_deref(), Some(COPYRIGHT_TEXT));
    let result = created(root, &request);
    // One file, and it is the record.
    assert_eq!(changed(&before, &snapshot(root)), [FRESH]);
    let text = read(root, FRESH);
    assert_eq!(
        text,
        format!(
            "---\n# {COPYRIGHT_TAG}{COPYRIGHT_TEXT}\n# {LICENCE_TAG}{LICENCE_ID}\n{}",
            FRESH_BETA.strip_prefix("---\n").expect("a fence")
        )
    );
    assert_eq!(result.path.as_str(), FRESH);

    // The lines read back as the comments that precede the first key, which is
    // how the change records of the repository carry the annotation that
    // `reuse lint` reads.
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../work/changes");
    let mut records: Vec<_> = std::fs::read_dir(&directory)
        .expect("the change records of the repository")
        .map(|entry| entry.expect("an entry").path())
        .collect();
    records.sort();
    let existing = std::fs::read_to_string(&records[0]).expect("a record");
    let existing = leading_comments(&existing);
    assert_eq!(existing.len(), 2, "{existing:?}");
    assert_eq!(
        leading_comments(&text),
        [
            format!("# {COPYRIGHT_TAG}{COPYRIGHT_TEXT}"),
            format!("# {LICENCE_TAG}{LICENCE_ID}"),
        ]
    );
    assert!(existing[0].starts_with(&format!("# {COPYRIGHT_TAG}")));
    assert!(existing[1].starts_with(&format!("# {LICENCE_TAG}")));
    // The record loads with no finding.
    on_disk(root, |plan| {
        assert!(plan.change("fresh-change").is_ok());
        assert!(plan.check().iter().all(|finding| finding.file != FRESH));
    });
}

#[test]
fn neither_spdx_field_writes_no_comment_line() {
    let scratch = repo("new-no-spdx");
    created(
        scratch.path(),
        &NewChangeRequest::new("fresh-change").capability("cap:beta"),
    );
    assert_eq!(read(scratch.path(), FRESH), FRESH_BETA);
    assert_eq!(leading_comments(FRESH_BETA), Vec::<String>::new());
}

#[test]
fn one_spdx_field_without_the_other_is_an_error_that_names_the_missing_one() {
    let scratch = repo("new-spdx-one");
    let root = scratch.path();
    let before = snapshot(root);
    for (request, missing) in [
        (
            NewChangeRequest::new("fresh-change")
                .capability("cap:beta")
                .copyright(COPYRIGHT_TEXT),
            "licence",
        ),
        (
            NewChangeRequest::new("fresh-change")
                .capability("cap:beta")
                .licence(LICENCE_ID),
            "copyright",
        ),
    ] {
        match create(root, &request) {
            Err(NewChangeError::SpdxUnset { field }) => assert_eq!(field, missing),
            other => panic!("the {missing} is missing, got {other:?}"),
        }
        let text = create(root, &request).expect_err("refused").to_string();
        assert!(
            text.contains(&format!("the {missing} is not given")),
            "{text}"
        );
        assert_eq!(snapshot(root), before, "nothing is written");
    }
}

#[test]
fn an_spdx_field_that_is_not_one_line_of_text_is_refused_and_nothing_is_written() {
    let scratch = repo("new-spdx-bad");
    let root = scratch.path();
    let before = snapshot(root);
    for (copyright, licence, field) in [
        ("", LICENCE_ID, "copyright"),
        ("   ", LICENCE_ID, "copyright"),
        ("two\nlines", LICENCE_ID, "copyright"),
        ("carriage\rreturn", LICENCE_ID, "copyright"),
        ("tab\there", LICENCE_ID, "copyright"),
        ("line\u{2028}separator", LICENCE_ID, "copyright"),
        ("paragraph\u{2029}separator", LICENCE_ID, "copyright"),
        ("next\u{85}line", LICENCE_ID, "copyright"),
        ("form\u{c}feed", LICENCE_ID, "copyright"),
        ("vertical\u{b}tab", LICENCE_ID, "copyright"),
        (COPYRIGHT_TEXT, "", "licence"),
        (COPYRIGHT_TEXT, "MIT\nOR Apache-2.0", "licence"),
        (COPYRIGHT_TEXT, "\u{7}", "licence"),
        (COPYRIGHT_TEXT, "MIT\u{2028}OR Apache-2.0", "licence"),
        (COPYRIGHT_TEXT, "MIT\u{2029}", "licence"),
    ] {
        let request = NewChangeRequest::new("fresh-change")
            .capability("cap:beta")
            .copyright(copyright)
            .licence(licence);
        match create(root, &request) {
            Err(NewChangeError::SpdxInvalid { field: refused, .. }) => {
                assert_eq!(refused, field, "{copyright:?} {licence:?}");
            }
            other => panic!("{copyright:?} {licence:?} should be refused, got {other:?}"),
        }
        assert_eq!(snapshot(root), before);
    }
    // The neighbour: surrounding blanks are trimmed and the text is kept.
    created(
        root,
        &NewChangeRequest::new("fresh-change")
            .capability("cap:beta")
            .copyright(format!("  {COPYRIGHT_TEXT}  "))
            .licence(LICENCE_ID),
    );
    assert!(read(root, FRESH).contains(&format!("# {COPYRIGHT_TAG}{COPYRIGHT_TEXT}\n")));
}

// --- the note about the generated blocks -------------------------------------

/// The text inside the first relationships block of the file, replaced by
/// `text`: a block that is stale before any record exists.
fn stale_relationships(root: &Path, path: &str) {
    let marker = "<!-- raf:generated:relationships -->\n";
    let text = read(root, path);
    assert!(text.contains(marker), "{path}");
    put(
        root,
        path,
        &text.replacen(marker, &format!("{marker}stale\n"), 1),
    );
}

#[test]
fn the_sync_note_is_there_when_the_target_file_was_already_stale() {
    // The record lists `alpha`, whose table is stale before the record exists.
    // A sync is owed, and the note says so.
    let scratch = repo("new-stale-before");
    let root = scratch.path();
    stale_relationships(root, ALPHA);
    assert_eq!(
        kinds(&findings_about(root, ALPHA)),
        ["generated-stale"],
        "the table was stale before the record"
    );
    let result = created(
        root,
        &NewChangeRequest::new("fresh-change").capability("cap:alpha"),
    );
    assert!(result.sync_needed);
    assert!(
        result
            .to_string()
            .contains("  git add work/changes/fresh-change.md\n  docs sync\n")
    );

    // The neighbours: the table was current (the record stales it), and the
    // record stales nothing while an unrelated file is stale.
    let scratch = repo("new-current-before");
    let result = created(
        scratch.path(),
        &NewChangeRequest::new("fresh-change").capability("cap:alpha"),
    );
    assert!(result.sync_needed);

    let scratch = repo("new-unrelated-stale");
    let root = scratch.path();
    stale_relationships(root, GAMMA);
    let result = created(
        root,
        &NewChangeRequest::new("fresh-change").capability("cap:delta"),
    );
    assert!(
        !result.sync_needed,
        "a stale file the record does not reach is not the record's"
    );
    assert!(!result.to_string().contains("docs sync"));
}

#[test]
fn the_guidance_for_a_delivered_capability_names_both_findings_it_avoids() {
    let scratch = repo("new-delivered-guidance");
    let result = created(
        scratch.path(),
        &NewChangeRequest::new("fresh-change")
            .capability("cap:gamma")
            .capability("cap:alpha"),
    );
    let text = result.to_string();
    let gamma = text
        .lines()
        .find(|line| line.starts_with("  cap:gamma  delivered  "))
        .expect("a line for the delivered capability");
    assert!(gamma.contains("`state-delivered-proposal`"), "{gamma}");
    assert!(gamma.contains("`state-delivered-open-record`"), "{gamma}");
    let alpha = text
        .lines()
        .find(|line| line.starts_with("  cap:alpha  designed  "))
        .expect("a line for the designed capability");
    assert!(alpha.contains("`state-designed-proposal`"), "{alpha}");
    assert!(!alpha.contains("open-record"), "{alpha}");
    assert_eq!(StateAdvice::Keep.name(), "keep");
    assert_eq!(StateAdvice::SetImplementing.name(), "set-implementing");
}
