// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The executors of bound tests, documentation tests and examples against the
//! real `cargo`, on every platform.
//!
//! Each fixture is a small crate generated into a scratch directory from the
//! templates below: its own `[workspace]`, a template `Cargo.lock`, no
//! dependency and a pinned edition, so nothing is fetched and no enclosing
//! workspace is read. Each builds into a target directory of its own, beside it
//! and removed with it (named in the fixture's `.cargo/config.toml`, so the
//! environment of this process is not touched): fixtures that name the same
//! package share a build fingerprint, so one directory for all of them lets a
//! sibling's `cargo` rewrite or lock an executable another is running. `cargo`
//! is the one that started this test binary (`$CARGO`).
//! Every test asserts a wall-clock budget. A refusal before any process is run
//! with a `cargo` that does not exist, so a refusal that started a process
//! would fail as an error, not pass.

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use raf_source::{DiskTree, MemoryTree, Workspace};
use raf_workplan::Workplan;
use raf_workplan::bound::{BoundOutcome, Ended, Evidence, Executor, Refusal};
use raf_workplan::exec::{ExecError, Limits, Programs, SystemRunner};

/// No fixture may take longer than this, builds included.
const BUDGET: Duration = Duration::from_secs(300);

/// The `cargo` that runs this test binary: `$CARGO` as `cargo test` sets it.
fn cargo() -> PathBuf {
    std::env::var_os("CARGO")
        .map(PathBuf::from)
        .or_else(|| option_env!("CARGO").map(PathBuf::from))
        .expect("cargo test sets CARGO")
}

/// A generated crate, removed when dropped together with its target directory.
struct Fixture {
    dir: PathBuf,
    target: PathBuf,
    package: String,
    source: Workspace,
    started: Instant,
}

impl Fixture {
    /// A crate named `package` of this edition. `manifest` is appended to its
    /// manifest and `files` are written below the crate directory.
    fn new(package: &str, edition: &str, manifest: &str, files: &[(&str, &str)]) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!(
            "raf-bound-{}-{nanos}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let target = dir.with_file_name(format!(
            "{}-target",
            dir.file_name().and_then(|n| n.to_str()).expect("a name")
        ));
        let put = |path: &str, text: &str| {
            let target = dir.join(path);
            fs::create_dir_all(target.parent().expect("a file has a directory"))
                .expect("the directory of a file");
            fs::write(target, text).expect("a file");
        };
        put(
            "Cargo.toml",
            &format!(
                "[package]\nname = \"{package}\"\nversion = \"0.1.0\"\nedition = \"{edition}\"\n\n\
                 {manifest}\n[workspace]\n"
            ),
        );
        put(
            "Cargo.lock",
            &format!("version = 4\n\n[[package]]\nname = \"{package}\"\nversion = \"0.1.0\"\n"),
        );
        put(
            ".cargo/config.toml",
            &format!(
                "[build]\ntarget-dir = '{0}'\nbuild-dir = '{0}'\n",
                target.display()
            ),
        );
        for (path, text) in files {
            put(path, text);
        }
        let source = Workspace::load(&DiskTree::new(&dir).expect("a tree"), &["."])
            .expect("the fixture loads");
        Self {
            dir,
            target,
            package: package.to_string(),
            source,
            started: Instant::now(),
        }
    }

    /// A runner for the real `cargo`.
    fn runner(&self) -> SystemRunner {
        SystemRunner::new(Programs::with_cargo_var(&self.dir, Some(cargo().into())))
    }

    /// A runner whose `cargo` does not exist: a binding that is refused before
    /// any process is run never notices, and one that starts a process fails
    /// with an [`ExecError`].
    fn runner_without_cargo(&self) -> SystemRunner {
        SystemRunner::new(Programs::with_cargo_var(
            &self.dir,
            Some(self.dir.join("no-such-cargo").into()),
        ))
    }

    /// Build what a binding needs before a short limit applies to it, so that
    /// the limit measures the run and not the compile.
    fn warm(&self, args: &[&str]) {
        let status = Command::new(cargo())
            .args(args)
            .arg("--frozen")
            .current_dir(&self.dir)
            .output()
            .expect("cargo starts");
        assert!(
            status.status.success(),
            "{}",
            String::from_utf8_lossy(&status.stderr)
        );
    }

    /// Run the binding `entry` of `list` with `runner` and these limits.
    fn judge(
        &self,
        runner: &SystemRunner,
        list: &str,
        entry: &str,
        limits: Option<(Limits, Limits)>,
    ) -> Result<Evidence, ExecError> {
        let doc = capability(list, entry);
        let docs = MemoryTree::from([("docs/capabilities/fixture.md", doc.as_str())]);
        let plan = Workplan::load(&docs, &self.source);
        let binding = plan
            .graph()
            .bindings()
            .first()
            .expect("the capability lists a binding");
        assert!(
            binding.resolution().is_ok(),
            "{entry} must resolve: {:?}",
            binding.resolution()
        );
        let mut executor = Executor::new(&self.source, runner).expect("the index lists its tests");
        if let Some((build, example)) = limits {
            executor = executor.with_limits(build, example);
        }
        executor.run(binding)
    }

    /// Run a binding with the real `cargo` and the limits of the gate.
    fn run(&self, list: &str, entry: &str) -> Evidence {
        let evidence = self
            .judge(&self.runner(), list, entry, None)
            .unwrap_or_else(|error| panic!("{entry}: {error}"));
        self.within_budget();
        evidence
    }

    /// A binding that must be refused before any process starts.
    fn refused(&self, list: &str, entry: &str) -> Refusal {
        let evidence = self
            .judge(&self.runner_without_cargo(), list, entry, None)
            .unwrap_or_else(|error| panic!("{entry} started a process: {error}"));
        assert!(evidence.runs().is_empty(), "{evidence}");
        match evidence.outcome() {
            BoundOutcome::Unresolved { reason } => reason.clone(),
            other => panic!("{entry}: {other:?}"),
        }
    }

    fn within_budget(&self) {
        assert!(
            self.started.elapsed() < BUDGET,
            "the fixture {} took {:?}, over its budget of {BUDGET:?}",
            self.package,
            self.started.elapsed()
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
        let _ = fs::remove_dir_all(&self.target);
    }
}

/// The capability document that lists `entry` in `list`.
fn capability(list: &str, entry: &str) -> String {
    let mut sections = String::new();
    for name in [
        "Purpose",
        "Contract",
        "Usage",
        "Examples",
        "Constraints",
        "Explanation",
        "Verification",
    ] {
        write!(
            sections,
            "## {name}

Text for {name}.

"
        )
        .unwrap();
    }
    let entry = format!("\"{entry}\"");
    let (code, tests, examples) = match list {
        "code" => (entry.as_str(), "", ""),
        "tests" => ("", entry.as_str(), ""),
        _ => ("", "", entry.as_str()),
    };
    format!(
        "---\ntype: Capability\nid: cap:fixture\ntitle: Fixture\ndescription: Fixture.\n\
         state: implementing\nbindings:\n  code: [{code}]\n  tests: [{tests}]\n  \
         examples: [{examples}]\ndepends: []\n---\n\n# Fixture\n\n{sections}"
    )
}

fn passed(evidence: &Evidence) {
    assert_eq!(evidence.outcome(), &BoundOutcome::Passed, "{evidence}");
}

fn failed(evidence: &Evidence) -> &str {
    match evidence.outcome() {
        BoundOutcome::Failed { reason } => reason,
        other => panic!("{other:?}: {evidence}"),
    }
}

fn unresolved(evidence: &Evidence) -> &Refusal {
    match evidence.outcome() {
        BoundOutcome::Unresolved { reason } => reason,
        other => panic!("{other:?}: {evidence}"),
    }
}

// ------------------------------------------------------------------- tests

const TESTS_MANIFEST: &str = "\
[features]
extra = []

[[test]]
name = \"no-harness\"
path = \"tests/no_harness.rs\"
harness = false

[[test]]
name = \"needs-feature\"
path = \"tests/needs_feature.rs\"
required-features = [\"extra\"]

[[bench]]
name = \"speed\"
path = \"benches/speed.rs\"
";

const TESTS_LIB: &str = "\
//! Fixture library.

/// Adds.
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

/// Not a test.
pub fn plain() {}

#[test]
fn same() {}

#[cfg(feature = \"extra\")]
#[test]
fn gated() {}

#[cfg(test)]
mod tests {
    #[test]
    fn passes() {
        assert_eq!(super::add(1, 2), 3);
    }

    #[test]
    fn fails() {
        panic!(\"this test fails\");
    }

    #[test]
    #[ignore]
    fn skipped() {}

    #[test]
    fn forever() {
        std::thread::sleep(std::time::Duration::from_secs(120));
    }
}
";

fn tests_fixture() -> Fixture {
    Fixture::new(
        "fx-tests",
        "2021",
        TESTS_MANIFEST,
        &[
            ("src/lib.rs", TESTS_LIB),
            (
                "src/bin/my-tool.rs",
                "fn main() {}\n\n#[test]\nfn bin_case() {}\n",
            ),
            (
                "tests/it.rs",
                "#[test]\nfn same() {\n    panic!(\"the integration test must not run\");\n}\n",
            ),
            (
                "tests/parse-errors.rs",
                "#[test]\nfn case() {}\n\nmod inner {\n    #[test]\n    fn deep() {}\n}\n\npub fn helper() {}\n",
            ),
            (
                "tests/no_harness.rs",
                "fn main() {}\n\n#[test]\nfn raw() {}\n",
            ),
            ("tests/needs_feature.rs", "#[test]\nfn gated() {}\n"),
            ("benches/speed.rs", "#[test]\nfn in_bench() {}\n"),
        ],
    )
}

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

#[test]
fn a_passing_test_passes_on_one_listed_entry_and_one_passed() {
    let fixture = tests_fixture();
    let evidence = fixture.run("tests", "fx_tests::tests::passes");
    passed(&evidence);
    assert_eq!(evidence.runs().len(), 2);
    assert_eq!(
        evidence.runs()[0].argv(),
        argv(&[
            "test",
            "--frozen",
            "--package=fx-tests",
            "--lib",
            "--",
            "--list",
            "--exact",
            "--",
            "tests::passes"
        ])
    );
    assert_eq!(
        evidence.runs()[1].argv(),
        argv(&[
            "test",
            "--frozen",
            "--package=fx-tests",
            "--lib",
            "--",
            "--exact",
            "--",
            "tests::passes"
        ])
    );
    let proof = evidence.proof();
    assert_eq!(
        (proof.listed, proof.passed, proof.failed, proof.ignored),
        (Some(1), Some(1), Some(0), Some(0))
    );
    assert!(evidence.wall() > Duration::ZERO);
}

#[test]
fn a_failing_test_fails_the_binding() {
    let fixture = tests_fixture();
    let evidence = fixture.run("tests", "fx_tests::tests::fails");
    let reason = failed(&evidence);
    assert!(reason.contains("1 failed"), "{reason}");
    assert_eq!(evidence.proof().failed, Some(1));
    // The neighbour that passes shares the target and the build.
    passed(&fixture.run("tests", "fx_tests::tests::passes"));
}

#[test]
fn an_ignored_test_is_listed_and_refused() {
    let fixture = tests_fixture();
    let evidence = fixture.run("tests", "fx_tests::tests::skipped");
    assert!(
        matches!(unresolved(&evidence), Refusal::TestIgnored { .. }),
        "{evidence}"
    );
    assert_eq!(evidence.proof().listed, Some(1));
    assert_eq!(evidence.proof().ignored, Some(1));
    assert_eq!(evidence.proof().passed, Some(0));
}

#[test]
fn a_test_behind_a_disabled_feature_lists_nothing_and_is_refused() {
    let fixture = tests_fixture();
    let evidence = fixture.run("tests", "fx_tests::gated");
    assert!(
        matches!(unresolved(&evidence), Refusal::NoTestListed { .. }),
        "{evidence}"
    );
    assert_eq!(evidence.proof().listed, Some(0));
    // Only the list pass ran: exit status 0 proved nothing.
    assert_eq!(evidence.runs().len(), 1);
}

#[test]
fn the_same_name_in_two_targets_runs_only_in_the_pinned_one() {
    let fixture = tests_fixture();
    // The library's `same` passes; the integration test's `same` panics. Each
    // binding runs its own target and never the other's.
    let library = fixture.run("tests", "fx_tests::same");
    passed(&library);
    assert_eq!(library.proof().listed, Some(1));
    assert!(library.runs()[0].argv().contains(&"--lib".to_string()));

    let integration = fixture.run("tests", "fx_tests::test_crate::it::same");
    let reason = failed(&integration);
    assert!(reason.contains("1 failed"), "{reason}");
    assert!(
        integration.runs()[0]
            .argv()
            .contains(&"--test=it".to_string())
    );
}

#[test]
fn an_integration_test_with_a_hyphen_runs_under_its_cargo_name() {
    let fixture = tests_fixture();
    let evidence = fixture.run("tests", "fx_tests::test_crate::parse_errors::case");
    passed(&evidence);
    assert!(
        evidence.runs()[1]
            .argv()
            .contains(&"--test=parse-errors".to_string()),
        "{evidence}"
    );
    let nested = fixture.run("tests", "fx_tests::test_crate::parse_errors::inner::deep");
    passed(&nested);
}

#[test]
fn a_test_of_a_binary_runs_under_bin() {
    let fixture = tests_fixture();
    let evidence = fixture.run("tests", "fx_tests::bin_crate::my_tool::bin_case");
    passed(&evidence);
    assert!(
        evidence.runs()[1]
            .argv()
            .contains(&"--bin=my-tool".to_string()),
        "{evidence}"
    );
}

#[test]
fn a_function_that_is_not_a_test_is_refused_before_any_process() {
    let fixture = tests_fixture();
    let refusal = fixture.refused("tests", "fx_tests::plain");
    assert!(matches!(refusal, Refusal::NotATest { .. }), "{refusal:?}");
    let refusal = fixture.refused("tests", "fx_tests::test_crate::parse_errors::helper");
    assert!(matches!(refusal, Refusal::NotATest { .. }), "{refusal:?}");
    // The test next to it runs.
    passed(&fixture.run("tests", "fx_tests::tests::passes"));
}

#[test]
fn targets_cargo_cannot_select_a_test_in_are_refused_before_any_process() {
    let fixture = tests_fixture();
    let refusal = fixture.refused("tests", "fx_tests::test_crate::no_harness::raw");
    assert!(matches!(refusal, Refusal::NoHarness { .. }), "{refusal:?}");
    assert!(refusal.to_string().contains("no-harness"));

    let refusal = fixture.refused("tests", "fx_tests::test_crate::needs_feature::gated");
    assert!(
        matches!(refusal, Refusal::RequiredFeatures { .. }),
        "{refusal:?}"
    );
    assert!(refusal.to_string().contains("needs-feature"));

    let refusal = fixture.refused("tests", "fx_tests::bench_crate::speed::in_bench");
    assert!(matches!(refusal, Refusal::TargetKind { .. }), "{refusal:?}");
    assert!(refusal.to_string().contains("speed"));

    passed(&fixture.run("tests", "fx_tests::test_crate::parse_errors::case"));
}

#[test]
fn hostile_bindings_are_unresolved_and_never_reach_cargo() {
    let fixture = tests_fixture();
    for hostile in [
        "-Zunstable-options",
        "--include-ignored",
        "--exact",
        "fx_tests::a;b",
        "fx_tests::a b",
        "fx_tests::tests::passes --exact",
    ] {
        for list in ["tests", "examples"] {
            let doc = capability(list, hostile);
            let docs = MemoryTree::from([("docs/capabilities/fixture.md", doc.as_str())]);
            let plan = Workplan::load(&docs, &fixture.source);
            let runner = fixture.runner_without_cargo();
            let executor = Executor::new(&fixture.source, &runner).unwrap();
            for binding in plan.graph().bindings() {
                let evidence = executor
                    .run(binding)
                    .unwrap_or_else(|error| panic!("{hostile:?} reached cargo: {error}"));
                assert!(
                    matches!(unresolved(&evidence), Refusal::Unresolved { .. }),
                    "{hostile:?}: {evidence}"
                );
                assert_eq!(evidence.runs().len(), 0);
            }
        }
    }
}

#[test]
fn a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran() {
    let fixture = tests_fixture();
    // Build the test binary first, so the limit measures the run.
    fixture.warm(&["test", "--no-run"]);
    let limit = Duration::from_secs(5);
    let limits = (
        Limits::build().with_timeout(limit),
        Limits::example().with_timeout(limit),
    );
    let started = Instant::now();
    let evidence = fixture
        .judge(
            &fixture.runner(),
            "tests",
            "fx_tests::tests::forever",
            Some(limits),
        )
        .expect("a kill is a verdict");
    let waited = started.elapsed();
    let reason = failed(&evidence);
    assert!(
        reason.contains("killed") && reason.contains("5 s limit"),
        "{reason}"
    );
    assert_eq!(
        evidence.runs().last().map(raf_workplan::bound::Run::ended),
        Some(Ended::TimedOut)
    );
    assert!(evidence.wall() >= limit, "{:?}", evidence.wall());
    assert!(waited >= limit && waited < limit * 12, "{waited:?}");
    // The same fixture, with the limit of the gate, still passes its neighbour.
    passed(&fixture.run("tests", "fx_tests::tests::passes"));
}

// ------------------------------------------------------- documentation tests

fn docs_lib(krate: &str) -> String {
    format!(
        "\
//! Fixture library.

/// Good.
///
/// ```
/// assert_eq!(2 + 2, 4);
/// ```
pub fn good() {{}}

/// Bad.
///
/// ```
/// assert_eq!(2 + 2, 5);
/// ```
pub fn bad() {{}}

/// Built and not run.
///
/// ```no_run
/// loop {{}}
/// ```
pub fn norun() {{}}

/// Passes when it does not build.
///
/// ```compile_fail
/// let number: u32 = \"text\";
/// ```
pub fn nofail() {{}}

/// Ignored.
///
/// ```ignore
/// assert!(false);
/// ```
pub fn skipped() {{}}

/// Documented and without an example.
pub fn bare() {{}}

/// A wrapper.
pub struct Wrapper<T>(pub T);

impl<T> Wrapper<T> {{
    /// Gets the value.
    ///
    /// ```
    /// assert_eq!({krate}::Wrapper(1).get(), &1);
    /// ```
    pub fn get(&self) -> &T {{
        &self.0
    }}
}}

/// Goes.
pub mod inner {{
    /// Passes.
    ///
    /// ```
    /// assert!(true);
    /// ```
    pub fn foo() {{}}

    /// Fails, and its item path starts with the one of `foo`.
    ///
    /// ```
    /// assert!(false);
    /// ```
    pub fn foo_more() {{}}

    /// Passes, with a name that no other starts with.
    ///
    /// ```
    /// assert!(true);
    /// ```
    pub fn solo() {{}}
}}

mod more;
"
    )
}

/// An impl of `Wrapper` in a file of its own, whose methods rustdoc names by
/// the module of that file.
const DOCS_MORE: &str = "\
//! More of the wrapper.

use crate::Wrapper;

impl<T> Wrapper<T> {
    /// Gets the value again.
    ///
    /// ```
    /// assert_eq!(fx_docs::Wrapper(2).again(), &2);
    /// ```
    pub fn again(&self) -> &T {
        &self.0
    }
}
";

fn docs_fixture(edition: &str) -> Fixture {
    Fixture::new(
        "fx-docs",
        edition,
        "",
        &[
            ("src/lib.rs", &docs_lib("fx_docs")),
            ("src/more.rs", DOCS_MORE),
        ],
    )
}

/// The documentation-test rules, under one edition.
fn doc_rules(edition: &str) {
    let fixture = docs_fixture(edition);

    let good = fixture.run("examples", "fx_docs::good");
    passed(&good);
    assert_eq!(
        good.runs()[0].argv(),
        argv(&[
            "test",
            "--frozen",
            "--package=fx-docs",
            "--doc",
            "--",
            "--list"
        ])
    );
    assert_eq!(
        good.runs()[1].argv(),
        argv(&[
            "test",
            "--frozen",
            "--package=fx-docs",
            "--doc",
            "--",
            "good"
        ])
    );
    assert_eq!(good.proof().selected, Some(1));
    assert_eq!(good.proof().ok_lines, Some(1));

    let wrapper = fixture.run("examples", "fx_docs::Wrapper::get");
    passed(&wrapper);
    assert_eq!(
        wrapper.runs()[1].argv(),
        argv(&[
            "test",
            "--frozen",
            "--package=fx-docs",
            "--doc",
            "--",
            "Wrapper<T>::get"
        ])
    );

    passed(&fixture.run("examples", "fx_docs::inner::solo"));

    // A method whose impl block sits in another file is named by the module of
    // that file, as rustdoc names it.
    let again = fixture.run("examples", "fx_docs::Wrapper::again");
    passed(&again);
    assert_eq!(
        again.runs()[1].argv(),
        argv(&[
            "test",
            "--frozen",
            "--package=fx-docs",
            "--doc",
            "--",
            "more::Wrapper<T>::again"
        ])
    );

    let bad = fixture.run("examples", "fx_docs::bad");
    assert!(
        matches!(bad.outcome(), BoundOutcome::Failed { .. }),
        "{bad}"
    );

    let norun = fixture.run("examples", "fx_docs::norun");
    let refusal = unresolved(&norun);
    assert!(matches!(refusal, Refusal::DocNotRunnable { .. }), "{norun}");
    assert!(refusal.to_string().contains("bind a runnable doc test"));

    let nofail = fixture.run("examples", "fx_docs::nofail");
    let refusal = unresolved(&nofail);
    assert!(
        matches!(refusal, Refusal::DocNotRunnable { .. }),
        "{nofail}"
    );
    assert!(refusal.to_string().contains("bind a runnable doc test"));

    let skipped = fixture.run("examples", "fx_docs::skipped");
    assert!(
        matches!(unresolved(&skipped), Refusal::DocNotRun { .. }),
        "{skipped}"
    );

    // The substring `inner::foo` also matches the failing `inner::foo_more`:
    // its failure fails the exit status and refuses the binding.
    let sibling = fixture.run("examples", "fx_docs::inner::foo");
    assert!(
        matches!(sibling.outcome(), BoundOutcome::Failed { .. }),
        "{sibling}"
    );
    assert_eq!(sibling.proof().ok_lines, Some(1), "{sibling}");

    // A documented item with no documentation test selects nothing.
    let bare = fixture.run("examples", "fx_docs::bare");
    assert!(
        matches!(unresolved(&bare), Refusal::NoDocTest { .. }),
        "{bare}"
    );
    assert_eq!(bare.runs().len(), 1);
    fixture.within_budget();
}

#[test]
fn documentation_tests_follow_the_rules_in_edition_2021() {
    doc_rules("2021");
}

#[test]
fn documentation_tests_follow_the_rules_in_edition_2024() {
    doc_rules("2024");
}

#[test]
fn an_item_path_that_begins_with_a_hyphen_never_reaches_cargo() {
    let fixture = docs_fixture("2021");
    let doc = capability("examples", "-fx_docs::good");
    let docs = MemoryTree::from([("docs/capabilities/fixture.md", doc.as_str())]);
    let plan = Workplan::load(&docs, &fixture.source);
    let runner = fixture.runner_without_cargo();
    let evidence = Executor::new(&fixture.source, &runner)
        .unwrap()
        .run(&plan.graph().bindings()[0])
        .unwrap();
    assert!(matches!(unresolved(&evidence), Refusal::Unresolved { .. }));
}

// ----------------------------------------------------------------- examples

const EXAMPLES_MANIFEST: &str = "";

fn examples_fixture() -> Fixture {
    Fixture::new(
        "fx-examples",
        "2021",
        EXAMPLES_MANIFEST,
        &[
            ("src/lib.rs", "//! Fixture library.\n"),
            ("examples/pass.rs", "fn main() {}\n"),
            (
                "examples/fail.rs",
                "fn main() {\n    eprintln!(\"the example fails\");\n    std::process::exit(3);\n}\n",
            ),
            ("examples/hyphen-ok.rs", "fn main() {}\n"),
            (
                "examples/stdin-closed.rs",
                "use std::io::Read;\n\nfn main() {\n    let mut text = String::new();\n    let read = std::io::stdin().read_to_string(&mut text).expect(\"stdin reads\");\n    std::process::exit(if read == 0 { 0 } else { 7 });\n}\n",
            ),
            (
                "examples/forever.rs",
                "fn main() {\n    std::thread::sleep(std::time::Duration::from_secs(120));\n}\n",
            ),
        ],
    )
}

#[test]
fn an_example_that_exits_zero_passes_and_one_that_does_not_fails() {
    let fixture = examples_fixture();
    let pass = fixture.run("examples", "fx_examples::example_crate::pass");
    passed(&pass);
    assert_eq!(
        pass.runs()[0].argv(),
        argv(&["run", "--frozen", "--package=fx-examples", "--example=pass"])
    );
    assert_eq!(pass.runs().len(), 1);

    let fail = fixture.run("examples", "fx_examples::example_crate::fail");
    let reason = failed(&fail);
    assert!(
        reason.contains("exited with code 3") && reason.contains("the example fails"),
        "{reason}"
    );
}

#[test]
fn a_hyphenated_example_runs_under_its_cargo_name() {
    let fixture = examples_fixture();
    let evidence = fixture.run("examples", "fx_examples::example_crate::hyphen_ok");
    passed(&evidence);
    assert_eq!(
        evidence.runs()[0].argv(),
        argv(&[
            "run",
            "--frozen",
            "--package=fx-examples",
            "--example=hyphen-ok"
        ])
    );
}

#[test]
fn an_example_that_reads_standard_input_sees_it_closed() {
    let fixture = examples_fixture();
    let evidence = fixture.run("examples", "fx_examples::example_crate::stdin_closed");
    passed(&evidence);
}

#[test]
fn an_example_that_never_ends_is_killed_and_reported_with_the_time_it_ran() {
    let fixture = examples_fixture();
    fixture.warm(&["build", "--examples"]);
    let limit = Duration::from_secs(5);
    let limits = (
        Limits::build().with_timeout(limit),
        Limits::example().with_timeout(limit),
    );
    let started = Instant::now();
    let evidence = fixture
        .judge(
            &fixture.runner(),
            "examples",
            "fx_examples::example_crate::forever",
            Some(limits),
        )
        .expect("a kill is a verdict");
    let waited = started.elapsed();
    let reason = failed(&evidence);
    assert!(
        reason.contains("killed") && reason.contains("5 s limit"),
        "{reason}"
    );
    assert_eq!(
        evidence.runs().last().map(raf_workplan::bound::Run::ended),
        Some(Ended::TimedOut)
    );
    assert!(waited >= limit && waited < limit * 12, "{waited:?}");
    // The neighbour passes under the same limits.
    let neighbour = fixture
        .judge(
            &fixture.runner(),
            "examples",
            "fx_examples::example_crate::pass",
            Some(limits),
        )
        .expect("the neighbour runs");
    passed(&neighbour);
    fixture.within_budget();
}

#[test]
fn a_process_that_cannot_start_is_an_error_and_not_a_verdict() {
    let fixture = examples_fixture();
    let result = fixture.judge(
        &fixture.runner_without_cargo(),
        "examples",
        "fx_examples::example_crate::pass",
        None,
    );
    assert!(matches!(result, Err(ExecError::Spawn { .. })), "{result:?}");
}
