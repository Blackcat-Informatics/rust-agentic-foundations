// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Reading the document directories through a tree.

use raf_okf::{
    Body, DocType, Document, ErrorTag, Index, Location, MAX_TEXT_BYTES, Profile, RafDocument, Rule,
    Violation, is_capability_index,
};
use raf_source::{EntryKind, RelPath, Tree, TreeError, Workspace};

use crate::finding::kind;
use crate::graph::BindingKind;
use crate::links::{self, LinkContext};
use crate::model::IndexSlot;
use crate::{Finding, Loaded, Workplan, plain, refs};

/// The most bytes a document read may hold: one more than the profile accepts,
/// so a text over the profile bound reaches the reader and is refused there.
const READ_LIMIT: u64 = MAX_TEXT_BYTES as u64 + 1;

/// The file name of the navigation file of a directory.
const INDEX: &str = "index.md";

/// The document types the load reads, each from its directory: the
/// documents people write. The directories of the record types are not read.
const LOADED: [DocType; 3] = [DocType::Capability, DocType::Change, DocType::Architecture];

pub(crate) fn load<'s>(
    docs: &dyn Tree,
    source: &'s Workspace,
    links: &LinkContext<'_>,
) -> Workplan<'s> {
    let mut plan = Workplan::empty(source);
    for doc_type in LOADED {
        scan_directory(docs, doc_type, links, &mut plan);
    }
    plan.relate();
    for (directory, reason) in links.unreadable() {
        plan.findings.push(Finding::new(
            kind::UNREADABLE_DIRECTORY,
            directory,
            Location::Document,
            format!("the directory cannot be listed for the tracked paths: {reason}"),
            "make the directory readable, or remove it; nothing is skipped",
        ));
    }
    plan
}

/// Walk one document directory: the files directly in it are read, and a
/// Markdown file below it is nested.
fn scan_directory(
    docs: &dyn Tree,
    doc_type: DocType,
    links: &LinkContext<'_>,
    plan: &mut Workplan<'_>,
) {
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
                    read_file(docs, doc_type, &path, links, plan);
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
fn read_file(
    docs: &dyn Tree,
    dir_type: DocType,
    path: &str,
    links: &LinkContext<'_>,
    plan: &mut Workplan<'_>,
) {
    let text = match RelPath::new(path)
        .map_err(|error| TreeError::OutsideRoot {
            path: error.input().to_string(),
        })
        .and_then(|rel| docs.read(&rel, READ_LIMIT))
    {
        Ok(text) => text,
        Err(error) => {
            plan.note_present(dir_type, path);
            let finding = unreadable(kind::UNREADABLE_FILE, path, "file", &error);
            if is_capability_index(path) {
                plan.index = IndexSlot::Invalid(finding.clone());
            }
            plan.findings.push(finding);
            return;
        }
    };
    refs::scan(path, &text, &mut plan.findings);
    if path.rsplit('/').next() == Some(INDEX) {
        // Navigation, not a document. The capability index is generated, so
        // it must read as an index; any other index is Markdown that every
        // rule shared by Markdown files judges.
        if is_capability_index(path) {
            match Index::parse(&text) {
                Ok(index) => {
                    links::check_body(path, index.body(), links, &mut plan.findings);
                    plan.index = IndexSlot::Valid(index);
                }
                Err(error) => {
                    let finding = Finding::from_parse_error(path, &error);
                    plan.index = IndexSlot::Invalid(finding.clone());
                    plan.findings.push(finding);
                }
            }
        } else {
            plain::rules(path, &text, links, &mut plan.findings);
        }
        return;
    }
    let document = match Document::parse(&text) {
        Ok(document) => document,
        Err(error) => {
            // A page without a frontmatter fence is no document; any other
            // failure is a document that is present and broken.
            if error.kind().tag() == ErrorTag::MissingFence {
                // The page is a stray file, and it is still Markdown: the
                // rules every Markdown file shares judge it.
                plan.findings.push(Finding::from_parse_error(path, &error));
                plain::rules(path, &text, links, &mut plan.findings);
            } else {
                plan.note_present(dir_type, path);
                plan.findings.push(Finding::from_parse_error(path, &error));
            }
            return;
        }
    };
    plan.note_present(document.doc_type().unwrap_or(dir_type), path);
    links::check_body(path, document.body(), links, &mut plan.findings);
    if let Some(declared) = document.doc_type().filter(|declared| *declared != dir_type) {
        plan.findings.push(Finding::new(
            kind::WRONG_DIRECTORY,
            path,
            Location::Key("type".to_string()),
            format!(
                "{} {} document sits in {}",
                declared.article(),
                declared.name(),
                dir_type.directory()
            ),
            format!("move the file directly into {}", declared.directory()),
        ));
    }
    match Profile.validate(&document, path) {
        Ok(typed) => admit(plan, path, typed, document.body().clone(), &text),
        Err(violations) => {
            // The blocks of a document that does not validate still name
            // change records; the rules between documents judge them once
            // every record is read.
            if doc_type_is_capability(&document, dir_type) {
                plan.unvalidated_blocks
                    .extend(document.body().proposed_blocks().iter().map(|block| {
                        (
                            path.to_string(),
                            block.slug().to_string(),
                            block.marker().lines.start,
                        )
                    }));
            }
            if doc_type_is_capability(&document, dir_type) {
                note_unbound(plan, path, &violations);
            }
            plan.findings
                .extend(violations.iter().map(Finding::from_violation));
        }
    }
}

/// Add a document that satisfies the profile to the model.
fn admit(plan: &mut Workplan<'_>, path: &str, typed: RafDocument, body: Body, text: &str) {
    let path = path.to_string();
    match typed {
        RafDocument::Capability(value) => {
            plan.capabilities.push(Loaded::new(path, value, body, text));
        }
        RafDocument::Change(value) => {
            plan.changes.push(Loaded::new(path, value, body, text));
        }
        RafDocument::Architecture(value) => {
            plan.architecture.push(Loaded::new(path, value, body, text));
        }
        // A record validates only in its own directory, which the load does
        // not read, so none reaches this point; one in a read directory is
        // `wrong-directory`, reported before validation.
        RafDocument::Evidence(_) | RafDocument::Assessment(_) | RafDocument::Coverage(_) => {}
    }
}

/// Record a capability document whose only violation is a delivered
/// capability that binds nothing in a list it must bind: the document is
/// otherwise sound, and what it owes is that binding.
fn note_unbound(plan: &mut Workplan<'_>, path: &str, violations: &[Violation]) {
    let required: Vec<BindingKind> = violations
        .iter()
        .filter_map(|violation| match violation.rule() {
            Rule::BindingRequired { binding } => BindingKind::from_key(binding),
            _ => None,
        })
        .collect();
    if !required.is_empty() && required.len() == violations.len() {
        plan.unbound.push((path.to_string(), required));
    }
}

/// Whether a document is read as a capability: it says so, or says nothing
/// and sits in the capability directory.
fn doc_type_is_capability(document: &Document, dir_type: DocType) -> bool {
    document.doc_type().unwrap_or(dir_type) == DocType::Capability
}
