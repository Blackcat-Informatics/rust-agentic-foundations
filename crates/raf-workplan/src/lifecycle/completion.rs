// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! What `complete` returns: the removal that happened, or every reason it did
//! not.

use std::fmt;

use raf_okf::Location;

use crate::Finding;
use crate::bound::Evidence;
use crate::finding::kind;
use crate::obligation::{Obligation, ObligationKind};

use super::verification::evidence_line;

/// The condition of the completion gate a [`Failure`] is a fault of.
///
/// The five conditions of the Contract are numbered as it numbers them. The
/// two others are not conditions of the list: the check of the repository as
/// it will be once the record is gone, and the repository changing while the
/// bound tests and examples ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Condition {
    /// Condition 1: the obligation list of `reconcile` is not empty.
    Obligations,
    /// Condition 2: an item has no valid `outcome`, or a `moved` outcome names
    /// no change record.
    Outcomes,
    /// Condition 3: a PROPOSED block of the change remains in a capability
    /// document.
    Proposals,
    /// Condition 4: `docs sync` would write something.
    GeneratedBlocks,
    /// Condition 5: a bound test or example does not pass.
    BoundRuns,
    /// The repository without the record has a `check` finding.
    AfterRemoval,
    /// The repository changed between the bound runs and the writes.
    ChangedDuringRun,
}

impl Condition {
    /// The number the Contract gives the condition, for the five of its list.
    #[must_use]
    pub const fn number(self) -> Option<u8> {
        match self {
            Self::Obligations => Some(1),
            Self::Outcomes => Some(2),
            Self::Proposals => Some(3),
            Self::GeneratedBlocks => Some(4),
            Self::BoundRuns => Some(5),
            Self::AfterRemoval | Self::ChangedDuringRun => None,
        }
    }
}

/// One reason `complete` refuses.
///
/// A fault that two conditions both state stands as one failure under each:
/// an unresolved PROPOSED block is an obligation of condition 1 and a failure
/// of condition 3, a stale block in a listed capability is an obligation and a
/// failure of condition 4, an item without outcome is an obligation and a
/// failure of condition 2, and a failing bound example is an `example-failing`
/// obligation and a failure of condition 5.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Failure {
    /// Condition 1: one obligation of the list, of any of the seven kinds.
    Owed(Obligation),
    /// Condition 2: an item of the record with no valid outcome (an
    /// `item-without-outcome` obligation).
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
            Self::AfterRemoval(_) => Condition::AfterRemoval,
            Self::Changed { .. } => Condition::ChangedDuringRun,
        }
    }

    /// The obligation the failure is, for conditions 1 to 3.
    #[must_use]
    pub fn obligation(&self) -> Option<&Obligation> {
        match self {
            Self::Owed(obligation) | Self::Item(obligation) | Self::Proposal(obligation) => {
                Some(obligation)
            }
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

    /// Whether the line of the failure repeats the line of another failure in
    /// `all`, so that `Display` of a [`Completion`] prints it once.
    fn repeats(&self, all: &[Failure]) -> bool {
        match self {
            Self::Item(_) | Self::Proposal(_) | Self::Example(_) => true,
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
            Self::Owed(obligation) | Self::Item(obligation) | Self::Proposal(obligation) => {
                obligation.fmt(f)
            }
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
            count(|f| matches!(f, Failure::Item(_))),
            "item without outcome",
            "items without outcome",
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

impl fmt::Display for Completion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Completed { removed, rewritten } => {
                writeln!(f, "completed: removed {removed}")?;
                for path in rewritten {
                    writeln!(f, "rewrote {path}")?;
                }
                Ok(())
            }
            Self::Refused { failures } => {
                writeln!(f, "refused: {}", summary(failures))?;
                for failure in failures.iter().filter(|failure| !failure.repeats(failures)) {
                    writeln!(f, "{failure}")?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use raf_okf::Location;

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
    fn conditions_are_numbered_as_the_contract_numbers_them() {
        let numbers: Vec<Option<u8>> = [
            Condition::Obligations,
            Condition::Outcomes,
            Condition::Proposals,
            Condition::GeneratedBlocks,
            Condition::BoundRuns,
            Condition::AfterRemoval,
            Condition::ChangedDuringRun,
        ]
        .into_iter()
        .map(Condition::number)
        .collect();
        assert_eq!(
            numbers,
            [Some(1), Some(2), Some(3), Some(4), Some(5), None, None]
        );
    }
}
