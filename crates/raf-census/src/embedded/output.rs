// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Scoped presentation of unchanged diagnostics under their original owners.

use super::{
    RefusalKind,
    resource::{Grant, Meter, Native, Refusal, Work},
};
use crate::ReportError;
use purrdf_lex::allocation::{Memory, StorageError};
use std::fmt;

/// The layer that refused an embedded output operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFailureKind {
    /// A native construction, lookup, parsing or codec operation refused.
    Native,
    /// The shared closed RDF profile refused.
    Profile,
    /// Embedded report or debt facts violate their domain contract.
    Domain,
    /// Static source analysis, exact debt comparison or publication refused.
    Analysis,
}

/// Machine-readable exact-allowance refusal borrowed from its original owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebtRefusal<'a> {
    /// The allowance record belongs to another analysis account.
    AccountMismatch,
    /// Exact allowance arithmetic exceeded its representation.
    Overflow,
    /// Fresh verified pair counts exceed a selected allowance.
    Growth {
        /// Original verified lexical digest.
        digest: &'a [u8; 32],
        /// Diagnostic hash of the complete left descriptor.
        left: &'a [u8; 32],
        /// Diagnostic hash of the complete right descriptor.
        right: &'a [u8; 32],
        /// Selected recorded allowance.
        recorded: usize,
        /// Fresh verified physical pair count.
        present: usize,
    },
}

/// An unchanged diagnostic borrowed only while its original owner is alive.
/// Operations supply this to a scoped observer. Formatting writes directly to
/// the caller's destination; no detached diagnostic string is constructed.
pub struct OutputFailure<'a> {
    diagnostic: &'a dyn fmt::Display,
    kind: OutputFailureKind,
    refusal: Option<RefusalKind>,
    debt: Option<DebtRefusal<'a>>,
}

impl OutputFailure<'_> {
    /// The original diagnostic's refusing layer.
    #[must_use]
    pub const fn kind(&self) -> OutputFailureKind {
        self.kind
    }

    /// The exact original resource refusal, when one occurred.
    /// Absence does not imply a resource failure or authorize a replacement.
    #[must_use]
    pub const fn refusal(&self) -> Option<RefusalKind> {
        self.refusal
    }
}

impl<'a> OutputFailure<'a> {
    /// The exact account, arithmetic or growth refusal, when present.
    /// Borrowed hashes remain valid only while the original diagnostic lives.
    #[must_use]
    pub const fn debt_refusal(&self) -> Option<DebtRefusal<'a>> {
        self.debt
    }

    pub(super) fn publication(error: &'a crate::Error) -> Self {
        Self {
            diagnostic: error,
            kind: OutputFailureKind::Analysis,
            refusal: None,
            debt: None,
        }
    }
    pub(super) fn rdf(cause: &'a raf_rdf::Error, captured: Option<Refusal>) -> Self {
        Self {
            diagnostic: cause,
            kind: rdf_kind(cause),
            refusal: captured.and_then(|cause| resource_kind(cause.into())),
            debt: None,
        }
    }

    pub(super) fn failure(error: &'a Failure<'_>) -> Self {
        let (kind, refusal) = match error {
            Failure::Resource(cause) => (OutputFailureKind::Domain, resource_kind((*cause).into())),
            Failure::Domain(_) => (OutputFailureKind::Domain, None),
            Failure::AccountMismatch | Failure::Overflow | Failure::Growth { .. } => {
                (OutputFailureKind::Analysis, None)
            }
            Failure::Percent(_) => (OutputFailureKind::Native, None),
            Failure::Query(cause) => (
                rdf_kind(&cause.cause),
                cause
                    .grant
                    .captured()
                    .and_then(|cause| resource_kind(cause.into())),
            ),
        };
        Self {
            diagnostic: error,
            kind,
            refusal,
            debt: match error {
                Failure::AccountMismatch => Some(DebtRefusal::AccountMismatch),
                Failure::Overflow => Some(DebtRefusal::Overflow),
                Failure::Growth {
                    digest,
                    left,
                    right,
                    recorded,
                    present,
                } => Some(DebtRefusal::Growth {
                    digest,
                    left,
                    right,
                    recorded: *recorded,
                    present: *present,
                }),
                _ => None,
            },
        }
    }

    pub(super) fn build(error: &'a raf_rdf::BuildFailure<Native<'_>, Refusal>) -> Self {
        Self {
            diagnostic: error,
            kind: ground_kind(error.cause()),
            refusal: error
                .captured()
                .and_then(|cause| resource_kind(cause.into())),
            debt: None,
        }
    }

    pub(super) fn import(
        error: &'a raf_rdf::ImportFailure<'_, '_, Native<'_>, Work<'_>, Refusal>,
    ) -> Self {
        let kind = match error {
            raf_rdf::ImportFailure::Parse { .. } => OutputFailureKind::Native,
            raf_rdf::ImportFailure::Build(error) => ground_kind(error.cause()),
            raf_rdf::ImportFailure::Profile { cause, .. } => ground_kind(cause),
        };
        let refusal = match error {
            raf_rdf::ImportFailure::Parse { cause, .. } => match cause.cause() {
                purrdf_lex::control::ControlledError::Control(refusal) => Some(*refusal),
                purrdf_lex::control::ControlledError::Native(_) => error.captured(),
            },
            _ => error.captured(),
        };
        Self {
            diagnostic: error,
            kind,
            refusal: refusal.and_then(|cause| resource_kind(cause.into())),
            debt: None,
        }
    }
}

fn resource_kind(cause: RefusalKind) -> Option<RefusalKind> {
    match cause {
        RefusalKind::Count { .. }
        | RefusalKind::Work { .. }
        | RefusalKind::Memory { .. }
        | RefusalKind::SyntaxExcess { .. }
        | RefusalKind::Storage(_) => Some(cause),
        RefusalKind::Invariant(_) | RefusalKind::Unsupported(_) => None,
    }
}

pub(super) struct DomainFailure<'m> {
    cause: ReportError,
    _grant: Grant<'m>,
}

pub(super) struct QueryFailure<'m> {
    cause: raf_rdf::Error,
    grant: Grant<'m>,
}

/// Every owned native/domain payload precedes its original account.
pub(super) enum Failure<'m> {
    Resource(Refusal),
    Domain(DomainFailure<'m>),
    Query(QueryFailure<'m>),
    Overflow,
    AccountMismatch,
    Percent(purrdf_lex::percent::PercentError),
    Growth {
        digest: [u8; 32],
        left: [u8; 32],
        right: [u8; 32],
        recorded: usize,
        present: usize,
    },
}

pub(super) enum BodyFailure<'m> {
    Native(raf_rdf::ConstructionStopped),
    Caller(Failure<'m>),
}
impl From<raf_rdf::ConstructionStopped> for BodyFailure<'_> {
    fn from(cause: raf_rdf::ConstructionStopped) -> Self {
        Self::Native(cause)
    }
}
impl<'m> From<Failure<'m>> for BodyFailure<'m> {
    fn from(cause: Failure<'m>) -> Self {
        Self::Caller(cause)
    }
}
impl From<Refusal> for BodyFailure<'_> {
    fn from(cause: Refusal) -> Self {
        Self::Caller(cause.into())
    }
}

pub(super) enum QueryCause {
    Native(raf_rdf::Error),
    Resource(Refusal),
}
impl From<raf_rdf::Error> for QueryCause {
    fn from(cause: raf_rdf::Error) -> Self {
        Self::Native(cause)
    }
}
impl From<StorageError> for QueryCause {
    fn from(cause: StorageError) -> Self {
        Self::Native(cause.into())
    }
}
impl From<Refusal> for QueryCause {
    fn from(cause: Refusal) -> Self {
        Self::Resource(cause)
    }
}

impl<'m> Failure<'m> {
    pub(super) fn report(
        meter: &'m Meter,
        variant: fn(String) -> ReportError,
        value: &impl fmt::Display,
    ) -> Self {
        let mut grant = Grant::new(meter);
        match grant.memory(|memory| Ok(memory.format_with_progress(value)?)) {
            Ok(text) => Self::Domain(DomainFailure {
                cause: variant(text),
                _grant: grant,
            }),
            Err(cause) => Self::Resource(cause),
        }
    }

    pub(super) fn query<T: Copy>(
        meter: &'m Meter,
        operation: impl FnOnce(&mut Memory<'_, Native<'m>>) -> Result<T, QueryCause>,
    ) -> Result<T, Self> {
        let mut grant = Grant::new(meter);
        let result = grant.raw_memory(operation);
        result.map_err(|cause| match cause {
            QueryCause::Native(cause) => Self::Query(QueryFailure { cause, grant }),
            QueryCause::Resource(cause) => Self::Resource(cause),
        })
    }
}

impl From<Refusal> for Failure<'_> {
    fn from(cause: Refusal) -> Self {
        Self::Resource(cause)
    }
}
impl From<StorageError> for Failure<'_> {
    fn from(cause: StorageError) -> Self {
        Self::Resource(cause.into())
    }
}
impl fmt::Display for Failure<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resource(cause) => cause.fmt(output),
            Self::Domain(cause) => cause.cause.fmt(output),
            Self::Query(cause) => cause.cause.fmt(output),
            Self::Overflow => output.write_str("embedded debt pair count overflow"),
            Self::AccountMismatch => output.write_str("embedded debt belongs to another run; import the record into the selected analysis account before comparing it"),
            Self::Percent(cause) => cause.fmt(output),
            Self::Growth { digest, left, right, recorded, present } => write!(output,
                "embedded debt grew for {} between descriptor hashes {} and {}: recorded {recorded}, present {present}; remove the new duplication; an existing record cannot adopt growth",
                purrdf_hash::hex::Lower(digest), purrdf_hash::hex::Lower(left), purrdf_hash::hex::Lower(right)),
        }
    }
}
impl fmt::Debug for Failure<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, output)
    }
}

fn ground_kind(cause: &raf_rdf::BuildCause) -> OutputFailureKind {
    match cause {
        raf_rdf::BuildCause::Profile(cause) => rdf_kind(cause),
        _ => OutputFailureKind::Native,
    }
}

fn rdf_kind(cause: &raf_rdf::Error) -> OutputFailureKind {
    match cause {
        raf_rdf::Error::Storage(_) | raf_rdf::Error::Lookup(_) | raf_rdf::Error::Invariant(_) => {
            OutputFailureKind::Native
        }
        _ => OutputFailureKind::Profile,
    }
}

impl fmt::Display for OutputFailure<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.diagnostic.fmt(output)
    }
}
impl fmt::Debug for OutputFailure<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, output)
    }
}
