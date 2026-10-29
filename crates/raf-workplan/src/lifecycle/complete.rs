// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `complete`: the transition from an open change to a delivered one.
//!
//! The order is the order of the Contract and is not interchangeable.
//!
//! 1. The static conditions are judged, all of them, and every failure is
//!    collected: the obligation list of `reconcile` (every kind except
//!    `example-failing`, which needs a run), the outcome of each item, the
//!    PROPOSED blocks of the change, the generated blocks, the assessment of
//!    each authored section in `affects`, and `check` of the repository as it
//!    will be once the record is gone.
//! 2. Only when none failed are the bound tests and examples run, so a
//!    repository with a documentation fault spends no process on `cargo`.
//! 3. The repository is opened again and the static conditions are judged
//!    again, because the documents and the source can change while `cargo`
//!    runs. Any failure, or a different set of bindings to run, refuses.
//! 4. The generated blocks the removal stales are rewritten, and then, as the
//!    very last step, the record is removed.

use std::fs;
use std::io;

use raf_okf::{DocType, Location};
use raf_source::{EntryKind, RelPath};

use crate::bound::{BoundOutcome, Evidence, Executor};
use crate::exec::Runner;
use crate::files::{self, Target};
use crate::finding::kind;
use crate::generated;
use crate::lookup::bare_slug;
use crate::obligation::{ObligationKind, Reconciled};
use crate::{Binding, BindingKind, Finding, Opened, SyncError, SyncOptions, SyncReport, Workplan};

use super::completion::{Completion, Failure};
use super::examples;
use super::session::{Lifecycle, LifecycleError, Side};
use super::touched;

/// The static conditions judged over one opening of the repository.
struct Evaluation {
    /// Every failure, in the order of the conditions.
    failures: Vec<Failure>,
    /// The test and example bindings of the listed capabilities, in the order
    /// they run.
    runs: Vec<Binding>,
    /// The path of the record.
    record: RelPath,
    /// The generated blocks the removal stales, rewritten as the sync of the
    /// repository without the record writes them. Empty while a failure
    /// stands.
    rewrite: SyncReport,
}

enum Verified {
    Refused(Completion),
    Ready {
        opened: Box<Opened>,
        after: Evaluation,
        evidence: Vec<Evidence>,
    },
}

/// What identifies a binding that runs.
type RunKey = (String, BindingKind, String);

fn run_keys(runs: &[Binding]) -> Vec<RunKey> {
    runs.iter()
        .map(|binding| {
            (
                binding.capability().to_string(),
                binding.kind(),
                binding.path().to_string(),
            )
        })
        .collect()
}

/// Condition 6: an `assessment-missing`, `assessment-stale` or
/// `assessment-rejected` obligation of `owed` is a section without a current
/// accepted assessment.
fn assessment_failures(owed: &Reconciled) -> impl Iterator<Item = Failure> + '_ {
    owed.obligations()
        .iter()
        .filter(|owed| {
            matches!(
                owed.kind,
                ObligationKind::AssessmentMissing
                    | ObligationKind::AssessmentStale
                    | ObligationKind::AssessmentRejected
            )
        })
        .cloned()
        .map(Failure::Assessment)
}

impl<R: Runner> Lifecycle<R> {
    /// Complete the change `change`: refuse with every reason it cannot be
    /// completed, or rewrite the generated blocks the removal stales and
    /// remove `work/changes/<slug>.md`.
    ///
    /// `change` is the slug, with or without the `change:` prefix. The six
    /// conditions of the completion gate are judged as the module documents:
    ///
    /// 1. the obligation list of [`Lifecycle::reconcile`](Lifecycle::reconcile)
    ///    is empty (every kind, `contract_effect: none` leaving out the
    ///    untouched contract alone; the list also holds
    ///    `contract-effect-understated`);
    /// 2. every item has an `outcome`, and a `moved` outcome names a change
    ///    record that exists; a `dropped` or `moved` item
    ///    also has a `reason` (`outcome-without-reason`);
    /// 3. no PROPOSED block of the change remains in any capability document,
    ///    whether the document loads or not;
    /// 4. `docs sync` would write nothing;
    /// 5. each test of `bindings.tests` and each example of
    ///    `bindings.examples` of a listed capability passes, by the proof its
    ///    kind requires ([`Executor`]). A capability with empty lists satisfies
    ///    this only while it is `designed` or `considered`; in any other state
    ///    the empty list is a `binding-missing` obligation of condition 1;
    /// 6. each authored section in `affects` has a current assessment with the
    ///    verdict `accepted` ([`assessment`](crate::assessment)): a section
    ///    with no assessment, one whose inputs moved, and one that is current
    ///    and `rejected` each refuse, whether the assessor was a human or an
    ///    agent. A section that holds only generated blocks would need none,
    ///    but the profile refuses one, so every section of a loaded capability
    ///    is authored.
    ///    This is judged with the other static conditions, before any bound
    ///    test or example runs.
    ///
    /// A failing example is both an `example-failing` obligation (condition 1)
    /// and a failure of condition 5, and a section without a current accepted
    /// assessment is one or more `assessment-*` obligations (condition 1), one
    /// `assessment-stale` for each input that moved, each also a failure of
    /// condition 6; the headline of the refusal counts the section once.
    ///
    /// The repository without the record must also pass `check`, apart from
    /// the generated blocks the removal stales (the `Changed by` rows of the
    /// relationship tables), which `complete` rewrites. This is where the
    /// state of a listed capability is judged: a capability that is not
    /// `delivered` and is listed by no other open record is `no-open-record`,
    /// so the author sets `state: delivered` before running `complete`.
    /// `complete` never edits prose, and it does not judge the repository
    /// before the removal by the state rules: a `delivered` capability beside
    /// its still-open record is exactly what it resolves.
    ///
    /// That check fails on every finding in the repository, not only on the
    /// ones about the listed capabilities: a broken link in an unrelated
    /// capability blocks completion, and so does a link in prose to the record
    /// itself, which is broken once the record is removed.
    ///
    /// An `outcome` that is not `implemented`, `dropped` or
    /// `moved:change:<slug>` is a value the profile refuses and makes the
    /// record not load, so `complete` returns the error of
    /// [`Workplan::change`] and not a condition-2 refusal, as
    /// [`Lifecycle::reconcile`](Lifecycle::reconcile) does. Condition 2
    /// refuses an item with no outcome and a `moved` outcome whose target is
    /// missing.
    ///
    /// ```rust,no_run
    /// use raf_workplan::exec::Programs;
    /// use raf_workplan::lifecycle::Lifecycle;
    ///
    /// let lifecycle = Lifecycle::open(Programs::new("."))?;
    /// let completion = lifecycle.complete("incremental-parsing")?;
    /// print!("{completion}");
    /// std::process::exit(i32::from(!completion.is_completed()));
    /// # Ok::<(), raf_workplan::lifecycle::LifecycleError>(())
    /// ```
    ///
    /// # Writes
    ///
    /// A refusal publishes no tool-owned files. A completion writes the generated
    /// blocks that the removal stales (through [`Workplan::write`], which stages
    /// every file and then renames) and then removes the record, as the very
    /// last step. If the rewrite fails the record is not removed. If the
    /// removal fails after the rewrite, the error says which files were
    /// rewritten. These guarantees concern this method's own publication.
    /// Native bound processes use the explicit completed filesystem view.
    /// Writable copied inputs are revalidated around each process, and
    /// process-local ancestor directories are read-only. Declared outputs and
    /// unrelated ambient locations remain writable; workspace code is trusted.
    /// Git can fetch missing objects into its metadata
    /// in a partial clone without changing working files or the index.
    ///
    /// A document changed between the second evaluation and the write is not
    /// detected: [`Workplan::write`] replaces a file without comparing it with
    /// the text the rewrite was computed from.
    ///
    /// # Errors
    ///
    /// [`LifecycleError::Change`] and [`LifecycleError::Reconcile`] when the
    /// record or a listed capability does not load; [`LifecycleError::Git`]
    /// and [`LifecycleError::Document`] when a section cannot be compared;
    /// [`LifecycleError::Exec`] when a process of a bound run cannot be
    /// started or read to its end; [`LifecycleError::RecordRefused`] when
    /// `work` or `work/changes` is a symbolic link or not a directory, or the
    /// record is not a regular file there, judged with each evaluation, the
    /// first before anything runs and the second after the bound runs and
    /// before the writes; [`LifecycleError::Gate`] when the
    /// repository cannot be opened again; [`LifecycleError::Rewrite`] and
    /// [`LifecycleError::Remove`] when a write fails. Each is the tool being
    /// unable to give an answer, which is not a refusal.
    pub fn complete(&self, change: &str) -> Result<Completion, LifecycleError> {
        let slug = bare_slug(change);

        // The source index at the merge base is loaded once for both
        // judgements: the files of one commit do not change, and the second
        // judgement asks for the merge base again and loads it again only when
        // it, or the packages it is read from, differ.
        let mut kept = None;
        let before = self.evaluate(&self.opened, slug, false, &mut kept)?;
        if !before.failures.is_empty() {
            return Ok(Completion::Refused {
                failures: before.failures,
                evidence: Vec::new(),
            });
        }

        let request = self.gate.capture_request(
            &self.opened,
            before.record.clone(),
            before.rewrite.edits.clone(),
            before.rewrite.diagram_assets.clone(),
        )?;
        let prepared = self.runner.prepare_view(&request)?;
        let verified = self.verify_prepared(slug, &before, &prepared, &mut kept);
        if let Err(source) = self.runner.finish_view(&prepared) {
            let (primary, refused) = match verified {
                Err(error) => (Some(Box::new(error)), None),
                Ok(Verified::Refused(refusal)) => (None, Some(Box::new(refusal))),
                Ok(Verified::Ready { .. }) => (None, None),
            };
            return Err(LifecycleError::ViewCleanup {
                primary,
                refused,
                source,
            });
        }
        match verified? {
            Verified::Refused(refusal) => Ok(refusal),
            Verified::Ready {
                opened,
                after,
                evidence,
            } => Self::write(&opened, &after.record, &after.rewrite, evidence),
        }
    }

    fn verify_prepared(
        &self,
        slug: &str,
        before: &Evaluation,
        prepared: &crate::exec::PreparedView,
        kept: &mut Option<touched::BaseIndex>,
    ) -> Result<Verified, LifecycleError> {
        let frozen = self.gate.open_captured(prepared.view.clone())?;
        let captured = self.evaluate(&frozen, slug, false, kept)?;
        if !captured.failures.is_empty()
            || run_keys(&captured.runs) != run_keys(&before.runs)
            || captured.rewrite.edits != before.rewrite.edits
        {
            let mut failures = vec![Failure::Changed {
                what: "captured completion model differs from the original static judgement"
                    .to_string(),
            }];
            failures.extend(captured.failures);
            return Ok(Verified::Refused(Completion::Refused {
                failures,
                evidence: Vec::new(),
            }));
        }
        let (failing, evidence) = self.run_bound(&frozen, &captured.runs, prepared)?;
        if !failing.is_empty() {
            return Ok(Verified::Refused(Completion::Refused {
                failures: failing,
                evidence,
            }));
        }
        let opened = self.gate.open_with_runner(&self.runner)?;
        let after = self.evaluate(&opened, slug, true, kept)?;
        let changed_source = match self.runner.revalidate_view(prepared) {
            Ok(()) => false,
            Err(crate::exec::ExecError::View(source))
                if source.kind() == crate::exec::ViewErrorKind::Changed =>
            {
                true
            }
            Err(error) => return Err(error.into()),
        };
        if changed_source
            || run_keys(&after.runs) != run_keys(&captured.runs)
            || after.rewrite.edits != captured.rewrite.edits
            || !after.failures.is_empty()
        {
            let mut failures = vec![Failure::Changed {
                what: if run_keys(&after.runs) == run_keys(&captured.runs) {
                    "a condition that held before the bound tests and examples ran fails now"
                        .to_string()
                } else {
                    "the bound tests and examples to run differ from the ones that ran".to_string()
                },
            }];
            failures.extend(after.failures);
            return Ok(Verified::Refused(Completion::Refused {
                failures,
                evidence,
            }));
        }
        Ok(Verified::Ready {
            opened: Box::new(opened),
            after,
            evidence,
        })
    }

    /// Judge the static conditions over `opened`; `ran` says whether the
    /// bound tests and examples have run, which a refused way to the record
    /// reports, and `kept` holds the source index at the merge base of an
    /// earlier judgement.
    fn evaluate(
        &self,
        opened: &Opened,
        slug: &str,
        ran: bool,
        kept: &mut Option<touched::BaseIndex>,
    ) -> Result<Evaluation, LifecycleError> {
        let plan = opened.workplan();
        let record = plan.change(slug)?;
        let fixed = plan.reconcile(slug)?;
        let impact = plan.impact(slug)?;
        let record_path =
            RelPath::new(record.path()).map_err(|error| LifecycleError::Document {
                path: record.path().to_string(),
                side: Side::WorkingTree,
                reason: error.to_string(),
            })?;
        // The record is removed as the last step, so the way to it is judged
        // as every write of the crate judges it, before anything runs and
        // again before the writes.
        Self::judge_record(opened, &record_path, ran)?;

        // Condition 1, all kinds that need no run of `cargo`.
        let mut obligations = fixed.obligations().to_vec();
        obligations.extend(self.process_obligations(
            opened,
            &plan,
            &impact,
            record.value().contract_effect,
            false,
            kept,
        )?);
        let owed = Reconciled::new(impact.change.clone(), obligations);
        let mut failures: Vec<Failure> = owed
            .obligations()
            .iter()
            .cloned()
            .map(Failure::Owed)
            .collect();

        // Condition 2: the items, from the obligations that state it: an
        // item without outcome, and under the complete policy one whose outcome needs
        // a reason it does not give.
        failures.extend(
            owed.of_kind(ObligationKind::ItemWithoutOutcome)
                .chain(owed.of_kind(ObligationKind::OutcomeWithoutReason))
                .cloned()
                .map(Failure::Item),
        );

        // Condition 3: every block of the change, in a document that loaded
        // or not.
        let mut proposals = plan.unresolved_proposals(&impact.change);
        proposals.sort_by(|a, b| {
            (&a.location.path, a.location.line).cmp(&(&b.location.path, b.location.line))
        });
        let proposal_places: Vec<(String, Option<usize>)> = proposals
            .iter()
            .map(|owed| (owed.location.path.clone(), owed.location.line))
            .collect();
        failures.extend(proposals.into_iter().map(Failure::Proposal));

        // Condition 4: what `docs sync` would write or refuse.
        let analysis = generated::analyze(&plan);
        let mut faults: Vec<Finding> = analysis.refusals;
        faults.extend(analysis.repairs);
        faults.extend(analysis.invalid_index);
        failures.extend(faults.iter().cloned().map(Failure::Generated));
        failures.extend(
            analysis
                .failures
                .into_iter()
                .map(|(file, reason)| Failure::Rewrite { file, reason }),
        );

        // Condition 6: the affected authored sections without a current
        // accepted assessment, from the obligations that state it.
        failures.extend(assessment_failures(&owed));

        // The repository as it will be once the writes are done, judged by
        // `check`.
        let (found, rewrite, unsynced) = Self::after_removal(opened, &record_path);
        for finding in found {
            let repeats = faults.contains(&finding)
                || (finding.kind == kind::PROPOSED_MISSING_CHANGE
                    && proposal_places.iter().any(|(path, line)| {
                        *path == finding.file
                            && matches!(
                                finding.location,
                                Location::Line(at) if *line == Some(at)
                            )
                    }));
            if !repeats {
                failures.push(Failure::AfterRemoval(finding));
            }
        }
        // A sync that fails with no finding to say why is an error and not a
        // pass.
        if failures.is_empty()
            && let Some(error) = unsynced
        {
            return Err(LifecycleError::Sync(error));
        }
        let rewrite = if failures.is_empty() {
            rewrite
        } else {
            SyncReport::default()
        };

        let mut runs: Vec<Binding> = impact
            .bindings
            .into_iter()
            .filter(|binding| matches!(binding.kind(), BindingKind::Test | BindingKind::Example))
            .collect();
        runs.sort_by(|a, b| {
            (a.capability(), a.kind(), a.path()).cmp(&(b.capability(), b.kind(), b.path()))
        });
        Ok(Evaluation {
            failures,
            runs,
            record: record_path,
            rewrite,
        })
    }

    /// The findings of `check` over the repository as it will be once the
    /// writes are done, and the rewrite that gets it there.
    ///
    /// The record is not shown and the files of the rewrite hold their new
    /// text, so the generated blocks the removal stales are current in the
    /// model that is judged, and a link inside one to the removed record is not
    /// a fault. When the sync cannot run the blocks stay as they are and the
    /// findings are those of the repository without the record, minus the
    /// blocks it stales; the error is returned with them.
    fn after_removal(
        opened: &Opened,
        record: &RelPath,
    ) -> (Vec<Finding>, SyncReport, Option<SyncError>) {
        let without = opened.workplan_after(record, &[], &[]);
        match without.sync(&SyncOptions::default()) {
            Ok(report) => {
                let mut removed: Vec<_> = report.removals().cloned().collect();
                // Native ordering admits the same exact removal set used by the view.
                if let Err(error) = purrdf_lex::walk::try_sort_unstable_by(&mut removed, |a, b| {
                    Ok::<_, io::Error>(a.cmp(b))
                }) {
                    return (
                        without.check(),
                        SyncReport::default(),
                        Some(SyncError::Rewrite {
                            file: record.to_string(),
                            reason: error.to_string(),
                        }),
                    );
                }
                let findings = opened
                    .workplan_after(record, &report.edits, &removed)
                    .check();
                (findings, report, None)
            }
            Err(error) => {
                let findings = without
                    .check()
                    .into_iter()
                    .filter(|finding| finding.kind != kind::GENERATED_STALE)
                    .collect();
                (findings, SyncReport::default(), Some(error))
            }
        }
    }

    /// Condition 5: run every binding and return every one that did not pass.
    /// A failing example is also an `example-failing` obligation.
    fn run_bound(
        &self,
        opened: &Opened,
        runs: &[Binding],
        prepared: &crate::exec::PreparedView,
    ) -> Result<(Vec<Failure>, Vec<Evidence>), LifecycleError> {
        let executor = Executor::new(opened.workspace(), &self.runner)?.with_view(prepared);
        let mut owed = Vec::new();
        let mut receipts = Vec::new();
        let mut tests = Vec::new();
        let mut examples = Vec::new();
        for binding in runs {
            let evidence: Evidence = match executor.run(binding) {
                Ok(evidence) => evidence,
                Err(crate::exec::ExecError::View(source))
                    if source.kind() == crate::exec::ViewErrorKind::Changed =>
                {
                    owed.push(Failure::Changed {
                        what: source.to_string(),
                    });
                    break;
                }
                Err(error) => return Err(error.into()),
            };
            let satisfied = matches!(
                evidence.outcome(),
                BoundOutcome::Passed | BoundOutcome::NotApplicable { .. }
            );
            receipts.push(evidence.clone());
            if satisfied {
                continue;
            }
            if binding.kind() == BindingKind::Example {
                owed.extend(examples::obligation(binding, &evidence).map(Failure::Owed));
                examples.push(Failure::Example(evidence));
            } else {
                tests.push(Failure::Test(evidence));
            }
        }
        owed.extend(tests);
        owed.extend(examples);
        Ok((owed, receipts))
    }

    /// Require that the record is a regular file directly in a `work/changes`
    /// that is a directory below a `work` that is a directory, neither a link,
    /// so the removal removes the record and no other file.
    fn judge_record(opened: &Opened, record: &RelPath, ran: bool) -> Result<(), LifecycleError> {
        let refused = |path: String, reason: &str| LifecycleError::RecordRefused {
            path,
            ran,
            source: io::Error::new(io::ErrorKind::InvalidInput, reason.to_string()),
        };
        let target = if opened.captured() {
            files::target_in(opened.tree(), DocType::Change.directory(), record)
        } else {
            files::target(opened.dir(), DocType::Change.directory(), record)
        };
        match target {
            Ok(Target {
                kind: Some(EntryKind::File),
                ..
            }) => Ok(()),
            Ok(_) => Err(refused(
                record.as_str().to_string(),
                "it is not a regular file",
            )),
            Err(error) => Err(LifecycleError::RecordRefused {
                path: error.path,
                ran,
                source: error.source,
            }),
        }
    }

    /// The writes, in their fixed order: the generated blocks, and then, as the
    /// last step, the record.
    fn write(
        opened: &Opened,
        record: &RelPath,
        rewrite: &SyncReport,
        evidence: Vec<Evidence>,
    ) -> Result<Completion, LifecycleError> {
        let rewritten: Vec<String> = rewrite
            .edits
            .iter()
            .map(|(path, _)| path.as_str().to_string())
            .collect();
        if !rewrite.is_empty() {
            Workplan::write(rewrite, opened.dir())
                .map_err(|source| LifecycleError::Rewrite { source })?;
        }
        fs::remove_file(opened.dir().join(record.as_str())).map_err(|source| {
            LifecycleError::Remove {
                path: record.as_str().to_string(),
                rewritten: rewritten.clone(),
                source,
            }
        })?;
        Ok(Completion::Completed {
            removed: record.as_str().to_string(),
            rewritten,
            evidence,
        })
    }
}
