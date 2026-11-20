// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed repository snapshots of RAF authoring documents and declarations.

use std::{collections::BTreeMap, fmt, sync::Arc};

use purrdf_core::{QuadIds, RdfDataset, TermId, TermRef};
use raf_okf::{
    Body, Capability, CapabilitySection, DOCUMENT_IRI_BASE, DatasetError, DocType, Document,
    EvidenceRecord, Index, Input,
};
use raf_rdf::vocab::REIFIES;
use raf_source::RelPath;

use crate::{
    Finding, Workplan,
    evidence::{CLAIMED_STATEMENT, ClaimProjection, ClaimRow, Object, QuotedClaim, statement_text},
    generated::INDEX_PATH,
    model::IndexSlot,
};

mod read;
mod write;

const FRAME: &str = "https://blackcatinformatics.ca/raf/workplan/repository/1";
const INDEX_IRI: &str = "https://blackcatinformatics.ca/raf/workplan/repository/1/index";
const INDEX_EDGE: &str = "https://blackcatinformatics.ca/raf/workplan/authoring/1#index";
const VOCAB: &str = "https://blackcatinformatics.ca/raf/workplan/authoring/1#";
const TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
const MAX_DOCUMENTS: usize = 4_096;
const MAX_ROWS: usize = 1_048_576;
const MAX_LEXICAL_BYTES: usize = 128 * 1024 * 1024;
const MAX_NATIVE_BYTES: usize = 256 * 1024 * 1024;

/// A closed repository snapshot could not be written or recovered.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The loader could not admit the complete profile-document inventory.
    Incomplete(Box<Finding>),
    /// The exact document codec refused a named document.
    Document {
        /// Repository-relative path.
        path: String,
        /// Original document refusal.
        source: Box<DatasetError>,
    },
    /// The navigation Index codec refused the repository Index.
    Index(Box<DatasetError>),
    /// The inventory or projected declarations do not match the closed shape.
    Shape {
        /// The document, assertion or repository identity to repair.
        subject: String,
        /// The violated shape requirement.
        reason: &'static str,
    },
    /// A repository-wide bound prevented admission.
    Limit {
        /// Document or repository identity at the refusal.
        subject: String,
        /// The fixed limit that was exceeded.
        bound: &'static str,
    },
    /// Native storage refused the repository graph operation.
    Native {
        /// Document or repository identity at the refusal.
        subject: String,
        /// Original native refusal.
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Incomplete(finding) => {
                write!(formatter, "incomplete authoring inventory: {finding}")
            }
            Self::Document { path, source } => {
                write!(formatter, "authoring document {path}: {source}")
            }
            Self::Index(source) => write!(formatter, "authoring Index {INDEX_PATH}: {source}"),
            Self::Shape { subject, reason } => {
                write!(formatter, "authoring graph {subject}: {reason}")
            }
            Self::Limit { subject, bound } => {
                write!(formatter, "authoring graph {subject} exceeds {bound}")
            }
            Self::Native { subject, source } => {
                write!(formatter, "native authoring graph {subject}: {source}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Document { source, .. } | Self::Index(source) => Some(source.as_ref()),
            Self::Native { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl Error {
    fn shape(subject: &str, reason: &'static str) -> Self {
        Self::Shape {
            subject: bounded(subject),
            reason,
        }
    }

    fn limit(subject: &str, bound: &'static str) -> Self {
        Self::Limit {
            subject: bounded(subject),
            bound,
        }
    }

    fn document(path: &str, source: DatasetError) -> Self {
        Self::Document {
            path: bounded(path),
            source: Box::new(source),
        }
    }

    fn index(source: DatasetError) -> Self {
        Self::Index(Box::new(source))
    }

    fn native(subject: &str, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Native {
            subject: bounded(subject),
            source: Box::new(source),
        }
    }
}

fn bounded(subject: &str) -> String {
    subject.chars().take(160).collect()
}

fn canonical_path(path: &str) -> Result<RelPath, Error> {
    let file = path.rsplit('/').next().unwrap_or(path);
    if path.len() > raf_okf::MAX_TEXT_BYTES
        || path.split('/').any(|part| matches!(part, "" | "." | ".."))
        || !crate::load::is_markdown(file)
        || crate::load::is_navigation_index(file)
    {
        return Err(Error::shape(
            path,
            "a canonical relative profile document path is required",
        ));
    }
    RelPath::new(path).map_err(|_| {
        Error::shape(
            path,
            "a canonical relative profile document path is required",
        )
    })
}

impl Workplan<'_> {
    /// Project profile documents with their declared bindings, dependencies,
    /// evidence relationships, record inputs and authored claim reifiers with
    /// occurrence annotations into a closed default-graph RDF 1.2 dataset,
    /// including the loaded navigation Index.
    ///
    /// The export uses the profile Document model and declared spellings. A
    /// missing capability target still has its declared IRI, and a binding
    /// does not require a resolved source symbol.
    ///
    /// # Errors
    /// Refuses incomplete profile input, duplicate paths or document IRIs,
    /// individual document refusals, and repository resource bounds.
    pub fn authoring_dataset(&self) -> Result<Arc<RdfDataset>, Error> {
        if let Some(finding) = &self.authoring_fault {
            return Err(Error::Incomplete(Box::new(finding.clone())));
        }
        let index = match &self.index {
            IndexSlot::Absent => None,
            IndexSlot::Valid(index) => Some(index),
            IndexSlot::Invalid(finding) => {
                return Err(Error::Incomplete(Box::new(finding.clone())));
            }
        };
        write::repository(self, index)
    }
}

/// The recovered authoring content of a repository.
#[derive(Debug)]
pub struct RepositoryAuthoring {
    /// Profile authoring documents keyed by canonical repository-relative paths.
    pub documents: BTreeMap<RelPath, Document>,
    /// Navigation at `docs/capabilities/index.md`, when present.
    pub index: Option<Index>,
}

/// Recover the exact profile documents and optional navigation Index.
///
/// Checks the closed inventory and declared relationship occurrences against
/// the recovered profile values. No source index is read.
///
/// # Errors
/// Refuses missing, duplicate or unrelated subjects, malformed documents,
/// invalid Index membership or carrier, inconsistent declarations or claim
/// rows, unsupported RDF layers and resource limits.
pub fn repository_from_dataset(dataset: &RdfDataset) -> Result<RepositoryAuthoring, Error> {
    read::repository(dataset)
}

fn property(name: &str) -> String {
    format!("{VOCAB}{name}")
}

fn record_iri(root: &str) -> Result<String, Error> {
    let local = root.strip_prefix(DOCUMENT_IRI_BASE).ok_or_else(|| {
        Error::shape(
            root,
            "document identity must use the asserted OKF namespace",
        )
    })?;
    Ok(format!("{FRAME}/document/{local}"))
}

fn root_statements(
    mut emit: impl FnMut(&str, &str, Object<'_>) -> Result<(), Error>,
) -> Result<(), Error> {
    emit(FRAME, TYPE, Object::Iri(&property("Repository")))
}

#[derive(Clone, Copy)]
enum Declarations<'a> {
    None,
    Capability(&'a Capability),
    Evidence(&'a EvidenceRecord),
    Inputs(&'a [Input]),
}

fn declarations(
    (path, root): (&str, &str),
    declared: Declarations<'_>,
    mut emit: impl FnMut(&str, &str, Object<'_>) -> Result<(), Error>,
) -> Result<(), Error> {
    let record = record_iri(root)?;
    emit(FRAME, &property("document"), Object::Iri(&record))?;
    emit(&record, TYPE, Object::Iri(&property("DocumentPath")))?;
    emit(&record, &property("documentIdentity"), Object::Iri(root))?;
    emit(&record, &property("path"), Object::Literal(path, STRING))?;
    let (segment, edge, class, targets) = match declared {
        Declarations::None => return Ok(()),
        Declarations::Inputs(rows) => return inputs(&record, root, rows, emit),
        Declarations::Capability(capability) => {
            bindings(&record, root, capability, &mut emit)?;
            (
                "dependency",
                "dependency",
                "Dependency",
                &capability.depends,
            )
        }
        Declarations::Evidence(evidence) => {
            inputs(&record, root, &evidence.inputs, &mut emit)?;
            (
                "evidence-for",
                "evidenceFor",
                "EvidenceFor",
                &evidence.covers,
            )
        }
    };
    let edge = property(edge);
    let class = property(class);
    for (ordinal, declared_target) in targets.iter().enumerate() {
        let target = DocType::Capability
            .document_iri(declared_target)
            .map_err(|source| Error::document(path, DatasetError::Identity(source)))?;
        let row = format!("{record}/{segment}/{ordinal}");
        emit(&record, &edge, Object::Iri(&row))?;
        emit(&row, TYPE, Object::Iri(&class))?;
        emit(&row, &property("owner"), Object::Iri(root))?;
        emit(
            &row,
            &property("ordinal"),
            Object::Literal(&ordinal.to_string(), INTEGER),
        )?;
        emit(&row, &property("target"), Object::Iri(&target))?;
    }
    Ok(())
}

fn claims(
    (path, capability): (&str, &str),
    body: &Body,
    mut emit: impl FnMut(ClaimRow<'_>) -> Result<(), Error>,
) -> Result<(), Error> {
    if body.claims().is_empty() {
        return Ok(());
    }
    let record = record_iri(capability)?;
    for marker in body.claims() {
        let evidence = marker.evidence().iter().map(|slug| {
            DocType::Evidence
                .document_iri(&format!("{}{slug}", DocType::Evidence.id_prefix()))
                .map_err(|source| Error::document(path, DatasetError::Identity(source)))
        });
        ClaimProjection {
            quote: QuotedClaim {
                capability,
                statement: statement_text(body, marker),
            },
            path,
            line: marker.line(),
            section: marker.section().and_then(CapabilitySection::from_slug),
        }
        .rows(&record, evidence, &mut emit)?;
    }
    Ok(())
}

fn bindings(
    record: &str,
    root: &str,
    capability: &Capability,
    mut emit: impl FnMut(&str, &str, Object<'_>) -> Result<(), Error>,
) -> Result<(), Error> {
    for (kind, values) in [
        ("code", &capability.bindings.code),
        ("tests", &capability.bindings.tests),
        ("examples", &capability.bindings.examples),
    ] {
        for (ordinal, spelling) in values.iter().enumerate() {
            let row = format!("{record}/binding/{kind}/{ordinal}");
            emit(record, &property("binding"), Object::Iri(&row))?;
            emit(&row, TYPE, Object::Iri(&property("Binding")))?;
            emit(&row, &property("owner"), Object::Iri(root))?;
            emit(&row, &property("kind"), Object::Iri(&property(kind)))?;
            emit(
                &row,
                &property("ordinal"),
                Object::Literal(&ordinal.to_string(), INTEGER),
            )?;
            emit(
                &row,
                &property("spelling"),
                Object::Literal(spelling, STRING),
            )?;
        }
    }
    Ok(())
}

fn inputs(
    record: &str,
    root: &str,
    inputs: &[Input],
    mut emit: impl FnMut(&str, &str, Object<'_>) -> Result<(), Error>,
) -> Result<(), Error> {
    for (ordinal, input) in inputs.iter().enumerate() {
        let row = format!("{record}/input/{ordinal}");
        emit(record, &property("input"), Object::Iri(&row))?;
        emit(&row, TYPE, Object::Iri(&property("Input")))?;
        emit(&row, &property("owner"), Object::Iri(root))?;
        emit(
            &row,
            &property("ordinal"),
            Object::Literal(&ordinal.to_string(), INTEGER),
        )?;
        emit(
            &row,
            &property("kind"),
            Object::Iri(&property(input.kind.name())),
        )?;
        emit(
            &row,
            &property("name"),
            Object::Literal(&input.name, STRING),
        )?;
        emit(
            &row,
            &property("digest"),
            Object::Literal(&input.digest, STRING),
        )?;
    }
    Ok(())
}

#[derive(Default)]
struct Counts {
    rows: usize,
    bytes: usize,
}

impl Counts {
    fn row<const N: usize>(&mut self, subject: &str, terms: [&str; N]) -> Result<(), Error> {
        self.rows += 1;
        if self.rows > MAX_ROWS {
            return Err(Error::limit(
                subject,
                "1048576 assertion, reifier and annotation rows",
            ));
        }
        self.bytes = terms
            .iter()
            .fold(self.bytes, |sum, term| sum.saturating_add(term.len()));
        if self.bytes > MAX_LEXICAL_BYTES {
            return Err(Error::limit(
                subject,
                "128 MiB of assertion, reifier and annotation lexical text",
            ));
        }
        Ok(())
    }

    fn reifier(&mut self, subject: &str, quote: QuotedClaim<'_>) -> Result<(), Error> {
        self.row(
            subject,
            [
                subject,
                REIFIES,
                quote.capability,
                CLAIMED_STATEMENT,
                quote.statement,
            ],
        )
    }

    fn native_reifier(&mut self, dataset: &RdfDataset, row: QuadIds) -> Result<(), Error> {
        let subject = iri(dataset, row.s)?;
        let bad = || {
            Error::shape(
                subject,
                "use a default-graph reifier of one authored claim triple term",
            )
        };
        if row.g.is_some() {
            return Err(bad());
        }
        let TermRef::Triple { s, p, o } = dataset.resolve(row.o) else {
            return Err(bad());
        };
        let TermRef::Literal {
            lexical,
            datatype,
            language: None,
            direction: None,
        } = dataset.resolve(o)
        else {
            return Err(bad());
        };
        if iri(dataset, p)? != CLAIMED_STATEMENT || iri(dataset, datatype)? != STRING {
            return Err(bad());
        }
        self.reifier(
            subject,
            QuotedClaim {
                capability: iri(dataset, s)?,
                statement: lexical,
            },
        )
    }

    fn native(&mut self, dataset: &RdfDataset, row: QuadIds) -> Result<(), Error> {
        let subject = iri(dataset, row.s)?;
        let predicate = iri(dataset, row.p)?;
        if row.g.is_some() {
            return Err(Error::shape(subject, "default graph rows are required"));
        }
        let object = match dataset.resolve(row.o) {
            TermRef::Iri(iri) => iri,
            TermRef::Literal {
                lexical,
                language: None,
                direction: None,
                ..
            } => lexical,
            _ => return Err(Error::shape(subject, "use IRIs and untagged literals only")),
        };
        self.row(subject, [subject, predicate, object])
    }
}

fn iri(dataset: &RdfDataset, id: TermId) -> Result<&str, Error> {
    match dataset.resolve(id) {
        TermRef::Iri(iri) => Ok(iri),
        _ => Err(Error::shape(FRAME, "expected an IRI")),
    }
}
