// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native semantic claims; natural-language text remains an uninterpreted literal.

use std::{error::Error, fmt, sync::Arc};

use purrdf_core::{RdfDataset, RdfDatasetBuilder, RdfLiteral, TermId};
use raf_okf::{DocType, DocumentIdentityError};

use super::{EvidenceLookupError, evidence_id};
use crate::Workplan;
use crate::digest::semantic_hash as hash;

/// The semantic Workplan vocabulary, separate from snapshot report terms.
pub const CLAIMS_VOCAB: &str = "https://blackcatinformatics.ca/raf/workplan/1#";
const TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
const NODES: &str = "https://blackcatinformatics.ca/raf/workplan/claims/";

/// Why a complete semantic claim selection could not be produced.
#[derive(Debug)]
#[non_exhaustive]
pub enum ClaimsDatasetError {
    /// The same unknown-id or incomplete-capability refusal as the text query.
    Lookup(EvidenceLookupError),
    /// The shared document identity rules refused a reference.
    Identity(DocumentIdentityError),
    /// Original native storage or RDF validation failure.
    Native(Box<dyn Error + Send + Sync>),
}

impl fmt::Display for ClaimsDatasetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lookup(error) => error.fmt(f),
            Self::Identity(error) => error.fmt(f),
            Self::Native(error) => write!(f, "native evidence claims: {error}"),
        }
    }
}
impl Error for ClaimsDatasetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Lookup(error) => Some(error),
            Self::Identity(error) => Some(error),
            Self::Native(error) => Some(error.as_ref()),
        }
    }
}
impl ClaimsDatasetError {
    fn native(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Native(Box::new(error))
    }
}

struct Writer(RdfDatasetBuilder);
impl Writer {
    fn iri(&mut self, iri: &str) -> Result<TermId, ClaimsDatasetError> {
        self.0.intern_iri(iri).map_err(ClaimsDatasetError::native)
    }
    fn property(&mut self, name: &str) -> Result<TermId, ClaimsDatasetError> {
        self.iri(&format!("{CLAIMS_VOCAB}{name}"))
    }
    fn put(
        &mut self,
        subject: TermId,
        name: &str,
        object: TermId,
    ) -> Result<(), ClaimsDatasetError> {
        let predicate = self.property(name)?;
        self.0
            .push_quad(subject, predicate, object, None)
            .map_err(ClaimsDatasetError::native)
    }
    fn literal(&mut self, text: &str, datatype: &str) -> Result<TermId, ClaimsDatasetError> {
        self.0
            .intern_literal(RdfLiteral {
                lexical_form: text.to_string(),
                datatype: Some(datatype.to_string()),
                language: None,
                direction: None,
            })
            .map_err(ClaimsDatasetError::native)
    }
    fn text(&mut self, subject: TermId, name: &str, text: &str) -> Result<(), ClaimsDatasetError> {
        let value = self.literal(text, STRING)?;
        self.put(subject, name, value)
    }
    fn class(&mut self, subject: TermId, name: &str) -> Result<(), ClaimsDatasetError> {
        let predicate = self.iri(TYPE)?;
        let role = self.property(name)?;
        self.0
            .push_quad(subject, predicate, role, None)
            .map_err(ClaimsDatasetError::native)
    }
}

impl Workplan<'_> {
    /// Every authored claim naming `evidence`, as a complete semantic dataset.
    ///
    /// Each marker occurrence reifies the unasserted triple term whose subject
    /// is its stable capability document IRI, predicate is `claimedStatement`
    /// in [`CLAIMS_VOCAB`], and object is the exact statement string. The
    /// occurrence names the stable supporting evidence IRI and preserves its
    /// path, line and optional section. This interprets no natural language
    /// and asserts neither the quoted triple nor the statement's truth.
    ///
    /// A deterministic `ClaimsSelection` root names the selected evidence even
    /// when no claim names it. Distinct markers have distinct occurrence IRIs,
    /// derived from the capability IRI's native hash and marker line. The
    /// document and occurrence identities do not depend on this query's order.
    ///
    /// # Errors
    /// Preserves [`Self::evidence_claims`] lookup refusals, checked identity
    /// errors and native storage/validation error sources. No file is written.
    pub fn evidence_claims_dataset(
        &self,
        evidence: &str,
    ) -> Result<Arc<RdfDataset>, ClaimsDatasetError> {
        let claims = self
            .evidence_claims(evidence)
            .map_err(ClaimsDatasetError::Lookup)?;
        let evidence = evidence_id(evidence).map_err(ClaimsDatasetError::Lookup)?;
        let evidence = DocType::Evidence
            .document_iri(&evidence)
            .map_err(ClaimsDatasetError::Identity)?;
        let mut writer = Writer(RdfDatasetBuilder::new());
        let selection = writer.iri(&format!("{NODES}selection/{}", hash(&evidence)))?;
        let evidence = writer.iri(&evidence)?;
        writer.class(selection, "ClaimsSelection")?;
        writer.put(selection, "evidence", evidence)?;
        for claim in claims {
            let capability = DocType::Capability
                .document_iri(&claim.capability)
                .map_err(ClaimsDatasetError::Identity)?;
            let occurrence = writer.iri(&format!(
                "{NODES}occurrence/{}/{}",
                hash(&capability),
                claim.line
            ))?;
            let capability = writer.iri(&capability)?;
            let predicate = writer.property("claimedStatement")?;
            let statement = writer.literal(&claim.statement, STRING)?;
            let triple = writer
                .0
                .intern_triple(capability, predicate, statement)
                .map_err(ClaimsDatasetError::native)?;
            writer
                .0
                .push_reifier(occurrence, triple)
                .map_err(ClaimsDatasetError::native)?;
            writer.class(occurrence, "ClaimOccurrence")?;
            writer.put(selection, "occurrence", occurrence)?;
            writer.put(occurrence, "supportedBy", evidence)?;
            writer.text(occurrence, "path", &claim.path)?;
            let line = writer.literal(&claim.line.to_string(), INTEGER)?;
            writer.put(occurrence, "line", line)?;
            if let Some(section) = claim.section {
                writer.text(occurrence, "section", section.slug())?;
            }
        }
        writer.0.freeze().map_err(ClaimsDatasetError::native)
    }
}
