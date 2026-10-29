// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `Lifecycle::complete`: the six conditions, the check of the repository
//! without the record, the order of the steps and the order of the writes.
//!
//! Two kinds of fixture are used. A real one is a generated repository in a
//! scratch directory: a crate with its own `[workspace]`, a template
//! `Cargo.lock`, no dependency and a pinned edition, the capability documents
//! that bind it, a change record, and a git history with a remote-tracking
//! default branch; the processes are the real `git` and the `cargo` that started
//! this test binary. Each builds into a target directory of its own, beside it
//! and removed with it. A scripted one has the same files and the same
//! documents but answers every process from a [`FakeRunner`], on every
//! platform, and records the calls, so the order of the steps and the processes
//! a refusal spends can be read from them.

mod support;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use raf_okf::{Actor, AssessmentVerdict};
use raf_source::RelPath;
use raf_workplan::bound::BoundOutcome;
use raf_workplan::exec::{
    Call, ExecError, FakeRunner, Invocation, Limits, Output, Program, Programs, Runner,
    SystemRunner,
};
use raf_workplan::lifecycle::{Completion, Condition, Failure, Lifecycle, LifecycleError, Side};
use raf_workplan::obligation::ObligationKind;
use raf_workplan::{ChangeLookupError, Gate, SyncOptions, Workplan};
use support::assessed::{assess_on_disk, assess_on_disk_as};
use support::changes::{DONE, capability, record};
use support::checkout::git;
use support::disk::{Scratch, put, read, snapshot};
use support::evidence::{Evidence, digest_of};

const CAP: &str = "docs/capabilities/fixture.md";
/// The manifest of the fixture package, a workspace of its own.
const MANIFEST: &str =
    "[package]\nname = \"fx-life\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n";
const INDEX: &str = "docs/capabilities/index.md";
const RECORD: &str = "work/changes/shape.md";

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
}
";

const GOOD_EXAMPLE: &str = "fn main() {\n    println!(\"fine\");\n}\n";
const BROKEN_EXAMPLE: &str = "fn main() {\n    let number: u32 = \"text\";\n    drop(number);\n}\n";
const EXITING_EXAMPLE: &str = "fn main() {\n    std::process::exit(3);\n}\n";

const PASSES: &str = "[fx_life::tests::passes]";

/// The evidence record of the trace cases, and the claim on it.
const EVIDENCE: &str = "work/evidence/bound.md";
const CLAIM: &str = "\nThe recorded bound execution supports the fixture's addition behavior.\n<!-- raf:claim evidence=bound -->\n<!-- raf:tested-by fx_life::tests::passes -->\n";

/// The digest of `fx_life::add` in the library `lib`.
fn add_digest(lib: &str) -> String {
    let source = raf_source::Workspace::load(
        &raf_source::MemoryTree::from([("Cargo.toml", MANIFEST), ("src/lib.rs", lib)]),
        &["."],
    )
    .expect("the fixture library loads");
    digest_of(&source, "fx_life::add")
}

/// The library with `add` edited.
fn edited_lib() -> String {
    LIB.replace("    a + b\n", "    b + a\n")
}

/// A passing record for `fixture`, made against `add` as `LIB` has it.
fn evidence() -> Evidence {
    Evidence::passing("bound", "[cap:fixture]")
        .inputs(vec![("fx_life::add".to_string(), add_digest(LIB))])
}

/// An item with an outcome, and one without.
const NO_OUTCOME: &str = "  - claim: \"fixture is shaped\"\n";

/// A candidate contract of the change `shape`.
const PROPOSAL: &str = "\n> [!PROPOSED change:shape]\n> A candidate contract.\n";

/// What a fixture holds: the capability `fixture`, the change `shape` that lists
/// it, and what else is written beside them.
#[derive(Clone)]
struct Spec {
    state: &'static str,
    code: &'static str,
    tests: &'static str,
    examples: &'static str,
    /// Text added to the end of a section of the capability, by section name.
    extra: Vec<(&'static str, &'static str)>,
    effect: &'static str,
    affects: Vec<&'static str>,
    items: &'static str,
    /// The example targets, by name.
    example_files: Vec<(&'static str, &'static str)>,
    /// Other documents, written before the sync.
    docs: Vec<(String, String)>,
    /// The library `src/lib.rs`.
    lib: String,
    /// Other source files of the package, by path.
    src: Vec<(String, String)>,
    /// The coverage record.
    coverage: String,
}

impl Default for Spec {
    /// A capability that completes: `delivered`, bound to code and one passing
    /// test, and a record with the one item resolved and the effect `none`.
    fn default() -> Self {
        Self {
            state: "delivered",
            code: "[fx_life::add]",
            tests: PASSES,
            examples: "[fx_life::add]",
            extra: Vec::new(),
            effect: "none",
            affects: vec!["cap:fixture#contract"],
            items: DONE,
            example_files: Vec::new(),
            docs: Vec::new(),
            lib: LIB.to_string(),
            src: Vec::new(),
            coverage: strict_coverage(),
        }
    }
}

impl Spec {
    fn state(mut self, state: &'static str) -> Self {
        self.state = state;
        self
    }

    fn tests(mut self, tests: &'static str) -> Self {
        self.tests = tests;
        self
    }

    fn examples(mut self, examples: &'static str, files: &[(&'static str, &'static str)]) -> Self {
        self.examples = examples;
        self.example_files = files.to_vec();
        self
    }

    fn extra(mut self, section: &'static str, text: &'static str) -> Self {
        self.extra.push((section, text));
        self
    }

    fn effect(mut self, effect: &'static str, affects: &[&'static str]) -> Self {
        self.effect = effect;
        self.affects = affects.to_vec();
        self
    }

    fn items(mut self, items: &'static str) -> Self {
        self.items = items;
        self
    }

    fn doc(mut self, path: &str, text: String) -> Self {
        self.docs.push((path.to_string(), text));
        self
    }

    /// The documents: the capability, the record, and the others.
    fn documents(&self) -> Vec<(String, String)> {
        // These executable fixture guarantees have proof ownership while
        // implementing too, so promotion does not invent their markers.
        let template_state = if self.state == "implementing" && self.tests != "[]" {
            "delivered"
        } else {
            self.state
        };
        let mut files = vec![
            (
                CAP.to_string(),
                capability(
                    "fixture",
                    template_state,
                    [self.code, self.tests, self.examples],
                    &self.extra,
                )
                .replacen(
                    &format!("state: {template_state}"),
                    &format!("state: {}", self.state),
                    1,
                ),
            ),
            (
                RECORD.to_string(),
                record(
                    "shape",
                    "[cap:fixture]",
                    &self.affects,
                    self.effect,
                    self.items,
                ),
            ),
        ];
        files.extend(self.docs.clone());
        files
    }
}

/// A second capability that is `designed`, listed by an open record `other`,
/// so that a record that moves an item to `other` names a change that exists
/// and the state rules are satisfied beside it.
fn other_record(spec: Spec) -> Spec {
    spec.doc(
        "docs/capabilities/second.md",
        capability("second", "designed", ["[]", "[]", "[]"], &[]),
    )
    .doc(
        "work/changes/other.md",
        record("other", "[cap:second]", &["cap:second#purpose"], "none", ""),
    )
}

// --- a capability that strict mode accepts -----------------------------------------

/// Twenty words of prose for a section that holds nothing else.
const PROSE: &str = "\nThis section of the fixture states one plain fact in twenty words so that strict mode reads it whole.\n";

/// The Contract: the opening and one bold-lead clause, each covered.
const CONTRACT: &str = "\n**Adds.** The function adds its two numbers and returns their sum, which the bound test and the documentation test both show.\n<!-- raf:tested-by fx_life::tests::passes -->\n";

/// The Usage: the bound documentation test named in a code span.
const USAGE: &str = "\nCall `fx_life::add` to add two numbers; its documentation test shows the call and the sum it returns.\n";

/// The Constraints: the opening paragraph and one limitation, each covered.
const CONSTRAINTS: &str = "\n- One limitation of the fixture, stated as a list item so that the complete policy finds a limitation in it.\n<!-- raf:tested-by fx_life::tests::passes -->\n";

/// The coverage record of the strict-ready fixture, after its `id` line: the
/// public helper and the unbound tests are excluded, so the package is fully
/// explained, and excepted from the orphan lines with the same reasons.
const EXCLUSIONS: &str = "scope: []\nexclusions:\n  - item: fx_life::plain\n    reason: \"A helper of the fixtures that no guarantee needs.\"\n  - item: \"fx_life::tests::*\"\n    reason: \"The tests of the fixtures, of which one is bound.\"\nexceptions:\n  - item: fx_life::plain\n    reason: \"A helper of the fixtures that no guarantee needs.\"\n  - item: \"fx_life::tests::*\"\n    reason: \"The tests of the fixtures, of which one is bound.\"\n";

/// [`strict_ready`] with a ceiling of ten orphan lines for the package, so a
/// private item that nothing reaches, which these fixtures add to show what
/// the touch rule leaves alone, is no finding of the line report.
fn roomy(mut spec: Spec) -> Spec {
    let keys = spec
        .coverage
        .strip_suffix("---\n")
        .expect("the record ends with its fence")
        .to_string();
    spec.coverage = format!("{keys}ceilings:\n  - crate: fx-life\n    lines: 10\n---\n");
    spec
}

/// Explicit coverage ownership for the completion fixture.
fn strict_coverage() -> String {
    format!("---\ntype: Coverage\nid: coverage:default\n{EXCLUSIONS}---\n")
}

/// The fixture with a delivered capability that every rule of strict mode
/// accepts: it binds the documentation test of `fx_life::add` as its example
/// and names it in Usage, each section holds twenty words of prose, each claim
/// has a tested-by marker, Constraints lists a limitation, Verification holds
/// the matrix, and the package has explicit fixture coverage ownership.
fn strict_ready() -> Spec {
    let mut spec = Spec::default()
        .examples("[fx_life::add]", &[])
        .extra("Purpose", PROSE)
        .extra("Contract", CONTRACT)
        .extra("Usage", USAGE)
        .extra("Examples", PROSE)
        .extra("Constraints", CONSTRAINTS)
        .extra("Explanation", PROSE)
        .extra("Verification", PROSE);
    spec.coverage = strict_coverage();
    spec
}

/// An item dropped without a reason, and the same item with one.
const DROPPED: &str = "  - claim: \"fixture is shaped\"\n    outcome: dropped\n";
const DROPPED_WITH_REASON: &str = "  - claim: \"fixture is shaped\"\n    outcome: dropped\n    reason: \"The shape needs no work in this change.\"\n";

/// A generated repository, removed when dropped.
struct Fixture {
    scratch: Scratch,
    real: bool,
}

impl Fixture {
    /// A real fixture: a git history whose `feature` branch forks from `main`
    /// at the commit that holds everything, so nothing is touched since the
    /// merge base until a test edits a file.
    fn real(spec: &Spec) -> Self {
        Self::build(spec, true)
    }

    /// A fixture for a scripted runner: the same files, no git history.
    fn scripted(spec: &Spec) -> Self {
        Self::build(spec, false)
    }

    fn build(spec: &Spec, real: bool) -> Self {
        let scratch = Scratch::new("complete");
        let root = scratch.path().to_path_buf();
        let root = root.as_path();
        put(root, "Cargo.toml", MANIFEST);
        put(
            root,
            "Cargo.lock",
            "version = 4\n\n[[package]]\nname = \"fx-life\"\nversion = \"0.1.0\"\n",
        );
        put(root, ".cargo/config.toml", &scratch.cargo_config());
        put(root, "src/lib.rs", &spec.lib);
        for (path, text) in &spec.src {
            put(root, path, text);
        }
        for (name, text) in &spec.example_files {
            put(root, &format!("examples/{name}.rs"), text);
        }
        for (path, text) in spec.documents() {
            put(root, &path, &text);
        }
        put(root, support::COVERAGE, &spec.coverage);
        if real {
            git(root, &["init", "-q", "-b", "main"]);
        }
        let fixture = Self { scratch, real };
        fixture.resync();
        // The sections the change affects are assessed as they stand, so the
        // gate is judged on what each fixture is about.
        fixture.assess();
        if real {
            git(root, &["add", "-A"]);
            git(root, &["commit", "-q", "-m", "base"]);
            git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
            git(
                root,
                &[
                    "symbolic-ref",
                    "refs/remotes/origin/HEAD",
                    "refs/remotes/origin/main",
                ],
            );
            git(root, &["checkout", "-q", "-b", "feature"]);
        }
        fixture
    }

    fn root(&self) -> &Path {
        self.scratch.path()
    }

    /// Assess the sections the change affects as they stand now, as an author
    /// does after an edit: through the shared helper and the real
    /// `record_assessment`, replacing the assessments that are there.
    fn assess(&self) {
        assess_on_disk(self.root(), "shape");
    }

    /// Run `docs sync` over the documents as they are and write what it returns.
    fn resync(&self) {
        let opened = Gate::workspace(self.root())
            .programs(self.programs())
            .tracked_paths(Vec::new())
            .open()
            .expect("the fixture opens");
        let report = opened
            .workplan()
            .sync(&SyncOptions::new(
                "2026 Example Org <dev@example.org>",
                "CC-BY-4.0",
            ))
            .unwrap_or_else(|error| {
                panic!(
                    "the fixture syncs: {error}; findings: {:#?}",
                    opened.workplan().check()
                )
            });
        Workplan::write(&report, self.root()).expect("the sync is written");
    }

    /// The text of a file.
    fn text(&self, path: &str) -> String {
        read(self.root(), path)
    }

    /// Replace the first `from` in the file `path` by `to`.
    fn edit(&self, path: &str, from: &str, to: &str) {
        let text = self.text(path);
        assert!(text.contains(from), "{path} has no `{from}`");
        put(self.root(), path, &text.replacen(from, to, 1));
    }

    /// Rebind the fixture tests, reauthor their claim markers and assess the
    /// affected sections; frontmatter alone would leave stale proof ownership.
    fn rebind_and_assess_tests(&self, from: &str, to: &str) {
        self.edit(CAP, from, to);
        let paths = |value: &str| {
            value
                .trim_matches(['[', ']'])
                .split(',')
                .map(str::trim)
                .collect::<Vec<_>>()
                .join(",")
        };
        let text = self.text(CAP).replace(
            &format!("<!-- raf:tested-by {} -->", paths(from)),
            &format!("<!-- raf:tested-by {} -->", paths(to)),
        );
        put(self.root(), CAP, &text);
        self.resync();
        self.assess();
    }

    fn exists(&self, path: &str) -> bool {
        self.root().join(path).exists()
    }

    /// Every file below the root except the git directory, with its bytes.
    fn snapshot(&self) -> BTreeMap<String, Vec<u8>> {
        snapshot(self.root())
            .into_iter()
            .filter(|(path, _)| !path.starts_with(".git/"))
            .collect()
    }

    /// The tracked paths of a scripted fixture: every file below the root.
    fn tracked(&self) -> Vec<RelPath> {
        self.snapshot()
            .keys()
            .map(|path| RelPath::new(path).expect("a path"))
            .collect()
    }

    /// The real programs: `git` and the `cargo` of this test binary.
    fn programs(&self) -> Programs {
        Programs::with_cargo_var(self.root(), Some(cargo().into())).discover_git()
    }

    /// Programs whose `cargo` does not exist: a `cargo` process is an error.
    fn programs_without_cargo(&self) -> Programs {
        self.programs()
            .with_cargo(self.root().join("no-such-cargo"))
    }

    fn lifecycle(&self) -> Lifecycle {
        Lifecycle::open(self.programs()).unwrap_or_else(|error| panic!("opens: {error}"))
    }

    /// The gate of a scripted fixture, which asks `git` nothing.
    fn gate(&self) -> Gate {
        Gate::workspace(self.root())
            .programs(self.programs())
            .tracked_paths(self.tracked())
    }

    /// A lifecycle over a scripted runner.
    fn over<R: Runner>(&self, runner: R) -> Lifecycle<R> {
        assert!(!self.real, "a scripted lifecycle is for a scripted fixture");
        Lifecycle::open_with(&self.gate(), runner)
            .unwrap_or_else(|error| panic!("the fixture opens: {error}"))
    }

    fn complete(&self) -> Completion {
        self.lifecycle()
            .complete("shape")
            .unwrap_or_else(|error| panic!("complete should answer: {error}"))
    }
}

/// The refusal of a completion.
fn refusal(completion: &Completion) -> &[Failure] {
    assert!(!completion.is_completed(), "{completion}");
    completion.failures()
}

/// The kinds of the obligations that stand as failures, without repeats.
fn kinds(completion: &Completion) -> Vec<ObligationKind> {
    let mut kinds: Vec<ObligationKind> = completion.obligations().map(|o| o.kind).collect();
    kinds.dedup();
    kinds
}

fn count(completion: &Completion, condition: Condition) -> usize {
    completion.of_condition(condition).count()
}

// --- a scripted runner ------------------------------------------------------------

fn said(text: &str) -> Output {
    Output::new(Some(0), text.as_bytes().to_vec(), Vec::new())
}

/// The answers of `git` to the two questions the open asks.
fn git_answers() -> FakeRunner {
    let runner = FakeRunner::new()
        .with_view_check(|view| {
            use raf_source::Tree as _;
            let request = view.request();
            if view
                .entry(request.removed())
                .map_err(|error| error.to_string())?
                .is_some()
            {
                return Err("completed virtual model still contains the selected record".into());
            }
            for (path, text) in request.rewrites() {
                if view
                    .read(path, u64::MAX)
                    .map_err(|error| error.to_string())?
                    != *text
                {
                    return Err(format!("virtual generated replacement differs at {path}"));
                }
            }
            Ok(())
        })
        .on(
            Program::Git,
            &["--version"],
            Ok(said("git version 2.45.1\n")),
        );
    at_merge_base(runner, &[("Cargo.toml", MANIFEST), ("src/lib.rs", LIB)])
        .on(
            Program::Git,
            &["cat-file", "blob", &format!("{COMMIT}:{CAP}")],
            Ok(said(&Spec::default().documents()[0].1)),
        )
        .on(
            Program::Git,
            &["rev-parse", "--show-prefix"],
            Ok(said("\n")),
        )
}

/// A commit id of the scripted answers.
const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

/// `runner` with the answers of `git` to the three questions that lead to the
/// merge base, the listing of its tree, which holds `files`, and the text of
/// each of `files` at it.
fn at_merge_base(runner: FakeRunner, files: &[(&str, &str)]) -> FakeRunner {
    let mut listing = String::new();
    for (path, _) in files {
        listing.push_str("100644 blob ");
        listing.push_str(COMMIT);
        listing.push('\t');
        listing.push_str(path);
        listing.push('\0');
    }
    let mut runner = runner
        .on(
            Program::Git,
            &["symbolic-ref", "refs/remotes/origin/HEAD"],
            Ok(said("refs/remotes/origin/main\n")),
        )
        .on(
            Program::Git,
            &[
                "rev-parse",
                "--verify",
                "--end-of-options",
                "refs/remotes/origin/main^{commit}",
            ],
            Ok(said(&format!("{COMMIT}\n"))),
        )
        .on(
            Program::Git,
            &["merge-base", "HEAD", COMMIT],
            Ok(said(&format!("{COMMIT}\n"))),
        )
        .on(
            Program::Git,
            &["ls-tree", "-r", "-z", "--full-tree", COMMIT],
            Ok(said(&listing)),
        );
    for (path, text) in files {
        runner = runner.on(
            Program::Git,
            &["cat-file", "blob", &format!("{COMMIT}:{path}")],
            Ok(said(text)),
        );
    }
    runner
}

/// The answers of `cargo` to the two passes of the test `fx_life::tests::<name>`.
fn passing(runner: FakeRunner, name: &str) -> FakeRunner {
    let test = format!("tests::{name}");
    runner
        .on_view(
            Program::Cargo,
            &["test", "--frozen", "--package=fx-life", "--lib", "--", "--list", "--exact", "--", &test],
            Ok(said(&format!("{test}: test\n\n1 test, 0 benchmarks\n"))),
        )
        .on_view(
            Program::Cargo,
            &["test", "--frozen", "--package=fx-life", "--lib", "--", "--exact", "--", &test],
            Ok(said(&format!(
                "running 1 test\ntest {test} ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
            ))),
        )
        .on_view(
            Program::Cargo,
            &["test", "--frozen", "--package=fx-life", "--doc", "--", "--list"],
            Ok(said("src/lib.rs - add (line 5): test\n\n1 test, 0 benchmarks\n")),
        )
        .on_view(
            Program::Cargo,
            &["test", "--frozen", "--package=fx-life", "--doc", "--", "add"],
            Ok(said("running 1 test\ntest src/lib.rs - add (line 5) ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n")),
        )
}

/// A runner that calls a function once, after the first `cargo` process ended.
struct AfterCargo<F: FnOnce()> {
    inner: FakeRunner,
    then: Mutex<Option<F>>,
}

impl<F: FnOnce()> AfterCargo<F> {
    fn new(inner: FakeRunner, then: F) -> Self {
        Self {
            inner,
            then: Mutex::new(Some(then)),
        }
    }

    fn commands(&self) -> Vec<String> {
        self.inner.calls().iter().map(Call::command).collect()
    }
}

impl<F: FnOnce()> Runner for AfterCargo<F> {
    fn prepare_view(
        &self,
        request: &raf_workplan::exec::ViewRequest,
    ) -> Result<raf_workplan::exec::PreparedView, ExecError> {
        self.inner.prepare_view(request)
    }
    fn revalidate_view(&self, view: &raf_workplan::exec::PreparedView) -> Result<(), ExecError> {
        self.inner.revalidate_view(view)
    }
    fn finish_view(&self, view: &raf_workplan::exec::PreparedView) -> Result<(), ExecError> {
        self.inner.finish_view(view)
    }

    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError> {
        let answer = self.inner.run(invocation, limits);
        if invocation.program() == Program::Cargo
            && let Some(then) = self.then.lock().expect("a lock").take()
        {
            then();
        }
        answer
    }
}

fn programs_of(commands: &[String]) -> Vec<&str> {
    commands
        .iter()
        .map(|command| command.split(' ').next().unwrap_or(""))
        .collect()
}

// --- the happy path ----------------------------------------------------------------

#[test]
fn a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal()
 {
    primary_cargo_configuration_is_not_duplicated();
    supplied_tracking_completes();
    let fixture = Fixture::real(&completed_view_spec());
    completed_view_inputs(&fixture);
    let original = fixture
        .lifecycle()
        .verify(Some("shape"))
        .expect("original-root verification answers");
    assert!(
        original
            .evidence()
            .iter()
            .any(|evidence| matches!(evidence.outcome(), BoundOutcome::Failed { .. })),
        "the real bound case must fail against the still-open original root"
    );
    let before = fixture.snapshot();
    let cap_before = fixture.text(CAP);
    assert!(cap_before.contains("| Changed by | [shape](../../work/changes/shape.md) |"));
    assert!(
        git(fixture.root(), &["status", "--porcelain"]).is_empty(),
        "the fixture is committed"
    );

    let completion = fixture.complete();
    assert_eq!(
        completion,
        Completion::Completed {
            evidence: completion.evidence().to_vec(),
            removed: RECORD.to_string(),
            rewritten: vec![CAP.to_string()],
        }
    );
    assert_eq!(
        completion.to_string(),
        format!("completed: removed {RECORD}\nrewrote {CAP}\n")
    );

    // The record is gone, the relationship table lost its one row, and no other
    // byte of the tree moved.
    assert!(!fixture.exists(RECORD));
    assert_eq!(
        fixture.text(CAP),
        cap_before.replace(
            "| Changed by | [shape](../../work/changes/shape.md) |\n",
            ""
        )
    );
    for evidence in completion.evidence() {
        for run in evidence.runs() {
            let view = run
                .invocation()
                .view()
                .expect("every completed-view process is explicitly scoped");
            assert_eq!(view.logical_root(), fixture.root());
            let physical = view.physical_root().expect("real native materialization");
            assert_ne!(physical, fixture.root());
            assert!(!physical.exists(), "owned execution root was released");
            assert!(matches!(
                view.proof().index(),
                raf_workplan::exec::IndexProvenance::Native { .. }
            ));
        }
    }
    let after = fixture.snapshot();
    assert_eq!(
        support::disk::changed(&before, &after),
        [CAP.to_string(), RECORD.to_string()]
    );
    #[cfg(unix)]
    for path in ["absolute-link", ".private/absolute-link"] {
        assert_eq!(
            std::fs::read_link(fixture.root().join(path)).unwrap(),
            fixture.root().join(".private/input.bin")
        );
    }
    assert_eq!(
        git(fixture.root(), &["status", "--porcelain"]),
        format!(" M {CAP}\n D {RECORD}")
    );

    // The tree is clean for the gate, and a sync has nothing left to write.
    Gate::workspace(fixture.root())
        .programs(fixture.programs())
        .assert_clean();
    let opened = Gate::workspace(fixture.root())
        .programs(fixture.programs())
        .open()
        .expect("opens");
    assert!(
        opened
            .workplan()
            .sync(&SyncOptions::default())
            .expect("syncs")
            .is_empty()
    );
}

fn primary_cargo_configuration_is_not_duplicated() {
    let mut spec = Spec::default()
        .doc(".cargo/config.toml", "[build]\nrustflags = [\"--cfg=raf_view_marker\"]\n".to_string())
        .doc("build.rs", r#"fn main() {
            let flags = std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap();
            let count = flags.split('\u{1f}').filter(|flag| *flag == "--cfg=raf_view_marker").count();
            println!("cargo:rustc-env=RAF_CONFIG_COUNT={count}");
        }"#.to_string());
    spec.lib = LIB.replace(
        "        assert_eq!(super::add(1, 2), 3);",
        "        assert_eq!(env!(\"RAF_CONFIG_COUNT\"), \"1\");",
    );
    spec.code = "[fx_life::add, fx_life::build_script::main]";
    let fixture = Fixture::real(&spec);
    put(
        fixture.root(),
        ".cargo/config.toml",
        &format!(
            "{}\n[env]\nRAF_LOGICAL_ROOT = '{}'\n\n[target.'cfg(unix)']\nrunner = '{}'\n",
            fixture.text(".cargo/config.toml"),
            fixture.root().display(),
            fixture.root().join("native-runner").display()
        ),
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        put(
            fixture.root(),
            "native-runner",
            "#!/bin/sh\nexport RAF_CONFIG_RUNNER=checked\nexec \"$@\"\n",
        );
        std::fs::set_permissions(
            fixture.root().join("native-runner"),
            std::fs::Permissions::from_mode(0o751),
        )
        .unwrap();
    }
    std::fs::write(fixture.root().join("absolute-input.bin"), [0, 255, 13, 10]).unwrap();
    let nested = fixture.root().join("vendor/nested");
    std::fs::create_dir_all(&nested).unwrap();
    git(&nested, &["init", "-q", "-b", "main"]);
    std::fs::write(nested.join("note.txt"), "nested repository input\n").unwrap();
    git(&nested, &["add", "note.txt"]);
    git(&nested, &["commit", "-q", "-m", "nested source"]);
    fixture.edit(
        "src/lib.rs",
        "assert_eq!(env!(\"RAF_CONFIG_COUNT\"), \"1\");",
        r#"
        assert_eq!(env!("RAF_CONFIG_COUNT"), "1");
        let logical = std::path::Path::new(env!("RAF_LOGICAL_ROOT"));
        assert_eq!(std::env::current_dir().unwrap(), logical);
        assert_eq!(std::env::var("RAF_CONFIG_RUNNER").unwrap(), "checked");
        assert_eq!(std::fs::read(logical.join("absolute-input.bin")).unwrap(), [0, 255, 13, 10]);
        let top = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(logical).output().unwrap();
        assert!(top.status.success(), "{:?}", top);
        assert_eq!(String::from_utf8(top.stdout).unwrap().trim(), logical.to_str().unwrap());
        let nested = logical.join("vendor/nested");
        let nested_top = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(&nested).output().unwrap();
        assert!(nested_top.status.success(), "{:?}", nested_top);
        assert_eq!(String::from_utf8(nested_top.stdout).unwrap().trim(), nested.to_str().unwrap());
        let record = "work/changes/shape.md";
        let indexed = std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--", record])
            .current_dir(logical).output().unwrap();
        assert_eq!(indexed.status.success(), logical.join(record).exists());
    "#,
    );
    fixture.assess();
    let original = fixture.lifecycle().verify(Some("shape")).unwrap();
    assert!(
        original
            .evidence()
            .iter()
            .all(|row| matches!(row.outcome(), BoundOutcome::Passed)),
        "original configuration applies exactly once: {original:?}"
    );
    let index_before = std::fs::read(fixture.root().join(".git/index")).unwrap();
    let source_before = std::fs::read(fixture.root().join("src/lib.rs")).unwrap();
    let configuration_before = std::fs::read(fixture.root().join(".cargo/config.toml")).unwrap();
    let completion = fixture.complete();
    assert!(
        completion.is_completed(),
        "prepared configuration must apply exactly once: {completion:?}"
    );
    assert_eq!(
        std::fs::read(fixture.root().join(".git/index")).unwrap(),
        index_before
    );
    assert_eq!(
        std::fs::read(fixture.root().join("src/lib.rs")).unwrap(),
        source_before
    );
    assert_eq!(
        std::fs::read(fixture.root().join(".cargo/config.toml")).unwrap(),
        configuration_before
    );
}

fn supplied_tracking_completes() {
    let fixture = Fixture::scripted(&Spec::default());
    let gate = Gate::workspace(fixture.root()).programs(
        fixture
            .programs()
            .with_git(fixture.root().join("no-such-git")),
    );
    let runner = tracking_answers(&fixture);
    let before = fixture.snapshot();
    let cap_before = fixture.text(CAP);
    let lifecycle =
        Lifecycle::open_with(&gate, &runner).expect("supplied runner opens without real Git");
    assert_eq!(fixture.snapshot(), before);
    let commands: Vec<_> = runner.calls().iter().map(Call::command).collect();
    assert_eq!(
        commands,
        [
            "git --version",
            "git rev-parse --show-prefix",
            "git rev-parse --is-inside-work-tree",
            "git ls-files -z"
        ]
    );
    let completed = lifecycle
        .complete("shape")
        .expect("supplied runner also refreshes the repository");
    assert!(completed.is_completed(), "{completed}");
    assert_eq!(
        fixture.text(CAP),
        cap_before.replace(
            "| Changed by | [shape](../../work/changes/shape.md) |\n",
            ""
        )
    );
    assert_eq!(
        support::disk::changed(&before, &fixture.snapshot()),
        [CAP.to_string(), RECORD.to_string()]
    );
    let calls = runner.calls();
    let last_cargo = calls
        .iter()
        .rposition(|call| call.program() == Program::Cargo)
        .expect("bound runs reached the supplied runner");
    let last_listing = calls
        .iter()
        .rposition(|call| call.command() == "git ls-files -z")
        .expect("refreshed listing");
    assert!(
        last_listing > last_cargo,
        "refresh must follow the bound runs"
    );
    for command in ["git rev-parse --is-inside-work-tree", "git ls-files -z"] {
        assert_eq!(
            calls
                .iter()
                .filter(|call| call.command() == command)
                .count(),
            2,
            "initial and refreshed listing: {command}"
        );
    }
}

fn tracking_answers(fixture: &Fixture) -> FakeRunner {
    let listing = fixture
        .tracked()
        .iter()
        .fold(String::new(), |mut text, path| {
            text.push_str(path.as_str());
            text.push('\0');
            text
        });
    passing(git_answers(), "passes")
        .on(
            Program::Git,
            &["rev-parse", "--is-inside-work-tree"],
            Ok(said("true\n")),
        )
        .on(Program::Git, &["ls-files", "-z"], Ok(said(&listing)))
}

/// Refuse one actual tracked-list call, either during opening or refresh.
struct RefuseTracking {
    inner: FakeRunner,
    listings: std::cell::Cell<usize>,
    fail_at: usize,
    missing: bool,
}

impl Runner for RefuseTracking {
    fn prepare_view(
        &self,
        request: &raf_workplan::exec::ViewRequest,
    ) -> Result<raf_workplan::exec::PreparedView, ExecError> {
        self.inner.prepare_view(request)
    }
    fn revalidate_view(&self, view: &raf_workplan::exec::PreparedView) -> Result<(), ExecError> {
        self.inner.revalidate_view(view)
    }
    fn finish_view(&self, view: &raf_workplan::exec::PreparedView) -> Result<(), ExecError> {
        self.inner.finish_view(view)
    }

    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError> {
        if invocation == &Invocation::git_ls_files() {
            let count = self.listings.get() + 1;
            self.listings.set(count);
            if count == self.fail_at {
                return if self.missing {
                    FakeRunner::new().run(invocation, limits)
                } else {
                    Ok(Output::new(
                        Some(23),
                        Vec::new(),
                        b"scripted tracked-list refusal".to_vec(),
                    ))
                };
            }
        }
        self.inner.run(invocation, limits)
    }
}

fn refused_tracking_preserves_documents() {
    for fail_at in [1, 2] {
        for missing in [true, false] {
            let fixture = Fixture::scripted(&Spec::default());
            let gate = Gate::workspace(fixture.root()).programs(
                fixture
                    .programs()
                    .with_git(fixture.root().join("no-such-git")),
            );
            let runner = RefuseTracking {
                inner: tracking_answers(&fixture),
                listings: std::cell::Cell::new(0),
                fail_at,
                missing,
            };
            let before = fixture.snapshot();
            let result = Lifecycle::open_with(&gate, &runner)
                .and_then(|lifecycle| lifecycle.complete("shape"));
            let error =
                result.expect_err("supplied runner refusal propagates without another runner");
            let expected = if missing {
                "no scripted answer"
            } else {
                "scripted tracked-list refusal"
            };
            assert!(
                matches!(&error, LifecycleError::Gate(raf_workplan::GateError::Tracked { reason, .. }) if reason.contains(expected) && reason.contains("git ls-files -z")),
                "{error:?}"
            );
            assert_eq!(runner.listings.get(), fail_at);
            assert_eq!(
                fixture.snapshot(),
                before,
                "failed listing {fail_at} changed documents"
            );
        }
    }
}

#[test]
fn complete_accepts_the_change_prefix_and_an_unknown_slug_or_a_record_that_did_not_load_is_an_error()
 {
    let fixture = Fixture::scripted(&Spec::default());
    // A sync refuses documents that did not load, so the record is written
    // after the fixture is built.
    put(
        fixture.root(),
        "work/changes/broken.md",
        "---\ntype: Change\nid: change:broken\n---\n\n## Summary\n\nNo keys.\n",
    );
    let lifecycle = fixture.over(git_answers());
    let unknown = lifecycle.complete("nothing-like-it").expect_err("unknown");
    assert!(
        matches!(
            &unknown,
            LifecycleError::Change(ChangeLookupError::Unknown { .. })
        ),
        "{unknown:?}"
    );
    let broken = lifecycle.complete("broken").expect_err("did not load");
    assert!(
        matches!(
            &broken,
            LifecycleError::Change(ChangeLookupError::NotLoaded { .. })
        ),
        "{broken:?}"
    );
    assert!(
        lifecycle
            .runner()
            .calls()
            .iter()
            .all(|c| c.program() == Program::Git),
        "no cargo process ran"
    );
    assert!(fixture.exists("work/changes/broken.md") && fixture.exists(RECORD));
}

#[test]
fn a_prefixed_slug_names_the_same_change() {
    let fixture = Fixture::scripted(&Spec::default());
    let runner = passing(git_answers(), "passes");
    let completion = fixture
        .over(runner)
        .complete("change:shape")
        .expect("complete answers");
    assert!(completion.is_completed(), "{completion}");
    assert!(!fixture.exists(RECORD));
}

/// Real bound source observes the completed files, including ignored native inputs.
fn completed_view_spec() -> Spec {
    let checks = r#"        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(!root.join("work/changes/shape.md").exists());
        let root_write = std::fs::write(root.join("unadmitted-source-write"), b"changed").unwrap_err();
        assert_eq!(root_write.kind(), std::io::ErrorKind::ReadOnlyFilesystem);
        let cap = std::fs::read_to_string(root.join("docs/capabilities/fixture.md")).unwrap();
        assert!(!cap.contains("| Changed by | [shape]"));
        assert_eq!(std::fs::read(root.join(".private/input.bin")).unwrap(), [0, 255, 13, 10]);
        assert_eq!(std::fs::read_to_string(root.join(".private/settings.toml")).unwrap(), "enabled=true\n");
        let alias = std::fs::read_to_string(root.join(".private/cap-alias.md")).unwrap();
        assert!(alias.contains("| Changed by | [shape]"));
        #[cfg(unix)] {
            use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
            let file = std::fs::metadata(root.join(".private/input.bin")).unwrap();
            let hard = std::fs::metadata(root.join(".private/hard.bin")).unwrap();
            assert_eq!(file.permissions().mode() & 0o777, 0o751);
            assert_eq!((file.dev(), file.ino()), (hard.dev(), hard.ino()));
            assert_eq!(std::fs::read_link(root.join(".private/link")).unwrap(), std::path::Path::new("input.bin"));
            let direct = std::fs::read_link(root.join("absolute-link")).unwrap();
            let nested = std::fs::read_link(root.join(".private/absolute-link")).unwrap();
            assert_eq!(direct, nested);
            assert_eq!(direct, root.join(".private/input.bin"));
            let resolved = std::fs::metadata(&direct).unwrap();
            assert_eq!((resolved.dev(), resolved.ino()), (file.dev(), file.ino()));
            assert_eq!(std::fs::read(direct).unwrap(), [0, 255, 13, 10]);
        }
"#;
    let mut spec = Spec::default().doc(".gitignore", "/.private/\n/absolute-link\n".to_string());
    spec.lib = LIB.replace("        assert_eq!(super::add(1, 2), 3);", checks);
    spec
}

fn completed_view_inputs(fixture: &Fixture) {
    let root = fixture.root();
    std::fs::create_dir(root.join(".private")).expect("ignored fixture inputs");
    std::fs::write(root.join(".private/input.bin"), [0, 255, 13, 10]).expect("binary input");
    put(root, ".private/settings.toml", "enabled=true\n");
    std::fs::hard_link(root.join(CAP), root.join(".private/cap-alias.md"))
        .expect("original document alias");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(
            root.join(".private/input.bin"),
            std::fs::Permissions::from_mode(0o751),
        )
        .expect("executable input");
        std::fs::hard_link(
            root.join(".private/input.bin"),
            root.join(".private/hard.bin"),
        )
        .expect("input alias");
        std::os::unix::fs::symlink("input.bin", root.join(".private/link"))
            .expect("opaque link spelling");
        for path in ["absolute-link", ".private/absolute-link"] {
            std::os::unix::fs::symlink(root.join(".private/input.bin"), root.join(path)).unwrap();
        }
    }
}

/// A policy refusal after record removal starts no cargo and writes nothing,
/// even when the unchanged library itself owes no contract-effect obligation.
#[test]
fn completion_after_record_removal_enforces_the_complete_policy() {
    let fixture = Fixture::scripted(&Spec::default().examples("[]", &[]));
    let text = fixture.text(CAP)
        .replace("This demonstration describes the supported behavior of its library, tests and examples, which readers can inspect together with the generated document relationships.\n", "")
        .replace("- This demonstration describes one library and its executable fixtures; it models no external application state or network service.", "One limitation applies.");
    put(fixture.root(), CAP, &text);
    fixture.assess();
    let before = fixture.snapshot();
    let runner = at_merge_base(
        git_answers(),
        &[("Cargo.toml", MANIFEST), ("src/lib.rs", LIB)],
    );
    let strict = Lifecycle::open_with(&fixture.gate(), passing(runner, "passes"))
        .unwrap_or_else(|error| panic!("the fixture opens: {error}"));
    let completion = strict.complete("shape").expect("complete answers");
    let failures = refusal(&completion);
    assert!(
        failures
            .iter()
            .all(|failure| failure.condition() == Condition::AfterRemoval),
        "{completion}"
    );
    let names: Vec<&str> = failures.iter().map(Failure::name).collect();
    for kind in [
        "strict-example-missing",
        "strict-section-thin",
        "strict-usage-unbound",
        "strict-limitations-missing",
    ] {
        assert!(names.contains(&kind), "{kind} in {names:?}");
    }
    assert_eq!(fixture.snapshot(), before, "a refusal writes nothing");
    assert!(
        !programs_of(
            &strict
                .runner()
                .calls()
                .iter()
                .map(Call::command)
                .collect::<Vec<_>>()
        )
        .contains(&"cargo"),
        "a static refusal runs no cargo"
    );
}

/// The record is removed through real directories only. A linked `work`
/// inside the root, made after the repository is opened, is refused before
/// any `cargo` process or write, with the record behind it kept; a linked
/// `work/changes` is never read as the directory of change records, so the
/// change is not found and nothing runs either. The neighbour with real
/// directories completes.
#[cfg(unix)]
#[test]
fn a_linked_work_directory_is_refused_before_anything_runs_and_a_real_one_completes() {
    let fixture = Fixture::scripted(&Spec::default());
    let root = fixture.root();
    let lifecycle = fixture.over(passing(git_answers(), "passes"));
    for link in ["work", "work/changes"] {
        std::fs::rename(root.join(link), root.join("moved")).expect("moves");
        let up = "../".repeat(link.matches('/').count());
        std::os::unix::fs::symlink(format!("{up}moved"), root.join(link)).expect("links");
        let before = fixture.snapshot();
        let error = lifecycle.complete("shape").expect_err(link);
        if link == "work" {
            assert!(
                matches!(&error, LifecycleError::RecordRefused { path, .. } if path == "work"),
                "{error:?}"
            );
            assert!(
                matches!(&error, LifecycleError::RecordRefused { ran: false, .. }),
                "{error:?}"
            );
            let text = error.to_string();
            assert!(
                text.contains("at the first judgement, before anything ran: ")
                    && text.contains("; nothing was written, so make"),
                "{text}"
            );
        } else {
            assert!(
                matches!(
                    &error,
                    LifecycleError::Change(ChangeLookupError::Unknown { .. })
                ),
                "{error:?}"
            );
        }
        assert_eq!(fixture.snapshot(), before, "{link}: nothing is written");
        assert!(
            lifecycle
                .runner()
                .calls()
                .iter()
                .all(|c| c.program() == Program::Git),
            "{link}: no cargo process ran"
        );
        std::fs::remove_file(root.join(link)).expect("unlinks");
        std::fs::rename(root.join("moved"), root.join(link)).expect("moves back");
    }
    let completion = lifecycle.complete("shape").expect("complete answers");
    assert!(completion.is_completed(), "{completion}");
    assert!(!fixture.exists(RECORD));
}

/// A change record that becomes a symbolic link while the bound tests run is
/// refused by the second judgement, after the tests ran: the change still
/// loads through the link, the error names the record and says the tests
/// ran, and nothing is written, the link and the file behind it kept. The
/// neighbour, whose run changes nothing, completes.
#[cfg(unix)]
#[test]
fn a_record_linked_while_the_tests_ran_is_refused_at_the_second_judgement() {
    let fixture = Fixture::scripted(&Spec::default());
    let root = fixture.root();
    let runner = AfterCargo::new(passing(git_answers(), "passes"), || {
        std::fs::rename(root.join(RECORD), root.join("moved.md")).expect("moves");
        std::os::unix::fs::symlink("../../moved.md", root.join(RECORD)).expect("links");
    });
    let lifecycle = fixture.over(runner);
    let cap = fixture.text(CAP);
    let error = lifecycle.complete("shape").expect_err("refused");
    assert_eq!(fixture.text(CAP), cap, "no generated block is rewritten");
    assert!(
        matches!(&error, LifecycleError::RecordRefused { path, ran: true, .. } if path == RECORD),
        "{error:?}"
    );
    let text = error.to_string();
    assert!(
        text.contains("at the second judgement, after the bound tests and examples ran: ")
            && text.contains("; nothing was written, so make"),
        "{text}"
    );
    assert!(
        programs_of(&lifecycle.runner().commands()).contains(&"cargo"),
        "the bound tests ran"
    );
    assert!(
        std::fs::symlink_metadata(root.join(RECORD))
            .expect("the link is kept")
            .file_type()
            .is_symlink()
    );
    assert!(
        fixture.exists("moved.md"),
        "the file behind the link is kept"
    );

    let neighbour = Fixture::scripted(&Spec::default());
    let runner = AfterCargo::new(passing(git_answers(), "passes"), || {});
    let completion = neighbour.over(runner).complete("shape").expect("answers");
    assert!(completion.is_completed(), "{completion}");
    assert!(!neighbour.exists(RECORD));
}

// --- condition 6 ---------------------------------------------------------------------

#[test]
fn an_agent_assessment_satisfies_condition_six_and_a_rejected_one_by_either_never_does() {
    // The fixture's Contract is assessed and accepted by an agent: that alone
    // satisfies the gate.
    let fixture = Fixture::scripted(&Spec::default());
    let accepted = fixture.text(ASSESSMENT);
    assert!(accepted.contains("\nby: agent\n"), "{accepted}");
    // A rejection by a human or by an agent refuses under conditions 1 and 6,
    // spends no process on `cargo`, and writes nothing.
    for by in Actor::ALL {
        assess_on_disk_as(fixture.root(), "shape", AssessmentVerdict::Rejected, by);
        let before = fixture.snapshot();
        let lifecycle = fixture.over(git_answers());
        let refused = lifecycle.complete("shape").expect("answers");
        assert_eq!(count(&refused, Condition::Assessments), 1, "{refused}");
        assert_eq!(kinds(&refused), [ObligationKind::AssessmentRejected]);
        let text = refused.to_string();
        assert!(
            text.starts_with(
                "refused: 1 obligation, 1 section without a current accepted assessment\n"
            ),
            "{text}"
        );
        // The fault prints once, under its obligation.
        assert_eq!(text.matches("assessment-rejected").count(), 1, "{text}");
        assert!(
            text.contains(&format!("has the verdict rejected, by {})", by.name())),
            "{text}"
        );
        assert!(
            lifecycle
                .runner()
                .calls()
                .iter()
                .all(|call| call.program() == Program::Git),
            "a refusal of condition 6 starts no cargo"
        );
        assert_eq!(fixture.snapshot(), before);
    }
    // The neighbour: accepted again by an agent, the change completes, and the
    // assessment is left as it is.
    fixture.assess();
    assert_eq!(fixture.text(ASSESSMENT), accepted);
    let completion = fixture
        .over(passing(git_answers(), "passes"))
        .complete("shape")
        .expect("answers");
    assert!(completion.is_completed(), "{completion}");
    assert_eq!(fixture.text(ASSESSMENT), accepted);
    assert!(!fixture.exists(RECORD));
}

// --- one refusal for each kind of obligation ---------------------------------------

const GOOD_LIST: &str = "[fx_life::example_crate::good]";
const BROKEN_LIST: &str = "[fx_life::example_crate::broken]";

/// A fault that one kind of obligation stands for, the repair that removes it,
/// and a spec whose repository holds the fault.
struct Case {
    kind: ObligationKind,
    spec: Spec,
    /// Made after the repository is built and synced.
    fault: fn(&Fixture),
    /// The edit the author makes, including the `docs sync` that follows an
    /// edit of the generated content.
    repair: fn(&Fixture),
}

fn nothing(_: &Fixture) {}

fn cases() -> Vec<Case> {
    let mut cases = document_cases();
    cases.extend(trace_cases());
    cases.extend(assessment_cases());
    cases.extend(review_cases());
    cases
}

/// The cases of the kinds the documents and the bound runs owe.
fn document_cases() -> Vec<Case> {
    let implementing = || {
        Spec::default()
            .state("implementing")
            .examples(GOOD_LIST, &[("good", GOOD_EXAMPLE)])
    };
    vec![
        Case {
            kind: ObligationKind::UnresolvedProposal,
            spec: implementing().extra("Contract", PROPOSAL),
            fault: nothing,
            repair: |fixture| {
                fixture.edit(CAP, PROPOSAL, "");
                fixture.edit(CAP, "state: implementing", "state: delivered");
                fixture.resync();
                fixture.assess();
            },
        },
        Case {
            kind: ObligationKind::ItemWithoutOutcome,
            spec: Spec::default().items(NO_OUTCOME),
            fault: nothing,
            repair: |fixture| fixture.edit(RECORD, NO_OUTCOME, DONE_FIXTURE),
        },
        Case {
            kind: ObligationKind::OutcomeWithoutReason,
            spec: strict_ready().items(DROPPED),
            fault: nothing,
            repair: |fixture| fixture.edit(RECORD, DROPPED, DROPPED_WITH_REASON),
        },
        Case {
            kind: ObligationKind::BindingMissing,
            spec: Spec::default().tests("[fx_life::tests::nowhere]"),
            fault: nothing,
            repair: |fixture| {
                fixture.rebind_and_assess_tests("[fx_life::tests::nowhere]", PASSES);
            },
        },
        Case {
            kind: ObligationKind::SectionUntouched,
            spec: Spec::default().effect("changes", &["cap:fixture#contract"]),
            fault: nothing,
            repair: |fixture| {
                fixture.edit(
                    CAP,
                    "Text for Contract of fixture.",
                    "Text for Contract of fixture, as revised.",
                );
                fixture.assess();
            },
        },
        Case {
            kind: ObligationKind::ContractEffectUnderstated,
            spec: strict_ready(),
            fault: |fixture| {
                put(fixture.root(), "src/lib.rs", &edited_lib());
                fixture.assess();
            },
            // The change states its effect and reconciles the section it owes.
            repair: |fixture| {
                fixture.edit(RECORD, "contract_effect: none", "contract_effect: changes");
                fixture.edit(
                    CAP,
                    "Text for Contract of fixture.",
                    "Prose for Contract of fixture.",
                );
                fixture.assess();
            },
        },
        Case {
            kind: ObligationKind::ExampleFailing,
            spec: Spec::default().examples(BROKEN_LIST, &[("broken", BROKEN_EXAMPLE)]),
            fault: nothing,
            repair: |fixture| put(fixture.root(), "examples/broken.rs", GOOD_EXAMPLE),
        },
        Case {
            kind: ObligationKind::GeneratedStale,
            spec: Spec::default(),
            fault: |fixture| fixture.edit(CAP, "description: Shaped.", "description: Reshaped."),
            repair: Fixture::resync,
        },
        Case {
            kind: ObligationKind::TenseMismatch,
            spec: Spec::default(),
            fault: |fixture| {
                fixture.edit(
                    CAP,
                    "Text for Purpose of fixture.",
                    "Text for Purpose of fixture. This will be revisited.",
                );
            },
            repair: |fixture| {
                fixture.edit(
                    CAP,
                    "Text for Purpose of fixture. This will be revisited.",
                    "Text for Purpose of fixture.",
                );
            },
        },
    ]
}

/// The cases of the kinds the claims and the evidence records owe.
fn trace_cases() -> Vec<Case> {
    vec![
        Case {
            kind: ObligationKind::ClaimUnsupported,
            spec: Spec::default()
                .extra("Contract", CLAIM)
                .doc(EVIDENCE, evidence().verdict("fail").text()),
            fault: nothing,
            repair: |fixture| {
                fixture.edit(EVIDENCE, "verdict: fail", "verdict: pass");
                fixture.assess();
            },
        },
        Case {
            kind: ObligationKind::EvidenceStale,
            spec: Spec::default()
                .extra("Contract", CLAIM)
                .doc(EVIDENCE, evidence().text()),
            fault: |fixture| {
                let lib = LIB.replace(
                    "    a + b\n",
                    "    // Addition retains its tokens.\n    a + b\n",
                );
                put(fixture.root(), "src/lib.rs", &lib);
                fixture.assess();
            },
            repair: |fixture| {
                let lib = LIB.replace(
                    "    a + b\n",
                    "    // Addition retains its tokens.\n    a + b\n",
                );
                fixture.edit(EVIDENCE, &add_digest(LIB), &add_digest(&lib));
                fixture.assess();
            },
        },
        Case {
            kind: ObligationKind::EvidenceChanged,
            spec: Spec::default().extra("Contract", CLAIM).doc(
                EVIDENCE,
                evidence()
                    .values(&[("peak", "1")])
                    .limits(&[("peak", "2")])
                    .text(),
            ),
            fault: |fixture| {
                put(
                    fixture.root(),
                    EVIDENCE,
                    &evidence().values(&[("peak", "1")]).text(),
                );
                fixture.assess();
            },
            repair: |fixture| {
                put(
                    fixture.root(),
                    EVIDENCE,
                    &evidence()
                        .values(&[("peak", "1")])
                        .limits(&[("peak", "2")])
                        .text(),
                );
                fixture.assess();
            },
        },
    ]
}

/// The assessment of the Contract of `fixture`, the one section the change
/// affects.
const ASSESSMENT: &str = "work/assessments/fixture-contract.md";

/// The cases of the kinds the assessments owe: the assessment removed, the
/// section edited after it was made, and a current assessment that rejects
/// the section. Each repair assesses the section again with the verdict
/// `accepted`.
fn assessment_cases() -> Vec<Case> {
    vec![
        Case {
            kind: ObligationKind::AssessmentMissing,
            spec: Spec::default(),
            fault: |fixture| {
                std::fs::remove_file(fixture.root().join(ASSESSMENT)).expect("removed");
            },
            repair: Fixture::assess,
        },
        Case {
            kind: ObligationKind::AssessmentStale,
            spec: Spec::default(),
            fault: |fixture| {
                fixture.edit(
                    CAP,
                    "Text for Contract of fixture.",
                    "Text for Contract of fixture, edited after the assessment.",
                );
            },
            repair: Fixture::assess,
        },
        Case {
            kind: ObligationKind::AssessmentRejected,
            spec: Spec::default(),
            fault: |fixture| {
                assess_on_disk_as(
                    fixture.root(),
                    "shape",
                    AssessmentVerdict::Rejected,
                    Actor::Human,
                );
            },
            repair: Fixture::assess,
        },
    ]
}

/// Required reviews refuse both absence and changed owned source, and become
/// current after recording the review or restoring the reviewed source.
fn review_cases() -> Vec<Case> {
    vec![
        Case {
            kind: ObligationKind::ReviewMissing,
            spec: Spec::default(),
            fault: require_fixture_review,
            repair: record_fixture_review,
        },
        Case {
            kind: ObligationKind::ReviewStale,
            spec: Spec::default(),
            fault: |fixture| {
                require_fixture_review(fixture);
                record_fixture_review(fixture);
                fixture.edit(
                    "src/lib.rs",
                    "    a + b\n",
                    "    // Reviewed source changed.\n    a + b\n",
                );
                fixture.assess();
            },
            repair: |fixture| {
                put(fixture.root(), "src/lib.rs", LIB);
                fixture.assess();
            },
        },
    ]
}

fn require_fixture_review(fixture: &Fixture) {
    fixture.edit(CAP, "depends:", "review_required: [security]\ndepends:");
    fixture.resync();
    fixture.assess();
}

fn record_fixture_review(fixture: &Fixture) {
    use raf_okf::{DocType, RecordHeader, ReviewKind, ReviewRecord, ReviewScope, ReviewVerdict};
    let mut header = RecordHeader::new(DocType::Review, "fixture-security");
    header.timestamp = Some("2026-01-01T00:00:00Z".to_string());
    let mut draft = ReviewRecord::new(
        header,
        ReviewScope::Capability("cap:fixture".to_string()),
        git(fixture.root(), &["rev-parse", "HEAD"]),
        ReviewKind::Security,
        ReviewVerdict::Findings,
        Actor::Human,
        Vec::new(),
    );
    draft.findings_ref = Some("https://example.invalid/review".to_string());
    raf_workplan::review::record_review(
        fixture.root(),
        &SystemRunner::new(fixture.programs()),
        &draft,
        &raf_workplan::record::RecordOptions::new(),
    )
    .expect("the current source review is recorded");
}

/// The item of the fixtures' record once it has its outcome.
const DONE_FIXTURE: &str = DONE;

#[test]
fn the_cases_cover_every_kind_of_obligation_in_both_directions() {
    let covered: Vec<ObligationKind> = cases().iter().map(|case| case.kind).collect();
    assert_eq!(
        covered,
        ObligationKind::ALL,
        "one case for each kind, in order"
    );
}

#[test]
fn each_kind_of_obligation_makes_complete_refuse_and_leaves_the_tree_as_it_was_and_its_repair_completes()
 {
    for case in cases() {
        let name = case.kind.name();
        let fixture = Fixture::real(&case.spec);
        (case.fault)(&fixture);
        let before = fixture.snapshot();
        let record_before = fixture.text(RECORD);

        let refused = fixture.complete();
        let failures = refusal(&refused);
        assert!(!failures.is_empty(), "{name}");
        // The kind is the only kind that stands, and it stands as an obligation
        // of condition 1.
        let mut stood = kinds(&refused);
        stood.sort();
        stood.dedup();
        assert_eq!(stood, [case.kind], "{name}: {refused}");
        assert!(count(&refused, Condition::Obligations) >= 1, "{name}");
        // An assessment kind stands under condition 6 too, and no other kind
        // does.
        let assessed = matches!(
            case.kind,
            ObligationKind::AssessmentMissing
                | ObligationKind::AssessmentStale
                | ObligationKind::AssessmentRejected
        );
        assert_eq!(
            count(&refused, Condition::Assessments),
            usize::from(assessed),
            "{name}: {refused}"
        );
        assert!(
            refused.to_string().starts_with("refused: "),
            "{name}: {refused}"
        );
        // Nothing was written: the record is byte-identical and so is the rest.
        assert_eq!(fixture.text(RECORD), record_before, "{name}");
        assert_eq!(fixture.snapshot(), before, "{name}");

        // The passing neighbour: the same repository once the author repaired it.
        (case.repair)(&fixture);
        let completed = fixture.complete();
        assert!(
            completed.is_completed(),
            "{name} after its repair: {completed}"
        );
        assert!(!fixture.exists(RECORD), "{name}");
        Gate::workspace(fixture.root())
            .programs(fixture.programs())
            .assert_clean();
    }
}

/// The file an obligation of each kind points at in the cases above, and
/// whether it names a line.
fn place_of(kind: ObligationKind) -> (&'static str, bool) {
    match kind {
        ObligationKind::UnresolvedProposal
        | ObligationKind::TenseMismatch
        | ObligationKind::ClaimUnsupported
        | ObligationKind::EvidenceChanged => (CAP, true),
        ObligationKind::ItemWithoutOutcome
        | ObligationKind::OutcomeWithoutReason
        | ObligationKind::ContractEffectUnderstated => (RECORD, false),
        ObligationKind::BindingMissing
        | ObligationKind::SectionUntouched
        | ObligationKind::ExampleFailing
        | ObligationKind::AssessmentMissing
        | ObligationKind::ReviewMissing
        | ObligationKind::ReviewStale => (CAP, false),
        ObligationKind::GeneratedStale => (INDEX, true),
        ObligationKind::EvidenceStale => (EVIDENCE, false),
        ObligationKind::AssessmentStale | ObligationKind::AssessmentRejected => (ASSESSMENT, false),
        _ => unreachable!("every kind has a case"),
    }
}

#[test]
fn each_kind_of_obligation_is_a_reconcile_line_in_a_git_repository_and_fixing_it_removes_the_line()
{
    for case in cases() {
        let name = case.kind.name();
        let fixture = Fixture::real(&case.spec);
        (case.fault)(&fixture);
        let before = fixture.snapshot();

        let owed = fixture
            .lifecycle()
            .reconcile("shape")
            .unwrap_or_else(|error| panic!("{name}: reconcile answers: {error}"));
        assert!(!owed.is_empty(), "{name}");
        let (path, with_line) = place_of(case.kind);
        let text = owed.to_string();
        for obligation in owed.obligations() {
            // The kind is the only one owed, at its place, with its evidence.
            assert_eq!(obligation.kind, case.kind, "{name}: {text}");
            assert_eq!(obligation.location.path, path, "{name}: {text}");
            assert_eq!(
                obligation.location.line.is_some(),
                with_line,
                "{name}: {text}"
            );
            let line = obligation.to_string();
            assert!(line.contains(&format!("  {name}  ")), "{name}: {line}");
            assert!(
                line.contains(obligation.evidence.contract_text()),
                "{name}: {line}"
            );
        }
        // Reconcile writes nothing in the repository.
        assert_eq!(fixture.snapshot(), before, "{name}");

        // Fixing it removes the line.
        (case.repair)(&fixture);
        let repaired = fixture
            .lifecycle()
            .reconcile("shape")
            .unwrap_or_else(|error| panic!("{name} repaired: reconcile answers: {error}"));
        assert_eq!(repaired.of_kind(case.kind).count(), 0, "{name}: {repaired}");
        assert!(repaired.is_empty(), "{name}: {repaired}");
    }
}

// --- one refusal for each condition -------------------------------------------------

#[test]
fn an_item_without_outcome_and_a_move_to_nowhere_or_to_itself_refuse_condition_two() {
    let moved = |target: &str| {
        format!(
            "  - claim: \"fixture is shaped\"\n    outcome: moved:change:{target}\n    reason: \"Another capability owns this behavior.\"\n"
        )
    };
    for (outcome, why) in [
        (moved("nowhere"), "names no change record"),
        (
            moved("shape"),
            "moves to the change that is being completed",
        ),
    ] {
        let fixture = Fixture::scripted(&Spec::default());
        let text = fixture.text(RECORD).replace(DONE, &outcome);
        put(fixture.root(), RECORD, &text);
        let before = fixture.snapshot();
        let lifecycle = fixture.over(passing(git_answers(), "passes"));
        let refused = lifecycle.complete("shape").expect("answers");
        assert_eq!(count(&refused, Condition::Outcomes), 1, "{refused}");
        assert_eq!(count(&refused, Condition::Obligations), 1);
        assert_eq!(kinds(&refused), [ObligationKind::ItemWithoutOutcome]);
        assert!(refused.to_string().contains(why), "{refused}");
        assert_eq!(fixture.snapshot(), before);
    }

    // The neighbour: a move to a record that exists completes. The other record
    // lists another capability, so the state rules hold beside it.
    let spec = other_record(Spec::default().items(NO_OUTCOME));
    let fixture = Fixture::scripted(&spec);
    let text = fixture.text(RECORD).replace(NO_OUTCOME, &moved("other"));
    put(fixture.root(), RECORD, &text);
    let completed = fixture
        .over(passing(git_answers(), "passes"))
        .complete("shape")
        .expect("answers");
    assert!(completed.is_completed(), "{completed}");
}

#[test]
fn a_proposed_block_of_the_change_refuses_condition_three_in_a_loaded_and_in_an_unloaded_document()
{
    // Loaded: the block stands in the capability the change lists.
    let fixture = Fixture::scripted(
        &Spec::default()
            .state("implementing")
            .examples(GOOD_LIST, &[("good", GOOD_EXAMPLE)])
            .extra("Contract", PROPOSAL),
    );
    let before = fixture.snapshot();
    let refused = fixture
        .over(git_answers())
        .complete("shape")
        .expect("answers");
    assert_eq!(count(&refused, Condition::Proposals), 1, "{refused}");
    assert_eq!(kinds(&refused), [ObligationKind::UnresolvedProposal]);
    assert_eq!(fixture.snapshot(), before);

    // Unloaded: the document has no title, so it is not part of the model, and
    // its block is still the change's.
    let fixture = Fixture::scripted(&Spec::default());
    let zeta = capability(
        "zeta",
        "delivered",
        ["[fx_life::add]", PASSES, "[]"],
        &[("Purpose", PROPOSAL)],
    )
    .replace("title: \"Title of zeta\"\n", "");
    put(fixture.root(), "docs/capabilities/zeta.md", &zeta);
    let before = fixture.snapshot();
    let refused = fixture
        .over(passing(git_answers(), "passes"))
        .complete("shape")
        .expect("answers");
    assert_eq!(count(&refused, Condition::Proposals), 1, "{refused}");
    let Some(Failure::Proposal(owed)) = refused.of_condition(Condition::Proposals).next() else {
        panic!("a proposal failure: {refused}")
    };
    assert_eq!(owed.location.path, "docs/capabilities/zeta.md");
    assert_eq!(owed.subject.to_string(), "cap:zeta");
    // The document is also a finding of the repository without the record.
    assert!(
        refused.of_condition(Condition::AfterRemoval).count() >= 1,
        "{refused}"
    );
    assert_eq!(fixture.snapshot(), before);

    // The neighbour: without the document the change completes.
    std::fs::remove_file(fixture.root().join("docs/capabilities/zeta.md")).expect("removed");
    let completed = fixture
        .over(passing(git_answers(), "passes"))
        .complete("shape")
        .expect("answers");
    assert!(completed.is_completed(), "{completed}");
}

#[test]
fn a_block_that_is_stale_or_missing_anywhere_refuses_condition_four() {
    // A stale block in a capability the change does not list is no obligation of
    // the change, and `docs sync` would still write it.
    let fixture = Fixture::scripted(&other_record(Spec::default()));
    fixture.edit(
        "docs/capabilities/second.md",
        "| Relation | Capability |",
        "| Relation | Capabilities |",
    );
    let before = fixture.snapshot();
    let refused = fixture
        .over(passing(git_answers(), "passes"))
        .complete("shape")
        .expect("answers");
    assert_eq!(count(&refused, Condition::GeneratedBlocks), 1, "{refused}");
    assert_eq!(refused.obligations().count(), 0, "{refused}");
    let text = refused.to_string();
    assert!(
        text.starts_with("refused: 1 stale generated block\n"),
        "{text}"
    );
    assert!(
        text.contains("docs/capabilities/second.md: line ") && text.contains("[generated-stale]"),
        "{text}"
    );
    assert_eq!(fixture.snapshot(), before);

    // A missing index is a fault of the condition too.
    let fixture = Fixture::scripted(&Spec::default());
    std::fs::remove_file(fixture.root().join(INDEX)).expect("removed");
    let refused = fixture
        .over(passing(git_answers(), "passes"))
        .complete("shape")
        .expect("answers");
    assert!(
        refused
            .of_condition(Condition::GeneratedBlocks)
            .any(|failure| failure.finding_kind() == Some("index-missing")),
        "{refused}"
    );

    // The neighbours: after the sync, both complete.
    fixture.resync();
    assert!(
        fixture
            .over(passing(git_answers(), "passes"))
            .complete("shape")
            .expect("answers")
            .is_completed()
    );
}

#[test]
fn a_failing_bound_test_refuses_condition_five_alone() {
    let fixture = Fixture::real(&Spec::default().tests("[fx_life::tests::fails]"));
    let before = fixture.snapshot();
    let refused = fixture.complete();
    assert_eq!(count(&refused, Condition::BoundRuns), 1, "{refused}");
    assert!(
        matches!(refused.failures(), [Failure::Test(evidence)]
            if matches!(evidence.outcome(), BoundOutcome::Failed { .. })),
        "{refused}"
    );
    // A failing test is no obligation: the list is empty.
    assert_eq!(refused.obligations().count(), 0, "{refused}");
    let text = refused.to_string();
    assert!(
        text.starts_with("refused: 1 bound test failing\n"),
        "{text}"
    );
    assert!(
        text.contains("fail cap:fixture tests `fx_life::tests::fails`: "),
        "{text}"
    );
    assert_eq!(fixture.snapshot(), before);

    fixture.rebind_and_assess_tests("[fx_life::tests::fails]", PASSES);
    fixture.resync();
    assert!(fixture.complete().is_completed());
}

#[test]
fn a_failing_doc_test_a_test_that_runs_nothing_and_an_ignored_test_refuse_beside_passing_neighbours()
 {
    // A documentation test that fails: an `example-failing` obligation and a
    // failure of condition 5. Its neighbour, the passing doc test, completes.
    let doc = "/// assert_eq!(fx_life::add(1, 2), 3);";
    let fixture = Fixture::real(&Spec::default().examples("[fx_life::add]", &[]));
    fixture.edit("src/lib.rs", doc, "/// assert_eq!(fx_life::add(1, 2), 4);");
    declare_effect(&fixture);
    // The documentation is part of the text of the bound symbol.
    fixture.assess();
    let before = fixture.snapshot();
    let refused = fixture.complete();
    assert_eq!(count(&refused, Condition::BoundRuns), 1, "{refused}");
    assert!(
        matches!(
            refused.of_condition(Condition::BoundRuns).next(),
            Some(Failure::Example(_))
        ),
        "{refused}"
    );
    assert_eq!(kinds(&refused), [ObligationKind::ExampleFailing]);
    assert_eq!(fixture.snapshot(), before);
    fixture.edit("src/lib.rs", "/// assert_eq!(fx_life::add(1, 2), 4);", doc);
    fixture.assess();
    assert!(fixture.complete().is_completed());

    // A tests binding to a function that is not a `#[test]` runs zero tests:
    // it does not fit its list, so it is owed and no process proves it.
    let fixture = Fixture::real(&Spec::default().tests("[fx_life::plain]"));
    let before = fixture.snapshot();
    let refused = fixture.complete();
    assert_eq!(
        kinds(&refused),
        [ObligationKind::BindingMissing],
        "{refused}"
    );
    assert_eq!(count(&refused, Condition::BoundRuns), 0, "{refused}");
    assert_eq!(fixture.snapshot(), before);

    // An `#[ignore]`d test is listed and does not run: condition 5 refuses.
    let fixture = Fixture::real(&Spec::default());
    fixture.edit(
        "src/lib.rs",
        "    fn also_passes() {}\n",
        "    fn also_passes() {}\n\n    #[test]\n    #[ignore]\n    fn ignored() {}\n",
    );
    fixture.rebind_and_assess_tests(PASSES, "[fx_life::tests::ignored]");
    fixture.resync();
    let before = fixture.snapshot();
    let refused = fixture.complete();
    assert_eq!(count(&refused, Condition::BoundRuns), 1, "{refused}");
    assert!(
        matches!(refused.failures(), [Failure::Test(evidence)] if !evidence.passed()),
        "{refused}"
    );
    assert_eq!(fixture.snapshot(), before);
    // The neighbour: the passing test completes.
    fixture.rebind_and_assess_tests("[fx_life::tests::ignored]", PASSES);
    fixture.resync();
    assert!(fixture.complete().is_completed());
}

#[test]
fn a_failing_bound_example_stands_under_conditions_one_and_five() {
    let fixture =
        Fixture::real(&Spec::default().examples(BROKEN_LIST, &[("broken", EXITING_EXAMPLE)]));
    let before = fixture.snapshot();
    let refused = fixture.complete();
    assert_eq!(count(&refused, Condition::Obligations), 1, "{refused}");
    assert_eq!(count(&refused, Condition::BoundRuns), 1, "{refused}");
    assert_eq!(kinds(&refused), [ObligationKind::ExampleFailing]);
    assert!(
        matches!(
            refused.of_condition(Condition::BoundRuns).next(),
            Some(Failure::Example(_))
        ),
        "{refused}"
    );
    // The contract's second example: the counts, then the one obligation line.
    let text = refused.to_string();
    let mut lines = text.lines();
    assert_eq!(
        lines.next(),
        Some("refused: 1 obligation, 1 bound example failing")
    );
    let line = lines.next().expect("the obligation line");
    assert!(
        line.starts_with("cap:fixture  examples  example-failing  ")
            && line
                .contains("`fx_life::example_crate::broken` fails to build or run (exit code 3)"),
        "{line}"
    );
    assert_eq!(lines.next(), None, "{text}");
    assert_eq!(fixture.snapshot(), before);

    // A test and an example that both fail are two failures of one condition.
    fixture.rebind_and_assess_tests(PASSES, "[fx_life::tests::fails]");
    fixture.resync();
    let refused = fixture.complete();
    assert_eq!(count(&refused, Condition::BoundRuns), 2, "{refused}");
    assert!(
        refused
            .to_string()
            .starts_with("refused: 1 obligation, 1 bound test failing, 1 bound example failing\n"),
        "{refused}"
    );

    // The neighbour: both repaired.
    fixture.rebind_and_assess_tests("[fx_life::tests::fails]", PASSES);
    fixture.resync();
    put(fixture.root(), "examples/broken.rs", GOOD_EXAMPLE);
    assert!(fixture.complete().is_completed());
}

#[test]
fn an_item_without_outcome_and_a_stale_block_print_the_counts_of_the_contract() {
    let fixture = Fixture::scripted(&Spec::default().items(NO_OUTCOME));
    fixture.edit(CAP, "description: Shaped.", "description: Reshaped.");
    let refused = fixture
        .over(passing(git_answers(), "passes"))
        .complete("shape")
        .expect("answers");
    let text = refused.to_string();
    let mut lines = text.lines();
    assert_eq!(
        lines.next(),
        Some("refused: 2 obligations, 1 item without outcome, 1 stale generated block")
    );
    // One line for each obligation; the condition-two and condition-four
    // failures repeat them and print no line of their own.
    assert_eq!(lines.count(), 2, "{text}");
    assert!(
        text.contains("change:shape  item 1  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>"),
        "{text}"
    );
    assert!(
        text.contains("docs/capabilities/index.md  document  generated-stale  line ")
            && text.contains("a docs sync run (the `catalogue` block is stale)"),
        "{text}"
    );
}

/// The headline counts sections for condition 6: one section whose
/// assessment has three moved inputs (its own text and two bound symbols) is
/// three obligations, each printed once, and one section; two sections with
/// one moved input each are two.
#[test]
fn a_section_with_three_moved_inputs_is_one_section_in_the_refusal_headline() {
    let spec = Spec {
        code: "[fx_life::add, fx_life::plain]",
        ..Spec::default()
    }
    .extra("Contract", "\nThe fixture adds.\n");
    let fixture = Fixture::scripted(&spec);
    fixture.edit(CAP, "The fixture adds.", "The fixture adds two numbers.");
    fixture.edit(
        "src/lib.rs",
        "    a + b\n",
        "    // Explanation inside the item's span.\n    a + b\n",
    );
    fixture.edit(
        "src/lib.rs",
        "pub fn plain() {}",
        "pub fn plain() { /* Another authored explanation. */ }",
    );
    let before = fixture.snapshot();
    let refused = fixture
        .over(git_answers())
        .complete("shape")
        .expect("answers");
    assert_eq!(
        refused.headline(),
        "refused: 3 obligations, 1 section without a current accepted assessment",
        "{refused}"
    );
    assert_eq!(kinds(&refused), [ObligationKind::AssessmentStale]);
    assert_eq!(count(&refused, Condition::Assessments), 3);
    assert_eq!(refused.to_string().lines().count(), 4, "{refused}");
    assert_eq!(fixture.snapshot(), before);

    // The neighbour: two sections, each with its own text moved.
    let two = Spec::default()
        .extra("Contract", "\nThe fixture adds.\n")
        .extra("Usage", "\nCall it.\n")
        .effect("none", &["cap:fixture#contract", "cap:fixture#usage"]);
    let fixture = Fixture::scripted(&two);
    fixture.edit(CAP, "The fixture adds.", "The fixture adds two numbers.");
    fixture.edit(CAP, "Call it.", "Call it twice.");
    let refused = fixture
        .over(git_answers())
        .complete("shape")
        .expect("answers");
    assert_eq!(
        refused.headline(),
        "refused: 2 obligations, 2 sections without a current accepted assessment",
        "{refused}"
    );
}

// --- the state of a listed capability -----------------------------------------------

#[test]
fn a_capability_left_implementing_with_no_other_record_refuses_and_says_what_to_change() {
    let spec = Spec::default()
        .state("implementing")
        .examples(GOOD_LIST, &[("good", GOOD_EXAMPLE)]);
    let fixture = Fixture::real(&spec);
    let before = fixture.snapshot();
    let refused = fixture.complete();
    assert_eq!(refused.failures().len(), 1, "{refused}");
    let Some(Failure::AfterRemoval(finding)) = refused.failures().first() else {
        panic!("a finding of the repository without the record: {refused}")
    };
    assert_eq!(finding.kind, "no-open-record");
    assert_eq!(finding.file, CAP);
    assert!(
        finding.fix.contains("move the state to delivered"),
        "{finding}"
    );
    assert!(
        refused
            .to_string()
            .contains("once the record is removed, docs/capabilities/fixture.md"),
        "{refused}"
    );
    assert_eq!(fixture.snapshot(), before, "complete never edits prose");

    // The author sets the state; the neighbour completes.
    fixture.edit(CAP, "state: implementing", "state: delivered");
    fixture.resync();
    assert!(fixture.complete().is_completed());
}

#[test]
fn a_delivered_capability_beside_its_open_record_is_what_complete_resolves() {
    // The repository before the removal has a `delivered` capability and its
    // open record, which `check` reports as `state-delivered-open-record`.
    let fixture = Fixture::scripted(&Spec::default());
    let opened = fixture.gate().open().expect("opens");
    assert!(
        opened
            .workplan()
            .check()
            .iter()
            .any(|finding| finding.kind == "state-delivered-open-record")
    );
    // The state rules are judged on the repository without the record.
    let completed = fixture
        .over(passing(git_answers(), "passes"))
        .complete("shape")
        .expect("answers");
    assert!(completed.is_completed(), "{completed}");
}

#[test]
fn a_delivered_capability_with_a_binding_that_does_not_resolve_refuses() {
    let fixture = Fixture::scripted(&Spec::default().tests("[fx_life::tests::nowhere]"));
    let before = fixture.snapshot();
    let refused = fixture
        .over(git_answers())
        .complete("shape")
        .expect("answers");
    assert_eq!(kinds(&refused), [ObligationKind::BindingMissing]);
    assert!(
        refused
            .to_string()
            .contains("`fx_life::tests::nowhere` does not resolve"),
        "{refused}"
    );
    assert_eq!(fixture.snapshot(), before);

    // A delivered capability that binds no code does not load; what it owes is
    // that binding.
    let fixture = Fixture::scripted(&Spec::default());
    fixture.edit(CAP, "code: [fx_life::add]", "code: []");
    let refused = fixture
        .over(git_answers())
        .complete("shape")
        .expect("answers");
    // The catalogue no longer lists a document that does not load, so it is
    // stale too.
    assert_eq!(
        kinds(&refused),
        [
            ObligationKind::BindingMissing,
            ObligationKind::GeneratedStale
        ]
    );
    assert!(
        refused.to_string().contains("bindings.code is empty"),
        "{refused}"
    );
}

// --- empty bindings -------------------------------------------------------------------

/// A record `keep` that also lists the capability, so that the state rules hold
/// when `shape` is gone.
fn keep(spec: Spec, section: &str) -> Spec {
    spec.doc(
        "work/changes/keep.md",
        record(
            "keep",
            "[cap:fixture]",
            &[&format!("cap:fixture#{section}")],
            "none",
            "",
        ),
    )
}

#[test]
fn empty_bindings_satisfy_condition_five_only_while_the_capability_is_designed_or_considered() {
    let empty = |state| {
        let mut spec = Spec::default().state(state).tests("[]").examples("[]", &[]);
        spec.code = "[]";
        spec.lib.clear();
        spec.coverage = support::open_coverage();
        spec
    };
    // `cargo` is not scripted: a process started would be an error.
    let designed = Fixture::scripted(&keep(empty("designed"), "contract"));
    let completed = designed
        .over(git_answers())
        .complete("shape")
        .expect("no cargo process is needed");
    assert!(completed.is_completed(), "{completed}");

    let considered = Fixture::scripted(&keep(
        empty("considered").extra("Purpose", "\n> [!PROPOSED change:keep]\n> A candidate.\n"),
        "purpose",
    ));
    let completed = considered
        .over(git_answers())
        .complete("shape")
        .expect("no cargo process is needed");
    assert!(completed.is_completed(), "{completed}");

    // The same lists in `implementing` are an obligation, and the delivered
    // state refuses them too.
    let implementing = Fixture::scripted(&keep(empty("implementing"), "contract"));
    let refused = implementing
        .over(git_answers())
        .complete("shape")
        .expect("answers");
    assert_eq!(kinds(&refused), [ObligationKind::BindingMissing]);
    let text = refused.to_string();
    assert!(
        text.contains("bindings.tests is empty") && text.contains("bindings.examples is empty"),
        "{text}"
    );
}

// --- contract_effect: none --------------------------------------------------------------

#[test]
fn contract_effect_none_leaves_only_contract_untouched_and_preserves_other_refusals() {
    let sections = [
        "cap:fixture#contract",
        "cap:fixture#usage",
        "cap:fixture#examples",
    ];

    // `none` leaves out only the untouched Contract.
    let fixture = Fixture::real(&Spec::default().effect("none", &sections));
    let refused = fixture.complete();
    assert_eq!(kinds(&refused), [ObligationKind::SectionUntouched]);
    assert_eq!(refused.obligations().count(), 2, "{refused}");
    for section in ["Usage", "Examples"] {
        fixture.edit(
            CAP,
            &format!("Text for {section} of fixture."),
            &format!("Revised supported {section} of fixture."),
        );
    }
    fixture.assess();
    let completed = fixture.complete();
    assert!(completed.is_completed(), "{completed}");

    // `changes` owes each of the three.
    let fixture = Fixture::real(&Spec::default().effect("changes", &sections));
    let refused = fixture.complete();
    let owed: Vec<String> = refused.obligations().map(ToString::to_string).collect();
    assert_eq!(owed.len(), 3, "{refused}");
    assert!(
        owed.iter().all(|line| line.contains("section-untouched")),
        "{owed:?}"
    );
    assert_eq!(kinds(&refused), [ObligationKind::SectionUntouched]);

    // `none` still refuses for every other kind: here a forward-looking phrase.
    let fixture = Fixture::real(&Spec::default().effect("none", &sections));
    fixture.edit(
        CAP,
        "Text for Usage of fixture.",
        "Text for Usage of fixture. It is going to change.",
    );
    fixture.edit(
        CAP,
        "Text for Examples of fixture.",
        "Revised supported Examples of fixture.",
    );
    fixture.assess();
    let refused = fixture.complete();
    assert_eq!(
        kinds(&refused),
        [ObligationKind::TenseMismatch],
        "{refused}"
    );
}

// --- hostile bindings --------------------------------------------------------------------

#[test]
fn hostile_bindings_refuse_and_never_reach_cargo() {
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
    let quoted: &'static str = Box::leak(format!("[{quoted}]").into_boxed_str());
    let fixture = Fixture::scripted(&keep(Spec::default().state("implementing"), "contract"));
    fixture.edit(CAP, PASSES, quoted);
    fixture.edit(CAP, "examples: [fx_life::add]", "examples: [\"--exact\"]");
    fixture.resync();
    let before = fixture.snapshot();
    // Only `git` is scripted: a `cargo` process would be an unscripted call.
    let lifecycle = fixture.over(git_answers());
    let refused = lifecycle
        .complete("shape")
        .expect("a refusal and not an error");
    assert_eq!(kinds(&refused), [ObligationKind::BindingMissing]);
    assert_eq!(
        refused.obligations().count(),
        hostile.len() + 1,
        "{refused}"
    );
    for entry in hostile {
        assert!(
            refused.to_string().contains(&format!("`{entry}`")),
            "{entry}: {refused}"
        );
    }
    assert!(
        lifecycle
            .runner()
            .calls()
            .iter()
            .all(|call| call.program() == Program::Git),
        "no cargo call"
    );
    assert_eq!(count(&refused, Condition::BoundRuns), 0);
    assert_eq!(fixture.snapshot(), before);
}

// --- the order of the steps ---------------------------------------------------------------

#[test]
fn a_static_failure_spends_no_process_on_cargo_and_a_clean_change_runs_cargo_after_the_static_checks()
 {
    // A static failure and a test binding: nothing is started but `git`.
    let fixture = Fixture::scripted(&Spec::default().items(NO_OUTCOME));
    let lifecycle = fixture.over(passing(git_answers(), "passes"));
    let refused = lifecycle.complete("shape").expect("answers");
    assert!(!refused.is_completed());
    let commands: Vec<String> = lifecycle
        .runner()
        .calls()
        .iter()
        .map(Call::command)
        .collect();
    assert_eq!(programs_of(&commands), ["git"; 8], "no cargo process");

    // The neighbour: the same repository with the item resolved runs the two
    // passes of the one test, after the questions to `git`, and completes.
    let fixture = Fixture::scripted(&Spec::default());
    let lifecycle = fixture.over(passing(git_answers(), "passes"));
    assert!(lifecycle.complete("shape").expect("answers").is_completed());
    let commands: Vec<String> = lifecycle
        .runner()
        .calls()
        .iter()
        .map(Call::command)
        .collect();
    assert_eq!(
        commands[..8],
        [
            "git --version".to_string(),
            "git rev-parse --show-prefix".to_string(),
            "git symbolic-ref refs/remotes/origin/HEAD".to_string(),
            "git rev-parse --verify --end-of-options refs/remotes/origin/main^{commit}".to_string(),
            format!("git merge-base HEAD {COMMIT}"),
            format!("git ls-tree -r -z --full-tree {COMMIT}"),
            format!("git cat-file blob {COMMIT}:Cargo.toml"),
            format!("git cat-file blob {COMMIT}:src/lib.rs"),
        ]
    );
    assert_eq!(
        commands[8..],
        [
            "git symbolic-ref refs/remotes/origin/HEAD".to_string(),
            "git rev-parse --verify --end-of-options refs/remotes/origin/main^{commit}".to_string(),
            format!("git merge-base HEAD {COMMIT}"),
            "cargo test --frozen --package=fx-life --lib -- --list --exact -- tests::passes"
                .to_string(),
            "cargo test --frozen --package=fx-life --lib -- --exact -- tests::passes".to_string(),
            "cargo test --frozen --package=fx-life --doc -- --list".to_string(),
            "cargo test --frozen --package=fx-life --doc -- add".to_string(),
            "git symbolic-ref refs/remotes/origin/HEAD".to_string(),
            "git rev-parse --verify --end-of-options refs/remotes/origin/main^{commit}".to_string(),
            format!("git merge-base HEAD {COMMIT}"),
        ]
    );
}

// --- the repository changing while the bound runs go --------------------------------------

#[test]
fn a_document_edited_while_the_tests_ran_refuses_and_nothing_is_written() {
    changed_completed_view_refuses();
    let fixture = Fixture::scripted(&Spec::default());
    let text = fixture.text(CAP);
    let edited = text.replace(
        "Text for Purpose of fixture.",
        "Text for Purpose of fixture. This will be revisited.",
    );
    let runner = AfterCargo::new(passing(git_answers(), "passes"), || {
        put(fixture.root(), CAP, &edited);
    });
    let lifecycle = fixture.over(runner);
    let refused = lifecycle.complete("shape").expect("answers");
    let failures = refusal(&refused);
    assert!(
        matches!(&failures[0], Failure::Changed { .. }),
        "the change is named first: {refused}"
    );
    assert_eq!(
        kinds(&refused),
        [ObligationKind::TenseMismatch],
        "{refused}"
    );
    assert!(
        refused
            .to_string()
            .contains("the repository changed after the bound tests and examples ran"),
        "{refused}"
    );
    // Nothing was written: the record is there and the document is as the edit
    // left it.
    assert!(fixture.exists(RECORD));
    assert_eq!(fixture.text(CAP), edited);
}

fn changed_completed_view_refuses() {
    let spec = Spec { lib: LIB.replace(
        "        assert_eq!(super::add(1, 2), 3);",
        "        std::fs::write(std::path::Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"docs/capabilities/fixture.md\"), \"changed selected input\").unwrap();",
    ), ..Spec::default() };
    let fixture = Fixture::real(&spec);
    let before = fixture.snapshot();
    let registrations = git(fixture.root(), &["worktree", "list", "--porcelain"]);
    let refused = fixture.complete();
    assert!(
        matches!(refusal(&refused).first(), Some(Failure::Changed { .. })),
        "{refused}"
    );
    assert_eq!(
        fixture.snapshot(),
        before,
        "a changed execution copy publishes nothing"
    );
    assert_eq!(
        git(fixture.root(), &["worktree", "list", "--porcelain"]),
        registrations,
        "failed future-view execution releases its exact owned registration"
    );
    assert!(
        refused
            .evidence()
            .iter()
            .all(|row| row.kind() != raf_workplan::BindingKind::Example),
        "no later example consumes the changed view"
    );
}

#[test]
fn a_binding_added_while_the_tests_ran_is_a_change_even_when_every_condition_still_holds() {
    let fixture = Fixture::scripted(&Spec::default());
    let runner = AfterCargo::new(passing(git_answers(), "passes"), || {
        fixture.edit(
            CAP,
            PASSES,
            "[fx_life::tests::passes, fx_life::tests::also_passes]",
        );
        fixture.resync();
    });
    let lifecycle = fixture.over(runner);
    let refused = lifecycle.complete("shape").expect("answers");
    assert!(
        matches!(refused.failures(), [Failure::Changed { what }]
            if what.contains("differ from the ones that ran")),
        "{refused}"
    );
    assert!(fixture.exists(RECORD));
    // The test and runnable documentation example each have a list/run pair.
    assert_eq!(
        programs_of(&lifecycle.runner().commands())
            .iter()
            .filter(|p| **p == "cargo")
            .count(),
        4
    );
}

#[test]
fn a_test_renamed_while_the_tests_ran_refuses() {
    let fixture = Fixture::scripted(&Spec::default());
    let runner = AfterCargo::new(passing(git_answers(), "passes"), || {
        let lib = fixture.text("src/lib.rs");
        put(
            fixture.root(),
            "src/lib.rs",
            &lib.replace("fn passes()", "fn renamed()"),
        );
    });
    let refused = fixture.over(runner).complete("shape").expect("answers");
    assert!(
        matches!(&refused.failures()[0], Failure::Changed { .. }),
        "{refused}"
    );
    assert_eq!(
        kinds(&refused),
        [ObligationKind::BindingMissing],
        "{refused}"
    );
    assert!(fixture.exists(RECORD));
}

#[test]
fn a_record_removed_while_the_tests_ran_is_an_error_and_nothing_is_written() {
    let fixture = Fixture::scripted(&Spec::default());
    let before = fixture.text(CAP);
    let runner = AfterCargo::new(passing(git_answers(), "passes"), || {
        std::fs::remove_file(fixture.root().join(RECORD)).expect("removed");
    });
    let error = fixture
        .over(runner)
        .complete("shape")
        .expect_err("no record");
    assert!(
        matches!(
            error,
            LifecycleError::Change(ChangeLookupError::Unknown { .. })
        ),
        "{error:?}"
    );
    assert_eq!(fixture.text(CAP), before);
}

#[test]
fn a_run_that_cannot_start_is_an_error_and_not_a_refusal() {
    refused_tracking_preserves_documents();
    #[cfg(unix)]
    output_overlap_refuses_loaded_document();
    {
        let fixture = Fixture::real(&Spec::default());
        let stale = fixture.root().join(".worktrees/stale");
        git(
            fixture.root(),
            &[
                "worktree",
                "add",
                "--detach",
                stale.to_str().unwrap(),
                "HEAD",
            ],
        );
        std::fs::remove_dir_all(&stale).expect("remove only the owned fixture worktree");
        let before = fixture.snapshot();
        let error = fixture
            .lifecycle()
            .complete("shape")
            .expect_err("a registered worktree must resolve");
        let LifecycleError::Exec(ExecError::View(source)) = &error else {
            panic!("expected native preparation failure: {error:?}");
        };
        assert_eq!(source.kind(), raf_workplan::exec::ViewErrorKind::Native);
        assert!(
            error.to_string().contains(stale.to_str().unwrap()),
            "{error}"
        );
        assert!(
            error
                .to_string()
                .contains("restore access or prune its stale registration"),
            "{error}"
        );
        assert_eq!(
            std::error::Error::source(source)
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .unwrap()
                .kind(),
            std::io::ErrorKind::NotFound
        );
        assert_eq!(fixture.snapshot(), before, "preparation publishes nothing");
        git(fixture.root(), &["worktree", "prune", "--expire", "now"]);
        assert!(
            fixture.complete().is_completed(),
            "the repaired neighbor completes"
        );
    }
    let fixture = Fixture::real(&Spec::default());
    let lifecycle = Lifecycle::open(fixture.programs_without_cargo()).expect("opens");
    let before = fixture.snapshot();
    let error = lifecycle.complete("shape").expect_err("no cargo");
    assert!(
        matches!(error, LifecycleError::Exec(ExecError::Spawn { .. })),
        "{error:?}"
    );
    assert_eq!(fixture.snapshot(), before);
}

#[cfg(unix)]
fn output_overlap_refuses_loaded_document() {
    use purrdf_lex::json::{Object, Value, write_compact};
    use std::os::unix::fs::PermissionsExt as _;
    for case in 0..4 {
        let document_overlap = case == 0;
        let malformed_output = case == 2;
        let external_asset = case == 3;
        let fixture = Fixture::real(&Spec::default());
        let assessment = "work/assessments/fixture-contract.md";
        let output = if document_overlap {
            git(fixture.root(), &["rm", "--cached", "--", assessment]);
            assert!(
                fixture.exists(assessment),
                "the untracked assessment is still loaded"
            );
            fixture.root().join("work/assessments")
        } else if malformed_output {
            fixture.root().join("native-output/..")
        } else if external_asset {
            let output = fixture.scratch.target();
            std::fs::write(output.join("payload"), "generated bytes").unwrap();
            std::fs::create_dir(fixture.root().join(".private")).unwrap();
            std::os::unix::fs::symlink(
                output.join("payload"),
                fixture.root().join(".private/selected-asset"),
            )
            .unwrap();
            output
        } else {
            std::fs::create_dir(fixture.root().join("native-output")).unwrap();
            put(fixture.root(), "native-output/payload", "generated bytes");
            std::os::unix::fs::symlink(
                "native-output/payload",
                fixture.root().join("selected-asset"),
            )
            .unwrap();
            fixture.root().join("native-output")
        };
        let metadata = Value::from(
            Object::new()
                .with("version", 1_u64)
                .with("workspace_root", fixture.root().to_str().unwrap())
                .with("target_directory", output.to_str().unwrap()),
        );
        let program = fixture.root().join("metadata-cargo");
        put(
            fixture.root(),
            "metadata-cargo",
            &format!(
                "#!/bin/sh\n[ \"$1\" = metadata ] || exit 71\ncat <<'RAF_METADATA'\n{}\nRAF_METADATA\n",
                write_compact(&metadata)
            ),
        );
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        let before = fixture.snapshot();
        let registrations = git(fixture.root(), &["worktree", "list", "--porcelain"]);
        let error = Lifecycle::open(fixture.programs().with_cargo(program))
            .unwrap()
            .complete("shape")
            .expect_err("loaded assessment cannot be excluded as output");
        assert!(
            matches!(&error, LifecycleError::Exec(ExecError::View(source)) if source.kind() == raf_workplan::exec::ViewErrorKind::Refused),
            "{error:?}"
        );
        if !malformed_output && !external_asset {
            assert!(error.to_string().contains("excluded"), "{error}");
        }
        assert!(
            error.to_string().contains(if malformed_output {
                "target_directory must be absolute without parent components"
            } else if document_overlap {
                "loaded input work/assessments/fixture-contract.md"
            } else if external_asset {
                "does not name its owning Cargo workspace's exact output root"
            } else {
                "selected link depends"
            }),
            "{error}"
        );
        assert_eq!(fixture.snapshot(), before);
        assert_eq!(
            git(fixture.root(), &["worktree", "list", "--porcelain"]),
            registrations
        );
    }
}

// --- the order of the writes --------------------------------------------------------------

#[cfg(unix)]
mod writes {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    use super::*;

    /// A directory made read-only, restored when dropped so the scratch
    /// directory can be removed.
    struct ReadOnly(PathBuf, fs::Permissions);

    impl ReadOnly {
        /// Returns `None` when the permission does not bind this process (it is
        /// privileged), so a test of a failing write cannot run.
        fn new(dir: &Path) -> Option<Self> {
            let permissions = fs::metadata(dir).expect("a directory").permissions();
            let mut closed = permissions.clone();
            closed.set_mode(0o555);
            fs::set_permissions(dir, closed).expect("closes");
            let guard = Self(dir.to_path_buf(), permissions);
            let probe = dir.join(".probe");
            if fs::write(&probe, b"x").is_ok() {
                let _ = fs::remove_file(&probe);
                eprintln!("{} is writable although read-only: skipped", dir.display());
                return None;
            }
            Some(guard)
        }
    }

    impl Drop for ReadOnly {
        fn drop(&mut self) {
            let _ = fs::set_permissions(&self.0, self.1.clone());
        }
    }

    /// The invariant of the order: a repository without the record never holds
    /// stale generated blocks.
    fn not_removed_beside_stale_blocks(fixture: &Fixture) {
        if !fixture.exists(RECORD) {
            Gate::workspace(fixture.root())
                .programs(fixture.programs())
                .assert_clean();
        }
    }

    #[test]
    fn a_rewrite_that_fails_leaves_the_record_and_the_blocks_as_they_were() {
        let fixture = Fixture::scripted(&Spec::default());
        let before = fixture.snapshot();
        {
            let Some(_closed) = ReadOnly::new(&fixture.root().join("docs/capabilities")) else {
                return;
            };
            let error = fixture
                .over(passing(git_answers(), "passes"))
                .complete("shape")
                .expect_err("the rewrite fails");
            assert!(matches!(error, LifecycleError::Rewrite { .. }), "{error:?}");
            assert!(
                error.to_string().contains("the record was not removed"),
                "{error}"
            );
            // The record is still there, so no removed record beside stale blocks.
            assert!(fixture.exists(RECORD));
            not_removed_beside_stale_blocks(&fixture);
            assert_eq!(fixture.snapshot(), before);
        }
        // The neighbour: with the directory writable again, it completes.
        let completed = fixture
            .over(passing(git_answers(), "passes"))
            .complete("shape")
            .expect("answers");
        assert!(completed.is_completed(), "{completed}");
        not_removed_beside_stale_blocks(&fixture);
    }

    #[test]
    fn a_removal_that_fails_reports_the_files_already_rewritten_and_removes_nothing_else() {
        let fixture = Fixture::scripted(&Spec::default());
        let cap_before = fixture.text(CAP);
        {
            let Some(_closed) = ReadOnly::new(&fixture.root().join("work/changes")) else {
                return;
            };
            let error = fixture
                .over(passing(git_answers(), "passes"))
                .complete("shape")
                .expect_err("the removal fails");
            let LifecycleError::Remove {
                path, rewritten, ..
            } = &error
            else {
                panic!("a removal error: {error:?}")
            };
            assert_eq!(path, RECORD);
            assert_eq!(rewritten, &[CAP.to_string()]);
            assert!(error.to_string().contains(CAP), "{error}");
            // The rewrite was done first, so the record that is left lists a
            // capability whose table no longer names it, and the record is the
            // only thing that was not removed.
            assert!(fixture.exists(RECORD));
            assert_ne!(fixture.text(CAP), cap_before);
        }
        // The author removes the record by hand: the repository is clean.
        fs::remove_file(fixture.root().join(RECORD)).expect("removed by hand");
        not_removed_beside_stale_blocks(&fixture);
    }
}

#[test]
fn an_error_after_the_rewrite_does_not_leave_a_removed_record_beside_stale_blocks() {
    // The order is observable without a failing disk: when the removal is the
    // last write, a repository in which the record is gone has current blocks.
    let fixture = Fixture::scripted(&Spec::default());
    let before = fixture.snapshot();
    let completed = fixture
        .over(passing(git_answers(), "passes"))
        .complete("shape")
        .expect("answers");
    assert!(completed.is_completed());
    assert!(!fixture.exists(RECORD));
    Gate::workspace(fixture.root())
        .programs(fixture.programs())
        .assert_clean();
    assert_eq!(
        support::disk::changed(&before, &fixture.snapshot()),
        [CAP.to_string(), RECORD.to_string()]
    );
}

// --- a link to the record ------------------------------------------------------------------

/// Prose, outside every generated block, that links to the record.
const LINK_TO_RECORD: &str = "\nSee [the record](../../work/changes/shape.md).\n";

#[test]
fn prose_that_links_to_the_record_refuses_because_the_link_is_broken_once_it_is_removed() {
    let fixture = Fixture::scripted(&Spec::default().extra("Purpose", LINK_TO_RECORD));
    // Before the removal the link resolves: the record is tracked.
    assert!(
        fixture
            .gate()
            .open()
            .expect("opens")
            .workplan()
            .check()
            .iter()
            .all(|finding| finding.kind != "link-broken")
    );
    let before = fixture.snapshot();
    let record_before = fixture.text(RECORD);
    let refused = fixture
        .over(passing(git_answers(), "passes"))
        .complete("shape")
        .expect("answers");
    let failures = refusal(&refused);
    assert_eq!(failures.len(), 1, "{refused}");
    let Some(Failure::AfterRemoval(finding)) = failures.first() else {
        panic!("a finding of the repository without the record: {refused}")
    };
    assert_eq!(finding.kind, "link-broken");
    assert_eq!(finding.file, CAP);
    assert!(
        finding.message.contains("../../work/changes/shape.md"),
        "{finding}"
    );
    assert!(
        refused.to_string().contains("[link-broken]"),
        "the refusal names the link: {refused}"
    );
    assert_eq!(fixture.text(RECORD), record_before);
    assert_eq!(fixture.snapshot(), before);

    // The neighbour: with the link removed it completes.
    fixture.edit(CAP, LINK_TO_RECORD, "");
    let completed = fixture
        .over(passing(git_answers(), "passes"))
        .complete("shape")
        .expect("answers");
    assert!(completed.is_completed(), "{completed}");
    assert!(!fixture.exists(RECORD));
}

#[test]
fn a_broken_link_in_a_capability_the_change_does_not_list_also_blocks_completion() {
    let fixture = Fixture::scripted(&other_record(Spec::default()));
    fixture.edit(
        "docs/capabilities/second.md",
        "Text for Purpose of second.",
        "Text for Purpose of second. See [nothing](./nowhere.md).",
    );
    let before = fixture.snapshot();
    let refused = fixture
        .over(passing(git_answers(), "passes"))
        .complete("shape")
        .expect("answers");
    assert!(
        matches!(refused.failures(), [Failure::AfterRemoval(finding)]
            if finding.kind == "link-broken" && finding.file == "docs/capabilities/second.md"),
        "{refused}"
    );
    assert_eq!(fixture.snapshot(), before);

    fixture.edit(
        "docs/capabilities/second.md",
        " See [nothing](./nowhere.md).",
        "",
    );
    assert!(
        fixture
            .over(passing(git_answers(), "passes"))
            .complete("shape")
            .expect("answers")
            .is_completed()
    );
}

#[test]
fn an_outcome_the_profile_refuses_makes_the_record_not_load_and_is_an_error() {
    let fixture = Fixture::scripted(&Spec::default());
    // A sync refuses a record that does not load, so the value is written after.
    fixture.edit(RECORD, "outcome: implemented", "outcome: finished");
    let before = fixture.snapshot();
    let error = fixture
        .over(git_answers())
        .complete("shape")
        .expect_err("the record does not load");
    assert!(
        matches!(
            error,
            LifecycleError::Change(ChangeLookupError::NotLoaded { .. })
        ),
        "{error:?}"
    );
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn conditions_and_failures_have_stable_names_and_the_headline_is_the_first_line() {
    for (condition, name, number) in [
        (Condition::Obligations, "obligations", Some(1)),
        (Condition::Outcomes, "outcomes", Some(2)),
        (Condition::Proposals, "proposals", Some(3)),
        (Condition::GeneratedBlocks, "generated-blocks", Some(4)),
        (Condition::BoundRuns, "bound-runs", Some(5)),
        (Condition::Assessments, "assessments", Some(6)),
        (Condition::AfterRemoval, "after-removal", None),
        (Condition::ChangedDuringRun, "changed-during-run", None),
    ] {
        assert_eq!(condition.name(), name);
        assert_eq!(condition.number(), number);
    }

    let fixture = Fixture::scripted(&Spec::default().items(NO_OUTCOME));
    fixture.edit(CAP, "description: Shaped.", "description: Reshaped.");
    let refused = fixture
        .over(passing(git_answers(), "passes"))
        .complete("shape")
        .expect("answers");
    assert_eq!(
        refused.headline(),
        refused.to_string().lines().next().expect("a first line")
    );
    assert!(refused.headline().starts_with("refused: "));
    let failures = refused.failures();
    for failure in failures {
        let repeated = failure.is_repeated_in(failures);
        match failure {
            Failure::Owed(obligation) => {
                assert!(!repeated, "{failure}");
                assert_eq!(failure.name(), obligation.kind.name());
            }
            Failure::Item(_) => {
                assert!(repeated);
                assert_eq!(failure.name(), "item-without-outcome");
            }
            Failure::Generated(finding) => {
                assert!(repeated, "{failure}");
                assert_eq!(failure.name(), finding.kind);
            }
            other => panic!("unexpected failure {other}"),
        }
    }
    assert_eq!(
        failures
            .iter()
            .filter(|failure| !failure.is_repeated_in(failures))
            .count(),
        refused.to_string().lines().count() - 1
    );

    // The other names, from values of the other variants.
    let completed = Fixture::real(&Spec::default()).complete();
    assert!(completed.is_completed());
    assert_eq!(
        completed.headline(),
        "completed: removed work/changes/shape.md"
    );
    assert_eq!(
        completed.to_string().lines().next(),
        Some(completed.headline().as_str())
    );
}

// --- the change lifecycle under strict mode -----------------------------------------

/// The library with the first `from` replaced by `to`.
fn lib_with(from: &str, to: &str) -> String {
    assert!(LIB.contains(from), "the library has no `{from}`");
    LIB.replacen(from, to, 1)
}

/// The strict-ready fixture once the library is `lib` and the affected
/// sections are assessed again.
fn edited(spec: &Spec, lib: &str) -> Fixture {
    let fixture = Fixture::real(spec);
    put(fixture.root(), "src/lib.rs", lib);
    fixture.assess();
    fixture
}

/// A public-token change refuses without writes until its effect is declared
/// and the affected Contract is revised and assessed.
fn refuses_public_token_changes_until_the_effect_is_declared(
    spec: &Spec,
    lib: &str,
    items: &[&str],
) {
    let fixture = edited(spec, lib);
    let before = fixture.snapshot();
    let refused = fixture.complete();
    let failures = refusal(&refused);
    assert!(
        failures
            .iter()
            .all(|failure| failure.obligation_kind()
                == Some(ObligationKind::ContractEffectUnderstated)),
        "{refused}"
    );
    assert_eq!(
        count(&refused, Condition::Obligations),
        items.len(),
        "{refused}"
    );
    let plural = if items.len() == 1 { "" } else { "s" };
    assert_eq!(
        refused.headline(),
        format!("refused: {} obligation{plural}", items.len())
    );
    for item in items {
        assert!(
            refused
                .to_string()
                .contains(&format!("(`{item}` at src/lib.rs:")),
            "{item}: {refused}"
        );
    }
    for line in refused.to_string().lines().skip(1) {
        assert!(
            line.starts_with("change:shape  document  contract-effect-understated  contract_effect: changes, the capability listed and the sections it owes reconciled (`"),
            "{line}"
        );
        assert!(line.ends_with("; `cap:fixture` binds it)"), "{line}");
    }
    assert_eq!(fixture.snapshot(), before, "a refusal writes nothing");

    assert_eq!(
        fixture.complete(),
        refused,
        "an unchanged refusal is stable"
    );
    declare_effect(&fixture);
    let completed = fixture.complete();
    assert!(completed.is_completed(), "{completed}");
}

fn declare_effect(fixture: &Fixture) {
    fixture.edit(RECORD, "contract_effect: none", "contract_effect: changes");
    fixture.edit(
        CAP,
        "Text for Contract of fixture.",
        "Revised supported Contract of fixture.",
    );
    fixture.assess();
}

/// Under strict mode the same change completes: nothing it touches is a token
/// of a bound public item.
fn completed_under_strict(spec: &Spec, lib: &str) {
    let fixture = edited(spec, lib);
    let owed = fixture
        .lifecycle()
        .reconcile("shape")
        .unwrap_or_else(|error| panic!("reconcile answers: {error}"));
    assert!(owed.is_empty(), "{owed}");
    let completed = fixture.complete();
    assert!(completed.is_completed(), "{completed}");
}

#[test]
fn a_none_change_that_edits_a_bound_signature_refuses() {
    refuses_public_token_changes_until_the_effect_is_declared(
        &strict_ready(),
        &lib_with("pub fn add(", "pub const fn add("),
        &["fx_life::add"],
    );
}

#[test]
fn an_edit_of_a_bound_body_alone_refuses() {
    refuses_public_token_changes_until_the_effect_is_declared(
        &strict_ready(),
        &edited_lib(),
        &["fx_life::add"],
    );
}

#[test]
fn an_edit_of_a_bound_doc_comment_refuses() {
    refuses_public_token_changes_until_the_effect_is_declared(
        &strict_ready(),
        &lib_with("/// Adds.\n", "/// Adds two numbers.\n"),
        &["fx_life::add"],
    );
}

#[test]
fn a_changed_attribute_of_a_bound_item_refuses() {
    refuses_public_token_changes_until_the_effect_is_declared(
        &strict_ready(),
        &lib_with("pub fn add(", "#[must_use]\npub fn add("),
        &["fx_life::add"],
    );
}

#[test]
fn a_new_bound_public_item_refuses() {
    let fixture = Fixture::real(&strict_ready());
    put(
        fixture.root(),
        "src/lib.rs",
        &lib_with(
            "pub fn plain() {}\n",
            "pub fn plain() {}\n\n/// Subtracts.\npub fn sub(a: u32, b: u32) -> u32 {\n    a - b\n}\n",
        ),
    );
    fixture.edit(
        CAP,
        "code: [fx_life::add]",
        "code: [fx_life::add, fx_life::sub]",
    );
    fixture.resync();
    fixture.assess();
    let before = fixture.snapshot();
    let refused = fixture.complete();
    assert_eq!(kinds(&refused), [ObligationKind::ContractEffectUnderstated]);
    assert_eq!(refused.headline(), "refused: 1 obligation");
    assert!(
        refused.to_string().contains(
            "(`fx_life::sub` at src/lib.rs:15: the merge base has no such public item; `cap:fixture` binds it)"
        ),
        "{refused}"
    );
    assert_eq!(fixture.snapshot(), before, "a refusal writes nothing");
    declare_effect(&fixture);
    let completed = fixture.complete();
    assert!(completed.is_completed(), "{completed}");
}

/// A bound module whose item is renamed: the module's tokens changed, the new
/// name is a new item, and the old one is gone from the working tree.
#[test]
fn an_item_renamed_below_a_bound_module_refuses() {
    let module = "\npub mod shapes {\n    /// The area of a square.\n    pub fn square(side: u32) -> u32 {\n        side * side\n    }\n}\n";
    let mut spec = strict_ready();
    spec.lib = format!("{LIB}{module}");
    spec.code = "[fx_life::add, fx_life::shapes]";
    let fixture = edited(&spec, &spec.lib.replace("pub fn square(", "pub fn area("));
    let before = fixture.snapshot();
    let refused = fixture.complete();
    assert_eq!(kinds(&refused), [ObligationKind::ContractEffectUnderstated]);
    assert_eq!(refused.headline(), "refused: 3 obligations");
    let text = refused.to_string();
    for wanted in [
        "(`fx_life::shapes` at src/lib.rs:",
        "(`fx_life::shapes::area` at src/lib.rs:",
        "(`fx_life::shapes::square` at src/lib.rs:",
    ] {
        assert!(text.contains(wanted), "{wanted}: {text}");
    }
    assert!(
        text.contains("(`fx_life::shapes::square` at src/lib.rs:32 at the merge base: the working tree has no such public item; `cap:fixture` binds it)"),
        "{text}"
    );
    assert_eq!(fixture.snapshot(), before, "a refusal writes nothing");
    declare_effect(&fixture);
    let completed = fixture.complete();
    assert!(completed.is_completed(), "{completed}");
}

#[test]
fn a_reformatted_bound_body_and_a_plain_comment_complete() {
    let lib = lib_with(
        "pub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n",
        "pub fn add(\n    a: u32,\n    b: u32\n) -> u32 {\n    // The sum, in the order of the arguments.\n    a +   b /* no carry */\n}\n",
    );
    completed_under_strict(&strict_ready(), &lib);
}

#[test]
fn a_plain_comment_outside_every_bound_item_completes() {
    let lib = lib_with(
        "pub fn plain() {}\n",
        "// A helper that no capability binds.\npub fn plain() {}\n\n// The tests below.\n",
    );
    completed_under_strict(&strict_ready(), &lib);
}

#[test]
fn an_item_moved_within_its_file_completes() {
    let add = "/// Adds.\n///\n/// ```\n/// assert_eq!(fx_life::add(1, 2), 3);\n/// ```\npub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n";
    let lib = lib_with(
        &format!("{add}\npub fn plain() {{}}\n"),
        &format!("pub fn plain() {{}}\n\n{add}"),
    );
    assert!(lib.find("pub fn plain").expect("plain") < lib.find("pub fn add").expect("add"));
    completed_under_strict(&strict_ready(), &lib);
}

#[test]
fn a_bound_item_moved_to_its_module_file_keeps_identity_but_changed_tokens_refuse() {
    let square =
        "/// The area of a square.\npub fn square(side: u32) -> u32 {\n    side * side\n}\n";
    // The public re-export binds the function without binding its private module.
    // Changing the module wrapper therefore cannot mask the function's identity.
    let base = format!("{LIB}\nmod shapes {{\n{square}}}\npub use shapes::square;\n");
    let spec = with("[fx_life::add, fx_life::square]", &base, &[]);
    let moved = format!("{LIB}\nmod shapes;\npub use shapes::square;\n");
    untouched_under_strict(&spec, &|fixture| {
        put_lib(fixture, &moved);
        put(fixture.root(), "src/shapes.rs", square);
    });

    let changed = square.replace("side * side", "side * side + 1");
    touched_under_strict(
        &spec,
        &|fixture| {
            put_lib(fixture, &moved);
            put(fixture.root(), "src/shapes.rs", &changed);
        },
        &[("fx_life::shapes::square", "src/shapes.rs", TOKENS)],
    );
}

/// Repeated reconciliation reports the public-token obligation until repaired.
#[test]
fn a_none_change_repeatedly_owes_its_public_contract_obligation_until_repaired() {
    let fixture = edited(&strict_ready(), &edited_lib());
    let owed = fixture.lifecycle().reconcile("shape").expect("answers");
    let strict = fixture.lifecycle().reconcile("shape").expect("answers");
    assert_eq!(owed.obligations(), strict.obligations());
    assert_eq!(
        strict
            .obligations()
            .iter()
            .map(|owed| owed.kind)
            .collect::<Vec<_>>(),
        [ObligationKind::ContractEffectUnderstated]
    );
    declare_effect(&fixture);
    assert!(
        fixture
            .lifecycle()
            .reconcile("shape")
            .expect("repaired")
            .is_empty()
    );
}

#[test]
fn an_item_dropped_without_a_reason_refuses_condition_two() {
    let fixture = Fixture::real(&strict_ready().items(DROPPED));
    let before = fixture.snapshot();
    let refused = fixture.complete();
    assert_eq!(count(&refused, Condition::Outcomes), 1, "{refused}");
    assert_eq!(count(&refused, Condition::Obligations), 1, "{refused}");
    assert_eq!(refusal(&refused).len(), 2, "{refused}");
    assert_eq!(kinds(&refused), [ObligationKind::OutcomeWithoutReason]);
    assert_eq!(
        refused.to_string(),
        "refused: 1 obligation, 1 item without a reason\nchange:shape  item 1  outcome-without-reason  a reason for the outcome (`outcome: dropped` has no `reason`)\n"
    );
    assert_eq!(fixture.snapshot(), before, "a refusal writes nothing");
    assert_eq!(
        fixture.complete(),
        refused,
        "an unchanged refusal is stable"
    );
    fixture.edit(RECORD, DROPPED, DROPPED_WITH_REASON);
    let completed = fixture.complete();
    assert!(completed.is_completed(), "{completed}");

    // The neighbour: the item with its reason completes under strict.
    let fixture = Fixture::real(&strict_ready().items(DROPPED_WITH_REASON));
    let completed = fixture.complete();
    assert!(completed.is_completed(), "{completed}");
}

/// An untouched Usage or Examples remains owed until that section is revised.
#[test]
fn an_untouched_usage_or_examples_of_a_none_change_refuses() {
    for (section, affects) in [
        ("usage", "cap:fixture#usage"),
        ("examples", "cap:fixture#examples"),
    ] {
        let fixture =
            Fixture::real(&strict_ready().effect("none", &["cap:fixture#contract", affects]));
        let before = fixture.snapshot();
        let refused = fixture.complete();
        assert_eq!(
            kinds(&refused),
            [ObligationKind::SectionUntouched],
            "{refused}"
        );
        assert_eq!(
            refused.to_string(),
            format!(
                "refused: 1 obligation\ncap:fixture  {section}  section-untouched  a diff of that section against the merge base, or an edit that records why none is needed\n"
            )
        );
        assert_eq!(
            fixture.snapshot(),
            before,
            "{section}: a refusal writes nothing"
        );
        assert_eq!(
            fixture.complete(),
            refused,
            "an unchanged refusal is stable"
        );
        let name = if section == "usage" {
            "Usage"
        } else {
            "Examples"
        };
        fixture.edit(
            CAP,
            &format!("Text for {name} of fixture."),
            &format!("Revised supported {name} of fixture."),
        );
        fixture.assess();
        let completed = fixture.complete();
        assert!(completed.is_completed(), "{section}: {completed}");
    }
}

/// Repeated completion preserves test and wording refusals until their actual
/// repairs; condition 5 and condition 1 keep their distinct failure kinds.
#[test]
fn a_failing_bound_test_and_a_tense_mismatch_refuse_until_their_repairs_complete() {
    let summary = |completion: &Completion| -> Vec<(Condition, &'static str)> {
        completion
            .failures()
            .iter()
            .map(|failure| (failure.condition(), failure.name()))
            .collect()
    };
    let failing =
        Fixture::real(&strict_ready().tests("[fx_life::tests::passes, fx_life::tests::fails]"));
    let before = failing.snapshot();
    let strict = failing.complete();
    let default = failing.complete();
    assert_eq!(
        summary(&strict),
        [(Condition::BoundRuns, "bound-test")],
        "{strict}"
    );
    assert_eq!(summary(&strict), summary(&default));
    assert_eq!(strict.headline(), default.headline());
    assert_eq!(failing.snapshot(), before, "a refusal writes nothing");
    failing.rebind_and_assess_tests("[fx_life::tests::passes, fx_life::tests::fails]", PASSES);
    failing.resync();
    failing.assess();
    let completed = failing.complete();
    assert!(completed.is_completed(), "{completed}");

    let tense = Fixture::real(&strict_ready());
    tense.edit(
        CAP,
        "Text for Purpose of fixture.",
        "Text for Purpose of fixture. This will be revisited.",
    );
    let before = tense.snapshot();
    let strict = tense.complete();
    let default = tense.complete();
    assert_eq!(kinds(&strict), [ObligationKind::TenseMismatch], "{strict}");
    assert_eq!(strict, default);
    assert_eq!(tense.snapshot(), before, "a refusal writes nothing");
    tense.edit(
        CAP,
        "Text for Purpose of fixture. This will be revisited.",
        "Text for Purpose of fixture.",
    );
    let completed = tense.complete();
    assert!(completed.is_completed(), "{completed}");
}

// --- what strict mode reads as a touch of a bound public item ---------------------

/// What the evidence of `contract-effect-understated` says, by how the item
/// differs from the merge base.
const TOKENS: &str = "its tokens differ from those it had at the merge base";
const NEW: &str = "the merge base has no such public item";
const REMOVED: &str = "the working tree has no such public item";
const HIDDEN: &str = "it was public at the merge base and is off the public surface now";
const RETARGETED: &str = "it names another item than at the merge base";

/// The strict-ready fixture whose `code` bindings are `code`, whose library
/// is `lib` and which holds the other source files `src`.
fn with(code: &'static str, lib: &str, src: &[(&str, &str)]) -> Spec {
    let mut spec = strict_ready();
    spec.code = code;
    spec.lib = lib.to_string();
    spec.src = src
        .iter()
        .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
        .collect();
    spec
}

/// The real fixture of `spec` once `edit` has changed a file, synced and
/// assessed again.
fn after(spec: &Spec, edit: &dyn Fn(&Fixture)) -> Fixture {
    let fixture = Fixture::real(spec);
    let pristine = fixture.snapshot();
    edit(&fixture);
    assert_ne!(fixture.snapshot(), pristine, "the edit changes a file");
    fixture.resync();
    fixture.assess();
    fixture
}

/// Write `lib` as the library of the fixture.
fn put_lib(fixture: &Fixture, lib: &str) {
    put(fixture.root(), "src/lib.rs", lib);
}

/// A changed public contract refuses without writes, naming every changed item,
/// until the declared effect and assessed Contract explain the change.
fn touched_under_strict(spec: &Spec, edit: &dyn Fn(&Fixture), owed: &[(&str, &str, &str)]) {
    let fixture = after(spec, edit);
    let before = fixture.snapshot();
    let refused = fixture.complete();
    assert_eq!(
        kinds(&refused),
        [ObligationKind::ContractEffectUnderstated],
        "{refused}"
    );
    let text = refused.to_string();
    let lines: Vec<&str> = text.lines().skip(1).collect();
    assert_eq!(lines.len(), owed.len(), "{refused}");
    for (item, file, what) in owed {
        let says = |line: &&&str| {
            line.contains(&format!("(`{item}` at {file}:"))
                && line.ends_with(&format!(": {what}; `cap:fixture` binds it)"))
        };
        assert_eq!(lines.iter().filter(says).count(), 1, "{item}: {refused}");
    }
    assert_eq!(fixture.snapshot(), before, "a refusal writes nothing");
    assert_eq!(
        fixture.complete(),
        refused,
        "an unchanged refusal is stable"
    );
    declare_effect(&fixture);
    let completed = fixture.complete();
    assert!(completed.is_completed(), "{completed}");
}

/// Under strict mode the change that `edit` makes owes nothing and completes.
fn untouched_under_strict(spec: &Spec, edit: &dyn Fn(&Fixture)) {
    let fixture = after(spec, edit);
    let owed = fixture
        .lifecycle()
        .reconcile("shape")
        .unwrap_or_else(|error| panic!("reconcile answers: {error}"));
    assert!(owed.is_empty(), "{owed}");
    let completed = fixture.complete();
    assert!(completed.is_completed(), "{completed}");
}

const MUL: &str = "\n/// Multiplies.\npub fn mul(a: u32, b: u32) -> u32 {\n    a * b\n}\n";

/// A bound type with an inherent impl beside it.
const FRAME: &str = "\n/// A frame.\npub struct Frame;\n\nimpl Frame {\n    /// Pushes.\n    pub fn push(&self) -> u32 {\n        1\n    }\n}\n";

#[test]
fn an_edited_public_item_of_a_bound_binary_refuses_until_the_effect_is_repaired() {
    let binary =
        "/// Returns a number.\npub fn shown() -> u32 { 1 }\nfn main() { let _ = shown(); }\n";
    let spec = with(
        "[fx_life::add, fx_life::bin_crate::fx_life::shown, fx_life::bin_crate::fx_life::main]",
        LIB,
        &[("src/main.rs", binary)],
    );
    let changed = binary.replace("-> u32 { 1 }", "-> u32 { 2 }");
    touched_under_strict(
        &spec,
        &|fixture| put(fixture.root(), "src/main.rs", &changed),
        &[("fx_life::bin_crate::fx_life::shown", "src/main.rs", TOKENS)],
    );
}

#[test]
fn a_bound_item_made_private_or_visible_to_its_crate_alone_refuses() {
    let base = format!("{LIB}{MUL}");
    let spec = with("[fx_life::add, fx_life::mul]", &base, &[]);
    for narrowed in [
        "#[allow(dead_code)]\nfn mul",
        "#[allow(dead_code)]\npub(crate) fn mul",
    ] {
        let lib = base.replace("pub fn mul", narrowed);
        touched_under_strict(
            &spec,
            &|fixture| put_lib(fixture, &lib),
            &[("fx_life::mul", "src/lib.rs", HIDDEN)],
        );
    }
}

#[test]
fn a_bound_module_made_private_refuses_for_it_and_its_items() {
    let base = format!(
        "{LIB}\npub mod shapes {{\n    /// The area of a square.\n    pub fn square(side: u32) -> u32 {{\n        side * side\n    }}\n}}\n"
    );
    let spec = with("[fx_life::add, fx_life::shapes]", &base, &[]);
    let lib = base.replace("pub mod shapes", "#[allow(dead_code)]\nmod shapes");
    touched_under_strict(
        &spec,
        &|fixture| put_lib(fixture, &lib),
        &[
            ("fx_life::shapes", "src/lib.rs", HIDDEN),
            ("fx_life::shapes::square", "src/lib.rs", HIDDEN),
        ],
    );
}

#[test]
fn a_public_method_of_a_bound_type_made_private_refuses() {
    let base = format!("{LIB}{FRAME}");
    let spec = with("[fx_life::add, fx_life::Frame]", &base, &[]);
    let lib = base.replace("    pub fn push", "    #[allow(dead_code)]\n    fn push");
    touched_under_strict(
        &spec,
        &|fixture| put_lib(fixture, &lib),
        &[("fx_life::Frame::push", "src/lib.rs", HIDDEN)],
    );
}

/// The header of an impl block is part of the identity of each member: a
/// changed generic bound, or an attribute such as `#[cfg(any())]` that takes
/// the methods away, makes each bound member new.
#[test]
fn a_changed_header_of_the_impl_block_of_a_bound_type_refuses_for_its_member() {
    let base = format!(
        "{LIB}\n/// A wrapper.\npub struct Wrap<T>(pub T);\n\nimpl<T: Clone> Wrap<T> {{\n    /// The value.\n    pub fn get(&self) -> T {{\n        self.0.clone()\n    }}\n}}\n"
    );
    let spec = with("[fx_life::add, fx_life::Wrap]", &base, &[]);
    let lib = base.replace("impl<T: Clone>", "impl<T: Clone + Default>");
    touched_under_strict(
        &spec,
        &|fixture| put_lib(fixture, &lib),
        &[("fx_life::Wrap::get", "src/lib.rs", NEW)],
    );

    let base = format!("{LIB}{FRAME}");
    let spec = with("[fx_life::add, fx_life::Frame]", &base, &[]);
    let lib = base.replace("impl Frame {", "#[cfg(any())]\nimpl Frame {");
    touched_under_strict(
        &spec,
        &|fixture| put_lib(fixture, &lib),
        &[("fx_life::Frame::push", "src/lib.rs", NEW)],
    );
}

/// Each item is compared with its own counterpart, so a method edited to
/// the tokens of a method of another type of the same file is touched.
#[test]
fn a_method_edited_to_the_tokens_of_a_sibling_method_refuses() {
    let base = format!(
        "{LIB}\n/// One.\npub struct A;\n\n/// Two.\npub struct B;\n\nimpl A {{\n    /// The value.\n    pub fn get(&self) -> u32 {{\n        2\n    }}\n}}\n\nimpl B {{\n    /// The value.\n    pub fn get(&self) -> u32 {{\n        1\n    }}\n}}\n"
    );
    let spec = with("[fx_life::add, fx_life::A, fx_life::B]", &base, &[]);
    let lib = base.replacen("        2\n", "        1\n", 1);
    touched_under_strict(
        &spec,
        &|fixture| put_lib(fixture, &lib),
        &[("fx_life::A::get", "src/lib.rs", TOKENS)],
    );
}

/// A bound type whose inherent impl is in another module and file.
const FRAME_LIB: &str = "\n/// A frame.\npub struct Frame;\n\nmod imp;\n";
const IMP: &str =
    "impl crate::Frame {\n    /// Pushes.\n    pub fn push(&self) -> u32 {\n        1\n    }\n}\n";

/// The members of a bound type are compared across the two source indexes,
/// so a method removed from an impl in another module, or with the file of
/// that impl, is found although the binding of the type still resolves.
#[test]
fn a_method_removed_from_an_impl_in_another_file_or_with_its_file_refuses() {
    let spec = with(
        "[fx_life::add, fx_life::Frame]",
        &format!("{LIB}{FRAME_LIB}"),
        &[("src/imp.rs", IMP)],
    );
    touched_under_strict(
        &spec,
        &|fixture| put(fixture.root(), "src/imp.rs", "impl crate::Frame {}\n"),
        &[("fx_life::Frame::push", "src/imp.rs", REMOVED)],
    );
    touched_under_strict(
        &spec,
        &|fixture| {
            std::fs::remove_file(fixture.root().join("src/imp.rs")).expect("removes the file");
            put_lib(
                fixture,
                &format!("{LIB}\n/// A frame.\npub struct Frame;\n"),
            );
        },
        &[("fx_life::Frame::push", "src/imp.rs", REMOVED)],
    );
}

/// A file that is not at the merge base has nothing to compare with, so each
/// bound public item of it is new; a new file of private items completes.
#[test]
fn the_bound_items_of_a_file_absent_at_the_merge_base_refuse_and_a_new_file_of_private_items_completes()
 {
    let spec = with("[fx_life::add]", LIB, &[]);
    touched_under_strict(
        &spec,
        &|fixture| {
            put_lib(fixture, &format!("{LIB}\npub mod fresh;\n"));
            put(
                fixture.root(),
                "src/fresh.rs",
                "/// One.\npub fn new_one() -> u32 {\n    1\n}\n",
            );
            fixture.edit(
                CAP,
                "code: [fx_life::add]",
                "code: [fx_life::add, fx_life::fresh]",
            );
        },
        &[
            ("fx_life::fresh", "src/lib.rs", NEW),
            ("fx_life::fresh::new_one", "src/fresh.rs", NEW),
        ],
    );
    untouched_under_strict(&roomy(spec), &|fixture| {
        put_lib(fixture, &format!("{LIB}\nmod fresh;\n"));
        put(
            fixture.root(),
            "src/fresh.rs",
            "#[allow(dead_code)]\npub(crate) fn helper() -> u32 {\n    1\n}\n",
        );
    });
}

/// Only the public surface is compared: a private helper removed from a bound
/// module completes, and a public item removed beside it refuses.
#[test]
fn a_private_helper_removed_from_a_bound_module_completes_and_a_public_item_removed_refuses() {
    let shapes = "/// The area of a square.\npub fn square(side: u32) -> u32 {\n    side * side\n}\n\n#[allow(dead_code)]\nfn helper() {}\n";
    let spec = with(
        "[fx_life::add, fx_life::shapes]",
        &format!("{LIB}\npub mod shapes;\n"),
        &[("src/shapes.rs", shapes)],
    );
    untouched_under_strict(&spec, &|fixture| {
        put(
            fixture.root(),
            "src/shapes.rs",
            &shapes.replace("\n#[allow(dead_code)]\nfn helper() {}\n", ""),
        );
    });
    touched_under_strict(
        &spec,
        &|fixture| {
            put(
                fixture.root(),
                "src/shapes.rs",
                "#[allow(dead_code)]\nfn helper() {}\n",
            );
        },
        &[("fx_life::shapes::square", "src/shapes.rs", REMOVED)],
    );
}

/// The members of a trait impl of a bound public type are what the type
/// does, so an edit or the removal of one refuses; those of a private type
/// are not on the surface and complete.
#[test]
fn a_trait_impl_member_of_a_bound_type_edited_or_removed_refuses_and_one_of_a_private_type_completes()
 {
    let cloned = "\n/// A frame.\npub struct Frame;\n\nimpl Clone for Frame {\n    fn clone(&self) -> Self {\n        Frame\n    }\n}\n";
    let base = format!("{LIB}{cloned}");
    let spec = with("[fx_life::add, fx_life::Frame]", &base, &[]);
    let lib = base.replace("        Frame\n", "        Self\n");
    touched_under_strict(
        &spec,
        &|fixture| put_lib(fixture, &lib),
        &[("fx_life::Frame::clone", "src/lib.rs", TOKENS)],
    );
    let lib = format!("{LIB}\n/// A frame.\npub struct Frame;\n");
    touched_under_strict(
        &spec,
        &|fixture| put_lib(fixture, &lib),
        &[("fx_life::Frame::clone", "src/lib.rs", REMOVED)],
    );

    let quiet = "\n#[allow(dead_code)]\nstruct Quiet;\n\nimpl Clone for Quiet {\n    fn clone(&self) -> Self {\n        Quiet\n    }\n}\n";
    let base = format!("{LIB}{quiet}");
    let spec = roomy(with("[fx_life::add]", &base, &[]));
    let lib = base.replace("        Quiet\n", "        Self\n");
    untouched_under_strict(&spec, &|fixture| put_lib(fixture, &lib));
}

/// A re-export path of the surface that names another item than at the merge
/// base is touched, though neither item changed.
#[test]
fn a_bound_reexport_that_names_another_item_refuses() {
    let base = format!(
        "{LIB}\npub mod a {{\n    /// One.\n    pub struct T;\n}}\n\npub mod b {{\n    /// Two.\n    pub struct T(pub u32);\n}}\n\npub use a::T;\n"
    );
    let spec = with(
        "[fx_life::add, fx_life::T, fx_life::a, fx_life::b]",
        &base,
        &[],
    );
    let lib = base.replace("pub use a::T;", "pub use b::T;");
    touched_under_strict(
        &spec,
        &|fixture| put_lib(fixture, &lib),
        &[("fx_life::T", "src/lib.rs", RETARGETED)],
    );
}

/// A reflow that adds a token, such as the trailing comma of one parameter
/// per line, and a space added inside a doc comment change the tokens of the
/// item, so each refuses.
#[test]
fn a_reflow_that_adds_a_trailing_comma_or_a_space_in_a_doc_comment_refuses() {
    let base = format!("{LIB}\n/// Multiplies.\npub fn mul(a: u32, b: u32) -> u32 {{ a * b }}\n");
    let spec = with("[fx_life::add, fx_life::mul]", &base, &[]);
    for lib in [
        base.replace(
            "pub fn mul(a: u32, b: u32) -> u32 { a * b }",
            "pub fn mul(\n    a: u32,\n    b: u32,\n) -> u32 {\n    a * b\n}",
        ),
        base.replace("/// Multiplies.\n", "/// Multiplies. \n"),
    ] {
        touched_under_strict(
            &spec,
            &|fixture| put_lib(fixture, &lib),
            &[("fx_life::mul", "src/lib.rs", TOKENS)],
        );
    }
}

/// An item a macro generates is not an item of the source index, so an edit
/// of the macro that changes it is not judged: the stated limit.
#[test]
fn an_edit_of_a_macro_that_generates_an_item_of_a_bound_module_is_not_judged() {
    let base = format!(
        "{LIB}\nmacro_rules! make {{\n    ($name:ident) => {{\n        /// Three.\n        pub fn $name() -> u32 {{\n            1\n        }}\n    }};\n}}\n\npub mod made {{\n    make!(three);\n}}\n"
    );
    let spec = with("[fx_life::add, fx_life::made]", &base, &[]);
    let lib = base.replace("            1\n", "            2\n");
    untouched_under_strict(&spec, &|fixture| put_lib(fixture, &lib));
}

#[test]
fn a_bound_package_whose_surface_cannot_be_listed_is_an_error_and_without_the_second_glob_it_answers()
 {
    // Two globs bring `dup` in twice, so the surface cannot be listed.
    let dup = "\npub mod left {\n    /// One.\n    pub fn dup() {}\n}\n\npub mod right {\n    /// Two.\n    pub fn dup() {}\n}\n\npub use left::*;\npub use right::*;\n";
    let spec = with(
        "[fx_life::add, fx_life::left, fx_life::right]",
        &format!("{LIB}{dup}"),
        &[],
    );
    let comment = |fixture: &Fixture| {
        fixture.edit(
            "src/lib.rs",
            "pub fn plain() {}\n",
            "// A helper that no capability binds.\npub fn plain() {}\n",
        );
    };
    let fixture = after(&spec, &comment);
    let before = fixture.snapshot();
    let error = fixture
        .lifecycle()
        .reconcile("shape")
        .expect_err("the surface cannot be listed");
    assert!(
        matches!(
            &error,
            LifecycleError::Surface { package, target, side: Side::WorkingTree, .. }
                if package == "fx-life" && target == "fx_life"
        ),
        "{error:?}"
    );
    assert!(
        error.to_string().starts_with(
            "the public surface of `fx_life` in the package `fx-life` in the working tree cannot be listed, so the bound public items the change touches are unknown: "
        ),
        "{error}"
    );
    let error = fixture
        .lifecycle()
        .complete("shape")
        .expect_err("complete cannot judge either");
    assert!(matches!(error, LifecycleError::Surface { .. }), "{error:?}");
    assert_eq!(fixture.snapshot(), before, "an error writes nothing");

    // The neighbour: one glob, the same edit, answers and owes nothing.
    let spec = with(
        "[fx_life::add, fx_life::left, fx_life::right]",
        &format!("{LIB}{}", dup.replace("pub use right::*;\n", "")),
        &[],
    );
    untouched_under_strict(&spec, &comment);
}

/// A runner that runs through the real one and records each command.
struct Recording {
    inner: SystemRunner,
    commands: Mutex<Vec<String>>,
}

impl Runner for Recording {
    fn prepare_view(
        &self,
        request: &raf_workplan::exec::ViewRequest,
    ) -> Result<raf_workplan::exec::PreparedView, ExecError> {
        self.inner.prepare_view(request)
    }
    fn revalidate_view(&self, view: &raf_workplan::exec::PreparedView) -> Result<(), ExecError> {
        self.inner.revalidate_view(view)
    }
    fn finish_view(&self, view: &raf_workplan::exec::PreparedView) -> Result<(), ExecError> {
        self.inner.finish_view(view)
    }

    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError> {
        self.commands
            .lock()
            .expect("a lock")
            .push(invocation.to_string());
        self.inner.run(invocation, limits)
    }
}

/// A lifecycle of the real programs over a gate with the floor `floor`,
/// whose runner records each command.
fn recording(fixture: &Fixture) -> Lifecycle<Recording> {
    let programs = fixture.programs();
    let gate = Gate::workspace(fixture.root()).programs(programs.clone());
    let runner = Recording {
        inner: SystemRunner::new(programs),
        commands: Mutex::new(Vec::new()),
    };
    Lifecycle::open_with(&gate, runner).unwrap_or_else(|error| panic!("opens: {error}"))
}

/// The questions about the merge base that `lifecycle` asked of `git`: each
/// `merge-base`, each listing of its tree and the path of each file read at
/// it.
fn merge_base_reads(lifecycle: &Lifecycle<Recording>) -> Vec<String> {
    lifecycle
        .runner()
        .commands
        .lock()
        .expect("a lock")
        .iter()
        .filter_map(|command| {
            if command.starts_with("git merge-base ") {
                Some("merge-base".to_string())
            } else if command.starts_with("git ls-tree ") {
                Some("ls-tree".to_string())
            } else {
                command
                    .strip_prefix("git cat-file blob ")
                    .and_then(|named| named.split_once(':'))
                    .map(|(_, path)| path.to_string())
            }
        })
        .collect()
}

/// Under strict mode a `none` change lists the tree of the merge base once and
/// reads each file of the bound package once there, the manifest and the two
/// source files, and `complete` reads them once for both of its judgements;
/// a change with `contract_effect: changes` compares the affected document
/// without loading a merge-base source tree for the public-token rule.
#[test]
fn a_none_change_reads_each_file_of_the_bound_package_once_at_the_merge_base_and_otherwise_none() {
    let mut spec = roomy(strict_ready());
    spec.lib = format!("{LIB}\nmod extra;\n");
    spec.src = vec![(
        "src/extra.rs".to_string(),
        "#[allow(dead_code)]\npub(crate) fn helper() {}\n".to_string(),
    )];
    spec.src.extend([
        ("a".into(), "An unrelated committed file.\n".into()),
        ("a-".into(), "A valid lexical sibling.\n".into()),
        ("a0/data".into(), "A separate committed directory.\n".into()),
    ]);
    let fixture = after(&spec, &|fixture| {
        fixture.edit(
            "src/lib.rs",
            "pub fn plain() {}\n",
            "// A helper that no capability binds.\npub fn plain() {}\n",
        );
    });
    let files = [
        "merge-base",
        "ls-tree",
        "Cargo.toml",
        "src/lib.rs",
        "src/extra.rs",
    ];

    let strict = recording(&fixture);
    let owed = strict.reconcile("shape").expect("reconcile answers");
    assert!(owed.is_empty(), "{owed}");
    assert_eq!(merge_base_reads(&strict), files);

    let strict = recording(&fixture);
    let completed = strict.complete("shape").expect("complete answers");
    assert!(completed.is_completed(), "{completed}");
    let mut twice = files.to_vec();
    twice.push("merge-base");
    twice.push("merge-base");
    assert_eq!(
        merge_base_reads(&strict),
        twice,
        "captured and final judgements reread the base identity, not its files"
    );

    let fixture = after(&spec, &|fixture| {
        fixture.edit(
            "src/lib.rs",
            "pub fn plain() {}\n",
            "// A helper that no capability binds.\npub fn plain() {}\n",
        );
    });
    let default = recording(&fixture);
    default.reconcile("shape").expect("reconcile answers");
    assert_eq!(merge_base_reads(&default), files);

    let changes = after(
        &spec.clone().effect("changes", &["cap:fixture#contract"]),
        &|fixture| {
            fixture.edit(
                "src/lib.rs",
                "pub fn plain() {}\n",
                "// A helper that no capability binds.\npub fn plain() {}\n",
            );
        },
    );
    let strict = recording(&changes);
    strict.reconcile("shape").expect("reconcile answers");
    assert_eq!(
        merge_base_reads(&strict),
        ["merge-base", "docs/capabilities/fixture.md"]
    );
}

/// A few words, fewer than a section of strict mode holds.
const THIN: &str = "\nA few words.\n";

/// The debt record `work/debt/default.md` holding `rows` of `findings`.
fn debt_record(rows: &str) -> String {
    format!("---\ntype: Debt\nid: debt:default\nfindings:{rows}\nlines: []\n---\n")
}

/// The row that holds the thin Purpose of the fixture.
const PURPOSE_ROW: &str = "\n  - kind: strict-section-thin\n    subject: \"cap:fixture#purpose\"";

/// A strict fixture whose Purpose is thin, assessed.
fn thin_purpose() -> Fixture {
    let fixture = Fixture::real(&strict_ready());
    let text = fixture.text(CAP);
    let (prefix, rest) = text.split_once("## Purpose\n").expect("Purpose");
    let (_, rest) = rest.split_once("## Contract\n").expect("Contract");
    put(
        fixture.root(),
        CAP,
        &format!("{prefix}## Purpose\n{THIN}\n## Contract\n{rest}"),
    );
    fixture.assess();
    fixture
}

#[test]
fn the_check_once_the_record_is_removed_applies_the_debt_record_so_a_held_finding_does_not_refuse()
{
    let fixture = thin_purpose();
    let before = fixture.snapshot();
    // Without a debt record the thin section refuses the completion.
    let refused = fixture.complete();
    let Some(Failure::AfterRemoval(finding)) = refused.failures().first() else {
        panic!("the thin section must refuse: {refused}");
    };
    assert_eq!(finding.kind, "strict-section-thin", "{refused}");
    assert_eq!(fixture.snapshot(), before, "a refusal writes nothing");
    // The neighbour: the record holds it, and the change completes.
    put(
        fixture.root(),
        "work/debt/default.md",
        &debt_record(PURPOSE_ROW),
    );
    let completed = fixture.complete();
    assert!(completed.is_completed(), "{completed}");
    assert!(!fixture.exists(RECORD));
}

#[test]
fn the_check_once_the_record_is_removed_refuses_a_debt_row_that_is_resolved() {
    let fixture = thin_purpose();
    // The record holds the thin Purpose and a thin Examples that is not thin.
    let rows = format!(
        "{PURPOSE_ROW}\n  - kind: strict-section-thin\n    subject: \"cap:fixture#examples\""
    );
    put(fixture.root(), "work/debt/default.md", &debt_record(&rows));
    let before = fixture.snapshot();
    let refused = fixture.complete();
    let kinds_found: Vec<&str> = refused
        .failures()
        .iter()
        .filter_map(|failure| match failure {
            Failure::AfterRemoval(finding) => Some(finding.kind),
            _ => None,
        })
        .collect();
    assert_eq!(kinds_found, ["debt-resolved"], "{refused}");
    assert_eq!(fixture.snapshot(), before, "a refusal writes nothing");
    // The neighbour: the record that equals the present debt completes.
    put(
        fixture.root(),
        "work/debt/default.md",
        &debt_record(PURPOSE_ROW),
    );
    assert!(fixture.complete().is_completed());
}

#[test]
fn completion_retains_the_justified_host_nonexecution_receipt() {
    let guard = if cfg!(windows) { "unix" } else { "windows" };
    let spec = Spec {
        tests: "[fx_life::tests::passes, fx_life::opposite_host]",
        lib: format!(
            "{LIB}\n#[cfg({guard})]\n#[test]\nfn opposite_host() {{ panic!(\"must not run\"); }}\n"
        ),
        ..Spec::default()
    };
    let fixture = Fixture::real(&spec);
    let completion = fixture.complete();
    assert!(completion.is_completed(), "{completion}");
    assert!(!fixture.exists(RECORD));
    assert_eq!(completion.evidence().len(), 3);
    assert_eq!(
        completion.evidence().iter().filter(|e| e.passed()).count(),
        2
    );
    let skipped: Vec<_> = completion
        .evidence()
        .iter()
        .filter(|e| matches!(e.outcome(), BoundOutcome::NotApplicable { .. }))
        .collect();
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0].path(), "fx_life::opposite_host");
    assert!(!skipped[0].passed());
    assert_eq!(skipped[0].runs().len(), 1);
    assert!(
        skipped[0].runs()[0]
            .argv()
            .iter()
            .any(|arg| arg == "--print=cfg")
    );
    assert!(
        completion
            .to_string()
            .contains("not-applicable cap:fixture tests `fx_life::opposite_host`")
    );
}
