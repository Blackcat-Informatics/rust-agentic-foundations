// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `verify` and the `example-failing` obligation of `Lifecycle::reconcile`
//! against the real `cargo`, on every platform.
//!
//! Each fixture is a small repository generated into a scratch directory from
//! the templates below: a crate with its own `[workspace]`, a template
//! `Cargo.lock`, no dependency and a pinned edition, the capability documents
//! that bind it and a change record. Each builds into a target directory of its
//! own, beside it and removed with it (named in the fixture's
//! `.cargo/config.toml`, so the environment of this process is not touched),
//! and `cargo` is the one that started this test binary (`$CARGO`). Every test asserts a wall-clock budget. A fixture that
//! must start no process runs with a `cargo` and a `git` that do not exist, so
//! a process started anyway fails as an error and cannot pass.

mod support;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use raf_workplan::bound::{BoundOutcome, Evidence};
use raf_workplan::exec::{
    ExecError, Invocation, Limits, Output, Program, Programs, Runner, SystemRunner,
};
use raf_workplan::lifecycle::{Lifecycle, LifecycleError, VerifyReport, verify};
use raf_workplan::obligation::ObligationKind;
use raf_workplan::{ChangeLookupError, Gate, Opened, SyncOptions, Workplan};
use std::sync::Mutex;
use support::changes::{DONE, capability, record};
use support::checkout::git;
use support::disk::{Scratch, put, snapshot};

/// No fixture may take longer than this, builds included.
const BUDGET: Duration = Duration::from_secs(300);

/// The `cargo` that runs this test binary: `$CARGO` as `cargo test` sets it.
fn cargo() -> PathBuf {
    std::env::var_os("CARGO")
        .map(PathBuf::from)
        .or_else(|| option_env!("CARGO").map(PathBuf::from))
        .expect("cargo test sets CARGO")
}

const LIB: &str = "\
//! Fixture library.

/// Adds.
///
/// ```
/// assert_eq!(fx_life::add(1, 2), 3);
/// ```
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

/// Documented, and with no example in its documentation.
pub fn bare() {}

/// Documented, with an example that fails.
///
/// ```
/// assert_eq!(2 + 2, 5);
/// ```
pub fn wrong() {}

pub fn plain() {}

#[cfg(test)]
mod tests {
    #[test]
    fn passes() {
        assert_eq!(super::add(1, 2), 3);
    }

    #[test]
    fn also_passes() {}

    #[test]
    fn fails() {
        panic!(\"this test fails\");
    }

    #[test]
    #[ignore]
    fn skipped() {}
}
";

/// An example that builds and runs.
const GOOD_EXAMPLE: &str = "fn main() {\n    println!(\"fine\");\n}\n";

/// An example that does not compile.
const BROKEN_EXAMPLE: &str = "fn main() {\n    let number: u32 = \"text\";\n    drop(number);\n}\n";

/// An example that compiles and exits with a failing status.
const EXITING_EXAMPLE: &str = "fn main() {\n    std::process::exit(3);\n}\n";

/// A generated repository, removed when dropped.
struct Fixture {
    scratch: Scratch,
    started: Instant,
}

impl Fixture {
    /// The crate `fx-life` with these examples, and `docs` written below the
    /// root, then synced.
    fn new(examples: &[(&str, &str)], docs: &[(&str, String)]) -> Self {
        let fixture = Self::unsynced(examples, docs);
        fixture.sync();
        fixture
    }

    /// As [`Fixture::new`], without the sync: for documents that do not load,
    /// which a sync refuses.
    fn unsynced(examples: &[(&str, &str)], docs: &[(&str, String)]) -> Self {
        let scratch = Scratch::new("verify");
        let root = scratch.path();
        put(
            root,
            "Cargo.toml",
            "[package]\nname = \"fx-life\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
        );
        put(
            root,
            "Cargo.lock",
            "version = 4\n\n[[package]]\nname = \"fx-life\"\nversion = \"0.1.0\"\n",
        );
        put(root, ".cargo/config.toml", &scratch.cargo_config());
        put(root, "src/lib.rs", LIB);
        for (name, text) in examples {
            put(root, &format!("examples/{name}.rs"), text);
        }
        for (path, text) in docs {
            put(root, path, text);
        }
        // `Lifecycle` asks `git` where the root is in its work tree.
        git(root, &["init", "-q"]);
        Self {
            scratch,
            started: Instant::now(),
        }
    }

    fn root(&self) -> &Path {
        self.scratch.path()
    }

    /// Run a sync over the documents and write what it returns.
    fn sync(&self) {
        let opened = Gate::workspace(self.root())
            .open()
            .expect("the fixture opens");
        let report = opened
            .workplan()
            .sync(&SyncOptions::new(
                "2026 Example Org <dev@example.org>",
                "CC-BY-4.0",
            ))
            .unwrap_or_else(|error| panic!("the fixture syncs: {error}"));
        Workplan::write(&report, self.root()).expect("the sync is written");
    }

    /// The programs of the fixture with the real `cargo` and `git`.
    fn programs(&self) -> Programs {
        Programs::with_cargo_var(self.root(), Some(cargo().into()))
    }

    /// The programs of the fixture with a `cargo` and a `git` that do not
    /// exist: a process started with them is an error.
    fn programs_without_processes(&self) -> Programs {
        self.programs()
            .with_cargo(self.root().join("no-such-cargo"))
            .with_git(self.root().join("no-such-git"))
    }

    /// The repository opened with the tracked paths given, so that no `git`
    /// process is needed to open it.
    fn opened(&self) -> Opened {
        Gate::workspace(self.root())
            .tracked_paths(Vec::new())
            .open()
            .expect("the fixture opens")
    }

    fn lifecycle(&self) -> Lifecycle {
        Lifecycle::open(self.programs()).unwrap_or_else(|error| panic!("opens: {error}"))
    }

    fn within_budget(&self) {
        assert!(
            self.started.elapsed() < BUDGET,
            "the fixture took {:?}, over its budget of {BUDGET:?}",
            self.started.elapsed()
        );
    }
}

/// The capability `fixture` with these bindings (the text after `code:`,
/// `tests:` and `examples:`), implementing.
fn fixture_cap(slug: &str, tests: &str, examples: &str) -> (String, String) {
    (
        format!("docs/capabilities/{slug}.md"),
        capability(
            slug,
            "implementing",
            ["[fx_life::add]", tests, examples],
            &[],
        ),
    )
}

/// The change `shape`, which lists these capabilities and affects only the
/// contract of the first, which `contract_effect: none` leaves out of the
/// merge-base comparison.
fn shape(capabilities: &[&str]) -> (String, String) {
    let ids = capabilities
        .iter()
        .map(|slug| format!("cap:{slug}"))
        .collect::<Vec<_>>()
        .join(", ");
    (
        "work/changes/shape.md".to_string(),
        record(
            "shape",
            &format!("[{ids}]"),
            &[&format!("cap:{}#contract", capabilities[0])],
            "none",
            DONE,
        ),
    )
}

fn docs(files: &[(String, String)]) -> Vec<(&str, String)> {
    files
        .iter()
        .map(|(path, text)| (path.as_str(), text.clone()))
        .collect()
}

/// A fixture with the capability `fixture` bound as given, and the change
/// `shape` that lists it.
fn single(tests: &str, examples: &str, example_files: &[(&str, &str)]) -> Fixture {
    let files = [fixture_cap("fixture", tests, examples), shape(&["fixture"])];
    Fixture::new(example_files, &docs(&files))
}

fn report_of(fixture: &Fixture, change: Option<&str>) -> VerifyReport {
    let report = verify(
        &fixture.opened(),
        &SystemRunner::new(fixture.programs()),
        change,
    )
    .unwrap_or_else(|error| panic!("verify should run: {error}"));
    fixture.within_budget();
    report
}

fn report_lines(report: &VerifyReport) -> Vec<String> {
    report.to_string().lines().map(str::to_string).collect()
}

const ADD_DOC: &str = "fx_life::add";

// --- verify ---------------------------------------------------------------------

#[test]
fn one_passing_and_one_failing_test_print_one_line_each_and_the_run_fails() {
    let fixture = single("[fx_life::tests::passes, fx_life::tests::fails]", "[]", &[]);
    let report = report_of(&fixture, Some("shape"));
    assert!(!report.passed());
    assert_eq!(report.len(), 2);
    assert_eq!(report.change(), Some("shape"));
    let lines = report_lines(&report);
    // One line a binding, sorted by path.
    assert_eq!(lines.len(), 2, "{lines:#?}");
    assert_eq!(
        lines[0].split(": ").next(),
        Some("fail cap:fixture tests `fx_life::tests::fails`")
    );
    assert!(lines[0].contains("1 failed"), "{}", lines[0]);
    assert_eq!(
        lines[1],
        "pass cap:fixture tests `fx_life::tests::passes` (1 listed, 1 passed, 0 failed, 0 ignored)"
    );
    assert_eq!(report.failures().count(), 1);
    assert_eq!(
        report.evidence()[1].outcome(),
        &BoundOutcome::Passed,
        "the passing neighbour passed"
    );

    // The neighbour run: only the passing test, and the run passes.
    let fixture = single(
        "[fx_life::tests::passes, fx_life::tests::also_passes]",
        "[]",
        &[],
    );
    let report = report_of(&fixture, Some("shape"));
    assert!(report.passed(), "{report}");
    assert!(
        report_lines(&report)
            .iter()
            .all(|line| line.starts_with("pass "))
    );
}

#[test]
fn no_slug_runs_every_capability_and_a_slug_only_the_ones_the_change_lists() {
    let files = [
        fixture_cap("fixture", "[fx_life::tests::passes]", "[]"),
        fixture_cap("other", "[fx_life::tests::fails]", "[]"),
        shape(&["fixture"]),
    ];
    let fixture = Fixture::new(&[], &docs(&files));

    let one = report_of(&fixture, Some("shape"));
    assert!(one.passed(), "{one}");
    assert_eq!(one.len(), 1);

    let all = report_of(&fixture, None);
    assert_eq!(all.change(), None);
    assert!(!all.passed());
    assert_eq!(all.len(), 2);
    let lines = report_lines(&all);
    assert!(
        lines[0].starts_with("pass cap:fixture tests `fx_life::tests::passes`"),
        "{lines:#?}"
    );
    assert!(
        lines[1].starts_with("fail cap:other tests `fx_life::tests::fails`"),
        "{lines:#?}"
    );
}

#[test]
fn examples_and_doc_tests_are_run_and_judged_by_their_proof() {
    let fixture = single(
        "[fx_life::tests::passes]",
        "[fx_life::example_crate::good, fx_life::example_crate::exiting, fx_life::add, fx_life::wrong, fx_life::bare]",
        &[("good", GOOD_EXAMPLE), ("exiting", EXITING_EXAMPLE)],
    );
    let report = report_of(&fixture, Some("shape"));
    assert!(!report.passed());
    let by_path = |path: &str| {
        report
            .evidence()
            .iter()
            .find(|evidence| evidence.path() == path)
            .unwrap_or_else(|| panic!("no evidence for {path}: {report}"))
    };
    assert!(by_path("fx_life::example_crate::good").passed());
    assert!(by_path(ADD_DOC).passed());
    assert_eq!(by_path(ADD_DOC).proof().selected, Some(1));
    assert_eq!(by_path(ADD_DOC).proof().ok_lines, Some(1));
    assert!(matches!(
        by_path("fx_life::example_crate::exiting").outcome(),
        BoundOutcome::Failed { reason } if reason.contains("exited with code 3")
    ));
    assert!(matches!(
        by_path("fx_life::wrong").outcome(),
        BoundOutcome::Failed { .. }
    ));
    // A documented item with no documentation test runs zero tests.
    assert!(matches!(
        by_path("fx_life::bare").outcome(),
        BoundOutcome::Unresolved { .. }
    ));
    let text = report.to_string();
    assert!(
        text.contains("fail cap:fixture examples `fx_life::example_crate::exiting`: "),
        "{text}"
    );
    assert!(
        text.contains(
            "fail cap:fixture examples `fx_life::bare`: unresolved binding: no documentation test"
        ),
        "{text}"
    );
    assert!(
        text.contains("pass cap:fixture examples `fx_life::add` (")
            && text.contains("1 selected, 1 ok lines)"),
        "{text}"
    );
    // Tests come before examples, the lines sort by path within a list.
    let order: Vec<&str> = report.evidence().iter().map(Evidence::path).collect();
    assert_eq!(
        order,
        [
            "fx_life::tests::passes",
            "fx_life::add",
            "fx_life::bare",
            "fx_life::example_crate::exiting",
            "fx_life::example_crate::good",
            "fx_life::wrong",
        ]
    );
}

#[test]
fn zero_tests_and_ignored_tests_fail_the_run() {
    let fixture = single(
        "[fx_life::plain, fx_life::tests::skipped, fx_life::tests::passes]",
        "[]",
        &[],
    );
    let report = report_of(&fixture, Some("shape"));
    assert!(!report.passed());
    let text = report.to_string();
    assert!(
        text.contains("fail cap:fixture tests `fx_life::plain`: unresolved binding:")
            && text.contains("would run zero tests"),
        "{text}"
    );
    assert!(
        text.contains("fail cap:fixture tests `fx_life::tests::skipped`: unresolved binding:")
            && text.contains("#[ignore]"),
        "{text}"
    );
    assert!(
        text.contains("pass cap:fixture tests `fx_life::tests::passes`"),
        "{text}"
    );
}

#[test]
fn hostile_bindings_are_unresolved_results_that_never_reach_cargo() {
    let hostile = [
        "-Zunstable-options",
        "--include-ignored",
        "--exact",
        "a;b",
        "a b",
    ];
    let quoted = hostile
        .iter()
        .map(|entry| format!("\"{entry}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let fixture = single(
        &format!("[{quoted}]"),
        &format!("[\"{}\"]", hostile[2]),
        &[],
    );
    // A `cargo` and a `git` that do not exist: a process started would be an
    // error and not an unresolved binding.
    let runner = SystemRunner::new(fixture.programs_without_processes());
    let report = verify(&fixture.opened(), &runner, Some("shape")).expect("no process is needed");
    assert!(!report.passed());
    assert_eq!(report.len(), hostile.len() + 1);
    for evidence in report.evidence() {
        assert!(evidence.runs().is_empty(), "{evidence}");
        assert!(
            matches!(evidence.outcome(), BoundOutcome::Unresolved { .. }),
            "{evidence}"
        );
    }
    let text = report.to_string();
    assert_eq!(text.lines().count(), hostile.len() + 1, "{text}");
    for entry in hostile {
        assert!(
            text.contains("fail cap:fixture ")
                && text.contains(&format!("`{entry}`: unresolved binding")),
            "{entry}: {text}"
        );
    }

    // The neighbour: a plain path runs, so the same fixture with a real
    // `cargo` passes it.
    let fixture = single("[fx_life::tests::passes]", "[]", &[]);
    assert!(report_of(&fixture, Some("shape")).passed());
}

/// A runner that records which programs it was asked to start.
struct Recording<'r> {
    inner: &'r SystemRunner,
    programs: Mutex<Vec<Program>>,
}

impl Runner for Recording<'_> {
    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError> {
        self.programs
            .lock()
            .expect("a lock")
            .push(invocation.program());
        self.inner.run(invocation, limits)
    }
}

#[test]
fn verify_asks_git_nothing_and_does_not_go_through_the_version_gate() {
    let fixture = single("[fx_life::tests::passes]", "[]", &[]);
    let inner = SystemRunner::new(fixture.programs_without_processes().with_cargo(cargo()));
    let runner = Recording {
        inner: &inner,
        programs: Mutex::new(Vec::new()),
    };
    // `git` does not exist here, which `Lifecycle::open` would refuse at the
    // version gate; `verify` does not ask.
    assert!(Lifecycle::open(fixture.programs_without_processes()).is_err());
    let report = verify(&fixture.opened(), &runner, Some("shape")).expect("verify runs");
    assert!(report.passed());
    let started = runner.programs.lock().expect("a lock").clone();
    assert_ne!(started.len(), 0);
    assert!(
        started.iter().all(|program| *program == Program::Cargo),
        "{started:?}"
    );
}

#[test]
fn verify_writes_nothing() {
    let fixture = single(
        "[fx_life::tests::passes, fx_life::tests::fails]",
        "[fx_life::example_crate::good]",
        &[("good", GOOD_EXAMPLE)],
    );
    let before = snapshot(fixture.root());
    let first = report_of(&fixture, Some("shape"));
    let second = report_of(&fixture, None);
    assert_eq!(first.len(), 3);
    assert_eq!(second.len(), 3);
    assert_eq!(
        snapshot(fixture.root()),
        before,
        "the tree is byte-identical"
    );
}

#[test]
fn an_unknown_slug_and_a_record_that_did_not_load_are_different_errors() {
    let broken =
        "---\ntype: Change\nid: change:broken\n---\n\n## Summary\n\nNo keys.\n".to_string();
    let files = [
        fixture_cap("fixture", "[fx_life::tests::passes]", "[]"),
        shape(&["fixture"]),
        ("work/changes/broken.md".to_string(), broken),
    ];
    let fixture = Fixture::unsynced(&[], &docs(&files));
    let opened = fixture.opened();
    let runner = SystemRunner::new(fixture.programs_without_processes());

    let unknown = verify(&opened, &runner, Some("nothing-like-it")).expect_err("unknown");
    assert!(
        matches!(&unknown, LifecycleError::Change(ChangeLookupError::Unknown { known, .. })
            if known == &["shape".to_string()]),
        "{unknown:?}"
    );
    let not_loaded = verify(&opened, &runner, Some("broken")).expect_err("not loaded");
    assert!(
        matches!(
            &not_loaded,
            LifecycleError::Change(ChangeLookupError::NotLoaded { .. })
        ),
        "{not_loaded:?}"
    );
    assert_ne!(unknown.to_string(), not_loaded.to_string());
    // The prefix is accepted for a slug that exists: the neighbour.
    let runner = SystemRunner::new(fixture.programs());
    assert!(
        verify(&opened, &runner, Some("change:shape"))
            .expect("known")
            .passed()
    );
}

#[test]
fn a_capability_that_did_not_load_is_an_error_and_not_a_pass() {
    let (path, text) = fixture_cap("fixture", "[fx_life::tests::passes]", "[]");
    let unloaded = text.replace("title: \"Title of fixture\"\n", "");
    assert_ne!(unloaded, text);
    let files = [(path.clone(), unloaded), shape(&["fixture"])];
    let fixture = Fixture::unsynced(&[], &docs(&files));
    let runner = SystemRunner::new(fixture.programs_without_processes());

    let listed = verify(&fixture.opened(), &runner, Some("shape")).expect_err("not loaded");
    assert!(matches!(listed, LifecycleError::Reconcile(_)), "{listed:?}");
    assert!(listed.to_string().contains("cap:fixture"), "{listed}");

    let every = verify(&fixture.opened(), &runner, None).expect_err("not loaded");
    assert!(
        matches!(&every, LifecycleError::Unloaded { capabilities, .. }
            if capabilities == &["cap:fixture".to_string()]),
        "{every:?}"
    );
    assert!(
        every.to_string().contains(&path) || every.to_string().contains("title"),
        "{every}"
    );

    // The neighbour: the same capability with its title loads and verifies.
    let fixture = single("[fx_life::tests::passes]", "[]", &[]);
    assert!(report_of(&fixture, None).passed());
}

#[test]
fn a_run_with_nothing_bound_passes_and_prints_nothing() {
    let fixture = single("[]", "[]", &[]);
    let runner = SystemRunner::new(fixture.programs_without_processes());
    let report = verify(&fixture.opened(), &runner, Some("shape")).expect("nothing to run");
    assert!(report.is_empty());
    assert!(report.passed());
    assert_eq!(report.to_string(), "");
}

#[test]
fn a_process_that_cannot_start_is_an_error_and_not_a_failing_binding() {
    let fixture = single("[fx_life::tests::passes]", "[]", &[]);
    let runner = SystemRunner::new(fixture.programs_without_processes());
    let error = verify(&fixture.opened(), &runner, Some("shape")).expect_err("no cargo");
    assert!(
        matches!(error, LifecycleError::Exec(ExecError::Spawn { .. })),
        "{error:?}"
    );
}

// --- example-failing ----------------------------------------------------------------

const EXAMPLE: &str = "fx_life::example_crate::broken";

fn broken_example_fixture() -> Fixture {
    let files = [
        fixture_cap(
            "fixture",
            "[fx_life::tests::passes]",
            &format!("[{EXAMPLE}]"),
        ),
        shape(&["fixture"]),
    ];
    Fixture::new(&[("broken", BROKEN_EXAMPLE)], &docs(&files))
}

#[test]
fn an_example_that_does_not_build_is_a_line_and_is_gone_once_it_builds_and_runs() {
    let fixture = broken_example_fixture();
    let list = fixture
        .lifecycle()
        .reconcile("shape")
        .expect("reconcile runs");
    fixture.within_budget();
    // Nothing else is owed: the line is the whole list.
    assert_eq!(list.len(), 1, "{list}");
    let line = list.to_string();
    assert_eq!(
        line,
        format!(
            "cap:fixture  examples  example-failing  a passing verify run for the change, or a binding edit that removes the broken example (`{EXAMPLE}` fails to build or run (exit code 101))\n"
        )
    );
    let owed = &list.obligations()[0];
    assert_eq!(owed.kind, ObligationKind::ExampleFailing);
    assert_eq!(owed.location.path, "docs/capabilities/fixture.md");
    // The same input prints the same text.
    assert_eq!(
        fixture
            .lifecycle()
            .reconcile("shape")
            .expect("again")
            .to_string(),
        line
    );

    // Fixing the example removes the line, and the list is empty.
    put(fixture.root(), "examples/broken.rs", GOOD_EXAMPLE);
    let fixed = fixture
        .lifecycle()
        .reconcile("shape")
        .expect("reconcile runs");
    assert!(fixed.is_empty(), "{fixed}");
    fixture.within_budget();
}

#[test]
fn an_example_that_exits_with_a_failure_or_a_doc_test_that_fails_is_a_line() {
    let files = [
        fixture_cap(
            "fixture",
            "[fx_life::tests::passes]",
            "[fx_life::example_crate::exiting, fx_life::wrong, fx_life::add, fx_life::example_crate::good]",
        ),
        shape(&["fixture"]),
    ];
    let fixture = Fixture::new(
        &[("exiting", EXITING_EXAMPLE), ("good", GOOD_EXAMPLE)],
        &docs(&files),
    );
    let list = fixture
        .lifecycle()
        .reconcile("shape")
        .expect("reconcile runs");
    let owed: Vec<String> = list
        .of_kind(ObligationKind::ExampleFailing)
        .map(ToString::to_string)
        .collect();
    assert_eq!(owed.len(), 2, "{list}");
    assert!(
        owed[0].contains("`fx_life::example_crate::exiting` fails to build or run (exit code 3)"),
        "{}",
        owed[0]
    );
    assert!(
        owed[1].contains("`fx_life::wrong` fails to build or run (exit code 101)"),
        "{}",
        owed[1]
    );
    // The passing neighbours (`add`, `good`) are not listed.
    assert!(!list.to_string().contains("fx_life::add"), "{list}");
    assert!(!list.to_string().contains("example_crate::good"), "{list}");
}

#[test]
fn a_binding_to_a_function_with_no_test_reports_zero_tests_as_failing() {
    // `bare` is documented and has no documentation test; `plain` is neither
    // documented nor an example. Both run zero tests.
    let files = [
        fixture_cap(
            "fixture",
            "[fx_life::tests::passes]",
            "[fx_life::bare, fx_life::plain]",
        ),
        shape(&["fixture"]),
    ];
    let fixture = Fixture::new(&[], &docs(&files));
    let list = fixture
        .lifecycle()
        .reconcile("shape")
        .expect("reconcile runs");
    let owed: Vec<String> = list
        .of_kind(ObligationKind::ExampleFailing)
        .map(ToString::to_string)
        .collect();
    assert_eq!(owed.len(), 2, "{list}");
    assert!(
        owed[0].contains("`fx_life::bare` proves nothing: no documentation test of the package `fx-life` is listed for the item `bare`"),
        "{}",
        owed[0]
    );
    assert!(
        owed[1].contains("`fx_life::plain` proves nothing:")
            && owed[1].contains("neither the crate root"),
        "{}",
        owed[1]
    );
    // The wrong role of `plain` is also a `binding-missing` line of its own.
    assert_eq!(
        list.of_kind(ObligationKind::BindingMissing).count(),
        1,
        "{list}"
    );
}

#[test]
fn test_bindings_are_not_run_by_reconcile_and_unresolved_examples_are_not_started() {
    let files = [
        fixture_cap(
            "fixture",
            "[fx_life::tests::fails]",
            "[fx_life::example_crate::nowhere]",
        ),
        shape(&["fixture"]),
    ];
    let fixture = Fixture::new(&[], &docs(&files));
    // A `cargo` that does not exist: a test binding that was run, or an
    // example that does not resolve and was run, would be an error here.
    let lifecycle = Lifecycle::open(fixture.programs_without_processes().with_git("git"))
        .unwrap_or_else(|error| panic!("opens: {error}"));
    let list = lifecycle
        .reconcile("shape")
        .expect("no cargo process is needed");
    assert_eq!(
        list.of_kind(ObligationKind::ExampleFailing).count(),
        0,
        "{list}"
    );
    // The unresolved example is `binding-missing`, which owes it already.
    assert_eq!(
        list.of_kind(ObligationKind::BindingMissing).count(),
        1,
        "{list}"
    );
}

#[test]
fn an_example_that_cannot_be_started_is_an_error_and_not_a_line() {
    let fixture = broken_example_fixture();
    let lifecycle = Lifecycle::open(fixture.programs_without_processes().with_git("git"))
        .unwrap_or_else(|error| panic!("opens: {error}"));
    let error = lifecycle.reconcile("shape").expect_err("no cargo");
    assert!(
        matches!(error, LifecycleError::Exec(ExecError::Spawn { .. })),
        "{error:?}"
    );
}

#[test]
fn only_the_examples_of_the_capabilities_the_change_lists_are_run() {
    let files = [
        fixture_cap(
            "fixture",
            "[fx_life::tests::passes]",
            "[fx_life::example_crate::good]",
        ),
        fixture_cap("other", "[fx_life::tests::passes]", &format!("[{EXAMPLE}]")),
        shape(&["fixture"]),
    ];
    let fixture = Fixture::new(
        &[("good", GOOD_EXAMPLE), ("broken", BROKEN_EXAMPLE)],
        &docs(&files),
    );
    let list = fixture
        .lifecycle()
        .reconcile("shape")
        .expect("reconcile runs");
    assert_eq!(
        list.of_kind(ObligationKind::ExampleFailing).count(),
        0,
        "{list}"
    );
    assert!(list.is_empty(), "{list}");
}

#[test]
fn the_lines_of_a_report_are_its_text_and_each_verdict_has_a_name_and_a_reason() {
    let fixture = single(
        "[fx_life::tests::passes, fx_life::tests::fails]",
        "[fx_life::wrong]",
        &[],
    );
    let report = report_of(&fixture, Some("shape"));
    assert_eq!(report.lines().join("\n") + "\n", report.to_string());
    assert_eq!(report.lines().len(), report.len());
    let verdicts: Vec<(&str, bool)> = report
        .evidence()
        .iter()
        .map(|evidence| {
            (
                evidence.outcome().name(),
                evidence.outcome().reason().is_some(),
            )
        })
        .collect();
    // `fails` (failed), `passes` (passed) and `wrong` (a doc test that fails).
    assert_eq!(
        verdicts
            .iter()
            .filter(|(name, _)| *name == "passed")
            .count(),
        1
    );
    for evidence in report.evidence() {
        match evidence.outcome() {
            BoundOutcome::Passed => {
                assert_eq!(evidence.outcome().name(), "passed");
                assert_eq!(evidence.outcome().reason(), None);
            }
            BoundOutcome::Failed { reason } => {
                assert_eq!(evidence.outcome().name(), "failed");
                assert_eq!(
                    evidence.outcome().reason().as_deref(),
                    Some(reason.as_str())
                );
            }
            BoundOutcome::Unresolved { reason } => {
                assert_eq!(evidence.outcome().name(), "unresolved");
                assert_eq!(
                    evidence.outcome().reason().as_deref(),
                    Some(reason.to_string().as_str())
                );
            }
            other => panic!("unexpected verdict {other:?}"),
        }
    }
    assert!(verdicts.iter().any(|(name, _)| *name == "failed"));
}
