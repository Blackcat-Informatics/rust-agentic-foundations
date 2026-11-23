// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The repository's tracked document paths, contents, and link context.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use raf_okf::{DocType, Document};
use raf_source::{MemoryTree, RelPath};
use raf_workplan::{LinkContext, plain};
use rust_agentic_foundations::gate_files::{decode_text, read_regular};

pub(super) const INDEX: &str = "docs/capabilities/index.md";

pub(super) type Repo = BTreeMap<PathBuf, String>;

pub(super) struct Checkout {
    pub(super) paths: Vec<PathBuf>,
    pub(super) symlinks: Vec<PathBuf>,
    pub(super) markdown: Repo,
    pub(super) diagram_assets: Repo,
}

pub(super) fn path_name(path: &Path) -> Result<&str, String> {
    path.to_str()
        .ok_or_else(|| violation(path, "tracked path is not UTF-8"))
}

pub(super) fn tree_path(path: &Path) -> Result<RelPath, String> {
    RelPath::new(path_name(path)?)
        .map_err(|error| violation(path, format!("tracked path is not a tree path: {error}")))
}

pub(super) fn violation(path: &Path, message: impl std::fmt::Display) -> String {
    format!("{}: {message}", path.display())
}

fn is_markdown(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "md")
}

pub(super) fn in_document_directory(path: &Path) -> bool {
    DocType::ALL
        .into_iter()
        .any(|doc_type| path.starts_with(doc_type.directory()))
}

pub(super) fn load_checkout(
    root: &Path,
    paths: Vec<PathBuf>,
    symlinks: Vec<PathBuf>,
) -> Result<Checkout, String> {
    let mut markdown = Repo::new();
    let mut diagram_assets = Repo::new();
    for path in paths.iter().filter(|path| !symlinks.contains(path)) {
        if is_markdown(path) {
            let bytes = read_regular(root, path)?;
            let text = decode_text(&bytes)
                .ok_or_else(|| violation(path, "tracked Markdown contains a NUL byte"))?;
            markdown.insert(path.clone(), text.into_owned());
        } else if path.starts_with("docs/diagrams")
            && matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("json" | "svg" | "ttl" | "trig" | "txt")
            )
        {
            let text = String::from_utf8(read_regular(root, path)?)
                .map_err(|error| violation(path, format!("invalid UTF-8 diagram data: {error}")))?;
            diagram_assets.insert(path.clone(), text);
        }
    }
    Ok(Checkout {
        paths,
        symlinks,
        markdown,
        diagram_assets,
    })
}

pub(super) fn with_links<T>(
    checkout: &Checkout,
    check: impl FnOnce(&MemoryTree, &LinkContext<'_>) -> T,
) -> Result<T, String> {
    let entries = checkout
        .markdown
        .iter()
        .chain(&checkout.diagram_assets)
        .map(|(path, text)| path_name(path).map(|name| (name, text.as_str())))
        .collect::<Result<Vec<_>, _>>()?;
    let tree = MemoryTree::try_from_iter(entries).map_err(|error| {
        format!("the tracked document and diagram text does not form a tree: {error}")
    })?;
    let paths = checkout
        .paths
        .iter()
        .map(|path| tree_path(path))
        .collect::<Result<Vec<_>, _>>()?;
    let symlinks = checkout
        .symlinks
        .iter()
        .map(|path| tree_path(path))
        .collect::<Result<Vec<_>, _>>()?;
    let context = LinkContext::new(&tree, paths).with_symlinks(symlinks);
    Ok(check(&tree, &context))
}

pub(super) fn checkout_findings(checkout: &Checkout) -> Vec<String> {
    let mut findings = Vec::new();
    findings.extend(
        checkout
            .symlinks
            .iter()
            .map(|path| violation(path, "a tracked symlink is not allowed")),
    );
    findings.extend(
        checkout
            .paths
            .iter()
            .filter(|path| in_document_directory(path) && !is_markdown(path))
            .map(|path| {
                violation(
                    path,
                    "only Markdown documents belong in a document directory",
                )
            }),
    );
    findings.extend(outside_violations(&checkout.markdown));
    findings
}

fn is_documentation(path: &Path) -> bool {
    let mut components = path.components();
    let first = components.next();
    components.next().is_none()
        || first.is_some_and(|part| part.as_os_str() == "docs" || part.as_os_str() == "work")
}

fn outside_violations(markdown: &Repo) -> impl Iterator<Item = String> {
    markdown
        .iter()
        .filter(|(path, _)| is_documentation(path) && !in_document_directory(path))
        .filter_map(|(path, text)| {
            let document = Document::parse(text).ok()?;
            let message = match document.doc_type() {
                Some(declared) => format!(
                    "`type`: {} {} document belongs directly in {}",
                    declared.article(),
                    declared.name(),
                    declared.directory()
                ),
                None => format!(
                    "frontmatter belongs only in the documents of {}; this page is outside them",
                    DocType::ALL.map(DocType::directory).join(", ")
                ),
            };
            Some(violation(path, message))
        })
}

pub(super) fn static_findings(checkout: &Checkout) -> Result<Vec<String>, String> {
    if checkout.markdown.is_empty() {
        return Err("no tracked Markdown to check".to_owned());
    }
    let mut findings = checkout_findings(checkout);
    with_links(checkout, |_, context| {
        for (path, text) in &checkout.markdown {
            let name = path_name(path)?;
            if name == INDEX || !in_document_directory(path) {
                findings.extend(
                    plain::check(name, text, context)
                        .iter()
                        .map(ToString::to_string),
                );
            } else {
                match Document::parse(text) {
                    Ok(document) => findings.extend(
                        context
                            .check_links(name, document.body())
                            .iter()
                            .map(ToString::to_string),
                    ),
                    Err(error) => findings.push(violation(path, error)),
                }
            }
        }
        Ok(findings)
    })?
}
