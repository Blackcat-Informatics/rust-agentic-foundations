// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! What `complete` returns: the removal that happened, or every reason it did
//! not.

use std::collections::BTreeSet;
use std::fmt;

use raf_okf::Location;

use crate::Finding;
use crate::bound::Evidence;
use crate::finding::kind;
use crate::obligation::{Obligation, ObligationKind};

use super::verification::evidence_line;

/// The condition of the completion gate a [`Failure`] is a fault of.
///
/// The six conditions of the Contract are numbered as it numbers them. The
/// two others are not conditions of the list: the check of the repository as
/// it will be once the record is gone, and the repository changing while the
/// bound tests and examples ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Condition {
    /// Condition 1: the obligation list of `reconcile` is not empty.
    Obligations,
    /// Condition 2: an item has no valid `outcome`, or a `moved` outcome names
    /// no change record, or, under strict mode, a `dropped` or `moved` item
    /// has no `reason`.
    Outcomes,
    /// Condition 3: a PROPOSED block of the change remains in a capability
    /// document.
    Proposals,
    /// Condition 4: `docs sync` would write something.
    GeneratedBlocks,
    /// Condition 5: a bound test or example does not pass.
    BoundRuns,
    /// Condition 6: an authored section in `affects` has no current
    /// assessment with the verdict `accepted`.
    Assessments,
    /// The repository without the record has a `check` finding.
    AfterRemoval,
    /// The repository changed between the bound runs and the writes.
    ChangedDuringRun,
}

impl Condition {
    /// The stable kebab-case name of the condition.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Obligations => "obligations",
            Self::Outcomes => "outcomes",
            Self::Proposals => "proposals",
            Self::GeneratedBlocks => "generated-blocks",
            Self::BoundRuns => "bound-runs",
            Self::Assessments => "assessments",
            Self::AfterRemoval => "after-removal",
            Self::ChangedDuringRun => "changed-during-run",
        }
    }

    /// The number the Contract gives the condition, for the six of its list.
    #[must_use]
    pub const fn number(self) -> Option<u8> {
        match self {
            Self::Obligations => Some(1),
            Self::Outcomes => Some(2),
            Self::Proposals => Some(3),
            Self::GeneratedBlocks => Some(4),
            Self::BoundRuns => Some(5),
            Self::Assessments => Some(6),
            Self::AfterRemoval | Self::ChangedDuringRun => None,
        }
    }
}

/// One reason `complete` refuses.
///
/// A fault that two conditions both state stands as one failure under each:
/// an unresolved PROPOSED block is an obligation of condition 1 and a failure
/// of condition 3, a stale block in a listed capability is an obligation and a
/// failure of condition 4, an item without outcome, or under strict mode a
/// `dropped` or `moved` item without reason, is an obligation and a failure of
/// condition 2, a failing bound example is an `example-failing`
/// obligation and a failure of condition 5, and an affected section without a
/// current accepted assessment is an `assessment-missing`,
/// `assessment-stale` or `assessment-rejected` obligation and a failure of
/// condition 6.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Failure {
    /// Condition 1: one obligation of the list, of any kind.
    Owed(Obligation),
    /// Condition 2: an item of the record with no valid outcome (an
    /// `item-without-outcome` obligation), or, under strict mode, a `dropped`
    /// or `moved` item with no `reason` (an `outcome-without-reason`
    /// obligation).
    Item(Obligation),
    /// Condition 3: a PROPOSED block of the change that remains, in a document
    /// that loaded or not (an `unresolved-proposal` obligation).
    Proposal(Obligation),
    /// Condition 4: a generated block `docs sync` would rewrite, or a file it
    /// would refuse.
    Generated(Finding),
    /// Condition 4: the rewrite of a file does not read back as a document.
    Rewrite {
        /// The file.
        file: String,
        /// Why.
        reason: String,
    },
    /// Condition 5: a bound test that did not pass, or whose run proves
    /// nothing.
    Test(Evidence),
    /// Condition 5: a bound example or documentation test that did not pass.
    /// It is also an `example-failing` obligation, in [`Failure::Owed`].
    Example(Evidence),
    /// Condition 6: an authored section in `affects` with no current
    /// assessment whose verdict is `accepted` (an `assessment-missing`,
    /// `assessment-stale` or `assessment-rejected` obligation). A section has
    /// one for each such obligation, one `assessment-stale` for each input
    /// that moved, and the headline counts the section once.
    Assessment(Obligation),
    /// A finding of `check` over the repository without the record. The state
    /// rules are judged here: a capability that is not `delivered` and is
    /// listed by no other open record is `no-open-record`.
    AfterRemoval(Finding),
    /// The repository changed after the bound tests and examples ran, so what
    /// they proved is not what would be completed.
    Changed {
        /// What differs.
        what: String,
    },
}

impl Failure {
    /// The condition the failure is a fault of.
    #[must_use]
    pub fn condition(&self) -> Condition {
        match self {
            Self::Owed(_) => Condition::Obligations,
            Self::Item(_) => Condition::Outcomes,
            Self::Proposal(_) => Condition::Proposals,
            Self::Generated(_) | Self::Rewrite { .. } => Condition::GeneratedBlocks,
            Self::Test(_) | Self::Example(_) => Condition::BoundRuns,
            Self::Assessment(_) => Condition::Assessments,
            Self::AfterRemoval(_) => Condition::AfterRemoval,
            Self::Changed { .. } => Condition::ChangedDuringRun,
        }
    }

    /// The obligation the failure is, for conditions 1 to 3 and 6.
    #[must_use]
    pub fn obligation(&self) -> Option<&Obligation> {
        match self {
            Self::Owed(obligation)
            | Self::Item(obligation)
            | Self::Proposal(obligation)
            | Self::Assessment(obligation) => Some(obligation),
            _ => None,
        }
    }

    /// The kind of the obligation the failure is.
    #[must_use]
    pub fn obligation_kind(&self) -> Option<ObligationKind> {
        self.obligation().map(|obligation| obligation.kind)
    }

    /// The kind of the `check` finding the failure is, for the failures that
    /// are findings.
    #[must_use]
    pub fn finding_kind(&self) -> Option<&'static str> {
        match self {
            Self::Generated(finding) | Self::AfterRemoval(finding) => Some(finding.kind),
            _ => None,
        }
    }

    /// The stable kebab-case name of what failed: the kind of the obligation
    /// for conditions 1 to 3 and 6, the kind of the finding for a generated block or
    /// a finding once the record is removed, and `generated-rewrite`,
    /// `bound-test`, `bound-example` or `changed-during-run` for the others.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Owed(obligation)
            | Self::Item(obligation)
            | Self::Proposal(obligation)
            | Self::Assessment(obligation) => obligation.kind.name(),
            Self::Generated(finding) | Self::AfterRemoval(finding) => finding.kind,
            Self::Rewrite { .. } => "generated-rewrite",
            Self::Test(_) => "bound-test",
            Self::Example(_) => "bound-example",
            Self::Changed { .. } => "changed-during-run",
        }
    }

    /// Whether the line of the failure repeats the line of another failure in
    /// `all`, so that `Display` of a [`Completion`] prints it once.
    #[must_use]
    pub fn is_repeated_in(&self, all: &[Failure]) -> bool {
        match self {
            Self::Item(_) | Self::Proposal(_) | Self::Example(_) | Self::Assessment(_) => true,
            Self::Generated(finding) => {
                finding.kind == kind::GENERATED_STALE
                    && all.iter().filter_map(Failure::obligation).any(|owed| {
                        owed.kind == ObligationKind::GeneratedStale
                            && owed.location.path == finding.file
                            && matches!(
                                finding.location,
                                Location::Line(line) if owed.location.line == Some(line)
                            )
                    })
            }
            _ => false,
        }
    }
}

impl fmt::Display for Failure {
    /// One line, with no line ending.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Owed(obligation)
            | Self::Item(obligation)
            | Self::Proposal(obligation)
            | Self::Assessment(obligation) => obligation.fmt(f),
            Self::Generated(finding) => finding.fmt(f),
            Self::Rewrite { file, reason } => write!(
                f,
                "{file}: the rewritten generated blocks do not read back as a document: {reason}"
            ),
            Self::Test(evidence) | Self::Example(evidence) => f.write_str(&evidence_line(evidence)),
            Self::AfterRemoval(finding) => {
                write!(f, "once the record is removed, {finding}")
            }
            Self::Changed { what } => write!(
                f,
                "the repository changed after the bound tests and examples ran, so nothing was written: {what}"
            ),
        }
    }
}

/// How `change complete` ended when it could look at the repository.
///
/// [`Completion::Refused`] is the exit code 1 of the command and
/// [`Completion::Completed`] the exit code 0. `Display` is the text the
/// command prints: for a refusal, `refused:` and a count for each kind of
/// failure, then one line for each failure that no other line repeats; for a
/// completion, `completed: removed <path>` and a line for each file whose
/// generated blocks were rewritten.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Completion {
    /// The generated blocks the removal stales were rewritten and the record
    /// was removed, in that order, and nothing else was written.
    Completed {
        /// The repository-relative path of the record that was removed.
        removed: String,
        /// The repository-relative paths of the files whose generated blocks
        /// were rewritten, sorted.
        rewritten: Vec<String>,
    },
    /// Nothing was written. Every failure is listed, in the order of the
    /// conditions, and none is left out because another was found.
    Refused {
        /// The failures, never empty.
        failures: Vec<Failure>,
    },
}

impl Completion {
    /// Whether the record was removed.
    #[must_use]
    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }

    /// The failures of a refusal; none for a completion.
    #[must_use]
    pub fn failures(&self) -> &[Failure] {
        match self {
            Self::Completed { .. } => &[],
            Self::Refused { failures } => failures,
        }
    }

    /// The failures of one condition.
    pub fn of_condition(&self, condition: Condition) -> impl Iterator<Item = &Failure> {
        self.failures()
            .iter()
            .filter(move |failure| failure.condition() == condition)
    }

    /// The obligations that stand as failures, of every kind.
    pub fn obligations(&self) -> impl Iterator<Item = &Obligation> {
        self.failures().iter().filter_map(|failure| match failure {
            Failure::Owed(obligation) => Some(obligation),
            _ => None,
        })
    }
}

/// `1 obligation` or `2 obligations`.
fn counted(count: usize, one: &str, many: &str) -> Option<String> {
    match count {
        0 => None,
        1 => Some(format!("1 {one}")),
        _ => Some(format!("{count} {many}")),
    }
}

/// The sections that condition 6 refuses: one for each capability and
/// section, however many `assessment-*` obligations it has (one
/// `assessment-stale` for each input that moved).
fn sections_without_assessment(failures: &[Failure]) -> usize {
    failures
        .iter()
        .filter_map(|failure| match failure {
            Failure::Assessment(owed) => Some((&owed.subject, owed.section)),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .len()
}

/// The counts of a refusal, in the order of the conditions.
fn summary(failures: &[Failure]) -> String {
    let count = |wanted: fn(&Failure) -> bool| failures.iter().filter(|f| wanted(f)).count();
    let stale = count(
        |f| matches!(f, Failure::Generated(finding) if finding.kind == kind::GENERATED_STALE),
    );
    let faulty = count(|f| matches!(f, Failure::Generated(_) | Failure::Rewrite { .. })) - stale;
    let parts = [
        counted(
            count(|f| matches!(f, Failure::Owed(_))),
            "obligation",
            "obligations",
        ),
        counted(
            count(
                |f| matches!(f, Failure::Item(owed) if owed.kind != ObligationKind::OutcomeWithoutReason),
            ),
            "item without outcome",
            "items without outcome",
        ),
        counted(
            count(
                |f| matches!(f, Failure::Item(owed) if owed.kind == ObligationKind::OutcomeWithoutReason),
            ),
            "item without a reason",
            "items without a reason",
        ),
        counted(
            count(|f| matches!(f, Failure::Proposal(_))),
            "unresolved PROPOSED block",
            "unresolved PROPOSED blocks",
        ),
        counted(stale, "stale generated block", "stale generated blocks"),
        counted(faulty, "generated block fault", "generated block faults"),
        counted(
            count(|f| matches!(f, Failure::Test(_))),
            "bound test failing",
            "bound tests failing",
        ),
        counted(
            count(|f| matches!(f, Failure::Example(_))),
            "bound example failing",
            "bound examples failing",
        ),
        counted(
            sections_without_assessment(failures),
            "section without a current accepted assessment",
            "sections without a current accepted assessment",
        ),
        counted(
            count(|f| matches!(f, Failure::AfterRemoval(_))),
            "finding once the record is removed",
            "findings once the record is removed",
        ),
        counted(
            count(|f| matches!(f, Failure::Changed { .. })),
            "change during the run",
            "changes during the run",
        ),
    ];
    parts.into_iter().flatten().collect::<Vec<_>>().join(", ")
}

impl Completion {
    /// The first line of the text of the completion, without its line ending:
    /// `completed: removed <path>`, or `refused:` and the count of each kind
    /// of failure.
    #[must_use]
    pub fn headline(&self) -> String {
        match self {
            Self::Completed { removed, .. } => format!("completed: removed {removed}"),
            Self::Refused { failures } => format!("refused: {}", summary(failures)),
        }
    }
}

impl fmt::Display for Completion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.headline())?;
        match self {
            Self::Completed { rewritten, .. } => {
                for path in rewritten {
                    writeln!(f, "rewrote {path}")?;
                }
            }
            Self::Refused { failures } => {
                for failure in failures
                    .iter()
                    .filter(|failure| !failure.is_repeated_in(failures))
                {
                    writeln!(f, "{failure}")?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use raf_okf::{CapabilitySection, Location};

    use super::{Completion, Condition, Failure};
    use crate::Finding;
    use crate::finding::kind;
    use crate::obligation::{
        Needed, Obligation, ObligationKind, OutcomeProblem, Part, Place, Subject, SyncProblem,
    };

    fn item(number: usize) -> Obligation {
        Obligation {
            kind: ObligationKind::ItemWithoutOutcome,
            subject: Subject::Change("change:shape".to_string()),
            section: Part::Item(number),
            location: Place {
                path: "work/changes/shape.md".to_string(),
                line: None,
            },
            evidence: Needed::Outcome(OutcomeProblem::Missing),
        }
    }

    fn stale(path: &str, line: usize) -> Obligation {
        Obligation {
            kind: ObligationKind::GeneratedStale,
            subject: Subject::Index,
            section: Part::Document,
            location: Place {
                path: path.to_string(),
                line: Some(line),
            },
            evidence: Needed::SyncRun(SyncProblem::Stale("catalogue".to_string())),
        }
    }

    fn finding(kind: &'static str, file: &str, line: usize) -> Finding {
        Finding::new(kind, file, Location::Line(line), "a message", "a fix")
    }

    #[test]
    fn a_completion_prints_the_removal_and_each_rewritten_file() {
        let done = Completion::Completed {
            removed: "work/changes/shape.md".to_string(),
            rewritten: Vec::new(),
        };
        assert_eq!(
            done.to_string(),
            "completed: removed work/changes/shape.md\n"
        );
        assert!(done.is_completed());
        assert_eq!(done.failures(), []);
        let done = Completion::Completed {
            removed: "work/changes/shape.md".to_string(),
            rewritten: vec!["a.md".to_string(), "b.md".to_string()],
        };
        assert_eq!(
            done.to_string(),
            "completed: removed work/changes/shape.md\nrewrote a.md\nrewrote b.md\n"
        );
    }

    #[test]
    fn the_counts_of_a_refusal_are_singular_or_plural_and_in_the_order_of_the_conditions() {
        let refused = Completion::Refused {
            failures: vec![
                Failure::Owed(item(1)),
                Failure::Owed(item(2)),
                Failure::Item(item(1)),
                Failure::Item(item(2)),
                Failure::Generated(finding(kind::GENERATED_STALE, "a.md", 3)),
                Failure::Generated(finding(kind::GENERATED_UNKNOWN_KIND, "b.md", 4)),
                Failure::Rewrite {
                    file: "c.md".to_string(),
                    reason: "too long".to_string(),
                },
                Failure::AfterRemoval(finding(kind::NO_OPEN_RECORD, "d.md", 1)),
                Failure::Changed {
                    what: "it moved".to_string(),
                },
            ],
        };
        let text = refused.to_string();
        assert_eq!(
            text.lines().next(),
            Some(
                "refused: 2 obligations, 2 items without outcome, 1 stale generated block, 2 generated block faults, 1 finding once the record is removed, 1 change during the run"
            )
        );
        // The obligations print once each, the repeats not at all.
        assert_eq!(text.matches("item-without-outcome").count(), 2, "{text}");
        assert_eq!(refused.failures().len(), 9);
        assert_eq!(refused.of_condition(Condition::Outcomes).count(), 2);
        assert_eq!(refused.obligations().count(), 2);
    }

    #[test]
    fn a_stale_block_prints_a_line_unless_an_obligation_already_does() {
        let owed = Failure::Owed(stale("docs/capabilities/index.md", 7));
        let same = Failure::Generated(finding(
            kind::GENERATED_STALE,
            "docs/capabilities/index.md",
            7,
        ));
        let elsewhere = Failure::Generated(finding(
            kind::GENERATED_STALE,
            "docs/capabilities/other.md",
            7,
        ));
        let other_line = Failure::Generated(finding(
            kind::GENERATED_STALE,
            "docs/capabilities/index.md",
            9,
        ));
        let refused = Completion::Refused {
            failures: vec![owed, same, elsewhere, other_line],
        };
        let text = refused.to_string();
        assert_eq!(
            text.lines().count(),
            4,
            "the counts, the obligation and two blocks: {text}"
        );
        assert!(text.contains("other.md: line 7"), "{text}");
        assert!(text.contains("index.md: line 9"), "{text}");
        assert!(!text.contains("index.md: line 7"), "{text}");
    }

    #[test]
    fn a_section_without_an_accepted_assessment_counts_under_condition_six_and_prints_once() {
        let missing = |section| Obligation {
            kind: ObligationKind::AssessmentMissing,
            subject: Subject::Capability("cap:shape".to_string()),
            section: Part::Section(section),
            location: Place {
                path: "docs/capabilities/shape.md".to_string(),
                line: None,
            },
            evidence: Needed::Assessed {
                subject: "cap:shape#usage".to_string(),
                assessment: "assessment:shape-usage".to_string(),
            },
        };
        let usage = missing(CapabilitySection::Usage);
        let contract = missing(CapabilitySection::Contract);
        let refused = Completion::Refused {
            failures: vec![
                Failure::Owed(contract.clone()),
                Failure::Owed(usage.clone()),
                Failure::Assessment(contract),
                Failure::Assessment(usage),
            ],
        };
        let text = refused.to_string();
        assert_eq!(
            text.lines().next(),
            Some("refused: 2 obligations, 2 sections without a current accepted assessment")
        );
        assert_eq!(text.lines().count(), 3, "{text}");
        assert_eq!(refused.of_condition(Condition::Assessments).count(), 2);
        let one = Completion::Refused {
            failures: vec![Failure::Assessment(missing(CapabilitySection::Usage))],
        };
        assert_eq!(
            one.headline(),
            "refused: 1 section without a current accepted assessment"
        );
        assert_eq!(one.failures()[0].name(), "assessment-missing");
        assert_eq!(one.failures()[0].condition().number(), Some(6));
    }

    #[test]
    fn a_section_with_three_moved_inputs_is_one_section_in_the_headline() {
        let stale = |section, input: &str| Obligation {
            kind: ObligationKind::AssessmentStale,
            subject: Subject::Capability("cap:shape".to_string()),
            section: Part::Section(section),
            location: Place {
                path: "docs/capabilities/shape.md".to_string(),
                line: None,
            },
            evidence: Needed::Assessed {
                subject: "cap:shape#usage".to_string(),
                assessment: format!("assessment:shape-usage ({input} moved)"),
            },
        };
        let refusal = |obligations: Vec<Obligation>| Completion::Refused {
            failures: obligations
                .iter()
                .cloned()
                .map(Failure::Owed)
                .chain(obligations.iter().cloned().map(Failure::Assessment))
                .collect(),
        };
        // One section, three moved inputs: three obligations and lines, one
        // section.
        let one = refusal(vec![
            stale(CapabilitySection::Usage, "section"),
            stale(CapabilitySection::Usage, "shape::push"),
            stale(CapabilitySection::Usage, "shape::finish"),
        ]);
        assert_eq!(
            one.headline(),
            "refused: 3 obligations, 1 section without a current accepted assessment"
        );
        assert_eq!(one.to_string().lines().count(), 4, "{one}");
        assert_eq!(one.of_condition(Condition::Assessments).count(), 3);
        // The neighbour: two sections, one moved input each.
        let two = refusal(vec![
            stale(CapabilitySection::Usage, "section"),
            stale(CapabilitySection::Contract, "section"),
        ]);
        assert_eq!(
            two.headline(),
            "refused: 2 obligations, 2 sections without a current accepted assessment"
        );
        // The same section of another capability is another section.
        let mut other = stale(CapabilitySection::Usage, "section");
        other.subject = Subject::Capability("cap:other".to_string());
        let apart = refusal(vec![stale(CapabilitySection::Usage, "section"), other]);
        assert_eq!(
            apart.headline(),
            "refused: 2 obligations, 2 sections without a current accepted assessment"
        );
    }

    #[test]
    fn conditions_are_numbered_as_the_contract_numbers_them() {
        let numbers: Vec<Option<u8>> = [
            Condition::Obligations,
            Condition::Outcomes,
            Condition::Proposals,
            Condition::GeneratedBlocks,
            Condition::BoundRuns,
            Condition::Assessments,
            Condition::AfterRemoval,
            Condition::ChangedDuringRun,
        ]
        .into_iter()
        .map(Condition::number)
        .collect();
        assert_eq!(
            numbers,
            [
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(6),
                None,
                None
            ]
        );
    }
}
