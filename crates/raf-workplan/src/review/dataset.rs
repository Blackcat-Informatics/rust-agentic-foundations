// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Review claims in the shared semantic Workplan vocabulary.

use super::ReviewError;
use crate::{Workplan, evidence::CLAIMS_VOCAB};
use purrdf_core::{RdfDataset, RdfDatasetBuilder, RdfLiteral, TermId};
use purrdf_lex::walk::{try_dedup_by, try_sort_unstable_by};
use raf_okf::{DocType, ReviewDisposition, ReviewLocation, ReviewScope};
use std::sync::Arc;

/// Only Workplan property naming and the string datatype are local policy.
/// Storage, triple terms, reifiers and RDF validation are native operations.
struct Writer(RdfDatasetBuilder);
impl Writer {
    fn iri(&mut self, text: &str) -> Result<TermId, ReviewError> {
        self.0.intern_iri(text).map_err(ReviewError::native)
    }
    fn property(&mut self, name: &str) -> Result<TermId, ReviewError> {
        self.iri(&format!("{CLAIMS_VOCAB}{name}"))
    }
    fn text(&mut self, subject: TermId, name: &str, text: &str) -> Result<(), ReviewError> {
        self.scalar(
            subject,
            name,
            text,
            "http://www.w3.org/2001/XMLSchema#string",
        )
    }
    fn scalar(
        &mut self,
        subject: TermId,
        name: &str,
        text: &str,
        datatype: &str,
    ) -> Result<(), ReviewError> {
        let value = self
            .0
            .intern_literal(RdfLiteral {
                lexical_form: text.to_string(),
                datatype: Some(datatype.to_string()),
                language: None,
                direction: None,
            })
            .map_err(ReviewError::native)?;
        self.put(subject, name, value)
    }
    fn put(&mut self, subject: TermId, name: &str, object: TermId) -> Result<(), ReviewError> {
        let predicate = self.property(name)?;
        self.0
            .push_quad(subject, predicate, object, None)
            .map_err(ReviewError::native)
    }
    fn class(&mut self, subject: TermId, name: &str) -> Result<(), ReviewError> {
        let predicate = self.iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")?;
        let class = self.property(name)?;
        self.0
            .push_quad(subject, predicate, class, None)
            .map_err(ReviewError::native)
    }
}

impl Workplan<'_> {
    /// The complete review claim as native RDF 1.2 triple terms and metadata.
    ///
    /// The review reifies an unasserted verdict about its source set. Inputs
    /// preserve each canonical symbol path and its digest. Findings belong to
    /// that same stable review document IRI, joining the authoring and evidence
    /// graph without interpreting the reviewer's natural language.
    ///
    /// # Errors
    /// Preserves the closed record lookup, shared identity and native RDF errors.
    pub fn review_claims_dataset(&self, id: &str) -> Result<Arc<RdfDataset>, ReviewError> {
        self.review_claims_selection(&[id])
    }

    /// Export a complete selection of validated review claims, including an
    /// explicit selection root when the requested set is empty.
    ///
    /// # Errors
    /// Refuses the entire selection on an unknown or invalid review, or a native
    /// identity, storage or RDF validation failure.
    pub fn review_claims_selection(&self, ids: &[&str]) -> Result<Arc<RdfDataset>, ReviewError> {
        let mut records = ids
            .iter()
            .map(|id| self.review_show(id))
            .collect::<Result<Vec<_>, _>>()?;
        try_sort_unstable_by(&mut records, |left, right| {
            Ok::<_, ReviewError>(left.value().header.id.cmp(&right.value().header.id))
        })?;
        try_dedup_by(&mut records, |left, right| {
            Ok::<_, ReviewError>(left.value().header.id == right.value().header.id)
        })?;
        let key = records
            .iter()
            .map(|record| record.value().header.id.as_str())
            .collect::<Vec<_>>()
            .join("\0");
        let hash = purrdf_hash::hex::Lower(purrdf_hash::blake3::hash(key.as_bytes()).as_bytes())
            .to_string();
        let mut writer = Writer(RdfDatasetBuilder::new());
        let selection = writer.iri(&format!(
            "https://blackcatinformatics.ca/raf/workplan/reviews/selection/{hash}"
        ))?;
        writer.class(selection, "ReviewSelection")?;
        for record in records {
            let record = record.value();
            let iri = DocType::Review
                .document_iri(&record.header.id)
                .map_err(ReviewError::native)?;
            let review = writer.iri(&iri)?;
            writer.put(selection, "review", review)?;
            let scope = writer.iri(&format!("{iri}/scope"))?;
            writer.class(review, "CodeReview")?;
            writer.class(scope, "ReviewedSourceSet")?;
            let predicate = writer.property("reviewVerdict")?;
            let verdict = writer.property(record.verdict.name())?;
            let claim = writer
                .0
                .intern_triple(scope, predicate, verdict)
                .map_err(ReviewError::native)?;
            writer
                .0
                .push_reifier(review, claim)
                .map_err(ReviewError::native)?;
            writer.put(review, "reviewedScope", scope)?;
            writer.text(review, "reviewKind", record.kind.name())?;
            writer.text(review, "reviewerKind", record.by.name())?;
            writer.text(review, "revision", &record.revision)?;
            {
                let date = record
                    .header
                    .timestamp
                    .as_deref()
                    .ok_or_else(|| ReviewError::InvalidRecord(record.header.id.clone()))?;
                writer.scalar(
                    review,
                    "date",
                    date,
                    "http://www.w3.org/2001/XMLSchema#dateTime",
                )?;
            }
            if let Some(reference) = &record.findings_ref {
                writer.text(review, "findingsReference", reference)?;
            }
            let (kind, values) = match &record.scope {
                ReviewScope::Crate(package) => ("crate", vec![package.as_str()]),
                ReviewScope::Module(path) => ("module-tree", vec![path.as_str()]),
                ReviewScope::Items(paths) => ("items", paths.iter().map(String::as_str).collect()),
                ReviewScope::Capability(id) => ("capability", vec![id.as_str()]),
            };
            writer.text(scope, "scopeKind", kind)?;
            for value in values {
                writer.text(scope, "scopeSelector", value)?;
            }
            for input in &record.inputs {
                let key = purrdf_hash::hex::Lower(
                    purrdf_hash::blake3::hash(input.name.as_bytes()).as_bytes(),
                )
                .to_string();
                let node = writer.iri(&format!("{iri}/input/{key}"))?;
                writer.class(node, "ReviewInput")?;
                writer.put(review, "input", node)?;
                writer.put(scope, "member", node)?;
                writer.text(node, "inputKind", input.kind.name())?;
                writer.text(node, "symbolPath", &input.name)?;
                writer.text(node, "digest", &input.digest)?;
            }
            write_findings(&mut writer, review, &iri, &record.findings)?;
        }
        writer.0.freeze().map_err(ReviewError::native)
    }
}

fn write_findings(
    writer: &mut Writer,
    review: TermId,
    iri: &str,
    findings: &[raf_okf::ReviewFinding],
) -> Result<(), ReviewError> {
    for finding in findings {
        let node = writer.iri(&format!("{iri}/finding/{}", finding.id))?;
        writer.class(node, "ReviewFinding")?;
        writer.put(review, "finding", node)?;
        writer.text(node, "severity", finding.severity.name())?;
        match &finding.location {
            ReviewLocation::Item(path) => writer.text(node, "symbolPath", path)?,
            ReviewLocation::Lines { file, first, last } => {
                writer.text(node, "path", file)?;
                writer.scalar(
                    node,
                    "firstLine",
                    &first.to_string(),
                    "http://www.w3.org/2001/XMLSchema#positiveInteger",
                )?;
                writer.scalar(
                    node,
                    "lastLine",
                    &last.to_string(),
                    "http://www.w3.org/2001/XMLSchema#positiveInteger",
                )?;
            }
        }
        let (status, extra) = match &finding.disposition {
            ReviewDisposition::Open => ("open", None),
            ReviewDisposition::Fixed(revision) => ("fixed", Some(("fixingRevision", revision))),
            ReviewDisposition::Accepted(reason) => ("accepted", Some(("reason", reason))),
            ReviewDisposition::FalsePositive(reason) => {
                ("false-positive", Some(("reason", reason)))
            }
        };
        writer.text(node, "status", status)?;
        if let Some((name, text)) = extra {
            writer.text(node, name, text)?;
        }
    }
    Ok(())
}
