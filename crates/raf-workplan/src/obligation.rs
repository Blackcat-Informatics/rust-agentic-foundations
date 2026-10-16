// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Obligations: what the documentation still owes a change, with the evidence
//! that would settle each.
//!
//! [`Workplan::reconcile`] reads a change record and the loaded documents and
//! returns a [`Reconciled`] list. An [`Obligation`] is a statement of work
//! with an [`ObligationKind`], the [`Subject`] and [`Part`] it is about, a
//! [`Place`] in a file and the [`Needed`] evidence that resolves it. It never
//! says "update the docs": it names the section and the evidence.
//!
//! Every obligation is derived from a value the model already holds, so
//! `reconcile` and `check` cannot disagree:
//!
//! | Kind | Derived from |
//! | --- | --- |
//! | `unresolved-proposal` | [`Graph::proposals`](crate::Graph::proposals) for the change, the line from the marker of the block; and the blocks the load kept from capability documents that did not validate, which the graph does not hold |
//! | `item-without-outcome` | the items of the record; a `moved` outcome that names no change record is raised too |
//! | `binding-missing` | the gaps of [`Workplan::status`] for each listed `implementing` or `delivered` capability (a `delivered` one owes `code` and `tests` only), and the bindings that resolve to an item their list does not take |
//! | `generated-stale` | the stale and missing generated blocks of a sync, in the listed capabilities and the index |
//! | `tense-mismatch` | the fixed [`tense::PHRASES`] list in the prose of a listed `implementing` or `delivered` capability, and a PROPOSED block in a `delivered` one |
//! | `claim-unsupported` | the claim markers of each listed capability ([`Body::claims`](raf_okf::Body::claims)) against the loaded evidence records: one for each named record that does not load, fails, or holds a value over its limit, at the line of the marker |
//! | `evidence-stale` | the `inputs` of each evidence record whose `for` lists a listed capability or that one of its claims names, against [`Digest::symbol`](crate::digest::Digest::symbol) of the source now: one for each input whose digest differs or that does not resolve, at the record |
//! | `section-untouched`, `example-failing`, `evidence-changed` | the merge base and the bound examples, which need a process; `Workplan::reconcile` does not raise them (see [`ObligationKind::is_static`]) and [`Lifecycle::reconcile`](crate::lifecycle::Lifecycle::reconcile) does, from `git` and from the executor of [`bound`](crate::bound) |
//!
//! The list is sorted by subject (capabilities, then change records, then the
//! index, each by id), part, kind, line and then evidence text, so equal
//! documents give equal output and two blocks in one section print in
//! document order. The parts of a capability sort as the document, then the
//! sections in the order a capability holds them (purpose, contract, usage,
//! examples, constraints, explanation, verification), then the items of a
//! record; the kinds sort in the order of [`ObligationKind::ALL`]. `contract_effect: none` suppresses `section-untouched` for
//! the `contract`, `usage` and `examples` sections and nothing else
//! ([`suppressed`]).
//!
//! A listed capability whose document did not load is an error
//! ([`ReconcileError::CapabilityNotLoaded`]), because what it owes is unknown.
//! The one exception is a `delivered` capability whose only fault is a `code`
//! or `tests` list with no entry, which the profile refuses: the document is
//! otherwise sound, so the empty list is a `binding-missing` obligation and
//! the capability is judged by nothing else until it loads. A sync cannot
//! settle a stale block while any document is unloaded, but it is still owed,
//! so it is listed.
//!
//! Nothing here writes a file or starts a process.

mod sources;
pub mod tense;
pub(crate) mod trace;

#[cfg(test)]
mod tests;

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

use raf_okf::{CapabilitySection, ContractEffect};

use crate::bound::{BoundOutcome, Ended, Evidence, Refusal, Run};
use crate::graph::{BindingKind, position};
use crate::lookup::{ChangeLookupError, write_findings};
use crate::{Finding, Workplan};

/// What an obligation is about.
///
/// The kinds of the reconcile contract, in the order of its table. The set is
/// closed: a finding about the documents is one of these kinds or it is not an
/// obligation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ObligationKind {
    /// A PROPOSED block names the change.
    UnresolvedProposal,
    /// An item of the record has no valid `outcome`.
    ItemWithoutOutcome,
    /// A listed capability binds nothing where it should, or a binding does
    /// not resolve or does not fit its list.
    BindingMissing,
    /// A section in `affects` has the same authored text as at the merge
    /// base.
    SectionUntouched,
    /// A bound example target or documentation test of a listed capability
    /// fails to build or run.
    ExampleFailing,
    /// A generated block differs from what a sync writes.
    GeneratedStale,
    /// Delivered or implementing text uses proposal or forward-looking
    /// wording, or a delivered capability has a PROPOSED block.
    TenseMismatch,
    /// A claim of a listed capability names an evidence record that does not
    /// load, whose verdict is `fail`, or that holds a value over its limit.
    ClaimUnsupported,
    /// An input of an evidence record that supports a listed capability has
    /// another digest now than the one the record keeps.
    EvidenceStale,
    /// An evidence record a claim of a listed capability names passed at the
    /// merge base and has changed since: it fails, a value is over a limit, a
    /// limit is gone, or the record is gone.
    EvidenceChanged,
}

impl ObligationKind {
    /// Every kind, in the order of the table in the reconcile contract. The
    /// order is the order of [`Ord`] and of the sort of a [`Reconciled`] list.
    pub const ALL: [Self; 10] = [
        Self::UnresolvedProposal,
        Self::ItemWithoutOutcome,
        Self::BindingMissing,
        Self::SectionUntouched,
        Self::ExampleFailing,
        Self::GeneratedStale,
        Self::TenseMismatch,
        Self::ClaimUnsupported,
        Self::EvidenceStale,
        Self::EvidenceChanged,
    ];

    /// The stable kebab-case name printed on an obligation line.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::UnresolvedProposal => "unresolved-proposal",
            Self::ItemWithoutOutcome => "item-without-outcome",
            Self::BindingMissing => "binding-missing",
            Self::SectionUntouched => "section-untouched",
            Self::ExampleFailing => "example-failing",
            Self::GeneratedStale => "generated-stale",
            Self::TenseMismatch => "tense-mismatch",
            Self::ClaimUnsupported => "claim-unsupported",
            Self::EvidenceStale => "evidence-stale",
            Self::EvidenceChanged => "evidence-changed",
        }
    }

    /// Whether the kind comes from the loaded documents alone. The other
    /// kinds need a process: `git` for `section-untouched` and
    /// `evidence-changed`, and `cargo` for `example-failing`.
    #[must_use]
    pub const fn is_static(self) -> bool {
        !matches!(
            self,
            Self::SectionUntouched | Self::ExampleFailing | Self::EvidenceChanged
        )
    }

    /// The place of the kind in [`ObligationKind::ALL`]. The `match` has no
    /// wildcard, so a new kind does not compile until it has a place, and the
    /// length check below does not compile until `ALL` lists it.
    const fn rank(self) -> usize {
        match self {
            Self::UnresolvedProposal => 0,
            Self::ItemWithoutOutcome => 1,
            Self::BindingMissing => 2,
            Self::SectionUntouched => 3,
            Self::ExampleFailing => 4,
            Self::GeneratedStale => 5,
            Self::TenseMismatch => 6,
            Self::ClaimUnsupported => 7,
            Self::EvidenceStale => 8,
            Self::EvidenceChanged => 9,
        }
    }
}

/// The last rank plus one is the number of kinds, which `ALL` must list.
const _: () = assert!(ObligationKind::EvidenceChanged.rank() + 1 == ObligationKind::ALL.len());

impl PartialOrd for ObligationKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ObligationKind {
    /// In the order of [`ObligationKind::ALL`].
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl fmt::Display for ObligationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The document an obligation is about.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Subject {
    /// A capability, by id: `cap:<slug>`.
    Capability(String),
    /// The change record itself, by id: `change:<slug>`.
    Change(String),
    /// The capability index, `docs/capabilities/index.md`.
    Index,
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Capability(id) | Self::Change(id) => f.write_str(id),
            Self::Index => f.write_str(crate::generated::INDEX_PATH),
        }
    }
}

/// The part of the subject an obligation is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Part {
    /// The document as a whole, or text before its first section.
    Document,
    /// One of the seven sections of a capability.
    Section(CapabilitySection),
    /// An item of a change record, counted from 1 in the order the record
    /// lists them.
    Item(usize),
}

impl Part {
    fn rank(self) -> (u8, usize) {
        match self {
            Self::Document => (0, 0),
            Self::Section(section) => (1, position(section)),
            Self::Item(number) => (2, number),
        }
    }
}

impl PartialOrd for Part {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Part {
    /// The document, then the sections in the order a capability holds them,
    /// then the items in the order of the record.
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Document => f.write_str("document"),
            Self::Section(section) => f.write_str(section.slug()),
            Self::Item(number) => write!(f, "item {number}"),
        }
    }
}

/// Where in the repository an obligation is: the file and, for a finding in
/// the text of a document, the 1-based line.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Place {
    /// The repository-relative path of the file, with `/` separators.
    pub path: String,
    /// The line, for an obligation about a line of text.
    pub line: Option<usize>,
}

/// Why an item of a record is unresolved.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OutcomeProblem {
    /// The item has no `outcome`.
    Missing,
    /// The outcome is `moved:change:<slug>` and no change record has the
    /// slug.
    MovedToMissing(String),
    /// The outcome is `moved:change:<slug>` with the slug of the record
    /// itself, which is deleted when the change completes.
    MovedToItself,
}

/// Why a binding is owed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BindingWhy {
    /// The list has no entry.
    Empty,
    /// The path does not resolve through the source index; the reason the
    /// index gives.
    DoesNotResolve(String),
    /// The path resolves to an item its list does not take.
    WrongRole(&'static str),
}

/// Why a bound example or documentation test is not a passing run.
///
/// It is read from the [`Evidence`] of the run and carries no time and no
/// output of the process, so equal inputs print equal lines.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RunProblem {
    /// A process of the run exited with a failing status: the build failed or
    /// the example, or a documentation test, failed. `None` is a signal.
    Fails {
        /// The exit code of the last process, or `None` when a signal ended
        /// it.
        code: Option<i32>,
    },
    /// A process ran past its wall-clock limit and was killed.
    TimedOut,
    /// A process wrote more than the output bound and was killed.
    Overflowed,
    /// The binding cannot be run and counted: it ran no documentation test
    /// (zero tests), a documentation test was built and not run, or the index
    /// facts or a value refuse it.
    Unresolved(Refusal),
}

impl RunProblem {
    /// What is wrong with the run of `evidence`, or `None` when it passed.
    pub(crate) fn of(evidence: &Evidence) -> Option<Self> {
        match evidence.outcome() {
            BoundOutcome::Passed => None,
            BoundOutcome::Unresolved { reason } => Some(Self::Unresolved(reason.clone())),
            BoundOutcome::Failed { .. } => Some(match evidence.runs().last().map(Run::ended) {
                Some(Ended::TimedOut) => Self::TimedOut,
                Some(Ended::Overflowed) => Self::Overflowed,
                Some(Ended::Exited(code)) => Self::Fails { code },
                _ => Self::Fails { code: None },
            }),
        }
    }
}

/// What a sync would repair.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SyncProblem {
    /// A generated block of this kind differs from what a sync writes.
    Stale(String),
    /// `docs/capabilities/index.md` does not exist.
    IndexMissing,
}

/// What was found in text that is not the supported behaviour.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Found {
    /// A phrase of [`tense::PHRASES`].
    Phrase(&'static str),
    /// A PROPOSED block of the change with this slug in a delivered
    /// capability.
    ProposedBlock(String),
}

/// Why a claim is not supported by one evidence record it names.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ClaimProblem {
    /// No evidence record with the id loaded: there is none, or its file does
    /// not validate.
    Missing,
    /// The record's verdict is `fail`.
    Fails,
    /// The record holds a value over its limit.
    OverLimit(OverLimit),
}

/// A measured value over a limit.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct OverLimit {
    /// The name of the value.
    pub name: String,
    /// The value, as written.
    pub value: String,
    /// The limit, as written.
    pub limit: String,
}

impl fmt::Display for OverLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "`{}` is {}, over its limit {}",
            self.name, self.value, self.limit
        )
    }
}

/// Why an input of an evidence record is stale.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum InputMoved {
    /// The text the input names has another digest now.
    Moved,
    /// The input names nothing that resolves now; what the source index says.
    Unresolved(String),
}

/// How an evidence record changed since the merge base, where it passed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EvidenceChange {
    /// The verdict went from `pass` to `fail`.
    Failed,
    /// A value is over a limit the record or the record at the merge base
    /// states.
    OverLimit(OverLimit),
    /// A limit of the record at the merge base is gone.
    LimitRemoved(String),
    /// No record with the id loads now.
    Removed,
}

/// The evidence that settles an obligation.
///
/// `Display` renders the evidence column of the reconcile contract, and where
/// the obligation is about one thing, the thing in parentheses after it:
/// `a binding that resolves through the source index (bindings.tests is
/// empty)`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Needed {
    /// The block is replaced by plain contract text, or removed because the
    /// claim was dropped.
    ProposalResolved,
    /// An outcome of `implemented`, `dropped` or `moved:change:<slug>`.
    Outcome(OutcomeProblem),
    /// A binding that resolves through the source index.
    ResolvedBinding {
        /// The list the binding is in.
        list: BindingKind,
        /// The path of the entry, or `None` for an empty list.
        path: Option<String>,
        /// What is wrong.
        why: BindingWhy,
    },
    /// A diff of the section against the merge base, or an edit that records
    /// why none is needed.
    SectionDiff,
    /// A passing `verify` run for the change, or a binding edit that removes
    /// the broken example.
    PassingRun {
        /// The binding path of the example or documentation test.
        binding: String,
        /// What is wrong with the run.
        problem: RunProblem,
    },
    /// A `docs sync` run.
    SyncRun(SyncProblem),
    /// Text rewritten as the supported behaviour.
    SupportedBehaviour(Found),
    /// A passing record within its limits for each evidence id the claim
    /// names, or the claim reworded or removed.
    SupportingEvidence {
        /// The id of the record, `evidence:<slug>`.
        evidence: String,
        /// What is wrong with it.
        problem: ClaimProblem,
    },
    /// The evidence recorded again with `evidence record`.
    RecordedAgain {
        /// The id of the record.
        evidence: String,
        /// The name of the input that moved.
        input: String,
        /// How it moved.
        moved: InputMoved,
    },
    /// The claim reviewed against the new result: the evidence passes within
    /// the limits of the merge base again, or the claim no longer names it.
    ReviewedClaim {
        /// The id of the record.
        evidence: String,
        /// How it changed.
        change: EvidenceChange,
    },
}

impl Needed {
    /// The evidence as the reconcile contract words it, without the detail.
    #[must_use]
    pub const fn contract_text(&self) -> &'static str {
        match self {
            Self::ProposalResolved => {
                "the block is replaced by plain contract text, or removed because the claim was dropped"
            }
            Self::Outcome(_) => "an outcome of implemented, dropped or moved:change:<slug>",
            Self::ResolvedBinding { .. } => "a binding that resolves through the source index",
            Self::SectionDiff => {
                "a diff of that section against the merge base, or an edit that records why none is needed"
            }
            Self::PassingRun { .. } => {
                "a passing verify run for the change, or a binding edit that removes the broken example"
            }
            Self::SyncRun(_) => "a docs sync run",
            Self::SupportedBehaviour(_) => "text rewritten as the supported behaviour",
            Self::SupportingEvidence { .. } => {
                "a passing record within its limits for each evidence id the claim names, or the claim reworded or removed"
            }
            Self::RecordedAgain { .. } => "the evidence recorded again with evidence record",
            Self::ReviewedClaim { .. } => {
                "the claim reviewed against the new result: the evidence passes within the limits of the merge base again, or the claim no longer names it"
            }
        }
    }

    /// The one thing the evidence is about, when there is one.
    fn detail(&self) -> Option<String> {
        match self {
            Self::ProposalResolved | Self::SectionDiff | Self::Outcome(OutcomeProblem::Missing) => {
                None
            }
            Self::Outcome(OutcomeProblem::MovedToMissing(slug)) => {
                Some(format!("`moved:change:{slug}` names no change record"))
            }
            Self::Outcome(OutcomeProblem::MovedToItself) => {
                Some("the item moves to the change that is being completed".to_string())
            }
            Self::ResolvedBinding { list, path, why } => {
                let list = list.key();
                Some(match (path, why) {
                    (None, _) | (_, BindingWhy::Empty) => format!("bindings.{list} is empty"),
                    (Some(path), BindingWhy::DoesNotResolve(reason)) => {
                        format!("bindings.{list} `{path}` does not resolve: {reason}")
                    }
                    (Some(path), BindingWhy::WrongRole(why)) => {
                        format!("bindings.{list} `{path}` {why}")
                    }
                })
            }
            Self::PassingRun { binding, problem } => Some(match problem {
                RunProblem::Fails { code: Some(code) } => {
                    format!("`{binding}` fails to build or run (exit code {code})")
                }
                RunProblem::Fails { code: None } => {
                    format!("`{binding}` fails to build or run (ended by a signal)")
                }
                RunProblem::TimedOut => {
                    format!("`{binding}` ran past its time limit and was killed")
                }
                RunProblem::Overflowed => {
                    format!("`{binding}` wrote more output than the bound and was killed")
                }
                RunProblem::Unresolved(refusal) => format!("`{binding}` proves nothing: {refusal}"),
            }),
            Self::SyncRun(SyncProblem::Stale(block)) => {
                Some(format!("the `{block}` block is stale"))
            }
            Self::SyncRun(SyncProblem::IndexMissing) => {
                Some(format!("{} does not exist", crate::generated::INDEX_PATH))
            }
            Self::SupportedBehaviour(Found::Phrase(phrase)) => {
                Some(format!("the text says \"{phrase}\""))
            }
            Self::SupportedBehaviour(Found::ProposedBlock(change)) => Some(format!(
                "a PROPOSED block of change:{change} stands in a delivered capability"
            )),
            Self::SupportingEvidence { evidence, problem } => Some(match problem {
                ClaimProblem::Missing => format!("`{evidence}` is not a loaded evidence record"),
                ClaimProblem::Fails => format!("`{evidence}` has the verdict fail"),
                ClaimProblem::OverLimit(over) => format!("`{evidence}`: {over}"),
            }),
            Self::RecordedAgain {
                evidence,
                input,
                moved,
            } => Some(match moved {
                InputMoved::Moved => {
                    format!("`{evidence}` was made against `{input}`, which has changed since")
                }
                InputMoved::Unresolved(reason) => format!(
                    "`{evidence}` was made against `{input}`, which does not resolve now: {reason}"
                ),
            }),
            Self::ReviewedClaim { evidence, change } => Some(match change {
                EvidenceChange::Failed => {
                    format!("`{evidence}` passed at the merge base and fails now")
                }
                EvidenceChange::OverLimit(over) => format!("`{evidence}`: {over}"),
                EvidenceChange::LimitRemoved(name) => format!(
                    "`{evidence}` no longer has the limit `{name}` it had at the merge base"
                ),
                EvidenceChange::Removed => {
                    format!("`{evidence}` passed at the merge base and is not a loaded record now")
                }
            }),
        }
    }
}

impl fmt::Display for Needed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.contract_text())?;
        match self.detail() {
            Some(detail) => write!(f, " ({detail})"),
            None => Ok(()),
        }
    }
}

/// One statement of work the documentation owes.
///
/// `Display` is one line in the shape of the reconcile contract: the subject,
/// the part, the kind and the evidence, each separated by two spaces, with
/// `line <n>: ` before the evidence of an obligation about a line of text.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Obligation {
    /// What is owed.
    pub kind: ObligationKind,
    /// The capability, the change record or the index it is about.
    pub subject: Subject,
    /// The section or item of the subject.
    pub section: Part,
    /// The file and, for text, the line.
    pub location: Place,
    /// What settles it.
    pub evidence: Needed,
}

/// The fields an obligation is ordered by: subject, part, kind, line,
/// evidence text, then path.
type SortKey = (Subject, Part, ObligationKind, Option<usize>, String, String);

impl Obligation {
    fn key(&self) -> SortKey {
        (
            self.subject.clone(),
            self.section,
            self.kind,
            self.location.line,
            self.evidence.to_string(),
            self.location.path.clone(),
        )
    }
}

impl fmt::Display for Obligation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}  {}  {}  ", self.subject, self.section, self.kind)?;
        if let Some(line) = self.location.line {
            write!(f, "line {line}: ")?;
        }
        write!(f, "{}", self.evidence)
    }
}

/// Whether `effect` leaves an obligation of `kind` in `section` out of the
/// list.
///
/// `contract_effect: none` records that the contract, usage and examples are
/// unaffected, so `section-untouched` is not raised for those three sections.
/// Nothing else is suppressed: stale bindings, stale generated blocks, open
/// proposals, items without outcome, tense and the claims and evidence the
/// documents rest on are owed whatever the effect says.
#[must_use]
pub fn suppressed(kind: ObligationKind, section: Part, effect: ContractEffect) -> bool {
    effect == ContractEffect::None
        && kind == ObligationKind::SectionUntouched
        && matches!(
            section,
            Part::Section(
                CapabilitySection::Contract
                    | CapabilitySection::Usage
                    | CapabilitySection::Examples
            )
        )
}

/// The obligations of a change, sorted.
///
/// `Display` prints one line for each obligation and nothing for an empty
/// list. The exit code of the command that prints it is 0 for an empty list
/// and 1 otherwise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reconciled {
    change: String,
    obligations: Vec<Obligation>,
}

impl Reconciled {
    /// The list for the change with the slug `change`: sorted by subject,
    /// part, kind, line and evidence text, without repeats.
    pub(crate) fn new(change: String, mut obligations: Vec<Obligation>) -> Self {
        obligations.sort_by_cached_key(Obligation::key);
        obligations.dedup();
        Self {
            change,
            obligations,
        }
    }

    /// The slug of the change, without the `change:` prefix.
    #[must_use]
    pub fn change(&self) -> &str {
        &self.change
    }

    /// The obligations, in order.
    #[must_use]
    pub fn obligations(&self) -> &[Obligation] {
        &self.obligations
    }

    /// The obligations of one kind, in order.
    pub fn of_kind(&self, kind: ObligationKind) -> impl Iterator<Item = &Obligation> {
        self.obligations
            .iter()
            .filter(move |obligation| obligation.kind == kind)
    }

    /// Whether nothing is owed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.obligations.is_empty()
    }

    /// The number of obligations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.obligations.len()
    }
}

impl fmt::Display for Reconciled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for obligation in &self.obligations {
            writeln!(f, "{obligation}")?;
        }
        Ok(())
    }
}

/// Why a change cannot be reconciled.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReconcileError {
    /// The slug names no loaded change record.
    Change(ChangeLookupError),
    /// The change lists a capability that is not loaded, so what it owes is
    /// unknown. (A delivered capability that is refused only for an empty
    /// `code` or `tests` list is not this error: it owes that binding.)
    CapabilityNotLoaded {
        /// The slug of the change.
        change: String,
        /// The id of the capability.
        capability: String,
        /// The findings about the capability's document, when it exists and
        /// did not load.
        findings: Vec<Finding>,
    },
}

impl fmt::Display for ReconcileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Change(error) => error.fmt(f),
            Self::CapabilityNotLoaded {
                change,
                capability,
                findings,
            } => {
                write!(
                    f,
                    "change `{change}` lists {capability}, which is not loaded, so what it owes is unknown; fix the capability first"
                )?;
                write_findings(f, findings)
            }
        }
    }
}

impl Error for ReconcileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Change(error) => Some(error),
            Self::CapabilityNotLoaded { .. } => None,
        }
    }
}

impl From<ChangeLookupError> for ReconcileError {
    fn from(error: ChangeLookupError) -> Self {
        Self::Change(error)
    }
}

impl Workplan<'_> {
    /// The obligations of the change `change`: what the documentation still
    /// owes it, each with the evidence that settles it.
    ///
    /// `change` is the slug of a change record, with or without the `change:`
    /// prefix. The list is empty exactly when the change owes nothing that
    /// the loaded documents can show; `section-untouched`, `example-failing`
    /// and `evidence-changed` need a process and are not part of it (see
    /// [`ObligationKind::is_static`]). Nothing is written and no process is
    /// started.
    ///
    /// ```rust
    /// use raf_source::{MemoryTree, Workspace};
    /// use raf_workplan::{ChangeLookupError, Workplan, obligation::ReconcileError};
    ///
    /// let source = Workspace::load(
    ///     &MemoryTree::from([
    ///         ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///         ("src/lib.rs", ""),
    ///     ]),
    ///     &["."],
    /// )?;
    /// let plan = Workplan::load(&MemoryTree::from([("docs/architecture/.keep", "")]), &source);
    /// assert!(matches!(
    ///     plan.reconcile("grow"),
    ///     Err(ReconcileError::Change(ChangeLookupError::Unknown { .. }))
    /// ));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`ReconcileError::Change`] when no change record has the slug or the
    /// record did not load; [`ReconcileError::CapabilityNotLoaded`] when the
    /// record lists a capability that is not loaded.
    pub fn reconcile(&self, change: &str) -> Result<Reconciled, ReconcileError> {
        sources::reconcile(self, change)
    }
}
