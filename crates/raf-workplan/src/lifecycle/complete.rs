// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `complete`: the transition from an open change to a delivered one.
//!
//! Static obligations and a prospective document overlay precede publication.
//! Reopening validates the static judgement again without executing inspected code.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::rc::Rc;

use raf_okf::{Change, DocType, Location, State};
use raf_source::{EntryKind, RelPath};

use crate::digest::semantic_hash;
use crate::files::{self, Target};
use crate::finding::kind;
use crate::generated;
use crate::lookup::bare_slug;
use crate::obligation::{ObligationKind, Reconciled};
use crate::publication_io::Root;
use crate::sync::restore_state_edits;
use crate::{
    Binding, BindingKind, Finding, Loaded, Opened, SyncError, SyncOptions, SyncReport, Workplan,
};

use super::completion::{Completion, Failure};
use super::session::{Lifecycle, LifecycleError, Side};
use super::{state, touched};

/// The static conditions judged over one opening of the repository.
struct Evaluation {
    /// Exact loaded documentation identities before any publication.
    documents: Vec<(String, purrdf_core::ContentDigest)>,
    /// Every failure, in the order of the conditions.
    failures: Vec<Failure>,
    /// Declared bindings of the listed capabilities, in deterministic order.
    bindings: Vec<Binding>,
    /// The path of the record.
    record: RelPath,
    record_digest: String,
    /// The admitted source bytes of the capability state edits.
    state_originals: BTreeMap<RelPath, String>,
    /// The generated blocks the removal stales, rewritten as the sync of the
    /// repository without the record writes them. Empty while a failure
    /// stands.
    rewrite: SyncReport,
}

/// Identity of a declared binding across static judgements.
type BindingKey = (String, BindingKind, String);

fn binding_keys(bindings: &[Binding]) -> Vec<BindingKey> {
    bindings
        .iter()
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

impl Lifecycle {
    /// Complete a change using static contracts and a prospective document overlay.
    /// No test, example, compiler or other program is started.
    ///
    /// # Errors
    /// Refuses unavailable history, unreadable inputs and invalid publication paths.
    /// Static contract failures return a refusal without publishing any file.
    pub fn complete(&self, change: &str) -> Result<Completion, LifecycleError> {
        let root = Root::from_tree(self.opened.shared_tree())
            .map_err(|source| LifecycleError::Rewrite { source })?;
        let _exclusive = root
            .lock_lifecycle()
            .map_err(|source| LifecycleError::Rewrite { source })?;
        let slug = bare_slug(change);
        let mut kept = None;
        let before = self.evaluate(&self.opened, slug, &mut kept)?;
        if !before.failures.is_empty() {
            return Ok(Completion::Refused {
                failures: before.failures,
            });
        }
        let opened = self.gate.open()?;
        let after = self.evaluate(&opened, slug, &mut None)?;
        if self.opened.workspace() != opened.workspace()
            || self.opened.tracked() != opened.tracked()
            || before.documents != after.documents
            || binding_keys(&before.bindings) != binding_keys(&after.bindings)
            || after.rewrite.edits != before.rewrite.edits
            || !after.failures.is_empty()
        {
            let mut failures = vec![Failure::Changed {
                what: "bindings, generated replacements or static conditions differ on reopening"
                    .into(),
            }];
            failures.extend(after.failures);
            return Ok(Completion::Refused { failures });
        }
        Self::write(&root, &after)
    }

    /// Judge static conditions and the prospective state without writing.
    fn evaluate(
        &self,
        opened: &Opened,
        slug: &str,
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
        Self::judge_record(opened, &record_path)?;

        // Condition 1, all kinds that need no run of `cargo`.
        let mut state_originals = BTreeMap::new();
        let state_edits = promote(&plan, record, &mut state_originals)?;
        let mut obligations = fixed.obligations().to_vec();
        obligations.extend(self.history_obligations(
            opened,
            &plan,
            &impact,
            record.value().contract_effect,
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
        let (found, rewrite, unsynced) = Self::after_removal(opened, &record_path, state_edits);
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
                failures.push(if finding.kind.starts_with("binding-") {
                    Failure::Binding(finding)
                } else {
                    Failure::AfterRemoval(finding)
                });
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

        let mut bindings: Vec<Binding> = impact.bindings.into_iter().collect();
        bindings.sort_by(|a, b| {
            (a.capability(), a.kind(), a.path()).cmp(&(b.capability(), b.kind(), b.path()))
        });
        Ok(Evaluation {
            documents: document_identities(&plan),
            failures,
            bindings,
            record: record_path,
            record_digest: semantic_hash(&record.text()),
            state_originals,
            rewrite,
        })
    }

    /// The findings of `check` over the repository as it will be once the
    /// writes are done, and the rewrite that gets it there.
    ///
    /// The record is hidden and completed capabilities hold their delivered
    /// state, so the generated blocks the removal stales are current in the
    /// model that is judged, and a link inside one to the removed record is not
    /// a fault. When the sync cannot run the blocks stay as they are and the
    /// findings are those of the repository without the record, minus the
    /// blocks it stales; the error is returned with them.
    fn after_removal(
        opened: &Opened,
        record: &RelPath,
        state_edits: Vec<(RelPath, String)>,
    ) -> (Vec<Finding>, SyncReport, Option<SyncError>) {
        let without = opened.workplan_after(record, &state_edits, &[]);
        match without.sync(&SyncOptions::default()) {
            Ok(mut report) => {
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
                let mut edits: BTreeMap<_, _> = state_edits.into_iter().collect();
                // Sync starts with the promoted text, so its whole-file
                // replacements already include the state changes.
                edits.extend(report.edits);
                report.edits = edits.into_iter().collect();
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

    /// Require that the record is a regular file directly in a `work/changes`
    /// that is a directory below a `work` that is a directory, neither a link,
    /// so the removal removes the record and no other file.
    fn judge_record(opened: &Opened, record: &RelPath) -> Result<(), LifecycleError> {
        let refused = |path: String, reason: &str| LifecycleError::RecordRefused {
            path,
            source: io::Error::new(io::ErrorKind::InvalidInput, reason.to_string()),
        };
        let target = files::target(opened.dir(), DocType::Change.directory(), record);
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
                source: error.source,
            }),
        }
    }

    /// Publish the admitted document edits, then remove the exact record.
    fn write(root: &Rc<Root>, completed: &Evaluation) -> Result<Completion, LifecycleError> {
        let record = &completed.record;
        let rewritten: Vec<String> = completed
            .rewrite
            .edits
            .iter()
            .map(|(path, _)| path.as_str().to_string())
            .collect();
        let refused = |source| LifecycleError::RecordRefused {
            path: record.to_string(),
            source,
        };
        let target = root.capture(record).map_err(refused)?;
        target
            .verify_content(Some(&completed.record_digest))
            .map_err(refused)?;
        let published =
            Workplan::write_guarded(&completed.rewrite, root, &completed.state_originals)
                .map_err(|source| LifecycleError::Rewrite { source })?;
        target.remove().map_err(|source| LifecycleError::Remove {
            path: record.as_str().to_string(),
            rewritten: rewritten.clone(),
            source: restore_state_edits(source, &published, &completed.state_originals),
        })?;
        Ok(Completion::Completed {
            removed: record.as_str().to_string(),
            rewritten,
        })
    }
}

fn promote(
    plan: &Workplan<'_>,
    completed: &Loaded<Change>,
    before: &mut BTreeMap<RelPath, String>,
) -> Result<Vec<(RelPath, String)>, LifecycleError> {
    let other_owners: BTreeSet<&str> = plan
        .changes()
        .iter()
        .filter(|record| record.path() != completed.path())
        .flat_map(|record| record.value().changes.iter().map(String::as_str))
        .collect();
    let mut edits = Vec::new();
    for cap in plan.capabilities() {
        let value = cap.value();
        if value.state != State::Implementing
            || !completed.value().changes.contains(&value.header.id)
            || other_owners.contains(value.header.id.as_str())
        {
            continue;
        }
        let (path, text) = state::edit(cap, State::Delivered)
            .map_err(|source| LifecycleError::Rewrite { source })?;
        before.insert(path.clone(), cap.text());
        edits.push((path, text));
    }
    edits.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    Ok(edits)
}

fn document_identities(plan: &Workplan<'_>) -> Vec<(String, purrdf_core::ContentDigest)> {
    fn add<T>(rows: &mut Vec<(String, purrdf_core::ContentDigest)>, documents: &[Loaded<T>]) {
        rows.extend(documents.iter().map(|document| {
            (
                document.path().to_owned(),
                purrdf_core::ContentDigest::of(document.text().as_bytes()),
            )
        }));
    }
    let mut rows = Vec::new();
    add(&mut rows, plan.capabilities());
    add(&mut rows, plan.changes());
    add(&mut rows, plan.architecture());
    add(&mut rows, plan.evidence());
    add(&mut rows, plan.assessments());
    add(&mut rows, plan.reviews());
    if let Some(document) = plan.coverage() {
        add(&mut rows, std::slice::from_ref(document));
    }
    if let Some(document) = plan.debt() {
        add(&mut rows, std::slice::from_ref(document));
    }
    rows.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    rows
}
