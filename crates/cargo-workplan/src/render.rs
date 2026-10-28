// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Human rendering and exit policy over typed results projected as RDF.

use crate::{exit::Exit, result::Result as CommandResult, result_graph};
use clap::ValueEnum;
use purrdf_rdf::{
    NativeRdfFormat, RdfDataset, SerializeGraph, SerializeOptions, StatementLayer,
    serialize_dataset_with,
};
use raf_workplan::Finding;
use raf_workplan::evidence::ClaimStatement;
use std::io::{self, Write};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum Format {
    Text,
    Turtle,
    Ntriples,
    Json,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Class {
    Finding,
    Outcome,
}
#[derive(Debug, Clone)]
pub(crate) struct Record {
    class: Class,
    result: CommandResult,
    text: String,
}
impl Record {
    pub(crate) fn outcome(result: CommandResult, text: impl Into<String>) -> Self {
        Self {
            class: Class::Outcome,
            result,
            text: text.into(),
        }
    }
    pub(crate) fn finding(result: CommandResult, text: impl Into<String>) -> Self {
        Self {
            class: Class::Finding,
            result,
            text: text.into(),
        }
    }
    pub(crate) fn of_finding(finding: &Finding) -> Self {
        Self::finding(CommandResult::Finding(finding.clone()), finding.to_string())
    }
}

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
enum Payload {
    Records(Vec<Record>),
    ClaimStatements(Vec<ClaimStatement>),
    Dataset {
        dataset: Arc<RdfDataset>,
        command: &'static str,
    },
}
impl Default for Payload {
    fn default() -> Self {
        Self::Records(Vec::new())
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Report {
    command: &'static str,
    payload: Payload,
    notes: Vec<String>,
    semantic: Option<Arc<RdfDataset>>,
}
impl Report {
    pub(crate) fn of(records: impl IntoIterator<Item = Record>) -> Self {
        Self {
            command: "workplan",
            payload: Payload::Records(records.into_iter().collect()),
            notes: Vec::new(),
            semantic: None,
        }
    }
    pub(crate) fn document(dataset: Arc<RdfDataset>) -> Self {
        Self {
            payload: Payload::Dataset {
                dataset,
                command: "docs export",
            },
            ..Self::default()
        }
    }
    pub(crate) fn claims_dataset(dataset: Arc<RdfDataset>) -> Self {
        Self {
            payload: Payload::Dataset {
                dataset,
                command: "evidence claims",
            },
            ..Self::default()
        }
    }
    pub(crate) fn claim_statements(claims: Vec<ClaimStatement>) -> Self {
        Self {
            payload: Payload::ClaimStatements(claims),
            ..Self::default()
        }
    }
    pub(crate) fn command(mut self, command: &'static str) -> Self {
        self.command = command;
        self
    }
    /// Human records and their authoritative native semantic selection.
    pub(crate) fn semantic_dataset(mut self, dataset: Arc<RdfDataset>) -> Self {
        self.semantic = Some(dataset);
        self
    }
    pub(crate) fn block(class: Class, text: String, result: CommandResult) -> Self {
        Self::of([Record {
            class,
            result,
            text,
        }])
    }
    pub(crate) fn note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }
    pub(crate) fn exit(&self) -> Exit {
        if matches!(&self.payload, Payload::Records(records) if records.iter().any(|r| r.class == Class::Finding))
        {
            Exit::Findings
        } else {
            Exit::Clean
        }
    }
    /// Write both sinks, propagating either sink's I/O failure.
    pub(crate) fn print(
        &self,
        format: Format,
        quiet: bool,
        out: &mut dyn Write,
        diagnostics: &mut dyn Write,
    ) -> io::Result<()> {
        if format != Format::Text
            && let Some(dataset) = &self.semantic
        {
            let semantic = Self {
                command: self.command,
                payload: Payload::Dataset {
                    dataset: Arc::clone(dataset),
                    command: self.command,
                },
                notes: self.notes.clone(),
                semantic: None,
            };
            return semantic.print(format, quiet, out, diagnostics);
        }
        let records = match &self.payload {
            Payload::Records(records) => records,
            Payload::ClaimStatements(claims) => {
                if format != Format::Text {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "evidence claims RDF requires its semantic dataset",
                    ));
                }
                if !quiet {
                    for claim in claims {
                        let at = claim.section.map_or_else(
                            || claim.capability.clone(),
                            |section| format!("{}#{}", claim.capability, section.slug()),
                        );
                        let mut text = format!("{}:{}: {at}", claim.path, claim.line);
                        for line in claim.statement.lines() {
                            text.push_str("\n    ");
                            text.push_str(line);
                        }
                        writeln!(out, "{}", raf_workplan::escape::shown_lines(&text))?;
                    }
                }
                out.flush()?;
                return self.print_notes(quiet, diagnostics);
            }
            Payload::Dataset { dataset, command } => {
                let format = match (format, quiet) {
                    (Format::Turtle, false) => NativeRdfFormat::Turtle,
                    (Format::Ntriples, false) => NativeRdfFormat::NTriples,
                    (Format::Json, false) => NativeRdfFormat::JsonLd,
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("{command} requires an RDF format and refuses quiet mode"),
                        ));
                    }
                };
                out.write_all(&rdf_bytes(dataset, format)?)?;
                out.flush()?;
                return self.print_notes(quiet, diagnostics);
            }
        };
        let shown: Vec<_> = records
            .iter()
            .filter(|r| !quiet || r.class == Class::Finding)
            .collect();
        match format {
            Format::Text => {
                for r in &shown {
                    if !r.text.is_empty() {
                        out.write_all(r.text.as_bytes())?;
                        if !r.text.ends_with('\n') {
                            out.write_all(b"\n")?;
                        }
                    }
                }
            }
            Format::Turtle | Format::Ntriples => {
                let results: Vec<_> = shown.iter().map(|r| &r.result).collect();
                let graph =
                    result_graph::graph(self.command, &results).map_err(io::Error::other)?;
                let format = if format == Format::Turtle {
                    raf_rdf::Format::Turtle
                } else {
                    raf_rdf::Format::NTriples
                };
                out.write_all(graph.to_text(format).map_err(io::Error::other)?.as_bytes())?;
            }
            Format::Json => {
                let results: Vec<_> = shown.iter().map(|record| &record.result).collect();
                let graph =
                    result_graph::graph(self.command, &results).map_err(io::Error::other)?;
                // The report emitter assigns digest-derived owned IRIs. Reuse that
                // authoritative snapshot before native JSON-LD serialization.
                let text = graph
                    .to_text(raf_rdf::Format::NTriples)
                    .map_err(io::Error::other)?;
                let snapshot =
                    purrdf_rdf::parse_dataset(text.as_bytes(), "application/n-triples", None)
                        .map_err(io::Error::other)?;
                out.write_all(&rdf_bytes(&snapshot, NativeRdfFormat::JsonLd)?)?;
            }
        }
        out.flush()?;
        self.print_notes(quiet, diagnostics)
    }
    fn print_notes(&self, quiet: bool, diagnostics: &mut dyn Write) -> io::Result<()> {
        if !quiet {
            for note in &self.notes {
                writeln!(diagnostics, "cargo-workplan: {note}")?;
            }
            if !self.notes.is_empty() {
                diagnostics.flush()?;
            }
        }
        Ok(())
    }
}

/// Every machine carrier retains the complete native statement layer.
fn rdf_bytes(dataset: &RdfDataset, format: NativeRdfFormat) -> io::Result<Vec<u8>> {
    let rendered = serialize_dataset_with(
        dataset,
        format,
        None,
        &SerializeOptions {
            selection: SerializeGraph::Dataset,
            statement_layer: StatementLayer::Emit,
            jsonld_options: None,
        },
    )
    .map_err(io::Error::other)?;
    if rendered.statement_rows_dropped != 0
        || rendered.directional_literals_dropped != 0
        || rendered.named_graph_rows_dropped != 0
    {
        return Err(io::Error::other(
            "native dataset serialization discarded RDF facts",
        ));
    }
    Ok(rendered.bytes)
}
