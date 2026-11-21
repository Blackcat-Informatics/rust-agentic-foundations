// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native SHACL over top-level key presence, independent of the authoring codec.

use std::{
    fmt,
    sync::{Arc, OnceLock},
};

use purrdf_core::{RdfDataset, RdfDatasetBuilder, RdfDiagnostic, RdfLiteral};
use purrdf_lex::allocation::{Memory, OrderedTable, Resident, StorageError};
use purrdf_shapes::{
    ShapesError,
    engine::PreparedShapes,
    report::ValidationReport,
    shapes,
    term::{NamedNode, Term},
};

use super::{Check, DocType, Map, Rule};

const KEY: &str = "urn:raf:okf:key:";
const FOCUS: &str = "urn:raf:okf:header:";
const SHAPE: &str = "urn:raf:okf:header-shape:";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const XSD_BOOLEAN: &str = "http://www.w3.org/2001/XMLSchema#boolean";
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
const SH: &str = "http://www.w3.org/ns/shacl#";
const CLOSED: &str = "http://www.w3.org/ns/shacl#ClosedConstraintComponent";
const MIN_COUNT: &str = "http://www.w3.org/ns/shacl#MinCountConstraintComponent";

/// A failure to obtain a complete native verdict for a document's header.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HeaderShapeError {
    /// The authored key bytes and their line separators exceed the document bound.
    KeyBytes,
    /// Native RDF graph storage refused schema or header construction.
    Storage(StorageError),
    /// The native schema or header graph refused finalization.
    Graph(Box<RdfDiagnostic>),
    /// Native SHACL preparation, binding, or validation refused the graph.
    Native(Box<ShapesError>),
    /// The native report does not match the generated shape's result contract.
    UnexpectedReport,
}

impl fmt::Display for HeaderShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyBytes => write!(
                f,
                "top-level keys exceed the {} byte authoring bound",
                crate::MAX_TEXT_BYTES
            ),
            Self::Storage(source) => write!(
                f,
                "native header graph storage refused the operation ({source})"
            ),
            Self::Graph(source) => {
                write!(f, "native header graph could not be finalized ({source})")
            }
            Self::Native(source) => write!(f, "native SHACL refused the operation ({source})"),
            Self::UnexpectedReport => {
                f.write_str("native SHACL returned an unexpected header result")
            }
        }
    }
}

impl std::error::Error for HeaderShapeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(source) => Some(source),
            Self::Graph(source) => Some(source.as_ref()),
            Self::Native(source) => Some(source.as_ref()),
            Self::KeyBytes | Self::UnexpectedReport => None,
        }
    }
}

impl From<StorageError> for HeaderShapeError {
    fn from(source: StorageError) -> Self {
        Self::Storage(source)
    }
}

impl From<ShapesError> for HeaderShapeError {
    fn from(source: ShapesError) -> Self {
        Self::Native(Box::new(source))
    }
}

impl From<RdfDiagnostic> for HeaderShapeError {
    fn from(source: RdfDiagnostic) -> Self {
        Self::Graph(Box::new(source))
    }
}

pub(super) fn check(
    check: &mut Check<'_>,
    map: &Map,
    doc_type: DocType,
) -> Result<(), HeaderShapeError> {
    static SCHEMA: OnceLock<PreparedShapes> = OnceLock::new();
    let focus = format!("{FOCUS}{}", doc_type.name());
    let data = header_graph(&focus, map)?;
    let binding = if let Some(schema) = SCHEMA.get() {
        schema.bind_shared_dataset(data)?
    } else {
        let schema = prepare()?;
        // Binding surfaces any refusal retained by the native preparation.
        let binding = schema.bind_shared_dataset(data)?;
        SCHEMA.get_or_init(|| schema);
        binding
    };
    let focus = NamedNode::from(focus.as_str()).into_term();
    let report = binding.validate_focus_nodes(std::slice::from_ref(&focus))?;
    report_keys(&report, &focus, map, doc_type, check)
}

fn prepare() -> Result<PreparedShapes, HeaderShapeError> {
    let mut graph = RdfDatasetBuilder::new();
    let rdf_type = graph.intern_iri(RDF_TYPE)?;
    let node_shape = graph.intern_iri(&format!("{SH}NodeShape"))?;
    let target = graph.intern_iri(&format!("{SH}targetNode"))?;
    let closed = graph.intern_iri(&format!("{SH}closed"))?;
    let property = graph.intern_iri(&format!("{SH}property"))?;
    let path = graph.intern_iri(&format!("{SH}path"))?;
    let min_count = graph.intern_iri(&format!("{SH}minCount"))?;
    let yes = graph.intern_literal(RdfLiteral::typed("true", XSD_BOOLEAN))?;
    let one = graph.intern_literal(RdfLiteral::typed("1", XSD_INTEGER))?;
    for doc_type in DocType::ALL {
        let name = doc_type.name();
        let shape = graph.intern_iri(&format!("{SHAPE}{name}"))?;
        let focus = graph.intern_iri(&format!("{FOCUS}{name}"))?;
        graph.push_quad(shape, rdf_type, node_shape, None)?;
        graph.push_quad(shape, target, focus, None)?;
        graph.push_quad(shape, closed, yes, None)?;
        for spec in doc_type.keys() {
            let key = spec.key;
            let field = graph.intern_iri(&format!("{SHAPE}{name}:{key}"))?;
            let predicate = graph.intern_iri(&format!("{KEY}{key}"))?;
            graph.push_quad(shape, property, field, None)?;
            graph.push_quad(field, path, predicate, None)?;
            if spec.required {
                graph.push_quad(field, min_count, one, None)?;
            }
        }
    }
    let shapes = shapes::from_dataset(&graph.freeze()?)?;
    Ok(PreparedShapes::new(Arc::new(shapes)))
}

fn header_graph(focus: &str, map: &Map) -> Result<Arc<RdfDataset>, HeaderShapeError> {
    // Both document constructors include at least the key, colon, and newline
    // in the authoring byte limit. Each admitted key gives one default-graph
    // triple. The separate report profile would require a report root.
    let mut available = crate::MAX_TEXT_BYTES;
    for entry in map.entries() {
        available = available
            .checked_sub(2)
            .and_then(|remaining| remaining.checked_sub(entry.key().len()))
            .ok_or(HeaderShapeError::KeyBytes)?;
    }
    let mut graph = RdfDatasetBuilder::new();
    let focus = graph.intern_iri(focus)?;
    let present = graph.intern_iri("urn:raf:okf:key-present")?;
    for entry in map.entries() {
        let predicate = graph.intern_iri(&format!("{KEY}{}", entry.key()))?;
        graph.push_quad(focus, predicate, present, None)?;
    }
    Ok(graph.freeze()?)
}

fn report_keys(
    report: &ValidationReport,
    focus: &Term,
    map: &Map,
    doc_type: DocType,
    check: &mut Check<'_>,
) -> Result<(), HeaderShapeError> {
    if report.conforms != report.results.is_empty() || !report.diagnostics.is_empty() {
        return Err(HeaderShapeError::UnexpectedReport);
    }
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut unknown = OrderedTable::default();
    let mut missing = OrderedTable::default();
    for result in &report.results {
        let Some(Term::NamedNode(path)) = &result.result_path else {
            return Err(HeaderShapeError::UnexpectedReport);
        };
        let key = path
            .as_str()
            .strip_prefix(KEY)
            .ok_or(HeaderShapeError::UnexpectedReport)?;
        let keys = match result.source_constraint_component.as_str() {
            CLOSED => &mut unknown,
            MIN_COUNT => &mut missing,
            _ => return Err(HeaderShapeError::UnexpectedReport),
        };
        if &result.focus_node != focus || keys.insert(key, (), &mut memory)?.is_some() {
            return Err(HeaderShapeError::UnexpectedReport);
        }
    }
    // Report unknown keys in author order, then missing keys in schema order.
    for entry in map.entries() {
        let key = entry.key();
        if unknown.remove(&key).is_some() {
            let rule = if key == "resource" {
                Rule::ResourceKey { doc_type }
            } else {
                Rule::UnknownKey { key: key.into() }
            };
            check.key(key, rule);
        }
    }
    for spec in doc_type.keys() {
        if spec.required && missing.remove(&spec.key).is_some() {
            check.key(
                spec.key,
                Rule::MissingKey {
                    key: spec.key.into(),
                },
            );
        }
    }
    if !unknown.is_empty() || !missing.is_empty() {
        return Err(HeaderShapeError::UnexpectedReport);
    }
    Ok(())
}
