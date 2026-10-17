// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The `example-failing` mapping from a run to an obligation, over scripted
//! answers: every way a run can end that real `cargo` fixtures cannot make
//! quickly (a kill at the time limit, a kill for output) and the arguments of
//! the one process an example is.

use std::fmt::Write as _;
use std::time::Duration;

use raf_source::{MemoryTree, Workspace};

use crate::Workplan;
use crate::bound::Executor;
use crate::exec::{ExecError, FakeRunner, Output, Program, Stream};
use crate::obligation::{Needed, ObligationKind, Part, RunProblem, Subject};

use super::{LifecycleError, examples};

const RUN: [&str; 4] = ["run", "--frozen", "--package=demo", "--example=show"];

fn workspace() -> Workspace {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        ),
        (
            "src/lib.rs",
            "/// Documented, no example.\npub fn bare() {}\n",
        ),
        ("examples/show.rs", "fn main() {}\n"),
    ]);
    Workspace::load(&tree, &["."]).expect("the workspace loads")
}

fn documents() -> MemoryTree {
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
        write!(sections, "## {name}\n\nText.\n\n").expect("a string takes text");
    }
    let capability = format!(
        "---\ntype: Capability\nid: cap:demo\ntitle: Demo\ndescription: Demo.\n\
         state: implementing\nbindings:\n  code: []\n  tests: []\n  \
         examples: [demo::example_crate::show, demo::bare]\ndepends: []\n---\n\n# Demo\n\n{sections}"
    );
    let change = "---\ntype: Change\nid: change:shape\ntitle: Shape\ndescription: Shapes it.\n\
                  changes: [cap:demo]\naffects: []\ncontract_effect: none\nitems: []\n---\n\n\
                  ## Summary\n\nShapes.\n";
    MemoryTree::from([
        ("docs/capabilities/demo.md", capability.as_str()),
        ("work/changes/shape.md", change),
    ])
}

/// The obligations for the example `demo::example_crate::show` run with this
/// answer, and the answers for the list pass of `bare`.
fn raised(
    answer: Result<Output, ExecError>,
) -> Result<Vec<crate::obligation::Obligation>, LifecycleError> {
    let source = workspace();
    let docs = documents();
    let plan = Workplan::load(&docs, &source);
    let impact = plan.impact("shape").expect("the change loads");
    let argv: Vec<&str> = RUN
        .iter()
        .copied()
        .filter(|word| !word.is_empty())
        .collect();
    let runner = FakeRunner::new().on(Program::Cargo, &argv, answer).on(
        Program::Cargo,
        &[
            "test",
            "--frozen",
            "--package=demo",
            "--doc",
            "--",
            "--list",
        ],
        Ok(Output::new(Some(0), Vec::new(), Vec::new())),
    );
    let executor = Executor::new(&source, &runner).expect("the index lists its tests");
    examples::raise(&executor, &impact)
}

fn exited(code: Option<i32>) -> Output {
    Output::new(code, Vec::new(), b"error: whatever\n".to_vec())
}

/// The problem of the one example, ignoring the documented item with no
/// documentation test, which is owed in every case.
fn problem(answer: Result<Output, ExecError>) -> Option<RunProblem> {
    let owed = raised(answer).expect("the runs have verdicts");
    owed.into_iter()
        .find(|o| matches!(&o.evidence, Needed::PassingRun { binding, .. } if binding == "demo::example_crate::show"))
        .map(|o| match o.evidence {
            Needed::PassingRun { problem, .. } => problem,
            other => panic!("{other:?}"),
        })
}

#[test]
fn every_way_a_run_can_fail_is_a_problem_and_a_pass_is_none() {
    assert_eq!(problem(Ok(exited(Some(0)))), None);
    assert_eq!(
        problem(Ok(exited(Some(101)))),
        Some(RunProblem::Fails { code: Some(101) })
    );
    assert_eq!(
        problem(Ok(exited(None))),
        Some(RunProblem::Fails { code: None })
    );
    assert_eq!(
        problem(Err(ExecError::Timeout {
            command: "cargo run".to_string(),
            timeout: Duration::from_secs(300),
            elapsed: Duration::from_secs(300),
        })),
        Some(RunProblem::TimedOut)
    );
    assert_eq!(
        problem(Err(ExecError::OutputTooLarge {
            command: "cargo run".to_string(),
            stream: Stream::Stdout,
            limit: 1,
            elapsed: Duration::from_millis(40),
        })),
        Some(RunProblem::Overflowed)
    );
}

#[test]
fn an_example_that_cannot_be_started_is_an_error_and_has_no_problem() {
    let error = raised(Err(ExecError::Spawn {
        command: "cargo run".to_string(),
        program: "cargo".to_string(),
        kind: std::io::ErrorKind::NotFound,
        message: "No such file or directory".to_string(),
    }))
    .expect_err("no verdict");
    assert!(matches!(
        error,
        LifecycleError::Exec(ExecError::Spawn { .. })
    ));
}

#[test]
fn the_obligation_names_the_capability_the_examples_section_and_the_binding() {
    let owed = raised(Ok(exited(Some(101)))).expect("verdicts");
    // The example, and the documented item with no documentation test.
    assert_eq!(owed.len(), 2);
    for obligation in &owed {
        assert_eq!(obligation.kind, ObligationKind::ExampleFailing);
        assert_eq!(
            obligation.subject,
            Subject::Capability("cap:demo".to_string())
        );
        assert_eq!(
            obligation.section,
            Part::Section(raf_okf::CapabilitySection::Examples)
        );
        assert_eq!(obligation.location.path, "docs/capabilities/demo.md");
        assert_eq!(obligation.location.line, None);
    }
    let lines: Vec<String> = owed.iter().map(ToString::to_string).collect();
    assert!(
        lines[0].contains("(`demo::bare` proves nothing: no documentation test of the package `demo` is listed for the item `bare`"),
        "{}",
        lines[0]
    );
    assert!(
        lines[1].ends_with("(`demo::example_crate::show` fails to build or run (exit code 101))"),
        "{}",
        lines[1]
    );
}
