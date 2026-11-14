// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Static embedded analysis, independent of the whole-body Census gate.

use crate::{
    cli::{EmbeddedCommand, EmbeddedDebtAction},
    error::Error,
};
use raf_census::embedded::{Comparison, DEBT_FILE, Options, Run};
use raf_source::{DiskTree, LoadOptions, RelPath, Tree as _, Workspace};
use std::{io::Write, path::Path};

pub(crate) fn run(
    root: &Path,
    command: &EmbeddedCommand,
    output: &mut dyn Write,
    diagnostics: &mut dyn Write,
) -> Result<u8, Error> {
    if let EmbeddedCommand::Debt { action } = command {
        return publish(root, action, output, diagnostics);
    }
    let tree = DiskTree::new(root).map_err(|source| raf_census::Error::Tree {
        file: root.display().to_string(),
        source,
    })?;
    let previous = if matches!(command, EmbeddedCommand::Check) {
        let path = RelPath::new(DEBT_FILE).expect("literal embedded debt path");
        Some(
            tree.read(&path, raf_census::MAX_DEBT_BYTES as u64)
                .map_err(|source| raf_census::Error::Tree {
                    file: DEBT_FILE.into(),
                    source,
                })?,
        )
    } else {
        None
    };
    let source = Workspace::load_with(&tree, &["."], &LoadOptions::default())?;
    let analysis = Run::analyze(&source, &Options::default())?;
    match command {
        EmbeddedCommand::Report { format, out } => {
            analysis.report(|result| match result {
                Err(cause) => refused(&cause, diagnostics),
                Ok(evidence) => evidence.to_text(format.native(), |result| match result {
                    Err(cause) => refused(&cause, diagnostics),
                    Ok(text) => {
                        crate::command::write_report(root, text.as_str(), out.as_deref(), output)?;
                        Ok(0)
                    }
                }),
            })
        }
        EmbeddedCommand::Check => {
            analysis.import_debt(
                previous.as_deref().expect("check read its required debt"),
                raf_rdf::Format::Turtle,
                |outcome| match outcome {
                    Err(cause) => refused(&cause, diagnostics),
                    Ok(debt) => analysis.compare_debt(&debt, |outcome| match outcome {
                        Err(cause) => refused(&cause, diagnostics),
                        Ok(Comparison::Unchanged) => Ok(0),
                        Ok(Comparison::Shrunk { removed }) => {
                            writeln!(diagnostics, "cargo-census: {DEBT_FILE}: {removed} excess pair allowances; run cargo census embedded debt shrink")
                                .map_err(|source| raf_census::Error::Io { file: "stderr".into(), source })?;
                            Ok(1)
                        }
                    }),
                },
            )
        }
        EmbeddedCommand::Debt { .. } => unreachable!("publication dispatched before analysis"),
    }
}

fn refused(
    cause: &raf_census::embedded::OutputFailure<'_>,
    diagnostics: &mut dyn Write,
) -> Result<u8, Error> {
    writeln!(diagnostics, "cargo-census: {cause}").map_err(|source| raf_census::Error::Io {
        file: "stderr".into(),
        source,
    })?;
    Ok(1)
}

#[cfg(target_os = "linux")]
fn publish(
    root: &Path,
    action: &EmbeddedDebtAction,
    output: &mut dyn Write,
    diagnostics: &mut dyn Write,
) -> Result<u8, Error> {
    use raf_census::embedded::{DebtAction, DebtWrite, write_debt};
    let action = match action {
        EmbeddedDebtAction::Create => DebtAction::Create,
        EmbeddedDebtAction::Shrink => DebtAction::Shrink,
    };
    write_debt(
        root,
        &LoadOptions::default(),
        &Options::default(),
        action,
        |outcome| {
            let receipt = match outcome {
                Ok(receipt) => receipt,
                Err(cause) => return refused(&cause, diagnostics),
            };
            match receipt {
                DebtWrite::Created => writeln!(output, "{DEBT_FILE}: created"),
                DebtWrite::Unchanged => writeln!(output, "{DEBT_FILE}: unchanged"),
                DebtWrite::Shrunk { removed } => {
                    writeln!(output, "{DEBT_FILE}: removed {removed} pair allowances")
                }
            }
            .map_err(|source| raf_census::Error::Io {
                file: "stdout".into(),
                source,
            })?;
            Ok(0)
        },
    )?
}

#[cfg(not(target_os = "linux"))]
fn publish(
    _root: &Path,
    _action: &EmbeddedDebtAction,
    _output: &mut dyn Write,
    _diagnostics: &mut dyn Write,
) -> Result<u8, Error> {
    Err(raf_census::Error::Options(
        "embedded debt publication requires the native Linux directory capability".into(),
    )
    .into())
}
