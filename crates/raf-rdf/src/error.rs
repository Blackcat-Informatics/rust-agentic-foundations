// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Refusals of the closed report profile.

use std::fmt;

/// An invalid value, graph or canonical spelling.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Native allocation, layout or original admission refusal.
    Storage(purrdf_lex::allocation::StorageError),
    /// A native term could not be resolved in its original dataset.
    Lookup(purrdf_core::TermLookupError),
    /// An immutable validated graph violated a required rendering invariant.
    Invariant(&'static str),
    /// Invalid term or graph, retaining the raw reason as data.
    Value(String),
    /// A repeated statement is not a canonical graph encoding.
    Repeated,
    /// A token outside the grammar at its one-based position.
    Syntax {
        /// Line of the refused token.
        line: usize,
        /// Character column of the refused token.
        column: usize,
        /// Description of the refusal.
        reason: String,
    },
    /// The content digest does not match the declared snapshot base.
    Digest {
        /// Declared snapshot BASE.
        declared: String,
        /// Snapshot BASE computed from the validated graph.
        computed: String,
    },
    /// A spelling differs from the canonical bytes.
    NotCanonical {
        /// First differing line, comparing exact bytes including newlines.
        line: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "report storage: {error}"),
            Self::Lookup(error) => write!(f, "report term lookup: {error}"),
            Self::Invariant(reason) => write!(
                f,
                "report invariant: {reason}; regenerate the graph from validated input"
            ),
            Self::Value(value) => write!(f, "invalid report value: {}", value.escape_debug()),
            Self::Repeated => f.write_str("a report triple is repeated"),
            Self::Syntax {
                line,
                column,
                reason,
            } => write!(
                f,
                "report syntax at {line}:{column}: {}",
                reason.escape_debug()
            ),
            Self::Digest { declared, computed } => write!(
                f,
                "report snapshot digest disagrees: declared {}, computed {}; regenerate the canonical report from verified input",
                declared.escape_debug(),
                computed.escape_debug()
            ),
            Self::NotCanonical { line } => write!(
                f,
                "report spelling differs from canonical bytes at line {line}"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Lookup(error) => Some(error),
            _ => None,
        }
    }
}

impl From<purrdf_lex::allocation::StorageError> for Error {
    fn from(error: purrdf_lex::allocation::StorageError) -> Self {
        Self::Storage(error)
    }
}
