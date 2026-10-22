// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Typed refusals of the closed report profile.
use std::fmt;

/// Why text or a value is not a canonical census report.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReportError {
    /// A shared canonical ground-profile refusal.
    Ground(raf_rdf::Error),
    /// Unsupported report layout version.
    Form {
        /// Declared layout.
        declared: u32,
        /// Implemented layout.
        implemented: u32,
    },
    /// The other report kind was supplied.
    Kind {
        /// Requested class.
        expected: String,
        /// Actual class.
        found: String,
    },
    /// A class outside this report kind.
    Class(String),
    /// A predicate outside a subject's closed shape.
    UnknownPredicate(String),
    /// A required property is absent.
    Missing(String),
    /// A property exceeds its cardinality or repeats an object.
    Repeated(String),
    /// A mistyped or out-of-range value.
    Value(String),
    /// A reference names no node.
    Dangling(String),
    /// A node is not referenced by the report.
    Orphan(String),
    /// A node name disagrees with its properties.
    Name(String),
    /// Related facts disagree.
    Inconsistent(String),
    /// Two jobs would mint the same node.
    DuplicateJob(String),
}
impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ground(error) => error.fmt(f),
            Self::Form {
                declared,
                implemented,
            } => write!(f, "reportForm {declared}, implemented {implemented}"),
            Self::Kind { expected, found } => write!(
                f,
                "expected {}, found {}",
                expected.escape_debug(),
                found.escape_debug()
            ),
            Self::Class(value) => write!(f, "unknown class {}", value.escape_debug()),
            Self::UnknownPredicate(value) => {
                write!(f, "unknown predicate {}", value.escape_debug())
            }
            Self::Missing(value) => write!(f, "missing {}", value.escape_debug()),
            Self::Repeated(value) => write!(f, "repeated {}", value.escape_debug()),
            Self::Value(value) => write!(f, "invalid value {}", value.escape_debug()),
            Self::Dangling(value) => write!(f, "dangling reference {}", value.escape_debug()),
            Self::Orphan(value) => write!(f, "orphan node {}", value.escape_debug()),
            Self::Name(value) => write!(
                f,
                "node name disagrees with properties: {}",
                value.escape_debug()
            ),
            Self::Inconsistent(value) => write!(f, "inconsistent {}", value.escape_debug()),
            Self::DuplicateJob(value) => write!(f, "duplicate job id {}", value.escape_debug()),
        }
    }
}
impl std::error::Error for ReportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ground(error) => Some(error),
            _ => None,
        }
    }
}
impl From<ReportError> for crate::Error {
    fn from(error: ReportError) -> Self {
        Self::Report(error)
    }
}
