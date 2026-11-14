// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Explicit native Linux adoption/shrink publication from fresh actual source.

use super::{Comparison, DEBT_FILE, Error, Options, OutputFailure, Run};
use crate::publication::{Kind, Publication, PublicationMode};
use raf_source::{DiskTree, LoadOptions, Workspace};
use std::path::Path;

/// Explicit operation on the independent embedded allowance record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebtAction {
    /// Adopt fresh exact allowances only when the destination is absent.
    Create,
    /// Require an existing record and publish only shrinkage, never growth.
    Shrink,
}

/// Result of a completed owned metadata publication operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebtWrite {
    /// Initial allowances were atomically created without replacing a destination.
    Created,
    /// Fresh counts already equalled the existing record; no rewrite was needed.
    Unchanged,
    /// Excess allowances were removed after the entire growth comparison passed.
    Shrunk {
        /// Number of physical pair allowances removed.
        removed: usize,
    },
}

/// Analyze this explicit workspace and publish its independent embedded debt.
///
/// The original directory and its exclusive nonblocking lock cover input
/// reading, fresh Source loading, verification, comparison and publication.
/// Only the fixed owned debt and temporary destinations are written. No supplied
/// evidence or previously constructed Run can authorize allowances here.
/// Raw metadata has its byte ceiling. RDF operations retain the original Run's
/// grants through scoped observation; Source native admission remains incomplete.
/// Advisory locking coordinates cooperating writers, not an immutable source
/// snapshot or authentication of arbitrary concurrent directory mutations.
/// The filesystem must support locking and synchronizing the directory; missing
/// support fails without a weaker protocol. Older file-lock writers do not
/// coordinate with this directory-lock protocol.
///
/// # Errors
/// Refuses existing initial destinations, absent or malformed shrink input,
/// growth, analysis exhaustion, unexpected filesystem entries and I/O failures.
/// A publication error retains its native cause. A nested `Published` failure
/// explicitly identifies a record already committed before finalization failed.
/// Failures before a Run exists return through the outer result. Once the Run
/// exists, success or the unchanged scoped failure is observed before it drops.
pub fn write_debt<R>(
    root: &Path,
    source_options: &LoadOptions,
    options: &Options,
    action: DebtAction,
    observe: impl for<'f> FnOnce(Result<DebtWrite, OutputFailure<'f>>) -> R,
) -> Result<R, Error> {
    let tree = DiskTree::new(root).map_err(|source| crate::Error::Tree {
        file: root.display().to_string(),
        source,
    })?;
    let publication = Publication::acquire(&tree, Kind::Embedded)?;
    let previous = match action {
        DebtAction::Create => {
            if publication.exists()? {
                return Err(crate::Error::Options(format!(
                    "{DEBT_FILE}: already exists; initial adoption cannot replace it"
                ))
                .into());
            }
            None
        }
        DebtAction::Shrink => Some(publication.read_existing()?.ok_or_else(|| {
            crate::Error::Options(format!(
                "{DEBT_FILE}: missing; initial adoption requires explicit create"
            ))
        })?),
    };
    let source = Workspace::load_with(&tree, &["."], source_options).map_err(Error::Source)?;
    let analysis = Run::analyze(&source, options)?;
    let Some(previous) = previous else {
        return Ok(analysis.adopt_debt(|outcome| match outcome {
            Err(cause) => observe(Err(cause)),
            Ok(debt) => debt.to_text(raf_rdf::Format::Turtle, |outcome| match outcome {
                Err(cause) => observe(Err(cause)),
                Ok(text) => match publication.publish(text.as_str(), PublicationMode::Create) {
                    Ok(()) => observe(Ok(DebtWrite::Created)),
                    Err(cause) => observe(Err(OutputFailure::publication(&cause))),
                },
            }),
        }));
    };
    Ok(analysis.import_debt(
        &previous,
        raf_rdf::Format::Turtle,
        |outcome| match outcome {
            Err(cause) => observe(Err(cause)),
            Ok(debt) => analysis.compare_debt(&debt, |outcome| match outcome {
                Err(cause) => observe(Err(cause)),
                Ok(Comparison::Unchanged) => observe(Ok(DebtWrite::Unchanged)),
                Ok(Comparison::Shrunk { removed }) => {
                    analysis.shrink_debt(&debt, |outcome| match outcome {
                        Err(cause) => observe(Err(cause)),
                        Ok(fresh) => {
                            fresh.to_text(raf_rdf::Format::Turtle, |outcome| match outcome {
                                Err(cause) => observe(Err(cause)),
                                Ok(text) => match publication
                                    .publish(text.as_str(), PublicationMode::Replace)
                                {
                                    Ok(()) => observe(Ok(DebtWrite::Shrunk { removed })),
                                    Err(cause) => observe(Err(OutputFailure::publication(&cause))),
                                },
                            })
                        }
                    })
                }
            }),
        },
    ))
}
