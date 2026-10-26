// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Checked stable identities shared by authoring and semantic projections.

use std::{error::Error, fmt};

use crate::{DOCUMENT_IRI_BASE, DocType, MAX_TEXT_BYTES, Rule, profile};

/// Why a document id cannot name a stable document IRI.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DocumentIdentityError {
    /// The id alone exceeds the authoring document's byte bound.
    TooLarge,
    /// The existing prefix, slug or singleton rule was broken.
    Invalid(Rule),
}

impl fmt::Display for DocumentIdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => write!(f, "document id exceeds {MAX_TEXT_BYTES} bytes"),
            Self::Invalid(rule) => write!(f, "document identity refused: {rule}"),
        }
    }
}

impl Error for DocumentIdentityError {}

impl DocType {
    /// The stable authoring document IRI for a complete prefixed `id`.
    ///
    /// Uses the same prefix, kebab slug and singleton rules as the profile.
    /// Filename and document-content validation remain the profile's job.
    /// Navigation Index is not a profile document type.
    ///
    /// # Errors
    /// Refuses oversized ids and the original profile id-rule violations.
    pub fn document_iri(self, id: &str) -> Result<String, DocumentIdentityError> {
        if id.len() > MAX_TEXT_BYTES {
            return Err(DocumentIdentityError::TooLarge);
        }
        profile::id_slug(self, id).map_err(DocumentIdentityError::Invalid)?;
        Ok(format!(
            "{DOCUMENT_IRI_BASE}{}/{}",
            self.name().to_ascii_lowercase(),
            purrdf_lex::percent::encode(id, purrdf_lex::percent::PATH)
        ))
    }
}
