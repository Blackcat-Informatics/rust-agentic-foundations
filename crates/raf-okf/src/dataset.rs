// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The RAF authoring-cover policy over native general RDF storage.

mod read;
mod write;

use std::{error::Error, fmt, sync::Arc};

use purrdf_core::RdfDataset;

use crate::{ConstructError, Document, ParseError, Profile, Violation};

/// Stable asserted identities: document type name in lowercase, then encoded id.
pub const DOCUMENT_IRI_BASE: &str = "https://blackcatinformatics.ca/raf/okf/document/";
/// Maximum structural subjects of one authoring dataset, including its document.
pub const MAX_DATASET_NODES: usize = 131_072;
/// Maximum asserted triples of one authoring dataset.
pub const MAX_DATASET_TRIPLES: usize = 1_048_576;

const VOCAB: &str = "https://blackcatinformatics.ca/raf/okf/1#";
const TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";

/// A native carrier, closed authoring shape or RAF reconstruction refusal.
#[derive(Debug)]
#[non_exhaustive]
pub enum DatasetError {
    /// Navigation Index is a separate model, not a profile Document.
    NavigationIndex,
    /// The named document does not satisfy its existing RAF profile.
    Profile {
        /// The repository-relative document path.
        path: String,
        /// Every violation returned by the existing validator.
        violations: Vec<Violation>,
    },
    /// A malformed node or inconsistent representation.
    Shape {
        /// A bounded subject context, or the carrier when no root is known.
        subject: String,
        /// The rule and expected repair.
        reason: String,
    },
    /// A fixed node, triple, nesting or aggregate authoring-text limit.
    Limit(&'static str),
    /// Original native allocation, term or storage failure.
    Native(Box<dyn Error + Send + Sync>),
    /// The existing authoring constructor refused the decoded model.
    Construct(ConstructError),
    /// The existing RAF parser refused the reconstructed representation.
    Parse(ParseError),
    /// The shared checked document identity refused the id.
    Identity(crate::DocumentIdentityError),
}

impl fmt::Display for DatasetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NavigationIndex => {
                f.write_str("navigation Index is not a profile Document; use its separate model")
            }
            Self::Profile { path, violations } => write!(
                f,
                "{path}: RAF profile refused authoring dataset: {violations:?}"
            ),
            Self::Shape { subject, reason } => write!(f, "authoring dataset {subject}: {reason}"),
            Self::Limit(reason) => write!(f, "authoring dataset exceeds {reason}"),
            Self::Native(source) => write!(f, "native authoring dataset: {source}"),
            Self::Construct(source) => source.fmt(f),
            Self::Parse(source) => source.fmt(f),
            Self::Identity(source) => source.fmt(f),
        }
    }
}

impl Error for DatasetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Native(source) => Some(source.as_ref()),
            Self::Construct(source) => Some(source),
            Self::Parse(source) => Some(source),
            Self::Identity(source) => Some(source),
            _ => None,
        }
    }
}

impl DatasetError {
    fn native(source: impl Error + Send + Sync + 'static) -> Self {
        Self::Native(Box::new(source))
    }
    fn shape(subject: &str, reason: impl Into<String>) -> Self {
        Self::Shape {
            subject: subject.chars().take(160).collect(),
            reason: reason.into(),
        }
    }
}

impl Document {
    /// Project this validated authoring model into native RDF 1.2 storage.
    ///
    /// The stable asserted identity derives from the validated type/id. Values,
    /// styles, comments and list positions remain distinct structured nodes.
    ///
    /// # Errors
    /// Refuses navigation Index, profile violations, fixed resource bounds and
    /// native storage failures. No file or process is used.
    pub fn to_dataset(&self, path: &str) -> Result<Arc<RdfDataset>, DatasetError> {
        write::admit(self)?;
        validate(self, path)?;
        write::document(self)
    }

    /// Recover one complete authoring document from its closed native dataset.
    ///
    /// This uses the existing writer, parser and RAF profile to refuse any
    /// representation that would change the authoring model when rendered.
    ///
    /// # Errors
    /// Refuses unknown or unrelated nodes, invalid ownership or cardinalities,
    /// unsupported RDF layers, resource bounds and original parser/profile errors.
    pub fn from_dataset(dataset: &RdfDataset, path: &str) -> Result<Self, DatasetError> {
        if crate::is_capability_index(path) {
            return Err(DatasetError::NavigationIndex);
        }
        read::document(dataset, path)
    }
}

fn validate(document: &Document, path: &str) -> Result<(), DatasetError> {
    if crate::is_capability_index(path) {
        return Err(DatasetError::NavigationIndex);
    }
    Profile
        .validate(document, path)
        .map(|_| ())
        .map_err(|violations| DatasetError::Profile {
            path: path.into(),
            violations,
        })
}

fn identity(document: &Document) -> Result<String, DatasetError> {
    let kind = document
        .doc_type()
        .ok_or_else(|| DatasetError::shape("document", "missing validated type"))?;
    let id = document
        .frontmatter()
        .map()
        .get("id")
        .and_then(|entry| entry.value().as_scalar())
        .ok_or_else(|| DatasetError::shape("document", "missing validated id"))?
        .text();
    kind.document_iri(id).map_err(DatasetError::Identity)
}

fn descendants(root: &str) -> String {
    format!(
        "https://blackcatinformatics.ca/raf/okf/node/{}/",
        purrdf_hash::hex::Lower(purrdf_hash::blake3::hash(root.as_bytes()).as_bytes())
    )
}

fn property(name: &str) -> String {
    format!("{VOCAB}{name}")
}

#[derive(Default)]
struct Budget {
    nodes: usize,
    triples: usize,
    text: usize,
}

impl Budget {
    fn node(&mut self) -> Result<(), DatasetError> {
        self.nodes += 1;
        if self.nodes > MAX_DATASET_NODES {
            return Err(DatasetError::Limit("131072 structural nodes"));
        }
        Ok(())
    }
    fn triple(&mut self) -> Result<(), DatasetError> {
        self.triples += 1;
        if self.triples > MAX_DATASET_TRIPLES {
            return Err(DatasetError::Limit("1048576 asserted triples"));
        }
        Ok(())
    }
    fn text(&mut self, text: &str) -> Result<(), DatasetError> {
        self.text = self
            .text
            .checked_add(text.len())
            .ok_or(DatasetError::Limit("1 MiB aggregate authoring text"))?;
        if self.text > crate::MAX_TEXT_BYTES {
            return Err(DatasetError::Limit("1 MiB aggregate authoring text"));
        }
        Ok(())
    }
}
