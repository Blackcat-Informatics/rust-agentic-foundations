// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Graph queries for assertions over canonical machine output.

use super::Run;
use raf_rdf::{Format, Graph, TermValue, vocab};

pub(crate) struct Rdf(Graph);
#[derive(Clone, Copy)]
pub(crate) struct Node<'a> {
    graph: &'a Graph,
    node: raf_rdf::Node<'a>,
}
impl Rdf {
    pub(crate) fn of(run: &Run) -> Self {
        let format = if run.stdout.starts_with("VERSION") {
            Format::Turtle
        } else {
            Format::NTriples
        };
        Self(
            Graph::from_text(&run.stdout, format)
                .unwrap_or_else(|e| panic!("invalid canonical RDF: {e}\n{run:#?}")),
        )
    }
    pub(crate) fn graph(&self) -> &Graph {
        &self.0
    }
    pub(crate) fn reparsed(&self, format: Format) -> Self {
        Self(Graph::from_text(&self.0.to_text(format).unwrap(), format).unwrap())
    }
    pub(crate) fn rows(&self, class: &str) -> Vec<Node<'_>> {
        let mut rows: Vec<_> = self
            .0
            .nodes()
            .filter_map(|(_, node)| {
                node.values(vocab::TYPE)
                    .any(|value| {
                        value.iri().and_then(|iri| iri.strip_prefix(vocab::REPORT)) == Some(class)
                    })
                    .then_some(Node {
                        graph: &self.0,
                        node,
                    })
            })
            .collect();
        rows.sort_by_cached_key(|row| row.number("ordinal"));
        rows
    }
    pub(crate) fn result_count(&self) -> usize {
        self.0
            .get("")
            .expect("the report has an asserted root")
            .values(&format!("{}result", vocab::REPORT))
            .count()
    }
}
impl<'a> Node<'a> {
    pub(crate) fn reified(self, property: &str) -> Self {
        let mut values = self.node.values(vocab::REIFIES);
        let (subject, predicate, object) = values
            .next()
            .expect("the observation reifies an assertion")
            .quoted()
            .expect("RDF 1.2 triple term");
        assert!(values.next().is_none(), "one assertion per observation");
        let key = format!("{}{property}", vocab::REPORT);
        assert_eq!(predicate.iri(), Some(key.as_str()));
        let node = self
            .graph
            .get(subject.iri().expect("named subject"))
            .unwrap();
        let mut assertions = node.values(&key);
        assert_eq!(assertions.next().unwrap().to_owned(), object.to_owned());
        assert!(assertions.next().is_none(), "one reported outcome");
        Self {
            graph: self.graph,
            node,
        }
    }
    fn literal(self, property: &str) -> Option<(&'a str, &'a str)> {
        let mut values = self.node.values(&format!("{}{property}", vocab::REPORT));
        let value = values.next().map(|value| {
            value
                .literal()
                .unwrap_or_else(|| panic!("{property} is not a literal: {:?}", value.term()))
        });
        assert!(
            values.next().is_none(),
            "repeated scalar property {property}"
        );
        value
    }
    pub(crate) fn text(self, property: &str) -> Option<&'a str> {
        self.literal(property).map(|(lexical, _)| lexical)
    }
    pub(crate) fn required(self, property: &str) -> &'a str {
        self.text(property)
            .unwrap_or_else(|| panic!("missing {property}"))
    }
    pub(crate) fn number(self, property: &str) -> usize {
        let (lexical, datatype) = self
            .literal(property)
            .unwrap_or_else(|| panic!("missing integer {property}"));
        assert_eq!(datatype, vocab::INTEGER, "integer datatype for {property}");
        assert!(
            lexical.len() <= 39,
            "{property} exceeds unsigned scalar width"
        );
        let value = purrdf_xsd::numeric::parse_integer_typed(
            lexical,
            purrdf_xsd::XsdDatatype::NonNegativeInteger,
        )
        .unwrap_or_else(|error| panic!("invalid unsigned {property}: {error}"));
        assert_eq!(
            lexical,
            value.canonical_lexical(),
            "noncanonical {property}"
        );
        let value = value
            .into_bigint()
            .unsigned_abs_u128()
            .unwrap_or_else(|| panic!("{property} exceeds u128"));
        usize::try_from(value).unwrap_or_else(|_| panic!("{property} exceeds usize"))
    }
    pub(crate) fn boolean(self, property: &str) -> bool {
        let (lexical, datatype) = self
            .literal(property)
            .unwrap_or_else(|| panic!("missing boolean {property}"));
        assert_eq!(datatype, vocab::BOOLEAN, "boolean datatype for {property}");
        let value = purrdf_xsd::simple::read_boolean(lexical)
            .unwrap_or_else(|error| panic!("invalid boolean {property}: {error}"));
        assert_eq!(
            TermValue::typed_literal(lexical, datatype),
            TermValue::boolean(value),
            "noncanonical boolean {property}"
        );
        value
    }
    pub(crate) fn children(self, property: &str) -> Vec<Self> {
        let mut values: Vec<_> = self
            .node
            .values(&format!("{}{property}", vocab::REPORT))
            .map(|value| {
                let iri = value.iri().unwrap_or_else(|| {
                    panic!("{property} is not a node reference: {:?}", value.term())
                });
                Self {
                    graph: self.graph,
                    node: self
                        .graph
                        .get(iri)
                        .unwrap_or_else(|| panic!("{property} names an unasserted node {iri}")),
                }
            })
            .collect();
        values.sort_by_cached_key(|row| row.number("ordinal"));
        values
    }
}
