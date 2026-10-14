// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Reading the document directories through a tree.

use raf_okf::{
    DocType, Document, Index, Location, MAX_TEXT_BYTES, Profile, RafDocument, is_capability_index,
};
use raf_source::{EntryKind, RelPath, Tree, TreeError, Workspace};

use crate::finding::kind;
use crate::{Finding, Loaded, Workplan};

/// The most bytes a document read may hold: one more than the profile accepts,
/// so a text over the profile bound reaches the reader and is refused there.
const READ_LIMIT: u64 = MAX_TEXT_BYTES as u64 + 1;

/// The file name of the navigation file of a directory.
const INDEX: &str = "index.md";

pub(crate) fn load<'s>(docs: &dyn Tree, source: &'s Workspace) -> Workplan<'s> {
    let mut plan = Workplan::empty(source);
    for doc_type in DocType::ALL {
        scan_directory(docs, doc_type, &mut plan);
    }
    plan
}

/// Walk one document directory: the files directly in it are read, and a
/// Markdown file below it is nested.
fn scan_directory(docs: &dyn Tree, doc_type: DocType, plan: &mut Workplan<'_>) {
    let home = doc_type.directory();
    let mut pending = vec![(home.to_string(), 0_usize)];
    while let Some((directory, depth)) = pending.pop() {
        let entries = match list(docs, &directory) {
            Ok(entries) => entries,
            Err(TreeError::NotFound { .. }) => continue,
            Err(error) => {
                plan.findings.push(unreadable(
                    kind::UNREADABLE_DIRECTORY,
                    &directory,
                    "directory",
                    &error,
                ));
                continue;
            }
        };
        // Popped last-in first, so the subdirectories are visited in name order.
        let mut subdirectories = Vec::new();
        for entry in entries {
            let path = format!("{directory}/{}", entry.name);
            if entry.kind == EntryKind::Dir {
                subdirectories.push((path, depth + 1));
            } else if is_markdown(&entry.name) {
                if depth == 0 {
                    read_file(docs, doc_type, &path, plan);
                } else {
                    plan.findings.push(nested(&path, home));
                }
            }
        }
        pending.extend(subdirectories.into_iter().rev());
    }
}

/// Whether a file name is a Markdown document name. The profile spells the
/// extension in lowercase only, so `NOTES.MD` is not a document.
#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "the profile defines a document file name by the lowercase suffix `.md`"
)]
fn is_markdown(name: &str) -> bool {
    name.ends_with(".md")
}

fn list(docs: &dyn Tree, directory: &str) -> Result<Vec<raf_source::Entry>, TreeError> {
    let path = RelPath::new(directory).map_err(|error| TreeError::OutsideRoot {
        path: error.input().to_string(),
    })?;
    docs.list(&path)
}

fn unreadable(kind: &'static str, path: &str, what: &str, error: &TreeError) -> Finding {
    Finding::new(
        kind,
        path,
        Location::Document,
        format!("the {what} cannot be read: {error}"),
        format!("make the {what} readable, or remove it; nothing is skipped"),
    )
}

fn nested(path: &str, home: &str) -> Finding {
    Finding::new(
        kind::NESTED_DIRECTORY,
        path,
        Location::Document,
        format!("a document sits below {home}; documents sit directly in their directory"),
        format!("move the file directly into {home}"),
    )
}

/// Read one file that sits directly in the directory of `dir_type`.
fn read_file(docs: &dyn Tree, dir_type: DocType, path: &str, plan: &mut Workplan<'_>) {
    let text = match RelPath::new(path)
        .map_err(|error| TreeError::OutsideRoot {
            path: error.input().to_string(),
        })
        .and_then(|rel| docs.read(&rel, READ_LIMIT))
    {
        Ok(text) => text,
        Err(error) => {
            plan.findings
                .push(unreadable(kind::UNREADABLE_FILE, path, "file", &error));
            return;
        }
    };
    if path.rsplit('/').next() == Some(INDEX) {
        // Navigation, not a document. The capability index is generated, so
        // it must read as an index.
        if is_capability_index(path)
            && let Err(error) = Index::parse(&text)
        {
            plan.findings.push(Finding::from_parse_error(path, &error));
        }
        return;
    }
    let document = match Document::parse(&text) {
        Ok(document) => document,
        Err(error) => {
            plan.findings.push(Finding::from_parse_error(path, &error));
            return;
        }
    };
    if let Some(declared) = document.doc_type().filter(|declared| *declared != dir_type) {
        plan.findings.push(Finding::new(
            kind::WRONG_DIRECTORY,
            path,
            Location::Key("type".to_string()),
            format!(
                "a {} document sits in {}",
                declared.name(),
                dir_type.directory()
            ),
            format!("move the file directly into {}", declared.directory()),
        ));
    }
    match Profile.validate(&document, path) {
        Ok(typed) => {
            let body = document.body().clone();
            let path = path.to_string();
            match typed {
                RafDocument::Capability(value) => {
                    plan.capabilities.push(Loaded::new(path, value, body));
                }
                RafDocument::Change(value) => plan.changes.push(Loaded::new(path, value, body)),
                RafDocument::Architecture(value) => {
                    plan.architecture.push(Loaded::new(path, value, body));
                }
            }
        }
        Err(violations) => plan
            .findings
            .extend(violations.iter().map(Finding::from_violation)),
    }
}
