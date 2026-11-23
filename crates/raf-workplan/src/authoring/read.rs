// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Validate the inventory, route each subject once, and check the declarations.

use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

use purrdf_core::{GraphMatch, QuadIds, RdfDataset, TermId, TermRef};
use raf_okf::{
    DATASET_NODE_IRI_BASE, DOCUMENT_IRI_BASE, Document, Index, RafDocument, dataset_node_prefix,
};

use super::{
    Counts, Declarations, Error, FRAME, INDEX_EDGE, INDEX_IRI, INDEX_PATH, MAX_DOCUMENTS, Object,
    RepositoryAuthoring, STRING, canonical_path, claims, declarations, iri, property, record_iri,
    root_statements,
};
use crate::evidence::{CLAIMED_STATEMENT, ClaimRelation, QuotedClaim};

struct Group<'a> {
    path: &'a str,
    subjects: Vec<TermId>,
}

type Groups<'a> = BTreeMap<TermId, Group<'a>>;

pub(super) fn repository(dataset: &RdfDataset) -> Result<RepositoryAuthoring, Error> {
    if dataset.named_graphs().next().is_some() {
        return Err(Error::shape(FRAME, "use the default graph only"));
    }
    let index_subject = index_member(dataset)?;
    let mut verifier = Verifier {
        dataset,
        expected: usize::from(index_subject.is_some()),
        expected_reifiers: 0,
        remaining_annotations: BTreeSet::new(),
    };
    root_statements(|s, p, o| verifier.statement(ClaimRelation::Assertion, s, p, o))?;
    let mut groups = inventory(dataset)?;
    let (metadata_rows, reifier_rows) = route(dataset, &mut groups, index_subject)?;
    verifier.remaining_annotations.extend(dataset.annotations());
    let index = index_subject
        .map(|_| Index::from_dataset_subject(dataset, INDEX_IRI).map_err(Error::index))
        .transpose()?;
    let mut documents = BTreeMap::new();
    for (id, group) in groups {
        let (document, typed) =
            Document::from_dataset_subjects(dataset, group.path, id, &group.subjects)
                .map_err(|source| Error::document(group.path, source))?;
        let declared = match &typed {
            RafDocument::Capability(capability) => Declarations::Capability(capability),
            RafDocument::Evidence(evidence) => Declarations::Evidence(evidence),
            RafDocument::Assessment(assessment) => Declarations::Inputs(&assessment.inputs),
            RafDocument::Review(review) => Declarations::Inputs(&review.inputs),
            _ => Declarations::None,
        };
        let root = iri(dataset, id)?;
        declarations((group.path, root), declared, |s, p, o| {
            verifier.statement(ClaimRelation::Assertion, s, p, o)
        })?;
        if matches!(declared, Declarations::Capability(_)) {
            claims((group.path, root), document.body(), |row| {
                row.apply(&mut verifier, Verifier::statement, Verifier::reifier)
            })?;
        }
        documents.insert(canonical_path(group.path)?, document);
    }
    if metadata_rows != verifier.expected {
        return Err(Error::shape(
            FRAME,
            "unexpected inventory or declaration assertions; use only the recovered document projections",
        ));
    }
    if reifier_rows != verifier.expected_reifiers {
        return Err(Error::shape(
            FRAME,
            "unexpected claim reifiers; use only authored claim occurrences",
        ));
    }
    if !verifier.remaining_annotations.is_empty() {
        return Err(Error::shape(
            FRAME,
            "unexpected claim annotations; use only authored occurrence properties",
        ));
    }
    Ok(RepositoryAuthoring { documents, index })
}

fn rows(
    dataset: &RdfDataset,
    subject: TermId,
    predicate: TermId,
    object: Option<TermId>,
) -> impl Iterator<Item = QuadIds> + '_ {
    dataset.quads_for_pattern_with_plan(
        &RdfDataset::probe_plan(true, true, object.is_some(), GraphMatch::Default),
        Some(subject),
        Some(predicate),
        object,
        GraphMatch::Default,
    )
}

fn index_member(dataset: &RdfDataset) -> Result<Option<TermId>, Error> {
    let (Some(frame), Some(predicate)) = (
        dataset.term_id_by_iri(FRAME),
        dataset.term_id_by_iri(INDEX_EDGE),
    ) else {
        return Ok(None);
    };
    let mut membership = rows(dataset, frame, predicate, None);
    let Some(row) = membership.next() else {
        return Ok(None);
    };
    if dataset.term_id_by_iri(INDEX_IRI) != Some(row.o) || membership.next().is_some() {
        return Err(Error::shape(
            INDEX_PATH,
            "use one repository Index assertion to the fixed Index IRI",
        ));
    }
    Ok(Some(row.o))
}

fn one(dataset: &RdfDataset, subject: TermId, key: &str) -> Result<TermId, Error> {
    if let Some(predicate) = dataset.term_id_by_iri(&property(key)) {
        let mut matches = rows(dataset, subject, predicate, None);
        if let Some(row) = matches.next()
            && matches.next().is_none()
        {
            return Ok(row.o);
        }
    }
    Err(Error::shape(
        iri(dataset, subject)?,
        "an inventory entry must have one path and one document identity",
    ))
}

fn inventory(dataset: &RdfDataset) -> Result<Groups<'_>, Error> {
    let mut groups = BTreeMap::new();
    let (Some(frame), Some(property)) = (
        dataset.term_id_by_iri(FRAME),
        dataset.term_id_by_iri(&property("document")),
    ) else {
        return Ok(groups);
    };
    let mut paths = BTreeSet::new();
    let mut counts = Counts::default();
    for edge in rows(dataset, frame, property, None) {
        if groups.len() == MAX_DOCUMENTS {
            return Err(Error::limit(FRAME, "4096 documents"));
        }
        let record = iri(dataset, edge.o)?;
        let root_id = one(dataset, edge.o, "documentIdentity")?;
        let root = iri(dataset, root_id)?;
        if !root.starts_with(DOCUMENT_IRI_BASE)
            || root.len() > raf_okf::MAX_TEXT_BYTES + DOCUMENT_IRI_BASE.len() + 64
            || record != record_iri(root)?
        {
            return Err(Error::shape(
                record,
                "inventory identity must derive from the asserted document IRI",
            ));
        }
        let path_id = one(dataset, edge.o, "path")?;
        let TermRef::Literal {
            lexical: path,
            datatype,
            language: None,
            direction: None,
        } = dataset.resolve(path_id)
        else {
            return Err(Error::shape(record, "inventory path must be an xsd:string"));
        };
        if iri(dataset, datatype)? != STRING {
            return Err(Error::shape(record, "inventory path must be an xsd:string"));
        }
        // These strings are all counted by the later statement scan. Check
        // this subset before allocating or hashing untrusted identities.
        counts.row(record, [record, root, path])?;
        canonical_path(path)?;
        if !paths.insert(path) {
            return Err(Error::shape(path, "duplicate document path"));
        }
        if groups
            .insert(
                root_id,
                Group {
                    path,
                    subjects: Vec::new(),
                },
            )
            .is_some()
        {
            return Err(Error::shape(root, "duplicate document identity"));
        }
    }
    Ok(groups)
}

fn route(
    dataset: &RdfDataset,
    groups: &mut Groups<'_>,
    index_subject: Option<TermId>,
) -> Result<(usize, usize), Error> {
    let mut prefixes = BTreeMap::new();
    for &root in groups.keys() {
        let root_iri = iri(dataset, root)?;
        if prefixes
            .insert(dataset_node_prefix(root_iri), root)
            .is_some()
        {
            return Err(Error::shape(
                root_iri,
                "structural subjects must have one document owner",
            ));
        }
    }
    let mut seen = BTreeMap::new();
    let mut counts = Counts::default();
    let mut metadata = 0;
    for row in dataset.quads() {
        counts.native(dataset, row)?;
        if index_subject == Some(row.s) {
            continue;
        }
        let is_metadata = match seen.entry(row.s) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                *entry.insert(route_subject(dataset, row.s, groups, &prefixes)?)
            }
        };
        metadata += usize::from(is_metadata);
    }
    let mut reifiers = 0;
    for row in dataset.reifier_quads() {
        counts.native_reifier(dataset, row)?;
        reifiers += 1;
    }
    for row in dataset.annotation_quads() {
        counts.native(dataset, row)?;
    }
    Ok((metadata, reifiers))
}

fn route_subject(
    dataset: &RdfDataset,
    subject: TermId,
    groups: &mut Groups<'_>,
    prefixes: &BTreeMap<String, TermId>,
) -> Result<bool, Error> {
    let name = iri(dataset, subject)?;
    if name == INDEX_IRI {
        return Err(Error::shape(
            INDEX_PATH,
            "the Index carrier requires repository membership",
        ));
    }
    if name == FRAME
        || name
            .strip_prefix(FRAME)
            .is_some_and(|tail| tail.starts_with("/document/"))
    {
        return Ok(true);
    }
    let owner = if name.starts_with(DOCUMENT_IRI_BASE) {
        Some(subject)
    } else if name.starts_with(DATASET_NODE_IRI_BASE) {
        name.rfind('/')
            .and_then(|index| name.get(..=index))
            .and_then(|prefix| prefixes.get(prefix).copied())
    } else {
        None
    };
    let group = owner.and_then(|id| groups.get_mut(&id)).ok_or_else(|| {
        Error::shape(
            name,
            "unowned carrier subject; every document must have one inventory path",
        )
    })?;
    if group.subjects.len() == raf_okf::MAX_DATASET_NODES {
        return Err(Error::limit(
            group.path,
            "131072 structural subjects in one document",
        ));
    }
    group.subjects.push(subject);
    Ok(false)
}

struct Verifier<'a> {
    dataset: &'a RdfDataset,
    expected: usize,
    expected_reifiers: usize,
    remaining_annotations: BTreeSet<(TermId, TermId, TermId)>,
}

impl Verifier<'_> {
    fn reifier(&mut self, subject: &str, quote: QuotedClaim<'_>) -> Result<(), Error> {
        let dataset = self.dataset;
        let bad = || {
            Error::shape(
                subject,
                "missing or changed reifier; restore the exact authored claim triple term",
            )
        };
        let (Some(reifier), Some(s), Some(p), Some(o)) = (
            dataset.term_id_by_iri(subject),
            dataset.term_id_by_iri(quote.capability),
            dataset.term_id_by_iri(CLAIMED_STATEMENT),
            dataset.term_id_by_literal(quote.statement, STRING, None, None),
        ) else {
            return Err(bad());
        };
        let target = dataset.term_id_by_triple(s, p, o).ok_or_else(bad)?;
        if !dataset
            .reifier_quads_of(reifier)
            .any(|row| row.o == target && row.g.is_none())
        {
            return Err(bad());
        }
        self.expected_reifiers += 1;
        Ok(())
    }

    fn statement(
        &mut self,
        relation: ClaimRelation,
        subject: &str,
        predicate: &str,
        object: Object<'_>,
    ) -> Result<(), Error> {
        let dataset = self.dataset;
        let object = match object {
            Object::Iri(iri) => dataset.term_id_by_iri(iri),
            Object::Literal(text, datatype) => {
                dataset.term_id_by_literal(text, datatype, None, None)
            }
        };
        let ids = (
            dataset.term_id_by_iri(subject),
            dataset.term_id_by_iri(predicate),
            object,
        );
        match relation {
            ClaimRelation::Assertion => {
                if let (Some(s), Some(p), Some(o)) = ids
                    && rows(dataset, s, p, Some(o)).next().is_some()
                {
                    self.expected += 1;
                    return Ok(());
                }
                Err(Error::shape(
                    subject,
                    "missing inventory or declaration assertion; restore the exact document projection",
                ))
            }
            ClaimRelation::OccurrenceProperty => {
                if let (Some(s), Some(p), Some(o)) = ids
                    && self.remaining_annotations.remove(&(s, p, o))
                {
                    return Ok(());
                }
                Err(Error::shape(
                    subject,
                    "missing or changed claim annotation; restore the exact authored occurrence property",
                ))
            }
        }
    }
}
