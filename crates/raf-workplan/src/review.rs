// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Code review currency over persisted source inputs, independent of Git reachability.

mod dataset;
mod record;
pub use record::{RecordedReview, record_review, record_review_in};

use purrdf_lex::walk::{try_dedup_by, try_equal_range_by, try_sort_unstable_by};
use std::{cmp::Ordering, error::Error, fmt};

use raf_okf::{DocType, Input, InputKind, ReviewKind, ReviewRecord, ReviewScope, ReviewVerdict};
use raf_source::{SymbolKind, Workspace};

use crate::{Loaded, Workplan, digest::Digest, graph::BindingKind};
use crate::{
    exec::{Commit, Runner},
    git,
    lifecycle::BaseTree,
};

/// Resolve a scope from the actual first-party source at a validated commit.
///
/// The shared lazy Git tree reads only files the native source index and
/// document loader request. It preserves source bounds, link handling and
/// original process failures; unrelated binary assets are never decoded.
///
/// # Errors
/// Refuses invalid or absent commits, incomplete historical source or documents,
/// unresolved scopes and original native or Git failures. Later currency queries
/// use the persisted inputs and never call this function again.
pub fn inputs_at_revision(
    runner: &dyn Runner,
    revision: &Commit,
    scope: &ReviewScope,
) -> Result<Vec<Input>, ReviewError> {
    at_revision(runner, revision, scope, |plan, _| plan.review_inputs(scope))
}

fn at_revision<T>(
    runner: &dyn Runner,
    revision: &Commit,
    scope: &ReviewScope,
    inspect: impl FnOnce(&Workplan<'_>, &BaseTree<'_>) -> Result<T, ReviewError>,
) -> Result<T, ReviewError> {
    git::require_version(runner).map_err(ReviewError::native)?;
    let resolved = git::resolve_commit_id(runner, revision).map_err(ReviewError::native)?;
    let prefix = git::show_prefix(runner).map_err(ReviewError::native)?;
    let listing = git::tree_at(runner, &resolved, &prefix).map_err(ReviewError::native)?;
    let tree = BaseTree::at_revision(runner, &resolved, &prefix, listing);
    let source = match Workspace::load(&tree, &["."]) {
        Ok(source) => source,
        Err(error) => {
            return Err(tree
                .take_failure()
                .map_or_else(|| ReviewError::native(error), ReviewError::native));
        }
    };
    let plan = if matches!(scope, ReviewScope::Capability(_)) {
        Workplan::load(&tree, &source)
    } else {
        Workplan::empty(&source)
    };
    let result = inspect(&plan, &tree);
    if let Some(error) = tree.take_failure() {
        return Err(ReviewError::native(error));
    }
    result
}

/// Why a review query cannot give a complete answer.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReviewError {
    /// A named review is absent.
    UnknownReview(String),
    /// A scope names no source set or has an unresolved code binding.
    Scope(String),
    /// A capability binding could not be resolved through the source index.
    Resolve {
        /// The authored binding path.
        path: String,
        /// The original source lookup failure.
        error: raf_source::ResolveError,
    },
    /// A persisted review did not pass the closed profile.
    InvalidRecord(String),
    /// An original digest, identity or native value failure.
    Native(Box<dyn Error + Send + Sync>),
}

impl fmt::Display for ReviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownReview(id) => write!(f, "review `{id}` does not exist"),
            Self::Scope(scope) => write!(f, "review scope cannot be resolved: {scope}"),
            Self::Resolve { path, error } => write!(f, "review binding `{path}`: {error}"),
            Self::InvalidRecord(path) => write!(
                f,
                "review record `{path}` is invalid; repair its document findings"
            ),
            Self::Native(error) => error.fmt(f),
        }
    }
}
impl Error for ReviewError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Native(error) => Some(error.as_ref()),
            Self::Resolve { error, .. } => Some(error),
            _ => None,
        }
    }
}
impl ReviewError {
    fn native(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Native(Box::new(error))
    }
}

/// How an item differs from the latest review of its named scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewChangeKind {
    /// A current item has a different owned-source digest.
    Changed,
    /// A current item was absent from the review.
    Added,
    /// A reviewed item no longer belongs to the current scope.
    Removed,
}

/// One deterministic source difference. Renames are a removal and an addition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewChange {
    /// Canonical item path, independent of load-local identifiers and positions.
    pub item: String,
    /// The difference from the persisted review inputs.
    pub kind: ReviewChangeKind,
}

/// Currency of the latest completed review of one scope and kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewStatus {
    /// The review id; absent when this scope has never been reviewed.
    pub review: Option<String>,
    /// The validated provenance commit, which may no longer be reachable.
    pub revision: Option<String>,
    /// Source differences, ordered by canonical item path.
    pub changes: Vec<ReviewChange>,
}

/// Review coverage of one crate, using canonical indexed item paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewCoverage {
    /// The Cargo package name.
    pub package: String,
    /// The kind of review counted.
    pub kind: ReviewKind,
    /// Number of current canonical item paths.
    pub total: usize,
    /// Paths with at least one unchanged, completed review input.
    pub reviewed: usize,
    /// Percentage in hundredths, truncated deterministically.
    pub percent_hundredths: u16,
}

impl ReviewStatus {
    /// A completed review covers the complete current source set unchanged.
    #[must_use]
    pub fn is_current(&self) -> bool {
        self.review.is_some() && self.changes.is_empty()
    }
}

impl Workplan<'_> {
    /// Judge only the review kinds a capability explicitly requires.
    pub(crate) fn required_reviews(
        &self,
        capability: &Loaded<raf_okf::Capability>,
    ) -> Result<Vec<crate::obligation::Obligation>, ReviewError> {
        use crate::obligation::{Needed, Obligation, ObligationKind, Part, Place, Subject};
        let mut owed = Vec::new();
        for kind in &capability.value().review_required {
            let status = self.review_status(
                &ReviewScope::Capability(capability.value().header.id.clone()),
                *kind,
            )?;
            if status.is_current() {
                continue;
            }
            owed.push(Obligation {
                kind: if status.review.is_none() {
                    ObligationKind::ReviewMissing
                } else {
                    ObligationKind::ReviewStale
                },
                subject: Subject::Capability(capability.value().header.id.clone()),
                section: Part::Document,
                location: Place {
                    path: capability.path().to_string(),
                    line: None,
                },
                evidence: Needed::CurrentReview {
                    kind: *kind,
                    status,
                },
            });
        }
        Ok(owed)
    }

    /// Return every validated review in stable identifier order.
    ///
    /// # Errors
    /// Refuses an invalid review record set rather than returning a partial list.
    pub fn review_list(&self) -> Result<Vec<&Loaded<ReviewRecord>>, ReviewError> {
        self.require_review_records()?;
        let mut records: Vec<_> = self.reviews().iter().collect();
        try_sort_unstable_by(&mut records, |left, right| {
            Ok::<_, ReviewError>(left.value().header.id.cmp(&right.value().header.id))
        })?;
        Ok(records)
    }

    /// Find one validated review without querying its historical commit.
    ///
    /// # Errors
    /// Refuses an invalid record set, invalid id or absent review.
    pub fn review_show(&self, id: &str) -> Result<&Loaded<ReviewRecord>, ReviewError> {
        self.require_review_records()?;
        let id = if id.starts_with("review:") {
            id.to_string()
        } else {
            format!("review:{id}")
        };
        DocType::Review
            .document_iri(&id)
            .map_err(ReviewError::native)?;
        self.reviews()
            .iter()
            .find(|record| record.value().header.id == id)
            .ok_or(ReviewError::UnknownReview(id))
    }

    /// Compute the actual source inputs of a named review scope.
    ///
    /// Module and crate scopes include their indexed descendants. Explicit item
    /// scopes include only the named definitions; no call graph is inferred.
    /// Conditional sites of a canonical item path contribute to the same input.
    ///
    /// ```rust
    /// use raf_okf::ReviewScope;
    /// use raf_source::{MemoryTree, Workspace};
    /// use raf_workplan::Workplan;
    ///
    /// let tree = MemoryTree::from([
    ///     ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///     ("src/lib.rs", "pub fn selected() {}\npub fn other() {}\n"),
    /// ]);
    /// let source = Workspace::load(&tree, &["."])?;
    /// let plan = Workplan::load(&tree, &source);
    /// let inputs = plan.review_inputs(&ReviewScope::Items(vec!["demo::selected".into()]))?;
    /// assert_eq!(inputs.len(), 1);
    /// assert_eq!(inputs[0].name, "demo::selected");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    /// Refuses absent scopes, unresolved capability bindings and missing source.
    pub fn review_inputs(&self, scope: &ReviewScope) -> Result<Vec<Input>, ReviewError> {
        self.review_inputs_for(scope, false)
    }

    fn review_inputs_for(
        &self,
        scope: &ReviewScope,
        reviewed: bool,
    ) -> Result<Vec<Input>, ReviewError> {
        let source = self.source();
        let scope = canonical_scope(scope)?;
        let mut paths = Vec::new();
        match &scope {
            ReviewScope::Crate(package) => {
                if !source.packages().iter().any(|found| found.name == *package) {
                    if reviewed {
                        return Ok(Vec::new());
                    }
                    return Err(ReviewError::Scope(format!("unknown crate `{package}`")));
                }
                paths.extend(
                    source
                        .items()
                        .filter(|item| {
                            source
                                .target(item.target)
                                .is_some_and(|target| target.package == *package)
                        })
                        .map(|item| item.path.to_string()),
                );
            }
            ReviewScope::Module(path) => {
                if !source
                    .items_under(path)
                    .any(|item| item.path.as_str() == path && item.kind == SymbolKind::Mod)
                {
                    if reviewed {
                        return Ok(Vec::new());
                    }
                    return Err(ReviewError::Scope(format!("unknown module `{path}`")));
                }
                paths.extend(source.items_under(path).map(|item| item.path.to_string()));
            }
            ReviewScope::Items(items) => {
                for path in items {
                    if !source
                        .items_under(path)
                        .any(|item| item.path.as_str() == path)
                    {
                        if reviewed {
                            continue;
                        }
                        return Err(ReviewError::Scope(format!("unknown item `{path}`")));
                    }
                    paths.push(path.clone());
                }
            }
            ReviewScope::Capability(id) => {
                if !self
                    .capabilities()
                    .iter()
                    .any(|cap| cap.value().header.id == *id)
                {
                    if reviewed && !self.is_present(id) {
                        return Ok(Vec::new());
                    }
                    return Err(ReviewError::Scope(format!("unknown capability `{id}`")));
                }
                for binding in self
                    .graph()
                    .bindings_of(id)
                    .filter(|binding| binding.kind() == BindingKind::Code)
                {
                    let definitions =
                        binding.resolution().map_err(|error| ReviewError::Resolve {
                            path: binding.path().to_string(),
                            error: (*error).clone(),
                        })?;
                    for definition in definitions {
                        let item = source
                            .item(*definition)
                            .ok_or_else(|| ReviewError::Scope(binding.path().to_string()))?;
                        if item.kind == SymbolKind::Mod {
                            paths.extend(
                                source
                                    .items_under(&item.path)
                                    .map(|item| item.path.to_string()),
                            );
                        } else {
                            paths.push(item.path.to_string());
                        }
                    }
                }
            }
        }
        try_sort_unstable_by(&mut paths, |left, right| {
            Ok::<_, ReviewError>(left.cmp(right))
        })?;
        try_dedup_by(&mut paths, |left, right| {
            Ok::<_, ReviewError>(left == right)
        })?;
        if paths.is_empty() && !reviewed {
            return Err(ReviewError::Scope(
                "the selected source set is empty".to_string(),
            ));
        }
        review_path_inputs(source, paths)
    }

    /// Compare the latest completed review with current source, without Git.
    ///
    /// Dates compare as native timezone-aware values; equal instants use record
    /// id order. A blocked review never replaces a completed review.
    ///
    /// # Errors
    /// Refuses malformed records, unresolved scopes and native value failures.
    pub fn review_status(
        &self,
        scope: &ReviewScope,
        kind: ReviewKind,
    ) -> Result<ReviewStatus, ReviewError> {
        self.require_review_records()?;
        let scope = canonical_scope(scope)?;
        let mut latest: Option<&ReviewRecord> = None;
        for record in self.reviews().iter().map(Loaded::value).filter(|record| {
            record.scope == scope && record.kind == kind && record.verdict != ReviewVerdict::Blocked
        }) {
            if let Some(previous) = latest
                && review_order(record, previous)? != Ordering::Greater
            {
                continue;
            }
            latest = Some(record);
        }
        let current = self.review_inputs_for(&scope, latest.is_some())?;
        let mut previous: Vec<_> = latest
            .into_iter()
            .flat_map(|record| record.inputs.iter())
            .collect();
        try_sort_unstable_by(&mut previous, |left, right| {
            Ok::<_, ReviewError>(left.name.cmp(&right.name))
        })?;
        let mut changes = Vec::new();
        for input in &current {
            let range = try_equal_range_by(&previous, |old| {
                Ok::<_, ReviewError>(old.name.cmp(&input.name))
            })?;
            let old = previous[range].first().copied();
            let kind = match old {
                None => Some(ReviewChangeKind::Added),
                Some(old) if old.digest != input.digest => Some(ReviewChangeKind::Changed),
                Some(_) => None,
            };
            if let Some(kind) = kind {
                changes.push(ReviewChange {
                    item: input.name.clone(),
                    kind,
                });
            }
        }
        if let Some(record) = latest {
            for input in &record.inputs {
                if try_equal_range_by(&current, |now| {
                    Ok::<_, ReviewError>(now.name.cmp(&input.name))
                })?
                .is_empty()
                {
                    changes.push(ReviewChange {
                        item: input.name.clone(),
                        kind: ReviewChangeKind::Removed,
                    });
                }
            }
        }
        try_sort_unstable_by(&mut changes, |left, right| {
            Ok::<_, ReviewError>(left.item.cmp(&right.item))
        })?;
        Ok(ReviewStatus {
            review: latest.map(|record| record.header.id.clone()),
            revision: latest.map(|record| record.revision.clone()),
            changes,
        })
    }

    /// Canonical item paths with no unchanged input of a completed review.
    ///
    /// A stale review can still cover its unchanged items. Blocked reviews
    /// cover none. An item added after a review is always returned.
    ///
    /// # Errors
    /// Refuses an invalid record set, unknown crate or unreadable source input.
    pub fn review_unreviewed(
        &self,
        kind: ReviewKind,
        package: Option<&str>,
    ) -> Result<Vec<String>, ReviewError> {
        self.require_review_records()?;
        let covered = self.reviewed_inputs(kind)?;
        let packages: Vec<_> = match package {
            Some(package) => vec![package.to_string()],
            None => self
                .source()
                .packages()
                .iter()
                .map(|package| package.name.clone())
                .collect(),
        };
        let mut missing = Vec::new();
        for package in packages {
            for input in self.review_inputs(&ReviewScope::Crate(package))? {
                if try_equal_range_by(&covered, |old| {
                    Ok::<_, ReviewError>(old.cmp(&(input.name.as_str(), input.digest.as_str())))
                })?
                .is_empty()
                {
                    missing.push(input.name);
                }
            }
        }
        try_sort_unstable_by(&mut missing, |left, right| {
            Ok::<_, ReviewError>(left.cmp(right))
        })?;
        try_dedup_by(&mut missing, |left, right| {
            Ok::<_, ReviewError>(left == right)
        })?;
        Ok(missing)
    }

    /// Per-crate coverage of current items for one review kind.
    ///
    /// # Errors
    /// Preserves invalid-record and source-input refusals from the item query.
    pub fn review_coverage(&self, kind: ReviewKind) -> Result<Vec<ReviewCoverage>, ReviewError> {
        self.require_review_records()?;
        let covered = self.reviewed_inputs(kind)?;
        let mut rows = Vec::new();
        for package in self.source().packages() {
            let inputs = self.review_inputs(&ReviewScope::Crate(package.name.clone()))?;
            let total = inputs.len();
            let mut reviewed = 0;
            for input in &inputs {
                if !try_equal_range_by(&covered, |old| {
                    Ok::<_, ReviewError>(old.cmp(&(input.name.as_str(), input.digest.as_str())))
                })?
                .is_empty()
                {
                    reviewed += 1;
                }
            }
            let percent = (reviewed as u128) * 10_000 / (total as u128);
            let percent_hundredths = u16::try_from(percent).map_err(ReviewError::native)?;
            rows.push(ReviewCoverage {
                package: package.name.clone(),
                kind,
                total,
                reviewed,
                percent_hundredths,
            });
        }
        try_sort_unstable_by(&mut rows, |left, right| {
            Ok::<_, ReviewError>(left.package.cmp(&right.package))
        })?;
        Ok(rows)
    }

    fn reviewed_inputs(&self, kind: ReviewKind) -> Result<Vec<(&str, &str)>, ReviewError> {
        let mut rows: Vec<_> = self
            .reviews()
            .iter()
            .map(Loaded::value)
            .filter(|record| record.kind == kind && record.verdict != ReviewVerdict::Blocked)
            .flat_map(|record| {
                record
                    .inputs
                    .iter()
                    .map(|input| (input.name.as_str(), input.digest.as_str()))
            })
            .collect();
        try_sort_unstable_by(&mut rows, |left, right| {
            Ok::<_, ReviewError>(left.cmp(right))
        })?;
        try_dedup_by(&mut rows, |left, right| Ok::<_, ReviewError>(left == right))?;
        Ok(rows)
    }

    fn require_review_records(&self) -> Result<(), ReviewError> {
        if let Some(finding) = self
            .findings
            .iter()
            .find(|finding| finding.file.starts_with("work/reviews/"))
        {
            return Err(ReviewError::InvalidRecord(finding.file.clone()));
        }
        Ok(())
    }
}

fn review_order(left: &ReviewRecord, right: &ReviewRecord) -> Result<Ordering, ReviewError> {
    let date = |record: &ReviewRecord| {
        let text = record
            .header
            .timestamp
            .as_deref()
            .ok_or_else(|| ReviewError::InvalidRecord(record.header.id.clone()))?;
        purrdf_xsd::temporal::parse_datetime_stamp(text).map_err(ReviewError::native)
    };
    let order = purrdf_xsd::temporal::cmp_datetime(&date(left)?, &date(right)?)
        .ok_or_else(|| ReviewError::InvalidRecord(left.header.id.clone()))?;
    Ok(order.then_with(|| left.header.id.cmp(&right.header.id)))
}

pub(crate) fn reference_findings(
    tree: &dyn raf_source::Tree,
    path: &str,
    record: &ReviewRecord,
) -> Vec<crate::Finding> {
    let Some(reference) = &record.findings_ref else {
        return Vec::new();
    };
    match record::validate_reference(tree, reference) {
        Ok(()) => Vec::new(),
        Err(error) => vec![crate::Finding::new(
            crate::finding::kind::REVIEW_FINDINGS_UNREADABLE,
            path,
            raf_okf::Location::Key("findings_ref".to_string()),
            format!("review findings reference `{reference}` cannot be read: {error}"),
            "name a readable repository findings document or a valid opaque external URL",
        )],
    }
}

fn review_path_inputs(source: &Workspace, paths: Vec<String>) -> Result<Vec<Input>, ReviewError> {
    paths
        .into_iter()
        .map(|path| {
            let digest = Digest::review_symbol(source, &path).map_err(ReviewError::native)?;
            Ok(Input::new(InputKind::Symbol, path, digest.to_string()))
        })
        .collect()
}

fn canonical_scope(scope: &ReviewScope) -> Result<ReviewScope, ReviewError> {
    let selectors = match scope {
        ReviewScope::Crate(package) => vec![format!("crate:{package}")],
        ReviewScope::Module(path) => vec![format!("module:{path}")],
        ReviewScope::Capability(id) => vec![id.clone()],
        ReviewScope::Items(paths) => paths.iter().map(|path| format!("item:{path}")).collect(),
    };
    let values: Vec<_> = selectors.iter().map(String::as_str).collect();
    ReviewScope::from_selectors(&values)
        .ok_or_else(|| ReviewError::Scope("invalid or duplicate review selectors".to_string()))
}
