// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Document-contract test for this repository.
//!
//! It enforces the RAF profile of OKF documents, stated in
//! `docs/architecture/okf-profile.md`, over the committed Markdown. The reading
//! is done by `raf-okf`: every capability, change and architecture document goes
//! through `Document::parse` and `Profile::validate`, the generated capability
//! index through `Index::parse`, and every other Markdown file through
//! `Body::parse`. Bound symbols are resolved by `raf-source` against the crates
//! of this repository.
//!
//! What stays here is what no single document can decide: the cross-document
//! rules, link resolution against the tracked file list, the refusal of tracked
//! symlinks, the directory layout, the capability inventory and the
//! banned-wording scan. The checker is a pure function over an in-memory
//! repository (relative path to contents), so every rule has a negative test
//! that breaks exactly one thing in a valid fixture.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use raf_okf::{
    Bindings, Body, DocType, Document, ErrorKind, FencedBlock, Index, LinkTarget, Outcome,
    ParseError, Profile, RafDocument, Rule, State, is_capability_index,
};
use raf_source::{DiskTree, MemoryTree, Workspace};

mod support;

use support::{banned_in_line, decode_text, gate_root, read_regular, read_tracked, tracked_paths};

/// A repository held in memory: relative path to file contents.
type Repo = BTreeMap<PathBuf, String>;

/// The text recorded for a tracked symlink. A NUL byte cannot occur in decoded
/// text, so it never collides with a real file; a link whose target holds it is
/// refused.
const SYMLINK_TEXT: &str = "\0symlink";

/// Unfinished-work tokens, for the fixtures that plant one. The text is
/// assembled with `concat!` so this list never contains a token as one
/// literal. `raf-okf` owns the rule that finds them.
const PLACEHOLDER_TOKENS: [&str; 5] = [
    concat!("t", "bd"),
    concat!("to", "do"),
    concat!("fix", "me"),
    concat!("x", "xx"),
    concat!("to be ", "written"),
];

/// The capability inventory.
///
/// This table is the capability inventory: the real tree must hold exactly
/// these capability documents, each with the stated `state`, exactly these
/// architecture documents, and an open change record named after every
/// capability whose state is not `delivered`. Changing a state or adding a
/// capability edits this table in the same commit. `raf_workplan::Gate`
/// replaces it.
struct Inventory {
    /// Capability slug and the state its document declares.
    capabilities: &'static [(&'static str, &'static str)],
    /// Architecture slugs.
    architecture: &'static [&'static str],
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
        ("workplan-cli", "designed"),
        ("workplan-complete-gate", "designed"),
        ("workplan-current-rationale", "considered"),
        ("workplan-document-model", "designed"),
        ("workplan-entity-history", "considered"),
        ("workplan-evidence-claims", "considered"),
        ("workplan-generated-docs", "designed"),
        ("workplan-prose-assessment", "considered"),
        ("workplan-reconcile", "designed"),
        ("workplan-two-way-coverage", "considered"),
    ],
    architecture: &["normal-form", "okf-profile"],
};

// ---------------------------------------------------------------------------
// Paths and placeholders
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

fn is_markdown(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "md")
}

fn is_scanned(path: &Path) -> bool {
    let mut components = path.components();
    let first = components.next();
    let top_level = components.next().is_none();
    is_markdown(path)
        && (top_level || first.is_some_and(|c| c.as_os_str() == "docs" || c.as_os_str() == "work"))
}

/// The OKF type a document must declare because of where it lives.
fn expected_type(path: &Path) -> Option<DocType> {
    DocType::ALL
        .into_iter()
        .find(|doc_type| path.parent() == Some(Path::new(doc_type.directory())))
}

fn scan_banned(path: &Path, text: &str, out: &mut Vec<String>) {
    for (index, line) in text.lines().enumerate() {
        if let Some(kind) = banned_in_line(line) {
            out.push(violation(
                path,
                format!("line {}: {kind} is banned", index + 1),
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// What the cross-document checks need from each document
// ---------------------------------------------------------------------------

struct CapabilityFacts {
    path: PathBuf,
    id: String,
    state: State,
    depends: Vec<String>,
    bindings: Bindings,
}

struct ChangeFacts {
    path: PathBuf,
    slug: String,
    changes: Vec<String>,
    /// The `affects` entries, each as `cap:<slug>#<section>`.
    affects: Vec<String>,
    /// The slugs that `moved:change:<slug>` outcomes name.
    moved: Vec<String>,
}

/// A well-formed PROPOSED block.
struct Marker {
    path: PathBuf,
    slug: String,
    /// The slug of the second-level section that holds the block.
    section: Option<String>,
}

#[derive(Default)]
struct Found {
    capabilities: Vec<CapabilityFacts>,
    changes: Vec<ChangeFacts>,
    markers: Vec<Marker>,
    architecture: usize,
}

impl Found {
    /// Record a document that satisfies the profile.
    fn record(&mut self, path: &Path, typed: RafDocument) {
        match typed {
            RafDocument::Capability(capability) => self.capabilities.push(CapabilityFacts {
                path: path.to_path_buf(),
                id: capability.header.id,
                state: capability.state,
                depends: capability.depends,
                bindings: capability.bindings,
            }),
            RafDocument::Change(change) => {
                let mut moved = Vec::new();
                for item in &change.items {
                    match &item.outcome {
                        Some(Outcome::Moved(slug)) => moved.push(slug.clone()),
                        Some(Outcome::Implemented | Outcome::Dropped) | None => {}
                        Some(other) => panic!("unmodelled outcome {other:?}"),
                    }
                }
                self.changes.push(ChangeFacts {
                    path: path.to_path_buf(),
                    slug: change.header.slug,
                    changes: change.changes,
                    affects: change
                        .affects
                        .iter()
                        .map(|a| format!("{}#{}", a.capability, a.section.slug()))
                        .collect(),
                    moved,
                });
            }
            RafDocument::Architecture(_) => self.architecture += 1,
        }
    }
}

// ---------------------------------------------------------------------------
// Reading each Markdown file through raf-okf
// ---------------------------------------------------------------------------

fn unparseable(path: &Path, error: &ParseError) -> String {
    violation(path, format!("unparseable document: {error}"))
}

/// A link split across lines is not a link; every body reports it once.
fn report_split_links(path: &Path, body: &Body, out: &mut Vec<String>) {
    for split in body.split_links() {
        out.push(violation(
            path,
            format!(
                "line {}: a link split across lines ({:?}) is not allowed",
                split.line(),
                split.form()
            ),
        ));
    }
}

fn record_markers(path: &Path, body: &Body, found: &mut Found) {
    for block in body.proposed_blocks() {
        found.markers.push(Marker {
            path: path.to_path_buf(),
            slug: block.slug().to_string(),
            section: block.section().map(str::to_string),
        });
    }
}

/// A document with frontmatter: the profile judges it and the typed form feeds
/// the cross-document checks.
fn read_document(path: &Path, document: &Document, found: &mut Found, out: &mut Vec<String>) {
    record_markers(path, document.body(), found);
    report_split_links(path, document.body(), out);
    match Profile.validate(document, name(path)) {
        Ok(typed) => found.record(path, typed),
        Err(violations) => out.extend(
            violations
                .iter()
                // Reported once for every body, above.
                .filter(|v| !matches!(v.rule(), Rule::SplitLink { .. }))
                .map(ToString::to_string),
        ),
    }
}

/// A scanned Markdown file without frontmatter: the generated capability
/// index, or prose that must not sit where a typed document belongs.
fn read_plain(path: &Path, text: &str, found: &mut Found, out: &mut Vec<String>) -> Option<Body> {
    let parsed = if is_capability_index(name(path)) {
        Index::parse(text)
            .map(|index| index.body().clone())
            .map_err(|error| {
                violation(
                    path,
                    format!("a hand-written capability index is not allowed: {error}"),
                )
            })
    } else {
        if let Some(expected) = expected_type(path) {
            out.push(violation(
                path,
                format!(
                    "missing frontmatter; expected a {} document",
                    expected.name()
                ),
            ));
        }
        Body::parse(text).map_err(|error| unparseable(path, &error))
    };
    let body = match parsed {
        Ok(body) => body,
        Err(message) => {
            out.push(message);
            return None;
        }
    };
    report_split_links(path, &body, out);
    for (line, token) in body.placeholder_lines() {
        out.push(violation(
            path,
            format!("line {line}: placeholder `{token}` is not resolved text"),
        ));
    }
    record_markers(path, &body, found);
    for block in body.proposed_blocks() {
        out.push(violation(
            path,
            format!(
                "line {}: PROPOSED block `{}` outside a capability",
                block.marker().lines.start,
                block.slug()
            ),
        ));
    }
    Some(body)
}

/// Read one Markdown file and return its body when it reads.
fn read_markdown(
    path: &Path,
    text: &str,
    found: &mut Found,
    out: &mut Vec<String>,
) -> Option<Body> {
    if !is_scanned(path) {
        return match Body::parse(text) {
            Ok(body) => {
                report_split_links(path, &body, out);
                Some(body)
            }
            Err(error) => {
                out.push(unparseable(path, &error));
                None
            }
        };
    }
    scan_banned(path, text, out);
    match Document::parse(text) {
        Ok(document) => {
            read_document(path, &document, found, out);
            Some(document.body().clone())
        }
        Err(error) if matches!(error.kind(), ErrorKind::MissingFence) => {
            read_plain(path, text, found, out)
        }
        Err(error) => {
            out.push(unparseable(path, &error));
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Links
// ---------------------------------------------------------------------------

/// Where a link sits, for its messages.
struct Site<'a> {
    path: &'a Path,
    line: usize,
    target: &'a str,
}

impl Site<'_> {
    fn report(&self, message: impl std::fmt::Display) -> String {
        violation(self.path, format!("line {}: {message}", self.line))
    }
}

/// Check a link to a path inside the repository against the tracked paths.
fn check_relative(
    repo: &Repo,
    bodies: &BTreeMap<PathBuf, Body>,
    site: &Site<'_>,
    relative: &str,
    anchor: Option<&str>,
    out: &mut Vec<String>,
) {
    let resolved = PathBuf::from(relative);
    let is_file = repo.contains_key(&resolved);
    let exists = is_file
        || repo
            .keys()
            .any(|k| k.starts_with(&resolved) && k != &resolved);
    if !exists {
        out.push(site.report(format!("broken link `{}`", site.target)));
        return;
    }
    if repo.get(&resolved).is_some_and(|text| text == SYMLINK_TEXT) {
        out.push(site.report(format!("link `{}` targets a tracked symlink", site.target)));
        return;
    }
    let Some(anchor) = anchor.filter(|a| !a.is_empty()) else {
        return;
    };
    if !is_file || !is_markdown(&resolved) {
        return;
    }
    match bodies.get(&resolved) {
        Some(body) if body.has_anchor(anchor) => {}
        Some(_) => out.push(site.report(format!("broken anchor `{}`", site.target))),
        None => out.push(site.report(format!(
            "link `{}` targets `{relative}`, which cannot be read as Markdown",
            site.target
        ))),
    }
}

fn check_links(
    repo: &Repo,
    bodies: &BTreeMap<PathBuf, Body>,
    path: &Path,
    body: &Body,
    out: &mut Vec<String>,
) {
    for link in body.links() {
        let site = Site {
            path,
            line: link.line(),
            target: link.target(),
        };
        match link.target_in(name(path)) {
            Err(error) => out.push(site.report(error)),
            Ok(LinkTarget::External(_)) => {}
            Ok(LinkTarget::Fragment(anchor)) => {
                if !anchor.is_empty() && !body.has_anchor(&anchor) {
                    out.push(site.report(format!("broken anchor `{}`", site.target)));
                }
            }
            Ok(LinkTarget::Relative {
                path: relative,
                anchor,
            }) => check_relative(repo, bodies, &site, &relative, anchor.as_deref(), out),
            Ok(other) => out.push(site.report(format!("unmodelled link target {other:?}"))),
        }
    }
}

// ---------------------------------------------------------------------------
// Cross-document checks
// ---------------------------------------------------------------------------

fn find_cycles(capabilities: &[CapabilityFacts], out: &mut Vec<String>) {
    let graph: BTreeMap<&str, (&Path, &[String])> = capabilities
        .iter()
        .map(|c| (c.id.as_str(), (c.path.as_path(), c.depends.as_slice())))
        .collect();
    let mut reported: BTreeSet<Vec<&str>> = BTreeSet::new();
    let mut done: BTreeSet<&str> = BTreeSet::new();
    for start in graph.keys() {
        let mut stack: Vec<(&str, usize)> = vec![(start, 0)];
        let mut on_path: Vec<&str> = vec![start];
        while let Some((node, next)) = stack.pop() {
            let deps = graph.get(node).map_or(&[][..], |(_, d)| *d);
            let Some(dep) = deps.get(next) else {
                on_path.pop();
                done.insert(node);
                continue;
            };
            stack.push((node, next + 1));
            let dep = dep.as_str();
            if let Some(position) = on_path.iter().position(|n| *n == dep) {
                let mut cycle: Vec<&str> = on_path.iter().skip(position).copied().collect();
                if let Some(min) = cycle
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, n)| **n)
                    .map(|(i, _)| i)
                {
                    cycle.rotate_left(min);
                }
                if reported.insert(cycle.clone()) {
                    let path = graph.get(cycle[0]).map_or(Path::new(""), |(p, _)| p);
                    out.push(violation(
                        path,
                        format!("depends cycle: {} -> {}", cycle.join(" -> "), cycle[0]),
                    ));
                }
            } else if graph.contains_key(dep) && !done.contains(dep) {
                on_path.push(dep);
                stack.push((dep, 0));
            }
        }
    }
}

/// State rules for one capability: PROPOSED blocks, their `affects` entries
/// and the open change record.
fn check_capability_state(cap: &CapabilityFacts, found: &Found, out: &mut Vec<String>) {
    let proposed: Vec<&Marker> = found
        .markers
        .iter()
        .filter(|m| m.path == cap.path)
        .collect();
    let listing: Vec<&ChangeFacts> = found
        .changes
        .iter()
        .filter(|c| c.changes.contains(&cap.id))
        .collect();
    for marker in &proposed {
        let Some(change) = listing.iter().find(|c| c.slug == marker.slug) else {
            out.push(violation(
                &cap.path,
                format!("PROPOSED change `{}` does not list {}", marker.slug, cap.id),
            ));
            continue;
        };
        match &marker.section {
            Some(section) => {
                let entry = format!("{}#{section}", cap.id);
                if !change.affects.contains(&entry) {
                    out.push(violation(
                        &cap.path,
                        format!(
                            "PROPOSED block in section `{section}` needs `{entry}` in the \
                             affects of change `{}`",
                            change.slug
                        ),
                    ));
                }
            }
            // `Profile::validate` reports a block outside a section first and
            // then returns no typed capability, so a capability that reaches
            // this check has every block inside a section.
            None => panic!(
                "{}: PROPOSED block `{}` outside a section reached a validated capability",
                cap.path.display(),
                marker.slug
            ),
        }
    }
    match cap.state {
        State::Considered if proposed.is_empty() => out.push(violation(
            &cap.path,
            "state considered requires its candidate contract in PROPOSED blocks",
        )),
        State::Designed if !proposed.is_empty() => out.push(violation(
            &cap.path,
            "state designed must have no PROPOSED blocks",
        )),
        State::Delivered => {
            if !proposed.is_empty() {
                out.push(violation(
                    &cap.path,
                    "state delivered must have no PROPOSED blocks",
                ));
            }
            for change in &listing {
                out.push(violation(
                    &cap.path,
                    format!(
                        "state delivered must not be listed by open change record {}",
                        change.path.display()
                    ),
                ));
            }
        }
        State::Considered | State::Designed | State::Implementing => {}
    }
    if cap.state != State::Delivered && listing.is_empty() {
        out.push(violation(
            &cap.path,
            format!(
                "state {} requires an open change record listing {}",
                cap.state.name(),
                cap.id
            ),
        ));
    }
}

fn check_cross_references(repo: &Repo, found: &Found, out: &mut Vec<String>) {
    let ids: BTreeSet<&str> = found.capabilities.iter().map(|c| c.id.as_str()).collect();
    for cap in &found.capabilities {
        for dep in &cap.depends {
            if !ids.contains(dep.as_str()) {
                out.push(violation(&cap.path, format!("dangling depends `{dep}`")));
            }
        }
    }
    find_cycles(&found.capabilities, out);
    for change in &found.changes {
        for id in &change.changes {
            if !ids.contains(id.as_str()) {
                out.push(violation(
                    &change.path,
                    format!("changes names unknown `{id}`"),
                ));
            }
        }
    }
    for marker in &found.markers {
        let record = PathBuf::from(format!("work/changes/{}.md", marker.slug));
        if !repo.contains_key(&record) {
            out.push(violation(
                &marker.path,
                format!(
                    "orphan PROPOSED marker: {} does not exist",
                    record.display()
                ),
            ));
        }
    }
    for cap in &found.capabilities {
        check_capability_state(cap, found, out);
    }
    let change_slugs: BTreeSet<&str> = found.changes.iter().map(|c| c.slug.as_str()).collect();
    for change in &found.changes {
        for target in &change.moved {
            if !change_slugs.contains(target.as_str()) {
                out.push(violation(
                    &change.path,
                    format!("item moved to change `{target}`, but work/changes/{target}.md does not exist"),
                ));
            }
        }
    }
}

fn check_directories(repo: &Repo, out: &mut Vec<String>) {
    for path in repo.keys() {
        let parts: Vec<&str> = name(path).split('/').collect();
        let typed_directory = matches!(
            parts.as_slice(),
            ["docs", "capabilities" | "architecture", _, ..] | ["work", "changes", _, ..]
        );
        if typed_directory && !is_markdown(path) {
            out.push(violation(
                path,
                "only Markdown documents belong in a document directory",
            ));
        }
        if is_scanned(path) && typed_directory && parts.len() > 3 {
            out.push(violation(
                path,
                "documents must sit directly in their directory",
            ));
        }
    }
}

/// What the checker found: every violation, and the facts the binding check
/// and the counts use.
struct Analysis {
    violations: Vec<String>,
    found: Found,
}

/// Check every rule of the profile over an in-memory repository.
fn analyse(repo: &Repo) -> Analysis {
    let mut out = Vec::new();
    let mut found = Found::default();
    check_directories(repo, &mut out);
    for (path, _) in repo.iter().filter(|(_, text)| *text == SYMLINK_TEXT) {
        out.push(violation(path, "a tracked symlink is not allowed"));
    }
    let mut bodies: BTreeMap<PathBuf, Body> = BTreeMap::new();
    for (path, text) in repo
        .iter()
        .filter(|(path, text)| *text != SYMLINK_TEXT && is_markdown(path))
    {
        if let Some(body) = read_markdown(path, text, &mut found, &mut out) {
            bodies.insert(path.clone(), body);
        }
    }
    for (path, body) in &bodies {
        check_links(repo, &bodies, path, body, &mut out);
    }
    check_cross_references(repo, &found, &mut out);
    out.sort();
    out.dedup();
    Analysis {
        violations: out,
        found,
    }
}

fn check(repo: &Repo) -> Vec<String> {
    analyse(repo).violations
}

// ---------------------------------------------------------------------------
// Bindings
// ---------------------------------------------------------------------------

/// Resolve every binding of every capability in the source index. A binding
/// that names nothing, names two things, or goes through a cycle is a
/// violation that names the capability and the binding.
fn check_bindings(found: &Found, workspace: &Workspace) -> Vec<String> {
    let mut out = Vec::new();
    for cap in &found.capabilities {
        let lists = [
            ("code", &cap.bindings.code),
            ("tests", &cap.bindings.tests),
            ("examples", &cap.bindings.examples),
        ];
        for (kind, bindings) in lists {
            for binding in bindings {
                if let Err(error) = workspace.resolve(binding) {
                    out.push(violation(
                        &cap.path,
                        format!(
                            "{} {kind} binding `{binding}` does not resolve: {error}",
                            cap.id
                        ),
                    ));
                }
            }
        }
    }
    out
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
/// stated `id` and `state`, every open capability has its change record, and
/// no document exists that the inventory does not list.
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
    for path in repo.keys().filter(|p| is_markdown(p)) {
        let known = path.to_str().is_some_and(|p| listed.contains(p));
        if expected_type(path).is_some() && !known && !is_capability_index(name(path)) {
            out.push(violation(
                path,
                "the document is not in the inventory table",
            ));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

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

/// Load the real tree from the git index, read under the gate root: every
/// tracked path (for link targets) and the text of every Markdown file.
fn load_repo(root: &Path) -> Repo {
    let symlinks = tracked_symlinks();
    tracked_paths()
        .into_iter()
        .map(|path| {
            let text = if symlinks.contains(&path) {
                SYMLINK_TEXT.to_string()
            } else if path.extension().is_some_and(|e| e == "md") {
                let bytes = read_tracked(root, &path);
                match decode_text(&bytes) {
                    Some(text) => text.to_string(),
                    None => panic!("{} is not UTF-8 text", path.display()),
                }
            } else {
                String::new()
            };
            (path, text)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

fn report(violations: &[String]) -> String {
    let mut report = String::new();
    for v in violations {
        let _ = writeln!(report, "  {v}");
    }
    report
}

#[test]
fn repository_documents_satisfy_the_profile() {
    let repo = load_repo(&gate_root());
    let analysis = analyse(&repo);
    let mut violations = analysis.violations;
    violations.extend(check_inventory(&repo, &INVENTORY));
    assert!(
        violations.is_empty(),
        "{} profile violations:\n{}",
        violations.len(),
        report(&violations)
    );
    for (kind, count) in [
        ("Architecture", analysis.found.architecture),
        ("Capability", analysis.found.capabilities.len()),
        ("Change", analysis.found.changes.len()),
    ] {
        assert!(count >= 1, "no {kind} document found");
    }
}

#[test]
fn repository_bindings_resolve_in_the_source_index() -> Result<(), Box<dyn Error>> {
    let root = gate_root();
    let analysis = analyse(&load_repo(&root));
    let workspace = Workspace::load(&DiskTree::new(&root)?, &["crates"])?;
    // A load that found nothing would pass the check below, so require known
    // symbols first.
    for symbol in ["raf_okf::Document", "raf_source::Workspace"] {
        workspace.resolve(symbol)?;
    }
    let violations = check_bindings(&analysis.found, &workspace);
    assert!(
        violations.is_empty(),
        "{} unresolved bindings:\n{}",
        violations.len(),
        report(&violations)
    );
    Ok(())
}
mod fixtures {
    use super::{Path, PathBuf, Repo, check};

    pub(super) fn capability(slug: &str, state: &str, depends: &str, body: &str) -> String {
        let sections = [
            "Purpose",
            "Contract",
            "Usage",
            "Examples",
            "Constraints",
            "Explanation",
            "Verification",
        ]
        .iter()
        .map(|name| {
            let extra = if *name == "Purpose" { body } else { "" };
            format!("## {name}\n\n{extra}Text for {name}.\n")
        })
        .collect::<Vec<_>>()
        .join("\n");
        let (code, tests) = if state == "delivered" {
            ("[src/lib.rs]", "[tests/it.rs]")
        } else {
            ("[]", "[]")
        };
        format!(
            "---\n# SPDX-FileCopyrightText: 2026 Example\n# SPDX-License-Identifier: CC-BY-4.0\n\
             type: Capability\nid: cap:{slug}\ntitle: \"Title of {slug}\"\n\
             description: A description of {slug}.\nstate: {state}\n\
             bindings:\n  code: {code}\n  tests: {tests}\n  examples: []\n\
             depends: {depends}\n---\n\n# {slug}\n\n{sections}"
        )
    }

    const PROPOSED: &str = "> [!PROPOSED change:add-alpha]\n> The candidate contract.\n\n";

    pub(super) fn valid() -> Repo {
        let mut repo = Repo::new();
        let mut put = |path: &str, text: String| {
            repo.insert(PathBuf::from(path), text);
        };
        put(
            "docs/capabilities/alpha.md",
            capability("alpha", "designed", "[cap:beta]", ""),
        );
        put(
            "docs/capabilities/beta.md",
            capability("beta", "considered", "[]", PROPOSED),
        );
        put(
            "docs/capabilities/gamma.md",
            capability("gamma", "delivered", "[cap:beta, cap:alpha]", ""),
        );
        put(
            "docs/architecture/overview.md",
            "---\ntype: Architecture\nid: arch:overview\ntitle: Overview\n\
             description: How it fits.\n---\n\n# Overview\n\nBody text.\n"
                .to_string(),
        );
        put(
            "work/changes/add-alpha.md",
            "---\ntype: Change\nid: change:add-alpha\ntitle: Add alpha\n\
             description: Adds alpha and beta.\n\
             changes: [cap:alpha, cap:beta]\naffects:\n  - cap:alpha#contract\n  - cap:beta#purpose\n\
             contract_effect: changes\nitems:\n  - claim: \"alpha reads input\"\n    outcome: implemented\n  \
             - claim: \"beta is sketched\"\n---\n\n## Summary\n\nAdds both.\n"
                .to_string(),
        );
        put(
            "README.md",
            "# Readme\n\nSee [alpha](docs/capabilities/alpha.md#purpose) and [the overview](./docs/architecture/overview.md).\n"
                .to_string(),
        );
        repo
    }

    /// A change record with the given `changes` list, `affects` lines and
    /// `items` block (the text after `items:`).
    pub(super) fn change_record(slug: &str, changes: &str, affects: &str, items: &str) -> String {
        let items = if items.is_empty() {
            "items: []\n".to_string()
        } else {
            format!("items:\n{items}")
        };
        format!(
            "---\ntype: Change\nid: change:{slug}\ntitle: Change {slug}\n\
             description: Changes {slug}.\nchanges: {changes}\naffects:\n{affects}\
             contract_effect: changes\n{items}---\n\n## Summary\n\nChanges it.\n"
        )
    }

    pub(super) fn mutated(path: &str, from: &str, to: &str) -> Vec<String> {
        let mut repo = valid();
        let text = repo
            .get_mut(Path::new(path))
            .unwrap_or_else(|| panic!("fixture has no {path}"));
        assert!(text.contains(from), "fixture {path} has no `{from}`");
        *text = text.replacen(from, to, 1);
        check(&repo)
    }
}

use fixtures::{capability, mutated, valid};

fn assert_hit(violations: &[String], needle: &str) {
    assert!(
        violations.iter().any(|v| v.contains(needle)),
        "expected a violation containing `{needle}`, got: {violations:#?}"
    );
}

#[test]
fn accepts_valid_fixture() {
    let analysis = analyse(&valid());
    assert!(analysis.violations.is_empty(), "{:#?}", analysis.violations);
    assert_eq!(analysis.found.capabilities.len(), 3);
    assert_eq!(analysis.found.changes.len(), 1);
    assert_eq!(analysis.found.architecture, 1);
}

#[test]
fn accepts_quoted_and_block_forms() {
    assert_clean(&mutated(OVERVIEW, "title: Overview", "title: 'It''s'"));
    assert_clean(&mutated(OVERVIEW, "title: Overview", "title: \"a: b\""));
    assert_clean(&mutated(
        ALPHA,
        "depends: [cap:beta]",
        "depends:\n  - cap:beta",
    ));
    assert_clean(&mutated(
        CHANGE,
        "claim: \"alpha reads input\"\n    outcome: implemented",
        "claim: \"a: b\"\n    outcome: dropped",
    ));
    assert_clean(&mutated(
        OVERVIEW,
        "title: Overview",
        "# note\ntitle: Overview\ntags:\n  - one\n  - two",
    ));
}

#[test]
fn rejects_unparseable_frontmatter() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "title: Overview",
        "title Overview",
    );
    assert_hit(&violations, "unparseable document: line 4");
}

#[test]
fn rejects_unknown_key() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "title: Overview",
        "title: Overview\nextra: 1",
    );
    assert_hit(&violations, "unknown key `extra`");
}

#[test]
fn accepts_optional_tags_and_timestamp() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "title: Overview",
        "title: Overview\ntags: [a, b]\ntimestamp: 2026-01-01T00:00:00Z",
    );
    assert!(violations.is_empty(), "{violations:#?}");
}

#[test]
fn rejects_duplicate_tag() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "title: Overview",
        "title: Overview\ntags: [a, a]",
    );
    assert_hit(&violations, "duplicate tag `a`");
}

#[test]
fn rejects_scalar_tags() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "title: Overview",
        "title: Overview\ntags: a",
    );
    assert_hit(&violations, "`tags`: must be a list of strings");
}

#[test]
fn rejects_missing_required_key() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "description: How it fits.\n",
        "",
    );
    assert_hit(&violations, "missing required key `description`");
}

#[test]
fn rejects_bad_type() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "type: Architecture",
        "type: Notes",
    );
    assert_hit(&violations, "bad type `Notes`");
}

#[test]
fn rejects_type_in_the_wrong_directory() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "type: Architecture",
        "type: Change",
    );
    assert_hit(
        &violations,
        "`type`: a Change document belongs directly in work/changes",
    );
}

#[test]
fn rejects_id_filename_mismatch() {
    let violations = mutated(
        "docs/capabilities/alpha.md",
        "id: cap:alpha",
        "id: cap:other",
    );
    assert_hit(&violations, "does not match the file name");
}

#[test]
fn rejects_bad_state() {
    let violations = mutated(
        "docs/capabilities/alpha.md",
        "state: designed",
        "state: done",
    );
    assert_hit(&violations, "bad state `done`");
}

#[test]
fn rejects_wrong_bindings_keys() {
    let violations = mutated(
        "docs/capabilities/alpha.md",
        "  examples: []\n",
        "  docs: []\n",
    );
    assert_hit(&violations, "bindings must have exactly the keys");
}

#[test]
fn rejects_missing_section() {
    let violations = mutated(
        "docs/capabilities/alpha.md",
        "## Usage\n\nText for Usage.\n\n",
        "",
    );
    assert_hit(&violations, "section `Usage`: missing section");
}

#[test]
fn rejects_sections_out_of_order() {
    let violations = mutated("docs/capabilities/alpha.md", "## Purpose", "## Swap");
    assert_hit(&violations, "section `Swap`: unexpected section");
    let mut repo = valid();
    let text = repo
        .get_mut(Path::new("docs/capabilities/alpha.md"))
        .expect("fixture capability");
    *text = text
        .replace("## Contract", "## TMP")
        .replace("## Usage", "## Contract")
        .replace("## TMP", "## Usage");
    assert_hit(&check(&repo), "sections out of order");
}

#[test]
fn rejects_empty_section() {
    let violations = mutated("docs/capabilities/alpha.md", "Text for Examples.\n", "");
    assert_hit(&violations, "section `Examples`: section is empty");
}

#[test]
fn rejects_dangling_depends() {
    let violations = mutated("docs/capabilities/alpha.md", "[cap:beta]", "[cap:missing]");
    assert_hit(&violations, "dangling depends `cap:missing`");
}

#[test]
fn rejects_depends_two_cycle() {
    let violations = mutated(
        "docs/capabilities/beta.md",
        "depends: []",
        "depends: [cap:alpha]",
    );
    assert_hit(
        &violations,
        "depends cycle: cap:alpha -> cap:beta -> cap:alpha",
    );
}

#[test]
fn rejects_broken_relative_link() {
    let violations = mutated(
        "README.md",
        "docs/capabilities/alpha.md#purpose",
        "docs/capabilities/nope.md",
    );
    assert_hit(&violations, "broken link `docs/capabilities/nope.md`");
}

#[test]
fn rejects_broken_anchor() {
    let violations = mutated("README.md", "alpha.md#purpose", "alpha.md#no-such-heading");
    assert_hit(&violations, "broken anchor");
}

#[test]
fn rejects_link_to_a_tracked_symlink() {
    let mut repo = valid();
    repo.insert(
        PathBuf::from("docs/outside-link.svg"),
        SYMLINK_TEXT.to_string(),
    );
    repo.insert(
        PathBuf::from(".github/PULL_REQUEST_TEMPLATE.md"),
        "# Template\n\n![x](../docs/outside-link.svg)\n".to_string(),
    );
    assert_hit(
        &check(&repo),
        "link `../docs/outside-link.svg` targets a tracked symlink",
    );
}

#[test]
fn rejects_a_tracked_symlink_that_nothing_links_to() {
    let mut repo = valid();
    repo.insert(PathBuf::from("NOTES.md"), SYMLINK_TEXT.to_string());
    assert_hit(&check(&repo), "NOTES.md: a tracked symlink is not allowed");
}

#[test]
fn rejects_link_leaving_the_repository() {
    let violations = mutated(
        "README.md",
        "./docs/architecture/overview.md",
        "../../outside.md",
    );
    assert_hit(&violations, "leaves the repository");
}

#[test]
fn anchors_follow_github_rules() {
    let target = "# Target\n\n## Why `x`: it's (fine)\n\n## Same\n\n## Same\n";
    let link = |anchor: &str| {
        let mut repo = valid();
        repo.insert(PathBuf::from("TARGET.md"), target.to_string());
        repo.insert(
            PathBuf::from("LINKS.md"),
            format!("[a](TARGET.md#{anchor})\n"),
        );
        check(&repo)
    };
    for anchor in ["why-x-its-fine", "same", "same-1", "target"] {
        assert_clean(&link(anchor));
    }
    assert_hit(&link("same-2"), "broken anchor `TARGET.md#same-2`");
    assert_hit(&link("Why-x-its-fine"), "broken anchor");
}

#[test]
fn rejects_orphan_proposed_marker() {
    let violations = mutated(
        "docs/capabilities/beta.md",
        "change:add-alpha",
        "change:ghost",
    );
    assert_hit(&violations, "orphan PROPOSED marker: work/changes/ghost.md");
}

#[test]
fn rejects_malformed_proposed_marker() {
    let violations = mutated(
        "docs/capabilities/beta.md",
        "> [!PROPOSED change:add-alpha]",
        "> [!PROPOSED add-alpha]",
    );
    assert_hit(&violations, "malformed PROPOSED marker");
}

#[test]
fn rejects_considered_without_proposed_block() {
    let violations = mutated(
        "docs/capabilities/beta.md",
        "> [!PROPOSED change:add-alpha]\n> The candidate contract.\n",
        "",
    );
    assert_hit(
        &violations,
        "requires its candidate contract in PROPOSED blocks",
    );
}

#[test]
fn rejects_open_capability_without_change_record() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "changes: [cap:alpha, cap:beta]",
        "changes: [cap:beta]",
    );
    assert_hit(
        &violations,
        "requires an open change record listing cap:alpha",
    );
}

#[test]
fn rejects_delivered_with_proposed_block() {
    let violations = mutated(
        "docs/capabilities/gamma.md",
        "Text for Purpose.\n",
        "Text for Purpose.\n\n> [!PROPOSED change:add-alpha]\n> Left over.\n",
    );
    assert_hit(&violations, "delivered must have no PROPOSED blocks");
}

#[test]
fn rejects_delivered_with_open_change_record() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "changes: [cap:alpha, cap:beta]",
        "changes: [cap:alpha, cap:beta, cap:gamma]",
    );
    assert_hit(
        &violations,
        "delivered must not be listed by open change record",
    );
}

#[test]
fn rejects_affects_with_bad_section() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "cap:alpha#contract",
        "cap:alpha#nonsense",
    );
    assert_hit(&violations, "bad section `nonsense`");
}

#[test]
fn rejects_affects_slug_not_in_changes() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "cap:alpha#contract",
        "cap:gamma#contract",
    );
    assert_hit(&violations, "missing from `changes`");
}

#[test]
fn rejects_bad_contract_effect() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "contract_effect: changes",
        "contract_effect: maybe",
    );
    assert_hit(&violations, "bad contract_effect `maybe`");
}

#[test]
fn rejects_bad_outcome() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "outcome: implemented",
        "outcome: finished",
    );
    assert_hit(&violations, "bad outcome `finished`");
}

#[test]
fn accepts_moved_outcome() {
    let mut repo = valid();
    repo.insert(
        PathBuf::from("work/changes/other-work.md"),
        fixtures::change_record("other-work", "[cap:alpha]", "  - cap:alpha#usage\n", ""),
    );
    let record = repo
        .get_mut(Path::new("work/changes/add-alpha.md"))
        .expect("fixture change");
    *record = record.replacen(
        "outcome: implemented",
        "outcome: moved:change:other-work",
        1,
    );
    let violations = check(&repo);
    assert!(violations.is_empty(), "{violations:#?}");
}

#[test]
fn rejects_change_without_summary() {
    let violations = mutated("work/changes/add-alpha.md", "## Summary", "## Notes");
    assert_hit(&violations, "section `Summary`: missing section");
}

#[test]
fn rejects_unclosed_generated_block() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "Body text.",
        "Body text.\n\n<!-- raf:generated:matrix -->\n",
    );
    assert_hit(&violations, "is never closed");
}

#[test]
fn rejects_hand_written_capability_index() {
    let mut repo = valid();
    repo.insert(
        PathBuf::from("docs/capabilities/index.md"),
        "# Index\n".to_string(),
    );
    assert_hit(&check(&repo), "hand-written capability index");
}

#[test]
fn rejects_issue_number_token() {
    let token = concat!("Per ", "#", "12");
    let violations = mutated(
        "docs/architecture/overview.md",
        "Body text.",
        &format!("Body text. {token} said so."),
    );
    assert_hit(&violations, "issue or pull request number is banned");
    let url = concat!("https://example.invalid/", "issues/", "7");
    let violations = mutated("README.md", "# Readme", &format!("# Readme\n\n{url}"));
    assert_hit(&violations, "issue or pull request number is banned");
}

#[test]
fn issue_token_scan_spares_hex_colours() {
    assert!(banned_in_line("fill #111214 and #fbbc05 and #1f2").is_none());
    assert!(banned_in_line(concat!("see PR", "#", "9")).is_some());
    assert!(banned_in_line(concat!("CHANGE", "-", "17")).is_some());
}

#[test]
fn rejects_process_wording() {
    let phrase = concat!("As agreed in ", "this", " branch");
    let violations = mutated(
        "docs/architecture/overview.md",
        "Body text.",
        &format!("Body text. {phrase}."),
    );
    assert_hit(&violations, "development-process wording is banned");
    let label = concat!("Task", " 4 is done");
    let violations = mutated("README.md", "# Readme", &format!("# Readme\n\n{label}"));
    assert_hit(&violations, "development-process wording is banned");
}

#[test]
fn fixture_builder_is_self_consistent() {
    let doc = capability("delta", "implementing", "[]", "");
    assert!(doc.contains("id: cap:delta"));
}

#[test]
fn rejects_capability_without_frontmatter() {
    let mut repo = valid();
    repo.insert(
        PathBuf::from("docs/capabilities/plain.md"),
        "# Plain\n\nNo frontmatter here.\n".to_string(),
    );
    assert_hit(
        &check(&repo),
        "docs/capabilities/plain.md: missing frontmatter",
    );
}

#[test]
fn rejects_unclosed_frontmatter() {
    let violations = mutated(
        "docs/architecture/overview.md",
        "---\n\n# Overview",
        "# Overview",
    );
    assert_hit(&violations, "frontmatter is not closed");
}

#[test]
fn rejects_proposed_marker_in_architecture() {
    let marker = "> [!PROPOSED change:add-alpha]\n> Not allowed here.";
    let violations = mutated("docs/architecture/overview.md", "Body text.", marker);
    assert_hit(
        &violations,
        "overview.md: line 10: PROPOSED block `add-alpha` outside a capability",
    );
}

#[test]
fn rejects_proposed_marker_in_change() {
    let marker = "Adds both.\n\n> [!PROPOSED change:add-alpha]\n> Not allowed here.";
    let violations = mutated("work/changes/add-alpha.md", "Adds both.", marker);
    assert_hit(
        &violations,
        "add-alpha.md: line 21: PROPOSED block `add-alpha` outside a capability",
    );
}

#[test]
fn rejects_change_naming_unknown_capability() {
    let violations = mutated(
        "work/changes/add-alpha.md",
        "changes: [cap:alpha, cap:beta]",
        "changes: [cap:alpha, cap:beta, cap:ghost]",
    );
    assert_hit(&violations, "changes names unknown `cap:ghost`");
}

/// Nested documents are rejected, not validated: every document sits directly
/// in its directory.
#[test]
fn rejects_nested_documents() {
    for nested in ["docs/capabilities/sub/x.md", "work/changes/sub/y.md"] {
        let mut repo = valid();
        repo.insert(PathBuf::from(nested), "# Nested\n".to_string());
        assert_hit(
            &check(&repo),
            &format!("{nested}: documents must sit directly in their directory"),
        );
    }
}

#[test]
fn rejects_acceptance_label() {
    let label = concat!("Criterion ", "AC", "3", " is met");
    let violations = mutated("README.md", "# Readme", &format!("# Readme\n\n{label}"));
    assert_hit(&violations, "development-process wording is banned");
}

#[test]
fn rejects_resource_key() {
    // `resource` is rejected by its own rule, which names the keys of the type,
    // and not by the unknown-key rule.
    let violations = mutated(
        "docs/architecture/overview.md",
        "title: Overview",
        "title: Overview\nresource: https://example.invalid/overview",
    );
    assert_hit(&violations, "`resource` is not a key of a");
    let violations = mutated(
        "docs/capabilities/alpha.md",
        "state: designed",
        "state: designed\nresource: https://example.invalid/alpha",
    );
    assert_hit(&violations, "`resource` is not a key of a");
}

// ---------------------------------------------------------------------------
// Fixtures for the hardened rules
// ---------------------------------------------------------------------------

/// Apply each `(from, to, needle)` mutation to `path` and require the needle.
fn assert_cases(path: &str, cases: &[(&str, &str, &str)]) {
    for (from, to, needle) in cases {
        let violations = mutated(path, from, to);
        assert!(
            violations.iter().any(|v| v.contains(needle)),
            "replacing `{from}` with `{to}` should report `{needle}`, got: {violations:#?}"
        );
    }
}

fn assert_clean(violations: &[String]) {
    assert!(violations.is_empty(), "{violations:#?}");
}

const OVERVIEW: &str = "docs/architecture/overview.md";
const ALPHA: &str = "docs/capabilities/alpha.md";
const BETA: &str = "docs/capabilities/beta.md";
const GAMMA: &str = "docs/capabilities/gamma.md";
const CHANGE: &str = "work/changes/add-alpha.md";

// --- capability inventory ---------------------------------------------------

const FIXTURE_INVENTORY: Inventory = Inventory {
    capabilities: &[
        ("alpha", "designed"),
        ("beta", "considered"),
        ("gamma", "delivered"),
    ],
    architecture: &["overview"],
};

/// A valid tree whose documents are exactly `FIXTURE_INVENTORY`.
fn inventory_repo() -> Repo {
    let mut repo = valid();
    repo.remove(Path::new(CHANGE));
    let beta = repo.get_mut(Path::new(BETA)).expect("fixture beta");
    *beta = beta.replace("change:add-alpha", "change:beta");
    for (slug, section) in [("alpha", "contract"), ("beta", "purpose")] {
        repo.insert(
            PathBuf::from(format!("work/changes/{slug}.md")),
            fixtures::change_record(
                slug,
                &format!("[cap:{slug}]"),
                &format!("  - cap:{slug}#{section}\n"),
                "",
            ),
        );
    }
    repo
}

#[test]
fn inventory_accepts_a_matching_tree() {
    let repo = inventory_repo();
    assert_clean(&check(&repo));
    assert_clean(&check_inventory(&repo, &FIXTURE_INVENTORY));
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
    assert_eq!(count("designed"), 14);
    assert_eq!(count("delivered"), 2);
    assert_eq!(count("considered"), 5);
    assert_eq!(INVENTORY.architecture.len(), 2);
}

#[test]
fn inventory_rejects_a_missing_capability() {
    let mut repo = inventory_repo();
    repo.remove(Path::new(BETA));
    let found = check_inventory(&repo, &FIXTURE_INVENTORY);
    assert_hit(&found, "docs/capabilities/beta.md: the document is missing");
}

#[test]
fn inventory_rejects_a_missing_change_record() {
    let mut repo = inventory_repo();
    repo.remove(Path::new("work/changes/alpha.md"));
    let found = check_inventory(&repo, &FIXTURE_INVENTORY);
    assert_hit(&found, "work/changes/alpha.md: the document is missing");
}

#[test]
fn inventory_rejects_a_missing_architecture_document() {
    let mut repo = inventory_repo();
    repo.remove(Path::new(OVERVIEW));
    let found = check_inventory(&repo, &FIXTURE_INVENTORY);
    assert_hit(
        &found,
        "docs/architecture/overview.md: the document is missing",
    );
}

#[test]
fn inventory_rejects_a_state_change() {
    let mut repo = inventory_repo();
    let alpha = repo.get_mut(Path::new(ALPHA)).expect("fixture alpha");
    *alpha = alpha.replace("state: designed", "state: implementing");
    let found = check_inventory(&repo, &FIXTURE_INVENTORY);
    assert_hit(
        &found,
        "inventory expects state `designed`, found `implementing`",
    );
}

#[test]
fn inventory_rejects_an_id_change() {
    let mut repo = inventory_repo();
    let overview = repo.get_mut(Path::new(OVERVIEW)).expect("fixture overview");
    *overview = overview.replace("arch:overview", "arch:other");
    let found = check_inventory(&repo, &FIXTURE_INVENTORY);
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
        capability("extra", "designed", "[]", ""),
    );
    let found = check_inventory(&repo, &FIXTURE_INVENTORY);
    assert_hit(
        &found,
        "docs/capabilities/extra.md: the document is not in the inventory table",
    );
}

// --- placeholders and empty sections ----------------------------------------

#[test]
fn rejects_placeholder_tokens() {
    for token in PLACEHOLDER_TOKENS {
        let violations = mutated(
            OVERVIEW,
            "Body text.",
            &format!("Body text. Still {token} here."),
        );
        assert_hit(&violations, "is not resolved text");
    }
    let shouted = concat!("Left TO", "DO: finish");
    let violations = mutated(ALPHA, "Text for Usage.", shouted);
    assert_hit(&violations, "is not resolved text");
    let mixed_case = concat!("It is To Be ", "Written.");
    let violations = mutated(ALPHA, "Text for Usage.", mixed_case);
    assert_hit(&violations, "is not resolved text");
}

#[test]
fn placeholder_scan_spares_code_and_longer_words() {
    let token = concat!("T", "BD");
    let fenced = format!("Body text.\n\n```text\n{token}\n```\n\nUse `{token}` and {token}x.\n");
    assert_clean(&mutated(OVERVIEW, "Body text.\n", &fenced));
}

#[test]
fn rejects_sections_left_empty_by_comments() {
    for comment in [
        "<!-- later -->\n",
        "<!--\nlater\n-->\n",
        "<!-- a --> <!-- b -->\n",
    ] {
        let violations = mutated(ALPHA, "Text for Examples.\n", comment);
        assert_hit(&violations, "section `Examples`: section is empty");
    }
    let violations = mutated(CHANGE, "Adds both.\n", "<!-- soon -->\n");
    assert_hit(&violations, "section `Summary`: section is empty");
    let violations = mutated(
        OVERVIEW,
        "# Overview\n\nBody text.\n",
        "<!-- nothing yet -->\n",
    );
    assert_hit(&violations, "the body is empty");
}

#[test]
fn comments_hide_text_but_not_the_text_beside_them() {
    assert_clean(&mutated(
        ALPHA,
        "Text for Examples.\n",
        "<!-- x -->\ntext\n",
    ));
    assert_clean(&mutated(ALPHA, "Text for Examples.\n", "a <!-- x --> b\n"));
    let violations = mutated(ALPHA, "Text for Examples.\n", "  <!-- x -->\n  \n");
    assert_hit(&violations, "section `Examples`: section is empty");
}

// --- frontmatter scalars ----------------------------------------------------

#[test]
fn rejects_unterminated_quote() {
    assert_cases(
        OVERVIEW,
        &[
            (
                "title: Overview",
                "title: \"Overview",
                "unterminated quoted value",
            ),
            (
                "title: Overview",
                "title: 'Overview",
                "unterminated quoted value",
            ),
            (
                "title: Overview",
                "title: \"Over\" view",
                "text after the closing quote",
            ),
        ],
    );
}

#[test]
fn rejects_unquoted_colon_space() {
    assert_cases(
        OVERVIEW,
        &[
            (
                "title: Overview",
                "title: Built-in: census",
                "contains `: `",
            ),
            ("title: Overview", "title: Overview:", "contains `: `"),
        ],
    );
}

#[test]
fn rejects_leading_indicator_character() {
    for (indicator, needle) in [
        ('>', "cannot start with the YAML indicator character"),
        ('|', "cannot start with the YAML indicator character"),
        ('&', "anchor `&Overview` is not accepted"),
        ('*', "alias `*Overview` is not accepted"),
        ('!', "tag `!Overview` is not accepted"),
        ('%', "cannot start with the YAML indicator character"),
        ('@', "cannot start with the YAML indicator character"),
        ('`', "cannot start with the YAML indicator character"),
    ] {
        let violations = mutated(
            OVERVIEW,
            "title: Overview",
            &format!("title: {indicator}Overview"),
        );
        assert_hit(&violations, needle);
    }
}

#[test]
fn rejects_block_scalar_indicator_value() {
    for indicator in [">", "|", ">-", "|+", ">2", "|-"] {
        let violations = mutated(
            OVERVIEW,
            "description: How it fits.",
            &format!("description: {indicator}"),
        );
        assert_hit(&violations, "block scalar header");
    }
}

#[test]
fn accepts_comments_after_scalars() {
    assert_clean(&mutated(
        OVERVIEW,
        "title: Overview",
        "title: Overview # a note",
    ));
    assert_clean(&mutated(
        OVERVIEW,
        "title: Overview",
        "title: \"a # b\" # c\ntags: [one, \"two # 2\"] # c",
    ));
    assert_clean(&mutated(
        OVERVIEW,
        "description: How it fits.",
        "description: 'It''s # x' # c",
    ));
    assert_clean(&mutated(
        OVERVIEW,
        "description: How it fits.",
        "description: a#b",
    ));
    assert_clean(&mutated(
        CHANGE,
        "outcome: implemented",
        "outcome: implemented # done",
    ));
    assert_clean(&mutated(
        ALPHA,
        "state: designed",
        "state: designed # still planned",
    ));
    assert_clean(&mutated(
        ALPHA,
        "depends: [cap:beta]",
        "depends: [cap:beta] # one edge",
    ));
}

#[test]
fn rejects_bad_scalars_in_every_value_position() {
    assert_cases(
        ALPHA,
        &[
            (
                "depends: [cap:beta]",
                "depends: [cap:beta, \"x]",
                "unterminated quoted value",
            ),
            (
                "depends: [cap:beta]",
                "depends: [cap:beta, a: b]",
                "contains `: `",
            ),
            (
                "depends: [cap:beta]",
                "depends:\n  - \"cap:beta",
                "unterminated quoted value",
            ),
            (
                "depends: [cap:beta]",
                "depends:\n  - >",
                "block scalar header",
            ),
            (
                "  code: []\n",
                "  code:\n    - 'src/lib.rs\n",
                "unterminated quoted value",
            ),
            (
                "  code: []\n",
                "  code: [*anchor]",
                "alias `*anchor` is not accepted",
            ),
        ],
    );
    let items = "  - claim: \"unterminated\n";
    let mut repo = valid();
    repo.insert(
        PathBuf::from(CHANGE),
        fixtures::change_record(
            "add-alpha",
            "[cap:alpha, cap:beta]",
            "  - cap:alpha#contract\n  - cap:beta#purpose\n",
            items,
        ),
    );
    assert_hit(&check(&repo), "unterminated quoted value");
    let items = "  - claim: fine\n    outcome: > folded\n";
    repo.insert(
        PathBuf::from(CHANGE),
        fixtures::change_record(
            "add-alpha",
            "[cap:alpha, cap:beta]",
            "  - cap:alpha#contract\n  - cap:beta#purpose\n",
            items,
        ),
    );
    assert_hit(
        &check(&repo),
        "cannot start with the YAML indicator character",
    );
}

// --- PROPOSED markers -------------------------------------------------------

#[test]
fn rejects_lowercase_proposed_marker() {
    assert_cases(
        BETA,
        &[(
            "> [!PROPOSED change:add-alpha]",
            "> [!proposed change:add-alpha]",
            "malformed PROPOSED marker",
        )],
    );
    let violations = mutated(
        GAMMA,
        "# gamma\n",
        "# gamma\n\n> [!Proposed change:add-alpha]\n> Left over.\n",
    );
    assert_hit(&violations, "malformed PROPOSED marker");
}

#[test]
fn rejects_proposed_marker_without_space() {
    assert_cases(
        BETA,
        &[(
            "> [!PROPOSED change:add-alpha]",
            ">[!PROPOSED change:add-alpha]",
            "malformed PROPOSED marker",
        )],
    );
}

#[test]
fn rejects_proposed_marker_with_extra_spacing_or_nesting() {
    for marker in [
        ">  [!PROPOSED change:add-alpha]",
        "> > [!PROPOSED change:add-alpha]",
        "- > [!PROPOSED change:add-alpha]",
        "[!PROPOSED change:add-alpha]",
    ] {
        let violations = mutated(BETA, "> [!PROPOSED change:add-alpha]", marker);
        assert_hit(&violations, "malformed PROPOSED marker");
    }
}

#[test]
fn rejects_designed_with_proposed_block() {
    let violations = mutated(
        ALPHA,
        "Text for Contract.\n",
        "> [!PROPOSED change:add-alpha]\n> Not yet.\n\nText for Contract.\n",
    );
    assert_hit(&violations, "state designed must have no PROPOSED blocks");
}

#[test]
fn rejects_proposed_section_missing_from_affects() {
    let violations = mutated(CHANGE, "  - cap:beta#purpose\n", "  - cap:beta#usage\n");
    assert_hit(
        &violations,
        "PROPOSED block in section `purpose` needs `cap:beta#purpose` in the affects of change `add-alpha`",
    );
}

#[test]
fn rejects_proposed_block_outside_a_section() {
    let violations = mutated(
        BETA,
        "# beta\n",
        "# beta\n\n> [!PROPOSED change:add-alpha]\n> Early.\n",
    );
    assert_hit(&violations, "PROPOSED block `add-alpha` outside a section");
}

// --- bindings by state ------------------------------------------------------

#[test]
fn rejects_bindings_before_delivery() {
    for (path, state) in [(ALPHA, "designed"), (BETA, "considered")] {
        for key in ["code", "tests", "examples"] {
            let violations = mutated(
                path,
                &format!("  {key}: []"),
                &format!("  {key}: [src/lib.rs]"),
            );
            assert_hit(
                &violations,
                &format!("state {state} must have empty bindings, but `{key}` is not empty"),
            );
        }
    }
}

#[test]
fn rejects_delivered_without_code_or_test_binding() {
    let violations = mutated(GAMMA, "  code: [src/lib.rs]", "  code: []");
    assert_hit(
        &violations,
        "state delivered must bind at least one `code` entry",
    );
    let violations = mutated(GAMMA, "  tests: [tests/it.rs]", "  tests: []");
    assert_hit(
        &violations,
        "state delivered must bind at least one `tests` entry",
    );
}

// --- links ------------------------------------------------------------------

#[test]
fn rejects_broken_reference_definition() {
    assert_cases(
        "README.md",
        &[
            (
                "# Readme",
                "# Readme\n\n[ref]: ./missing.md",
                "broken link `./missing.md`",
            ),
            (
                "# Readme",
                "# Readme\n\n  [ref]: <./missing.md> \"Title\"",
                "broken link `./missing.md`",
            ),
            (
                "# Readme",
                "# Readme\n\n[ref]: docs/capabilities/alpha.md#no-such",
                "broken anchor",
            ),
        ],
    );
}

#[test]
fn rejects_broken_href_and_src_attributes() {
    assert_cases(
        "README.md",
        &[
            (
                "# Readme",
                "# Readme\n\n<a href=\"docs/missing.md\">gone</a>",
                "broken link `docs/missing.md`",
            ),
            (
                "# Readme",
                "# Readme\n\n<img src='docs/missing.svg' width=\"2\">",
                "broken link `docs/missing.svg`",
            ),
            (
                "# Readme",
                "# Readme\n\n<a HREF=\"../../outside.md\">out</a>",
                "leaves the repository",
            ),
        ],
    );
}

#[test]
fn rejects_broken_link_outside_the_document_directories() {
    for path in [
        ".github/PULL_REQUEST_TEMPLATE.md",
        ".github/ISSUE_TEMPLATE/bug.md",
        "crates/tool/README.md",
    ] {
        let mut repo = valid();
        repo.insert(
            PathBuf::from(path),
            "See [gone](docs/missing.md).\n".to_string(),
        );
        assert_hit(&check(&repo), &format!("{path}: line 1: broken link"));
    }
}

#[test]
fn accepts_resolving_reference_and_attribute_links() {
    let mut repo = valid();
    repo.insert(
        PathBuf::from(".github/PULL_REQUEST_TEMPLATE.md"),
        "# Template\n\n[docs]: ../docs/architecture/overview.md\n\
         <img src=\"../docs/architecture/overview.md\" alt=\"x\">\n\
         <a href=\"https://example.invalid/a\">site</a> <a href=\"#template\">top</a>\n\
         A footnote[^1] and `[code]: ./missing.md` stay quiet.\n\n[^1]: A note.\n"
            .to_string(),
    );
    assert_clean(&check(&repo));
}

// --- moved outcomes, typed documents, directories ---------------------------

#[test]
fn rejects_moved_outcome_naming_a_missing_change() {
    let violations = mutated(
        CHANGE,
        "outcome: implemented",
        "outcome: moved:change:ghost",
    );
    assert_hit(
        &violations,
        "item moved to change `ghost`, but work/changes/ghost.md does not exist",
    );
}

#[test]
fn rejects_typed_document_outside_its_directory() {
    let capability_text = capability("stray", "designed", "[]", "");
    let architecture_text = valid()
        .remove(Path::new(OVERVIEW))
        .expect("fixture overview")
        .replace("overview", "stray");
    let change_text = fixtures::change_record("stray", "[cap:alpha]", "  - cap:alpha#usage\n", "");
    for (path, text, kind, home) in [
        (
            "docs/other/stray.md",
            &capability_text,
            "Capability",
            "docs/capabilities",
        ),
        ("docs/stray.md", &change_text, "Change", "work/changes"),
        (
            "work/stray.md",
            &architecture_text,
            "Architecture",
            "docs/architecture",
        ),
        (
            "stray.md",
            &capability_text,
            "Capability",
            "docs/capabilities",
        ),
    ] {
        let mut repo = valid();
        repo.insert(PathBuf::from(path), text.clone());
        assert_hit(
            &check(&repo),
            &format!("{path}: `type`: a {kind} document belongs directly in {home}"),
        );
    }
}

#[test]
fn rejects_non_markdown_file_in_a_document_directory() {
    for path in [
        "docs/capabilities/notes.txt",
        "docs/capabilities/ALPHA.MD",
        "docs/architecture/diagram.svg",
        "work/changes/draft.yaml",
        "docs/capabilities/sub/data.json",
    ] {
        let mut repo = valid();
        repo.insert(PathBuf::from(path), String::new());
        assert_hit(
            &check(&repo),
            &format!("{path}: only Markdown documents belong in a document directory"),
        );
    }
}

#[test]
fn accepts_a_generated_capability_index() {
    let mut repo = valid();
    repo.insert(
        PathBuf::from("docs/capabilities/index.md"),
        "<!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n".to_string(),
    );
    assert_clean(&check(&repo));
}

// --- audit: one fixture for every other implemented rule --------------------

#[test]
fn rejects_empty_or_non_scalar_title() {
    assert_cases(
        ALPHA,
        &[
            (
                "title: \"Title of alpha\"",
                "title: \"\"",
                "`title`: must be a non-empty string",
            ),
            (
                "title: \"Title of alpha\"",
                "title: [a, b]",
                "`title`: must be a non-empty string",
            ),
            (
                "description: A description of alpha.",
                "description:",
                "`description`: must be a non-empty string",
            ),
        ],
    );
}

#[test]
fn rejects_bad_id_prefix_or_slug() {
    assert_cases(
        ALPHA,
        &[
            ("id: cap:alpha", "id: change:alpha", "must be `cap:<slug>`"),
            ("id: cap:alpha", "id: cap:Alpha", "must be `cap:<slug>`"),
            ("id: cap:alpha", "id: cap:a--b", "must be `cap:<slug>`"),
            ("id: cap:alpha", "id: cap:al_pha", "must be `cap:<slug>`"),
        ],
    );
    assert_cases(
        CHANGE,
        &[(
            "id: change:add-alpha",
            "id: cap:add-alpha",
            "must be `change:<slug>`",
        )],
    );
    assert_cases(
        OVERVIEW,
        &[("id: arch:overview", "id: overview", "must be `arch:<slug>`")],
    );
}

#[test]
fn rejects_non_scalar_state_and_scalar_bindings() {
    assert_cases(
        ALPHA,
        &[
            (
                "state: designed",
                "state: [designed]",
                "`state`: must be a scalar",
            ),
            (
                "bindings:\n  code: []\n  tests: []\n  examples: []\n",
                "bindings: none\n",
                "`bindings`: must be",
            ),
        ],
    );
}

#[test]
fn rejects_bad_depends_shapes() {
    assert_cases(
        ALPHA,
        &[
            (
                "depends: [cap:beta]",
                "depends: [beta]",
                "`depends`: entry `beta` is not a cap id",
            ),
            (
                "depends: [cap:beta]",
                "depends: cap:beta",
                "`depends`: must be a list of cap ids",
            ),
            (
                "depends: [cap:beta]",
                "depends: [cap:Beta]",
                "is not a cap id",
            ),
        ],
    );
}

#[test]
fn rejects_self_and_three_node_dependency_cycles() {
    assert_cases(
        ALPHA,
        &[(
            "depends: [cap:beta]",
            "depends: [cap:alpha]",
            "depends cycle: cap:alpha -> cap:alpha",
        )],
    );
    assert_cases(
        BETA,
        &[(
            "depends: []",
            "depends: [cap:gamma]",
            "depends cycle: cap:alpha -> cap:beta -> cap:gamma -> cap:alpha",
        )],
    );
}

#[test]
fn rejects_proposed_change_that_does_not_list_the_capability() {
    let violations = mutated(
        CHANGE,
        "changes: [cap:alpha, cap:beta]",
        "changes: [cap:alpha]",
    );
    assert_hit(
        &violations,
        "PROPOSED change `add-alpha` does not list cap:beta",
    );
}

#[test]
fn rejects_bad_changes_list() {
    assert_cases(
        CHANGE,
        &[
            (
                "changes: [cap:alpha, cap:beta]",
                "changes: []",
                "`changes`: must be a non-empty list",
            ),
            (
                "changes: [cap:alpha, cap:beta]",
                "changes: cap:alpha",
                "`changes`: must be a list of cap ids",
            ),
            (
                "changes: [cap:alpha, cap:beta]",
                "changes: [cap:alpha, beta]",
                "`changes`: entry `beta` is not a cap id",
            ),
        ],
    );
}

#[test]
fn rejects_bad_affects_shapes() {
    assert_cases(
        CHANGE,
        &[
            (
                "affects:\n  - cap:alpha#contract\n  - cap:beta#purpose\n",
                "affects: cap:alpha#contract\n",
                "`affects`: must be a list of strings",
            ),
            (
                "cap:alpha#contract",
                "alpha-contract",
                "must be `cap:<slug>#<section>`",
            ),
            (
                "cap:alpha#contract",
                "cap:alpha",
                "must be `cap:<slug>#<section>`",
            ),
        ],
    );
}

#[test]
fn rejects_non_scalar_contract_effect() {
    assert_cases(
        CHANGE,
        &[(
            "contract_effect: changes",
            "contract_effect: [none]",
            "bad contract_effect `<not a scalar>`",
        )],
    );
}

/// Replace the `add-alpha` change record with one whose `items` block is given.
fn with_items(items: &str) -> Vec<String> {
    let mut repo = valid();
    repo.insert(
        PathBuf::from(CHANGE),
        fixtures::change_record(
            "add-alpha",
            "[cap:alpha, cap:beta]",
            "  - cap:alpha#contract\n  - cap:beta#purpose\n",
            items,
        ),
    );
    check(&repo)
}

#[test]
fn rejects_item_shape_errors() {
    for (items, needle) in [
        (
            "  - claim: \"x\"\n    owner: me\n",
            "unknown item key `owner`",
        ),
        (
            "  - outcome: dropped\n",
            "every item needs a non-empty `claim`",
        ),
        ("  - claim: \"\"\n", "every item needs a non-empty `claim`"),
        (
            "  - claim: \"x\"\n    claim: \"y\"\n",
            "duplicate key `claim`",
        ),
        (
            "  - claim: \"x\"\n  - plain\n",
            "a list mixes scalar items and mapping items",
        ),
        (
            "  - plain\n    outcome: dropped\n",
            "indentation must be a multiple of two spaces",
        ),
        ("  - claim: \"x\"\n    !!!\n", "tag `!!!` is not accepted"),
        ("  - claim: \"x\"\n  junk\n", "expected a `- ` list item"),
        ("  - one\n  - two\n", "`items`: must be a list of maps"),
    ] {
        let violations = with_items(items);
        assert!(
            violations.iter().any(|v| v.contains(needle)),
            "items {items:?} should report `{needle}`, got: {violations:#?}"
        );
    }
    assert_hit(
        &mutated(CHANGE, "items:\n", "items: nothing\nignored_by_test:\n"),
        "`items`: must be a list of maps",
    );
}

#[test]
fn rejects_frontmatter_shape_errors() {
    assert_cases(
        OVERVIEW,
        &[
            (
                "title: Overview",
                "title: Overview\ntitle: Again",
                "line 5, column 1: duplicate key `title`",
            ),
            (
                "title: Overview",
                "title: Overview\nBad Key: 1",
                "line 5, column 1: key `Bad Key` holds characters outside",
            ),
        ],
    );
    assert_cases(
        ALPHA,
        &[
            (
                "  code: []\n",
                "  code: []\n  code: []\n",
                "duplicate key `code`",
            ),
            (
                "  code: []\n",
                "  code: foo\n",
                "binding `code` must be a list of strings",
            ),
            (
                "  code: []\n",
                "  code: [a\n",
                "flow sequence is not closed",
            ),
            (
                "  code: []\n",
                "  code: [a,,b]\n",
                "flow sequence has an empty entry",
            ),
            (
                "  code: []\n",
                "  code: []\n    - stray\n",
                "line 11, column 3: indentation must be a multiple of two spaces",
            ),
            (
                "  code: []\n",
                "  code: []\n      deep\n",
                "line 11, column 3: indentation must be a multiple of two spaces",
            ),
        ],
    );
    assert_cases(
        ALPHA,
        &[(
            "type: Capability",
            "title: again\ntype: Capability",
            "duplicate key",
        )],
    );
}

#[test]
fn rejects_missing_type_key() {
    assert_cases(
        OVERVIEW,
        &[("type: Architecture\n", "", "missing required key `type`")],
    );
}

#[test]
fn rejects_duplicate_section() {
    assert_cases(
        ALPHA,
        &[(
            "## Usage\n\nText for Usage.\n",
            "## Usage\n\nText for Usage.\n\n## Usage\n\nAgain.\n",
            "a section appears more than once",
        )],
    );
}

#[test]
fn rejects_generated_marker_errors() {
    assert_cases(
        OVERVIEW,
        &[
            (
                "Body text.",
                "Body text.\n\n<!-- raf:generated:end -->\n",
                "generated end marker without a start",
            ),
            (
                "Body text.",
                "Body text.\n\n<!-- raf:generated:a -->\n<!-- raf:generated:b -->\n<!-- raf:generated:end -->\n",
                "a generated block opens inside the open block",
            ),
            (
                "Body text.",
                "Body text.\n\n<!-- raf:generated: -->\n",
                "malformed generated marker",
            ),
            (
                "Body text.",
                "Body text.\n\n<!-- raf:generated:Bad Kind -->\n",
                "malformed generated marker",
            ),
            (
                "Body text.",
                "Body text.\n\n<!-- raf:generated:matrix\n",
                "malformed generated marker",
            ),
        ],
    );
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

// --- generated index, placeholder spellings, unquoted attributes ------------

/// Assembled in pieces so that this file carries no SPDX tag of its own here.
const SPDX_ID: &str = concat!("SPDX-License-", "Identifier: CC-BY-4.0");

fn index_repo(text: &str) -> Vec<String> {
    let mut repo = valid();
    repo.insert(
        PathBuf::from("docs/capabilities/index.md"),
        text.to_string(),
    );
    check(&repo)
}

#[test]
fn accepts_a_generated_index_with_an_spdx_header() {
    let header = format!("<!--\nSPDX-FileCopyrightText: 2026 Example\n{SPDX_ID}\n-->\n\n");
    let body = "<!-- raf:generated:catalogue -->\n| alpha |\n\n<!-- raf:generated:end -->\n";
    assert_clean(&index_repo(&format!("{header}{body}")));
    assert_clean(&index_repo(&format!("<!-- {SPDX_ID} -->\n{body}")));
}

#[test]
fn rejects_an_index_with_prose_outside_the_blocks() {
    let block = "<!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n";
    for text in [
        format!("# Index\n\n{block}"),
        format!("{block}\nHand-written note.\n"),
        format!("<!--\n{SPDX_ID}\nprose\n-->\n{block}"),
    ] {
        assert_hit(
            &index_repo(&text),
            "hand-written capability index is not allowed",
        );
    }
}

#[test]
fn rejects_an_index_whose_marker_is_only_in_a_code_span() {
    let text = "Generated by `<!-- raf:generated:catalogue -->` honestly.\n";
    assert_hit(&index_repo(text), "text outside a generated block");
    let fenced_in_span =
        "`<!-- raf:generated:catalogue -->`\n| alpha |\n`<!-- raf:generated:end -->`\n";
    assert_hit(
        &index_repo(fenced_in_span),
        "hand-written capability index is not allowed",
    );
}

#[test]
fn rejects_an_index_with_a_mid_line_marker() {
    for text in [
        "prose <!-- raf:generated:catalogue --> | alpha |\n<!-- raf:generated:end -->\n",
        "<!-- raf:generated:catalogue -->\n| alpha |\ntrailing <!-- raf:generated:end -->\n x\n",
        "  <!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n",
        "<!-- raf:generated:catalogue -->\n",
        "\n",
    ] {
        assert_hit(
            &index_repo(text),
            "hand-written capability index is not allowed",
        );
    }
}

#[test]
fn rejects_placeholder_spellings() {
    for spelling in [
        concat!("t", "bd"),
        concat!("T.B", ".D."),
        concat!("t.b", ".d"),
        concat!("To", "Do"),
        concat!("fix", "Me"),
        concat!("Fix", "ME:"),
        concat!("x", "xX"),
        concat!("to  be", "\twritten"),
        concat!("TO BE ", "WRITTEN"),
    ] {
        let violations = mutated(
            OVERVIEW,
            "Body text.",
            &format!("Body text. Still {spelling} soon."),
        );
        assert_hit(&violations, "is not resolved text");
    }
}

#[test]
fn placeholder_scan_spares_longer_words_and_dotted_prose() {
    let text = format!(
        "Body text. {} and {} and e.g. a.b.c, {}s.\n",
        concat!("t", "bdx"),
        concat!("to", "dos"),
        concat!("x", "xxx"),
    );
    assert_clean(&mutated(OVERVIEW, "Body text.\n", &text));
}

#[test]
fn rejects_broken_unquoted_attribute_values() {
    assert_cases(
        "README.md",
        &[
            (
                "# Readme",
                "# Readme\n\n<a href=docs/missing.md>x</a>",
                "broken link `docs/missing.md`",
            ),
            (
                "# Readme",
                "# Readme\n\n<img src=docs/missing.svg />",
                "broken link `docs/missing.svg`",
            ),
            (
                "# Readme",
                "# Readme\n\n<a href=../../outside.md>x</a>",
                "leaves the repository",
            ),
        ],
    );
    let mut repo = valid();
    repo.insert(
        PathBuf::from(".github/note.md"),
        "# Note\n\n<a href=../docs/architecture/overview.md>ok</a>\n".to_string(),
    );
    assert_clean(&check(&repo));
}

#[test]
fn rejects_an_index_header_with_unknown_tags_or_in_the_wrong_place() {
    let block = "<!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n";
    for text in [
        format!("<!-- SPDX-Anything: x -->\n{block}"),
        format!("<!--\nSPDX-Anything: x\n-->\n{block}"),
        format!("{block}<!-- {SPDX_ID} -->\n"),
        format!("{block}\n<!--\n{SPDX_ID}\n-->\n{block}"),
    ] {
        assert_hit(
            &index_repo(&text),
            "hand-written capability index is not allowed",
        );
    }
}

#[test]
fn rejects_an_index_with_an_indented_marker() {
    for indent in [" ", "    ", "\t"] {
        let text = format!(
            "{indent}<!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n"
        );
        assert_hit(
            &index_repo(&text),
            "hand-written capability index is not allowed",
        );
        let text = format!(
            "<!-- raf:generated:catalogue -->\n| alpha |\n{indent}<!-- raf:generated:end -->\n"
        );
        assert_hit(
            &index_repo(&text),
            "hand-written capability index is not allowed",
        );
    }
}

#[test]
fn reads_attributes_with_spaces_around_the_equals_sign() {
    assert_cases(
        "README.md",
        &[
            (
                "# Readme",
                "# Readme\n\n<a href = \"docs/missing.md\">x</a>",
                "broken link `docs/missing.md`",
            ),
            (
                "# Readme",
                "# Readme\n\n<img src\t=  docs/missing.svg>",
                "broken link `docs/missing.svg`",
            ),
        ],
    );
}

#[test]
fn does_not_read_attributes_that_only_end_in_href_or_src() {
    let text = "# Readme\n\n<a data-href=\"docs/missing.md\" x-src='docs/none.svg' data_src=docs/n.svg>x</a>\n\
                Use the href attribute and src = nothing here.\n";
    assert_clean(&mutated("README.md", "# Readme\n", text));
}

#[test]
fn rejects_broken_attributes_in_tags_split_across_lines() {
    for (text, needle) in [
        (
            "<img\n  src=\"docs/missing.svg\"\n  alt=\"x\">\n",
            "README.md: line 4: broken link `docs/missing.svg`",
        ),
        (
            "<img\n  alt=\"x\"\n  src=docs/missing.svg />\n",
            "README.md: line 5: broken link `docs/missing.svg`",
        ),
        (
            "<img\nalt=\"x\"\nsrc = 'docs/missing.svg'\nwidth=\"2\">\n",
            "README.md: line 5: broken link `docs/missing.svg`",
        ),
        (
            "<a\n  href=\"docs/missing.md\">gone</a>\n",
            "README.md: line 4: broken link `docs/missing.md`",
        ),
        (
            "<a title=\"x>y\" href=\"./nope.md\">gone</a>\n",
            "README.md: line 3: broken link `./nope.md`",
        ),
        (
            "<a title=\"two\nlines>\" href=\"./nope.md\">gone</a>\n",
            "README.md: line 4: broken link `./nope.md`",
        ),
    ] {
        let violations = mutated("README.md", "# Readme\n", &format!("# Readme\n\n{text}"));
        assert_hit(&violations, needle);
    }
}

#[test]
fn tag_scan_spares_prose_autolinks_and_closed_tags() {
    let text = "# Readme\n\nWrite src = nothing or href = nothing; if a < b then x > y.\n\
                <https://example.invalid/?src=a&href=b>\n\
                <p align=\"center\">\n  <img src=\"docs/architecture/overview.md\" alt=\"ok\">\n</p>\n\
                After the tag, src = nothing is prose again.\n";
    assert_clean(&mutated("README.md", "# Readme\n", text));
}

#[test]
fn rejects_an_index_header_tag_without_a_value() {
    let block = "<!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n";
    for header in [
        "<!-- SPDX-FileCopyrightText: -->\n".to_string(),
        "<!--\nSPDX-FileCopyrightText:   \n-->\n".to_string(),
        format!("<!--\n{SPDX_ID}\nSPDX-FileCopyrightText:\n-->\n"),
    ] {
        assert_hit(
            &index_repo(&format!("{header}{block}")),
            "hand-written capability index is not allowed",
        );
    }
}

#[test]
fn tag_state_does_not_survive_a_blank_line_or_a_fence() {
    let broken = "<img src=\"./nope.png\">\n";
    for prose in [
        "a<b it's fine\n\n",
        "a<b \"quoted\n\n",
        "a<b it's fine\n\n```text\ncode\n```\n",
    ] {
        let violations = mutated(
            "README.md",
            "# Readme\n",
            &format!("# Readme\n\n{prose}{broken}"),
        );
        assert!(
            violations
                .iter()
                .any(|v| v.contains("broken link `./nope.png`")),
            "{prose:?}: {violations:#?}"
        );
    }
}

#[test]
fn reads_an_attribute_value_that_starts_on_the_next_line() {
    for (text, needle) in [
        (
            "<img src=\n  \"docs/missing.svg\" alt=\"x\">\n",
            "README.md: line 4: broken link `docs/missing.svg`",
        ),
        (
            "<a href =\n  docs/missing.md>gone</a>\n",
            "README.md: line 4: broken link `docs/missing.md`",
        ),
    ] {
        let violations = mutated("README.md", "# Readme\n", &format!("# Readme\n\n{text}"));
        assert_hit(&violations, needle);
    }
    let fine = "<img src=\n  \"docs/architecture/overview.md\">\n";
    assert_clean(&mutated(
        "README.md",
        "# Readme\n",
        &format!("# Readme\n\n{fine}"),
    ));
}

// --- bindings ---------------------------------------------------------------

const DEMO_LIB: &str = "macro_rules! make {\n    ($name:ident) => {\n        pub fn $name() {}\n    };\n}\nmake!(generated);\npub fn real() {}\nmod body;\n";

fn demo_workspace() -> Workspace {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        ),
        ("src/lib.rs", DEMO_LIB),
        // `blocks.rs` sorts before `mod.rs`, which declares it, as in raf-okf.
        ("src/body/mod.rs", "mod blocks;\npub fn in_body() {}\n"),
        ("src/body/blocks.rs", "pub(crate) struct FenceBlock;\n"),
        ("tests/it.rs", "fn case() {}\n"),
    ]);
    Workspace::load(&tree, &["."]).expect("the demo workspace loads")
}

/// The fixture with `gamma` delivered and bound as given.
fn bound(code: &str, tests: &str, examples: &str) -> Repo {
    let mut repo = valid();
    let gamma = repo.get_mut(Path::new(GAMMA)).expect("fixture gamma");
    *gamma = gamma
        .replace("  code: [src/lib.rs]", &format!("  code: {code}"))
        .replace("  tests: [tests/it.rs]", &format!("  tests: {tests}"))
        .replace("  examples: []", &format!("  examples: {examples}"));
    repo
}

fn binding_violations(code: &str, tests: &str, examples: &str) -> Vec<String> {
    let analysis = analyse(&bound(code, tests, examples));
    assert_clean(&analysis.violations);
    check_bindings(&analysis.found, &demo_workspace())
}

#[test]
fn bindings_that_name_real_symbols_resolve() {
    assert_clean(&binding_violations(
        "[demo::real]",
        "[demo::test_crate::it::case]",
        "[demo::real]",
    ));
}

#[test]
fn a_binding_to_a_missing_path_is_reported_with_its_capability() {
    let violations = binding_violations("[demo::nope]", "[demo::test_crate::it::case]", "[]");
    assert_eq!(violations.len(), 1, "{violations:#?}");
    assert_hit(
        &violations,
        "docs/capabilities/gamma.md: cap:gamma code binding `demo::nope` does not resolve: `demo::nope` names nothing in the workspace",
    );
}

#[test]
fn a_binding_to_a_macro_generated_function_is_not_found() {
    let violations = binding_violations("[demo::generated]", "[demo::test_crate::it::case]", "[]");
    assert_eq!(violations.len(), 1, "{violations:#?}");
    assert_hit(
        &violations,
        "cap:gamma code binding `demo::generated` does not resolve: `demo::generated` names nothing in the workspace",
    );
}

#[test]
fn a_binding_to_an_item_under_the_module_that_declares_its_file_is_not_found() {
    let violations = binding_violations(
        "[demo::body::FenceBlock]",
        "[demo::test_crate::it::case]",
        "[]",
    );
    assert_eq!(violations.len(), 1, "{violations:#?}");
    assert_hit(
        &violations,
        "cap:gamma code binding `demo::body::FenceBlock` does not resolve: `demo::body::FenceBlock` names nothing in the workspace",
    );
    assert_clean(&binding_violations(
        "[demo::body::blocks::FenceBlock, demo::body::in_body]",
        "[demo::test_crate::it::case]",
        "[]",
    ));
}

#[test]
fn every_unresolved_binding_of_every_list_is_reported() {
    let violations = binding_violations(
        "[demo::real, demo::nope, demo::generated]",
        "[demo::test_crate::it::missing]",
        "[demo::real, demo::absent]",
    );
    assert_eq!(violations.len(), 4, "{violations:#?}");
    for needle in [
        "code binding `demo::nope`",
        "code binding `demo::generated`",
        "tests binding `demo::test_crate::it::missing`",
        "examples binding `demo::absent`",
    ] {
        assert_hit(&violations, needle);
    }
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

// --- Markdown outside the profile, read through Body -------------------------

/// The fixture with `text` added at `path`.
fn with_file(path: &str, text: &str) -> Vec<String> {
    let mut repo = valid();
    repo.insert(PathBuf::from(path), text.to_string());
    check(&repo)
}

#[test]
fn rejects_a_link_split_across_lines_in_any_tracked_markdown_file() {
    for path in [
        "README.md",
        "docs/notes.md",
        ".github/PULL_REQUEST_TEMPLATE.md",
        "crates/tool/README.md",
    ] {
        for (text, needle) in [
            (
                "# Notes\n\n[text\nmore](README.md)\n",
                "line 3: a link split across lines (Inline) is not allowed",
            ),
            (
                "# Notes\n\n[a](\nREADME.md)\n",
                "line 3: a link split across lines (Inline) is not allowed",
            ),
            (
                "# Notes\n\ntext\n\n![alt\ntext](README.md)\n",
                "line 5: a link split across lines (Image) is not allowed",
            ),
        ] {
            let violations = with_file(path, text);
            assert_hit(&violations, &format!("{path}: {needle}"));
        }
    }
}

#[test]
fn rejects_a_link_split_across_lines_in_a_profile_document() {
    let violations = mutated(
        OVERVIEW,
        "Body text.",
        "Body text [split\nlink](../../README.md).",
    );
    assert_hit(
        &violations,
        "docs/architecture/overview.md: line 10: a link split across lines (Inline) is not allowed",
    );
    assert_eq!(
        violations
            .iter()
            .filter(|v| v.contains("split across lines"))
            .count(),
        1,
        "{violations:#?}"
    );
}

#[test]
fn accepts_one_line_links_and_links_in_fences_beside_the_split_rule() {
    let text = "# Notes\n\n[a](README.md) and [b](README.md)\n[c](README.md)\n\n\
                ```text\n[split\nlink](missing.md)\n```\n\nA `[code span](missing.md)` too.\n";
    assert_clean(&with_file("NOTES.md", text));
}

#[test]
fn rejects_an_empty_link_target() {
    for (text, line) in [
        ("# Notes\n\n[a]()\n", 3),
        ("# Notes\n\n![img]()\n", 3),
        ("# Notes\n\n[a]( )\n", 3),
    ] {
        let violations = with_file("NOTES.md", text);
        assert_hit(
            &violations,
            &format!("NOTES.md: line {line}: the link has an empty target"),
        );
    }
    assert_clean(&with_file(
        "NOTES.md",
        "# Notes\n\n[a](#notes) and `[b]()`\n",
    ));
}

#[test]
fn does_not_read_links_or_placeholders_inside_html_comments() {
    let broken = "<!-- [gone](missing.md) <a href=\"missing.md\">x</a> -->\n";
    let multi = "<!--\n[gone](missing.md)\n[ref]: ./missing.md\nstill hidden\n-->\n";
    let token = concat!("T", "BD");
    let hidden = format!("<!-- {token} and {} -->\n", concat!("to", "do"));
    for body in [broken, multi, hidden.as_str()] {
        assert_clean(&with_file("NOTES.md", &format!("# Notes\n\n{body}")));
    }
    let violations = mutated(
        OVERVIEW,
        "Body text.",
        &format!("Body text.\n\n{broken}{hidden}"),
    );
    assert_clean(&violations);
    let visible = format!("# Notes\n\n<!-- hidden -->\n[gone](missing.md) {token}\n");
    let violations = with_file("NOTES.md", &visible);
    assert_hit(&violations, "NOTES.md: line 4: broken link `missing.md`");
    assert_hit(&violations, "NOTES.md: line 4: placeholder");
}

#[test]
fn reads_placeholders_in_plain_markdown_outside_code() {
    let token = concat!("fix", "me");
    let violations = with_file("NOTES.md", &format!("# Notes\n\nStill {token} here.\n"));
    assert_hit(
        &violations,
        &format!("NOTES.md: line 3: placeholder `{token}` is not resolved text"),
    );
    let spared = format!("# Notes\n\nUse `{token}` and {token}s.\n\n```text\n{token}\n```\n");
    assert_clean(&with_file("NOTES.md", &spared));
}

#[test]
fn reports_marker_errors_of_plain_markdown_once() {
    for (text, needle) in [
        (
            "# Notes\n\n> [!PROPOSED add-alpha]\n",
            "NOTES.md: unparseable document: line 3, column 1: malformed PROPOSED marker",
        ),
        (
            "# Notes\n\n<!-- raf:generated:matrix -->\n",
            "NOTES.md: unparseable document: line 3, column 1: the generated block `matrix` is never closed",
        ),
        (
            "# Notes\n\n<!-- raf:generated:end -->\n",
            "NOTES.md: unparseable document: line 3, column 1: generated end marker without a start",
        ),
    ] {
        let violations = with_file("NOTES.md", text);
        assert_hit(&violations, needle);
        assert_eq!(violations.len(), 1, "{violations:#?}");
    }
    let violations = with_file(".github/note.md", "# Note\n\n> [!PROPOSED add-alpha]\n");
    assert_hit(&violations, ".github/note.md: unparseable document: line 3");
}

#[test]
fn reads_a_proposed_block_in_plain_markdown_as_outside_a_capability() {
    let text = "# Notes\n\n> [!PROPOSED change:add-alpha]\n> Candidate text.\n";
    let violations = with_file("NOTES.md", text);
    assert_hit(
        &violations,
        "NOTES.md: line 3: PROPOSED block `add-alpha` outside a capability",
    );
    let orphan = text.replace("add-alpha", "ghost");
    assert_hit(
        &with_file("NOTES.md", &orphan),
        "NOTES.md: orphan PROPOSED marker: work/changes/ghost.md does not exist",
    );
    assert_clean(&with_file(
        "crates/tool/README.md",
        "# Tool\n\n> [!PROPOSED change:add-alpha]\n> Candidate text.\n",
    ));
}

#[test]
fn reads_markdown_outside_the_document_directories_with_or_without_frontmatter() {
    let template = "---\nname: Bug\nabout: Report a bug\ntitle: \"[Bug] \"\n---\n\n# Bug\n\nSee [the overview](../../docs/architecture/overview.md).\n";
    assert_clean(&with_file(".github/ISSUE_TEMPLATE/bug.md", template));
    let broken = template.replace("overview.md", "missing.md");
    assert_hit(
        &with_file(".github/ISSUE_TEMPLATE/bug.md", &broken),
        ".github/ISSUE_TEMPLATE/bug.md: line 9: broken link `../../docs/architecture/missing.md`",
    );
}

#[test]
fn rejects_unclosed_frontmatter_in_plain_markdown() {
    let violations = with_file("NOTES.md", "---\n# Notes\n\nNo closing fence.\n");
    assert_hit(&violations, "NOTES.md: unparseable document: line 1");
}

#[test]
fn rejects_absolute_and_empty_link_targets_beside_relative_ones() {
    assert_hit(
        &with_file("NOTES.md", "# Notes\n\n[a](/docs/x.md)\n"),
        "NOTES.md: line 3: link `/docs/x.md` is an absolute path",
    );
    assert_clean(&with_file(
        "NOTES.md",
        "# Notes\n\n[a](docs/architecture/overview.md)\n",
    ));
}

#[test]
fn rejects_a_link_whose_target_cannot_be_read_for_its_anchor() {
    let mut repo = valid();
    repo.insert(
        PathBuf::from("BAD.md"),
        "# Bad\n\n> [!PROPOSED add-alpha]\n".to_string(),
    );
    repo.insert(PathBuf::from("LINK.md"), "[a](BAD.md#bad)\n".to_string());
    let violations = check(&repo);
    assert_hit(
        &violations,
        "LINK.md: line 1: link `BAD.md#bad` targets `BAD.md`, which cannot be read as Markdown",
    );
    assert_hit(&violations, "BAD.md: unparseable document");
}
