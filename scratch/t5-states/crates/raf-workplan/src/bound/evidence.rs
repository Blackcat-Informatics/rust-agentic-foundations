// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! What a run of a binding leaves behind: the exact invocations, the proof the
//! kind of binding requires, the verdict and the time it took. It is a value;
//! nothing is written anywhere.

use std::fmt;
use std::time::Duration;

use crate::BindingKind;
use crate::exec::Invocation;

use super::plan::Refusal;

/// The verdict on a binding.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BoundOutcome {
    /// The proof its kind requires holds.
    Passed,
    /// The binding ran or tried to and it failed: the build or the run exited
    /// with a failing status, a test or example failed, or the process ran
    /// past its time limit or its output bound and was killed.
    Failed {
        /// What failed.
        reason: String,
    },
    /// The binding is not something that can be run and counted: the index
    /// facts or a value refuse it, or the run proves nothing (zero tests, an
    /// `#[ignore]`d test, a `no_run` documentation test).
    Unresolved {
        /// Why.
        reason: Refusal,
    },
}

impl BoundOutcome {
    /// The stable name of the verdict: `passed`, `failed` or `unresolved`.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed { .. } => "failed",
            Self::Unresolved { .. } => "unresolved",
        }
    }

    /// Why the binding did not pass, in full; `None` for a pass.
    #[must_use]
    pub fn reason(&self) -> Option<String> {
        match self {
            Self::Passed => None,
            Self::Failed { reason } => Some(reason.clone()),
            Self::Unresolved { reason } => Some(reason.to_string()),
        }
    }

    /// Whether the binding passed.
    #[must_use]
    pub fn is_passed(&self) -> bool {
        matches!(self, Self::Passed)
    }
}

/// How a process of a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Ended {
    /// It exited with this code, or `None` when a signal ended it.
    Exited(Option<i32>),
    /// It was killed at its wall-clock limit.
    TimedOut,
    /// It was killed for writing more than the output bound.
    Overflowed,
}

/// One process of a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    invocation: Invocation,
    ended: Ended,
    elapsed: Duration,
}

impl Run {
    pub(super) fn new(invocation: Invocation, ended: Ended, elapsed: Duration) -> Self {
        Self {
            invocation,
            ended,
            elapsed,
        }
    }

    /// What was run.
    #[must_use]
    pub fn invocation(&self) -> &Invocation {
        &self.invocation
    }

    /// The argv vector the process received, without the program.
    #[must_use]
    pub fn argv(&self) -> Vec<String> {
        self.invocation.argv()
    }

    /// How it ended.
    #[must_use]
    pub fn ended(&self) -> Ended {
        self.ended
    }

    /// How long it ran.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

/// The counts a proof is made of. A field is `None` when the binding's kind
/// has no such count or the run did not get that far.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Proof {
    /// A test binding: the `<name>: test` entries of the list pass. A
    /// doc-test binding: the entries of the list pass for the package.
    pub listed: Option<usize>,
    /// A doc-test binding: the entries whose item part is the bound item.
    pub selected: Option<usize>,
    /// A test binding: tests passed, from `test result:`.
    pub passed: Option<usize>,
    /// A test binding: tests failed.
    pub failed: Option<usize>,
    /// A test binding: tests ignored.
    pub ignored: Option<usize>,
    /// A doc-test binding: the selected entries with a `test <name> ... ok`
    /// line in the run output.
    pub ok_lines: Option<usize>,
}

impl fmt::Display for Proof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        for (what, count) in [
            ("listed", self.listed),
            ("selected", self.selected),
            ("passed", self.passed),
            ("failed", self.failed),
            ("ignored", self.ignored),
            ("ok lines", self.ok_lines),
        ] {
            if let Some(count) = count {
                parts.push(format!("{count} {what}"));
            }
        }
        if parts.is_empty() {
            f.write_str("no counts")
        } else {
            f.write_str(&parts.join(", "))
        }
    }
}

/// The evidence of one binding: what it was, what was run, the proof, the
/// verdict and the wall time. [`Evidence`] is returned by the executor and is
/// not persisted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    capability: String,
    kind: BindingKind,
    path: String,
    runs: Vec<Run>,
    proof: Proof,
    outcome: BoundOutcome,
    wall: Duration,
}

impl Evidence {
    pub(super) fn new(
        capability: &str,
        kind: BindingKind,
        path: &str,
        runs: Vec<Run>,
        proof: Proof,
        outcome: BoundOutcome,
    ) -> Self {
        let wall = runs.iter().map(Run::elapsed).sum();
        Self {
            capability: capability.to_string(),
            kind,
            path: path.to_string(),
            runs,
            proof,
            outcome,
            wall,
        }
    }

    /// The id of the capability that lists the binding.
    #[must_use]
    pub fn capability(&self) -> &str {
        &self.capability
    }

    /// The list the binding is in.
    #[must_use]
    pub fn kind(&self) -> BindingKind {
        self.kind
    }

    /// The bound path, as the document writes it.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Every process of the run, in order. Empty when the binding was refused
    /// before any process started.
    #[must_use]
    pub fn runs(&self) -> &[Run] {
        &self.runs
    }

    /// The counts of the proof.
    #[must_use]
    pub fn proof(&self) -> &Proof {
        &self.proof
    }

    /// The verdict.
    #[must_use]
    pub fn outcome(&self) -> &BoundOutcome {
        &self.outcome
    }

    /// Whether the binding passed.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.outcome.is_passed()
    }

    /// The wall time of every process of the run, added up.
    #[must_use]
    pub fn wall(&self) -> Duration {
        self.wall
    }
}

fn list_name(kind: BindingKind) -> &'static str {
    match kind {
        BindingKind::Code => "code",
        BindingKind::Test => "tests",
        BindingKind::Example => "examples",
    }
}

impl fmt::Display for Evidence {
    /// A line for the verdict, then a line for each process:
    ///
    /// ```text
    /// pass cap:demo tests `demo::case` (1 listed, 1 passed, 0 failed, 0 ignored; 1.2 s)
    ///   ran cargo test --frozen --package=demo --lib -- --list --exact -- case
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verdict = match &self.outcome {
            BoundOutcome::Passed => "pass",
            BoundOutcome::Failed { .. } => "fail",
            BoundOutcome::Unresolved { .. } => "unresolved",
        };
        write!(
            f,
            "{verdict} {} {} `{}` ({}; {:.1} s)",
            self.capability,
            list_name(self.kind),
            self.path,
            self.proof,
            self.wall.as_secs_f64()
        )?;
        match &self.outcome {
            BoundOutcome::Passed => {}
            BoundOutcome::Failed { reason } => write!(f, ": {reason}")?,
            BoundOutcome::Unresolved { reason } => write!(f, ": {reason}")?,
        }
        for run in &self.runs {
            write!(f, "\n  ran {}", run.invocation)?;
        }
        Ok(())
    }
}
