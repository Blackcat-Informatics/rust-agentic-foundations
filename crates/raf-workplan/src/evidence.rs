// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Authored evidence metadata and static claim relationships.
//! Stored verdicts describe what their authors recorded; reading them never
//! runs their subject or establishes a runtime verification result.

mod claims_dataset;
pub use claims_dataset::{CLAIMS_VOCAB, ClaimsDatasetError};

use crate::Workplan;
use raf_okf::{CapabilitySection, DocType, EvidenceVerdict, is_kebab};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// A statement of a capability document that rests on an evidence record:
/// the paragraph or list item a claim marker follows.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ClaimStatement {
    /// The capability id.
    pub capability: String,
    /// The repository-relative path of its document.
    pub path: String,
    /// The section that holds the marker, or `None` before the first section.
    pub section: Option<CapabilitySection>,
    /// The 1-based line of the marker.
    pub line: usize,
    /// The text of the statement, as written.
    pub statement: String,
}

/// An evidence record that supports a capability.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Support {
    /// The id of the record, `evidence:<slug>`.
    pub evidence: String,
    /// The repository-relative path of the record.
    pub path: String,
    /// The verdict of the record, or `None` when no record loaded under the
    /// id (it was removed, or does not validate).
    pub verdict: Option<EvidenceVerdict>,
    /// Whether the record's `for` lists the capability.
    pub listed: bool,
    /// The lines of the claim markers of the capability that name the record.
    pub claims: Vec<usize>,
}

/// Why a dependency view has no answer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EvidenceLookupError {
    /// The id is not `evidence:<slug>` or a slug.
    BadId {
        /// The id as given.
        id: String,
    },
    /// No evidence record has the id and no claim names it.
    UnknownEvidence {
        /// The id.
        id: String,
    },
    /// Capability documents exist as files and did not load, so the claims
    /// they hold are unknown and no answer about claims is complete.
    UnloadedCapabilities {
        /// The ids of those documents, sorted.
        ids: Vec<String>,
    },
    /// The capability is not loaded.
    Capability {
        /// The id as given.
        id: String,
        /// Whether a document of the id exists and did not load.
        not_loaded: bool,
    },
}

impl fmt::Display for EvidenceLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadId { id } => write!(
                f,
                "`{id}` is not an evidence id; write `evidence:<slug>` or the slug"
            ),
            Self::UnknownEvidence { id } => write!(
                f,
                "no evidence record has the id `{id}` and no claim names it"
            ),
            Self::UnloadedCapabilities { ids } => write!(
                f,
                "{} did not load, so a claim in it is unknown; fix its document first",
                ids.join(", ")
            ),
            Self::Capability {
                id,
                not_loaded: true,
            } => write!(f, "{id} did not load; fix its document first"),
            Self::Capability { id, .. } => write!(f, "`{id}` names no capability document"),
        }
    }
}

impl Error for EvidenceLookupError {}

/// The id `evidence:<slug>` of `id`, given with or without the prefix.
pub(crate) fn evidence_id(id: &str) -> Result<String, EvidenceLookupError> {
    let prefix = DocType::Evidence.id_prefix();
    let slug = id.strip_prefix(prefix).unwrap_or(id);
    if is_kebab(slug) {
        Ok(format!("{prefix}{slug}"))
    } else {
        Err(EvidenceLookupError::BadId { id: id.to_string() })
    }
}

/// The path of the record `evidence:<slug>`.
pub(crate) fn record_path(id: &str) -> String {
    let slug = id.strip_prefix(DocType::Evidence.id_prefix()).unwrap_or(id);
    format!("{}/{slug}.md", DocType::Evidence.directory())
}

impl Workplan<'_> {
    /// The statements that rest on the evidence `evidence` (`evidence:<slug>`
    /// or the slug): every claim marker of a capability document that names
    /// it, outside PROPOSED blocks, with its file, line, section and the text
    /// of its statement, by capability and line. A claim on a record that did
    /// not load is listed too, because the statement still rests on it.
    ///
    /// # Errors
    ///
    /// [`EvidenceLookupError::BadId`];
    /// [`EvidenceLookupError::UnloadedCapabilities`] when a capability
    /// document exists and did not load, since a claim in it may name the
    /// evidence (a change or evidence record that did not load holds no
    /// claim and leaves the answer as it is); and
    /// [`EvidenceLookupError::UnknownEvidence`] when no record file has the
    /// id and no claim names it.
    pub fn evidence_claims(
        &self,
        evidence: &str,
    ) -> Result<Vec<ClaimStatement>, EvidenceLookupError> {
        let id = evidence_id(evidence)?;
        let unloaded: Vec<String> = self
            .unloaded()
            .into_iter()
            .filter(|unloaded| unloaded.starts_with(DocType::Capability.id_prefix()))
            .collect();
        if !unloaded.is_empty() {
            return Err(EvidenceLookupError::UnloadedCapabilities { ids: unloaded });
        }
        let slug = id
            .strip_prefix(DocType::Evidence.id_prefix())
            .unwrap_or(&id);
        let mut found = Vec::new();
        for cap in self.capabilities() {
            let body = cap.body();
            let base = body.span().bytes.start;
            for claim in body.claims() {
                if !claim.evidence().iter().any(|named| named == slug) {
                    continue;
                }
                let span = &claim.statement().bytes;
                let statement = body
                    .text()
                    .get(span.start.saturating_sub(base)..span.end.saturating_sub(base))
                    .unwrap_or_default()
                    .to_string();
                found.push(ClaimStatement {
                    capability: cap.value().header.id.clone(),
                    path: cap.path().to_string(),
                    section: claim.section().and_then(CapabilitySection::from_slug),
                    line: claim.line(),
                    statement,
                });
            }
        }
        if found.is_empty() && !self.is_present(&id) {
            return Err(EvidenceLookupError::UnknownEvidence { id });
        }
        found.sort_by(|a, b| (&a.capability, a.line).cmp(&(&b.capability, b.line)));
        Ok(found)
    }

    /// The entry of `id` in `found`, made from the loaded record when there is
    /// none yet.
    fn support_entry<'m>(
        &self,
        found: &'m mut BTreeMap<String, Support>,
        id: &str,
        capability: &str,
    ) -> &'m mut Support {
        found.entry(id.to_string()).or_insert_with(|| {
            let record = self
                .evidence()
                .iter()
                .find(|record| record.value().header.id == id);
            Support {
                evidence: id.to_string(),
                path: record_path(id),
                verdict: record.map(|record| record.value().verdict),
                listed: record
                    .is_some_and(|record| record.value().covers.iter().any(|c| c == capability)),
                claims: Vec::new(),
            }
        })
    }

    /// The evidence records that support the capability `capability`: those
    /// whose `for` lists it and those its claims name, by id, each with its
    /// verdict and the lines of the claims that name it.
    ///
    /// # Errors
    ///
    /// [`EvidenceLookupError::Capability`] when the capability is not loaded.
    pub fn evidence_for(&self, capability: &str) -> Result<Vec<Support>, EvidenceLookupError> {
        let Some(cap) = self
            .capabilities()
            .iter()
            .find(|cap| cap.value().header.id == capability)
        else {
            return Err(EvidenceLookupError::Capability {
                id: capability.to_string(),
                not_loaded: self.is_present(capability),
            });
        };
        let mut found: BTreeMap<String, Support> = BTreeMap::new();
        for id in self.graph().evidence_of(capability) {
            self.support_entry(&mut found, id, capability);
        }
        let prefix = DocType::Evidence.id_prefix();
        for claim in cap.body().claims() {
            for slug in claim.evidence() {
                let support =
                    self.support_entry(&mut found, &format!("{prefix}{slug}"), capability);
                if !support.claims.contains(&claim.line()) {
                    support.claims.push(claim.line());
                }
            }
        }
        Ok(found.into_values().collect())
    }
}
