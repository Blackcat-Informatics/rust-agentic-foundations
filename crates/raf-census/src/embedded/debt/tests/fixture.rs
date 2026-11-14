// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Fixture snapshots and admitted rendering through the production resource path.

use super::super::{Failure, Record};
use crate::embedded::{
    OutputFailure, RefusalKind,
    resource::{Meter, Text},
};
use raf_rdf::{AdmittedGraph, Format};

pub(super) fn text(record: &Record<'_>, format: Format) -> String {
    record.to_text(format, |outcome| outcome.unwrap().to_string())
}

pub(super) fn read<'m>(text: &str, format: Format, meter: &'m Meter) -> Record<'m> {
    Record::from_text(text, format, meter, |outcome| outcome.unwrap())
}

pub(super) fn resource(cause: &Failure<'_>) -> RefusalKind {
    resource_output(&OutputFailure::failure(cause))
}

fn resource_output(cause: &OutputFailure<'_>) -> RefusalKind {
    cause
        .refusal()
        .unwrap_or_else(|| panic!("expected an original resource refusal: {cause}"))
}

pub(super) fn render<'m>(
    meter: &'m Meter,
    lexical: &[&str],
    format: Format,
) -> Result<Text<'m>, RefusalKind> {
    let graph = AdmittedGraph::build_captured(meter.native(), meter.capture(), |build| {
        for value in lexical {
            build.insert_literal(
                purrdf_core::RecordKind::Ordinary,
                "",
                "urn:predicate",
                value,
                raf_rdf::vocab::STRING,
            )?;
        }
        Ok(())
    });
    match graph {
        Ok(graph) => Text::render_admitted(meter, &graph, format, |outcome| {
            outcome.map_err(|cause| resource_output(&cause))
        }),
        Err(cause) => Err(resource_output(&OutputFailure::build(&cause))),
    }
}
