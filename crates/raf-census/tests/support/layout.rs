// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native prior-layout fixtures and complete canonical preservation assertions.

use raf_census::{
    Baseline, DebtRecord, Error, Index, Ledger, MigrationInput, ReportError, ReportKind,
    upgrade_report_form,
};
use raf_rdf::{Builder, Format, Graph, TermValue, Triple, vocab};

pub(crate) fn form3(text: &str, format: Format) -> String {
    let original = Graph::from_text(text, format).unwrap();
    let mut graph = Builder::from_graph(&original);
    let header = original
        .assertions()
        .find(|triple| {
            triple.subject == TermValue::iri("")
                && triple.predicate == TermValue::iri(format!("{}reportForm", vocab::REPORT))
        })
        .unwrap();
    assert!(graph.remove(header.clone()).unwrap());
    graph
        .insert(Triple {
            object: TermValue::integer(3),
            ..header
        })
        .unwrap();
    graph.finish().unwrap().to_text(format).unwrap()
}

pub(crate) fn assert_upgrade(kind: ReportKind, current: &str, format: Format) -> String {
    let old = form3(current, format);
    let input = MigrationInput {
        path: "prior-layout",
        text: &old,
    };
    let converted = upgrade_report_form(kind, input, format).unwrap();
    assert_eq!(
        converted, current,
        "all normalized facts, external IRIs and pin literals survive"
    );
    let refused = match (kind, format) {
        (ReportKind::Baseline, Format::Turtle) => Baseline::from_turtle(&old).unwrap_err(),
        (ReportKind::Baseline, Format::NTriples) => Baseline::from_ntriples(&old).unwrap_err(),
        (ReportKind::Index, Format::Turtle) => Index::from_turtle(&old).unwrap_err(),
        (ReportKind::Index, Format::NTriples) => Index::from_ntriples(&old).unwrap_err(),
        (ReportKind::Ledger, Format::Turtle) => Ledger::from_turtle(&old).unwrap_err(),
        (ReportKind::Ledger, Format::NTriples) => Ledger::from_ntriples(&old).unwrap_err(),
        (ReportKind::Debt, Format::Turtle) => DebtRecord::from_turtle(&old).unwrap_err(),
        (ReportKind::Debt, Format::NTriples) => DebtRecord::from_ntriples(&old).unwrap_err(),
    };
    assert!(matches!(
        refused,
        Error::Report(ReportError::Form {
            declared: 3,
            implemented: 4
        })
    ));
    assert!(
        upgrade_report_form(
            kind,
            MigrationInput {
                path: "current-layout",
                text: current
            },
            format
        )
        .is_err()
    );
    let original = Graph::from_text(&old, format).unwrap();
    let mut unknown = Builder::from_graph(&original);
    unknown
        .insert(Triple {
            subject: TermValue::iri(""),
            predicate: TermValue::iri(format!("{}unrecognized", vocab::REPORT)),
            object: TermValue::simple_literal("must survive or refuse"),
        })
        .unwrap();
    let invalid = unknown.finish().unwrap().to_text(format).unwrap();
    assert!(
        upgrade_report_form(
            kind,
            MigrationInput {
                path: "unknown-field",
                text: &invalid
            },
            format
        )
        .is_err()
    );
    let other = if kind == ReportKind::Baseline {
        ReportKind::Index
    } else {
        ReportKind::Baseline
    };
    assert!(upgrade_report_form(other, input, format).is_err());
    converted
}
