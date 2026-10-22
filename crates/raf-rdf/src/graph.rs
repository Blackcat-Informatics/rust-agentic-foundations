// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed-profile views over the authoritative native RDF dataset.

use crate::{Error, Format, TermValue, Triple, read, term, write};
use purrdf_core::{
    DatasetMut, GraphMatch, MutableDataset, QuadIds, QuadValues, RdfDataset, RdfDatasetBuilder,
    TermId, TermRef,
};
use std::{collections::BTreeSet, sync::Arc};

// Native storage requires absolute IRIs. Only this exact private owner and its
// fragments are mapped from local API input; external aliases are refused.
pub(crate) const CONSTRUCTION_OWNER: &str = "urn:raf:report:construction";

/// An immutable report backed by the original native RDF dataset.
#[derive(Clone, Debug)]
pub struct Graph {
    pub(crate) dataset: Arc<RdfDataset>,
    pub(crate) owner: String,
}

/// Native mutable construction with profile validation and duplicate refusal.
pub struct Builder {
    dataset: MutableDataset,
    owner: String,
}

/// A borrowed native term and its report-local IRI presentation.
#[derive(Clone, Copy)]
pub struct Value<'a> {
    graph: &'a Graph,
    id: TermId,
}

/// A subject query against native storage; it owns no assertion collection.
#[derive(Clone, Copy)]
pub struct Node<'a> {
    graph: &'a Graph,
    id: TermId,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Start an empty native dataset.
    ///
    /// # Panics
    /// If native validation of an empty dataset violates its invariant.
    pub fn new() -> Self {
        let base = RdfDatasetBuilder::new()
            .freeze()
            .expect("empty native RDF dataset");
        Self {
            dataset: MutableDataset::new(base),
            owner: CONSTRUCTION_OWNER.into(),
        }
    }

    /// Edit a native base without copying its assertions into another graph.
    pub fn from_graph(graph: &Graph) -> Self {
        Self {
            dataset: MutableDataset::new(Arc::clone(&graph.dataset)),
            owner: graph.owner.clone(),
        }
    }

    /// Insert one profile assertion through native set insertion.
    ///
    /// # Errors
    /// Refuses invalid roles, the reserved construction namespace and duplicates.
    pub fn insert(&mut self, mut triple: Triple) -> Result<(), Error> {
        triple.validate()?;
        for value in [
            &mut triple.subject,
            &mut triple.predicate,
            &mut triple.object,
        ] {
            term::expand(value, &self.owner, true)?;
        }
        let quad = QuadValues::triple(triple.subject, triple.predicate, triple.object);
        if !self
            .dataset
            .insert(quad)
            .map_err(|e| Error::Value(e.to_string()))?
        {
            return Err(Error::Repeated);
        }
        Ok(())
    }

    /// Query exact assertion membership through the native mutable index.
    ///
    /// # Errors
    /// Refuses invalid roles or an external alias of the construction owner.
    pub fn contains(&self, mut triple: Triple) -> Result<bool, Error> {
        triple.validate()?;
        term::expand(&mut triple.subject, &self.owner, true)?;
        term::expand(&mut triple.predicate, &self.owner, true)?;
        term::expand(&mut triple.object, &self.owner, true)?;
        Ok(self.dataset.contains(&QuadValues::triple(
            triple.subject,
            triple.predicate,
            triple.object,
        )))
    }

    /// Remove one exact assertion through native membership semantics.
    ///
    /// # Errors
    /// Refuses invalid profile roles or external owner aliases.
    pub fn remove(&mut self, mut triple: Triple) -> Result<bool, Error> {
        triple.validate()?;
        term::expand(&mut triple.subject, &self.owner, true)?;
        term::expand(&mut triple.predicate, &self.owner, true)?;
        term::expand(&mut triple.object, &self.owner, true)?;
        Ok(self.dataset.remove(&QuadValues::triple(
            triple.subject,
            triple.predicate,
            triple.object,
        )))
    }

    /// Freeze the native store and validate the closed report graph.
    ///
    /// # Errors
    /// Refuses invalid native state, an absent root or dangling owned reference.
    pub fn finish(self) -> Result<Graph, Error> {
        let graph = Graph {
            dataset: self
                .dataset
                .freeze()
                .map_err(|e| Error::Value(e.to_string()))?,
            owner: self.owner,
        };
        graph.validate()?;
        Ok(graph)
    }
}

impl Graph {
    pub(crate) fn rows(&self) -> impl Iterator<Item = QuadIds> + '_ {
        self.dataset
            .quads()
            .chain(self.dataset.reifier_quads())
            .chain(self.dataset.annotation_quads())
    }

    fn subject_rows(&self, s: TermId) -> impl Iterator<Item = QuadIds> + '_ {
        self.dataset
            .quads_for_pattern_with_plan(
                &RdfDataset::probe_plan(true, false, false, GraphMatch::Default),
                Some(s),
                None,
                None,
                GraphMatch::Default,
            )
            .chain(self.dataset.reifier_quads_of(s))
            .chain(
                self.dataset
                    .annotations_of_with_graph(s)
                    .map(move |(p, o, g)| QuadIds { s, p, o, g }),
            )
    }

    /// The native immutable storage, including its RDF 1.2 statement layers.
    pub fn dataset(&self) -> &RdfDataset {
        &self.dataset
    }

    /// Native flattened assertions with exact owned-root presentation.
    /// Each value is produced on demand; no parallel graph is retained.
    pub fn assertions(&self) -> impl Iterator<Item = Triple> + '_ {
        self.dataset.flat_default_graph_quads().map(|quad| {
            let mut triple = Triple {
                subject: quad.s,
                predicate: quad.p,
                object: quad.o,
            };
            term::normalize(&mut triple.subject, &self.owner);
            term::normalize(&mut triple.object, &self.owner);
            triple
        })
    }

    /// Present this exact owned absolute IRI as a local root/fragment.
    pub fn local<'a>(&self, iri: &'a str) -> &'a str {
        if iri == self.owner {
            ""
        } else {
            iri.strip_prefix(&self.owner)
                .filter(|tail| tail.starts_with('#'))
                .unwrap_or(iri)
        }
    }

    fn id(&self, local: &str) -> Option<TermId> {
        if term::local(local) {
            self.dataset
                .term_id_by_iri(&format!("{}{local}", self.owner))
        } else {
            self.dataset.term_id_by_iri(local)
        }
    }

    /// Query an asserted subject, not merely an interned object or unused term.
    pub fn get(&self, subject: &str) -> Option<Node<'_>> {
        let id = self.id(subject)?;
        self.subject_rows(id)
            .next()
            .map(|_| Node { graph: self, id })
    }

    /// Enumerate actual subject views without copying assertion values.
    pub fn nodes(&self) -> impl Iterator<Item = (String, Node<'_>)> {
        let subjects: BTreeSet<_> = self.rows().map(|row| row.s).collect();
        subjects.into_iter().map(|id| {
            let TermRef::Iri(iri) = self.dataset.resolve(id) else {
                unreachable!("validated subject")
            };
            (self.local(iri).to_owned(), Node { graph: self, id })
        })
    }

    /// Render either canonical spelling of this graph.
    ///
    /// # Errors
    /// Refuses invalid profile state or native output failure.
    pub fn to_text(&self, format: Format) -> Result<String, Error> {
        self.validate()?;
        write::render(self, format).map(|(_, text)| text)
    }

    /// Retain the native parser result after exact closed-profile acceptance.
    ///
    /// # Errors
    /// Refuses syntax, profile, digest or canonical-byte violations.
    pub fn from_text(text: &str, format: Format) -> Result<Self, Error> {
        read::parse(text, format)
    }

    /// The format-independent content-addressed snapshot IRI.
    ///
    /// # Errors
    /// Refuses invalid profile state or native output failure.
    pub fn snapshot(&self) -> Result<String, Error> {
        self.validate()?;
        write::base(self)
    }

    pub(crate) fn value(&self, id: TermId) -> Value<'_> {
        Value { graph: self, id }
    }

    pub(crate) fn validate(&self) -> Result<(), Error> {
        if self.dataset.named_graphs().next().is_some() {
            return Err(Error::Value(
                "named graphs are outside the report profile".into(),
            ));
        }
        if self.get("").is_none() {
            return Err(Error::Value("the report root is absent".into()));
        }
        let subjects: BTreeSet<_> = self.rows().map(|row| row.s).collect();
        // A native dataset can intentionally retain equal assertions in distinct
        // physical tables. This profile refuses that flat overlap. IDs are
        // compared only inside this one dictionary, never across datasets.
        let mut assertions = purrdf_core::FastSet::default();
        for row in self.rows() {
            if !assertions.insert(row) {
                return Err(Error::Repeated);
            }
            let subject = self
                .value(row.s)
                .iri()
                .ok_or_else(|| Error::Value("subject must be an IRI".into()))?;
            let TermRef::Iri(predicate) = self.dataset.resolve(row.p) else {
                return Err(Error::Value("predicate must be an IRI".into()));
            };
            let object = self.value(row.o).to_owned();
            term::validate(subject, predicate, &object)?;
            let check = |value: Value<'_>| -> Result<(), Error> {
                if value.iri().is_some_and(term::local) && !subjects.contains(&value.id) {
                    return Err(Error::Value(format!(
                        "dangling owned node {}",
                        value.iri().expect("IRI")
                    )));
                }
                Ok(())
            };
            let value = self.value(row.o);
            if let Some((s, _, o)) = value.quoted() {
                check(s)?;
                check(o)?;
            } else {
                check(value)?;
            }
        }
        Ok(())
    }
}

impl<'a> Value<'a> {
    /// The original borrowed native term representation.
    pub fn term(self) -> TermRef<'a> {
        self.graph.dataset.resolve(self.id)
    }
    /// Its IRI with exact owned-root normalization, when it is an IRI.
    pub fn iri(self) -> Option<&'a str> {
        match self.term() {
            TermRef::Iri(iri) => Some(self.graph.local(iri)),
            _ => None,
        }
    }
    /// Lexical text and datatype of a non-language literal.
    pub fn literal(self) -> Option<(&'a str, &'a str)> {
        if let TermRef::Literal {
            lexical,
            datatype,
            language: None,
            direction: None,
        } = self.term()
            && let TermRef::Iri(datatype) = self.graph.dataset.resolve(datatype)
        {
            return Some((lexical, datatype));
        }
        None
    }
    /// The borrowed native components of a quoted triple term.
    pub fn quoted(self) -> Option<(Self, Self, Self)> {
        match self.term() {
            TermRef::Triple { s, p, o } => Some((
                self.graph.value(s),
                self.graph.value(p),
                self.graph.value(o),
            )),
            _ => None,
        }
    }
    /// An operation-local native value key, with exact owner normalization.
    pub fn to_owned(self) -> TermValue {
        let mut value = self.graph.dataset.term_value(self.id);
        term::normalize(&mut value, &self.graph.owner);
        value
    }
}

impl<'a> Node<'a> {
    /// Values for an absolute property, borrowed from native storage.
    pub fn values(self, predicate: &str) -> impl Iterator<Item = Value<'a>> + use<'a> {
        let p = self.graph.dataset.term_id_by_iri(predicate);
        self.graph
            .subject_rows(self.id)
            .filter(move |row| Some(row.p) == p)
            .map(|row| self.graph.value(row.o))
    }
    /// Actual property names, including named triple-term provenance.
    pub fn properties(self) -> impl Iterator<Item = &'a str> {
        let predicates: BTreeSet<_> = self.graph.subject_rows(self.id).map(|row| row.p).collect();
        predicates
            .into_iter()
            .map(|id| match self.graph.dataset.resolve(id) {
                TermRef::Iri(iri) => iri,
                _ => unreachable!("validated predicate"),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use purrdf_core::RdfLiteral;

    #[test]
    fn native_physical_statement_overlap_is_refused_without_losing_the_valid_statement() {
        for overlap in [false, true] {
            let mut builder = RdfDatasetBuilder::new();
            let root = builder.intern_iri(CONSTRUCTION_OWNER).unwrap();
            let proof = builder
                .intern_iri(&format!("{CONSTRUCTION_OWNER}#proof"))
                .unwrap();
            let p = builder.intern_iri("urn:predicate").unwrap();
            let reifies = builder.intern_iri(crate::vocab::REIFIES).unwrap();
            let value = builder.intern_literal(RdfLiteral::simple("value")).unwrap();
            let statement = builder.intern_triple(root, p, value).unwrap();
            builder.push_quad(root, p, value, None).unwrap();
            builder.push_reifier(proof, statement).unwrap();
            builder.push_annotation(proof, p, value).unwrap();
            if overlap {
                builder.push_quad(proof, reifies, statement, None).unwrap();
                builder.push_quad(proof, p, value, None).unwrap();
            }
            let graph = Graph {
                dataset: builder.freeze().unwrap(),
                owner: CONSTRUCTION_OWNER.into(),
            };
            if overlap {
                assert_eq!(graph.validate(), Err(Error::Repeated));
            } else {
                graph.validate().unwrap();
                let node = graph.get("#proof").unwrap();
                assert_eq!(node.values(crate::vocab::REIFIES).count(), 1);
                assert_eq!(node.values("urn:predicate").count(), 1);
                for format in [Format::Turtle, Format::NTriples] {
                    let text = graph.to_text(format).unwrap();
                    let parsed = Graph::from_text(&text, format).unwrap();
                    assert_eq!(parsed.snapshot().unwrap(), graph.snapshot().unwrap());
                }
            }
        }
    }
}
