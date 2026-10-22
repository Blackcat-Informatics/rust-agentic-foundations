// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Human rendering and exit policy over typed results projected as RDF.

use crate::{exit::Exit, result::Result as CommandResult, result_graph};
use clap::ValueEnum;
use raf_workplan::Finding;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum Format {
    Text,
    Turtle,
    Ntriples,
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
#[derive(Debug, Clone, Default)]
pub(crate) struct Report {
    command: &'static str,
    records: Vec<Record>,
    notes: Vec<String>,
}
impl Report {
    pub(crate) fn of(records: impl IntoIterator<Item = Record>) -> Self {
        Self {
            command: "workplan",
            records: records.into_iter().collect(),
            notes: Vec::new(),
        }
    }
    pub(crate) fn command(mut self, command: &'static str) -> Self {
        self.command = command;
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
        if self.records.iter().any(|r| r.class == Class::Finding) {
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
        let shown: Vec<_> = self
            .records
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
        }
        out.flush()?;
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
