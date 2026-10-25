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
    /// A declared normal-form version is not the implemented one.
    Version {
        /// The version that was declared.
        declared: u32,
        /// The version this crate implements.
        implemented: u32,
    },
    /// The workspace could not be loaded.
    Load(LoadError),
    /// The owned unit-collection thread could not start.
    CollectionSpawn(std::io::Error),
    /// Native source-context storage refused an allocation or layout.
    SourceIndexStorage {
        /// The original native storage failure.
        source: purrdf_lex::allocation::StorageError,
    },
    /// A report could not be rendered, or a text is not a report: unknown
    /// predicates, missing properties and wrong types are refused.
    Report(crate::ReportError),
    /// A foreign adapter declared an invalid predicate IRI.
    ImportPredicate {
        /// The adapter's declared predicate, without normalization or repair.
        predicate: String,
        /// The original native absolute-IRI refusal.
        source: purrdf_core::IriError,
    },
    /// Foreign TOML was refused by the supplied native parser during import.
    ImportToml(purrdf_toml::TomlError),
    /// A foreign baseline was refused by the native JSON grammar and limits.
    ImportJson(purrdf_lex::json::Error),
    /// A foreign baseline was refused by the native strict record decoder.
    ImportRecord(purrdf_lex::json::record::DecodeError),
    /// The native import work list could not reserve its spill storage.
    ImportStorage(std::collections::TryReserveError),
    /// Native foreign-value relocation storage refused allocation or layout.
    ImportIndexStorage(purrdf_lex::allocation::StorageError),
    /// A native pattern work list could not reserve its spill storage.
    PatternStorage {
        /// The pattern whose work could not be retained.
        pattern: String,
        /// The original allocator refusal.
        source: std::collections::TryReserveError,
    },
    /// Native ordered pattern storage refused an allocation or layout.
    PatternIndexStorage {
        /// The original native storage failure.
        source: purrdf_lex::allocation::StorageError,
    },
    /// A source-owned code-path query refused a pattern occurrence.
    CodePath {
        /// The matcher requiring the observation.
        pattern: String,
        /// The source file.
        file: String,
        /// The one-based source line.
        line: usize,
        /// The original reading-context or resolver refusal.
        source: Box<raf_source::CodeResolveError>,
    },
    /// Native provider storage refused an allocation or layout.
    ProviderStorage {
        /// The functionality job whose native storage failed.
        job: String,
        /// The original native allocation or layout refusal.
        source: purrdf_lex::allocation::StorageError,
    },
    /// A preferred-provider route preserves its original source-resolution failure.
    ProviderPath {
        /// The functionality job requiring the route.
        job: String,
        /// The source or ledger file requiring the route.
        file: String,
        /// The one-based source line.
        line: usize,
        /// The declared or observed path that could not be proved.
        path: String,
        /// The original typed reading-context or resolver refusal.
        source: Box<raf_source::CodeResolveError>,
    },
    /// A bounded tree operation failed at the named path.
    Tree {
        /// The path that could not be read.
        file: String,
        /// The original source-tree refusal.
        source: raf_source::TreeError,
    },
    /// A filesystem operation failed at the named path.
    Io {
        /// The path of the failed operation.
        file: String,
        /// The original operating-system error.
        source: std::io::Error,
    },
    /// Findings prevent the gate from passing or recording debt.
    Findings(Vec<String>),
    /// A ledger needs repository-registered executable code.
    RegistrationRequired {
        /// Job requiring the component.
        job: String,
        /// The missing rule or extension and the required registration.
        component: String,
    },
    /// A parsed input failed, retaining its path and original typed cause.
    Input {
        /// The file or fixture whose contents were refused.
        file: String,
        /// The original census or native-adapter failure.
        source: Box<Error>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source { file, line, reason } => write!(f, "{file}:{line}: {reason}"),
            Self::Options(reason) => write!(f, "invalid census options: {reason}"),
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
            Self::CollectionSpawn(error) => write!(
                f,
                "the census unit-collection thread could not start: {}",
                error.to_string().escape_debug()
            ),
            Self::SourceIndexStorage { source } => {
                write!(
                    f,
                    "source contexts: cannot reserve ownership index: {source}"
                )
            }
            Self::Report(reason) => write!(f, "invalid census report: {reason}"),
            Self::ImportPredicate { predicate, source } => {
                write!(f, "foreign adapter predicate IRI {predicate:?}: {source}")
            }
            Self::ImportToml(error) => write!(f, "invalid foreign TOML ledger: {error}"),
            Self::ImportJson(error) => write!(f, "invalid foreign JSON baseline: {error}"),
            Self::ImportRecord(error) => write!(f, "invalid foreign baseline record: {error}"),
            Self::ImportStorage(error) => {
                write!(f, "cannot reserve foreign import frontier: {error}")
            }
            Self::ImportIndexStorage(error) => {
                write!(f, "cannot reserve foreign value index: {error}")
            }
            Self::PatternStorage { pattern, source } => {
                write!(
                    f,
                    "pattern {pattern:?}: cannot reserve matching frontier: {source}"
                )
            }
            Self::PatternIndexStorage { source } => {
                write!(f, "pattern paths: cannot reserve ownership index: {source}")
            }
            Self::CodePath {
                pattern,
                file,
                line,
                source,
            } => write!(f, "{file}:{line}: pattern {pattern:?}: {source}"),
            Self::ProviderStorage { job, source } => {
                write!(f, "job {job:?}: cannot reserve provider storage: {source}")
            }
            Self::ProviderPath {
                job,
                file,
                line,
                path,
                source,
            } => {
                write!(
                    f,
                    "{file}:{line}: job {job:?}, provider path {path:?}: {source}"
                )
            }
            Self::Tree { file, source } => write!(f, "{}: {source}", file.escape_debug()),
            Self::Io { file, source } => write!(f, "{}: {source}", file.escape_debug()),
            Self::Input { file, source } => write!(f, "{}: {source}", file.escape_debug()),
            Self::RegistrationRequired { job, component } => write!(
                f,
                "job {job:?}: {component}; use the repository's registered Gate and Gate::write_debt"
            ),
            Self::Findings(findings) => {
                for finding in findings {
                    writeln!(f, "{finding}")?;
                }
                Ok(())
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Load(error) => Some(error),
            Self::CollectionSpawn(error) => Some(error),
            Self::Report(error) => Some(error),
            Self::ImportToml(error) => Some(error),
            Self::ImportPredicate { source, .. } => Some(source),
            Self::ImportJson(error) => Some(error),
            Self::ImportRecord(error) => Some(error),
            Self::ImportStorage(error) => Some(error),
            Self::ImportIndexStorage(error) => Some(error),
            Self::PatternStorage { source, .. } => Some(source),
            Self::SourceIndexStorage { source }
            | Self::PatternIndexStorage { source }
            | Self::ProviderStorage { source, .. } => Some(source),
            Self::CodePath { source, .. } | Self::ProviderPath { source, .. } => {
                Some(source.as_ref())
            }
            Self::Tree { source, .. } => Some(source),
            Self::Io { source, .. } => Some(source),
            Self::Input { source, .. } => Some(source.as_ref()),
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
        let report = Error::Report(crate::ReportError::Form {
            declared: 2,
            implemented: 1,
        });
        assert!(report.to_string().contains("reportForm 2, implemented 1"));
        assert!(report.source().is_some());
        let spawn = Error::CollectionSpawn(std::io::Error::other("refused\u{1b}"));
        assert!(
            spawn
                .to_string()
                .contains("unit-collection thread could not start")
        );
        assert!(!spawn.to_string().contains('\u{1b}'));
        assert!(spawn.to_string().contains("\\u{1b}"));
        assert!(spawn.source().is_some());
        let neighbor = Error::CollectionSpawn(std::io::Error::other("refused"));
        assert!(neighbor.to_string().ends_with("refused"));
        let index = Error::SourceIndexStorage {
            source: purrdf_lex::allocation::StorageError::AllocationFailed,
        };
        assert!(index.to_string().contains("source contexts"));
        let cause = index
            .source()
            .unwrap()
            .downcast_ref::<purrdf_lex::allocation::StorageError>()
            .unwrap();
        assert!(matches!(
            cause,
            purrdf_lex::allocation::StorageError::AllocationFailed
        ));
    }

    #[test]
    fn an_options_error_has_no_nested_source() {
        assert!(Error::Options("x".into()).source().is_none());
    }
}
