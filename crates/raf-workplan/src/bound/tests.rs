// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The parsers on literal output, the planning from index facts, and the whole
//! argv matrix, the hostile bindings and the proof rules through the
//! in-process runner, which starts nothing and runs on every platform.

use std::fmt::Write as _;
use std::time::Duration;

use raf_source::{MemoryTree, TargetKind, Workspace};

use crate::Workplan;
use crate::exec::{ExecError, FakeRunner, Limits, Output, Program};

use super::parse::{
    self, BadResult, Counts, DocEntry, DocStatus, EntryKind, ListEntry, doc_list, doc_status,
};
use super::plan::{ExampleFacts, TestFacts, doc_plan, example_plan, test_plan};
use super::{BoundOutcome, Evidence, Executor, Refusal};

// ------------------------------------------------------------- the parsers

fn entry(name: &str) -> ListEntry {
    ListEntry {
        name: name.to_string(),
        kind: EntryKind::Test,
    }
}

#[test]
fn a_list_with_one_entry_has_one_entry() {
    let text = "documents::x: test\n\n1 test, 0 benchmarks\n";
    assert_eq!(parse::test_list(text), vec![entry("documents::x")]);
}

#[test]
fn a_list_with_no_entry_has_none() {
    assert_eq!(parse::test_list("\n0 tests, 0 benchmarks\n"), vec![]);
    assert_eq!(parse::test_list(""), vec![]);
}

#[test]
fn a_list_with_two_entries_has_two() {
    let text = "a::x: test\nb::x: test\n\n2 tests, 0 benchmarks\n";
    assert_eq!(parse::test_list(text), vec![entry("a::x"), entry("b::x")]);
}

#[test]
fn a_benchmark_is_not_a_test_entry() {
    let text = "x: benchmark\ny: test\n";
    assert_eq!(
        parse::test_list(text),
        vec![
            ListEntry {
                name: "x".to_string(),
                kind: EntryKind::Benchmark
            },
            entry("y")
        ]
    );
}

#[test]
fn noise_lines_are_not_entries() {
    let text = "\
warning: unused variable: `x`
note: a test
   Compiling demo v0.1.0
     Running unittests src/lib.rs (target/debug/deps/demo-0123)
: test
a b: test
documents::x: test

1 test, 0 benchmarks
";
    assert_eq!(parse::test_list(text), vec![entry("documents::x")]);
}

#[test]
fn windows_line_endings_are_read() {
    assert_eq!(
        parse::test_list("documents::x: test\r\n\r\n1 test, 0 benchmarks\r\n"),
        vec![entry("documents::x")]
    );
}

#[test]
fn a_passing_result_is_read() {
    let text = "running 1 test\ntest x ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s\n";
    assert_eq!(
        parse::results(text),
        Ok(vec![Counts {
            passed: 1,
            failed: 0,
            ignored: 0
        }])
    );
}

#[test]
fn zero_passed_failed_and_ignored_results_are_read() {
    let zero = "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s";
    let ignored = "test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s";
    let failed = "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s";
    assert_eq!(parse::results(zero).unwrap()[0].passed, 0);
    assert_eq!(parse::results(ignored).unwrap()[0].ignored, 1);
    assert_eq!(parse::results(failed).unwrap()[0].failed, 1);
}

#[test]
fn an_older_result_line_without_the_time_is_read() {
    let text = "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out";
    assert_eq!(parse::results(text).unwrap()[0].passed, 1);
}

#[test]
fn two_result_lines_are_two_results() {
    let text = "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n\
                test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out\n";
    assert_eq!(parse::results(text).unwrap().len(), 2);
}

#[test]
fn no_result_line_is_no_result() {
    assert_eq!(parse::results("running 0 tests\n"), Ok(vec![]));
}

#[test]
fn a_result_line_without_its_counts_is_an_error() {
    for bad in [
        "test result: ok. 1 passed; 0 failed",
        "test result: ok. x passed; 0 failed; 0 ignored",
        "test result: ok. 1 passed; 0 failed; 0 ignored; 1 passed",
        "test result: maybe. 1 passed; 0 failed; 0 ignored",
        "test result: ok.1 passed; 0 failed; 0 ignored",
        "test result: ok. -1 passed; 0 failed; 0 ignored",
    ] {
        assert_eq!(
            parse::results(bad),
            Err(BadResult(bad.to_string())),
            "{bad}"
        );
    }
}

const DOC_LIST: &str = "\
src/lib.rs - (line 3): test
src/lib.rs - Wrapper<T>::get (line 13): test
src/lib.rs - inner::foo (line 22): test
src/lib.rs - inner::foo_more (line 28): test
src/lib.rs - inner::np (line 34): test

5 tests, 0 benchmarks
src/lib.rs - inner::cf (line 40): test

1 test, 0 benchmarks
";

fn doc(name: &str, item: Option<&str>) -> DocEntry {
    DocEntry {
        name: name.to_string(),
        item: item.map(str::to_string),
    }
}

#[test]
fn the_doc_list_gives_names_and_item_parts_across_blocks() {
    assert_eq!(
        doc_list(DOC_LIST),
        vec![
            doc("src/lib.rs - (line 3)", None),
            doc(
                "src/lib.rs - Wrapper<T>::get (line 13)",
                Some("Wrapper<T>::get")
            ),
            doc("src/lib.rs - inner::foo (line 22)", Some("inner::foo")),
            doc(
                "src/lib.rs - inner::foo_more (line 28)",
                Some("inner::foo_more")
            ),
            doc("src/lib.rs - inner::np (line 34)", Some("inner::np")),
            doc("src/lib.rs - inner::cf (line 40)", Some("inner::cf")),
        ]
    );
}

#[test]
fn a_doc_list_entry_has_the_form_of_the_pattern() {
    let text = "\
src/a/b-c.rs - x::y (line 4): test
../escape.rs - x::y (line 4): test
src//b.rs - x::y (line 4): test
src/lib.rs - x::y (line x): test
src/lib.rs - x::y (line 4) : test
src/lib.rs x::y (line 4): test
src/lib.rs -  (line 4): test
src/lib.rs - x::y (line 4): benchmark
";
    assert_eq!(
        doc_list(text),
        vec![doc("src/a/b-c.rs - x::y (line 4)", Some("x::y"))]
    );
}

#[test]
fn a_doc_list_entry_with_the_platform_separator_is_read_and_keeps_its_printed_name() {
    let text = "\
src\\lib.rs - x::y (line 4): test
src\\..\\escape.rs - x::y (line 4): test
";
    assert_eq!(
        doc_list(text),
        vec![doc("src\\lib.rs - x::y (line 4)", Some("x::y"))]
    );
}

#[test]
fn a_doc_status_is_read_from_the_whole_line() {
    let name = "src/lib.rs - inner::foo (line 22)";
    let run = |line: &str| doc_status(&format!("running 1 test\n{line}\n"), name);
    assert_eq!(run(&format!("test {name} ... ok")), DocStatus::Ok);
    assert_eq!(
        run(&format!("test {name} - compile ... ok")),
        DocStatus::CompileOnly
    );
    assert_eq!(
        run(&format!("test {name} - compile fail ... ok")),
        DocStatus::CompileFail
    );
    assert_eq!(run(&format!("test {name} ... FAILED")), DocStatus::Failed);
    assert_eq!(
        run(&format!("test {name} - compile ... FAILED")),
        DocStatus::Failed
    );
    assert_eq!(run(&format!("test {name} ... ignored")), DocStatus::Ignored);
    assert_eq!(
        run(&format!("test {name} ... ignored, not now")),
        DocStatus::Ignored
    );
    assert_eq!(doc_status("running 0 tests\n", name), DocStatus::Missing);
}

#[test]
fn a_doc_status_is_never_taken_from_another_name_or_a_loose_line() {
    let name = "src/lib.rs - inner::foo (line 2)";
    for line in [
        // A name that this one is a prefix of.
        "test src/lib.rs - inner::foo (line 22) ... ok",
        // A longer item with the same start.
        "test src/lib.rs - inner::foo_more (line 2) ... ok",
        // Text before, or something between the name and the dots.
        " test src/lib.rs - inner::foo (line 2) ... ok",
        "test src/lib.rs - inner::foo (line 2) extra ... ok",
        "test src/lib.rs - inner::foo (line 2) ... okay",
        "test src/lib.rs - inner::foo (line 2) ... ok (but)",
        "test src/lib.rs - inner::foo (line 2) ...ok",
        "test src/lib.rs - inner::foo (line 2)",
        "src/lib.rs - inner::foo (line 2) ... ok",
        // Not a run line at all.
        "failures:\n    src/lib.rs - inner::foo (line 2)",
    ] {
        assert_eq!(
            doc_status(&format!("{line}\n"), name),
            DocStatus::Missing,
            "{line:?}"
        );
    }
}

// ---------------------------------------------------- the facts, hostile

fn facts<'a>(target: &'a str, name: &'a str) -> TestFacts<'a> {
    TestFacts {
        package: "demo-pkg",
        kind: TargetKind::Test,
        target,
        name,
        harness: true,
        required_features: &[],
    }
}

#[test]
fn a_test_with_clean_facts_is_planned_and_its_neighbours_are_refused() {
    let plan = test_plan("p::x", &facts("parse-errors", "documents::x")).unwrap();
    assert_eq!(
        plan.run().to_string(),
        "cargo test --frozen --package=demo-pkg --test=parse-errors -- --exact -- documents::x"
    );
    for (what, hostile) in [
        ("target name", facts("-Zunstable-options", "x")),
        ("target name", facts("a b", "x")),
        ("target name", facts("a;b", "x")),
        ("target name", facts("--include-ignored", "x")),
        ("test name", facts("parse-errors", "--exact")),
        ("test name", facts("parse-errors", "-Zunstable-options")),
        ("test name", facts("parse-errors", "a;b")),
        ("test name", facts("parse-errors", "a b")),
        ("test name", facts("parse-errors", "--include-ignored")),
        ("test name", facts("parse-errors", "")),
    ] {
        let refusal = test_plan("p::x", &hostile).unwrap_err();
        assert!(
            matches!(&refusal, Refusal::Pattern { what: found, .. } if *found == what),
            "{refusal:?}"
        );
    }
    let mut package = facts("t", "x");
    package.package = "--frozen";
    assert!(matches!(
        test_plan("p::x", &package).unwrap_err(),
        Refusal::Pattern {
            what: "package name",
            ..
        }
    ));
}

#[test]
fn the_pinned_target_is_the_library_a_binary_or_an_integration_test() {
    let pinned = |kind, target| {
        let mut f = facts(target, "x");
        f.kind = kind;
        test_plan("p::x", &f).unwrap().run().to_string()
    };
    assert!(pinned(TargetKind::Lib, "demo_pkg").contains(" --lib -- "));
    assert!(pinned(TargetKind::Bin, "my-tool").contains(" --bin=my-tool -- "));
    assert!(pinned(TargetKind::Test, "parse-errors").contains(" --test=parse-errors -- "));
    for kind in [
        TargetKind::Bench,
        TargetKind::Example,
        TargetKind::BuildScript,
    ] {
        let mut f = facts("t", "x");
        f.kind = kind;
        let refusal = test_plan("p::x", &f).unwrap_err();
        assert!(matches!(refusal, Refusal::TargetKind { .. }), "{refusal:?}");
    }
}

#[test]
fn a_target_without_the_harness_or_with_required_features_is_refused() {
    let mut f = facts("t", "x");
    f.harness = false;
    let refusal = test_plan("p::x", &f).unwrap_err();
    assert!(matches!(refusal, Refusal::NoHarness { .. }));
    assert!(refusal.to_string().contains("harness = false"));
    assert!(refusal.to_string().contains("`t`"));

    let features = ["extra".to_string(), "more".to_string()];
    let mut f = facts("t", "x");
    f.required_features = &features;
    let refusal = test_plan("p::x", &f).unwrap_err();
    assert!(matches!(refusal, Refusal::RequiredFeatures { .. }));
    assert!(
        refusal
            .to_string()
            .contains("required-features = [extra, more]")
    );
    assert!(refusal.to_string().contains("`t`"));
    assert!(test_plan("p::x", &facts("t", "x")).is_ok());
}

#[test]
fn an_example_with_clean_facts_is_planned_and_hostile_ones_are_refused() {
    let clean = ExampleFacts {
        package: "demo-pkg",
        example: "show-it",
        required_features: &[],
    };
    assert_eq!(
        example_plan("p", &clean).unwrap().run().to_string(),
        "cargo run --frozen --package=demo-pkg --example=show-it"
    );
    for hostile in ["--include-ignored", "-Zunstable-options", "a b", "a;b", ""] {
        let facts = ExampleFacts {
            example: hostile,
            ..clean
        };
        assert!(matches!(
            example_plan("p", &facts).unwrap_err(),
            Refusal::Pattern {
                what: "example name",
                ..
            }
        ));
    }
    let features = ["extra".to_string()];
    let gated = ExampleFacts {
        required_features: &features,
        ..clean
    };
    assert!(matches!(
        example_plan("p", &gated).unwrap_err(),
        Refusal::RequiredFeatures { .. }
    ));
}

#[test]
fn a_doc_item_path_is_refused_unless_it_matches_its_pattern() {
    let plan = doc_plan("p", "demo-pkg", "Wrapper<T>::get").unwrap();
    assert_eq!(
        plan.run().to_string(),
        "cargo test --frozen --package=demo-pkg --doc -- Wrapper<T>::get"
    );
    for hostile in [
        "-Zunstable-options",
        "--exact",
        "--include-ignored",
        "-x",
        "a;b",
        "a b",
        "",
        "a<T",
        "a< T>",
    ] {
        assert!(
            matches!(
                doc_plan("p", "demo-pkg", hostile).unwrap_err(),
                Refusal::Pattern {
                    what: "item path",
                    ..
                }
            ),
            "{hostile:?}"
        );
    }
}

// ------------------------------------------------ a workspace and its runner

const CARGO_TOML: &str = r#"[package]
name = "demo-pkg"
version = "0.1.0"
edition = "2021"

[features]
extra = []

[[test]]
name = "no-harness"
path = "tests/no_harness.rs"
harness = false

[[test]]
name = "needs-feature"
path = "tests/needs_feature.rs"
required-features = ["extra"]

[[bench]]
name = "speed"
path = "benches/speed.rs"
"#;

const LIB: &str = "\
pub fn plain() {}
/// A wrapper.
pub struct Wrapper<T>(pub T);
impl<T> Wrapper<T> {
    /// Gets the value.
    ///
    /// ```
    /// assert!(true);
    /// ```
    pub fn get(&self) -> &T {
        &self.0
    }
}
pub mod inner {
    /// Goes.
    ///
    /// ```
    /// assert!(true);
    /// ```
    pub fn go() {}
}
pub mod bindings {
    #[cfg(test)]
    mod tests {
        #[test]
        fn x() {}
        #[test]
        #[ignore]
        fn skipped() {}
    }
}
pub mod documents {
    #[test]
    fn x() {}
}
";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Cargo.toml", CARGO_TOML),
        ("src/lib.rs", LIB),
        (
            "src/bin/my-tool.rs",
            "fn main() {}\n#[test]\nfn bin_case() {}\n",
        ),
        (
            "tests/parse-errors.rs",
            "#[test]\nfn case() {}\nmod inner {\n    #[test]\n    fn deep() {}\n}\npub fn helper() {}\n",
        ),
        (
            "tests/no_harness.rs",
            "fn main() {}\n#[test]\nfn raw() {}\n",
        ),
        ("tests/needs_feature.rs", "#[test]\nfn gated() {}\n"),
        ("benches/speed.rs", "#[test]\nfn in_bench() {}\n"),
        ("examples/show-it.rs", "fn main() {}\n"),
    ]
}

/// The seven sections of a capability, each with a line of text.
fn sections() -> String {
    let mut text = String::new();
    for name in [
        "Purpose",
        "Contract",
        "Usage",
        "Examples",
        "Constraints",
        "Explanation",
        "Verification",
    ] {
        write!(text, "## {name}\n\nText for {name}.\n\n").unwrap();
    }
    text
}

/// The capability document that lists `entry` in `list`.
fn capability(list: &str, entry: &str) -> String {
    let sections = sections();
    let entry = format!("\"{entry}\"");
    let (code, tests, examples) = match list {
        "code" => (entry.as_str(), "", ""),
        "tests" => ("", entry.as_str(), ""),
        _ => ("", "", entry.as_str()),
    };
    format!(
        "---\ntype: Capability\nid: cap:demo\ntitle: Demo\ndescription: Demo.\n\
         state: implementing\nbindings:\n  code: [{code}]\n  tests: [{tests}]\n  \
         examples: [{examples}]\ndepends: []\n---\n\n# Demo\n\n{sections}"
    )
}

fn workspace() -> Workspace {
    let tree: MemoryTree = files().into_iter().collect();
    Workspace::load(&tree, &["."]).expect("the demo workspace loads")
}

/// Run the binding `entry` of `list` over the demo workspace.
fn run(
    list: &str,
    entry: &str,
    runner: &FakeRunner,
    limits: Option<(Limits, Limits)>,
) -> Result<Evidence, ExecError> {
    let source = workspace();
    let doc = capability(list, entry);
    let docs = MemoryTree::from([("docs/capabilities/demo.md", doc.as_str())]);
    let plan = Workplan::load(&docs, &source);
    let binding = plan
        .graph()
        .bindings()
        .first()
        .expect("the capability lists a binding");
    let mut executor = Executor::new(&source, runner).expect("the index lists its tests");
    if let Some((build, example)) = limits {
        executor = executor.with_limits(build, example);
    }
    executor.run(binding)
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "a script answers with a Result, like the runner"
)]
fn output(code: i32, stdout: &str) -> Result<Output, ExecError> {
    Ok(Output::new(
        Some(code),
        stdout.as_bytes().to_vec(),
        Vec::new(),
    ))
}

fn ok(stdout: &str) -> Result<Output, ExecError> {
    output(0, stdout)
}

const PASS_ONE: &str = "running 1 test\ntest x ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s\n";

/// A runner for a test binding: the list pass and the run pass.
fn test_runner(target: &str, name: &str, list: &str, run: Result<Output, ExecError>) -> FakeRunner {
    FakeRunner::new()
        .on(
            Program::Cargo,
            &[
                "test",
                "--frozen",
                "--package=demo-pkg",
                target,
                "--",
                "--list",
                "--exact",
                "--",
                name,
            ],
            ok(list),
        )
        .on(
            Program::Cargo,
            &[
                "test",
                "--frozen",
                "--package=demo-pkg",
                target,
                "--",
                "--exact",
                "--",
                name,
            ],
            run,
        )
}

fn commands(runner: &FakeRunner) -> Vec<String> {
    runner
        .calls()
        .iter()
        .map(crate::exec::Call::command)
        .collect()
}

// -------------------------------------------------------- the argv matrix

#[test]
fn a_cfg_test_module_of_the_library_runs_as_lib_with_the_name_in_the_crate() {
    let runner = test_runner(
        "--lib",
        "bindings::tests::x",
        "bindings::tests::x: test\n\n1 test, 0 benchmarks\n",
        ok(PASS_ONE),
    );
    let evidence = run("tests", "demo_pkg::bindings::tests::x", &runner, None).unwrap();
    assert_eq!(evidence.outcome(), &BoundOutcome::Passed, "{evidence}");
    assert_eq!(
        commands(&runner),
        [
            "cargo test --frozen --package=demo-pkg --lib -- --list --exact -- bindings::tests::x",
            "cargo test --frozen --package=demo-pkg --lib -- --exact -- bindings::tests::x",
        ]
    );
    let proof = evidence.proof();
    assert_eq!(
        (proof.listed, proof.passed, proof.failed, proof.ignored),
        (Some(1), Some(1), Some(0), Some(0))
    );
}

#[test]
fn a_test_of_a_library_module_is_named_by_its_path_in_the_crate() {
    let runner = test_runner(
        "--lib",
        "documents::x",
        "documents::x: test\n",
        ok(PASS_ONE),
    );
    let evidence = run("tests", "demo_pkg::documents::x", &runner, None).unwrap();
    assert!(evidence.passed(), "{evidence}");
    assert!(commands(&runner)[0].ends_with("--lib -- --list --exact -- documents::x"));
}

#[test]
fn an_integration_test_with_a_hyphen_is_pinned_under_its_cargo_name() {
    let runner = test_runner("--test=parse-errors", "case", "case: test\n", ok(PASS_ONE));
    let evidence = run(
        "tests",
        "demo_pkg::test_crate::parse_errors::case",
        &runner,
        None,
    )
    .unwrap();
    assert!(evidence.passed(), "{evidence}");
    assert_eq!(
        commands(&runner),
        [
            "cargo test --frozen --package=demo-pkg --test=parse-errors -- --list --exact -- case",
            "cargo test --frozen --package=demo-pkg --test=parse-errors -- --exact -- case",
        ]
    );
}

#[test]
fn a_nested_test_of_an_integration_test_keeps_the_path_inside_the_crate() {
    let runner = test_runner(
        "--test=parse-errors",
        "inner::deep",
        "inner::deep: test\n",
        ok(PASS_ONE),
    );
    let evidence = run(
        "tests",
        "demo_pkg::test_crate::parse_errors::inner::deep",
        &runner,
        None,
    )
    .unwrap();
    assert!(evidence.passed(), "{evidence}");
}

#[test]
fn a_binary_test_is_pinned_with_bin() {
    let runner = test_runner(
        "--bin=my-tool",
        "bin_case",
        "bin_case: test\n",
        ok(PASS_ONE),
    );
    let evidence = run(
        "tests",
        "demo_pkg::bin_crate::my_tool::bin_case",
        &runner,
        None,
    )
    .unwrap();
    assert!(evidence.passed(), "{evidence}");
    assert!(commands(&runner)[1].contains(" --bin=my-tool -- --exact -- bin_case"));
}

#[test]
fn a_doc_test_binding_lists_the_package_and_runs_the_bare_item_path() {
    let list = "src/lib.rs - inner::go (line 16): test\n\n1 test, 0 benchmarks\n";
    let runner = FakeRunner::new()
        .on(
            Program::Cargo,
            &["test", "--frozen", "--package=demo-pkg", "--doc", "--", "--list"],
            ok(list),
        )
        .on(
            Program::Cargo,
            &["test", "--frozen", "--package=demo-pkg", "--doc", "--", "inner::go"],
            ok("running 1 test\ntest src/lib.rs - inner::go (line 16) ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out\n"),
        );
    let evidence = run("examples", "demo_pkg::inner::go", &runner, None).unwrap();
    assert_eq!(evidence.outcome(), &BoundOutcome::Passed, "{evidence}");
    assert_eq!(
        commands(&runner),
        [
            "cargo test --frozen --package=demo-pkg --doc -- --list",
            "cargo test --frozen --package=demo-pkg --doc -- inner::go",
        ]
    );
    assert_eq!(evidence.proof().selected, Some(1));
    assert_eq!(evidence.proof().ok_lines, Some(1));
}

#[test]
fn an_impl_item_is_bound_under_its_self_type_as_rustdoc_writes_it() {
    let list = "src/lib.rs - Wrapper<T>::get (line 6): test\n";
    let runner = FakeRunner::new()
        .on(
            Program::Cargo,
            &[
                "test",
                "--frozen",
                "--package=demo-pkg",
                "--doc",
                "--",
                "--list",
            ],
            ok(list),
        )
        .on(
            Program::Cargo,
            &[
                "test",
                "--frozen",
                "--package=demo-pkg",
                "--doc",
                "--",
                "Wrapper<T>::get",
            ],
            ok("test src/lib.rs - Wrapper<T>::get (line 6) ... ok\n"),
        );
    let evidence = run("examples", "demo_pkg::Wrapper::get", &runner, None).unwrap();
    assert_eq!(evidence.outcome(), &BoundOutcome::Passed, "{evidence}");
    assert_eq!(
        commands(&runner)[1],
        "cargo test --frozen --package=demo-pkg --doc -- Wrapper<T>::get"
    );
}

#[test]
fn an_example_runs_under_its_cargo_name_and_the_exit_status_decides() {
    let argv = ["run", "--frozen", "--package=demo-pkg", "--example=show-it"];
    let runner = FakeRunner::new().on(Program::Cargo, &argv, ok(""));
    let evidence = run(
        "examples",
        "demo_pkg::example_crate::show_it",
        &runner,
        None,
    )
    .unwrap();
    assert_eq!(evidence.outcome(), &BoundOutcome::Passed, "{evidence}");
    assert_eq!(
        commands(&runner),
        ["cargo run --frozen --package=demo-pkg --example=show-it"]
    );

    let runner = FakeRunner::new().on(
        Program::Cargo,
        &argv,
        Ok(Output::new(Some(3), Vec::new(), b"boom\n".to_vec())),
    );
    let evidence = run(
        "examples",
        "demo_pkg::example_crate::show_it",
        &runner,
        None,
    )
    .unwrap();
    let BoundOutcome::Failed { reason } = evidence.outcome() else {
        panic!("{evidence}");
    };
    assert!(
        reason.contains("exited with code 3") && reason.contains("boom"),
        "{reason}"
    );
}

#[test]
fn every_pass_carries_the_limits_of_its_kind() {
    let build = Limits::new(1 << 20, Duration::from_secs(7), true);
    let example = Limits::new(1 << 10, Duration::from_secs(3), false);
    let runner = test_runner(
        "--lib",
        "documents::x",
        "documents::x: test\n",
        ok(PASS_ONE),
    );
    run(
        "tests",
        "demo_pkg::documents::x",
        &runner,
        Some((build, example)),
    )
    .unwrap();
    assert!(runner.calls().iter().all(|call| *call.limits() == build));

    let runner = FakeRunner::new().on(
        Program::Cargo,
        &["run", "--frozen", "--package=demo-pkg", "--example=show-it"],
        ok(""),
    );
    run(
        "examples",
        "demo_pkg::example_crate::show_it",
        &runner,
        Some((build, example)),
    )
    .unwrap();
    assert_eq!(*runner.calls()[0].limits(), example);

    // The defaults are the limits of the completion gate.
    let runner = test_runner(
        "--lib",
        "documents::x",
        "documents::x: test\n",
        ok(PASS_ONE),
    );
    run("tests", "demo_pkg::documents::x", &runner, None).unwrap();
    assert!(
        runner
            .calls()
            .iter()
            .all(|call| *call.limits() == Limits::build())
    );
}

// -------------------------------------------- refusals before any process

/// Run a binding that must be refused from the index alone and return the
/// refusal; the runner is asked nothing.
fn refused(list: &str, entry: &str) -> Refusal {
    let runner = FakeRunner::new();
    let evidence = run(list, entry, &runner, None).unwrap();
    assert!(runner.calls().is_empty(), "{:?}", runner.calls());
    assert_eq!(evidence.runs().len(), 0);
    let BoundOutcome::Unresolved { reason } = evidence.outcome().clone() else {
        panic!("{evidence}");
    };
    reason
}

#[test]
fn a_function_that_is_not_a_test_is_refused_by_name() {
    let refusal = refused("tests", "demo_pkg::plain");
    assert!(matches!(refusal, Refusal::NotATest { .. }), "{refusal:?}");
    assert!(refusal.to_string().contains("`demo_pkg::plain`"));
    assert!(refusal.to_string().contains("#[test]"));
    let refusal = refused("tests", "demo_pkg::test_crate::parse_errors::helper");
    assert!(matches!(refusal, Refusal::NotATest { .. }), "{refusal:?}");
}

#[test]
fn a_test_of_a_target_without_the_harness_is_refused() {
    let refusal = refused("tests", "demo_pkg::test_crate::no_harness::raw");
    assert!(matches!(refusal, Refusal::NoHarness { .. }), "{refusal:?}");
    assert!(refusal.to_string().contains("`no-harness`"));
}

#[test]
fn a_test_of_a_target_with_required_features_is_refused() {
    let refusal = refused("tests", "demo_pkg::test_crate::needs_feature::gated");
    assert!(
        matches!(refusal, Refusal::RequiredFeatures { .. }),
        "{refusal:?}"
    );
    assert!(refusal.to_string().contains("`needs-feature`"));
    assert!(refusal.to_string().contains("extra"));
}

#[test]
fn a_test_in_a_benchmark_target_is_refused() {
    let refusal = refused("tests", "demo_pkg::bench_crate::speed::in_bench");
    assert!(
        matches!(
            refusal,
            Refusal::TargetKind {
                kind: TargetKind::Bench,
                ..
            }
        ),
        "{refusal:?}"
    );
    assert!(refusal.to_string().contains("`speed`"));
}

#[test]
fn a_code_binding_and_an_item_that_is_no_example_are_refused() {
    assert!(matches!(
        refused("code", "demo_pkg::plain"),
        Refusal::NotRunnable { .. }
    ));
    assert!(matches!(
        refused("examples", "demo_pkg::plain"),
        Refusal::NotAnExample { .. }
    ));
}

#[test]
fn the_root_of_the_crate_has_no_item_part() {
    // The library root is a module with documentation only when the crate
    // documents itself; the demo library does not, so this binding is not an
    // example at all.
    assert!(matches!(
        refused("examples", "demo_pkg"),
        Refusal::NotAnExample { .. }
    ));
}

#[test]
fn hostile_bindings_never_reach_the_runner() {
    for hostile in [
        "-Zunstable-options",
        "--include-ignored",
        "--exact",
        "demo_pkg::a;b",
        "demo_pkg::a b",
        "demo_pkg::documents::x --exact",
        "demo_pkg::documents::x;touch",
        "$(touch x)",
        "",
    ] {
        for list in ["tests", "examples"] {
            let source = workspace();
            let doc = capability(list, hostile);
            let docs = MemoryTree::from([("docs/capabilities/demo.md", doc.as_str())]);
            let plan = Workplan::load(&docs, &source);
            let runner = FakeRunner::new();
            let executor = Executor::new(&source, &runner).unwrap();
            for binding in plan.graph().bindings() {
                let evidence = executor.run(binding).unwrap();
                assert!(
                    matches!(
                        evidence.outcome(),
                        BoundOutcome::Unresolved {
                            reason: Refusal::Unresolved { .. }
                        }
                    ),
                    "{hostile:?} in {list}: {evidence}"
                );
            }
            assert!(
                runner.calls().is_empty(),
                "{hostile:?} in {list} reached the runner: {:?}",
                runner.calls()
            );
        }
    }
}

/// A workspace whose manifest holds a name that no pattern allows. The index
/// reads the manifest as it is, so the pattern is the last defence between a
/// manifest and a command line.
fn hostile_workspace(package: &str, extra: &str, files: &[(&str, &str)]) -> Workspace {
    let manifest = format!("[package]\nname = \"{package}\"\nversion = \"0.1.0\"\n\n{extra}");
    let tree: MemoryTree = files
        .iter()
        .copied()
        .chain([("Cargo.toml", manifest.as_str())])
        .collect();
    Workspace::load(&tree, &["."]).expect("the index reads the manifest as it is")
}

#[test]
fn a_hostile_name_in_the_manifest_never_reaches_the_runner() {
    const CASE: &str = "#[test]\nfn case() {}\n";
    /// A package, the manifest text and files that hold the hostile name, the
    /// binding that reaches it and the value the pattern refuses.
    type Case<'a> = (&'a str, &'a str, &'a [(&'a str, &'a str)], &'a str, &'a str);
    let cases: [Case<'_>; 4] = [
        (
            "demo",
            "[[test]]\nname = \"-Zunstable-options\"\npath = \"tests/hostile.rs\"\n",
            &[("src/lib.rs", ""), ("tests/hostile.rs", CASE)],
            "demo::test_crate::_Zunstable_options::case",
            "target name",
        ),
        (
            "demo",
            "[[test]]\nname = \"--exact\"\npath = \"tests/hostile.rs\"\n",
            &[("src/lib.rs", ""), ("tests/hostile.rs", CASE)],
            "demo::test_crate::__exact::case",
            "target name",
        ),
        (
            "demo",
            "[[bin]]\nname = \"--include-ignored\"\npath = \"src/hostile.rs\"\n",
            &[
                ("src/lib.rs", ""),
                ("src/hostile.rs", "fn main() {}\n#[test]\nfn case() {}\n"),
            ],
            "demo::bin_crate::__include_ignored::case",
            "target name",
        ),
        (
            "--frozen",
            "",
            &[("src/lib.rs", CASE)],
            "__frozen::case",
            "package name",
        ),
    ];
    for (package, extra, files, path, what) in cases {
        let source = hostile_workspace(package, extra, files);
        let doc = capability("tests", path);
        let docs = MemoryTree::from([("docs/capabilities/demo.md", doc.as_str())]);
        let plan = Workplan::load(&docs, &source);
        let runner = FakeRunner::new();
        let executor = Executor::new(&source, &runner).unwrap();
        let binding = &plan.graph().bindings()[0];
        assert!(binding.resolution().is_ok(), "{path} must resolve");
        let evidence = executor.run(binding).unwrap();
        assert!(
            matches!(
                unresolved(&evidence),
                Refusal::Pattern { what: found, .. } if *found == what
            ),
            "{path}: {evidence}"
        );
        assert!(runner.calls().is_empty(), "{path} reached the runner");
    }
}

#[test]
fn a_hostile_example_name_and_package_never_reach_the_runner() {
    let source = hostile_workspace(
        "demo",
        "[[example]]\nname = \"-Zunstable-options\"\npath = \"examples/hostile.rs\"\n",
        &[
            ("src/lib.rs", ""),
            ("examples/hostile.rs", "fn main() {}\n"),
        ],
    );
    let doc = capability("examples", "demo::example_crate::_Zunstable_options");
    let docs = MemoryTree::from([("docs/capabilities/demo.md", doc.as_str())]);
    let plan = Workplan::load(&docs, &source);
    let runner = FakeRunner::new();
    let binding = &plan.graph().bindings()[0];
    assert!(binding.resolution().is_ok());
    let evidence = Executor::new(&source, &runner)
        .unwrap()
        .run(binding)
        .unwrap();
    assert!(matches!(
        unresolved(&evidence),
        Refusal::Pattern {
            what: "example name",
            ..
        }
    ));
    assert_eq!(runner.calls().len(), 0);
}

// ----------------------------------------------------------- the proof rules

fn unresolved(evidence: &Evidence) -> &Refusal {
    match evidence.outcome() {
        BoundOutcome::Unresolved { reason } => reason,
        other => panic!("{other:?}: {evidence}"),
    }
}

const NAME: &str = "documents::x";
const PATH: &str = "demo_pkg::documents::x";

#[test]
fn zero_listed_entries_are_unresolved_and_nothing_is_run() {
    let runner = test_runner("--lib", NAME, "\n0 tests, 0 benchmarks\n", ok(PASS_ONE));
    let evidence = run("tests", PATH, &runner, None).unwrap();
    assert!(matches!(
        unresolved(&evidence),
        Refusal::NoTestListed { .. }
    ));
    assert_eq!(runner.calls().len(), 1, "the run pass must not start");
    assert_eq!(evidence.proof().listed, Some(0));
}

#[test]
fn two_listed_entries_are_unresolved() {
    let runner = test_runner(
        "--lib",
        NAME,
        "documents::x: test\nother::x: test\n",
        ok(PASS_ONE),
    );
    let evidence = run("tests", PATH, &runner, None).unwrap();
    assert!(matches!(
        unresolved(&evidence),
        Refusal::TestListedWrong { listed, .. } if listed.len() == 2
    ));
    assert_eq!(runner.calls().len(), 1);
}

#[test]
fn one_entry_under_another_name_is_unresolved() {
    let runner = test_runner("--lib", NAME, "other::x: test\n", ok(PASS_ONE));
    let evidence = run("tests", PATH, &runner, None).unwrap();
    assert!(matches!(
        unresolved(&evidence),
        Refusal::TestListedWrong { .. }
    ));
}

#[test]
fn a_benchmark_of_the_name_is_not_a_listed_test() {
    let runner = test_runner("--lib", NAME, "documents::x: benchmark\n", ok(PASS_ONE));
    let evidence = run("tests", PATH, &runner, None).unwrap();
    assert!(matches!(
        unresolved(&evidence),
        Refusal::NoTestListed { .. }
    ));
}

#[test]
fn noise_around_the_one_entry_does_not_matter() {
    let list = "warning: unused\n   Compiling demo\n\ndocuments::x: test\n\n1 test, 0 benchmarks\n";
    let runner = test_runner("--lib", NAME, list, ok(PASS_ONE));
    let evidence = run("tests", PATH, &runner, None).unwrap();
    assert!(evidence.passed(), "{evidence}");
}

#[test]
fn zero_passed_is_unresolved_even_with_exit_status_zero() {
    let none = "running 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s\n";
    let runner = test_runner("--lib", NAME, "documents::x: test\n", ok(none));
    let evidence = run("tests", PATH, &runner, None).unwrap();
    assert!(
        matches!(
            unresolved(&evidence),
            Refusal::TestCounts {
                passed: 0,
                failed: 0,
                ignored: 0,
                ..
            }
        ),
        "{evidence}"
    );
    assert_eq!(evidence.proof().passed, Some(0));
}

#[test]
fn an_ignored_test_is_refused_though_the_list_shows_it() {
    let ignored = "running 1 test\ntest documents::x ... ignored\n\ntest result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s\n";
    let runner = test_runner("--lib", NAME, "documents::x: test\n", ok(ignored));
    let evidence = run("tests", PATH, &runner, None).unwrap();
    assert!(
        matches!(unresolved(&evidence), Refusal::TestIgnored { .. }),
        "{evidence}"
    );
    assert_eq!(evidence.proof().ignored, Some(1));
}

#[test]
fn a_failing_test_fails_the_binding() {
    let failed = "running 1 test\ntest documents::x ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n";
    let runner = test_runner("--lib", NAME, "documents::x: test\n", output(101, failed));
    let evidence = run("tests", PATH, &runner, None).unwrap();
    let BoundOutcome::Failed { reason } = evidence.outcome() else {
        panic!("{evidence}");
    };
    assert!(
        reason.contains("101") && reason.contains("1 failed"),
        "{reason}"
    );
}

#[test]
fn a_status_other_than_zero_is_never_a_pass_whatever_the_counts_say() {
    let runner = test_runner("--lib", NAME, "documents::x: test\n", output(101, PASS_ONE));
    let evidence = run("tests", PATH, &runner, None).unwrap();
    assert!(
        matches!(evidence.outcome(), BoundOutcome::Failed { .. }),
        "{evidence}"
    );
}

#[test]
fn a_list_pass_that_does_not_build_fails_the_binding() {
    let runner = FakeRunner::new().on(
        Program::Cargo,
        &[
            "test",
            "--frozen",
            "--package=demo-pkg",
            "--lib",
            "--",
            "--list",
            "--exact",
            "--",
            NAME,
        ],
        Ok(Output::new(
            Some(101),
            Vec::new(),
            b"error[E0432]: unresolved import\n".to_vec(),
        )),
    );
    let evidence = run("tests", PATH, &runner, None).unwrap();
    let BoundOutcome::Failed { reason } = evidence.outcome() else {
        panic!("{evidence}");
    };
    assert!(reason.contains("E0432"), "{reason}");
    assert_eq!(runner.calls().len(), 1);
}

#[test]
fn no_result_line_and_two_result_lines_are_unresolved() {
    let runner = test_runner(
        "--lib",
        NAME,
        "documents::x: test\n",
        ok("running 1 test\n"),
    );
    let evidence = run("tests", PATH, &runner, None).unwrap();
    assert!(
        matches!(unresolved(&evidence), Refusal::Unparsable { .. }),
        "{evidence}"
    );
    let twice = format!("{PASS_ONE}{PASS_ONE}");
    let runner = test_runner("--lib", NAME, "documents::x: test\n", ok(&twice));
    let evidence = run("tests", PATH, &runner, None).unwrap();
    assert!(matches!(unresolved(&evidence), Refusal::Unparsable { .. }));
}

#[test]
fn a_run_that_passes_one_test_with_more_passed_is_refused() {
    let two = "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n";
    let runner = test_runner("--lib", NAME, "documents::x: test\n", ok(two));
    let evidence = run("tests", PATH, &runner, None).unwrap();
    assert!(matches!(
        unresolved(&evidence),
        Refusal::TestCounts { passed: 2, .. }
    ));
}

// ------------------------------------------------------- doc-test proof

const DOC_ARGV: [&str; 6] = [
    "test",
    "--frozen",
    "--package=demo-pkg",
    "--doc",
    "--",
    "--list",
];

fn doc_runner(list: &str, item: &str, run: Result<Output, ExecError>) -> FakeRunner {
    FakeRunner::new()
        .on(Program::Cargo, &DOC_ARGV, ok(list))
        .on(
            Program::Cargo,
            &[
                "test",
                "--frozen",
                "--package=demo-pkg",
                "--doc",
                "--",
                item,
            ],
            run,
        )
}

const GO_NAME: &str = "src/lib.rs - inner::go (line 16)";
const GO_LIST: &str =
    "src/lib.rs - inner::go (line 16): test\nsrc/lib.rs - inner::go_more (line 30): test\n";

fn run_doc(list: &str, runs: Result<Output, ExecError>) -> Evidence {
    let runner = doc_runner(list, "inner::go", runs);
    run("examples", "demo_pkg::inner::go", &runner, None).unwrap()
}

#[test]
fn a_doc_test_passes_only_on_its_own_ok_line() {
    let evidence = run_doc(GO_LIST, ok(&format!("test {GO_NAME} ... ok\n")));
    assert!(evidence.passed(), "{evidence}");
    assert_eq!(evidence.proof().listed, Some(2));
    assert_eq!(evidence.proof().selected, Some(1));
    assert_eq!(evidence.proof().ok_lines, Some(1));
}

#[test]
fn a_selected_entry_with_no_ok_line_is_refused() {
    let evidence = run_doc(GO_LIST, ok("running 0 tests\n"));
    assert!(
        matches!(unresolved(&evidence), Refusal::DocNotRun { names, .. } if names == &[GO_NAME]),
        "{evidence}"
    );
    assert_eq!(evidence.proof().ok_lines, Some(0));
}

#[test]
fn another_entrys_ok_line_does_not_stand_for_the_selected_one() {
    let evidence = run_doc(
        GO_LIST,
        ok(
            "test src/lib.rs - inner::go_more (line 30) ... ok\ntest src/lib.rs - inner::go (line 160) ... ok\n",
        ),
    );
    assert!(matches!(unresolved(&evidence), Refusal::DocNotRun { .. }));
}

#[test]
fn a_loose_ok_line_is_not_proof() {
    for line in [
        format!("test {GO_NAME} ... okay"),
        format!("test {GO_NAME} ...ok"),
        format!("test {GO_NAME} extra ... ok"),
        format!("{GO_NAME} ... ok"),
        format!(" test {GO_NAME} ... ok"),
        format!("test {GO_NAME} ... ok but"),
    ] {
        let evidence = run_doc(GO_LIST, ok(&format!("{line}\n")));
        assert!(!evidence.passed(), "{line:?}: {evidence}");
    }
}

#[test]
fn a_no_run_doc_test_is_refused_with_a_request_for_a_runnable_one() {
    let evidence = run_doc(GO_LIST, ok(&format!("test {GO_NAME} - compile ... ok\n")));
    let refusal = unresolved(&evidence);
    assert!(
        matches!(refusal, Refusal::DocNotRunnable { .. }),
        "{refusal:?}"
    );
    assert!(refusal.to_string().contains("bind a runnable doc test"));
    assert_eq!(evidence.proof().ok_lines, Some(0));
}

#[test]
fn a_compile_fail_doc_test_is_refused_with_a_request_for_a_runnable_one() {
    let evidence = run_doc(
        GO_LIST,
        ok(&format!("test {GO_NAME} - compile fail ... ok\n")),
    );
    let refusal = unresolved(&evidence);
    assert!(
        matches!(refusal, Refusal::DocNotRunnable { .. }),
        "{refusal:?}"
    );
    assert!(refusal.to_string().contains("bind a runnable doc test"));
}

#[test]
fn no_selected_entry_is_a_refusal_and_runs_nothing() {
    let runner = doc_runner(
        "src/lib.rs - other::thing (line 3): test\n",
        "inner::go",
        ok(""),
    );
    let evidence = run("examples", "demo_pkg::inner::go", &runner, None).unwrap();
    assert!(matches!(unresolved(&evidence), Refusal::NoDocTest { .. }));
    assert_eq!(runner.calls().len(), 1);
    assert_eq!(evidence.proof().selected, Some(0));
}

#[test]
fn a_failing_sibling_the_filter_also_matches_fails_the_binding() {
    let evidence = run_doc(
        GO_LIST,
        output(
            101,
            &format!(
                "test src/lib.rs - inner::go_more (line 30) ... FAILED\ntest {GO_NAME} ... ok\n"
            ),
        ),
    );
    assert!(
        matches!(evidence.outcome(), BoundOutcome::Failed { .. }),
        "{evidence}"
    );
    assert_eq!(evidence.proof().ok_lines, Some(1));
}

#[test]
fn a_failing_selected_doc_test_fails_the_binding_by_name() {
    let evidence = run_doc(
        GO_LIST,
        output(101, &format!("test {GO_NAME} ... FAILED\n")),
    );
    let BoundOutcome::Failed { reason } = evidence.outcome() else {
        panic!("{evidence}");
    };
    assert!(reason.contains(GO_NAME), "{reason}");
}

#[test]
fn an_ignored_selected_doc_test_is_refused() {
    let evidence = run_doc(GO_LIST, ok(&format!("test {GO_NAME} ... ignored\n")));
    assert!(matches!(unresolved(&evidence), Refusal::DocNotRun { .. }));
}

#[test]
fn every_selected_entry_must_be_ok() {
    let list = "src/lib.rs - inner::go (line 16): test\nsrc/lib.rs - inner::go (line 40): test\n";
    let both = "test src/lib.rs - inner::go (line 16) ... ok\ntest src/lib.rs - inner::go (line 40) ... ok\n";
    let one = "test src/lib.rs - inner::go (line 16) ... ok\n";
    let evidence = run_doc(list, ok(both));
    assert!(evidence.passed(), "{evidence}");
    assert_eq!(evidence.proof().selected, Some(2));
    assert_eq!(evidence.proof().ok_lines, Some(2));
    let evidence = run_doc(list, ok(one));
    assert!(!evidence.passed(), "{evidence}");
}

// ------------------------------------------------------ limits and failures

#[test]
fn a_killed_test_is_a_failed_binding_that_reports_the_time() {
    let timeout = ExecError::Timeout {
        command: "cargo test".to_string(),
        timeout: Duration::from_secs(5),
        elapsed: Duration::from_millis(5100),
    };
    let runner = test_runner("--lib", NAME, "documents::x: test\n", Err(timeout));
    let evidence = run("tests", PATH, &runner, None).unwrap();
    let BoundOutcome::Failed { reason } = evidence.outcome() else {
        panic!("{evidence}");
    };
    assert!(
        reason.contains("5.1 s") && reason.contains("killed"),
        "{reason}"
    );
    assert_eq!(
        evidence.runs().last().map(super::Run::ended),
        Some(super::Ended::TimedOut)
    );
    assert!(evidence.wall() >= Duration::from_millis(5100));
}

#[test]
fn a_run_that_cannot_start_is_an_error_and_not_a_verdict() {
    let spawn = ExecError::Spawn {
        command: "cargo".to_string(),
        program: "cargo".to_string(),
        kind: std::io::ErrorKind::NotFound,
        message: "not found".to_string(),
    };
    let runner = test_runner("--lib", NAME, "documents::x: test\n", Err(spawn));
    assert!(matches!(
        run("tests", PATH, &runner, None),
        Err(ExecError::Spawn { .. })
    ));
}

#[test]
fn an_example_killed_at_its_limit_fails_with_the_elapsed_time() {
    let timeout = ExecError::Timeout {
        command: "cargo run".to_string(),
        timeout: Duration::from_secs(300),
        elapsed: Duration::from_secs(300),
    };
    let runner = FakeRunner::new().on(
        Program::Cargo,
        &["run", "--frozen", "--package=demo-pkg", "--example=show-it"],
        Err(timeout),
    );
    let evidence = run(
        "examples",
        "demo_pkg::example_crate::show_it",
        &runner,
        None,
    )
    .unwrap();
    let BoundOutcome::Failed { reason } = evidence.outcome() else {
        panic!("{evidence}");
    };
    assert!(reason.contains("300.0 s"), "{reason}");
}

#[test]
fn too_much_output_is_a_failed_binding() {
    let runner = FakeRunner::new().on(
        Program::Cargo,
        &["run", "--frozen", "--package=demo-pkg", "--example=show-it"],
        Ok(Output::new(Some(0), vec![b'x'; 64], Vec::new())),
    );
    let limits = (Limits::build(), Limits::example().with_max_output(16));
    let evidence = run(
        "examples",
        "demo_pkg::example_crate::show_it",
        &runner,
        Some(limits),
    )
    .unwrap();
    assert!(
        matches!(evidence.outcome(), BoundOutcome::Failed { .. }),
        "{evidence}"
    );
}

#[test]
fn evidence_reads_as_a_verdict_line_and_the_processes_that_ran() {
    let runner = test_runner(
        "--lib",
        NAME,
        "documents::x: test\n",
        Ok(
            Output::new(Some(0), PASS_ONE.as_bytes().to_vec(), Vec::new())
                .with_elapsed(Duration::from_millis(1500)),
        ),
    );
    let evidence = run("tests", PATH, &runner, None).unwrap();
    let text = evidence.to_string();
    assert!(
        text.starts_with(
            "pass cap:demo tests `demo_pkg::documents::x` (1 listed, 1 passed, 0 failed, 0 ignored; 1.5 s)"
        ),
        "{text}"
    );
    assert!(text.contains(
        "\n  ran cargo test --frozen --package=demo-pkg --lib -- --list --exact -- documents::x"
    ));
    assert!(text.ends_with("-- --exact -- documents::x"), "{text}");

    let refused = run("tests", "demo_pkg::plain", &FakeRunner::new(), None).unwrap();
    assert!(
        refused
            .to_string()
            .starts_with("unresolved cap:demo tests `demo_pkg::plain`")
    );
    assert_eq!(refused.runs().len(), 0);
}
