// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Two-way coverage: the public items that no capability binds (code to
//! documentation) and the tests that no capability binds (tests to claims).
//!
//! [`Workplan::coverage_report`] reads every package of the source workspace
//! and returns a [`CoverageReport`]; `check` enforces the same computation for
//! the crates the coverage record `work/coverage/default.md` lists in `scope`.
//!
//! # The public surface
//!
//! The public items of a package are the items of the public surface of each
//! of its library and binary targets, as
//! [`Workspace::exported_items`](raf_source::Workspace::exported_items) lists
//! them: what another crate can name through the crate root, a `pub` item of a
//! private module only through a `pub` re-export, the members of an exported
//! type or trait under its path, and nothing that `#[doc(hidden)]` hides. An
//! item exported under several paths is one item, the defining one, and is
//! reported under the first of its exported paths. A binary's items are
//! `pub(crate)` or private in practice (no other crate can name them), so its
//! surface is usually empty and the code direction asks nothing of it.
//!
//! A public re-export that names no item of the target is a
//! [`Reexport`]. One that names an item of another crate
//! of the workspace is counted in that crate, and one of a crate outside the
//! workspace (its written path starts with a name that is no crate of the
//! workspace and that the declaring module does not resolve) is not counted.
//! Any other, a type a macro generates or an enum variant the index cannot
//! follow, is an entry of the code direction of its own kind,
//! [`GapKind::Reexport`]: it is never skipped. It is covered when a `code`
//! binding covers a module that its exported path, or the path it names from
//! the declaring module, is below, since the re-export itself resolves to no
//! item a binding could name. The declaring module is the innermost inline
//! module whose braces hold the declaration, or else the module of its file,
//! and the path is read from it as `rustc` reads a `use` path: `crate::` from
//! the crate root, `self::` and a bare path from the module, and each
//! `super::` one module up.
//!
//! A surface that cannot be listed is a [`SurfaceError`] of the report, never
//! an empty surface: the item-bytes budget of the load passed by the listing
//! ([`SurfaceFailure::Budget`]) or by deciding whether the public globs carry
//! a re-export ([`SurfaceFailure::CarryBudget`], the refusal for several
//! binders of one name), or a public name that is ambiguous, goes through a
//! cycle of imports or reads a glob of nothing ([`SurfaceFailure::Unresolved`]).
//! What the listing does not model is a stated limit of the surface read here
//! (see the Constraints of `raf-source`'s exported items): a name that a local
//! `pub(crate)` item and a carried re-export of another crate share is listed
//! as the re-export, so that item is not counted.
//!
//! # Covered, excluded, gap
//!
//! A public item is covered when a `code` binding of a capability resolves to
//! it or to an item it is below: binding a type covers its methods and
//! associated items, binding a trait its members, and binding a module every
//! item below it, as a module binding also stands for the module's whole text
//! in [`Digest::symbol`](crate::digest::Digest::symbol). A binding is followed to the definition, so an item bound
//! through a re-export is covered under every path it is exported by. A crate
//! root covers nothing below it: a capability binds the modules and items it
//! explains, and a whole crate is taken out of enforcement by leaving it out
//! of `scope`. Only `code` bindings cover items; a documented item bound in
//! `examples` is a documentation test, not an explanation.
//!
//! A test is a function marked `#[test]` that the index knows: in an
//! integration test target, and in a `#[cfg(test)]` module of a library or a
//! binary (a unit test), which counts like any other. A documentation test is
//! not a test function of the index; it is bound in `examples`. A test is
//! covered when a `tests` binding resolves to it.
//!
//! An item or test that is not covered is excluded when an exclusion of the
//! coverage record matches it: an item path (`raf_okf::Body`) matches the item
//! exported or defined at that path, and a prefix (`raf_okf::body::*`) every
//! item or test whose exported or defining path is below the module. The first
//! exclusion that matches, in the order of the record, gives the reason the
//! report shows. Anything else is a [`Gap`].
//!
//! # What `check` enforces
//!
//! For each crate of `scope`, each gap is a finding: `coverage-item-unbound`
//! and `coverage-test-unbound` at the file and line of the item,
//! `coverage-reexport-unbound` at the declaration of an unresolved re-export,
//! and
//! `coverage-surface-unreadable` at the `scope` of the record for a surface
//! that cannot be listed. An exclusion that matches no public item and no test
//! of the workspace is `coverage-exclusion-unmatched`; an exclusion without a
//! reason, or of a whole crate, is refused by the profile when the record
//! loads. A repository with capability documents and no coverage record has
//! one `coverage-record-missing` finding: the record states what is enforced,
//! and `scope: []` enforces nothing, visibly. Crates outside `scope` are not
//! enforced and their gaps are in the report. While a capability document
//! exists and does not load, `check` reports no gap: what that document binds
//! is not in the graph, so each item and test it binds would read as unbound,
//! and its own findings already fail the check; the surfaces that cannot be
//! listed and the exclusions that match nothing are still reported.
//!
//! `check` lists the surface of each library and binary target of a crate of
//! `scope`, and of a crate that an exclusion names by its first path segment
//! (to judge whether the exclusion matches), once per load; a record whose
//! `scope` is empty and that has no exclusions lists none. The report lists
//! every library and binary target of the workspace once.
//!
//! Every list is sorted by package and path, so a report is the same on every
//! run over the same tree.

pub mod render;

#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;

use raf_okf::{CoverageRecord, DocType, Exclusion, ExclusionItem, Location};
use raf_source::{
    ExportError, ItemId, Reexport, ResolveError, SymbolKind, Target, TargetId, TargetKind,
    TestRole, Workspace,
};

use crate::finding::kind;
use crate::graph::{BindingKind, Graph, ScopeEdge};
use crate::work::{Work, add};
use crate::{Finding, Loaded, Workplan};

/// The path of the coverage record.
pub const RECORD_PATH: &str = "work/coverage/default.md";

/// What a gap or an exclusion is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum GapKind {
    /// An item of the public surface of a library or binary target.
    PublicItem,
    /// A public re-export of a library or binary target that names nothing
    /// the source index can follow, such as a type a macro generates or an
    /// enum variant, and that is not of a crate outside the workspace. It is
    /// in the code direction.
    Reexport,
    /// A function marked `#[test]`.
    Test,
}

impl GapKind {
    /// The name of the kind as the report writes it: `public item`,
    /// `unresolved re-export` or `test`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::PublicItem => "public item",
            Self::Reexport => "unresolved re-export",
            Self::Test => "test",
        }
    }

    /// Whether the kind is in the code direction (a public item or an
    /// unresolved re-export) rather than the tests direction.
    #[must_use]
    pub fn is_code(self) -> bool {
        !matches!(self, Self::Test)
    }
}

impl fmt::Display for GapKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A public item or a test that no capability binds and no exclusion matches.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Gap {
    kind: GapKind,
    package: String,
    path: String,
    item: String,
    file: String,
    line: usize,
}

impl Gap {
    /// Whether it is a public item or a test.
    #[must_use]
    pub fn kind(&self) -> GapKind {
        self.kind
    }

    /// The package name of the crate it is in.
    #[must_use]
    pub fn package(&self) -> &str {
        &self.package
    }

    /// The path another crate names it by (the first exported path of a
    /// public item), or the path of a test.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The path of the defining item; the same as [`Gap::path`] for a test
    /// and for an item exported where it is defined. For an unresolved
    /// re-export, the path its declaration names, read from the module that
    /// declares it.
    #[must_use]
    pub fn item(&self) -> &str {
        &self.item
    }

    /// The file that defines it, relative to the workspace root.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// The 1-based line of its name.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }
}

impl fmt::Display for Gap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} `{}` of {} at {}:{}",
            self.kind, self.path, self.package, self.file, self.line
        )
    }
}

/// A public item or test that no capability binds and that an exclusion of
/// the coverage record takes out, with the exclusion and its reason.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Excluded {
    gap: Gap,
    exclusion: String,
    reason: String,
}

impl Excluded {
    /// The item or test, as it would be reported without the exclusion.
    #[must_use]
    pub fn gap(&self) -> &Gap {
        &self.gap
    }

    /// The exclusion as the record writes it: an item path or
    /// `<module>::*`.
    #[must_use]
    pub fn exclusion(&self) -> &str {
        &self.exclusion
    }

    /// The reason of the exclusion.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Why the public surface of a target could not be listed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SurfaceFailure {
    /// What the surface holds passes the item-bytes budget left after the
    /// load.
    Budget,
    /// Deciding whether the public globs carry the re-export at `path` passes
    /// the item-bytes budget, as for several binders of one carried name.
    CarryBudget {
        /// The exported path of the re-export being decided.
        path: String,
    },
    /// A public name is ambiguous, goes through a cycle of imports, or reads
    /// a glob whose path names nothing.
    Unresolved,
    /// Another failure of the listing; the message says which.
    Other,
}

impl SurfaceFailure {
    /// The name of the failure as the JSON form writes it.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Budget => "budget",
            Self::CarryBudget { .. } => "carry-budget",
            Self::Unresolved => "unresolved",
            Self::Other => "other",
        }
    }

    fn of(error: &ExportError) -> Self {
        match error {
            ExportError::Budget(_) => Self::Budget,
            ExportError::CarryBudget { path, .. } => Self::CarryBudget { path: path.clone() },
            ExportError::Resolve(_) => Self::Unresolved,
            _ => Self::Other,
        }
    }
}

/// A library or binary target whose public surface could not be listed: its
/// items are neither covered nor gaps, and the report says so.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SurfaceError {
    package: String,
    target: String,
    root: String,
    failure: SurfaceFailure,
    message: String,
}

impl SurfaceError {
    /// The package name.
    #[must_use]
    pub fn package(&self) -> &str {
        &self.package
    }

    /// The crate path of the target (`raf_okf`,
    /// `cargo_workplan::bin_crate::cargo_workplan`).
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// The root file of the target.
    #[must_use]
    pub fn root(&self) -> &str {
        &self.root
    }

    /// What failed.
    #[must_use]
    pub fn failure(&self) -> &SurfaceFailure {
        &self.failure
    }

    /// The error of the listing, with the file and line it names.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the public surface of `{}` ({}) cannot be listed: {}",
            self.target, self.package, self.message
        )
    }
}

/// One package of the workspace in a report: whether `check` enforces it,
/// and how many public items and tests were judged.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PackageCoverage {
    name: String,
    in_scope: bool,
    items: usize,
    covered_items: usize,
    tests: usize,
    covered_tests: usize,
}

impl PackageCoverage {
    /// The package name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the coverage record lists the package in `scope`.
    #[must_use]
    pub fn in_scope(&self) -> bool {
        self.in_scope
    }

    /// The public items of its listed surfaces.
    #[must_use]
    pub fn items(&self) -> usize {
        self.items
    }

    /// The public items a binding covers.
    #[must_use]
    pub fn covered_items(&self) -> usize {
        self.covered_items
    }

    /// Its test functions.
    #[must_use]
    pub fn tests(&self) -> usize {
        self.tests
    }

    /// The tests a binding covers.
    #[must_use]
    pub fn covered_tests(&self) -> usize {
        self.covered_tests
    }
}

/// The coverage of the workspace, or of one package of it: the gaps in each
/// direction, what the exclusions take out, and the surfaces that could not
/// be listed, each sorted by package and path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct CoverageReport {
    packages: Vec<PackageCoverage>,
    code: Vec<Gap>,
    tests: Vec<Gap>,
    excluded: Vec<Excluded>,
    errors: Vec<SurfaceError>,
}

impl CoverageReport {
    /// The packages, sorted by name.
    #[must_use]
    pub fn packages(&self) -> &[PackageCoverage] {
        &self.packages
    }

    /// The public items without a binding or an exclusion.
    #[must_use]
    pub fn code(&self) -> &[Gap] {
        &self.code
    }

    /// The tests without a binding or an exclusion.
    #[must_use]
    pub fn tests(&self) -> &[Gap] {
        &self.tests
    }

    /// The public items and tests an exclusion takes out.
    #[must_use]
    pub fn excluded(&self) -> &[Excluded] {
        &self.excluded
    }

    /// The surfaces that could not be listed.
    #[must_use]
    pub fn errors(&self) -> &[SurfaceError] {
        &self.errors
    }

    /// The gaps of one direction: [`CoverageReport::code`] or
    /// [`CoverageReport::tests`].
    #[must_use]
    pub fn gaps(&self, kind: GapKind) -> &[Gap] {
        if kind.is_code() {
            &self.code
        } else {
            &self.tests
        }
    }

    /// True when there is no gap in either direction and every surface was
    /// listed.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.code.is_empty() && self.tests.is_empty() && self.errors.is_empty()
    }

    /// The report of one package, by its name; `None` for a name that is no
    /// package of the report.
    #[must_use]
    pub fn package(&self, name: &str) -> Option<CoverageReport> {
        let package = self.packages.iter().find(|p| p.name == name)?;
        Some(Self {
            packages: vec![package.clone()],
            code: self
                .code
                .iter()
                .filter(|g| g.package == name)
                .cloned()
                .collect(),
            tests: self
                .tests
                .iter()
                .filter(|g| g.package == name)
                .cloned()
                .collect(),
            excluded: self
                .excluded
                .iter()
                .filter(|e| e.gap.package == name)
                .cloned()
                .collect(),
            errors: self
                .errors
                .iter()
                .filter(|e| e.package == name)
                .cloned()
                .collect(),
        })
    }
}

impl fmt::Display for CoverageReport {
    /// The tables of [`render::table`] for both directions, code first.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&render::table(self, GapKind::PublicItem))?;
        f.write_str("\n")?;
        f.write_str(&render::table(self, GapKind::Test))
    }
}

impl Workplan<'_> {
    /// The coverage of every package of the source workspace, in both
    /// directions, judged against the bindings of the loaded capabilities and
    /// the exclusions of the coverage record (none without a record). Each
    /// package says whether the record's `scope` lists it. Nothing is
    /// written and no process is started.
    ///
    /// ```rust
    /// use raf_source::{MemoryTree, Workspace};
    /// use raf_workplan::Workplan;
    /// use raf_workplan::coverage::GapKind;
    ///
    /// let source = Workspace::load(
    ///     &MemoryTree::from([
    ///         ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///         ("src/lib.rs", "pub fn parse() {}\npub fn parse_partial() {}\n"),
    ///     ]),
    ///     &["."],
    /// )?;
    /// let plan = Workplan::load(&MemoryTree::from([("README.md", "# Demo\n")]), &source);
    /// let report = plan.coverage_report();
    /// let gaps: Vec<&str> = report.code().iter().map(|gap| gap.path()).collect();
    /// assert_eq!(gaps, ["demo::parse", "demo::parse_partial"]);
    /// assert_eq!(report.code()[0].kind(), GapKind::PublicItem);
    /// assert!(!report.packages()[0].in_scope());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn coverage_report(&self) -> CoverageReport {
        let all: BTreeSet<&str> = self
            .source()
            .packages()
            .iter()
            .map(|package| package.name.as_str())
            .collect();
        compute(
            self.source(),
            self.graph(),
            self.coverage().map(Loaded::value),
            &all,
        )
        .report
    }
}

/// The findings `check` adds for coverage: see the module documentation.
pub(crate) fn check(plan: &Workplan<'_>) -> Vec<Finding> {
    let Some(loaded) = plan.coverage() else {
        if plan.is_present("coverage:default") || !plan.has_capability_files() {
            return Vec::new();
        }
        return vec![Finding::new(
            kind::COVERAGE_RECORD_MISSING,
            RECORD_PATH,
            Location::Document,
            format!(
                "the repository has capability documents and no coverage record {RECORD_PATH}, so it does not say which crates must be covered"
            ),
            "add the coverage record with the crates whose coverage is enforced in `scope` and an exclusion with a reason for each item or test that supports no documented guarantee; `scope: []` enforces none",
        )];
    };
    let record = loaded.value();
    if record.scope.is_empty() && record.exclusions.is_empty() {
        return Vec::new();
    }
    let scope: BTreeSet<&str> = plan
        .graph()
        .scope()
        .iter()
        .map(ScopeEdge::package)
        .collect();
    let mut needed = scope.clone();
    for exclusion in &record.exclusions {
        let first = first_segment(exclusion_path(exclusion));
        for target in plan.source().targets() {
            if first_segment(&target.crate_path) == first {
                needed.insert(target.package.as_str());
            }
        }
    }
    let computed = compute(plan.source(), plan.graph(), Some(record), &needed);
    // A capability document that did not load binds nothing in the graph, so
    // every item and test it binds would read as a gap. Its own findings say
    // what is wrong with it, and the gaps are judged once it loads.
    let capability_unloaded = plan
        .unloaded()
        .iter()
        .any(|id| id.starts_with(DocType::Capability.id_prefix()));
    findings(loaded, &scope, &computed, !capability_unloaded)
}

/// The findings of the computed coverage: the gaps of the crates of `scope`
/// when `gaps` holds, and always the surfaces that cannot be listed and the
/// exclusions that match nothing.
fn findings(
    loaded: &Loaded<CoverageRecord>,
    scope: &BTreeSet<&str>,
    computed: &Computed,
    gaps: bool,
) -> Vec<Finding> {
    let report = &computed.report;
    let mut out = Vec::new();
    for error in report
        .errors
        .iter()
        .filter(|e| scope.contains(e.package.as_str()))
    {
        out.push(Finding::new(
            kind::COVERAGE_SURFACE_UNREADABLE,
            loaded.path(),
            Location::Key("scope".to_string()),
            format!("`scope` names `{}`, and {error}", error.package),
            "make every public name of the crate resolve to one item (the error names the file and line), raise the item-bytes budget of the load, or remove the crate from `scope`; a crate whose surface cannot be read is not judged covered",
        ));
    }
    let enforced = format!("the coverage record {} enforces", loaded.path());
    for gap in report
        .code
        .iter()
        .chain(&report.tests)
        .filter(|g| gaps && scope.contains(g.package.as_str()))
    {
        out.push(gap_finding(gap, &enforced));
    }
    for (exclusion, matched) in loaded.value().exclusions.iter().zip(&computed.matched) {
        if !matched {
            out.push(Finding::new(
                kind::COVERAGE_EXCLUSION_UNMATCHED,
                loaded.path(),
                Location::Key("exclusions".to_string()),
                format!(
                    "the exclusion `{}` matches no public item and no test of the workspace",
                    written(exclusion)
                ),
                "correct the path to the item, test or module it means, or remove the exclusion; an exclusion of an item that no longer exists, or that is not public, takes nothing out",
            ));
        }
    }
    out
}

/// The finding of a gap of a package that `enforced` says why it is
/// enforced, as the clause after `which`: `coverage-item-unbound`,
/// `coverage-reexport-unbound` or `coverage-test-unbound` at the file and line
/// of the gap.
pub(crate) fn gap_finding(gap: &Gap, enforced: &str) -> Finding {
    let (kind, fix) = match gap.kind {
        GapKind::PublicItem => (
            kind::COVERAGE_ITEM_UNBOUND,
            "bind the item, or an item it is below, in the `bindings.code` of the capability that explains it, or exclude it in the coverage record with the reason it needs no explanation",
        ),
        GapKind::Reexport => (
            kind::COVERAGE_REEXPORT_UNBOUND,
            "bind a module the re-export is below in the `bindings.code` of the capability that explains it (the index cannot follow the re-export to an item, so the path itself cannot be bound), or exclude it in the coverage record with the reason it needs no explanation",
        ),
        GapKind::Test => (
            kind::COVERAGE_TEST_UNBOUND,
            "bind the test in the `bindings.tests` of the capability whose guarantee it supports, or exclude it in the coverage record with the reason it supports none",
        ),
    };
    let defined = match gap.kind {
        _ if gap.item == gap.path => String::new(),
        GapKind::Reexport => format!(
            " (naming `{}`, which the source index cannot follow to an item)",
            gap.item
        ),
        _ => format!(" (defined as `{}`)", gap.item),
    };
    Finding::new(
        kind,
        &gap.file,
        Location::Line(gap.line),
        format!(
            "the {} `{}`{defined} of `{}`, which {enforced}, is bound by no capability and excluded by no exclusion",
            gap.kind, gap.path, gap.package,
        ),
        fix,
    )
}

/// A computed report and, for each exclusion of the record in its order,
/// whether it matched a public item or a test of the packages computed.
pub(crate) struct Computed {
    pub(crate) report: CoverageReport,
    matched: Vec<bool>,
}

/// What covers: the items `code` bindings resolve to, their paths (below
/// which every item is covered), and the tests `tests` bindings resolve to.
struct Bound {
    code: BTreeSet<ItemId>,
    below: BTreeSet<String>,
    tests: BTreeSet<ItemId>,
}

impl Bound {
    fn of(graph: &Graph, source: &Workspace) -> Self {
        let mut bound = Self {
            code: BTreeSet::new(),
            below: BTreeSet::new(),
            tests: BTreeSet::new(),
        };
        for binding in graph.bindings() {
            for id in binding.items() {
                match binding.kind() {
                    BindingKind::Code => {
                        bound.code.insert(*id);
                        if let Some(item) = source.item(*id) {
                            let root = source
                                .target(item.target)
                                .is_some_and(|target| *item.path == *target.crate_path);
                            if !root {
                                bound.below.insert(item.path.to_string());
                            }
                        }
                    }
                    BindingKind::Test => {
                        bound.tests.insert(*id);
                    }
                    BindingKind::Example => {}
                }
            }
        }
        bound
    }

    /// Whether a `code` binding names `id`, an item of the same source, or an
    /// item that one of them is below.
    fn covers(&self, source: &Workspace, id: ItemId) -> bool {
        let Some(item) = source.item(id) else {
            return false;
        };
        let same = source.same_source(item);
        let ids = if same.is_empty() {
            std::slice::from_ref(&item.id)
        } else {
            same
        };
        ids.iter().any(|id| {
            self.code.contains(id)
                || source
                    .item(*id)
                    .is_some_and(|item| ancestors(&item.path).any(|p| self.below.contains(p)))
        })
    }
}

/// `path` and each path it is below, longest first: `a::b::c`, `a::b`, `a`.
fn ancestors(path: &str) -> impl Iterator<Item = &str> {
    std::iter::successors(Some(path), |p| p.rsplit_once("::").map(|(head, _)| head))
}

fn exclusion_path(exclusion: &Exclusion) -> &str {
    match &exclusion.item {
        ExclusionItem::Item(path) | ExclusionItem::Prefix(path) => path,
    }
}

fn written(exclusion: &Exclusion) -> String {
    match &exclusion.item {
        ExclusionItem::Item(path) => path.clone(),
        ExclusionItem::Prefix(path) => format!("{path}::*"),
    }
}

fn first_segment(path: &str) -> &str {
    path.split_once("::").map_or(path, |(first, _)| first)
}

/// The exclusions of the record by the path each names, and whether each has
/// matched an entry yet.
///
/// An item exclusion is held under its path and matches a path equal to it; a
/// prefix exclusion is held under its module path and matches a path whose
/// proper ancestors, as [`ancestors`] gives them, hold that module path. So an
/// entry costs one lookup for each of its paths and each module above it, and
/// nothing for each exclusion. The positions held are those of the exclusions
/// in the record, which are also those of `matched`.
struct ExclusionIndex<'r> {
    exclusions: &'r [Exclusion],
    items: HashMap<&'r str, Vec<usize>>,
    prefixes: HashMap<&'r str, Vec<usize>>,
    matched: Vec<bool>,
}

impl<'r> ExclusionIndex<'r> {
    /// The index of `exclusions`, none of them matched.
    fn new(exclusions: &'r [Exclusion]) -> Self {
        let mut items: HashMap<&str, Vec<usize>> = HashMap::new();
        let mut prefixes: HashMap<&str, Vec<usize>> = HashMap::new();
        for (position, exclusion) in exclusions.iter().enumerate() {
            add(Work::ExclusionsIndexed, 1);
            let (by_path, path) = match &exclusion.item {
                ExclusionItem::Item(path) => (&mut items, path),
                ExclusionItem::Prefix(path) => (&mut prefixes, path),
            };
            by_path.entry(path.as_str()).or_default().push(position);
        }
        Self {
            exclusions,
            items,
            prefixes,
            matched: vec![false; exclusions.len()],
        }
    }

    /// The first exclusion in the order of the record that matches an entry
    /// with these paths, marking every exclusion that does. With no exclusion
    /// nothing is looked up.
    fn exclusion_for(&mut self, paths: &[&str]) -> Option<&'r Exclusion> {
        if self.exclusions.is_empty() {
            return None;
        }
        let mut first = None;
        for path in paths {
            add(Work::IndexLookups, 1);
            mark(self.items.get(path), &mut self.matched, &mut first);
            for module in ancestors(path).skip(1) {
                add(Work::IndexLookups, 1);
                mark(self.prefixes.get(module), &mut self.matched, &mut first);
            }
        }
        first.and_then(|position| self.exclusions.get(position))
    }
}

/// Mark the exclusions at `found` as matched, and keep in `first` the
/// smallest position seen.
fn mark(found: Option<&Vec<usize>>, matched: &mut [bool], first: &mut Option<usize>) {
    for &position in found.into_iter().flatten() {
        add(Work::ExclusionMarks, 1);
        matched[position] = true;
        *first = Some(first.map_or(position, |seen| seen.min(position)));
    }
}

/// The text of a listing error, with the symbols an ambiguous name names, so
/// the message says where to look.
fn surface_message(source: &Workspace, error: &ExportError) -> String {
    let mut text = error.to_string();
    if let ExportError::Resolve(ResolveError::Ambiguous(ids)) = error {
        let mut sites: Vec<String> = ids
            .iter()
            .filter_map(|id| source.item(*id))
            .map(|item| format!("`{}` ({}:{})", item.path, item.file, item.name_line))
            .collect();
        sites.sort();
        text.push_str(": ");
        text.push_str(&sites.join(", "));
    }
    text
}

/// The order of a report: package, path, file and line.
fn order(gap: &Gap) -> (&str, &str, &str, usize) {
    (&gap.package, &gap.path, &gap.file, gap.line)
}

/// What judging one package set accumulates: the report, which exclusions
/// matched, and the counts of each package.
struct Judge<'a> {
    source: &'a Workspace,
    bound: Bound,
    index: ExclusionIndex<'a>,
    counts: BTreeMap<&'a str, PackageCoverage>,
    report: CoverageReport,
}

impl Judge<'_> {
    /// Judge one entry of its package: `gap` is what it is reported as when it
    /// is neither covered nor excluded, and `matching` the paths an exclusion
    /// is matched against.
    fn judge(&mut self, gap: Gap, matching: &[&str], covered: bool) {
        add(Work::EntriesJudged, 1);
        let exclusion = self.index.exclusion_for(matching);
        if let Some(count) = self.counts.get_mut(gap.package.as_str()) {
            let (seen, hit) = if gap.kind.is_code() {
                (&mut count.items, &mut count.covered_items)
            } else {
                (&mut count.tests, &mut count.covered_tests)
            };
            *seen += 1;
            *hit += usize::from(covered);
        }
        if covered {
            return;
        }
        match exclusion {
            Some(exclusion) => self.report.excluded.push(Excluded {
                gap,
                exclusion: written(exclusion),
                reason: exclusion.reason.clone(),
            }),
            None if gap.kind.is_code() => self.report.code.push(gap),
            None => self.report.tests.push(gap),
        }
    }

    /// Judge the item `item` of `package` exported under `paths` (the test
    /// path for a test).
    fn judge_item(
        &mut self,
        kind: GapKind,
        package: &str,
        item: ItemId,
        paths: &[&str],
        covered: bool,
    ) {
        let Some(defining) = self.source.item(item) else {
            return;
        };
        let gap = Gap {
            kind,
            package: package.to_string(),
            path: paths.first().copied().unwrap_or(&defining.path).to_string(),
            item: defining.path.to_string(),
            file: defining.file.to_string(),
            line: defining.name_line,
        };
        let mut matching: Vec<&str> = paths.to_vec();
        matching.push(&defining.path);
        self.judge(gap, &matching, covered);
    }

    /// Judge the public surface of one library or binary target, or report
    /// why it cannot be listed.
    fn surface(&mut self, id: TargetId, target: &Target) {
        match self.source.exported_items(id) {
            Ok(exports) => {
                let mut units: BTreeMap<ItemId, Vec<&str>> = BTreeMap::new();
                for export in &exports.items {
                    units.entry(export.item).or_default().push(&export.path);
                }
                for (item, paths) in units {
                    let covered = self.bound.covers(self.source, item);
                    self.judge_item(GapKind::PublicItem, &target.package, item, &paths, covered);
                }
                for reexport in &exports.reexports {
                    if reexport.item.is_some() || !self.is_local(id, target, reexport) {
                        continue;
                    }
                    let named = self.named_paths(id, target, reexport);
                    let mut matching = vec![reexport.path.as_str()];
                    matching.extend(named.iter().map(String::as_str));
                    let covered = matching
                        .iter()
                        .any(|path| ancestors(path).any(|p| self.bound.below.contains(p)));
                    let gap = Gap {
                        kind: GapKind::Reexport,
                        package: target.package.clone(),
                        path: reexport.path.clone(),
                        item: named
                            .first()
                            .cloned()
                            .unwrap_or_else(|| reexport.written.clone()),
                        file: reexport.file.to_string(),
                        line: reexport.line,
                    };
                    self.judge(gap, &matching, covered);
                }
            }
            Err(error) => self.report.errors.push(SurfaceError {
                package: target.package.clone(),
                target: target.crate_path.clone(),
                root: target.root.to_string(),
                failure: SurfaceFailure::of(&error),
                message: surface_message(self.source, &error),
            }),
        }
    }
}

impl Judge<'_> {
    /// The paths the written path of a re-export names from the module that
    /// declares it, one for each module path that module is reached under in
    /// the target, read as `rustc` reads a `use` path: `crate::` from the crate
    /// path, a path that starts with a crate of the workspace as written,
    /// `self::` and any other path from the declaring module, and each
    /// `super::` one module up from there. A path whose `super::` climbs
    /// above the crate root names nothing of the target and gives no path.
    fn named_paths(&self, id: TargetId, target: &Target, reexport: &Reexport) -> Vec<String> {
        let written = reexport.written.as_str();
        if let Some(rest) = written.strip_prefix("crate::") {
            return vec![format!("{}::{rest}", target.crate_path)];
        }
        let first = first_segment(written);
        if self
            .source
            .targets()
            .any(|other| first_segment(&other.crate_path) == first)
        {
            return vec![written.to_string()];
        }
        let mut rest = written.strip_prefix("self::").unwrap_or(written);
        let mut up = 0;
        while let Some(below) = rest.strip_prefix("super::") {
            up += 1;
            rest = below;
        }
        let mut paths: Vec<String> = self
            .declaring_modules(id, reexport)
            .into_iter()
            .filter_map(|module| {
                let mut base = module.as_str();
                for _ in 0..up {
                    base = base.rsplit_once("::")?.0;
                }
                Some(format!("{base}::{rest}"))
            })
            .collect();
        paths.sort();
        paths.dedup();
        paths
    }

    /// The paths of the module that holds the declaration of a re-export, in
    /// the target, one for each path its file is reached under: the innermost
    /// inline module of the file whose braces hold the declaration's line, or
    /// the module of the file itself.
    fn declaring_modules(&self, id: TargetId, reexport: &Reexport) -> Vec<String> {
        let Some(file) = self.source.file(&reexport.file) else {
            return Vec::new();
        };
        file.reached
            .iter()
            .filter(|reach| reach.target == id)
            .map(|reach| {
                let below = format!("{}::", reach.module_path);
                self.source
                    .items_in(&reexport.file)
                    .filter(|item| {
                        item.kind == SymbolKind::Mod
                            && item.target == id
                            && item.path.starts_with(&below)
                            && item
                                .body
                                .as_ref()
                                .is_some_and(|body| body.lines.contains(&reexport.line))
                    })
                    .min_by_key(|item| {
                        item.body
                            .as_ref()
                            .map_or(usize::MAX, |body| body.end_byte - body.start_byte)
                    })
                    .map_or_else(
                        || reach.module_path.to_string(),
                        |item| item.path.to_string(),
                    )
            })
            .collect()
    }

    /// Whether a re-export the index cannot follow names something of the
    /// workspace: its written path starts with `crate`, `self` or `super`,
    /// with a name the module that declares it ([`Self::declaring_modules`])
    /// can resolve, or with the crate name of a target of the workspace. Anything else names a crate outside
    /// the workspace, whose items are not counted.
    fn is_local(&self, id: TargetId, target: &Target, reexport: &Reexport) -> bool {
        if reexport.written.starts_with("::") {
            return false;
        }
        let first = first_segment(&reexport.written);
        if matches!(first, "crate" | "self" | "super")
            || self
                .source
                .targets()
                .any(|other| first_segment(&other.crate_path) == first)
        {
            return true;
        }
        self.declaring_modules(id, reexport).iter().any(|module| {
            !matches!(
                self.source
                    .resolve_in_package(&target.package, &format!("{module}::{first}")),
                Err(ResolveError::NotFound(_))
            )
        })
    }
}

/// The report over the packages named in `packages`.
pub(crate) fn compute(
    source: &Workspace,
    graph: &Graph,
    record: Option<&CoverageRecord>,
    packages: &BTreeSet<&str>,
) -> Computed {
    let exclusions: &[Exclusion] = record.map_or(&[], |record| &record.exclusions);
    let scope: BTreeSet<&str> = record.map_or_else(BTreeSet::new, |record| {
        record.scope.iter().map(String::as_str).collect()
    });
    let counts = packages
        .iter()
        .map(|name| {
            (
                *name,
                PackageCoverage {
                    name: (*name).to_string(),
                    in_scope: scope.contains(name),
                    items: 0,
                    covered_items: 0,
                    tests: 0,
                    covered_tests: 0,
                },
            )
        })
        .collect();
    let mut judge = Judge {
        source,
        bound: Bound::of(graph, source),
        index: ExclusionIndex::new(exclusions),
        counts,
        report: CoverageReport::default(),
    };
    for id in source.target_ids() {
        let Some(target) = source.target(id) else {
            continue;
        };
        if matches!(target.kind, TargetKind::Lib | TargetKind::Bin)
            && packages.contains(target.package.as_str())
        {
            judge.surface(id, target);
        }
    }
    for item in source.items().filter(|item| item.test == TestRole::TestFn) {
        let Some(target) = source.target(item.target) else {
            continue;
        };
        if packages.contains(target.package.as_str()) {
            let covered = judge.bound.tests.contains(&item.id);
            judge.judge_item(
                GapKind::Test,
                &target.package,
                item.id,
                &[&item.path],
                covered,
            );
        }
    }
    let Judge {
        mut report,
        index,
        counts,
        ..
    } = judge;
    let matched = index.matched;
    report.code.sort_by(|a, b| order(a).cmp(&order(b)));
    report.tests.sort_by(|a, b| order(a).cmp(&order(b)));
    report
        .excluded
        .sort_by(|a, b| (a.gap.kind, order(&a.gap)).cmp(&(b.gap.kind, order(&b.gap))));
    report
        .errors
        .sort_by(|a, b| (&a.package, &a.target).cmp(&(&b.package, &b.target)));
    report.packages = counts.into_values().collect();
    Computed { report, matched }
}
