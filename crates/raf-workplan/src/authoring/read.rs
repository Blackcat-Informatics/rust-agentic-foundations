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
    RepositoryAuthoring, STRING, canonical_path, declarations, iri, property, record_iri,
    root_statements,
};

struct Group<'a> {
    path: &'a str,
    subjects: Vec<TermId>,
}

type Groups<'a> = BTreeMap<TermId, Group<'a>>;

pub(super) fn repository(dataset: &RdfDataset) -> Result<RepositoryAuthoring, Error> {
    if dataset.named_graphs().next().is_some()
        || dataset.reifiers().next().is_some()
        || dataset.annotations().next().is_some()
    {
        return Err(Error::shape(
            FRAME,
            "only asserted default-graph statements are admitted",
        ));
    }
    let index_subject = index_member(dataset)?;
    let mut verifier = Verifier {
        dataset,
        expected: usize::from(index_subject.is_some()),
    };
    root_statements(|s, p, o| verifier.statement(s, p, o))?;
    let mut groups = inventory(dataset)?;
    let metadata_rows = route(dataset, &mut groups, index_subject)?;
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
            _ => Declarations::None,
        };
        declarations((group.path, iri(dataset, id)?), declared, |s, p, o| {
            verifier.statement(s, p, o)
        })?;
        documents.insert(canonical_path(group.path)?, document);
    }
    if metadata_rows != verifier.expected {
        return Err(Error::shape(
            FRAME,
            "unexpected inventory or declaration assertions; use only the recovered document projections",
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
) -> Result<usize, Error> {
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
    Ok(metadata)
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
            .and_then(|index| prefixes.get(&name[..=index]).copied())
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
}

impl Verifier<'_> {
    fn statement(
        &mut self,
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
        if let (Some(s), Some(p), Some(o)) = (
            dataset.term_id_by_iri(subject),
            dataset.term_id_by_iri(predicate),
            object,
        ) && rows(dataset, s, p, Some(o)).next().is_some()
        {
            self.expected += 1;
            return Ok(());
        }
        Err(Error::shape(
            subject,
            "missing inventory or declaration assertion; restore the exact document projection",
        ))
    }
}
