// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Explicit layout conversion without reconstructing retained policy assertions.

use raf_rdf::{Builder, Format, Graph, TermValue, Triple};

use super::{DebtRecord, Ledger, ReportError as E, shape};
use crate::{Error, MigrationInput, REPORT_FORM};

/// The stored Census carrier selected before applying its input bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportKind {
    /// Structural discovery output.
    Baseline,
    /// Complete units, groups and configured jobs.
    Index,
    /// Policy, including owned consumer extensions.
    Ledger,
    /// Counted group and job debt.
    Debt,
}

impl ReportKind {
    /// Maximum bytes admitted for both the original and converted carrier.
    #[must_use]
    pub const fn max_bytes(self) -> usize {
        match self {
            Self::Baseline | Self::Index => crate::MAX_INDEX_BYTES,
            Self::Ledger => super::MAX_LEDGER_BYTES,
            Self::Debt => super::MAX_DEBT_BYTES,
        }
    }

    const fn class(self) -> &'static str {
        match self {
            Self::Baseline => "Baseline",
            Self::Index => "Index",
            Self::Ledger => "Ledger",
            Self::Debt => "DebtRecord",
        }
    }

    fn validate(self, graph: &Graph) -> Result<(), Error> {
        match self {
            Self::Baseline => {
                shape::baseline(graph)?;
            }
            Self::Index => {
                shape::index(graph)?;
            }
            Self::Ledger => {
                // Layout conversion preserves historical normal forms. Ordinary
                // Ledger readers additionally require the executing normal form.
                shape::nf(graph, "Ledger")?;
                Ledger::from_graph(graph.clone())?;
            }
            Self::Debt => {
                DebtRecord::from_graph(graph)?;
            }
        }
        Ok(())
    }

    fn check_size(self, text: &str) -> Result<(), Error> {
        if text.len() > self.max_bytes() {
            return Err(E::Value(format!(
                "{} is {} bytes; maximum is {}",
                self.class(),
                text.len(),
                self.max_bytes()
            ))
            .into());
        }
        Ok(())
    }
}

/// Convert a canonical form-3 carrier to the current layout, in the same syntax.
///
/// Every assertion other than `reportForm` survives. Canonical owned snapshot
/// IRIs rebase; external IRIs, normal forms, fingerprints and pin literals do not
/// change. Conversion performs structural validation, never policy execution.
///
/// # Errors
/// Refuses other kinds or source layouts, noncanonical or structurally invalid
/// graphs and either size bound. The named input is never written.
pub fn upgrade_report_form(
    kind: ReportKind,
    input: MigrationInput<'_>,
    format: Format,
) -> Result<String, Error> {
    upgrade(kind, input.text, format).map_err(|source| Error::Input {
        file: input.path.into(),
        source: Box::new(source),
    })
}

fn upgrade(kind: ReportKind, text: &str, format: Format) -> Result<String, Error> {
    kind.check_size(text)?;
    let graph = Graph::from_text(text, format).map_err(E::Ground)?;
    let root = graph
        .get("")
        .ok_or_else(|| E::Missing("report root".into()))?;
    let class = shape::class(&root)?;
    if class != kind.class() {
        return Err(E::Kind {
            expected: kind.class().into(),
            found: class.into(),
        }
        .into());
    }
    let declared = u32::try_from(shape::count(&root, "reportForm")?)
        .map_err(|_| E::Value("reportForm: u32 overflow".into()))?;
    if declared != 3 {
        return Err(E::Form {
            declared,
            implemented: 3,
        }
        .into());
    }
    let header = |form| Triple {
        subject: TermValue::iri(""),
        predicate: TermValue::iri(shape::property("reportForm")),
        object: shape::integer(form),
    };
    let mut updated = Builder::from_graph(&graph);
    if !updated.remove(header(3)).map_err(E::Ground)? {
        return Err(E::Missing("original form-3 header".into()).into());
    }
    updated
        .insert(header(u128::from(REPORT_FORM)))
        .map_err(E::Ground)?;
    let updated = updated.finish().map_err(E::Ground)?;
    kind.validate(&updated)?;
    let output = updated.to_text(format).map_err(E::Ground)?;
    kind.check_size(&output)?;
    Ok(output)
}
