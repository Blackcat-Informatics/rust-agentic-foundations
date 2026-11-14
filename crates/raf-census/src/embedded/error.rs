// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Typed native failures at the public static-analysis boundary.

use super::{
    analysis::SourceFailure,
    resource::{Refusal, TerminalAccount},
};
use purrdf_lex::allocation::StorageError;
use std::fmt;

/// A resource, syntax-profile or internal consistency refusal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RefusalKind {
    /// A named source or discovery count would exceed its limit.
    Count {
        /// Counted resource.
        resource: &'static str,
        /// Selected ceiling.
        limit: usize,
    },
    /// Cumulative work exhausted its selected ceiling.
    Work {
        /// Selected ceiling.
        limit: usize,
    },
    /// Live storage exhausted its selected ceiling.
    Memory {
        /// Selected ceiling.
        limit: usize,
    },
    /// Native allocation or layout refusal.
    Storage(StorageError),
    /// Required source context violates an invariant.
    Invariant(&'static str),
    /// Required syntax is outside the implemented profile.
    Unsupported(&'static str),
}

impl From<Refusal> for RefusalKind {
    fn from(value: Refusal) -> Self {
        match value {
            Refusal::Count { resource, limit } => Self::Count { resource, limit },
            Refusal::Work { limit } => Self::Work { limit },
            Refusal::Memory { limit } => Self::Memory { limit },
            Refusal::Storage(error) => Self::Storage(error),
            Refusal::Invariant(reason) => Self::Invariant(reason),
            Refusal::Unsupported(reason) => Self::Unsupported(reason),
        }
    }
}

/// A native inspection diagnostic retaining its original resource account.
///
/// The cause is dropped before the immutable account. Creating this value
/// consumes the analysis account after its worker and retained buffers finish.
#[derive(Debug)]
pub struct InspectionError {
    cause: raf_source::LoadError,
    _account: Box<TerminalAccount>,
}

impl InspectionError {
    pub(super) const fn new(cause: raf_source::LoadError, account: Box<TerminalAccount>) -> Self {
        Self {
            cause,
            _account: account,
        }
    }

    /// The original native Source cause, including its located diagnostic.
    #[must_use]
    pub const fn cause(&self) -> &raf_source::LoadError {
        &self.cause
    }
}

impl fmt::Display for InspectionError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cause.fmt(output)
    }
}

impl std::error::Error for InspectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

/// A final embedded-analysis failure; no partial run is returned.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Source workspace loading failed before an analysis account existed.
    Source(raf_source::LoadError),
    /// Native inspection failed, retaining its original account with the cause.
    Inspection(InspectionError),
    /// Located static collection refusal.
    Collection {
        /// Actual Source path, retaining its authority.
        file: raf_source::RelPath,
        /// Original byte position.
        byte: usize,
        /// Original refusal category.
        cause: RefusalKind,
    },
    /// Static matching refused its original grant.
    Refused(RefusalKind),
    /// Native owned metadata publication failed, possibly after atomic publication.
    Publication(Box<crate::Error>),
}

impl From<Refusal> for Error {
    fn from(value: Refusal) -> Self {
        Self::Refused(value.into())
    }
}
impl From<crate::Error> for Error {
    fn from(value: crate::Error) -> Self {
        Self::Publication(Box::new(value))
    }
}
impl From<SourceFailure<'_>> for Error {
    fn from(value: SourceFailure<'_>) -> Self {
        match value {
            SourceFailure::Native { cause, .. } => Self::Inspection(cause),
            SourceFailure::Located { file, cause } => Self::Collection {
                file: file.clone(),
                byte: cause.byte,
                cause: cause.cause.into(),
            },
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(error) => error.fmt(f),
            Self::Inspection(error) => error.fmt(f),
            Self::Collection { file, byte, cause } => {
                write!(
                    f,
                    "{}: byte {byte}: {cause:?}",
                    file.as_str().escape_debug()
                )
            }
            Self::Refused(cause) => {
                write!(f, "embedded analysis refused: {cause:?}")?;
                if matches!(
                    cause,
                    RefusalKind::Count { .. }
                        | RefusalKind::Work { .. }
                        | RefusalKind::Memory { .. }
                ) {
                    f.write_str("; reduce input or increase the selected resource limit")?;
                }
                Ok(())
            }
            Self::Publication(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source(error) => Some(error),
            Self::Inspection(error) => Some(error.cause()),
            Self::Publication(error) => Some(error.as_ref()),
            Self::Refused(RefusalKind::Storage(error))
            | Self::Collection {
                cause: RefusalKind::Storage(error),
                ..
            } => Some(error),
            _ => None,
        }
    }
}
