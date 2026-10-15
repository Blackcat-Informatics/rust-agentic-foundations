// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! What a command prints: records, their text form and their JSON form, and
//! the exit code a set of records implies.
//!
//! A command returns a [`Report`], a list of [`Record`]s in the order they are
//! printed. A record is either a finding (a violation, an obligation, a
//! refusal or a failing binding: anything that makes the run exit 1) or an
//! outcome (a file rewritten, a roadmap, a preview: the result of a command
//! that did not find anything wrong). `-q` prints only the findings. The run
//! exits 1 exactly when the report holds a finding, whatever `-q` hides.
//!
//! The JSON encoding of a record is described in the crate documentation of
//! the binary (`src/main.rs`), and `tests/json.rs` pins it.

use std::io::{self, Write};

use clap::ValueEnum;
use raf_workplan::Finding;
use serde_json::Value;

use crate::exit::Exit;

/// The `--format` choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum Format {
    /// One line or block of text for each record.
    Text,
    /// One JSON array for the run.
    Json,
}

/// Whether a record makes the run exit 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Class {
    /// A finding; `-q` keeps it.
    Finding,
    /// A result that is not a finding; `-q` hides it.
    Outcome,
}

/// One printed item: a kind, its JSON fields and its text.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Record {
    class: Class,
    kind: String,
    fields: Vec<(&'static str, Value)>,
    text: String,
}

impl Record {
    /// An outcome of the given kind, with the fields that follow `kind` in
    /// JSON and the text that follows in text.
    pub(crate) fn outcome(
        kind: &str,
        fields: Vec<(&'static str, Value)>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            class: Class::Outcome,
            kind: kind.to_string(),
            fields,
            text: text.into(),
        }
    }

    /// A finding of the given kind, with the fields that follow `kind` in
    /// JSON and the text.
    pub(crate) fn finding(
        kind: &str,
        fields: Vec<(&'static str, Value)>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            class: Class::Finding,
            kind: kind.to_string(),
            fields,
            text: text.into(),
        }
    }

    /// A finding of the document model: its `Display` text, and the fields of
    /// a rule.
    pub(crate) fn of_finding(finding: &Finding) -> Self {
        let optional = |value: Option<Value>| value.unwrap_or(Value::Null);
        Self::finding(
            finding.kind,
            vec![
                ("path", Value::from(finding.file.as_str())),
                ("section", optional(finding.section().map(Value::from))),
                ("line", optional(finding.line().map(Value::from))),
                ("key", optional(finding.key().map(Value::from))),
                ("message", Value::from(finding.message.as_str())),
                ("fix", Value::from(finding.fix.as_str())),
            ],
            finding.to_string(),
        )
    }

    /// The record as one JSON object, `kind` first.
    fn json(&self) -> String {
        let mut object = String::from("{");
        member(&mut object, "kind", &Value::from(self.kind.as_str()));
        for (name, value) in &self.fields {
            object.push(',');
            member(&mut object, name, value);
        }
        object.push('}');
        object
    }
}

/// `"name":value` appended to `object`.
fn member(object: &mut String, name: &str, value: &Value) {
    object.push_str(&Value::from(name).to_string());
    object.push(':');
    object.push_str(&value.to_string());
}

/// What a command produced: its records, in print order, and the notes it has
/// for standard error.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Report {
    records: Vec<Record>,
    notes: Vec<String>,
}

impl Report {
    /// A report of these records.
    pub(crate) fn of(records: impl IntoIterator<Item = Record>) -> Self {
        Self {
            records: records.into_iter().collect(),
            notes: Vec::new(),
        }
    }

    /// Add a note for standard error. `-q` hides notes; a failure is never a
    /// note.
    pub(crate) fn note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }

    /// The exit of the run: 1 when any record is a finding, else 0.
    pub(crate) fn exit(&self) -> Exit {
        if self
            .records
            .iter()
            .any(|record| record.class == Class::Finding)
        {
            Exit::Findings
        } else {
            Exit::Clean
        }
    }

    /// Print the report: the records on `out` in `format`, the notes on
    /// `diagnostics`. With `quiet` only the findings are printed and the notes
    /// are not.
    ///
    /// # Errors
    ///
    /// The error of writing to `out`.
    pub(crate) fn print(
        &self,
        format: Format,
        quiet: bool,
        out: &mut dyn Write,
        diagnostics: &mut dyn Write,
    ) -> io::Result<()> {
        let shown: Vec<&Record> = self
            .records
            .iter()
            .filter(|record| !quiet || record.class == Class::Finding)
            .collect();
        match format {
            Format::Text => {
                // A record with no text has no line in the text form: its
                // fact is in the line of another record.
                for record in shown.into_iter().filter(|record| !record.text.is_empty()) {
                    out.write_all(record.text.as_bytes())?;
                    if !record.text.ends_with('\n') {
                        out.write_all(b"\n")?;
                    }
                }
            }
            Format::Json => {
                let objects: Vec<String> = shown.iter().map(|record| record.json()).collect();
                if objects.is_empty() {
                    out.write_all(b"[]\n")?;
                } else {
                    writeln!(out, "[\n{}\n]", objects.join(",\n"))?;
                }
            }
        }
        out.flush()?;
        if !quiet {
            for note in &self.notes {
                // A diagnostic that cannot be written changes nothing.
                let _ = writeln!(diagnostics, "cargo-workplan: {note}");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding() -> Record {
        Record::finding(
            "link-broken",
            vec![
                ("path", Value::from("docs/capabilities/a.md")),
                ("section", Value::Null),
                ("line", Value::from(7)),
                ("key", Value::Null),
                ("message", Value::from("a link is broken")),
                ("fix", Value::from("repair it")),
            ],
            "docs/capabilities/a.md: line 7: a link is broken [link-broken]; repair it",
        )
    }

    fn printed(report: &Report, format: Format, quiet: bool) -> (String, String) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        report
            .print(format, quiet, &mut out, &mut err)
            .expect("an in-memory write");
        (
            String::from_utf8(out).expect("UTF-8"),
            String::from_utf8(err).expect("UTF-8"),
        )
    }

    #[test]
    fn an_empty_report_is_nothing_in_text_and_an_empty_array_in_json() {
        let report = Report::default();
        assert_eq!(printed(&report, Format::Text, false).0, "");
        assert_eq!(printed(&report, Format::Json, false).0, "[]\n");
        assert_eq!(report.exit(), Exit::Clean);
    }

    #[test]
    fn a_finding_makes_the_exit_one_and_an_outcome_does_not() {
        let outcome = Record::outcome(
            "rewrote",
            vec![("path", Value::from("a.md"))],
            "rewrote a.md",
        );
        assert_eq!(Report::of([outcome.clone()]).exit(), Exit::Clean);
        let both = Report::of([outcome, finding()]);
        assert_eq!(both.exit(), Exit::Findings);
    }

    #[test]
    fn quiet_prints_only_findings_and_keeps_the_exit() {
        let outcome = Record::outcome(
            "rewrote",
            vec![("path", Value::from("a.md"))],
            "rewrote a.md",
        );
        let mut report = Report::of([outcome, finding()]);
        report.note("a note");
        let (loud, loud_err) = printed(&report, Format::Text, false);
        assert!(loud.contains("rewrote a.md") && loud.contains("[link-broken]"));
        assert!(loud_err.contains("a note"));
        let (quiet, quiet_err) = printed(&report, Format::Text, true);
        assert!(!quiet.contains("rewrote") && quiet.contains("[link-broken]"));
        assert_eq!(quiet_err, "");
        assert_eq!(report.exit(), Exit::Findings);
    }

    #[test]
    fn a_json_object_begins_with_its_kind_and_keeps_the_order_of_its_fields() {
        let (out, _) = printed(&Report::of([finding()]), Format::Json, false);
        assert!(
            out.starts_with("[\n{\"kind\":\"link-broken\",\"path\":\"docs/capabilities/a.md\",\"section\":null,\"line\":7,\"key\":null,\"message\":"),
            "{out}"
        );
        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(parsed.as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn a_record_with_no_text_has_no_line_in_text_and_an_object_in_json() {
        let silent = Record::finding("refusal", vec![("repeated", Value::from(true))], "");
        let report = Report::of([silent, finding()]);
        let (text, _) = printed(&report, Format::Text, false);
        assert_eq!(text.lines().count(), 1, "{text}");
        let (json, _) = printed(&report, Format::Json, false);
        let parsed: Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed.as_array().map(Vec::len), Some(2));
        assert_eq!(report.exit(), Exit::Findings);
    }

    #[test]
    fn text_with_quotes_and_control_characters_is_escaped() {
        let record = Record::outcome(
            "preview",
            vec![("text", Value::from("a \"quoted\"\nline\t\u{1}"))],
            "x",
        );
        let (out, _) = printed(&Report::of([record]), Format::Json, false);
        let parsed: Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(parsed[0]["text"], "a \"quoted\"\nline\t\u{1}");
    }
}
