// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The separate, closed navigation Index carrier.

use std::sync::Arc;

use purrdf_core::{BaseIri, RdfDataset, RdfDatasetBuilder, RdfLiteral, TermRef};

use super::{Budget, DOCUMENT_IRI_BASE, DatasetError as E, STRING, TYPE, VOCAB};
use crate::Index;

const CLASS: &str = "https://blackcatinformatics.ca/raf/okf/1#NavigationIndex";
const BODY: &str = "https://blackcatinformatics.ca/raf/okf/1#body";
const NODE_IRI_BASE: &str = "https://blackcatinformatics.ca/raf/okf/node/";

fn admit(iri: &str, body: &str) -> Result<(), E> {
    let mut budget = Budget::default();
    budget.text(iri)?;
    budget.text(body)?;
    BaseIri::parse(iri).map_err(E::native)?;
    if iri.starts_with(DOCUMENT_IRI_BASE) || iri.starts_with(NODE_IRI_BASE) {
        return Err(E::shape(
            iri,
            "navigation identity must not use the document or structural-node namespace",
        ));
    }
    Ok(())
}

impl Index {
    /// Carry this navigation Index as two asserted native RDF 1.2 statements.
    ///
    /// The caller supplies an absolute IRI whose exact spelling is retained.
    /// The body literal contains the complete original Index bytes.
    ///
    /// # Errors
    /// Refuses malformed or reserved identities, combined IRI/body text over
    /// 1 MiB, and native storage failures, retaining native error causes.
    pub fn to_dataset(&self, iri: &str) -> Result<Arc<RdfDataset>, E> {
        let body = self.body().text();
        admit(iri, body)?;
        let mut store = RdfDatasetBuilder::new();
        let root = store.intern_iri(iri).map_err(E::native)?;
        let predicate = store.intern_iri(TYPE).map_err(E::native)?;
        let class = store.intern_iri(CLASS).map_err(E::native)?;
        store
            .push_quad(root, predicate, class, None)
            .map_err(E::native)?;
        let predicate = store.intern_iri(BODY).map_err(E::native)?;
        let body = store
            .intern_literal(RdfLiteral::typed(body, STRING))
            .map_err(E::native)?;
        store
            .push_quad(root, predicate, body, None)
            .map_err(E::native)?;
        store.freeze().map_err(E::native)
    }

    /// Recover an Index from its separate closed carrier and expected identity.
    ///
    /// Identity equality uses exact spelling. Reconstruction uses [`Index::parse`]
    /// and retains its original body locations and errors.
    ///
    /// # Errors
    /// Refuses extra statements, subjects, predicates, duplicate assertions,
    /// wrong types or cardinalities, blank nodes, named graphs, statement layers,
    /// malformed or reserved identities, the aggregate text bound and invalid bodies.
    pub fn from_dataset(dataset: &RdfDataset, expected_iri: &str) -> Result<Self, E> {
        if dataset.named_graphs().next().is_some()
            || dataset.reifiers().next().is_some()
            || dataset.annotations().next().is_some()
        {
            return Err(E::shape(
                "Index",
                "named graphs and statement layers are unsupported",
            ));
        }
        let mut body = None;
        let mut typed = false;
        for (position, quad) in dataset.quads().enumerate() {
            if position >= 2 || quad.g.is_some() {
                return Err(E::shape(
                    "Index",
                    "exactly two asserted default-graph statements required",
                ));
            }
            if !matches!(dataset.resolve(quad.s), TermRef::Iri(iri) if iri == expected_iri) {
                return Err(E::shape(
                    "Index",
                    "every subject must equal the expected IRI exactly",
                ));
            }
            match dataset.resolve(quad.p) {
                TermRef::Iri(TYPE)
                    if !typed && matches!(dataset.resolve(quad.o), TermRef::Iri(CLASS)) =>
                {
                    typed = true;
                }
                TermRef::Iri(BODY) if body.is_none() => {
                    let TermRef::Literal {
                        lexical,
                        datatype,
                        language: None,
                        direction: None,
                    } = dataset.resolve(quad.o)
                    else {
                        return Err(E::shape("Index", "body must be one xsd:string literal"));
                    };
                    if !matches!(dataset.resolve(datatype), TermRef::Iri(STRING)) {
                        return Err(E::shape("Index", "body must have datatype xsd:string"));
                    }
                    body = Some(lexical);
                }
                _ => {
                    return Err(E::shape(
                        "Index",
                        format!(
                            "only one rdf:type {VOCAB}NavigationIndex and one okf:body are allowed"
                        ),
                    ));
                }
            }
        }
        let body = body.filter(|_| typed).ok_or_else(|| {
            E::shape(
                "Index",
                "one NavigationIndex type and one body are required",
            )
        })?;
        admit(expected_iri, body)?;
        Self::parse(body).map_err(E::Parse)
    }
}
