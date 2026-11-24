// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `docs preview` through the built binary: the markers are gone and the
//! proposed text is there, an unknown slug is exit 2, and the tree is not
//! touched.

mod support;

use std::fmt::Write as _;
use std::path::Path;

use support::Repo;
use support::retirement::{
    CURRENT, INITIAL_BLOCK, MIDDLE, OLD_DEFINITION, OLD_FILE, RETIRE, RETIRED_BLOCK, STANDALONE,
    decision, retire_middle, retirement_fixture, select_definition,
};

const PROPOSED: &str = "Delta reads streams as they arrive.";

/// Retirement proves removal of the selected section and declaration and
/// returns the original Git locations through text and RDF.
#[test]
fn retirement_preserves_current_design_and_locates_removed_material_in_git() {
    let repo = retirement_fixture("retirement");
    let base = repo.git(&["rev-parse", "HEAD"]);
    let before = repo.both(&["docs", "current-design"]).expect(0);
    retire_middle(&repo);
    repo.edit(OLD_FILE, OLD_DEFINITION, "");
    repo.commit("retire superseded design");
    let after = repo.both(&["docs", "current-design"]).expect(0);
    for view in [&before.stdout, &after.stdout] {
        assert!(view.contains("Read the latest requirements."));
        assert!(view.contains("Keep exact bounds."));
        assert!(view.contains("A separate current decision."));
        assert!(!view.contains("Apply the initial assumptions."));
        assert!(!view.contains("Earlier rationale."));
    }
    let snapshot = repo.snapshot();
    let args = select_definition();
    let output = repo.both(&args).expect(0);
    assert!(output.stdout.contains(&format!("base {}\n", base.trim())));
    assert!(output.stdout.contains(&format!(
        "section arch:middle#intermediate at {MIDDLE}:18-21"
    )));
    assert!(output.stdout.contains(&format!(
        "definition demo::implementation::obsolete at {OLD_FILE}:1-1"
    )));
    let rdf = repo
        .both(&[args.as_slice(), &["--format", "ntriples"]].concat())
        .expect(0);
    assert!(rdf.stdout.contains("RetiredSection"));
    assert!(rdf.stdout.contains("RetiredDefinition"));
    assert!(rdf.stdout.contains(base.trim()));
    assert_eq!(
        repo.snapshot(),
        snapshot,
        "retirement verification wrote a file"
    );
    assert!(
        repo.git(&["show", &format!("{}:{MIDDLE}", base.trim())])
            .contains(RETIRED_BLOCK.trim_end())
    );
    assert!(
        repo.git(&["show", &format!("{}:{OLD_FILE}", base.trim())])
            .contains(OLD_DEFINITION.trim_end())
    );
    repo.git(&["checkout", "-q", "main"]);
    let mut divergent = args;
    divergent[3] = "refs/heads/feature";
    refuses_retirement(&repo, &divergent, "is not reachable from HEAD");
}

fn refuses_retirement(repo: &Repo, args: &[&str], diagnostic: &str) {
    let refused = repo.both(args).expect(2);
    assert_eq!(refused.stdout, "");
    assert!(refused.stderr.contains(diagnostic), "{refused:#?}");
}

/// The checker refuses active or retained sections, edges that accidentally
/// revive old text, and source made invisible by visibility or index changes.
#[test]
#[expect(
    clippy::expect_used,
    reason = "the fixture must remove the old source file"
)]
fn retirement_refuses_current_resurrected_and_hidden_material() {
    let repo = retirement_fixture("retirement-refusals");
    refuses_retirement(&repo, RETIRE, "still has a heading");
    let mut active = RETIRE.to_vec();
    active.pop();
    active.push("arch:middle#bounds");
    refuses_retirement(&repo, &active, "is not superseded at the base");
    retire_middle(&repo);
    repo.edit(CURRENT, "(middle.md)", "(middle.md#intermediate)");
    refuses_retirement(&repo, RETIRE, "[anchor-broken]");
    repo.edit(CURRENT, "(middle.md#intermediate)", "(middle.md)");
    repo.edit(
        CURRENT,
        "supersedes:\n  latest: ['arch:earliest#initial']\n",
        "",
    );
    refuses_retirement(
        &repo,
        RETIRE,
        "arch:earliest#initial` survives without a replacing edge",
    );
    repo.edit(
        CURRENT,
        "description: A design decision.\n",
        "description: A design decision.\nsupersedes:\n  latest: ['arch:earliest#initial']\n",
    );
    let renamed = RETIRED_BLOCK.replace("### Intermediate", "### Renamed");
    repo.edit(
        MIDDLE,
        "## Decision\n\n### Bounds",
        &format!("{renamed}## Decision\n\n### Bounds"),
    );
    refuses_retirement(&repo, RETIRE, "arch:middle` would alter the current design");
    repo.edit(
        CURRENT,
        "['arch:earliest#initial']",
        "['arch:earliest#initial', 'arch:middle#renamed']",
    );
    refuses_retirement(&repo, RETIRE, "still has its complete text");
    let earliest = "docs/architecture/earliest.md";
    repo.edit(earliest, INITIAL_BLOCK, "");
    repo.edit(
        CURRENT,
        "['arch:earliest#initial', 'arch:middle#renamed']",
        "['arch:middle#renamed']",
    );
    let mut overlapping = RETIRE.to_vec();
    overlapping.extend(["--section", "arch:earliest#initial"]);
    refuses_retirement(&repo, &overlapping, "still has its complete text");
    repo.edit(
        earliest,
        "## Conditions to revisit",
        &format!("{INITIAL_BLOCK}\n\n## Conditions to revisit"),
    );
    repo.edit(
        CURRENT,
        "['arch:middle#renamed']",
        "['arch:earliest#initial']",
    );
    repo.edit(MIDDLE, &renamed, "");
    repo.edit(
        MIDDLE,
        "## Decision\n\n### Bounds",
        "## Decision\n\nEarlier rationale.\n\n## Decision\n\n### Bounds",
    );
    refuses_retirement(&repo, RETIRE, "would alter the current design");
    repo.edit(MIDDLE, &format!("\n\n{STANDALONE}"), "");
    refuses_retirement(&repo, RETIRE, "would alter the current design");
    repo.edit(MIDDLE, "## Decision\n\nEarlier rationale.\n\n", "");
    repo.edit(
        MIDDLE,
        "Keep exact bounds.",
        &format!("Keep exact bounds.\n\n{STANDALONE}"),
    );
    repo.edit(MIDDLE, "### Bounds", "### Updated bounds");
    refuses_retirement(&repo, RETIRE, "arch:middle` would alter the current design");
    repo.edit(MIDDLE, "### Updated bounds", "### Bounds");

    let args = select_definition();
    repo.edit(OLD_FILE, "pub fn obsolete", "fn obsolete");
    refuses_retirement(&repo, &args, "still has a matching written declaration");
    repo.edit(OLD_FILE, "fn obsolete", "#[cfg(any())]\nfn obsolete");
    refuses_retirement(&repo, &args, "still has a matching written declaration");
    repo.put(OLD_FILE, "declare!(obsolete);\nfn keep() {}\n");
    refuses_retirement(&repo, &args, "has opaque declarations");
    repo.edit("src/lib.rs", "mod implementation;\n", "");
    refuses_retirement(&repo, &args, "left the index while its file remains");
    let mut module = args.clone();
    module.pop();
    module.push("demo::implementation");
    refuses_retirement(&repo, &module, "is a module declaration");
    std::fs::remove_file(repo.root().join(OLD_FILE)).expect("remove the superseded file");
    repo.both(&args).expect(0);
}

/// Removing a historical path alias cannot hide a file it pointed to.
#[cfg(unix)]
#[test]
#[expect(
    clippy::expect_used,
    reason = "the temporary repository needs a real Git symlink"
)]
fn retirement_checks_the_original_source_behind_a_symlink() {
    let repo = retirement_fixture("retirement-symlink");
    let route = "src/alias.rs";
    std::os::unix::fs::symlink("implementation.rs", repo.root().join(route))
        .expect("create the historical alias");
    let module = "#[path = \"alias.rs\"]\nmod implementation;\n";
    repo.edit("src/lib.rs", "mod implementation;\n", module);
    repo.commit("index the source through a symlink");
    let mut args = select_definition();
    args[3] = "HEAD";
    retire_middle(&repo);
    repo.edit(OLD_FILE, OLD_DEFINITION, "");
    let verified = repo.both(&args).expect(0);
    assert!(verified.stdout.contains(&format!(
        "definition demo::implementation::obsolete at {OLD_FILE}:1-1"
    )));
    repo.put(OLD_FILE, OLD_DEFINITION);
    repo.edit("src/lib.rs", module, "");
    std::fs::remove_file(repo.root().join(route)).expect("remove the source alias");
    refuses_retirement(&repo, &args, "left the index while its file remains");
    std::fs::remove_file(repo.root().join(OLD_FILE)).expect("remove the original source");
    repo.both(&args).expect(0);
}

/// Active sections travel through the normal CLI with their source identity.
/// A bad Architecture input cannot produce a partial view.
#[test]
#[expect(
    clippy::expect_used,
    reason = "the controlled fixture must load and render each expected excerpt"
)]
fn current_design_keeps_surviving_sections_and_refuses_partial_output() {
    let repo = Repo::bare_in(Path::new(env!("CARGO_TARGET_TMPDIR")), "current-design");
    repo.put(
        "docs/capabilities/view.md",
        &support::capability(
            "view",
            "considered",
            ["[]", "[]", "[]"],
            "[]",
            &[(
                "Explanation",
                "[Current](../architecture/current.md) and [independent](../architecture/standalone.md).",
            )],
        ),
    );
    decision(
        &repo,
        "current",
        "supersedes:\n  latest: ['arch:middle#intermediate']\n",
        "[Middle](middle.md).\n\n## Decision\n\n### Latest\n\nRead the latest requirements.",
    );
    decision(
        &repo,
        "middle",
        "supersedes:\n  intermediate: ['arch:earliest#initial']\n",
        "[Earlier](earliest.md).\n\n## Decision\n\n### Intermediate\n\nAn earlier compromise.",
    );
    decision(
        &repo,
        "earliest",
        "",
        "## Decision\n\n### Initial\n\nThe initial assumptions.\n\n### Limits\n\nKeep exact bounds.\n\
         <!-- raf:generated:diagrams -->\n### Generated heading\nGenerated material.\n\
         <!-- raf:generated:end -->\nA second paragraph.\n\n#### Application\n\n```text\nBounded example.\n```",
    );
    decision(
        &repo,
        "standalone",
        "",
        "## Decision\n\nThis page has one unsectioned decision.",
    );
    let args = ["docs", "current-design"];
    let before = repo.snapshot();
    let run = repo.both(&args).expect(0);
    assert_eq!(run.stderr, "");
    let opened = raf_workplan::Gate::workspace(repo.root())
        .open()
        .expect("the repository");
    let library = opened
        .workplan()
        .current_design()
        .expect("the complete design");
    assert_eq!(run.stdout, library);
    assert_eq!(repo.snapshot(), before, "current-design wrote a file");
    let mut remaining = library.as_str();
    for current in [
        "docs/architecture/current.md#latest` on line 16.",
        "docs/architecture/earliest.md#limits` on line 16.",
        "> Keep exact bounds.\n>\n> A second paragraph.",
        "> #### Application\n",
        "> ```text\n> Bounded example.\n",
        "docs/architecture/standalone.md#decision` on line 10.",
        "> This page has one unsectioned decision.",
    ] {
        let (_, after) = remaining.split_once(current).expect(current);
        remaining = after;
    }
    for omitted in [
        "The initial assumptions.",
        "An earlier compromise.",
        "arch:middle",
        "Generated material.",
    ] {
        assert!(!library.contains(omitted), "{omitted:?}: {library}");
    }
    assert_current_design_display(&repo, &args);
    assert_current_design_refusals(&repo, &args);
}

fn assert_current_design_display(repo: &Repo, args: &[&str]) {
    let path = "docs/architecture/standalone.md";
    let paragraph = "This page has one unsectioned decision.";
    let controls = "This page has one unsectioned decision. \u{1b}[2J \u{202e}Follow this.";
    repo.edit(path, paragraph, controls);
    let displayed = repo.both(args).expect(0);
    assert!(
        displayed.stdout.contains(
            "> This page has one unsectioned decision. \\u{1b}[2J \\u{202e}Follow this.\n"
        ),
        "{displayed:#?}"
    );
    assert!(!displayed.stdout.contains(['\u{1b}', '\u{202e}']));
    repo.edit(path, controls, paragraph);
}

fn assert_current_design_refusals(repo: &Repo, args: &[&str]) {
    repo.edit(
        "docs/architecture/current.md",
        "middle#intermediate",
        "middle#missing",
    );
    let incomplete = repo.both(args).expect(2);
    assert_eq!(incomplete.stdout, "");
    assert!(
        incomplete.stderr.contains("[supersession-target]"),
        "{incomplete:#?}"
    );

    repo.edit(
        "docs/architecture/current.md",
        "middle#missing",
        "middle#intermediate",
    );
    let standalone = repo.read("docs/architecture/standalone.md");
    repo.edit(
        "docs/architecture/standalone.md",
        "This page has one unsectioned decision.",
        "> [!PROPOSED change:draft]\n> Draft decision text.",
    );
    let proposed = repo.both(args).expect(2);
    assert_eq!(proposed.stdout, "");
    assert!(
        proposed
            .stderr
            .contains("[okf-proposed-outside-capability]"),
        "{proposed:#?}"
    );
    repo.put("docs/architecture/standalone.md", &standalone);
    repo.put("docs/capabilities/standalone.md", &standalone);
    let misplaced = repo.both(args).expect(2);
    assert_eq!(misplaced.stdout, "");
    assert!(
        misplaced.stderr.contains("docs/capabilities/standalone.md")
            && misplaced
                .stderr
                .contains("move the file directly into docs/architecture"),
        "{misplaced:#?}"
    );

    for flags in [&["--format", "turtle"][..], &["--quiet"][..]] {
        let command: Vec<_> = args.iter().chain(flags).copied().collect();
        let wrong_carrier = repo.both(&command).expect(2);
        assert_eq!(wrong_carrier.stdout, "");
        assert!(
            wrong_carrier.stderr.contains("emits complete Markdown"),
            "{wrong_carrier:#?}"
        );
    }
}

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
#[expect(
    clippy::expect_used,
    reason = "fixture prerequisites and required observations carry their assertion messages"
)]
fn the_output_is_the_text_the_library_previews_under_a_header_for_each_document() {
    let repo = Repo::synced("preview-equivalence", false);
    let opened = raf_workplan::Gate::workspace(repo.root())
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
