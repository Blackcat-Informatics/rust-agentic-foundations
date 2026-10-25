// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `verify`: run the bound tests and examples and report each binding.

use std::collections::BTreeSet;
use std::fmt;

use crate::bound::{BoundOutcome, Evidence, Executor};
use crate::exec::Runner;
use crate::lookup::capability_path;
use crate::obligation::ReconcileError;
use crate::{Binding, BindingKind, Opened, Workplan};

use super::session::LifecycleError;

/// The most characters of a nonexecution reason shown, not counting its gap.
const REASON: usize = 240;

/// What a `verify` run found: the [`Evidence`] of each binding it ran, in
/// order of capability, then tests before examples, then path.
///
/// `Display` is one line for each binding, `pass`, `fail` or `not-applicable` and then the
/// capability, the list, the bound path and, for a pass, the counts of its
/// proof, for a failure the reason on the same line (the build or the test
/// failed, it ran past its limit, or it proves nothing: zero tests, an
/// `#[ignore]`d test, a path that does not resolve or does not match its
/// pattern). An executed failure retains its stored reason after whitespace
/// flattening, including the executor's bounded stream excerpts. Reasons from
/// custom runners are retained under that same rule. Long unresolved or
/// not-applicable reasons keep their start and end. It prints nothing for a run
/// with no binding. The report is a
/// value: nothing was written anywhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyReport {
    change: Option<String>,
    evidence: Vec<Evidence>,
}

impl VerifyReport {
    /// The slug of the change that was verified, or `None` for a run over
    /// every capability.
    #[must_use]
    pub fn change(&self) -> Option<&str> {
        self.change.as_deref()
    }

    /// The evidence of each binding, in order.
    #[must_use]
    pub fn evidence(&self) -> &[Evidence] {
        &self.evidence
    }

    /// The number of bindings that ran or were refused.
    #[must_use]
    pub fn len(&self) -> usize {
        self.evidence.len()
    }

    /// Whether there was no binding to run.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.evidence.is_empty()
    }

    /// Whether each binding passed or is justified as non-applicable on this host.
    /// True for a run with no binding, since
    /// none failed; the exit code of a command that prints this report is 0
    /// exactly then.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.evidence.iter().all(|evidence| {
            matches!(
                evidence.outcome(),
                BoundOutcome::Passed | BoundOutcome::NotApplicable { .. }
            )
        })
    }

    /// The bindings whose execution proof passed; excludes non-applicable bindings.
    #[must_use]
    pub fn passed_count(&self) -> usize {
        self.evidence
            .iter()
            .filter(|evidence| evidence.passed())
            .count()
    }

    /// The bindings whose known compiler platform guard excludes this host.
    #[must_use]
    pub fn not_applicable_count(&self) -> usize {
        self.evidence
            .iter()
            .filter(|evidence| matches!(evidence.outcome(), BoundOutcome::NotApplicable { .. }))
            .count()
    }

    /// The line of each binding, in order, as `Display` prints them: one line
    /// each, with no line ending.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        self.evidence.iter().map(evidence_line).collect()
    }

    /// The bindings that failed or are unresolved; excludes host non-applicability.
    pub fn failures(&self) -> impl Iterator<Item = &Evidence> {
        self.evidence.iter().filter(|evidence| {
            !matches!(
                evidence.outcome(),
                BoundOutcome::Passed | BoundOutcome::NotApplicable { .. }
            )
        })
    }
}

impl fmt::Display for VerifyReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.lines() {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

/// The line `verify` prints for one binding: `pass` or `fail`, the capability,
/// the list, the bound path and then the counts of the proof or the reason.
pub(super) fn evidence_line(evidence: &Evidence) -> String {
    let list = evidence.kind().key();
    match evidence.outcome() {
        BoundOutcome::NotApplicable { reason } => format!(
            "not-applicable {} {list} `{}`: {}",
            evidence.capability(),
            evidence.path(),
            one_line(reason)
        ),
        BoundOutcome::Passed => format!(
            "pass {} {list} `{}` ({})",
            evidence.capability(),
            evidence.path(),
            evidence.proof()
        ),
        BoundOutcome::Failed { reason } => format!(
            "fail {} {list} `{}`: {}",
            evidence.capability(),
            evidence.path(),
            reason.split_whitespace().collect::<Vec<_>>().join(" ")
        ),
        BoundOutcome::Unresolved { reason } => format!(
            "fail {} {list} `{}`: unresolved binding: {}",
            evidence.capability(),
            evidence.path(),
            one_line(&reason.to_string())
        ),
    }
}

/// A nonexecution reason on one line, keeping the ends of a long explanation.
fn one_line(text: &str) -> String {
    let joined = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = joined.chars().collect();
    if chars.len() <= REASON {
        return joined;
    }
    let half = REASON / 2;
    let head: String = chars[..half].iter().collect();
    let tail: String = chars[chars.len() - half..].iter().collect();
    format!("{head} ... {tail}")
}

/// The bindings of the capabilities the change `slug` lists, every one of which
/// must be loaded.
fn bindings_of_change(plan: &Workplan<'_>, slug: &str) -> Result<Vec<Binding>, LifecycleError> {
    let impact = plan.impact(slug)?;
    let loaded: BTreeSet<&str> = plan
        .capabilities()
        .iter()
        .map(|cap| cap.value().header.id.as_str())
        .collect();
    for id in &impact.capabilities {
        if !loaded.contains(id.as_str()) {
            return Err(ReconcileError::CapabilityNotLoaded {
                change: impact.change.clone(),
                capability: id.clone(),
                findings: capability_path(id).map_or_default(|path| plan.findings_of(&path)),
            }
            .into());
        }
    }
    Ok(impact.bindings)
}

/// The bindings of every capability, none of whose documents may be unloaded.
fn bindings_of_every_capability(plan: &Workplan<'_>) -> Result<Vec<Binding>, LifecycleError> {
    let unloaded: Vec<String> = plan
        .unloaded()
        .into_iter()
        .filter(|id| id.starts_with("cap:"))
        .collect();
    if unloaded.is_empty() {
        return Ok(plan.graph().bindings().to_vec());
    }
    let findings = unloaded
        .iter()
        .filter_map(|id| capability_path(id))
        .flat_map(|path| plan.findings_of(&path))
        .collect();
    Err(LifecycleError::Unloaded {
        capabilities: unloaded,
        findings,
    })
}

/// Run the bound tests and the bound examples of the change `change`, or of
/// every capability when `change` is `None`, through `runner`, and report each
/// binding.
///
/// A test is judged by the proof that exactly one test ran and passed, an
/// example target by its exit status within the limit, a documentation test by
/// the `ok` line of each entry the list pass selected (see
/// [`Executor`]). A binding that does not resolve, is not a `#[test]`, or whose
/// value fails its pattern (`-Zunstable-options`, `--include-ignored`,
/// `--exact`, a path with `;` or a space) is reported as unresolved and
/// never reaches `cargo`. `code` bindings are not run.
///
/// This does not ask `git` anything and does not go through the Git version
/// gate of [`Lifecycle::open`](super::Lifecycle::open). It writes nothing in
/// the repository; `cargo` builds into its own target directory.
///
/// For a change, every capability it lists must be loaded. For every
/// capability, no capability document may be unloaded: what it binds is
/// unknown, and a run that skipped it would pass.
///
/// # Errors
///
/// [`LifecycleError::Change`] when the slug names no loaded record;
/// [`LifecycleError::Reconcile`] when a listed capability did not load;
/// [`LifecycleError::Unloaded`] for a run over every capability while one did
/// not load; [`LifecycleError::Exec`] when a process cannot be started or
/// read to its end, which is no verdict.
pub fn verify<R: Runner + ?Sized>(
    opened: &Opened,
    runner: &R,
    change: Option<&str>,
) -> Result<VerifyReport, LifecycleError> {
    let plan = opened.workplan();
    let mut bindings = match change {
        Some(slug) => bindings_of_change(&plan, slug)?,
        None => bindings_of_every_capability(&plan)?,
    };
    bindings.retain(|binding| matches!(binding.kind(), BindingKind::Test | BindingKind::Example));
    bindings.sort_by(|a, b| {
        (a.capability(), a.kind(), a.path()).cmp(&(b.capability(), b.kind(), b.path()))
    });
    let executor = Executor::new(opened.workspace(), runner)?;
    let evidence = bindings
        .iter()
        .map(|binding| executor.run(binding))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(VerifyReport {
        change: change.map(|slug| crate::lookup::bare_slug(slug).to_string()),
        evidence,
    })
}
