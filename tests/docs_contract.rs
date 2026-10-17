// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Document-contract test for this repository.
//!
//! The document model is `raf-workplan`'s. This test loads the tracked
//! Markdown of the repository (the git index, read under the gate root) into a
//! memory tree, judges its links against the tracked paths and the tracked
//! symlinks, resolves the bindings of the capability documents against the
//! source index of the whole repository, and requires `Workplan::check` to
//! report nothing. `raf_workplan::plain::check` judges the tracked Markdown
//! outside the document directories (README, AGENTS.md, templates), and
//! `Gate::workspace` runs the same checks from disk.
//!
//! What stays here is this repository's own policy, which no model rule
//! states:
//!
//! * the capability inventory (`INVENTORY`), which pins the documents, their
//!   states and the generated capability index;
//! * the refusal of tracked symlinks;
//! * Markdown-only document directories: any other file in the directory of a
//!   document type would sit unread beside the documents;
//! * typed pages live in their directories: a page at the top level or below
//!   `docs/` or `work/` that is outside the document directories may not open
//!   with a frontmatter block at all, whatever its `type`, because the model
//!   reads such a page as plain Markdown and would never validate it (a page
//!   with a document type is told where that type lives);
//! * the equality of the Usage and Examples blocks of two capability documents
//!   with the crate documentation tests;
//! * the helpers that read tracked files safely, and the guards that keep these
//!   checks from passing on an empty load.
//!
//! `violations` runs every rule family over one checkout. The repository test
//! and the fixture tests of each family go through it, so a family that is left
//! out of it fails a fixture test.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use raf_okf::{Body, DocType, Document, ErrorKind, FencedBlock};
use raf_source::{DiskTree, MemoryTree, RelPath, Workspace};
use raf_workplan::{Finding, Gate, LinkContext, Workplan, plain};

mod support;

use support::{decode_text, gate_root, read_regular, read_tracked, tracked_paths};

/// Tracked Markdown by path: the text of every tracked Markdown file that is
/// not a symlink.
type Repo = BTreeMap<PathBuf, String>;

/// The capability inventory.
///
/// This table is the capability inventory: the real tree must hold exactly
/// these capability documents, each with the stated `state`, exactly these
/// architecture documents, exactly these records, exactly these generated
/// documents, and an open change record named after every capability whose
/// state is not `delivered`. Changing a state, adding a capability or adding a
/// record edits this table in the same commit.
struct Inventory {
    /// Capability slug and the state its document declares.
    capabilities: &'static [(&'static str, &'static str)],
    /// Architecture slugs.
    architecture: &'static [&'static str],
    /// Records (evidence, assessments and the coverage record), by id: each
    /// lives at the path its type and slug give.
    records: &'static [&'static str],
    /// Generated documents, by path. They have no frontmatter, so only their
    /// presence is held.
    generated: &'static [&'static str],
}

const INVENTORY: Inventory = Inventory {
    capabilities: &[
        ("census-cli", "designed"),
        ("census-debt-ratchet", "designed"),
        ("census-detectors", "designed"),
        ("census-ledger", "designed"),
        ("census-migration", "designed"),
        ("census-normal-form", "designed"),
        ("census-patterns", "designed"),
        ("census-reference-scan", "designed"),
        ("census-rule-api", "designed"),
        ("okf-profile", "delivered"),
        ("source-index", "delivered"),
        ("workplan-cli", "delivered"),
        ("workplan-complete-gate", "delivered"),
        ("workplan-current-rationale", "implementing"),
        ("workplan-document-model", "delivered"),
        ("workplan-entity-history", "delivered"),
        ("workplan-evidence-claims", "implementing"),
        ("workplan-generated-docs", "delivered"),
        ("workplan-prose-assessment", "implementing"),
        ("workplan-reconcile", "delivered"),
        ("workplan-two-way-coverage", "implementing"),
    ],
    architecture: &["normal-form", "okf-profile"],
    records: &[
        "assessment:okf-profile-contract",
        "assessment:okf-profile-verification",
        "assessment:source-index-constraints",
        "assessment:source-index-contract",
        "assessment:source-index-examples",
        "assessment:source-index-usage",
        "assessment:source-index-verification",
        "assessment:workplan-cli-constraints",
        "assessment:workplan-cli-contract",
        "assessment:workplan-cli-examples",
        "assessment:workplan-cli-usage",
        "assessment:workplan-cli-verification",
        "assessment:workplan-complete-gate-contract",
        "assessment:workplan-complete-gate-examples",
        "assessment:workplan-complete-gate-verification",
        "assessment:workplan-current-rationale-constraints",
        "assessment:workplan-current-rationale-contract",
        "assessment:workplan-current-rationale-examples",
        "assessment:workplan-current-rationale-explanation",
        "assessment:workplan-current-rationale-purpose",
        "assessment:workplan-current-rationale-usage",
        "assessment:workplan-current-rationale-verification",
        "assessment:workplan-document-model-contract",
        "assessment:workplan-document-model-usage",
        "assessment:workplan-document-model-verification",
        "assessment:workplan-entity-history-constraints",
        "assessment:workplan-entity-history-contract",
        "assessment:workplan-entity-history-examples",
        "assessment:workplan-entity-history-purpose",
        "assessment:workplan-entity-history-usage",
        "assessment:workplan-entity-history-verification",
        "assessment:workplan-evidence-claims-constraints",
        "assessment:workplan-evidence-claims-contract",
        "assessment:workplan-evidence-claims-examples",
        "assessment:workplan-evidence-claims-explanation",
        "assessment:workplan-evidence-claims-purpose",
        "assessment:workplan-evidence-claims-usage",
        "assessment:workplan-evidence-claims-verification",
        "assessment:workplan-generated-docs-verification",
        "assessment:workplan-prose-assessment-constraints",
        "assessment:workplan-prose-assessment-contract",
        "assessment:workplan-prose-assessment-examples",
        "assessment:workplan-prose-assessment-purpose",
        "assessment:workplan-prose-assessment-usage",
        "assessment:workplan-prose-assessment-verification",
        "assessment:workplan-reconcile-constraints",
        "assessment:workplan-reconcile-contract",
        "assessment:workplan-reconcile-verification",
        "assessment:workplan-two-way-coverage-constraints",
        "assessment:workplan-two-way-coverage-contract",
        "assessment:workplan-two-way-coverage-examples",
        "assessment:workplan-two-way-coverage-purpose",
        "assessment:workplan-two-way-coverage-usage",
        "assessment:workplan-two-way-coverage-verification",
        "coverage:default",
        "evidence:gate-kills-a-test-past-its-timeout",
    ],
    generated: &["docs/capabilities/index.md"],
};

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

fn violation(path: &Path, message: impl std::fmt::Display) -> String {
    format!("{}: {message}", path.display())
}

/// The path as the `/`-separated text `raf-okf` takes.
///
/// # Panics
///
/// Panics when the path is not UTF-8; the index listing already refuses such
/// a path.
fn name(path: &Path) -> &str {
    match path.to_str() {
        Some(text) => text,
        None => panic!("path {} is not UTF-8", path.display()),
    }
}

/// The path as a tree path.
///
/// # Panics
///
/// Panics when the path is not a relative path of the tree.
fn rel(path: &Path) -> RelPath {
    match RelPath::new(name(path)) {
        Ok(rel) => rel,
        Err(error) => panic!(
            "tracked path {} is not a tree path: {error}",
            path.display()
        ),
    }
}

fn is_markdown(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "md")
}

/// The OKF type a document must declare because of where it lives.
fn expected_type(path: &Path) -> Option<DocType> {
    DocType::ALL
        .into_iter()
        .find(|doc_type| path.parent() == Some(Path::new(doc_type.directory())))
}

/// Whether the path is in, or below, the directory of a document type.
fn in_document_directory(path: &Path) -> bool {
    DocType::ALL
        .into_iter()
        .any(|doc_type| path.starts_with(doc_type.directory()))
}

// ---------------------------------------------------------------------------
// Inventory
// ---------------------------------------------------------------------------

/// The `id` and `state` of the document at `path`, or why it cannot be read.
fn identity(repo: &Repo, path: &str) -> Result<(String, Option<String>), String> {
    let text = repo.get(Path::new(path)).ok_or("the document is missing")?;
    let document = Document::parse(text).map_err(|error| match error.kind() {
        ErrorKind::MissingFence => "the document has no frontmatter".to_string(),
        _ => format!("unparseable document: {error}"),
    })?;
    let scalar = |key: &str| {
        document
            .frontmatter()
            .map()
            .get(key)
            .and_then(|entry| entry.value().as_scalar())
            .map(|scalar| scalar.text().to_string())
    };
    let id = scalar("id").ok_or("the document has no `id`")?;
    Ok((id, scalar("state")))
}

/// Compare the tree with the inventory: every listed document exists with the
/// stated `id` and `state`, every open capability has its change record, every
/// generated document exists, and no document exists that the inventory does
/// not list.
fn check_inventory(repo: &Repo, inventory: &Inventory) -> Vec<String> {
    let mut out = Vec::new();
    let mut listed: BTreeSet<String> = BTreeSet::new();
    let mut expect = |path: String, id: String, state: Option<&str>| {
        match identity(repo, &path) {
            Err(reason) => out.push(violation(Path::new(&path), reason)),
            Ok((found_id, found_state)) => {
                if found_id != id {
                    out.push(violation(
                        Path::new(&path),
                        format!("inventory expects id `{id}`, found `{found_id}`"),
                    ));
                }
                if let Some(state) = state
                    && found_state.as_deref() != Some(state)
                {
                    let found = match &found_state {
                        Some(found) => found.as_str(),
                        None => "<no state>",
                    };
                    out.push(violation(
                        Path::new(&path),
                        format!("inventory expects state `{state}`, found `{found}`"),
                    ));
                }
            }
        }
        listed.insert(path);
    };
    for (slug, state) in inventory.capabilities {
        expect(
            format!("docs/capabilities/{slug}.md"),
            format!("cap:{slug}"),
            Some(state),
        );
        if *state != "delivered" {
            expect(
                format!("work/changes/{slug}.md"),
                format!("change:{slug}"),
                None,
            );
        }
    }
    for slug in inventory.architecture {
        expect(
            format!("docs/architecture/{slug}.md"),
            format!("arch:{slug}"),
            None,
        );
    }
    let mut unknown_records = Vec::new();
    for id in inventory.records {
        let home = DocType::ALL
            .into_iter()
            .filter(|doc_type| doc_type.is_record())
            .find_map(|doc_type| {
                id.strip_prefix(doc_type.id_prefix())
                    .map(|slug| format!("{}/{slug}.md", doc_type.directory()))
            });
        match home {
            Some(path) => expect(path, (*id).to_string(), None),
            None => {
                unknown_records.push(format!("inventory: `{id}` is not the id of a record type"));
            }
        }
    }
    for path in inventory.generated {
        if !repo.contains_key(Path::new(path)) {
            out.push(violation(Path::new(path), "the document is missing"));
        }
        listed.insert((*path).to_string());
    }
    out.extend(unknown_records);
    for path in repo.keys() {
        let known = path.to_str().is_some_and(|p| listed.contains(p));
        if expected_type(path).is_some() && !known {
            out.push(violation(
                path,
                "the document is not in the inventory table",
            ));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Repository policy over tracked paths
// ---------------------------------------------------------------------------

/// A tracked symlink is a violation: a link to it is refused by the model, and
/// the repository keeps none.
fn symlink_violations(symlinks: &[PathBuf]) -> Vec<String> {
    symlinks
        .iter()
        .map(|path| violation(path, "a tracked symlink is not allowed"))
        .collect()
}

/// The document directories hold Markdown only: another file there would sit
/// unread beside the documents.
fn layout_violations(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .filter(|path| in_document_directory(path) && !is_markdown(path))
        .map(|path| {
            violation(
                path,
                "only Markdown documents belong in a document directory",
            )
        })
        .collect()
}

/// Every tracked path whose git mode is a symlink (`120000`), relative and sorted.
///
/// # Panics
///
/// Panics when `git` cannot run, exits unsuccessfully or lists an entry that
/// is not `<mode> <object> <stage>\t<path>` in UTF-8.
fn tracked_symlinks() -> Vec<PathBuf> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(env!("CARGO_MANIFEST_DIR"))
        .args(["ls-files", "-s", "-z"])
        .output()
        .unwrap_or_else(|e| panic!("cannot run `git ls-files -s -z`: {e}"));
    assert!(
        output.status.success(),
        "`git ls-files -s -z` failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut paths: Vec<PathBuf> = output
        .stdout
        .split(|b| *b == 0)
        .filter(|entry| !entry.is_empty())
        .filter_map(|entry| {
            // Each entry is `<mode> <object> <stage>\t<path>`.
            let text = match std::str::from_utf8(entry) {
                Ok(text) => text,
                Err(e) => panic!("`git ls-files -s -z` entry is not UTF-8: {e}"),
            };
            let Some((meta, path)) = text.split_once('\t') else {
                panic!("`git ls-files -s -z` entry has no tab: {text:?}");
            };
            meta.starts_with("120000 ").then(|| PathBuf::from(path))
        })
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

/// Whether the Markdown file is a page of the repository's own documentation:
/// at the top level or below `docs/` or `work/`. Other Markdown, such as the
/// document fixtures of a crate's tests, may declare any type.
fn is_documentation(path: &Path) -> bool {
    let mut components = path.components();
    let first = components.next();
    let top_level = components.next().is_none();
    top_level || first.is_some_and(|c| c.as_os_str() == "docs" || c.as_os_str() == "work")
}

/// Frontmatter belongs to the documents of the document directories.
/// Documentation outside them is plain Markdown to the model, so a page there
/// that opens with a frontmatter block, whatever its `type` (or none), would
/// never be validated: it is refused here instead. A page that declares a
/// document type is told where that type lives.
fn outside_violations(markdown: &Repo) -> Vec<String> {
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
        .collect()
}

// ---------------------------------------------------------------------------
// The model over the tracked files
// ---------------------------------------------------------------------------

/// The tracked files of a checkout: every tracked path, the tracked symlinks
/// among them, and the text of every tracked Markdown file.
struct Checkout {
    paths: Vec<PathBuf>,
    symlinks: Vec<PathBuf>,
    markdown: Repo,
}

/// Read the checkout from the git index, with the files read under `root`.
fn load_checkout(root: &Path) -> Checkout {
    let symlinks = tracked_symlinks();
    let paths = tracked_paths();
    let markdown = paths
        .iter()
        .filter(|path| is_markdown(path) && !symlinks.contains(path))
        .map(|path| {
            let bytes = read_tracked(root, path);
            match decode_text(&bytes) {
                Some(text) => (path.clone(), text.to_string()),
                None => panic!("{} is not UTF-8 text", path.display()),
            }
        })
        .collect();
    Checkout {
        paths,
        symlinks,
        markdown,
    }
}

/// The source index of the whole repository: every package below the root.
fn repository_source(root: &Path) -> Result<Workspace, Box<dyn Error>> {
    Ok(Workspace::load(&DiskTree::new(root)?, &["."])?)
}

/// What the model says about a checkout.
struct Verdict {
    /// Every finding of `check` and of `plain::check`.
    findings: Vec<Finding>,
    capabilities: usize,
    changes: usize,
    architecture: usize,
    /// The number of bindings of each capability, and how many resolved.
    bindings: BTreeMap<String, (usize, usize)>,
}

/// Judge a checkout: `Workplan::check` over the document directories, with
/// links judged against the tracked paths and symlinks, and `plain::check`
/// over the tracked Markdown outside them.
fn judge(checkout: &Checkout, source: &Workspace) -> Verdict {
    let tree = MemoryTree::try_from_iter(
        checkout
            .markdown
            .iter()
            .map(|(path, text)| (name(path), text.as_str())),
    )
    .unwrap_or_else(|error| panic!("the tracked Markdown does not form a tree: {error}"));
    let context = LinkContext::new(&tree, checkout.paths.iter().map(|path| rel(path)))
        .with_symlinks(checkout.symlinks.iter().map(|path| rel(path)));
    let plan = Workplan::load_with(&tree, source, &context);
    let mut findings = plan.check();
    for (path, text) in &checkout.markdown {
        if !in_document_directory(path) {
            findings.extend(plain::check(name(path), text, &context));
        }
    }
    let mut bindings: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for binding in plan.graph().bindings() {
        let entry = bindings
            .entry(binding.capability().to_string())
            .or_default();
        entry.0 += 1;
        if binding.resolution().is_ok() {
            entry.1 += 1;
        }
    }
    Verdict {
        findings,
        capabilities: plan.capabilities().len(),
        changes: plan.changes().len(),
        architecture: plan.architecture().len(),
        bindings,
    }
}

/// Every rule family over a checkout, as one list of violations: the model
/// (`Workplan::check` and `plain::check`), the refusal of tracked symlinks, the
/// Markdown-only document directories, the pages with frontmatter outside the
/// document directories, and the inventory table.
fn violations(checkout: &Checkout, source: &Workspace, inventory: &Inventory) -> Vec<String> {
    let mut out: Vec<String> = judge(checkout, source)
        .findings
        .iter()
        .map(ToString::to_string)
        .collect();
    out.extend(symlink_violations(&checkout.symlinks));
    out.extend(layout_violations(&checkout.paths));
    out.extend(outside_violations(&checkout.markdown));
    out.extend(check_inventory(&checkout.markdown, inventory));
    out
}

fn report(violations: &[String]) -> String {
    let mut report = String::new();
    for v in violations {
        let _ = writeln!(report, "  {v}");
    }
    report
}

// ---------------------------------------------------------------------------
// The repository
// ---------------------------------------------------------------------------

#[test]
fn repository_documents_satisfy_the_profile() -> Result<(), Box<dyn Error>> {
    let root = gate_root();
    let checkout = load_checkout(&root);
    let source = repository_source(&root)?;
    let found = violations(&checkout, &source, &INVENTORY);
    assert!(
        found.is_empty(),
        "{} profile violations:\n{}",
        found.len(),
        report(&found)
    );
    let verdict = judge(&checkout, &source);
    // A load that found nothing would pass the checks above, so require the
    // documents of every type, and every capability of the inventory.
    for (kind, count) in [
        ("Architecture", verdict.architecture),
        ("Capability", verdict.capabilities),
        ("Change", verdict.changes),
    ] {
        assert!(count >= 1, "no {kind} document found");
    }
    assert_eq!(
        verdict.capabilities,
        INVENTORY.capabilities.len(),
        "every capability of the inventory loads into the model"
    );
    Ok(())
}

#[test]
fn repository_documents_satisfy_the_document_model() {
    // The documented dev-dependency usage over the gate root. Inside the
    // commit hook the root is a checkout of the index without `.git`, and the
    // gate takes every file of it for a tracked path.
    Gate::workspace(gate_root()).assert_clean();
}

#[test]
fn repository_bindings_resolve_in_the_source_index() -> Result<(), Box<dyn Error>> {
    let root = gate_root();
    let workspace = repository_source(&root)?;
    // The roots are the whole repository: every package below it, the fuzz
    // harness included.
    let packages: BTreeSet<&str> = workspace
        .packages()
        .iter()
        .map(|package| package.name.as_str())
        .collect();
    assert_eq!(
        packages,
        BTreeSet::from([
            "cargo-workplan",
            "raf-okf",
            "raf-source",
            "raf-source-fuzz",
            "raf-workplan",
            "rust-agentic-foundations",
        ])
    );
    // A load that found nothing would pass the check below, so require known
    // symbols first.
    for symbol in [
        "raf_okf::Document",
        "raf_source::Workspace",
        "raf_workplan::Workplan",
    ] {
        workspace.resolve(symbol)?;
    }
    let verdict = judge(&load_checkout(&root), &workspace);
    let unresolved: Vec<String> = verdict
        .findings
        .iter()
        .filter(|finding| finding.kind.starts_with("binding-"))
        .map(ToString::to_string)
        .collect();
    assert!(
        unresolved.is_empty(),
        "{} unresolved bindings:\n{}",
        unresolved.len(),
        report(&unresolved)
    );
    // And require that bindings were judged at all: the delivered
    // capabilities bind code, and every binding resolved.
    for slug in ["cap:okf-profile", "cap:source-index"] {
        let (bound, resolved) = verdict.bindings.get(slug).copied().unwrap_or_default();
        assert!(bound >= 1, "{slug} binds nothing");
        assert_eq!(bound, resolved, "{slug} has an unresolved binding");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The checker over small checkouts: every part of the wiring has a negative
// ---------------------------------------------------------------------------

mod checkouts {
    use super::{Checkout, Inventory, PathBuf, Repo, Workspace, violations};
    use raf_source::MemoryTree;

    /// A one-package source workspace for the checkouts to bind to.
    pub(super) fn source() -> Workspace {
        let tree = MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
            ),
            ("src/lib.rs", "pub fn real() {}\n"),
        ]);
        Workspace::load(&tree, &["."]).expect("the demo workspace loads")
    }

    pub(super) const OVERVIEW: &str = "docs/architecture/overview.md";
    pub(super) const README: &str = "README.md";
    pub(super) const TEMPLATE: &str = ".github/PULL_REQUEST_TEMPLATE.md";

    pub(super) const LAYERS: &str = "docs/architecture/layers.md";

    /// A checkout of a few files: two architecture pages, the README and a
    /// template that link to each other, and a manifest that is not Markdown.
    pub(super) fn files() -> Vec<(&'static str, String)> {
        vec![
            (
                OVERVIEW,
                "---\ntype: Architecture\nid: arch:overview\ntitle: Overview\n\
                 description: How it fits.\n---\n\n# Overview\n\nBody text. See [the layers](layers.md).\n"
                    .to_string(),
            ),
            (
                LAYERS,
                "---\ntype: Architecture\nid: arch:layers\ntitle: Layers\n\
                 description: The layers.\n---\n\n# Layers\n\nSee [the overview](overview.md).\n"
                    .to_string(),
            ),
            (
                README,
                "# Readme\n\nSee [overview](docs/architecture/overview.md).\n".to_string(),
            ),
            (
                TEMPLATE,
                "# Template\n\nSee [readme](../README.md).\n".to_string(),
            ),
        ]
    }

    /// The checkout of `files` with these tracked symlinks and these tracked
    /// files that are not Markdown.
    pub(super) fn checkout(
        files: &[(&str, String)],
        symlinks: &[&str],
        others: &[&str],
    ) -> Checkout {
        let markdown: Repo = files
            .iter()
            .map(|(path, text)| (PathBuf::from(path), text.clone()))
            .collect();
        let mut paths: Vec<PathBuf> = markdown.keys().cloned().collect();
        paths.extend(others.iter().map(PathBuf::from));
        paths.extend(symlinks.iter().map(PathBuf::from));
        Checkout {
            paths,
            symlinks: symlinks.iter().map(PathBuf::from).collect(),
            markdown,
        }
    }

    /// The checkout of exactly the pages of `repo`.
    pub(super) fn of_repo(repo: &Repo) -> Checkout {
        Checkout {
            paths: repo.keys().cloned().collect(),
            symlinks: Vec::new(),
            markdown: repo.clone(),
        }
    }

    /// The inventory of the pages of `files`: the two architecture pages.
    pub(super) const INVENTORY: Inventory = Inventory {
        capabilities: &[],
        architecture: &["layers", "overview"],
        records: &[],
        generated: &[],
    };

    /// Every violation of the checkout of `files` with these tracked symlinks
    /// and these tracked files that are not Markdown.
    pub(super) fn found(
        files: &[(&str, String)],
        symlinks: &[&str],
        others: &[&str],
    ) -> Vec<String> {
        violations(&checkout(files, symlinks, others), &source(), &INVENTORY)
    }

    /// The files with `from` replaced by `to` in `path`.
    pub(super) fn mutated(path: &str, from: &str, to: &str) -> Vec<(&'static str, String)> {
        let mut files = files();
        let (_, text) = files
            .iter_mut()
            .find(|(candidate, _)| *candidate == path)
            .unwrap_or_else(|| panic!("the checkout has no {path}"));
        assert!(text.contains(from), "{path} has no `{from}`");
        *text = text.replacen(from, to, 1);
        files
    }
}

use checkouts::{OVERVIEW, README, TEMPLATE, files, found, mutated};

/// The violations that are about `file` and carry the finding kind `kind`.
fn about(found: &[String], file: &str, kind: &str) -> usize {
    found
        .iter()
        .filter(|line| {
            line.starts_with(&format!("{file}: ")) && line.contains(&format!("[{kind}]"))
        })
        .count()
}

#[test]
fn the_checker_accepts_a_valid_checkout() {
    assert_eq!(found(&files(), &[], &[]), Vec::<String>::new());
    let verdict = judge(
        &checkouts::checkout(&files(), &[], &[]),
        &checkouts::source(),
    );
    assert_eq!(verdict.architecture, 2);
    assert_eq!(verdict.capabilities, 0);
}

#[test]
fn the_checker_judges_the_documents_through_the_model() {
    let broken = mutated(OVERVIEW, "title: Overview", "title: Overview\nextra: 1");
    let found = found(&broken, &[], &[]);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(about(&found, OVERVIEW, "okf-unknown-key"), 1, "{found:#?}");
}

#[test]
fn the_checker_judges_markdown_outside_the_document_directories() {
    let broken = mutated(TEMPLATE, "../README.md", "../NOPE.md");
    let found_broken = found(&broken, &[], &[]);
    assert_eq!(found_broken.len(), 1, "{found_broken:#?}");
    assert_eq!(about(&found_broken, TEMPLATE, "link-broken"), 1);
    let split = mutated(README, "[overview](", "[overview](\n");
    let found_split = found(&split, &[], &[]);
    assert_eq!(found_split.len(), 1, "{found_split:#?}");
    assert_eq!(about(&found_split, README, "okf-split-link"), 1);
}

#[test]
fn the_checker_judges_links_against_the_tracked_paths_and_symlinks() {
    let linked = mutated(README, "# Readme\n", "# Readme\n\n![x](docs/alias.svg)\n");
    // Tracked as a regular file: the link resolves.
    assert_eq!(
        found(&linked, &[], &["docs/alias.svg"]),
        Vec::<String>::new()
    );
    // Tracked as a symlink: a link must not lead to it. The symlink itself is
    // refused too.
    let as_symlink = found(&linked, &["docs/alias.svg"], &[]);
    assert_eq!(
        about(&as_symlink, README, "link-symlink"),
        1,
        "{as_symlink:#?}"
    );
    assert_eq!(as_symlink.len(), 2, "{as_symlink:#?}");
    // Not tracked at all: broken, whatever is on disk.
    let untracked = found(&linked, &[], &[]);
    assert_eq!(
        about(&untracked, README, "link-broken"),
        1,
        "{untracked:#?}"
    );
    assert_eq!(untracked.len(), 1, "{untracked:#?}");
}

#[test]
fn rejects_a_tracked_symlink_that_nothing_links_to() {
    let found = found(&files(), &["NOTES.md"], &[]);
    assert_eq!(found, ["NOTES.md: a tracked symlink is not allowed"]);
}

#[test]
fn rejects_non_markdown_file_in_a_document_directory() {
    for path in [
        "docs/capabilities/notes.txt",
        "docs/capabilities/ALPHA.MD",
        "docs/architecture/diagram.svg",
        "work/changes/draft.yaml",
        "docs/capabilities/sub/data.json",
        "work/evidence/run.log",
        "work/assessments/notes.txt",
        "work/coverage/default.toml",
    ] {
        let found = found(&files(), &[], &[path]);
        assert_eq!(
            found,
            [format!(
                "{path}: only Markdown documents belong in a document directory"
            )]
        );
    }
    // Markdown, and files elsewhere, are not judged by this rule.
    let elsewhere = [
        "docs/brand/logo.svg",
        "work/README.txt",
        "Cargo.toml",
        "crates/tool/src/lib.rs",
    ];
    assert_eq!(found(&files(), &[], &elsewhere), Vec::<String>::new());
}

#[test]
fn rejects_a_typed_document_outside_the_document_directories() {
    let page = |frontmatter: &str| format!("---\n{frontmatter}\n---\n\n# Page\n");
    let typed = |doc_type: &str| page(&format!("type: {doc_type}\nname: x"));
    let with = |path: &'static str, text: String| {
        let mut all = files();
        all.push((path, text));
        found(&all, &[], &[])
    };
    // A page with a document type is told where that type lives.
    for (path, doc_type, home) in [
        ("docs/other/stray.md", "a Capability", "docs/capabilities"),
        ("docs/stray.md", "a Change", "work/changes"),
        ("work/stray.md", "an Architecture", "docs/architecture"),
        ("stray.md", "a Capability", "docs/capabilities"),
        ("work/evidence.md", "an Evidence", "work/evidence"),
        ("docs/coverage.md", "a Coverage", "work/coverage"),
    ] {
        let name = doc_type.split_once(' ').map_or(doc_type, |(_, name)| name);
        assert_eq!(
            with(path, typed(name)),
            [format!(
                "{path}: `type`: {doc_type} document belongs directly in {home}"
            )]
        );
    }
    // Any other frontmatter on such a page is refused as well: an unknown
    // type, no type, or keys that are no document's.
    let home = "frontmatter belongs only in the documents of docs/capabilities, work/changes, docs/architecture, work/evidence, work/assessments, work/coverage; this page is outside them";
    for (path, text) in [
        ("docs/notes.md", page("title: Notes")),
        ("NOTES.md", typed("Bug")),
        ("work/notes.md", page("name: Notes\nabout: x")),
        ("README.md", page("foo: bar")),
    ] {
        let mut all = files();
        all.retain(|(candidate, _)| *candidate != path);
        all.push((path, text));
        let found = found(&all, &[], &[]);
        assert!(
            found.contains(&format!("{path}: {home}")),
            "{path}: {found:#?}"
        );
    }
    // A page without frontmatter, a page outside `docs/` and `work/` (an issue
    // template, the typed fixture of a crate) and a typed page in its own
    // directory are not judged by this rule.
    let mut all = files();
    all.push((
        ".github/ISSUE_TEMPLATE/bug.md",
        page("name: Bug\nabout: A bug"),
    ));
    all.push((
        "crates/tool/tests/fixtures/capability.md",
        typed("Capability"),
    ));
    assert_eq!(found(&all, &[], &[]), Vec::<String>::new());
}

// --- capability inventory ---------------------------------------------------

const OVERVIEW_PAGE: &str = "docs/architecture/overview.md";
const ALPHA: &str = "docs/capabilities/alpha.md";
const BETA: &str = "docs/capabilities/beta.md";
const INDEX: &str = "docs/capabilities/index.md";

const FIXTURE_INVENTORY: Inventory = Inventory {
    capabilities: &[
        ("alpha", "designed"),
        ("beta", "considered"),
        ("gamma", "delivered"),
    ],
    architecture: &["overview"],
    records: &["evidence:bound", "assessment:review", "coverage:default"],
    generated: &[INDEX],
};

/// The records of `FIXTURE_INVENTORY`, by path, type and id.
const FIXTURE_RECORDS: [(&str, &str, &str); 3] = [
    ("work/evidence/bound.md", "Evidence", "evidence:bound"),
    (
        "work/assessments/review.md",
        "Assessment",
        "assessment:review",
    ),
    ("work/coverage/default.md", "Coverage", "coverage:default"),
];

/// A document with only the frontmatter the inventory reads.
fn page(doc_type: &str, id: &str, state: Option<&str>) -> String {
    let state = state.map_or_else(String::new, |state| format!("state: {state}\n"));
    format!("---\ntype: {doc_type}\nid: {id}\n{state}---\n\n# Page\n")
}

/// A tree whose documents are exactly `FIXTURE_INVENTORY`.
fn inventory_repo() -> Repo {
    let mut repo = Repo::new();
    for (slug, state) in [
        ("alpha", "designed"),
        ("beta", "considered"),
        ("gamma", "delivered"),
    ] {
        repo.insert(
            PathBuf::from(format!("docs/capabilities/{slug}.md")),
            page("Capability", &format!("cap:{slug}"), Some(state)),
        );
    }
    for slug in ["alpha", "beta"] {
        repo.insert(
            PathBuf::from(format!("work/changes/{slug}.md")),
            page("Change", &format!("change:{slug}"), None),
        );
    }
    repo.insert(
        PathBuf::from(OVERVIEW_PAGE),
        page("Architecture", "arch:overview", None),
    );
    for (path, doc_type, id) in FIXTURE_RECORDS {
        repo.insert(PathBuf::from(path), page(doc_type, id, None));
    }
    repo.insert(
        PathBuf::from(INDEX),
        "<!-- raf:generated:catalogue -->\n<!-- raf:generated:end -->\n".to_string(),
    );
    repo
}

/// Every violation of the checkout of exactly the pages of `repo`, judged
/// against the fixture inventory.
fn inventory_found(repo: &Repo) -> Vec<String> {
    violations(
        &checkouts::of_repo(repo),
        &checkouts::source(),
        &FIXTURE_INVENTORY,
    )
}

fn assert_hit(violations: &[String], needle: &str) {
    assert!(
        violations.iter().any(|v| v.contains(needle)),
        "expected a violation containing `{needle}`, got: {violations:#?}"
    );
}

#[test]
fn inventory_accepts_a_matching_tree() {
    // The minimal pages are no valid documents, so the model has findings of
    // its own; the inventory has none.
    let found: Vec<String> = inventory_found(&inventory_repo())
        .into_iter()
        .filter(|line| line.contains("inventory") || line.contains("the document is missing"))
        .collect();
    assert_eq!(found, Vec::<String>::new());
}

#[test]
fn inventory_table_has_the_documented_shape() {
    let slugs: BTreeSet<&str> = INVENTORY.capabilities.iter().map(|(s, _)| *s).collect();
    assert_eq!(slugs.len(), 21, "21 capability slugs, each listed once");
    assert!(slugs.iter().all(|s| {
        !s.is_empty()
            && s.split('-').all(|part| {
                !part.is_empty()
                    && part
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            })
    }));
    let count = |wanted: &str| {
        INVENTORY
            .capabilities
            .iter()
            .filter(|(_, state)| *state == wanted)
            .count()
    };
    assert_eq!(count("designed"), 9);
    assert_eq!(count("implementing"), 4);
    assert_eq!(count("delivered"), 8);
    assert_eq!(count("considered"), 0);
    assert_eq!(INVENTORY.architecture.len(), 2);
    assert!(
        INVENTORY.records.windows(2).all(|pair| pair[0] < pair[1]),
        "the records are listed sorted, each once"
    );
    let records = |prefix: &str| {
        INVENTORY
            .records
            .iter()
            .filter(|id| id.starts_with(prefix))
            .count()
    };
    assert_eq!(records("assessment:"), 54);
    assert_eq!(records("coverage:"), 1);
    assert_eq!(records("evidence:"), 1);
    assert_eq!(INVENTORY.records.len(), 56);
    assert_eq!(INVENTORY.generated, ["docs/capabilities/index.md"]);
}

#[test]
fn inventory_rejects_a_missing_capability() {
    let mut repo = inventory_repo();
    repo.remove(Path::new(BETA));
    let found = inventory_found(&repo);
    assert_hit(&found, "docs/capabilities/beta.md: the document is missing");
}

#[test]
fn inventory_rejects_a_missing_change_record() {
    let mut repo = inventory_repo();
    repo.remove(Path::new("work/changes/alpha.md"));
    let found = inventory_found(&repo);
    assert_hit(&found, "work/changes/alpha.md: the document is missing");
}

#[test]
fn inventory_rejects_a_missing_architecture_document() {
    let mut repo = inventory_repo();
    repo.remove(Path::new(OVERVIEW_PAGE));
    let found = inventory_found(&repo);
    assert_hit(
        &found,
        "docs/architecture/overview.md: the document is missing",
    );
}

#[test]
fn inventory_rejects_a_missing_capability_index() {
    let mut repo = inventory_repo();
    repo.remove(Path::new(INDEX));
    let found = inventory_found(&repo);
    assert_hit(
        &found,
        "docs/capabilities/index.md: the document is missing",
    );
}

#[test]
fn inventory_rejects_a_state_change() {
    let mut repo = inventory_repo();
    let alpha = repo.get_mut(Path::new(ALPHA)).expect("fixture alpha");
    *alpha = alpha.replace("state: designed", "state: implementing");
    let found = inventory_found(&repo);
    assert_hit(
        &found,
        "inventory expects state `designed`, found `implementing`",
    );
}

#[test]
fn inventory_rejects_an_id_change() {
    let mut repo = inventory_repo();
    let overview = repo
        .get_mut(Path::new(OVERVIEW_PAGE))
        .expect("fixture overview");
    *overview = overview.replace("arch:overview", "arch:other");
    let found = inventory_found(&repo);
    assert_hit(
        &found,
        "inventory expects id `arch:overview`, found `arch:other`",
    );
}

#[test]
fn inventory_rejects_a_document_the_table_does_not_list() {
    let mut repo = inventory_repo();
    repo.insert(
        PathBuf::from("docs/capabilities/extra.md"),
        page("Capability", "cap:extra", Some("designed")),
    );
    let found = inventory_found(&repo);
    assert_hit(
        &found,
        "docs/capabilities/extra.md: the document is not in the inventory table",
    );
}

#[test]
fn inventory_rejects_a_missing_or_unlisted_record() {
    for (path, _, _) in FIXTURE_RECORDS {
        let mut repo = inventory_repo();
        repo.remove(Path::new(path));
        let found = inventory_found(&repo);
        assert_hit(&found, &format!("{path}: the document is missing"));
    }
    let mut repo = inventory_repo();
    repo.insert(
        PathBuf::from("work/evidence/extra.md"),
        page("Evidence", "evidence:extra", None),
    );
    let found = inventory_found(&repo);
    assert_hit(
        &found,
        "work/evidence/extra.md: the document is not in the inventory table",
    );
    let mut repo = inventory_repo();
    let coverage = repo
        .get_mut(Path::new("work/coverage/default.md"))
        .expect("fixture coverage");
    *coverage = coverage.replace("coverage:default", "coverage:other");
    assert_hit(
        &inventory_found(&repo),
        "inventory expects id `coverage:default`, found `coverage:other`",
    );
}

#[test]
fn inventory_refuses_a_record_id_of_no_record_type() {
    let inventory = Inventory {
        records: &["cap:alpha"],
        ..FIXTURE_INVENTORY
    };
    let found = violations(
        &checkouts::of_repo(&inventory_repo()),
        &checkouts::source(),
        &inventory,
    );
    assert_hit(
        &found,
        "inventory: `cap:alpha` is not the id of a record type",
    );
}

#[test]
fn inventory_rejects_an_index_the_table_does_not_list() {
    for path in ["docs/architecture/index.md", "work/changes/index.md"] {
        let mut repo = inventory_repo();
        repo.insert(PathBuf::from(path), "# Index\n".to_string());
        let found = inventory_found(&repo);
        assert_hit(
            &found,
            &format!("{path}: the document is not in the inventory table"),
        );
    }
}

// --- reading files ----------------------------------------------------------

/// A scratch directory removed when dropped, named by process id, a
/// nanosecond timestamp and a counter so no two runs share a path.
#[cfg(unix)]
struct Scratch(PathBuf);

#[cfg(unix)]
impl Scratch {
    fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!(
            "raf-docs-contract-{}-{nanos}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&dir).expect("scratch directory");
        Self(dir)
    }
}

#[cfg(unix)]
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(unix)]
#[test]
fn refuses_non_regular_files_instead_of_blocking() {
    use std::process::Command;
    use std::sync::mpsc;
    use std::time::Duration;

    let scratch = Scratch::new();
    let dir = scratch.0.clone();
    let made = Command::new("mkfifo")
        .arg(dir.join("trap.md"))
        .status()
        .expect("mkfifo runs");
    assert!(made.success(), "mkfifo failed");
    std::fs::write(dir.join("plain.md"), "text").expect("plain file");
    std::os::unix::fs::symlink(dir.join("plain.md"), dir.join("link.md")).expect("symlink");

    let (sender, receiver) = mpsc::channel();
    let root = dir.clone();
    std::thread::spawn(move || {
        let _ = sender.send(read_regular(&root, Path::new("trap.md")));
    });
    let outcome = receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("reading a FIFO must not block");
    assert!(
        outcome
            .as_ref()
            .is_err_and(|e| e.contains("not a regular file")),
        "{outcome:?}"
    );
    let link = read_regular(&dir, Path::new("link.md"));
    assert!(
        link.as_ref().is_err_and(
            |e| e.contains("link.md") && e.contains("symlinks are not allowed in the tree")
        ),
        "{link:?}"
    );
    assert_eq!(
        read_regular(&dir, Path::new("plain.md")),
        Ok(b"text".to_vec())
    );
}

#[cfg(unix)]
#[test]
fn refuses_a_file_swapped_in_after_inspection() {
    let scratch = Scratch::new();
    let dir = &scratch.0;
    std::fs::write(dir.join("first.md"), "one").expect("first file");
    std::fs::write(dir.join("second.md"), "two").expect("second file");
    let inspected = std::fs::symlink_metadata(dir.join("first.md")).expect("stat first");
    let opened = std::fs::File::open(dir.join("second.md"))
        .and_then(|f| f.metadata())
        .expect("stat second");
    let verdict = support::check_handle("first.md", &inspected, &opened);
    assert!(
        verdict.is_err_and(|e| e.contains("changed while it was being opened")),
        "a different inode must be refused"
    );
    let same = std::fs::File::open(dir.join("first.md"))
        .and_then(|f| f.metadata())
        .expect("stat first handle");
    assert_eq!(support::check_handle("first.md", &inspected, &same), Ok(()));
}

// --- Usage and Examples are the crate documentation tests --------------------

/// A fenced `rust` block: its info string and its code.
#[derive(Debug, PartialEq, Eq)]
struct RustBlock {
    info: String,
    code: String,
}

/// The lines between the fence lines of a block, when a closing fence ends it.
fn fenced_code(block: &FencedBlock) -> Result<String, String> {
    if block.is_closed() {
        Ok(block.content().to_string())
    } else {
        Err(format!(
            "the block at line {} is never closed",
            block.span().lines.start
        ))
    }
}

/// The `rust` blocks of a Markdown text, in order: all of them, or only those
/// under the second-level sections named in `within`.
fn rust_blocks(text: &str, within: Option<&[&str]>) -> Result<Vec<RustBlock>, String> {
    let body = Body::parse(text).map_err(|e| format!("the text does not parse: {e}"))?;
    let mut blocks = Vec::new();
    for fenced in body.fenced_blocks() {
        let start = fenced.span().bytes.start;
        let wanted = within.is_none_or(|names| {
            body.sections().iter().any(|section| {
                names.contains(&section.heading())
                    && section.content().bytes.start <= start
                    && start < section.content().bytes.end
            })
        });
        let info = fenced.info();
        if wanted && (info == "rust" || info.starts_with("rust,")) {
            blocks.push(RustBlock {
                info: info.to_string(),
                code: fenced_code(fenced)?,
            });
        }
    }
    Ok(blocks)
}

/// The crate-level documentation of a library source as Markdown.
fn crate_docs(library: &str) -> String {
    let mut docs = String::new();
    for line in library.lines().filter_map(|l| l.strip_prefix("//!")) {
        docs.push_str(line.strip_prefix(' ').map_or(line, |rest| rest));
        docs.push('\n');
    }
    docs
}

/// The 1-based line where two texts first differ; when one is a prefix of the
/// other, the line after the shorter one.
fn first_difference(left: &str, right: &str) -> usize {
    match left.lines().zip(right.lines()).position(|(a, b)| a != b) {
        Some(at) => at + 1,
        None => left.lines().count().min(right.lines().count()) + 1,
    }
}

/// Why the Usage and Examples blocks of a capability document are not the
/// leading crate-level documentation tests of its library, or nothing when they
/// are.
fn doctest_problems(document: &str, library: &str) -> Result<Vec<String>, String> {
    let documented = rust_blocks(document, Some(&["Usage", "Examples"]))?;
    let doctests = rust_blocks(&crate_docs(library), None)?;
    let mut problems = Vec::new();
    if documented.is_empty() {
        problems.push("the document has no `rust` block under Usage or Examples".to_string());
    }
    for (index, block) in documented.iter().enumerate() {
        let number = index + 1;
        let Some(doctest) = doctests.get(index) else {
            problems.push(format!("block {number} has no crate doctest to equal"));
            continue;
        };
        if block.info != doctest.info {
            problems.push(format!(
                "block {number} is `{}` in the document and `{}` in the crate",
                block.info, doctest.info
            ));
        }
        if block.code != doctest.code {
            let line = first_difference(&block.code, &doctest.code);
            problems.push(format!(
                "block {number} differs from the crate doctest at line {line} of the block"
            ));
        }
        if block.info == "rust" && block.code.contains("std::fs") {
            problems.push(format!("block {number} is run and uses `std::fs`"));
        }
        if block.info.contains("no_run") && !document.contains("compiled and not run") {
            problems.push(format!(
                "block {number} is not run, and the document does not say why"
            ));
        }
    }
    Ok(problems)
}

fn tracked_text(root: &Path, path: &str) -> String {
    let bytes = read_tracked(root, Path::new(path));
    match decode_text(&bytes) {
        Some(text) => text.into_owned(),
        None => panic!("{path} is not text"),
    }
}

const DOCTEST_PAIRS: [(&str, &str); 2] = [
    (
        "docs/capabilities/okf-profile.md",
        "crates/raf-okf/src/lib.rs",
    ),
    (
        "docs/capabilities/source-index.md",
        "crates/raf-source/src/lib.rs",
    ),
];

#[test]
fn usage_and_examples_blocks_are_the_crate_doctests() -> Result<(), String> {
    let root = gate_root();
    for (document, library) in DOCTEST_PAIRS {
        let problems = doctest_problems(
            &tracked_text(&root, document),
            &tracked_text(&root, library),
        )?;
        assert!(
            problems.is_empty(),
            "{document} against {library}: {problems:#?}"
        );
    }
    Ok(())
}

#[test]
fn a_block_that_differs_by_one_line_is_reported() -> Result<(), String> {
    let root = gate_root();
    for (document, library) in DOCTEST_PAIRS {
        let document = tracked_text(&root, document);
        let library = tracked_text(&root, library);
        let changed = library.replacen("//! use ", "//! use  ", 1);
        assert_ne!(changed, library);
        let problems = doctest_problems(&document, &changed)?;
        assert_eq!(problems.len(), 1, "{problems:#?}");
        assert!(
            problems[0].contains("block 1 differs from the crate doctest"),
            "{problems:#?}"
        );
    }
    Ok(())
}

#[test]
fn doctest_comparison_fixtures() -> Result<(), String> {
    let document = "## Contract\n\n```rust\nignored();\n```\n\n## Usage\n\n```rust\nlet a = 1;\nlet b = 2;\n```\n";
    let library = "//! Words.\n//!\n//! ```rust\n//! let a = 1;\n//! let b = 2;\n//! ```\n";
    assert_eq!(doctest_problems(document, library)?, Vec::<String>::new());
    let one_line = library.replace("let b = 2;", "let b = 3;");
    assert_eq!(
        doctest_problems(document, &one_line)?,
        ["block 1 differs from the crate doctest at line 2 of the block"]
    );
    let other_info = library.replace("```rust", "```rust,no_run");
    assert_eq!(
        doctest_problems(document, &other_info)?,
        ["block 1 is `rust` in the document and `rust,no_run` in the crate"]
    );
    assert_eq!(
        doctest_problems(document, "//! No blocks here.\n")?,
        ["block 1 has no crate doctest to equal"]
    );
    assert_eq!(
        doctest_problems("## Usage\n\nNo block.\n", library)?,
        ["the document has no `rust` block under Usage or Examples"]
    );
    let reads_files = document.replace("let a = 1;", "std::fs::read(\"x\");");
    let reads_files_library = library.replace("let a = 1;", "std::fs::read(\"x\");");
    assert_eq!(
        doctest_problems(&reads_files, &reads_files_library)?,
        ["block 1 is run and uses `std::fs`"]
    );
    let unexplained = document.replace("```rust\nlet a", "```rust,no_run\nlet a");
    let unexplained_library = library.replace("```rust", "```rust,no_run");
    assert_eq!(
        doctest_problems(&unexplained, &unexplained_library)?,
        ["block 1 is not run, and the document does not say why"]
    );
    let explained = format!("{unexplained}\nThe block is compiled and not run.\n");
    assert_eq!(
        doctest_problems(&explained, &unexplained_library)?,
        Vec::<String>::new()
    );
    Ok(())
}

#[test]
fn a_tilde_fenced_usage_block_is_accepted_and_an_unclosed_block_is_refused() -> Result<(), String> {
    let library = "//! Words.\n//!\n//! ```rust\n//! let a = 1;\n//! let b = 2;\n//! ```\n";
    let tilde = "## Usage\n\n~~~rust\nlet a = 1;\nlet b = 2;\n~~~\n";
    assert_eq!(doctest_problems(tilde, library)?, Vec::<String>::new());
    // A longer closing fence and trailing spaces still close the block.
    let longer = "## Usage\n\n~~~rust\nlet a = 1;\nlet b = 2;\n~~~~~ \n";
    assert_eq!(doctest_problems(longer, library)?, Vec::<String>::new());
    // A fence of the other character does not close it: the block is open to
    // the end of the text, which is refused rather than read as complete.
    let mixed = "## Usage\n\n~~~rust\nlet a = 1;\nlet b = 2;\n```\n";
    assert_eq!(
        doctest_problems(mixed, library),
        Err("the block at line 3 is never closed".to_string())
    );
    let open = "## Usage\n\n```rust\nlet a = 1;\nlet b = 2;\n";
    assert_eq!(
        doctest_problems(open, library),
        Err("the block at line 3 is never closed".to_string())
    );
    Ok(())
}
