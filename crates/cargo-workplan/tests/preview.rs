// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `docs preview` through the built binary: the markers are gone and the
//! proposed text is there, an unknown slug is exit 2, and the tree is not
//! touched.

mod support;

use std::fmt::Write as _;

use support::Repo;

const PROPOSED: &str = "Delta reads streams as they arrive.";

#[test]
fn the_proposed_text_is_printed_and_the_markers_are_not() {
    let repo = Repo::synced("preview-text", false);
    let before = repo.snapshot();
    assert!(
        repo.read("docs/capabilities/delta.md")
            .contains("[!PROPOSED change:grow]")
    );
    let run = repo.both(&["docs", "preview", "grow"]).expect(0);
    assert!(run.stdout.contains(PROPOSED), "{run:#?}");
    assert!(
        run.stdout.contains("It never buffers a whole document."),
        "{run:#?}"
    );
    assert!(!run.stdout.contains("PROPOSED"), "{run:#?}");
    assert!(
        !run.stdout.contains("> Delta"),
        "the quote marker stays: {run:#?}"
    );
    assert_eq!(run.stderr, "");
    assert_eq!(repo.snapshot(), before, "preview wrote");
    // The document on disk still holds the block: the preview never writes.
    assert!(
        repo.read("docs/capabilities/delta.md")
            .contains("[!PROPOSED change:grow]")
    );
}

#[test]
fn the_output_is_the_text_the_library_previews_under_a_header_for_each_document() {
    let repo = Repo::synced("preview-equivalence", false);
    let opened = raf_workplan::Gate::workspace(repo.root())
        .programs(raf_workplan::exec::Programs::new(repo.root()).discover_git())
        .open()
        .expect("a repository");
    let plan = opened.workplan();
    let previews = plan.preview("grow").expect("a preview");
    let mut expected = String::new();
    for preview in &previews {
        write!(expected, "==> {} <==\n{}", preview.path, preview.text).expect("a string");
    }
    let run = repo.both(&["docs", "preview", "grow"]).expect(0);
    assert_eq!(run.stdout, expected);
    let paths: Vec<&str> = previews.iter().map(|p| p.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "docs/capabilities/beta.md",
            "docs/capabilities/delta.md",
            "docs/capabilities/gamma.md"
        ]
    );
}

#[test]
fn an_unknown_slug_is_exit_two_and_names_the_open_changes() {
    let repo = Repo::synced("preview-unknown", false);
    let run = repo.both(&["docs", "preview", "shrink"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.contains("no change record `shrink`") && run.stderr.contains("grow"),
        "{run:#?}"
    );
    // The neighbour: a slug with a record previews.
    repo.both(&["docs", "preview", "grow"]).expect(0);
}

#[test]
fn quiet_prints_nothing_because_a_preview_is_not_a_finding() {
    let repo = Repo::synced("preview-quiet", false);
    let run = repo.both(&["-q", "docs", "preview", "grow"]).expect(0);
    assert_eq!((run.stdout.as_str(), run.stderr.as_str()), ("", ""));
    // A failure is not hidden by `-q`.
    let failed = repo.both(&["-q", "docs", "preview", "shrink"]).expect(2);
    assert!(failed.stderr.contains("no change record"), "{failed:#?}");
}

#[test]
fn a_change_record_that_did_not_load_is_exit_two_and_says_so() {
    let repo = Repo::synced("preview-not-loaded", false);
    repo.put(
        "work/changes/half.md",
        &support::change("half", "[cap:beta]", &["cap:beta#contract"])
            .replace("contract_effect: changes", "contract_effect: sometimes"),
    );
    let run = repo.both(&["docs", "preview", "half"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("did not load"), "{run:#?}");
    assert!(run.stderr.contains("[okf-bad-contract-effect]"), "{run:#?}");
}

#[test]
fn a_capability_the_change_lists_that_has_no_document_is_exit_two() {
    let repo = Repo::synced("preview-missing-capability", false);
    repo.edit(
        "work/changes/grow.md",
        "changes: [cap:beta, cap:gamma, cap:delta]",
        "changes: [cap:beta, cap:gamma, cap:delta, cap:ghost]",
    );
    let run = repo.both(&["docs", "preview", "grow"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("cap:ghost"), "{run:#?}");
}

#[test]
fn the_preview_is_the_same_by_root_from_another_directory() {
    let repo = Repo::synced("preview-rooted", false);
    let here = repo.both(&["docs", "preview", "grow"]).expect(0);
    let rooted = repo.rooted(&["docs", "preview", "grow"]).expect(0);
    assert_eq!(here.stdout, rooted.stdout);
}

#[test]
fn a_slug_that_is_not_a_slug_is_a_usage_error() {
    let repo = Repo::synced("preview-bad-slug", false);
    for slug in ["Grow", "../grow", "change:grow", "grow.md", "gr ow", ""] {
        let run = repo.both(&["docs", "preview", slug]).expect(2);
        assert_eq!(run.stdout, "", "{slug:?}");
        assert!(run.stderr.contains("slug"), "{slug:?}: {run:#?}");
    }
}
