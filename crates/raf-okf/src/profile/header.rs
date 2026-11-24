// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native SHACL over authored keys and closed scalar values.

use std::{
    fmt,
    sync::{Arc, OnceLock},
};

use purrdf_core::{
    RdfDataset, RdfDatasetBuilder, RdfDiagnostic, RdfLiteral,
    collections::{ListVocab, try_build_rdf_list},
};
use purrdf_lex::allocation::{Memory, OrderedTable, Resident, StorageError};
use purrdf_shapes::{
    ShapesError,
    engine::PreparedShapes,
    report::{ValidationReport, ValidationResult},
    shapes,
    term::{NamedNode, Term},
};

use super::{BINDING_KEYS, Check, DocType, Entry, ItemValue, Map, Rule};
use super::{finite::Vocabulary, review::FindingStatus};
use crate::frontmatter::value::List;

const KEY: &str = "urn:raf:okf:key:";
const PRESENT: &str = "urn:raf:okf:key-present";
const FOCUS: &str = "urn:raf:okf:header:";
const SHAPE: &str = "urn:raf:okf:header-shape:";
const BINDINGS_SHAPE: &str = "Capability-bindings";
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const XSD_BOOLEAN: &str = "http://www.w3.org/2001/XMLSchema#boolean";
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const SH: &str = "http://www.w3.org/ns/shacl#";
const CLOSED: &str = "http://www.w3.org/ns/shacl#ClosedConstraintComponent";
const MIN_COUNT: &str = "http://www.w3.org/ns/shacl#MinCountConstraintComponent";
const IN: &str = "http://www.w3.org/ns/shacl#InConstraintComponent";

/// A failure to obtain a complete native verdict for a document's header.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HeaderShapeError {
    /// The projected header bytes exceed the document bound.
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
                "projected frontmatter exceeds the {} byte authoring bound",
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

#[derive(Default)]
pub(super) struct HeaderVerdicts<'a> {
    rejected: Vec<(&'static str, Rule)>,
    pub(super) bindings_exact: bool,
    pub(super) items: Vec<RowKeys<'a>>,
    pub(super) debt_findings: Vec<RowKeys<'a>>,
    pub(super) debt_lines: Vec<RowKeys<'a>>,
    pub(super) inputs: Vec<RowKeys<'a>>,
    pub(super) exclusions: Vec<RowKeys<'a>>,
    pub(super) ceilings: Vec<RowKeys<'a>>,
    pub(super) exceptions: Vec<RowKeys<'a>>,
    pub(super) review_findings: Vec<RowKeys<'a>>,
}

impl HeaderVerdicts<'_> {
    pub(super) fn take_rejection(&mut self, key: &str) -> Option<Rule> {
        let index = self.rejected.iter().position(|(field, _)| *field == key)?;
        Some(self.rejected.swap_remove(index).1)
    }
}

pub(super) struct RowKeys<'a> {
    pub(super) unknown: Vec<&'a str>,
    pub(super) missing: Vec<&'static str>,
}

impl RowKeys<'_> {
    pub(super) fn is_exact(&self) -> bool {
        self.unknown.is_empty() && self.missing.is_empty()
    }
}

#[derive(Clone, Copy)]
enum RowShape {
    ChangeItem,
    DebtFinding,
    DebtLine,
    Input,
    Exclusion,
    Ceiling,
    Exception,
    ReviewFinding,
    ReviewFindingWithRevision,
    ReviewFindingWithReason,
}

struct RowSpec {
    list: &'static str,
    name: &'static str,
    member: &'static str,
    keys: &'static [(&'static str, bool)],
}

impl RowShape {
    const fn spec(self) -> RowSpec {
        match self {
            Self::ChangeItem => RowSpec {
                list: "items",
                name: "Change-item",
                member: "urn:raf:okf:change-item-member",
                keys: &[("claim", true), ("outcome", false), ("reason", false)],
            },
            Self::DebtFinding => RowSpec {
                list: "findings",
                name: "Debt-finding",
                member: "urn:raf:okf:debt-finding-member",
                keys: &[("kind", true), ("subject", true)],
            },
            Self::DebtLine => RowSpec {
                list: "lines",
                name: "Debt-line",
                member: "urn:raf:okf:debt-line-member",
                keys: &[("item", true), ("orphans", true)],
            },
            Self::Input => RowSpec {
                list: "inputs",
                name: "Record-input",
                member: "urn:raf:okf:record-input-member",
                keys: &[("kind", true), ("name", true), ("digest", true)],
            },
            Self::Exclusion => RowSpec {
                list: "exclusions",
                name: "Coverage-exclusion",
                member: "urn:raf:okf:coverage-exclusion-member",
                keys: &[("item", true), ("reason", true)],
            },
            Self::Ceiling => RowSpec {
                list: "ceilings",
                name: "Coverage-ceiling",
                member: "urn:raf:okf:coverage-ceiling-member",
                keys: &[
                    ("crate", true),
                    ("lines", false),
                    ("percent", false),
                    ("of", false),
                ],
            },
            Self::Exception => RowSpec {
                list: "exceptions",
                name: "Coverage-exception",
                member: "urn:raf:okf:coverage-exception-member",
                keys: &[("item", false), ("file", false), ("reason", true)],
            },
            Self::ReviewFinding => RowSpec {
                list: "findings",
                name: "Review-finding",
                member: "urn:raf:okf:review-finding-member",
                keys: &[
                    ("id", true),
                    ("severity", true),
                    ("location", true),
                    ("status", true),
                ],
            },
            Self::ReviewFindingWithRevision => RowSpec {
                list: "findings",
                name: "Review-finding-with-revision",
                member: "urn:raf:okf:review-finding-with-revision-member",
                keys: &[
                    ("id", true),
                    ("severity", true),
                    ("location", true),
                    ("status", true),
                    ("revision", true),
                ],
            },
            Self::ReviewFindingWithReason => RowSpec {
                list: "findings",
                name: "Review-finding-with-reason",
                member: "urn:raf:okf:review-finding-with-reason-member",
                keys: &[
                    ("id", true),
                    ("severity", true),
                    ("location", true),
                    ("status", true),
                    ("reason", true),
                ],
            },
        }
    }

    fn for_row(self, map: &Map) -> Self {
        if !matches!(self, Self::ReviewFinding) {
            return self;
        }
        match FindingStatus::in_row(map) {
            Some(FindingStatus::Fixed) => Self::ReviewFindingWithRevision,
            Some(FindingStatus::Accepted | FindingStatus::FalsePositive) => {
                Self::ReviewFindingWithReason
            }
            // A malformed status is refused by the typed decoder.
            Some(FindingStatus::Open) | None => self,
        }
    }
}

#[derive(Clone, Copy)]
struct RowGroup<'a> {
    shape: RowShape,
    list: &'a List,
}

impl<'a> RowGroup<'a> {
    fn maps(self) -> impl Iterator<Item = &'a Map> {
        self.list
            .items()
            .iter()
            .map_while(|item| match item.value() {
                ItemValue::Map(map) => Some(map),
                ItemValue::Scalar(_) => None,
            })
    }
}

/// Reports top-level key findings and returns scalar and nested key verdicts.
pub(super) fn check<'a>(
    check: &mut Check<'_>,
    map: &'a Map,
    doc_type: DocType,
) -> Result<HeaderVerdicts<'a>, HeaderShapeError> {
    static SCHEMA: OnceLock<PreparedShapes> = OnceLock::new();
    let focus = format!("{FOCUS}{}", doc_type.name());
    let bindings = if doc_type == DocType::Capability {
        map.get("bindings").and_then(|entry| entry.value().as_map())
    } else {
        None
    };
    let binding_focus = bindings.map(|_| format!("{FOCUS}{BINDINGS_SHAPE}"));
    let groups = row_groups(map, doc_type);
    let (data, focuses) = header_graph(
        (focus.as_str(), map),
        binding_focus.as_deref().zip(bindings),
        &groups,
        doc_type,
    )?;
    let binding = if let Some(schema) = SCHEMA.get() {
        schema.bind_shared_dataset(data)?
    } else {
        let schema = prepare()?;
        // Binding surfaces any refusal retained by the native preparation.
        let binding = schema.bind_shared_dataset(data)?;
        SCHEMA.get_or_init(|| schema);
        binding
    };
    let report = binding.validate_focus_nodes(&focuses)?;
    report_keys(&report, &focuses, map, doc_type, &groups, check)
}

fn prepare() -> Result<PreparedShapes, HeaderShapeError> {
    let mut graph = RdfDatasetBuilder::new();
    for doc_type in DocType::ALL {
        add_shape(
            &mut graph,
            doc_type.name(),
            ("targetNode", &format!("{FOCUS}{}", doc_type.name())),
            doc_type.keys().iter().map(|spec| (spec.key, spec.required)),
        )?;
        add_value_constraints(&mut graph, doc_type)?;
    }
    add_shape(
        &mut graph,
        BINDINGS_SHAPE,
        ("targetNode", &format!("{FOCUS}{BINDINGS_SHAPE}")),
        BINDING_KEYS.into_iter().map(|key| (key, true)),
    )?;
    for shape in [
        RowShape::ChangeItem,
        RowShape::DebtFinding,
        RowShape::DebtLine,
        RowShape::Input,
        RowShape::Exclusion,
        RowShape::Ceiling,
        RowShape::Exception,
        RowShape::ReviewFinding,
        RowShape::ReviewFindingWithRevision,
        RowShape::ReviewFindingWithReason,
    ] {
        let spec = shape.spec();
        add_shape(
            &mut graph,
            spec.name,
            ("targetObjectsOf", spec.member),
            spec.keys.iter().copied(),
        )?;
    }
    let shapes = shapes::from_dataset(&graph.freeze()?)?;
    Ok(PreparedShapes::new(Arc::new(shapes)))
}

fn add_shape(
    graph: &mut RdfDatasetBuilder,
    name: &str,
    (target, focus): (&str, &str),
    keys: impl Iterator<Item = (&'static str, bool)>,
) -> Result<(), HeaderShapeError> {
    let rdf_type = graph.intern_iri(&format!("{RDF}type"))?;
    let node_shape = graph.intern_iri(&format!("{SH}NodeShape"))?;
    let target = graph.intern_iri(&format!("{SH}{target}"))?;
    let closed = graph.intern_iri(&format!("{SH}closed"))?;
    let property = graph.intern_iri(&format!("{SH}property"))?;
    let path = graph.intern_iri(&format!("{SH}path"))?;
    let min_count = graph.intern_iri(&format!("{SH}minCount"))?;
    let yes = graph.intern_literal(RdfLiteral::typed("true", XSD_BOOLEAN))?;
    let one = graph.intern_literal(RdfLiteral::typed("1", XSD_INTEGER))?;
    let shape = graph.intern_iri(&format!("{SHAPE}{name}"))?;
    let focus = graph.intern_iri(focus)?;
    graph.push_quad(shape, rdf_type, node_shape, None)?;
    graph.push_quad(shape, target, focus, None)?;
    graph.push_quad(shape, closed, yes, None)?;
    for (key, required) in keys {
        let field = graph.intern_iri(&format!("{SHAPE}{name}:{key}"))?;
        let predicate = graph.intern_iri(&format!("{KEY}{key}"))?;
        graph.push_quad(shape, property, field, None)?;
        graph.push_quad(field, path, predicate, None)?;
        if required {
            graph.push_quad(field, min_count, one, None)?;
        }
    }
    Ok(())
}

fn finite_fields(doc_type: DocType) -> impl Iterator<Item = (&'static str, Vocabulary)> {
    doc_type.keys().iter().filter_map(|spec| {
        Vocabulary::for_kind(spec.value).map(|vocabulary| (spec.key, vocabulary))
    })
}

fn add_value_constraints(
    graph: &mut RdfDatasetBuilder,
    doc_type: DocType,
) -> Result<(), HeaderShapeError> {
    let predicate = graph.intern_iri(&format!("{SH}in"))?;
    let list = ListVocab {
        first: graph.intern_iri(&format!("{RDF}first"))?,
        rest: graph.intern_iri(&format!("{RDF}rest"))?,
        nil: graph.intern_iri(&format!("{RDF}nil"))?,
    };
    for (key, vocabulary) in finite_fields(doc_type) {
        let name = format!("{SHAPE}{}:{key}", doc_type.name());
        let field = graph.intern_iri(&name)?;
        let members = vocabulary
            .names(doc_type)?
            .into_iter()
            .map(|text| graph.intern_literal(RdfLiteral::simple(text)))
            .collect::<Result<Vec<_>, _>>()?;
        let cells = (0..members.len())
            .map(|index| graph.intern_iri(&format!("{name}:in:{index}")))
            .collect::<Result<Vec<_>, _>>()?;
        let head = try_build_rdf_list(
            members,
            &list,
            |index| Ok(cells[index]),
            |subject, predicate, object| graph.push_quad(subject, predicate, object, None),
        )?;
        graph.push_quad(field, predicate, head, None)?;
    }
    Ok(())
}

fn row_groups(map: &Map, doc_type: DocType) -> Vec<RowGroup<'_>> {
    let shapes: &[RowShape] = match doc_type {
        DocType::Change => &[RowShape::ChangeItem],
        DocType::Debt => &[RowShape::DebtFinding, RowShape::DebtLine],
        DocType::Evidence | DocType::Assessment => &[RowShape::Input],
        DocType::Review => &[RowShape::Input, RowShape::ReviewFinding],
        DocType::Coverage => &[RowShape::Exclusion, RowShape::Ceiling, RowShape::Exception],
        _ => &[],
    };
    shapes
        .iter()
        .filter_map(|&shape| {
            let list = map.get(shape.spec().list)?.value().as_list()?;
            // Change reports item keys until the first scalar. The other
            // lists refuse the entire list before checking any row keys.
            if !matches!(shape, RowShape::ChangeItem)
                && list
                    .items()
                    .iter()
                    .any(|item| matches!(item.value(), ItemValue::Scalar(_)))
            {
                return None;
            }
            Some(RowGroup { shape, list })
        })
        .collect()
}

fn header_graph<'a>(
    header: (&str, &'a Map),
    bindings: Option<(&str, &'a Map)>,
    groups: &[RowGroup<'a>],
    doc_type: DocType,
) -> Result<(Arc<RdfDataset>, Vec<Term>), HeaderShapeError> {
    // Both document constructors include at least the key, colon, and newline
    // in the authoring byte limit. Each admitted key gives one default-graph
    // triple. The separate report profile would require a report root.
    let rows = groups.iter().copied().flat_map(RowGroup::maps);
    let row_count = rows.count();
    // Each row also supplies the dash and space of its list marker.
    let mut available = row_count
        .checked_mul(2)
        .and_then(|markers| crate::MAX_TEXT_BYTES.checked_sub(markers))
        .ok_or(HeaderShapeError::KeyBytes)?;
    let maps = std::iter::once(header).chain(bindings);
    for entry in maps
        .clone()
        .map(|(_, map)| map)
        .chain(groups.iter().copied().flat_map(RowGroup::maps))
        .flat_map(Map::entries)
    {
        available = available
            .checked_sub(2)
            .and_then(|remaining| remaining.checked_sub(entry.key().len()))
            .ok_or(HeaderShapeError::KeyBytes)?;
    }
    let scalars = finite_fields(doc_type)
        .filter_map(|(key, vocabulary)| Some((key, vocabulary.texts(header.1.get(key)?)?)))
        .collect::<Vec<_>>();
    for text in scalars.iter().flat_map(|(_, texts)| texts) {
        available = available
            .checked_sub(text.len())
            .ok_or(HeaderShapeError::KeyBytes)?;
    }
    let mut graph = RdfDatasetBuilder::new();
    let present = graph.intern_iri(PRESENT)?;
    let mut focuses = Vec::with_capacity(maps.clone().count() + row_count);
    let mut project =
        |graph: &mut RdfDatasetBuilder, (name, map): (&str, &Map), values: &[(&str, Vec<&str>)]| {
            let focus = graph.intern_iri(name)?;
            focuses.push(NamedNode::from(name).into_term());
            for entry in map.entries() {
                let predicate = graph.intern_iri(&format!("{KEY}{}", entry.key()))?;
                let Some((_, texts)) = values.iter().find(|(key, _)| *key == entry.key()) else {
                    graph.push_quad(focus, predicate, present, None)?;
                    continue;
                };
                // Each list member is its own value; an empty list has none.
                for text in texts {
                    let object = graph.intern_literal(RdfLiteral::simple(*text))?;
                    graph.push_quad(focus, predicate, object, None)?;
                }
            }
            Ok::<_, HeaderShapeError>(focus)
        };
    project(&mut graph, header, &scalars)?;
    if let Some(bindings) = bindings {
        project(&mut graph, bindings, &[])?;
    }
    if row_count != 0 {
        let anchor = graph.intern_iri("urn:raf:okf:header-rows")?;
        for &group in groups {
            let spec = group.shape.spec();
            let member = graph.intern_iri(spec.member)?;
            for (index, map) in group.maps().enumerate() {
                let row_member = if matches!(group.shape, RowShape::ReviewFinding) {
                    graph.intern_iri(group.shape.for_row(map).spec().member)?
                } else {
                    member
                };
                let name = format!("{FOCUS}{}:{index}", spec.name);
                let focus = project(&mut graph, (&name, map), &[])?;
                graph.push_quad(anchor, row_member, focus, None)?;
            }
        }
    }
    Ok((graph.freeze()?, focuses))
}

fn focus_iri(term: &Term) -> Result<&str, HeaderShapeError> {
    match term {
        Term::NamedNode(name) => Ok(name.as_str()),
        _ => Err(HeaderShapeError::UnexpectedReport),
    }
}

fn missing_row_keys<'a>(
    missing: &mut OrderedTable<(&'a str, &'a str), ()>,
    focus: &'a str,
    row: &Map,
    keys: &[(&'static str, bool)],
) -> Result<Vec<&'static str>, HeaderShapeError> {
    let mut result = Vec::new();
    for &(key, required) in keys {
        if required {
            let absent = missing.remove(&(focus, key)).is_some();
            if absent != row.get(key).is_none() {
                return Err(HeaderShapeError::UnexpectedReport);
            }
            if absent {
                result.push(key);
            }
        }
    }
    Ok(result)
}

fn value_rejection(
    result: &ValidationResult,
    map: &Map,
    doc_type: DocType,
    key: &str,
) -> Result<(&'static str, Rule), HeaderShapeError> {
    let (key, vocabulary) = finite_fields(doc_type)
        .find(|(field, _)| *field == key)
        .ok_or(HeaderShapeError::UnexpectedReport)?;
    let entry = map.get(key).ok_or(HeaderShapeError::UnexpectedReport)?;
    if focus_iri(&result.source_shape)? != format!("{SHAPE}{}:{key}", doc_type.name()) {
        return Err(HeaderShapeError::UnexpectedReport);
    }
    match (vocabulary.texts(entry), &result.value) {
        (Some(texts), Some(Term::Literal(value)))
            if value.datatype_str() == XSD_STRING && texts.contains(&value.value()) => {}
        (None, Some(Term::NamedNode(value))) if value.as_str() == PRESENT => {}
        _ => return Err(HeaderShapeError::UnexpectedReport),
    }
    Ok((key, vocabulary.rejection(entry, doc_type)))
}

fn report_top_keys<'a>(
    map: &'a Map,
    doc_type: DocType,
    focus: &'a str,
    unknown: &mut OrderedTable<(&'a str, &'a str), ()>,
    missing: &mut OrderedTable<(&'a str, &'a str), ()>,
    check: &mut Check<'_>,
) {
    // Report unknown keys in author order, then missing keys in schema order.
    for entry in map.entries() {
        let key = entry.key();
        if unknown.remove(&(focus, key)).is_some() {
            let rule = if key == "resource" {
                Rule::ResourceKey { doc_type }
            } else {
                Rule::UnknownKey { key: key.into() }
            };
            check.key(key, rule);
        }
    }
    for spec in doc_type.keys() {
        if spec.required && missing.remove(&(focus, spec.key)).is_some() {
            check.key(
                spec.key,
                Rule::MissingKey {
                    key: spec.key.into(),
                },
            );
        }
    }
}

fn report_keys<'a>(
    report: &ValidationReport,
    focuses: &[Term],
    map: &'a Map,
    doc_type: DocType,
    groups: &[RowGroup<'a>],
    check: &mut Check<'_>,
) -> Result<HeaderVerdicts<'a>, HeaderShapeError> {
    if report.conforms != report.results.is_empty() || !report.diagnostics.is_empty() {
        return Err(HeaderShapeError::UnexpectedReport);
    }
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut unknown = OrderedTable::default();
    let mut missing = OrderedTable::default();
    let (header, nested) = focuses
        .split_first()
        .ok_or(HeaderShapeError::UnexpectedReport)?;
    let header = focus_iri(header)?;
    let (bindings, row_focuses) = if doc_type == DocType::Capability {
        (nested.first().map(focus_iri).transpose()?, &[][..])
    } else {
        (None, nested)
    };
    let mut verdicts = HeaderVerdicts {
        bindings_exact: true,
        ..HeaderVerdicts::default()
    };
    for result in &report.results {
        let Some(Term::NamedNode(path)) = &result.result_path else {
            return Err(HeaderShapeError::UnexpectedReport);
        };
        let key = path
            .as_str()
            .strip_prefix(KEY)
            .ok_or(HeaderShapeError::UnexpectedReport)?;
        let focus = focus_iri(&result.focus_node)?;
        let keys = match result.source_constraint_component.as_str() {
            CLOSED => &mut unknown,
            MIN_COUNT => &mut missing,
            IN if focus == header => {
                let rejection = value_rejection(result, map, doc_type, key)?;
                if verdicts.rejected.iter().any(|(field, _)| *field == key) {
                    // A list reports each refused member; one finding covers the key.
                    if finite_fields(doc_type)
                        .any(|(field, vocabulary)| field == key && vocabulary.is_list())
                    {
                        continue;
                    }
                    return Err(HeaderShapeError::UnexpectedReport);
                }
                verdicts.rejected.push(rejection);
                continue;
            }
            _ => return Err(HeaderShapeError::UnexpectedReport),
        };
        if bindings == Some(focus) {
            verdicts.bindings_exact = false;
            continue;
        }
        if keys.insert((focus, key), (), &mut memory)?.is_some() {
            return Err(HeaderShapeError::UnexpectedReport);
        }
    }
    report_top_keys(map, doc_type, header, &mut unknown, &mut missing, check);
    let mut focuses = row_focuses.iter();
    for &group in groups {
        let rows = match group.shape {
            RowShape::ChangeItem => &mut verdicts.items,
            RowShape::DebtFinding => &mut verdicts.debt_findings,
            RowShape::DebtLine => &mut verdicts.debt_lines,
            RowShape::Input => &mut verdicts.inputs,
            RowShape::Exclusion => &mut verdicts.exclusions,
            RowShape::Ceiling => &mut verdicts.ceilings,
            RowShape::Exception => &mut verdicts.exceptions,
            RowShape::ReviewFinding
            | RowShape::ReviewFindingWithRevision
            | RowShape::ReviewFindingWithReason => &mut verdicts.review_findings,
        };
        for map in group.maps() {
            let spec = group.shape.for_row(map).spec();
            let focus = focus_iri(focuses.next().ok_or(HeaderShapeError::UnexpectedReport)?)?;
            let missing_keys = missing_row_keys(&mut missing, focus, map, spec.keys)?;
            let unknown = map
                .entries()
                .iter()
                .map(Entry::key)
                .filter(|key| unknown.remove(&(focus, *key)).is_some())
                .collect();
            rows.push(RowKeys {
                unknown,
                missing: missing_keys,
            });
        }
    }
    if focuses.next().is_some() || !unknown.is_empty() || !missing.is_empty() {
        return Err(HeaderShapeError::UnexpectedReport);
    }
    Ok(verdicts)
}
