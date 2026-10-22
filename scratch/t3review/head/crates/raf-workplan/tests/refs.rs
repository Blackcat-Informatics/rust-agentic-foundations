// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The one pattern set for issue and pull request numbers and process
//! wording, and the `issue-reference` rule that applies it to documents.

mod support;

use raf_okf::Location;
use raf_workplan::refs::{Banned, banned_in_line};
use support::fixtures::{ALPHA, CHANGE, OVERVIEW, mutate, mutated};
use support::{assert_clean, assert_hit, check, line_of, with_file};

/// Fixtures spell `#` as `@` so the test source does not trip its own gate.
fn hashed(fixture: &str) -> String {
    fixture.replace('@', "#")
}

#[test]
fn the_matcher_bans_references_and_process_wording() {
    let numbers = [
        hashed("see @123"),
        hashed("org/repo@12"),
        hashed("(@1234)"),
        hashed("id:@12"),
        hashed("color: @123"),
        concat!("https://example.invalid/", "issues/", "7").to_string(),
        concat!("https://example.invalid/", "pull/", "7").to_string(),
        concat!("see PR", " ", "9").to_string(),
        concat!("Closes issue", " 12").to_string(),
        concat!("see GH", "-12").to_string(),
        concat!("Pull", " Request 11").to_string(),
        concat!("CHANGE", "-", "17").to_string(),
    ];
    for line in numbers {
        assert_eq!(banned_in_line(&line), Some(Banned::IssueNumber), "{line:?}");
    }
    let process = [
        concat!("As noted in this", " issue"),
        concat!("the plan", "'s step"),
        concat!("Per the", " plan we skip it"),
        concat!("As agreed in this", " branch"),
        concat!("Task", " 4 is done"),
        concat!("Criterion AC", "3 is met"),
    ];
    for line in process {
        assert_eq!(
            banned_in_line(line),
            Some(Banned::ProcessWording),
            "{line:?}"
        );
    }
}

#[test]
fn the_matcher_spares_colours_prose_numbers_and_near_misses() {
    let clean = [
        hashed("fill @111214 and @fbbc05"),
        hashed("stroke='@abc'"),
        hashed("fill=\"@000\""),
        hashed("&@38;"),
        hashed("@123456"),
        "an issue-reference is a token".to_string(),
        "the issue tracker is external".to_string(),
        "pull request templates live in .github".to_string(),
        "there are 12 sections and 3 tests".to_string(),
        "the PRESS release".to_string(),
        "CHANGE-me".to_string(),
        "the planet".to_string(),
        "the task list".to_string(),
        concat!("reissue", " 12 copies").to_string(),
        concat!("HIGH", "-12 grade").to_string(),
    ];
    for line in clean {
        assert_eq!(banned_in_line(&line), None, "{line:?}");
    }
}

#[test]
fn a_banned_line_names_why() {
    assert_eq!(
        Banned::IssueNumber.description(),
        "issue or pull request number"
    );
    assert_eq!(
        Banned::ProcessWording.to_string(),
        "development-process wording"
    );
}

#[test]
fn rejects_issue_number_token() {
    let token = concat!("Per ", "#", "12");
    let findings = mutated(
        OVERVIEW,
        "Body text.",
        &format!("Body text. {token} said so."),
    );
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "issue-reference");
    assert_eq!(findings[0].file, OVERVIEW);
    assert_eq!(findings[0].location, Location::Line(10));
    assert!(findings[0].message.contains("issue or pull request number"));
}

#[test]
fn rejects_an_issue_url_a_pull_label_and_an_issue_label_in_documents() {
    let url = concat!("https://example.invalid/", "issues/", "7");
    let label = concat!("Closes PR", " 5");
    let issue = concat!("Fixes issue", " 3");
    for text in [url, label, issue] {
        let findings = mutated(OVERVIEW, "Body text.", &format!("Body text. {text}"));
        assert_hit(&findings, "issue-reference", OVERVIEW, &Location::Line(10));
    }
}

#[test]
fn rejects_process_wording() {
    let phrase = concat!("As agreed in ", "this", " branch");
    let findings = mutated(OVERVIEW, "Body text.", &format!("Body text. {phrase}."));
    assert_hit(&findings, "issue-reference", OVERVIEW, &Location::Line(10));
    assert!(findings[0].message.contains("development-process wording"));
    let label = concat!("Task", " 4 is done");
    let findings = with_file("README.md", &format!("# Readme\n\n{label}"));
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "issue-reference");
}

#[test]
fn rejects_acceptance_label() {
    let label = concat!("Criterion AC", "3 is met");
    let findings = mutated(OVERVIEW, "Body text.", &format!("Body text. {label}."));
    assert_hit(&findings, "issue-reference", OVERVIEW, &Location::Line(10));
}

#[test]
fn issue_token_scan_spares_hex_colours() {
    assert_eq!(banned_in_line("fill #111214 and #fbbc05 and #1f2"), None);
    assert_eq!(
        banned_in_line(concat!("see PR", "#", "9")),
        Some(Banned::IssueNumber)
    );
    assert_eq!(
        banned_in_line(concat!("CHANGE", "-", "17")),
        Some(Banned::IssueNumber)
    );
    // A document that carries a colour and prose numbers is clean.
    let findings = mutated(
        OVERVIEW,
        "Body text.",
        "Body text. Fill #111214 and #fbbc05, in 12 sections, 3 tests and 40 files.",
    );
    assert_clean(&findings);
}

#[test]
fn every_document_type_and_the_frontmatter_are_scanned() {
    let token = concat!("Per ", "#", "12");
    // A capability, a change record and the frontmatter of a document.
    for (path, from, to) in [
        (
            ALPHA,
            "Text for Purpose.",
            format!("Text for Purpose. {token}"),
        ),
        (CHANGE, "Adds both.", format!("Adds both. {token}")),
        (
            OVERVIEW,
            "description: How it fits.",
            format!("description: How it fits, {token}"),
        ),
    ] {
        let repo = mutate(path, from, &to);
        let line = line_of(&repo[path], token);
        assert_hit(
            &check(&repo),
            "issue-reference",
            path,
            &Location::Line(line),
        );
    }
}

#[test]
fn a_reference_is_reported_once_for_a_stray_page_and_a_document_that_does_not_read() {
    let token = concat!("Per ", "#", "12");
    let mut repo = support::fixtures::valid();
    repo.insert(
        "docs/architecture/stray.md".to_string(),
        format!("# Stray\n\n{token}\n"),
    );
    repo.insert(
        "docs/architecture/unclosed.md".to_string(),
        format!("---\ntype: Architecture\n\n{token}\n"),
    );
    let findings = check(&repo);
    for file in [
        "docs/architecture/stray.md",
        "docs/architecture/unclosed.md",
    ] {
        let count = findings
            .iter()
            .filter(|f| f.kind == "issue-reference" && f.file == file)
            .count();
        assert_eq!(count, 1, "{file}: {findings:#?}");
    }
}

#[test]
fn the_matcher_pins_its_edge_cases() {
    // A second hash before the digits is not a reference.
    assert_eq!(banned_in_line(&hashed("@@123")), None);
    // A quoted three- or four-digit token is a colour; five digits is not.
    assert_eq!(banned_in_line(&hashed("fill=\"@1234\"")), None);
    assert_eq!(
        banned_in_line(&hashed("fill=\"@12\"")),
        Some(Banned::IssueNumber)
    );
    assert_eq!(
        banned_in_line(&hashed("fill=\"@12345\"")),
        Some(Banned::IssueNumber)
    );
    // The pull request label needs a non-word character before it.
    assert_eq!(banned_in_line(concat!("xPR", " 5")), None);
    assert_eq!(banned_in_line(concat!("SPR", "5")), None);
    assert_eq!(
        banned_in_line(concat!("(PR", " 5)")),
        Some(Banned::IssueNumber)
    );
}
