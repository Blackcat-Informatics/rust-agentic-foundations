// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The rules between documents: references that name a document that does not
//! exist, `depends` cycles, and the identity rules the profile enforces. Each
//! rejected fixture has its valid neighbour.

mod support;

use raf_workplan::Finding;
use support::cases::reference_cases;
use support::fixtures::{ALPHA, BETA, CHANGE, GAMMA, change_record, mutate, valid};
use support::{assert_clean, assert_hit, check, key};

fn kinds(findings: &[Finding], kind: &str) -> Vec<Finding> {
    findings
        .iter()
        .filter(|f| f.kind == kind)
        .cloned()
        .collect()
}

#[test]
fn every_reference_case_is_reported_exactly() {
    assert_clean(&check(&valid()));
    for case in reference_cases() {
        case.assert_exact();
    }
}

#[test]
fn accepts_a_depends_on_an_existing_capability_and_an_acyclic_graph() {
    // The valid repository holds a diamond: gamma depends on beta and alpha,
    // alpha on beta.
    assert_clean(&check(&valid()));
}

#[test]
fn rejects_depends_missing() {
    let findings = check(&mutate(
        ALPHA,
        "depends: [cap:beta]",
        "depends: [cap:ghost]",
    ));
    assert_hit(&findings, "depends-missing", ALPHA, &key("depends"));
    assert!(findings[0].message.contains("cap:ghost"));
}

#[test]
fn rejects_changes_missing() {
    let mut repo = valid();
    repo.insert(
        "work/changes/add-ghost.md".to_string(),
        change_record(
            "add-ghost",
            "[cap:ghost]",
            "  - cap:ghost#contract\n",
            "  - claim: \"x\"\n",
        ),
    );
    let findings = check(&repo);
    assert_hit(
        &findings,
        "changes-missing",
        "work/changes/add-ghost.md",
        &key("changes"),
    );
}

#[test]
fn accepts_a_moved_outcome_that_names_an_existing_change() {
    let repo = mutate(
        CHANGE,
        "outcome: implemented",
        "outcome: moved:change:add-alpha",
    );
    assert_clean(&check(&repo));
}

#[test]
fn accepts_moved_outcome() {
    // The item moved to a second record, which exists and lists the capability.
    let mut repo = mutate(
        CHANGE,
        "outcome: implemented",
        "outcome: moved:change:other-work",
    );
    repo.insert(
        "work/changes/other-work.md".to_string(),
        change_record("other-work", "[cap:alpha]", "  - cap:alpha#usage\n", ""),
    );
    assert_clean(&check(&repo));
}

#[test]
fn rejects_moved_missing() {
    let findings = check(&mutate(
        CHANGE,
        "outcome: implemented",
        "outcome: moved:change:nowhere",
    ));
    assert_hit(&findings, "moved-missing", CHANGE, &key("items"));
    assert!(findings[0].message.contains("change:nowhere"));
}

#[test]
fn a_reference_to_a_broken_document_is_not_a_missing_one() {
    // alpha is present but breaks the profile; gamma and the change still
    // name it, and only alpha's own findings speak.
    let findings = check(&mutate(ALPHA, "state: designed", "state: draft"));
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "okf-bad-state");
}

#[test]
fn a_reference_to_a_file_without_frontmatter_is_missing() {
    let mut repo = valid();
    repo.insert(
        "docs/capabilities/plain.md".to_string(),
        "# Plain\n".to_string(),
    );
    let text = repo[ALPHA].replace("depends: [cap:beta]", "depends: [cap:plain]");
    repo.insert(ALPHA.to_string(), text);
    let findings = check(&repo);
    assert_hit(&findings, "depends-missing", ALPHA, &key("depends"));
    assert!(findings.iter().any(|f| f.kind == "stray-markdown"));
}

#[test]
fn rejects_depends_cycles_once_for_each_group_naming_its_members() {
    for (name, edits, members) in [
        (
            "self",
            vec![(
                ALPHA,
                "depends: [cap:beta]",
                "depends: [cap:alpha, cap:beta]",
            )],
            vec!["cap:alpha"],
        ),
        (
            "two",
            vec![(BETA, "depends: []", "depends: [cap:alpha]")],
            vec!["cap:alpha", "cap:beta"],
        ),
        (
            "three",
            vec![
                (BETA, "depends: []", "depends: [cap:gamma]"),
                (
                    GAMMA,
                    "depends: [cap:beta, cap:alpha]",
                    "depends: [cap:alpha]",
                ),
            ],
            vec!["cap:alpha", "cap:beta", "cap:gamma"],
        ),
    ] {
        let mut repo = valid();
        for (path, from, to) in edits {
            let text = repo[path].replacen(from, to, 1);
            assert_ne!(text, repo[path], "{name}");
            repo.insert(path.to_string(), text);
        }
        let found = kinds(&check(&repo), "depends-cycle");
        assert_eq!(found.len(), 1, "{name}: {found:#?}");
        assert_eq!(found[0].file, ALPHA, "{name}");
        for member in &members {
            assert!(
                found[0].message.contains(&format!("`{member}`")),
                "{name}: {}",
                found[0].message
            );
        }
        assert_eq!(
            found[0].message.matches("`cap:").count(),
            members.len(),
            "{name}: {}",
            found[0].message
        );
    }
}

#[test]
fn two_separate_cycles_are_two_findings_and_a_cycle_with_a_tail_is_one() {
    let mut repo = valid();
    // alpha and beta depend on each other; gamma depends on alpha and itself.
    let edits = [
        (BETA, "depends: []", "depends: [cap:alpha]"),
        (
            GAMMA,
            "depends: [cap:beta, cap:alpha]",
            "depends: [cap:gamma, cap:alpha]",
        ),
    ];
    for (path, from, to) in edits {
        let text = repo[path].replacen(from, to, 1);
        repo.insert(path.to_string(), text);
    }
    let found = kinds(&check(&repo), "depends-cycle");
    assert_eq!(found.len(), 2, "{found:#?}");
    assert_eq!(found[0].file, ALPHA);
    assert_eq!(found[1].file, GAMMA);
    assert!(found[1].message.contains("itself"));
}

// A slug is lowercase kebab case and equals the file name.
#[test]
fn a_slug_id_that_differs_from_the_file_name_is_refused() {
    for (path, from, to) in [
        (ALPHA, "id: cap:alpha", "id: cap:other"),
        (CHANGE, "id: change:add-alpha", "id: change:add-other"),
        (
            support::fixtures::OVERVIEW,
            "id: arch:overview",
            "id: arch:other",
        ),
    ] {
        let findings = check(&mutate(path, from, to));
        assert_hit(&findings, "okf-id-file-name", path, &key("id"));
    }
}

#[test]
fn a_slug_that_is_not_kebab_case_is_refused() {
    for (path, from, to) in [
        (ALPHA, "id: cap:alpha", "id: cap:Alpha"),
        (ALPHA, "id: cap:alpha", "id: cap:al_pha"),
        (CHANGE, "id: change:add-alpha", "id: change:Add-alpha"),
        (
            support::fixtures::OVERVIEW,
            "id: arch:overview",
            "id: arch:over--view",
        ),
    ] {
        let findings = check(&mutate(path, from, to));
        assert_hit(&findings, "okf-invalid-id", path, &key("id"));
    }
}

#[test]
fn a_matching_kebab_slug_is_accepted() {
    assert_clean(&check(&valid()));
}
