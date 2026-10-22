// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `check` through the built binary: the exit codes 0, 1 and 2, the findings
//! it prints equal to those of the library gate, `-q`, and that it writes
//! nothing. Every case runs in both shapes of the argument vector.

mod support;

use raf_workplan::Gate;
use support::{Repo, rdf::Rdf, strict};

#[test]
fn a_clean_repository_exits_zero_and_prints_nothing() {
    let repo = Repo::synced("check-clean", false);
    let before = repo.snapshot();
    let run = repo.both(&["check"]).expect(0);
    assert_eq!(run.stdout, "", "{run:#?}");
    assert_eq!(run.stderr, "", "{run:#?}");
    assert_eq!(repo.snapshot(), before, "check wrote");
    assert_eq!(repo.gate_lines().len(), 0);
}

#[test]
fn a_repository_with_findings_exits_one_and_prints_each_finding() {
    let repo = Repo::synced("check-findings", true);
    let before = repo.snapshot();
    let run = repo.both(&["check"]).expect(1);
    assert!(
        run.stdout.contains("`demo::missing`") && run.stdout.contains("[binding-unresolved]"),
        "{run:#?}"
    );
    assert_eq!(run.stderr, "");
    assert_eq!(repo.snapshot(), before, "check wrote");
}

#[test]
fn the_findings_are_those_of_the_library_gate_in_the_same_order() {
    let repo = Repo::synced("check-equivalence", true);
    // A second kind of finding, so the order is more than one line.
    repo.edit(
        "docs/capabilities/alpha.md",
        "Text for Purpose of alpha.",
        "Text for Purpose of alpha, see [gone](missing.md).",
    );
    let expected = repo.gate_lines();
    assert!(expected.len() >= 2, "{expected:#?}");
    let run = repo.both(&["check"]).expect(1);
    assert_eq!(run.lines(), expected);
    // The same repository from a directory elsewhere, by `--root`.
    let rooted = repo.rooted(&["check"]).expect(1);
    assert_eq!(rooted.lines(), expected);
}

/// The message `Gate::assert_clean` panics with, or `None` when it does not.
fn assert_clean_message(root: &std::path::Path) -> Option<String> {
    let root = root.to_path_buf();
    let outcome = std::panic::catch_unwind(move || Gate::workspace(&root).assert_clean());
    outcome.err().map(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(ToString::to_string))
            .expect("a text panic")
    })
}

#[test]
fn the_exit_agrees_with_assert_clean_and_every_finding_is_in_its_message() {
    let clean = Repo::synced("check-assert-clean", false);
    assert_eq!(assert_clean_message(clean.root()), None);
    clean.both(&["check"]).expect(0);

    let broken = Repo::synced("check-assert-broken", true);
    broken.edit(
        "docs/capabilities/alpha.md",
        "Text for Purpose of alpha.",
        "Text for Purpose of alpha, see [gone](missing.md).",
    );
    let message = assert_clean_message(broken.root()).expect("assert_clean panics");
    let run = broken.both(&["check"]).expect(1);
    let findings = Gate::workspace(broken.root()).check();
    assert_eq!(run.lines().len(), findings.len());
    for finding in &findings {
        assert!(message.contains(&finding.message), "{message}");
        assert!(message.contains(finding.kind), "{message}");
        assert!(run.stdout.contains(&finding.message));
    }
    assert!(
        message.contains(&format!("{} finding(s)", findings.len())),
        "{message}"
    );
}

#[test]
fn a_finding_is_gone_once_it_is_fixed() {
    let repo = Repo::synced("check-fixed", true);
    repo.both(&["check"]).expect(1);
    repo.edit(
        "docs/capabilities/beta.md",
        "[demo::real, demo::missing]",
        "[demo::real]",
    );
    // The catalogue lists the gaps of an implementing capability.
    repo.sync();
    let run = repo.both(&["check"]).expect(0);
    assert_eq!(run.stdout, "");
}

#[test]
fn a_stale_generated_block_is_a_finding_and_a_sync_clears_it() {
    let repo = Repo::synced("check-stale", false);
    repo.edit(
        "docs/capabilities/alpha.md",
        "description: A description of alpha.",
        "description: A description of alpha, reworded.",
    );
    let run = repo.both(&["check"]).expect(1);
    assert_eq!(run.lines(), repo.gate_lines());
    assert!(run.stdout.contains("[generated-stale]"), "{run:#?}");
    repo.sync();
    repo.both(&["check"]).expect(0);
}

#[test]
fn quiet_keeps_the_findings_and_the_exit_code() {
    let repo = Repo::synced("check-quiet", true);
    let loud = repo.both(&["check"]).expect(1);
    let quiet = repo.both(&["-q", "check"]).expect(1);
    assert_eq!(quiet.stdout, loud.stdout);
    let long = repo.both(&["--quiet", "check"]).expect(1);
    assert_eq!(long.stdout, loud.stdout);
    let after = repo.both(&["check", "-q"]).expect(1);
    assert_eq!(after.stdout, loud.stdout);
}

#[test]
fn quiet_on_a_clean_repository_prints_nothing_and_exits_zero() {
    let repo = Repo::synced("check-quiet-clean", false);
    let run = repo.both(&["-q", "check"]).expect(0);
    assert_eq!((run.stdout.as_str(), run.stderr.as_str()), ("", ""));
}

#[test]
fn a_directory_that_is_not_a_repository_exits_two_and_says_why_on_standard_error() {
    let repo = Repo::bare("check-empty");
    let run = repo.rooted(&["check"]).expect(2);
    assert_eq!(run.stdout, "", "{run:#?}");
    assert!(
        run.stderr.starts_with("cargo-workplan: ") && run.stderr.contains("docs/capabilities"),
        "{run:#?}"
    );
    // The neighbour: the same directory with a capability document checks.
    repo.put(
        "docs/capabilities/alpha.md",
        &support::capability(
            "alpha",
            "delivered",
            [
                "[demo::real]",
                "[demo::test_crate::it::case]",
                "[demo::real]",
            ],
            "[]",
            &[("Verification", support::BLOCKS)],
        ),
    );
    repo.sync();
    repo.rooted(&["check"]).expect(0);
}

#[test]
fn a_root_that_does_not_exist_exits_two() {
    let repo = Repo::bare("check-no-root");
    let missing = repo.root().join("no-such-directory");
    let missing = missing.to_string_lossy();
    let run = repo.both(&["--root", &missing, "check"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("no-such-directory"), "{run:#?}");
}

#[test]
fn a_manifest_that_does_not_parse_exits_two() {
    let repo = Repo::synced("check-bad-manifest", false);
    repo.both(&["check"]).expect(0);
    repo.put("Cargo.toml", "[package\nname = \n");
    let run = repo.rooted(&["check"]).expect(2);
    assert_eq!(run.stdout, "", "{run:#?}");
    assert!(run.stderr.starts_with("cargo-workplan: "), "{run:#?}");
}

/// A file the process may not read is a run that could not look at the
/// repository. A process with the privilege to read any file cannot make one
/// unreadable, and there the premise of the case does not hold.
#[cfg(unix)]
#[test]
fn a_source_file_that_cannot_be_read_exits_two() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt as _;

    let repo = Repo::synced("check-unreadable", false);
    repo.both(&["check"]).expect(0);
    let lib = repo.root().join("src/lib.rs");
    fs::set_permissions(&lib, fs::Permissions::from_mode(0o000)).expect("chmod");
    if fs::read(&lib).is_ok() {
        return;
    }
    let run = repo.rooted(&["check"]);
    fs::set_permissions(&lib, fs::Permissions::from_mode(0o644)).expect("restore");
    let run = run.expect(2);
    assert_eq!(run.stdout, "", "{run:#?}");
    assert!(run.stderr.starts_with("cargo-workplan: "), "{run:#?}");
    // The neighbour: readable again, the same repository checks clean.
    repo.rooted(&["check"]).expect(0);
}

#[test]
fn usage_errors_exit_two_with_a_message_on_standard_error() {
    let repo = Repo::synced("check-usage", false);
    for args in [
        &["frobnicate"][..],
        &["check", "--no-such-option"],
        &["--format", "yaml", "check"],
        &["check", "extra"],
        &[],
        &["docs"],
    ] {
        let run = repo.both(args).expect(2);
        assert_eq!(run.stdout, "", "{args:?}");
        assert!(!run.stderr.is_empty(), "{args:?}");
    }
}

#[test]
fn help_and_version_exit_zero_on_standard_output() {
    let repo = Repo::synced("check-help", false);
    let help = repo.both(&["--help"]).expect(0);
    for word in ["check", "status", "docs", "--root", "--format", "-q"] {
        assert!(help.stdout.contains(word), "{word}: {help:#?}");
    }
    let version = repo.both(&["--version"]).expect(0);
    assert!(
        version.stdout.contains(env!("CARGO_PKG_VERSION")),
        "{version:#?}"
    );
}

#[test]
fn a_raf_comment_split_over_lines_fails_check_and_other_split_comments_pass() {
    let purpose = "Text for Purpose of alpha.";
    for (comment, kind) in [
        (
            "<!--\nraf:claim evidence=x\n-->",
            "okf-parse-malformed-claim",
        ),
        (
            "<!--\n\n  RAF:Claim evidence=x\n-->",
            "okf-parse-malformed-claim",
        ),
        ("<!--\nraf:foo\n-->", "okf-parse-unknown-marker"),
    ] {
        let repo = Repo::synced("check-split-marker", false);
        repo.edit(
            "docs/capabilities/alpha.md",
            purpose,
            &format!("{purpose}\n\n{comment}"),
        );
        let before = repo.snapshot();
        let run = repo.both(&["check"]).expect(1);
        assert!(
            run.stdout.contains(&format!("[{kind}]"))
                && run.stdout.contains("docs/capabilities/alpha.md"),
            "{comment:?}: {run:#?}"
        );
        assert_eq!(run.lines(), repo.gate_lines(), "{comment:?}");
        assert_eq!(repo.snapshot(), before, "check wrote");
    }
    // The neighbours: a split `raf:generated:` comment, an SPDX header split
    // over lines and an ordinary split comment are plain comments.
    for comment in [
        "<!--\nraf:generated:x\n-->",
        "<!--\nSPDX-FileCopyrightText: 2026 Example Org <dev@example.org>\nSPDX-License-Identifier: CC-BY-4.0\n-->",
        "<!--\nA note for the reader.\n-->",
    ] {
        let repo = Repo::synced("check-split-plain", false);
        repo.edit(
            "docs/capabilities/alpha.md",
            purpose,
            &format!("{purpose}\n\n{comment}"),
        );
        let run = repo.both(&["check"]).expect(0);
        assert_eq!(run.stdout, "", "{comment:?}: {run:#?}");
    }
}

#[test]
fn the_default_check_accepts_the_complete_fixture() {
    let repo = Repo::strict("check-strict-clean", &strict::capability(), "");
    let before = repo.snapshot();
    let run = repo.both(&["check"]).expect(0);
    assert_eq!((run.stdout.as_str(), run.stderr.as_str()), ("", ""));
    assert_eq!(repo.snapshot(), before, "check wrote");
}

#[test]
fn the_default_check_refuses_a_thin_section_without_a_policy_flag() {
    let repo = Repo::strict("check-strict-thin", &strict::thin("Examples"), "");
    let run = repo.both(&["check"]).expect(1);
    let expected: Vec<String> = Gate::workspace(repo.root())
        .check()
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(run.lines(), expected);
    assert_eq!(run.lines().len(), 1, "{run:#?}");
    assert!(
        run.stdout
            .starts_with("docs/capabilities/demo.md: section `Examples`: ")
            && run.stdout.contains("[strict-section-thin]"),
        "{run:#?}"
    );
    assert_eq!(run.stderr, "");
    let rooted = repo.rooted(&["check"]).expect(1);
    assert_eq!(rooted.stdout, run.stdout);
    let repeated = repo.both(&["check"]).expect(1);
    assert_eq!(repeated, run);
}

#[test]
fn the_default_check_without_capability_documents_is_a_run_failure() {
    let repo = Repo::bare("check-strict-empty");
    let run = repo.rooted(&["check"]).expect(2);
    assert_eq!(run.stdout, "", "{run:#?}");
    assert!(
        run.stderr.starts_with("cargo-workplan: ") && run.stderr.contains("docs/capabilities"),
        "{run:#?}"
    );
}

#[test]
fn a_removed_policy_key_is_refused_beside_the_default_content_refusal() {
    let thin = strict::thin("Examples");
    let recorded = Repo::strict("check-strict-recorded", &thin, "");
    let run = recorded.both(&["check"]).expect(1);
    assert!(run.stdout.contains("[strict-section-thin]"), "{run:#?}");
    assert_eq!(run.lines().len(), 1, "{run:#?}");
    // The flag adds nothing to a record that says strict.
    assert_eq!(recorded.both(&["check"]).expect(1), run);
    let default = Repo::strict("check-strict-default", &thin, "strictness: default\n");
    let run = default.both(&["check"]).expect(1);
    assert!(run.stdout.contains("[okf-unknown-key]"), "{run:#?}");
}

#[test]
fn policy_findings_keep_typed_locations_and_messages_in_rdf() {
    let repo = Repo::strict("check-strict-json", &strict::thin("Examples"), "");
    let run = repo.both(&["--format", "turtle", "check"]).expect(1);
    let rdf = Rdf::of(&run);
    let rows = rdf.rows("Finding");
    assert_eq!(rows.len(), 1);
    let actual = Gate::workspace(repo.root()).check();
    let finding = &actual[0];
    assert_eq!(rows[0].required("kind"), "strict-section-thin");
    assert_eq!(rows[0].required("file"), "docs/capabilities/demo.md");
    assert_eq!(rows[0].required("section"), "Examples");
    assert!(rows[0].text("line").is_none());
    assert!(rows[0].text("key").is_none());
    assert_eq!(rows[0].required("message"), finding.message);
    assert_eq!(rows[0].required("fix"), finding.fix);
    let text = repo.both(&["check"]).expect(1);
    assert_eq!(text.lines().len(), rows.len());
}
