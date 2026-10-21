// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The one error type of the census.

use std::error::Error as StdError;
use std::fmt;

use raf_source::LoadError;

/// Why a census run failed. Every variant is final: the census never falls
/// back to a partial result.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A source slice does not lex or the source index is inconsistent.
    Source {
        /// The file, relative to the workspace root.
        file: String,
        /// The 1-based line of the failure.
        line: usize,
        /// What went wrong.
        reason: String,
    },
    /// A [`CensusOptions`](crate::CensusOptions) value is refused.
    Options(String),
    /// Two units with different digests have one fingerprint.
    Collision {
        /// The shared fingerprint.
        fingerprint: String,
        /// The symbol of the first unit.
        first: String,
        /// The symbol of the second unit.
        second: String,
    },
    /// A declared normal-form version is not the implemented one.
    Version {
        /// The version that was declared.
        declared: u32,
        /// The version this crate implements.
        implemented: u32,
    },
    /// The workspace could not be loaded.
    Load(LoadError),
    /// A report could not be rendered, or a text is not a report: unknown
    /// predicates, missing properties and wrong types are refused.
    Report(crate::ReportError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source { file, line, reason } => write!(f, "{file}:{line}: {reason}"),
            Self::Options(reason) => write!(f, "invalid census options: {reason}"),
            Self::Collision {
                fingerprint,
                first,
                second,
            } => write!(
                f,
                "fingerprint {fingerprint} names two different forms, `{first}` and `{second}`"
            ),
            Self::Version {
                declared,
                implemented,
            } => write!(
                f,
                "normal_form {declared} is declared but this census implements normal_form \
                 {implemented}; fingerprints are comparable only inside one version, so run \
                 `cargo census migrate` to move the ledger to normal_form {implemented}"
            ),
            Self::Load(error) => write!(f, "the workspace did not load: {error}"),
            Self::Report(reason) => write!(f, "invalid census report: {reason}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Load(error) => Some(error),
            Self::Report(error) => Some(error),
            _ => None,
        }
    }
}

impl From<LoadError> for Error {
    fn from(error: LoadError) -> Self {
        Self::Load(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_variant_renders_its_facts() {
        let source = Error::Source {
            file: "src/a.rs".into(),
            line: 7,
            reason: "unterminated string".into(),
        };
        assert_eq!(source.to_string(), "src/a.rs:7: unterminated string");
        assert!(
            Error::Options("min_tokens is 0".into())
                .to_string()
                .contains("min_tokens is 0")
        );
        let collision = Error::Collision {
            fingerprint: "body:0123456789abcdef".into(),
            first: "a::f".into(),
            second: "b::g".into(),
        }
        .to_string();
        assert!(
            collision.contains("a::f") && collision.contains("b::g"),
            "{collision}"
        );
        let report = Error::Report(crate::ReportError::Form {
            declared: 2,
            implemented: 1,
        });
        assert!(report.to_string().contains("reportForm 2, implemented 1"));
        assert!(report.source().is_some());
    }

    #[test]
    fn an_options_error_has_no_nested_source() {
        assert!(Error::Options("x".into()).source().is_none());
    }
}
