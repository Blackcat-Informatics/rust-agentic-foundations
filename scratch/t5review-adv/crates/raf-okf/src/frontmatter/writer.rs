// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The deterministic writer of the frontmatter.
//!
//! The writer is total: every value that exists was built by the reader or by
//! a constructor that refuses what the reader would not read back.
//!
//! Key order follows the schema table of the document type, then any other
//! keys in input order; a document of an unknown type keeps input order. The
//! `tags` list is written sorted. Comments travel with the entry or item they
//! are attached to, and the comments after the last entry stay last.

use crate::frontmatter::reader::LineEnding;
use crate::frontmatter::scalar::write_scalar;
use crate::frontmatter::value::{
    Entry, Frontmatter, ItemValue, List, ListForm, ListItem, Scalar, ScalarStyle, Value,
};
use crate::profile::DocType;

/// The key whose list the writer sorts.
const TAGS_KEY: &str = "tags";

/// The entries of the top-level mapping in the order the writer emits them.
pub(crate) fn ordered_entries(frontmatter: &Frontmatter) -> Vec<&Entry> {
    let entries = frontmatter.map.entries.iter();
    let doc_type = frontmatter
        .map
        .get("type")
        .and_then(|entry| entry.value.as_scalar())
        .and_then(|scalar| DocType::from_name(&scalar.text));
    let Some(doc_type) = doc_type else {
        return entries.collect();
    };
    let table = doc_type.keys();
    let mut ordered: Vec<&Entry> = Vec::with_capacity(frontmatter.map.entries.len());
    for spec in table {
        if let Some(entry) = frontmatter.map.get(spec.key) {
            ordered.push(entry);
        }
    }
    ordered.extend(entries.filter(|entry| !table.iter().any(|spec| spec.key == entry.key)));
    ordered
}

/// Write the frontmatter lines, each followed by `eol`.
pub(crate) fn write_frontmatter(out: &mut String, frontmatter: &Frontmatter, eol: LineEnding) {
    let mut writer = Writer {
        out,
        eol: eol.as_str(),
    };
    for entry in ordered_entries(frontmatter) {
        let sort = entry.key == TAGS_KEY;
        writer.entry(entry, 0, "", sort);
    }
    for comment in &frontmatter.trailing_comments {
        writer.line(comment);
    }
}

struct Writer<'o> {
    out: &'o mut String,
    eol: &'static str,
}

impl Writer<'_> {
    fn line(&mut self, text: &str) {
        self.out.push_str(text);
        self.out.push_str(self.eol);
    }

    fn comments(&mut self, leading: &[String]) {
        for comment in leading {
            self.line(comment);
        }
    }

    /// Write `entry` whose mapping keys sit at column `indent`. `lead` is the
    /// text before the key on its line: the indentation, or the indentation
    /// of a list item ending in `- `.
    fn entry(&mut self, entry: &Entry, indent: usize, lead: &str, sort: bool) {
        self.comments(&entry.comments.leading);
        let mut line = String::new();
        if lead.is_empty() {
            line.push_str(&" ".repeat(indent));
        } else {
            line.push_str(lead);
        }
        line.push_str(&entry.key);
        line.push(':');
        match &entry.value {
            Value::Scalar(scalar) => {
                if !(scalar.style == ScalarStyle::Plain && scalar.text.is_empty()) {
                    line.push(' ');
                    write_scalar(&mut line, scalar);
                }
                push_trailing(&mut line, entry.comments.trailing.as_deref());
                self.line(&line);
            }
            Value::List(list) => {
                if let Some(scalars) = flow_scalars(list, sort) {
                    line.push(' ');
                    write_flow(&mut line, &scalars);
                    push_trailing(&mut line, entry.comments.trailing.as_deref());
                    self.line(&line);
                } else {
                    push_trailing(&mut line, entry.comments.trailing.as_deref());
                    self.line(&line);
                    self.block_list(list, indent + 2, sort);
                }
            }
            Value::Map(map) => {
                push_trailing(&mut line, entry.comments.trailing.as_deref());
                self.line(&line);
                for child in &map.entries {
                    self.entry(child, indent + 2, "", false);
                }
            }
        }
    }

    fn block_list(&mut self, list: &List, indent: usize, sort: bool) {
        for item in items_in_order(list, sort) {
            self.comments(&item.comments.leading);
            match &item.value {
                ItemValue::Scalar(scalar) => {
                    let mut line = " ".repeat(indent);
                    line.push_str("- ");
                    write_scalar(&mut line, scalar);
                    push_trailing(&mut line, item.comments.trailing.as_deref());
                    self.line(&line);
                }
                ItemValue::Map(map) => {
                    let lead = format!("{}- ", " ".repeat(indent));
                    for (index, child) in map.entries.iter().enumerate() {
                        let lead = if index == 0 { lead.as_str() } else { "" };
                        self.entry(child, indent + 2, lead, false);
                    }
                }
            }
        }
    }
}

fn push_trailing(line: &mut String, trailing: Option<&str>) {
    if let Some(text) = trailing {
        line.push_str(text);
    }
}

/// The items of `list`, sorted by scalar text when `sort` is set. The sort is
/// stable, so equal items keep author order.
fn items_in_order(list: &List, sort: bool) -> Vec<&ListItem> {
    let mut items: Vec<&ListItem> = list.items.iter().collect();
    if sort {
        items.sort_by(|left, right| scalar_text(left).cmp(&scalar_text(right)));
    }
    items
}

fn scalar_text(item: &ListItem) -> Option<&str> {
    item.value.as_scalar().map(|scalar| scalar.text.as_str())
}

/// The items of a list written as `[a, b]`: every item must be a scalar. A
/// list that holds a mapping item cannot be flow, so it is written as a block
/// list; the constructors never build that combination.
fn flow_scalars(list: &List, sort: bool) -> Option<Vec<&Scalar>> {
    if list.form != ListForm::Flow {
        return None;
    }
    items_in_order(list, sort)
        .into_iter()
        .map(|item| item.value.as_scalar())
        .collect()
}

fn write_flow(line: &mut String, scalars: &[&Scalar]) {
    line.push('[');
    for (index, scalar) in scalars.iter().enumerate() {
        if index > 0 {
            line.push_str(", ");
        }
        write_scalar(line, scalar);
    }
    line.push(']');
}
