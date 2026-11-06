// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Static completion conditions and publication outcomes.

use crate::Finding;
use crate::obligation::Obligation;
use std::fmt;

/// One static condition of change completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Condition {
    /// No reconciliation obligation remains.
    Obligations,
    /// Every item has a valid stated outcome.
    Outcomes,
    /// No proposed text remains.
    Proposals,
    /// Generated documentation is current and writable.
    GeneratedBlocks,
    /// Source bindings are structurally valid.
    Bindings,
    /// Affected sections have current accepted assessments.
    Assessments,
    /// The prospective repository passes its static check.
    AfterRemoval,
    /// Static inputs or generated replacements changed before publication.
    ChangedBeforeWrite,
}

impl Condition {
    /// The stable name of this condition.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Obligations => "obligations",
            Self::Outcomes => "outcomes",
            Self::Proposals => "proposals",
            Self::GeneratedBlocks => "generated-blocks",
            Self::Bindings => "bindings",
            Self::Assessments => "assessments",
            Self::AfterRemoval => "after-removal",
            Self::ChangedBeforeWrite => "changed-before-write",
        }
    }

    /// The condition number in the completion contract, when numbered.
    #[must_use]
    pub const fn number(self) -> Option<u8> {
        match self {
            Self::Obligations => Some(1),
            Self::Outcomes => Some(2),
            Self::Proposals => Some(3),
            Self::GeneratedBlocks => Some(4),
            Self::Bindings => Some(5),
            Self::Assessments => Some(6),
            Self::AfterRemoval | Self::ChangedBeforeWrite => None,
        }
    }
}

/// Why static completion refuses publication.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Failure {
    /// A reconciliation obligation remains.
    Owed(Obligation),
    /// An item has no valid outcome or required reason.
    Item(Obligation),
    /// Proposed text remains.
    Proposal(Obligation),
    /// Generated documentation is stale or invalid.
    Generated(Finding),
    /// A generated replacement cannot be formed.
    Rewrite {
        /// The document path.
        file: String,
        /// The failure detail.
        reason: String,
    },
    /// A declared source binding is invalid.
    Binding(Finding),
    /// An affected section lacks a current accepted assessment.
    Assessment(Obligation),
    /// A prospective document check fails.
    AfterRemoval(Finding),
    /// The static judgement changed before publication.
    Changed {
        /// The changed observation.
        what: String,
    },
}

impl Failure {
    /// The authored obligation underlying this failure, when present.
    #[must_use]
    pub fn obligation(&self) -> Option<&Obligation> {
        match self {
            Self::Owed(value)
            | Self::Item(value)
            | Self::Proposal(value)
            | Self::Assessment(value) => Some(value),
            _ => None,
        }
    }

    /// Whether an obligation line is already represented by the general list.
    #[must_use]
    pub fn is_repeated_in(&self, failures: &[Self]) -> bool {
        match self {
            Self::Item(value) | Self::Proposal(value) | Self::Assessment(value) => failures
                .iter()
                .any(|failure| matches!(failure, Self::Owed(owed) if owed == value)),
            _ => false,
        }
    }

    /// The completion condition this failure belongs to.
    #[must_use]
    pub const fn condition(&self) -> Condition {
        match self {
            Self::Owed(_) => Condition::Obligations,
            Self::Item(_) => Condition::Outcomes,
            Self::Proposal(_) => Condition::Proposals,
            Self::Generated(_) | Self::Rewrite { .. } => Condition::GeneratedBlocks,
            Self::Binding(_) => Condition::Bindings,
            Self::Assessment(_) => Condition::Assessments,
            Self::AfterRemoval(_) => Condition::AfterRemoval,
            Self::Changed { .. } => Condition::ChangedBeforeWrite,
        }
    }

    /// The stable kind of this failure.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Owed(_) => "obligation",
            Self::Item(_) => "item-outcome",
            Self::Proposal(_) => "unresolved-proposal",
            Self::Generated(_) => "generated-stale",
            Self::Rewrite { .. } => "generated-rewrite",
            Self::Binding(_) => "binding-invalid",
            Self::Assessment(_) => "assessment-invalid",
            Self::AfterRemoval(_) => "after-removal",
            Self::Changed { .. } => "changed-before-write",
        }
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Owed(value)
            | Self::Item(value)
            | Self::Proposal(value)
            | Self::Assessment(value) => value.fmt(f),
            Self::Generated(value) | Self::Binding(value) | Self::AfterRemoval(value) => write!(
                f,
                "{} {}: {} ({})",
                value.kind, value.file, value.message, value.fix
            ),
            Self::Rewrite { file, reason } => {
                write!(f, "cannot form generated replacement for {file}: {reason}")
            }
            Self::Changed { what } => write!(
                f,
                "static completion inputs changed: {what}; nothing was written"
            ),
        }
    }
}

/// The result of static change completion. It carries no execution verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Completion {
    /// Owned documentation was rewritten and the completed record removed.
    Completed {
        /// The removed change record.
        removed: String,
        /// The generated documentation paths rewritten.
        rewritten: Vec<String>,
    },
    /// Publication was refused and no owned file was written.
    Refused {
        /// Every static failure.
        failures: Vec<Failure>,
    },
}

impl Completion {
    /// Whether the record was removed.
    #[must_use]
    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }

    /// All refusal failures, or an empty slice after completion.
    #[must_use]
    pub fn failures(&self) -> &[Failure] {
        match self {
            Self::Completed { .. } => &[],
            Self::Refused { failures } => failures,
        }
    }

    /// Failures of the requested condition.
    pub fn of_condition(&self, condition: Condition) -> impl Iterator<Item = &Failure> {
        self.failures()
            .iter()
            .filter(move |failure| failure.condition() == condition)
    }

    /// Reconciliation obligations among the failures.
    pub fn obligations(&self) -> impl Iterator<Item = &Obligation> {
        self.failures().iter().filter_map(|failure| match failure {
            Failure::Owed(value) => Some(value),
            _ => None,
        })
    }

    /// A concise static publication outcome.
    #[must_use]
    pub fn headline(&self) -> String {
        match self {
            Self::Completed { removed, .. } => format!("completed: removed {removed}"),
            Self::Refused { failures } => format!("refused: {} static failure(s)", failures.len()),
        }
    }
}

impl fmt::Display for Completion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.headline())?;
        for failure in self.failures() {
            writeln!(
                f,
                "{}: {}",
                failure.condition().name(),
                crate::escape::shown(&failure.to_string())
            )?;
        }
        Ok(())
    }
}
