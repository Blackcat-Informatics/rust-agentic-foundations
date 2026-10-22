// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Human tables of typed history entries and removed bindings.
//! Cells escape table delimiters, controls and bidirectional formatting.

use super::{Entry, History, Removal, RemovedBinding, Source};
use crate::escape::shown;
use crate::generated::{cell, code_span};

/// The digits of an abbreviated commit id.
const SHORT: usize = 7;

/// `text` as a table cell: the generated-table rule, then each control or
/// bidirectional formatting character that rule leaves as its escape
/// ([`crate::escape`]).
pub(crate) fn table_cell(text: &str) -> String {
    shown(&cell(text))
}

/// A code span in a table cell, escaped in the same way.
pub(crate) fn table_code(text: &str) -> String {
    shown(&code_span(text))
}

fn short(commit: &str) -> &str {
    &commit[..SHORT.min(commit.len())]
}

fn source_cell(source: &Source) -> String {
    match source {
        Source::Document => "document".to_string(),
        Source::Symbol(path) => table_code(path),
    }
}

fn entry_row(entry: &Entry) -> String {
    let sources: Vec<String> = entry.sources().iter().map(source_cell).collect();
    format!(
        "| {} | {} | {} | {} |",
        short(entry.commit().as_str()),
        entry.date().day(),
        table_cell(entry.subject()),
        sources.join(", ")
    )
}

fn removed_row(removed: &RemovedBinding) -> String {
    let binding = table_code(removed.binding());
    match removed.removal() {
        Removal::Found {
            commit,
            date,
            subject,
            last_resolved,
        } => format!(
            "| {binding} | {} | {} | {} | {} |",
            short(commit.as_str()),
            date.day(),
            table_cell(subject),
            last_resolved
                .as_ref()
                .map_or("none: the first commit", |parent| short(parent.as_str()))
        ),
        Removal::NoCommit => format!(
            "| {binding} | none | - | no commit changed how often {} occurs in {} | - |",
            table_code(removed.identifier()),
            removed
                .dirs()
                .iter()
                .map(|dir| table_code(dir))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Removal::NoCrate => {
            format!("| {binding} | none | - | no package of the workspace provides its crate | - |")
        }
    }
}

/// The history as text: the table of entries and, when a binding no longer
/// resolves, the table of those bindings after a blank line.
#[must_use]
pub fn table(history: &History) -> String {
    let mut out = String::from("| Commit | Date | Subject | Source |\n| --- | --- | --- | --- |\n");
    for entry in history.entries() {
        out.push_str(&entry_row(entry));
        out.push('\n');
    }
    if !history.removed().is_empty() {
        out.push_str(
            "\n| Binding | Removed in | Date | Subject | Last resolved |\n| --- | --- | --- | --- | --- |\n",
        );
        for removed in history.removed() {
            out.push_str(&removed_row(removed));
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_table_cell_cannot_break_the_table_or_reach_the_terminal() {
        assert_eq!(table_cell("a | b"), "a \\| b");
        assert_eq!(table_cell("x\u{1b}[31m\u{1f}"), "x\\u{1b}\\[31m\\u{1f}");
        assert_eq!(table_cell("one\ntwo"), "one two");
        assert_eq!(table_cell("one\rtwo\r\nthree"), "one two three");
        assert_eq!(table_code("a`b"), "``a`b``");
        assert_eq!(
            table_cell("a\u{202e}b\u{2066}c\u{200f}"),
            "a\\u{202e}b\\u{2066}c\\u{200f}"
        );
        // The neighbour: other non-ASCII text is kept.
        assert_eq!(table_cell("été \u{2014} 名前"), "été \u{2014} 名前");
    }
}
