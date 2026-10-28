// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Record a review only after resolving its real historical source inputs.

use super::ReviewError;
use crate::{
    exec::{Commit, Runner},
    git,
    record::{self, DiskRecordStore, Record, RecordOptions, RecordStore},
};
use raf_okf::{DocType, ReviewDisposition, ReviewLocation, ReviewRecord};
use raf_source::{RelPath, Tree};
use std::path::Path;

/// The published review identity and path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedReview {
    /// Stable review document id.
    pub id: String,
    /// The created repository-relative path.
    pub path: RelPath,
}

/// Record a review draft against its actual commit, using the normal record store.
///
/// The draft must have no supplied inputs: the historical native source index
/// computes every persisted digest. The supplied runner must operate in `root`,
/// as for other process-backed Workplan commands. No record data becomes a
/// program or an unchecked Git argument. Existing records are never replaced.
///
/// # Errors
/// Refuses supplied inputs, malformed metadata, nonexistent commits or finding
/// locations, unreadable repository findings references and record-store errors.
/// A refused draft writes nothing.
pub fn record_review(
    root: &Path,
    runner: &dyn Runner,
    draft: &ReviewRecord,
    options: &RecordOptions,
) -> Result<RecordedReview, ReviewError> {
    record_review_in(&mut DiskRecordStore::new(root), runner, draft, options)
}

/// Record through a supplied native reader and publication authority.
///
/// # Errors
/// Preserves every refusal of [`record_review`] and the supplied store.
pub fn record_review_in(
    store: &mut dyn RecordStore,
    runner: &dyn Runner,
    draft: &ReviewRecord,
    options: &RecordOptions,
) -> Result<RecordedReview, ReviewError> {
    if !draft.inputs.is_empty() {
        return Err(ReviewError::Scope(
            "review recording computes its inputs; supply an empty draft input list".to_string(),
        ));
    }
    DocType::Review
        .document_iri(&draft.header.id)
        .map_err(ReviewError::native)?;
    let date = draft.header.timestamp.as_deref().ok_or_else(|| {
        ReviewError::InvalidRecord("a review requires its explicit review date".to_string())
    })?;
    raf_okf::validate_timestamp(date).map_err(ReviewError::native)?;
    let revision = Commit::new(&draft.revision).map_err(ReviewError::native)?;
    if let Some(reference) = &draft.findings_ref {
        validate_reference(store.tree().map_err(ReviewError::native)?, reference)?;
    }
    for finding in &draft.findings {
        if let ReviewDisposition::Fixed(revision) = &finding.disposition {
            let commit = Commit::new(revision).map_err(ReviewError::native)?;
            git::resolve_commit_id(runner, &commit).map_err(ReviewError::native)?;
        }
    }
    let inputs = super::at_revision(runner, &revision, &draft.scope, |plan, tree| {
        for finding in &draft.findings {
            match &finding.location {
                ReviewLocation::Item(path) => {
                    if !plan
                        .source()
                        .items_under(path)
                        .any(|item| item.path.as_str() == path)
                    {
                        return Err(ReviewError::Scope(format!(
                            "finding {} names absent item `{path}`",
                            finding.id
                        )));
                    }
                }
                ReviewLocation::Lines { file, first, last } => {
                    let path = RelPath::new(file).map_err(ReviewError::native)?;
                    let text = tree
                        .read(&path, raf_okf::MAX_TEXT_BYTES as u64 + 1)
                        .map_err(ReviewError::native)?;
                    if *first == 0 || last < first || *last > text.lines().count() {
                        return Err(ReviewError::Scope(format!(
                            "finding {} names lines outside `{file}` at the reviewed revision",
                            finding.id
                        )));
                    }
                }
            }
        }
        plan.review_inputs(&draft.scope)
    })?;
    let mut review = draft.clone();
    review.inputs = inputs;
    let path =
        record::create_in(store, &Record::Review(review), options).map_err(ReviewError::native)?;
    Ok(RecordedReview {
        id: draft.header.id.clone(),
        path,
    })
}

pub(super) fn validate_reference(tree: &dyn Tree, reference: &str) -> Result<(), ReviewError> {
    if reference.starts_with("https://") || reference.starts_with("http://") {
        // An external findings URL is explicitly opaque: validate native IRI
        // syntax without fetching it or claiming that the remote target exists.
        let mut builder = purrdf_core::RdfDatasetBuilder::new();
        builder.intern_iri(reference).map_err(ReviewError::native)?;
        return Ok(());
    }
    let path = RelPath::new(reference).map_err(ReviewError::native)?;
    tree.read(&path, raf_okf::MAX_TEXT_BYTES as u64 + 1)
        .map_err(ReviewError::native)?;
    Ok(())
}
