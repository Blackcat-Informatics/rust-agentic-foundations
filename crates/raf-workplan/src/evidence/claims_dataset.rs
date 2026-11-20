// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native semantic claims; natural-language text remains an uninterpreted literal.

use std::{error::Error, fmt, sync::Arc};

use purrdf_core::{RdfDataset, RdfDatasetBuilder, RdfLiteral};
use raf_okf::{CapabilitySection, DocType, DocumentIdentityError};

use super::{EvidenceLookupError, evidence_id};
use crate::Workplan;
use crate::digest::semantic_hash as hash;

/// The semantic Workplan vocabulary, separate from snapshot report terms.
pub const CLAIMS_VOCAB: &str = "https://blackcatinformatics.ca/raf/workplan/1#";
const TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";
const NODES: &str = "https://blackcatinformatics.ca/raf/workplan/claims/";
const OCCURRENCE_IRI_BASE: &str = "https://blackcatinformatics.ca/raf/workplan/claims/occurrence/";
pub(crate) const CLAIMED_STATEMENT: &str =
    "https://blackcatinformatics.ca/raf/workplan/1#claimedStatement";

#[derive(Clone, Copy)]
pub(crate) enum Object<'a> {
    Iri(&'a str),
    Literal(&'a str, &'static str),
}

#[derive(Clone, Copy)]
pub(crate) struct QuotedClaim<'a> {
    pub(crate) capability: &'a str,
    pub(crate) statement: &'a str,
}

#[derive(Clone, Copy)]
pub(crate) enum ClaimRelation {
    Assertion,
    OccurrenceProperty,
}

#[derive(Clone, Copy)]
pub(crate) enum ClaimRow<'a> {
    Statement(ClaimRelation, &'a str, &'a str, Object<'a>),
    Reifier(&'a str, QuotedClaim<'a>),
}

impl<'a> ClaimRow<'a> {
    pub(crate) fn apply<W, E>(
        self,
        sink: &mut W,
        statement: impl FnOnce(&mut W, ClaimRelation, &'a str, &'a str, Object<'a>) -> Result<(), E>,
        reifier: impl FnOnce(&mut W, &'a str, QuotedClaim<'a>) -> Result<(), E>,
    ) -> Result<(), E> {
        match self {
            Self::Statement(relation, s, p, o) => statement(sink, relation, s, p, o),
            Self::Reifier(s, quote) => reifier(sink, s, quote),
        }
    }
}

pub(crate) struct ClaimProjection<'a> {
    pub(crate) quote: QuotedClaim<'a>,
    pub(crate) path: &'a str,
    pub(crate) line: usize,
    pub(crate) section: Option<CapabilitySection>,
}

impl ClaimProjection<'_> {
    pub(crate) fn rows<E, S: AsRef<str>>(
        &self,
        owner: &str,
        evidence: impl IntoIterator<Item = Result<S, E>>,
        mut emit: impl FnMut(ClaimRow<'_>) -> Result<(), E>,
    ) -> Result<(), E> {
        let occurrence = format!(
            "{OCCURRENCE_IRI_BASE}{}/{}",
            hash(self.quote.capability),
            self.line
        );
        emit(ClaimRow::Reifier(&occurrence, self.quote))?;
        emit(ClaimRow::Statement(
            ClaimRelation::OccurrenceProperty,
            &occurrence,
            TYPE,
            Object::Iri(&format!("{CLAIMS_VOCAB}ClaimOccurrence")),
        ))?;
        emit(ClaimRow::Statement(
            ClaimRelation::Assertion,
            owner,
            &format!("{CLAIMS_VOCAB}occurrence"),
            Object::Iri(&occurrence),
        ))?;
        for target in evidence {
            emit(ClaimRow::Statement(
                ClaimRelation::OccurrenceProperty,
                &occurrence,
                &format!("{CLAIMS_VOCAB}supportedBy"),
                Object::Iri(target?.as_ref()),
            ))?;
        }
        for (property, text, datatype) in [
            ("path", self.path, STRING),
            ("line", &self.line.to_string(), INTEGER),
        ] {
            emit(ClaimRow::Statement(
                ClaimRelation::OccurrenceProperty,
                &occurrence,
                &format!("{CLAIMS_VOCAB}{property}"),
                Object::Literal(text, datatype),
            ))?;
        }
        if let Some(section) = self.section {
            emit(ClaimRow::Statement(
                ClaimRelation::OccurrenceProperty,
                &occurrence,
                &format!("{CLAIMS_VOCAB}section"),
                Object::Literal(section.slug(), STRING),
            ))?;
        }
        Ok(())
    }
}

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
    fn row(&mut self, row: ClaimRow<'_>) -> Result<(), ClaimsDatasetError> {
        let native = ClaimsDatasetError::native;
        let dataset = &mut self.0;
        match row {
            ClaimRow::Statement(_, subject, predicate, object) => {
                let s = dataset.intern_iri(subject).map_err(native)?;
                let p = dataset.intern_iri(predicate).map_err(native)?;
                let o = match object {
                    Object::Iri(iri) => dataset.intern_iri(iri),
                    Object::Literal(text, datatype) => {
                        dataset.intern_literal(RdfLiteral::typed(text, datatype))
                    }
                }
                .map_err(native)?;
                dataset.push_quad(s, p, o, None).map_err(native)
            }
            ClaimRow::Reifier(subject, quote) => {
                let reifier = dataset.intern_iri(subject).map_err(native)?;
                let s = dataset.intern_iri(quote.capability).map_err(native)?;
                let p = dataset.intern_iri(CLAIMED_STATEMENT).map_err(native)?;
                let o = dataset
                    .intern_literal(RdfLiteral::typed(quote.statement, STRING))
                    .map_err(native)?;
                let triple = dataset.intern_triple(s, p, o).map_err(native)?;
                dataset.push_reifier(reifier, triple).map_err(native)
            }
        }
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
        let selection = format!("{NODES}selection/{}", hash(&evidence));
        writer.row(ClaimRow::Statement(
            ClaimRelation::Assertion,
            &selection,
            TYPE,
            Object::Iri(&format!("{CLAIMS_VOCAB}ClaimsSelection")),
        ))?;
        writer.row(ClaimRow::Statement(
            ClaimRelation::Assertion,
            &selection,
            &format!("{CLAIMS_VOCAB}evidence"),
            Object::Iri(&evidence),
        ))?;
        for claim in claims {
            let capability = DocType::Capability
                .document_iri(&claim.capability)
                .map_err(ClaimsDatasetError::Identity)?;
            ClaimProjection {
                quote: QuotedClaim {
                    capability: &capability,
                    statement: &claim.statement,
                },
                path: &claim.path,
                line: claim.line,
                section: claim.section,
            }
            .rows(&selection, [Ok(evidence.as_str())], |row| writer.row(row))?;
        }
        writer.0.freeze().map_err(ClaimsDatasetError::native)
    }
}
