// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The rules between a capability, its PROPOSED blocks and the change
//! records: each state accepted with matching content and rejected with a
//! mismatch, each block rule rejected beside a valid neighbour, and the
//! single-document state rules surfaced from `raf-okf`.

mod support;

use raf_okf::Location;
use support::cases::BETA_BLOCK;
use support::fixtures::{
    ALPHA, BETA, CHANGE, GAMMA, OVERVIEW, capability, change_record, mutate, mutated, valid,
};
use support::{assert_clean, assert_hit, assert_kind, check, key, line_of};

/// Where the first PROPOSED marker of `path` is, in `repo`.
fn marker(repo: &support::Repo, path: &str) -> Location {
    Location::Line(line_of(&repo[path], "[!PROPOSED"))
}

/// Add the capability `slug` in `state`, with `body` in its Purpose, and an
/// open record `add-<slug>` that lists it and affects its Purpose.
fn with_listed(slug: &str, state: &str, body: &str) -> support::Repo {
    let mut repo = valid();
    repo.insert(
        format!("docs/capabilities/{slug}.md"),
        capability(slug, state, "[]", body),
    );
    repo.insert(
        format!("work/changes/add-{slug}.md"),
        change_record(
            &format!("add-{slug}"),
            &format!("[cap:{slug}]"),
            &format!("  - cap:{slug}#purpose\n"),
            "",
        ),
    );
    repo
}

#[test]
fn accepts_each_state_with_matching_content() {
    // Considered: the candidate contract in a block, an open record lists it.
    // Designed: no block, an open record lists it. Delivered: no block, no
    // open record. All three are in the valid repository.
    assert_clean(&check(&valid()));
    // Implementing: an open record lists it, and a block sits beside the
    // current text.
    let block = "> [!PROPOSED change:add-delta]\n> Chunked input.\n\n";
    assert_clean(&check(&with_listed("delta", "implementing", block)));
    assert_clean(&check(&with_listed("delta", "implementing", "")));
}

#[test]
fn rejects_orphan_proposed_marker() {
    let repo = mutate(BETA, "change:add-alpha", "change:ghost");
    let findings = check(&repo);
    assert_hit(
        &findings,
        "proposed-missing-change",
        BETA,
        &marker(&repo, BETA),
    );
    assert!(findings[0].message.contains("work/changes/ghost.md"));
}

#[test]
fn rejects_considered_without_proposed_block() {
    let findings = mutated(BETA, BETA_BLOCK, "");
    assert_hit(
        &findings,
        "state-considered-no-proposal",
        BETA,
        &key("state"),
    );
}

#[test]
fn rejects_open_capability_without_change_record() {
    let mut repo = valid();
    repo.insert(
        CHANGE.to_string(),
        change_record(
            "add-alpha",
            "[cap:beta]",
            "  - cap:beta#purpose\n",
            "  - claim: \"beta is sketched\"\n",
        ),
    );
    assert_hit(&check(&repo), "no-open-record", ALPHA, &key("state"));
}

#[test]
fn rejects_each_non_delivered_state_without_an_open_record() {
    for state in ["designed", "implementing"] {
        let mut repo = with_listed("delta", state, "");
        repo.remove("work/changes/add-delta.md");
        assert_hit(
            &check(&repo),
            "no-open-record",
            "docs/capabilities/delta.md",
            &key("state"),
        );
    }
    // A delivered capability needs none.
    let mut repo = valid();
    repo.remove(CHANGE);
    let kinds: Vec<_> = check(&repo)
        .into_iter()
        .filter(|f| f.file == GAMMA)
        .collect();
    assert!(kinds.is_empty(), "{kinds:#?}");
}

#[test]
fn rejects_delivered_with_proposed_block() {
    let findings = mutated(
        GAMMA,
        "Text for Purpose.\n",
        "Text for Purpose.\n\n> [!PROPOSED change:add-alpha]\n> Left over.\n",
    );
    assert_hit(&findings, "state-delivered-proposal", GAMMA, &key("state"));
}

#[test]
fn rejects_delivered_with_open_change_record() {
    let findings = mutated(
        CHANGE,
        "changes: [cap:alpha, cap:beta]",
        "changes: [cap:alpha, cap:beta, cap:gamma]",
    );
    assert_hit(
        &findings,
        "state-delivered-open-record",
        GAMMA,
        &key("state"),
    );
    let finding = findings
        .iter()
        .find(|f| f.kind == "state-delivered-open-record")
        .expect("the finding");
    assert!(finding.message.contains(CHANGE), "{finding:#?}");
}

#[test]
fn rejects_designed_with_proposed_block() {
    let findings = mutated(
        ALPHA,
        "Text for Contract.\n",
        "> [!PROPOSED change:add-alpha]\n> Not yet.\n\nText for Contract.\n",
    );
    assert_hit(&findings, "state-designed-proposal", ALPHA, &key("state"));
}

#[test]
fn rejects_proposed_section_missing_from_affects() {
    let repo = mutate(CHANGE, "  - cap:beta#purpose\n", "  - cap:beta#usage\n");
    let findings = check(&repo);
    assert_hit(
        &findings,
        "proposed-section-not-affected",
        BETA,
        &marker(&repo, BETA),
    );
    let finding = findings
        .iter()
        .find(|f| f.kind == "proposed-section-not-affected")
        .expect("the finding");
    assert!(
        finding.message.contains("`cap:beta#purpose`") && finding.message.contains("add-alpha"),
        "{finding:#?}"
    );
}

#[test]
fn rejects_proposed_change_that_does_not_list_the_capability() {
    let mut repo = valid();
    repo.insert(
        CHANGE.to_string(),
        change_record(
            "add-alpha",
            "[cap:alpha]",
            "  - cap:alpha#contract\n",
            "  - claim: \"alpha reads input\"\n",
        ),
    );
    let findings = check(&repo);
    assert_hit(
        &findings,
        "proposed-change-not-listing",
        BETA,
        &marker(&repo, BETA),
    );
    // The unlisted capability has no open record either.
    assert_kind(&findings, "no-open-record", BETA);
}

#[test]
fn a_block_that_names_a_broken_change_is_not_a_missing_change() {
    // The record exists and fails its own checks, which say so.
    let findings = mutated(CHANGE, "outcome: implemented", "outcome: done");
    assert_kind(&findings, "okf-bad-outcome", CHANGE);
    assert!(
        findings.iter().all(|f| f.kind != "proposed-missing-change"),
        "{findings:#?}"
    );
}

#[test]
fn a_marker_in_a_code_fence_is_not_read_as_a_block() {
    let fenced = "```markdown\n> [!PROPOSED change:ghost]\n> Quoted.\n```\n\n";
    // Designed and delivered capabilities hold no block, and the quoted
    // marker, which names a change that does not exist, is not one.
    for path in [ALPHA, GAMMA] {
        assert_clean(&mutated(
            path,
            "Text for Purpose.\n",
            &format!("{fenced}Text for Purpose.\n"),
        ));
    }
    // A considered capability whose only marker is fenced has no block.
    let findings = mutated(BETA, BETA_BLOCK, fenced);
    assert_hit(
        &findings,
        "state-considered-no-proposal",
        BETA,
        &key("state"),
    );
    assert!(
        findings.iter().all(|f| f.kind != "proposed-missing-change"),
        "{findings:#?}"
    );
}

#[test]
fn surfaces_the_single_document_state_rules_from_the_profile() {
    // A delivered capability with no code binding.
    let findings = mutated(GAMMA, "  code: [demo::real]", "  code: []");
    assert_hit(&findings, "okf-binding-required", GAMMA, &key("bindings"));
    // A designed capability that binds code.
    let findings = mutated(ALPHA, "  code: []", "  code: [demo::real]");
    assert_hit(&findings, "okf-binding-not-empty", ALPHA, &key("bindings"));
    // A delivered section left empty by a comment.
    let findings = mutated(GAMMA, "Text for Usage.\n", "<!-- nothing yet -->\n");
    assert_hit(
        &findings,
        "okf-section-empty",
        GAMMA,
        &support::section("Usage"),
    );
    // A block in a document that is not a capability.
    let repo = mutate(
        OVERVIEW,
        "Body text.",
        "> [!PROPOSED change:add-alpha]\n> Not allowed here.",
    );
    assert_hit(
        &check(&repo),
        "okf-proposed-outside-capability",
        OVERVIEW,
        &marker(&repo, OVERVIEW),
    );
    // None of these is reported a second time by a rule of this crate.
    let kinds: Vec<&str> = mutated(GAMMA, "  code: [demo::real]", "  code: []")
        .iter()
        .map(|f| f.kind)
        .collect();
    assert_eq!(kinds, ["okf-binding-required"]);
}

#[test]
fn a_record_that_lists_two_capabilities_must_affect_the_section_of_each() {
    // The change lists alpha and beta and affects a section of beta only.
    // Alpha is implementing, so it may hold a block, in its Purpose.
    let mut repo = mutate(ALPHA, "state: designed", "state: implementing");
    let alpha = repo.get_mut(ALPHA).expect("alpha");
    *alpha = alpha.replacen(
        "Text for Purpose.\n",
        "> [!PROPOSED change:add-alpha]\n> Candidate.\n\nText for Purpose.\n",
        1,
    );
    let change = repo.get_mut(CHANGE).expect("change");
    *change = change.replacen("  - cap:alpha#contract\n", "", 1);
    let findings = check(&repo);
    assert_hit(
        &findings,
        "proposed-section-not-affected",
        ALPHA,
        &marker(&repo, ALPHA),
    );
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].message.contains("`cap:alpha#purpose`"));
    // With the entry for alpha it is accepted.
    let change = repo.get_mut(CHANGE).expect("change");
    *change = change.replacen(
        "  - cap:beta#purpose\n",
        "  - cap:beta#purpose\n  - cap:alpha#purpose\n",
        1,
    );
    assert_clean(&check(&repo));
}

#[test]
fn a_broken_record_does_not_make_the_capabilities_it_lists_unlisted() {
    // The record exists and fails its own checks, which say so; it may list
    // any capability, so none is reported as having no open record.
    let findings = mutated(CHANGE, "outcome: implemented", "outcome: done");
    assert_kind(&findings, "okf-bad-outcome", CHANGE);
    assert!(
        findings.iter().all(|f| f.kind != "no-open-record"),
        "{findings:#?}"
    );
    // Without any record the same capabilities are reported.
    let mut repo = valid();
    repo.remove(CHANGE);
    let findings = check(&repo);
    let unlisted = findings
        .iter()
        .filter(|f| f.kind == "no-open-record")
        .count();
    assert_eq!(unlisted, 2, "{findings:#?}");
}

#[test]
fn the_blocks_of_a_capability_that_does_not_validate_still_name_a_record() {
    let mut repo = mutate(BETA, "change:add-alpha", "change:ghost");
    let beta = repo.get_mut(BETA).expect("beta");
    *beta = beta.replacen("title: \"Title of beta\"", "title: \"\"", 1);
    let findings = check(&repo);
    assert_kind(&findings, "okf-empty-text", BETA);
    assert_hit(
        &findings,
        "proposed-missing-change",
        BETA,
        &marker(&repo, BETA),
    );
}
