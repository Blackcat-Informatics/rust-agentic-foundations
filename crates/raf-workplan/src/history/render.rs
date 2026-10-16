// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The two forms of a [`History`]: a table for a reader and JSON for a tool,
//! carrying the same entries in the same order.
//!
//! [`table`] writes `| Commit | Date | Subject | Source |`, one row per entry:
//! the commit abbreviated to its first seven digits, the day of its author
//! date, the subject, and the sources (`document`, and each bound symbol as a
//! code span). A binding that no longer resolves is a row of a second table,
//! `| Binding | Removed in | Date | Subject | Last resolved |`. Cell text is
//! escaped by the rule of the generated tables (`\`, `|`, `[`, `]` and `<`),
//! and a control character or a bidirectional formatting character is written
//! as its `\u{..}` escape, so a subject can neither break the table, move the
//! cursor of a terminal, nor reorder how the row reads.
//!
//! [`json`] writes one array: an object `{"kind":"history-entry", ...}` per
//! entry with the full commit id, the author date as `git` printed it, the
//! subject, the sources (`{"source":"document"}` or
//! `{"source":"symbol","binding":<path>}`) and the paths, then an object
//! `{"kind":"history-removed", ...}` per binding that no longer resolves. It is
//! the array form of the command line's JSON, one object to a line.

use std::fmt::Write as _;

use super::{Entry, History, Removal, RemovedBinding, Source};
use crate::generated::{cell, code_span};

/// The digits of an abbreviated commit id.
const SHORT: usize = 7;

/// Whether `c` changes how a terminal shows the text around it: a control
/// character, or a bidirectional formatting character (the marks U+061C,
/// U+200E and U+200F, the embeddings and overrides U+202A to U+202E, and the
/// isolates U+2066 to U+2069).
fn hidden(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{61c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
        )
}

/// `text` with each [`hidden`] character as its `\u{..}` escape.
fn shown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if hidden(c) {
            let _ = write!(out, "\\u{{{:x}}}", u32::from(c));
        } else {
            out.push(c);
        }
    }
    out
}

/// `text` as a table cell: the generated-table rule, then each control or
/// bidirectional formatting character that rule leaves as its escape.
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

/// `text` as a JSON string (RFC 8259): `"` and `\` escaped, each control
/// character as a short escape or `\u00XX`.
pub(crate) fn string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if u32::from(c) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn array(items: impl IntoIterator<Item = String>) -> String {
    format!("[{}]", items.into_iter().collect::<Vec<_>>().join(","))
}

fn entry_object(entry: &Entry) -> String {
    let sources = array(entry.sources().iter().map(|source| match source {
        Source::Document => "{\"source\":\"document\"}".to_string(),
        Source::Symbol(path) => format!("{{\"source\":\"symbol\",\"binding\":{}}}", string(path)),
    }));
    format!(
        "{{\"kind\":\"history-entry\",\"commit\":{},\"date\":{},\"subject\":{},\"sources\":{sources},\"paths\":{}}}",
        string(entry.commit().as_str()),
        string(entry.date().as_str()),
        string(entry.subject()),
        array(entry.paths().iter().map(|path| string(path)))
    )
}

fn removed_object(removed: &RemovedBinding) -> String {
    let head = format!(
        "{{\"kind\":\"history-removed\",\"binding\":{},\"identifier\":{},\"dirs\":{}",
        string(removed.binding()),
        string(removed.identifier()),
        array(removed.dirs().iter().map(|dir| string(dir)))
    );
    match removed.removal() {
        Removal::Found {
            commit,
            date,
            subject,
            last_resolved,
        } => format!(
            "{head},\"lookup\":\"found\",\"commit\":{},\"date\":{},\"subject\":{},\"last_resolved\":{}}}",
            string(commit.as_str()),
            string(date.as_str()),
            string(subject),
            last_resolved
                .as_ref()
                .map_or_else(|| "null".to_string(), |parent| string(parent.as_str()))
        ),
        Removal::NoCommit => format!(
            "{head},\"lookup\":\"no-commit\",\"commit\":null,\"date\":null,\"subject\":null,\"last_resolved\":null}}"
        ),
        Removal::NoCrate => format!(
            "{head},\"lookup\":\"no-crate\",\"commit\":null,\"date\":null,\"subject\":null,\"last_resolved\":null}}"
        ),
    }
}

/// The history as one JSON array: the entries, then the bindings that no
/// longer resolve, one object to a line; `[]` when there is neither.
#[must_use]
pub fn json(history: &History) -> String {
    let objects: Vec<String> = history
        .entries()
        .iter()
        .map(entry_object)
        .chain(history.removed().iter().map(removed_object))
        .collect();
    if objects.is_empty() {
        "[]\n".to_string()
    } else {
        format!("[\n{}\n]\n", objects.join(",\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_json_string_escapes_quotes_backslashes_and_every_control_character() {
        assert_eq!(string("a\"b\\c"), "\"a\\\"b\\\\c\"");
        assert_eq!(string("\n\r\t\u{8}\u{c}"), "\"\\n\\r\\t\\b\\f\"");
        assert_eq!(string("\u{1e}\u{1f}\u{0}"), "\"\\u001e\\u001f\\u0000\"");
        for byte in 0u8..0x20 {
            let text = string(&char::from(byte).to_string());
            assert!(text.chars().all(|c| !c.is_control()), "{byte}: {text:?}");
        }
        assert_eq!(string("\u{7f}é"), "\"\u{7f}é\"");
    }

    #[test]
    fn a_table_cell_cannot_break_the_table_or_reach_the_terminal() {
        assert_eq!(table_cell("a | b"), "a \\| b");
        assert_eq!(table_cell("x\u{1b}[31m\u{1f}"), "x\\u{1b}\\[31m\\u{1f}");
        assert_eq!(table_cell("one\ntwo"), "one two");
        assert_eq!(table_code("a`b"), "``a`b``");
        assert_eq!(
            table_cell("a\u{202e}b\u{2066}c\u{200f}"),
            "a\\u{202e}b\\u{2066}c\\u{200f}"
        );
        // The neighbour: other non-ASCII text is kept.
        assert_eq!(table_cell("été \u{2014} 名前"), "été \u{2014} 名前");
    }
}
