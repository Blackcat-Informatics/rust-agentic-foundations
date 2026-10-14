// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The execution of the tests, documentation tests and examples that a
//! capability binds.
//!
//! An [`Executor`] takes a [`Binding`](crate::Binding) of the graph, decides
//! from the facts of the source index what it runs as (a [`Plan`]), runs it
//! through an [`exec::Runner`](crate::exec::Runner) and returns the
//! [`Evidence`]: the invocations that ran, the counts of the proof and the
//! verdict. Nothing is written and nothing is kept.
//!
//! # Planning
//!
//! Before any process starts, the binding is judged from the index. A path
//! that does not resolve, a function that is not a `#[test]`, a target with
//! `harness = false` or `required-features`, a test in a benchmark, example or
//! build-script target, and a value that fails its pattern are each a
//! [`Refusal`] that names the binding and the target, and nothing reaches
//! `cargo`. The libtest name of a test is its path within its crate
//! (`documents::x` for a test of an integration test target, `bindings::tests::x`
//! for a `cfg(test)` module of the library); the target is pinned with `--lib`,
//! `--bin=<bin>` or `--test=<test>` under the Cargo name of the target, which
//! keeps its hyphens where the index path spells them with underscores.
//!
//! # Proof
//!
//! * **Test.** The list pass shows exactly one `<name>: test` entry, and the run
//!   pass reports `1 passed; 0 failed; 0 ignored` in one `test result:` line.
//!   Zero entries, two entries, `0 passed` and an `#[ignore]`d test (listed as a
//!   test, reported as ignored) leave the binding [`BoundOutcome::Unresolved`].
//! * **Documentation test.** The list pass for the package selects the entries
//!   whose item part equals the bound item path within the crate (impl items as
//!   rustdoc writes them, `Wrapper<T>::get`); the run uses the bare item path as
//!   a substring filter without `--exact`; each selected entry must appear as the
//!   whole line `test <name> ... ok`. A `no_run` test (`- compile ... ok`) and a
//!   `compile_fail` test (`- compile fail ... ok`) were not built and run, and
//!   are refused with a request to bind a runnable doc test. A documentation
//!   test that the substring also matches, such as `foo_more` for `foo`, is not
//!   counted, but if it fails it fails the exit status and the binding.
//! * **Example.** `cargo run` with standard input null; the exit status within
//!   the time limit decides.
//!
//! A process that cannot be started is an [`ExecError`](crate::exec::ExecError)
//! and the run has no verdict; a process killed at its limit is a failed
//! binding that reports how long it ran.
//!
//! ```rust
//! use raf_source::{MemoryTree, Workspace};
//! use raf_workplan::Workplan;
//! use raf_workplan::bound::{BoundOutcome, Executor};
//! use raf_workplan::exec::{FakeRunner, Output, Program};
//!
//! let source = Workspace::load(
//!     &MemoryTree::from([
//!         ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
//!         ("src/lib.rs", "#[cfg(test)]\nmod tests {\n    #[test]\n    fn case() {}\n}\n"),
//!     ]),
//!     &["."],
//! )?;
//! let sections: String = [
//!     "Purpose", "Contract", "Usage", "Examples", "Constraints", "Explanation", "Verification",
//! ]
//! .iter()
//! .map(|name| format!("## {name}\n\nText.\n\n"))
//! .collect();
//! let document = format!(
//!     "---\ntype: Capability\nid: cap:demo\ntitle: Demo\ndescription: Demo.\n\
//!      state: implementing\nbindings:\n  code: []\n  tests: [demo::tests::case]\n  \
//!      examples: []\ndepends: []\n---\n\n# Demo\n\n{sections}"
//! );
//! let docs = MemoryTree::from([("docs/capabilities/demo.md", document.as_str())]);
//! let plan = Workplan::load(&docs, &source);
//! let binding = &plan.graph().bindings()[0];
//!
//! // Nothing is started here: the runner answers from a script.
//! let ok = |text: &str| Ok(Output::new(Some(0), text.as_bytes().to_vec(), Vec::new()));
//! let runner = FakeRunner::new()
//!     .on(
//!         Program::Cargo,
//!         &["test", "--frozen", "--package=demo", "--lib", "--", "--list", "--exact", "--", "tests::case"],
//!         ok("tests::case: test\n\n1 test, 0 benchmarks\n"),
//!     )
//!     .on(
//!         Program::Cargo,
//!         &["test", "--frozen", "--package=demo", "--lib", "--", "--exact", "--", "tests::case"],
//!         ok("running 1 test\ntest tests::case ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"),
//!     );
//! let evidence = Executor::new(&source, &runner)?.run(binding)?;
//! assert_eq!(evidence.outcome(), &BoundOutcome::Passed);
//! assert_eq!(evidence.runs().len(), 2);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod evidence;
mod parse;
mod plan;
mod run;

pub use evidence::{BoundOutcome, Ended, Evidence, Proof, Run};
pub use plan::{DocPlan, ExamplePlan, Plan, Refusal, TestPlan};
pub use run::Executor;

#[cfg(test)]
mod tests;
