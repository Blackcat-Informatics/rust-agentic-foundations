// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `change reconcile` through the built binary: one line for each obligation
//! in the shape of the contract, the exit 1 of a list that is not empty and the
//! exit 0 of one that is, the exit 2 of everything that stops it looking, `-q`,
//! typed RDF results, and the tool-owned no-write boundary for these fixtures.

mod support;
use support::rdf::Rdf;

use std::path::Path;

use raf_workplan::exec::Programs;
use raf_workplan::lifecycle::Lifecycle;
use support::{Repo, Run, change};

/// The list `change reconcile grow` prints for the roadmap repository on a
/// branch that has not edited a document.
const GROW: &str = "\
cap:beta  contract  section-untouched  a diff of that section against the merge base, or an edit that records why none is needed
cap:beta  verification  binding-missing  a binding that resolves through the source index (bindings.examples is empty)
cap:beta  verification  binding-missing  a binding that resolves through the source index (bindings.tests is empty)
cap:delta  contract  unresolved-proposal  line 26: the block is replaced by plain contract text, or removed because the claim was dropped
cap:delta  contract  section-untouched  a diff of that section against the merge base, or an edit that records why none is needed
cap:gamma  contract  section-untouched  a diff of that section against the merge base, or an edit that records why none is needed
";

/// The roadmap repository as a git checkout on a branch of its own, the
/// sections `grow` affects assessed as they stand.
fn checkout(tag: &str) -> Repo {
    let repo = Repo::synced(tag, false);
    assert_eq!(repo.assess("grow").len(), 3);
    repo.checkout();
    repo
}

/// The same repository with a change `calm` that lists the delivered `alpha`,
/// declares its contract unaffected, and owes nothing.
fn calm(tag: &str) -> Repo {
    let repo = Repo::synced(tag, false);
    repo.put(
        "work/changes/calm.md",
        &change("calm", "[cap:alpha]", &["cap:alpha#contract"])
            .replace("contract_effect: changes", "contract_effect: none"),
    );
    repo.sync();
    repo.assess("calm");
    repo.checkout();
    repo
}

/// The text the library gives for the same repository.
fn library(root: &Path, slug: &str) -> String {
    Lifecycle::open(Programs::new(root).discover_git())
        .expect("the lifecycle opens")
        .reconcile(slug)
        .expect("the change reconciles")
        .to_string()
}

#[test]
fn a_list_with_obligations_is_printed_one_line_each_and_exits_one() {
    let repo = checkout("reconcile-lines");
    let run = repo.both(&["change", "reconcile", "grow"]).expect(1);
    assert_eq!(run.stdout, GROW);
    assert_eq!(run.stderr, "");
    // The line is the library's: the binary decides nothing.
    assert_eq!(run.stdout, library(repo.root(), "grow"));
    // Deterministic: the same run twice, and from elsewhere by `--root`.
    assert_eq!(repo.run(&["change", "reconcile", "grow"]), run);
    assert_eq!(repo.rooted(&["change", "reconcile", "grow"]), run);
}

#[test]
fn an_empty_list_prints_nothing_and_exits_zero() {
    let repo = calm("reconcile-empty");
    let run = repo.both(&["change", "reconcile", "calm"]).expect(0);
    assert_eq!(run.stdout, "");
    assert_eq!(run.stderr, "");
    assert_eq!(library(repo.root(), "calm"), "");
    let rdf = repo
        .both(&["--format", "turtle", "change", "reconcile", "calm"])
        .expect(0);
    assert_eq!(Rdf::of(&rdf).result_count(), 0);
}

#[test]
fn an_unlistable_public_surface_is_exit_two_and_the_resolved_neighbor_answers() {
    let repo = calm("reconcile-surface-error");
    repo.separate_cargo_output();
    let path = repo.root().join("src/lib.rs");
    let ordinary = std::fs::read_to_string(path).expect("fixture source");
    let ambiguous = format!(
        "{ordinary}\npub mod left {{ pub fn duplicate() {{}} }}\n\
         pub mod right {{ pub fn duplicate() {{}} }}\n\
         pub use left::*;\npub use right::*;\n"
    );
    repo.put("src/lib.rs", &ambiguous);
    let before = repo.snapshot();
    let failed = repo.both(&["change", "reconcile", "calm"]).expect(2);
    assert_eq!(failed.stdout, "");
    assert!(
        failed.stderr.contains("public surface of `demo`")
            && failed.stderr.contains("cannot be listed"),
        "{failed:#?}"
    );
    assert_eq!(repo.snapshot(), before);

    repo.put("src/lib.rs", &ambiguous.replace("pub use right::*;\n", ""));
    let repaired = repo.snapshot();
    let accepted = repo.both(&["change", "reconcile", "calm"]).expect(0);
    assert_eq!(accepted.stdout, "");
    assert_eq!(accepted.stderr, "");
    assert_eq!(repo.snapshot(), repaired);
}

#[test]
fn unparseable_merge_base_source_is_an_error_beside_a_readable_base() {
    use raf_workplan::lifecycle::{LifecycleError, Side};

    let repo = calm("reconcile-base-source-error");
    repo.separate_cargo_output();
    let ordinary = std::fs::read_to_string(repo.root().join("src/lib.rs")).unwrap();
    repo.put("src/lib.rs", "pub fn real(\n");
    repo.commit("fixture invalid source baseline");
    repo.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    repo.put("src/lib.rs", &ordinary);
    let before = repo.snapshot();
    let error = Lifecycle::open(Programs::new(repo.root()).discover_git())
        .expect("the valid working tree opens")
        .reconcile("calm")
        .expect_err("invalid source at the merge base is not skipped");
    assert!(
        matches!(&error, LifecycleError::Document { path, side: Side::MergeBase, .. }
            if path == "src/lib.rs"),
        "{error:?}"
    );
    let failed = repo.both(&["change", "reconcile", "calm"]).expect(2);
    assert_eq!(failed.stdout, "");
    assert!(
        failed
            .stderr
            .contains("`src/lib.rs` at the merge base cannot be compared"),
        "{failed:#?}"
    );
    assert_eq!(repo.snapshot(), before);

    repo.commit("fixture readable source baseline");
    repo.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    let repaired = repo.snapshot();
    let accepted = repo.both(&["change", "reconcile", "calm"]).expect(0);
    assert_eq!(accepted.stdout, "");
    assert_eq!(accepted.stderr, "");
    assert_eq!(repo.snapshot(), repaired);
}

#[test]
fn fixing_an_obligation_removes_its_line_and_only_its_line() {
    let repo = checkout("reconcile-fixed");
    let before: Vec<String> = GROW.lines().map(ToString::to_string).collect();
    // Editing the contract of gamma touches the section.
    repo.edit(
        "docs/capabilities/gamma.md",
        "Text for Contract of gamma.",
        "Gamma states its contract here.",
    );
    repo.sync();
    // The author has the edited contract assessed again.
    repo.assess("grow");
    let run = repo.both(&["change", "reconcile", "grow"]).expect(1);
    let after: Vec<&str> = run.lines();
    let expected: Vec<&String> = before
        .iter()
        .filter(|line| !line.starts_with("cap:gamma  contract  section-untouched"))
        .collect();
    assert_eq!(after, expected);

    // Replacing the PROPOSED block of delta by plain text resolves it, and the
    // edit touches the section.
    repo.edit(
        "docs/capabilities/delta.md",
        "> [!PROPOSED change:grow]\n> Delta reads streams as they arrive.\n>\n> It never buffers a whole document.\n",
        "Delta reads streams as they arrive.\n",
    );
    repo.sync();
    repo.assess("grow");
    let run = repo.both(&["change", "reconcile", "grow"]).expect(1);
    assert!(!run.stdout.contains("unresolved-proposal"), "{run:#?}");
    assert!(!run.stdout.contains("cap:delta"), "{run:#?}");

    // An edit that is committed on the branch counts the same: the merge base
    // is the fork.
    repo.commit("edit");
    let committed = repo.both(&["change", "reconcile", "grow"]).expect(1);
    assert_eq!(committed, run);
}

#[test]
fn nothing_is_written_and_no_cargo_is_started_for_a_fixture_without_bound_examples() {
    let repo = checkout("reconcile-readonly");
    let before = repo.worktree();
    let root = repo.root().to_string_lossy().into_owned();
    // A cargo that does not exist: a run that started it would be exit 2, and
    // `--root` means discovery does not need it either.
    let mut command = support::command(
        &support::elsewhere(),
        support::Shape::Direct,
        &["--root", &root, "change", "reconcile", "grow"],
    );
    command.env("CARGO", repo.root().join("no-such-cargo"));
    let run = support::finish(&mut command).expect(1);
    assert_eq!(run.stdout, GROW);
    assert_eq!(repo.worktree(), before, "reconcile wrote");
    repo.git(&["diff", "--quiet"]);
}

#[test]
fn a_bound_example_that_fails_is_an_obligation_and_nothing_is_written_to_the_tree() {
    let repo = Repo::real("reconcile-example");
    repo.put(
        "docs/capabilities/fixture.md",
        &support::capability(
            "fixture",
            "implementing",
            [
                "[demo::add]",
                "[demo::tests::passes]",
                "[demo::example_crate::exiting]",
            ],
            "[]",
            &[("Verification", support::BLOCKS)],
        ),
    );
    repo.put(
        "work/changes/shape.md",
        &change("shape", "[cap:fixture]", &["cap:fixture#contract"]),
    );
    repo.sync();
    repo.checkout();
    repo.edit(
        "docs/capabilities/fixture.md",
        "Text for Contract of fixture.",
        "Fixture states its contract here.",
    );
    repo.sync();
    repo.assess("shape");
    let before = repo.worktree();
    let run = repo.both(&["change", "reconcile", "shape"]).expect(1);
    assert_eq!(run.lines().len(), 1, "{run:#?}");
    assert!(
        run.stdout
            .starts_with("cap:fixture  examples  example-failing  "),
        "{run:#?}"
    );
    assert!(
        run.stdout
            .contains("`demo::example_crate::exiting` fails to build or run (exit code 3)"),
        "{run:#?}"
    );
    assert_eq!(repo.worktree(), before, "reconcile wrote into the tree");

    // Once the example builds and runs the line is gone.
    repo.put("examples/exiting.rs", support::GOOD_EXAMPLE);
    let run = repo.both(&["change", "reconcile", "shape"]).expect(0);
    assert_eq!(run.stdout, "");
}

#[test]
fn everything_that_stops_it_looking_is_exit_two() {
    // A slug with no record.
    let repo = checkout("reconcile-two");
    let run = repo.both(&["change", "reconcile", "nowhere"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.contains("no change record `nowhere`"),
        "{run:#?}"
    );

    // A record that does not load: an outcome that is not one.
    let bad = checkout("reconcile-bad-outcome");
    bad.edit(
        "work/changes/grow.md",
        "items: []",
        "items:\n  - claim: \"x\"\n    outcome: bogus",
    );
    let run = bad.both(&["change", "reconcile", "grow"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("did not load"), "{run:#?}");

    // A repository with no default branch.
    let headless = checkout("reconcile-no-head");
    headless.git(&["symbolic-ref", "--delete", "refs/remotes/origin/HEAD"]);
    let run = headless.both(&["change", "reconcile", "grow"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.contains("git remote set-head origin --auto"),
        "{run:#?}"
    );

    // A directory that is not in a git work tree.
    let plain = Repo::synced("reconcile-no-git", false);
    let run = plain.both(&["change", "reconcile", "grow"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.starts_with("cargo-workplan: "), "{run:#?}");
}

#[test]
fn quiet_keeps_the_obligations_and_hides_nothing_that_is_a_finding() {
    let repo = checkout("reconcile-quiet");
    let loud = repo.run(&["change", "reconcile", "grow"]).expect(1);
    let quiet = repo.both(&["-q", "change", "reconcile", "grow"]).expect(1);
    assert_eq!(quiet.stdout, loud.stdout);
    // The neighbour: an empty list stays empty and exits zero.
    let calm = calm("reconcile-quiet-empty");
    let run = calm
        .both(&["--quiet", "change", "reconcile", "calm"])
        .expect(0);
    assert_eq!(run.stdout, "");
    // A failure is said under `-q`.
    let failed = repo
        .run(&["-q", "change", "reconcile", "nowhere"])
        .expect(2);
    assert!(failed.stderr.contains("no change record"), "{failed:#?}");
}

#[test]
fn rdf_preserves_each_obligation_and_optional_location_in_library_order() {
    let repo = checkout("reconcile-rdf");
    let run = repo
        .both(&["--format", "turtle", "change", "reconcile", "grow"])
        .expect(1);
    let rdf = Rdf::of(&run);
    let obligations = rdf.rows("Obligation");
    assert_eq!(obligations.len(), 6);
    let text: Run = repo.run(&["change", "reconcile", "grow"]);
    for (row, line) in obligations.iter().zip(text.lines()) {
        let fields = [
            row.required("subject"),
            row.required("section"),
            row.required("kind"),
        ];
        assert!(line.starts_with(&fields.join("  ")));
        assert!(line.ends_with(row.required("evidence")));
    }
    let proposal = obligations
        .iter()
        .find(|r| r.required("kind") == "unresolved-proposal")
        .unwrap();
    assert_eq!(proposal.required("subject"), "cap:delta");
    assert_eq!(proposal.required("section"), "contract");
    assert_eq!(proposal.required("path"), "docs/capabilities/delta.md");
    assert_eq!(proposal.number("line"), 26);
    assert_eq!(
        proposal.required("evidence"),
        "the block is replaced by plain contract text, or removed because the claim was dropped"
    );
    let untouched = obligations
        .iter()
        .find(|r| r.required("kind") == "section-untouched")
        .unwrap();
    assert!(untouched.text("line").is_none());
    let quiet = repo
        .run(&["-q", "--format", "turtle", "change", "reconcile", "grow"])
        .expect(1);
    assert_eq!(quiet.stdout, run.stdout);
}

/// The repository of `calm` with the change affecting the sections `affects`
/// of `alpha`, before the checkout, so a test can shape the merge base.
fn calm_before_checkout(tag: &str, affects: &[&str]) -> Repo {
    let repo = Repo::synced(tag, false);
    repo.put(
        "work/changes/calm.md",
        &change("calm", "[cap:alpha]", affects)
            .replace("contract_effect: changes", "contract_effect: none"),
    );
    repo.sync();
    repo.assess("calm");
    repo
}

/// Under `--strict`, `contract_effect: none` leaves out the untouched Contract
/// only: the untouched Usage in `affects` is owed and the run exits 1, while
/// the run without the flag owes nothing; once the Usage is edited and
/// assessed the strict run exits 0 too.
#[test]
fn an_untouched_usage_of_a_none_change_refuses_until_edited() {
    let repo = calm_before_checkout(
        "reconcile-strict-usage",
        &["cap:alpha#contract", "cap:alpha#usage"],
    );
    repo.checkout();
    repo.both(&["change", "reconcile", "calm"]).expect(1);
    let before = repo.worktree();
    let run = repo.both(&["change", "reconcile", "calm"]).expect(1);
    assert_eq!(
        run.stdout,
        "cap:alpha  usage  section-untouched  a diff of that section against the merge base, or an edit that records why none is needed\n"
    );
    assert_eq!(run.stderr, "");
    assert_eq!(repo.worktree(), before, "reconcile wrote");

    repo.edit(
        "docs/capabilities/alpha.md",
        "Text for Usage of alpha.",
        "Alpha is used as its Contract says.",
    );
    repo.sync();
    repo.assess("calm");
    let run = repo.both(&["change", "reconcile", "calm"]).expect(0);
    assert_eq!(run.stdout, "");
}

/// Under `--strict` a change with `contract_effect: none` reads each file that
/// holds a bound public item at the merge base, so a file there that does not
/// parse stops it looking: exit 2, with the file and the side named, and
/// nothing on stdout. The run without the flag reads no source file and
/// exits 0.
#[test]
fn unreadable_bound_source_at_the_merge_base_is_refused_beside_a_parsed_neighbor() {
    let repo = calm_before_checkout("reconcile-strict-unparsed", &["cap:alpha#contract"]);
    let lib = repo.read("src/lib.rs");
    repo.put("src/lib.rs", "pub fn real( {\n");
    repo.checkout();
    repo.put("src/lib.rs", &lib);
    let run = repo.both(&["change", "reconcile", "calm"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr
            .contains("`src/lib.rs` at the merge base cannot be compared: "),
        "{run:#?}"
    );
    let neighbor = calm_before_checkout("reconcile-parsed-neighbor", &["cap:alpha#contract"]);
    neighbor.checkout();
    neighbor.both(&["change", "reconcile", "calm"]).expect(0);
}

/// Under `--strict` a change with `contract_effect: none` lists the public
/// surface of each package that holds a bound item, so a surface that cannot
/// be listed (two globs bring one name in) stops it: exit 2, with the target
/// and the side named, and nothing on stdout. The run without the flag lists
/// no surface and exits 0, and the neighbour with one glob exits 0 under the
/// flag.
#[test]
fn an_unreadable_bound_surface_is_refused_beside_a_resolved_neighbor() {
    let globs = "\npub mod left {\n    /// One.\n    pub fn dup() {}\n}\n\npub mod right {\n    /// Two.\n    pub fn dup() {}\n}\n\npub use left::*;\npub use right::*;\n";
    for (tag, globs, code) in [
        ("reconcile-strict-surface", globs.to_string(), 2),
        (
            "reconcile-strict-surface-one-glob",
            globs.replace("pub use right::*;\n", ""),
            0,
        ),
    ] {
        let repo = calm_before_checkout(tag, &["cap:alpha#contract"]);
        let lib = repo.read("src/lib.rs");
        repo.put("src/lib.rs", &format!("{lib}{globs}"));
        repo.checkout();
        let run = repo.both(&["change", "reconcile", "calm"]).expect(code);
        assert_eq!(run.stdout, "", "{tag}");
        if code == 2 {
            assert!(
                run.stderr.contains("in the working tree cannot be listed, so the bound public items the change touches are unknown: "),
                "{run:#?}"
            );
        } else {
            assert_eq!(run.stderr, "", "{tag}");
        }
    }
}
