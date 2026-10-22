// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed, canonical ground RDF 1.2 report profiles.
//!
//! Named snapshot subjects, absolute vocabulary IRIs, typed literals and
//! depth-one triple terms are supported. Blank nodes, quads, language-tagged
//! literals and recursively nested triple terms are outside this profile.
//! Domain-specific classes, cardinalities and semantic invariants belong to
//! the consumer. This is not a general RDF parser.

mod error;
mod graph;
mod read;
mod term;
/// Shared report vocabulary and snapshot identity namespaces.
pub mod vocab;
mod write;

pub use error::Error;
pub use graph::{Builder, Graph, Node, Value};
pub use purrdf_rdf::{TermBox, TermValue};
pub use term::Triple;

/// The two canonical machine spellings of the same report graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// Turtle 1.2 with a version directive and a content-addressed BASE.
    Turtle,
    /// N-Triples 1.2 with absolute IRIs and no directives.
    NTriples,
}
