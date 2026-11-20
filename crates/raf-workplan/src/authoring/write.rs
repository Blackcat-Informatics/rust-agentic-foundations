// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native transfer of bounded per-document carriers into the repository graph.

use std::{collections::BTreeSet, sync::Arc};

use purrdf_core::{RdfDataset, RdfDatasetBuilder};
use purrdf_lex::allocation::{Ceiling, Memory};
use raf_okf::{DatasetError, Document, Index};
use raf_source::RelPath;

use super::{
    Counts, Declarations, Error, FRAME, INDEX_EDGE, INDEX_IRI, INDEX_PATH, MAX_DOCUMENTS,
    MAX_NATIVE_BYTES, Object, STRING, canonical_path, claims, declarations, root_statements,
};
use crate::evidence::{CLAIMED_STATEMENT, ClaimRelation, QuotedClaim};
use crate::{Loaded, Workplan};

struct Writer<'a> {
    dataset: RdfDatasetBuilder,
    memory: Memory<'a, Ceiling>,
    counts: Counts,
    paths: BTreeSet<RelPath>,
    roots: BTreeSet<String>,
}

pub(super) fn repository(
    plan: &Workplan<'_>,
    index: Option<&Index>,
) -> Result<Arc<RdfDataset>, Error> {
    let mut capacity = Ceiling(MAX_NATIVE_BYTES);
    let mut writer = Writer {
        dataset: RdfDatasetBuilder::new(),
        memory: Memory::new(&mut capacity),
        counts: Counts::default(),
        paths: BTreeSet::new(),
        roots: BTreeSet::new(),
    };
    root_statements(|s, p, o| writer.statement(ClaimRelation::Assertion, s, p, o))?;
    if let Some(index) = index {
        let carrier = index.to_dataset(INDEX_IRI).map_err(Error::index)?;
        writer.carrier(INDEX_PATH, &carrier)?;
        writer.statement(
            ClaimRelation::Assertion,
            FRAME,
            INDEX_EDGE,
            Object::Iri(INDEX_IRI),
        )?;
    }
    for capability in plan.capabilities() {
        writer.document(capability, Declarations::Capability(capability.value()))?;
    }
    writer.documents(plan.changes())?;
    writer.documents(plan.architecture())?;
    for evidence in plan.evidence() {
        writer.document(evidence, Declarations::Evidence(evidence.value()))?;
    }
    for assessment in plan.assessments() {
        writer.document(assessment, Declarations::Inputs(&assessment.value().inputs))?;
    }
    for review in plan.reviews() {
        writer.document(review, Declarations::Inputs(&review.value().inputs))?;
    }
    writer.documents(plan.coverage())?;
    writer.documents(plan.debt())?;
    writer
        .dataset
        .freeze_with_memory(&mut writer.memory)
        .map(Arc::new)
        .map_err(|source| Error::native(FRAME, source))
}

impl Writer<'_> {
    fn documents<'a, T: 'a>(
        &mut self,
        documents: impl IntoIterator<Item = &'a Loaded<T>>,
    ) -> Result<(), Error> {
        for document in documents {
            self.document(document, Declarations::None)?;
        }
        Ok(())
    }

    fn document<T>(&mut self, loaded: &Loaded<T>, declared: Declarations<'_>) -> Result<(), Error> {
        let path = loaded.path();
        if self.paths.len() == MAX_DOCUMENTS {
            return Err(Error::limit(path, "4096 documents"));
        }
        if !self.paths.insert(canonical_path(path)?) {
            return Err(Error::shape(path, "duplicate document path"));
        }
        let document = Document::parse(&loaded.text())
            .map_err(|source| Error::document(path, DatasetError::Parse(source)))?;
        let root = document
            .dataset_iri()
            .map_err(|source| Error::document(path, source))?;
        if self.roots.contains(&root) {
            return Err(Error::shape(&root, "duplicate document identity"));
        }
        let dataset = document
            .to_dataset(path)
            .map_err(|source| Error::document(path, source))?;
        self.carrier(path, &dataset)?;
        declarations((path, &root), declared, |s, p, o| {
            self.statement(ClaimRelation::Assertion, s, p, o)
        })?;
        if matches!(declared, Declarations::Capability(_)) {
            claims((path, &root), document.body(), |row| {
                row.apply(self, Self::statement, Self::reifier)
            })?;
        }
        self.roots.insert(root);
        Ok(())
    }

    fn carrier(&mut self, path: &str, dataset: &RdfDataset) -> Result<(), Error> {
        for row in dataset.quads() {
            self.counts.native(dataset, row)?;
        }
        self.dataset
            .push_dataset_with_memory(dataset, &mut self.memory)
            .map_err(|source| Error::native(path, source))
    }

    fn reifier(&mut self, subject: &str, quote: QuotedClaim<'_>) -> Result<(), Error> {
        self.counts.reifier(subject, quote)?;
        let dataset = &mut self.dataset;
        let memory = &mut self.memory;
        let storage = |source| Error::native(subject, source);
        let reifier = dataset
            .intern_iri_with_memory(subject, memory)
            .map_err(storage)?;
        let s = dataset
            .intern_iri_with_memory(quote.capability, memory)
            .map_err(storage)?;
        let p = dataset
            .intern_iri_with_memory(CLAIMED_STATEMENT, memory)
            .map_err(storage)?;
        let o = dataset
            .intern_literal_parts_with_memory(quote.statement, Some(STRING), None, None, memory)
            .map_err(storage)?;
        let triple = dataset
            .intern_triple_with_memory(s, p, o, memory)
            .map_err(storage)?;
        dataset
            .push_reifier_in_graph_with_memory(reifier, triple, None, memory)
            .map_err(storage)
    }

    fn statement(
        &mut self,
        relation: ClaimRelation,
        subject: &str,
        predicate: &str,
        object: Object<'_>,
    ) -> Result<(), Error> {
        let object_text = match object {
            Object::Iri(text) | Object::Literal(text, _) => text,
        };
        self.counts
            .row(subject, [subject, predicate, object_text])?;
        let dataset = &mut self.dataset;
        let memory = &mut self.memory;
        let storage = |source| Error::native(subject, source);
        let s = dataset
            .intern_iri_with_memory(subject, memory)
            .map_err(storage)?;
        let p = dataset
            .intern_iri_with_memory(predicate, memory)
            .map_err(storage)?;
        let o = match object {
            Object::Iri(iri) => dataset.intern_iri_with_memory(iri, memory),
            Object::Literal(text, datatype) => {
                dataset.intern_literal_parts_with_memory(text, Some(datatype), None, None, memory)
            }
        }
        .map_err(storage)?;
        match relation {
            ClaimRelation::Assertion => dataset.push_quad_with_memory(s, p, o, None, memory),
            ClaimRelation::OccurrenceProperty => {
                dataset.push_annotation_in_graph_with_memory(s, p, o, None, memory)
            }
        }
        .map_err(storage)
    }
}
