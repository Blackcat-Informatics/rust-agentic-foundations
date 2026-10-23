// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Source-profile framing around native IRI, percent and base16 operations.

use purrdf_hash::hex;
use purrdf_iri::IriReadError;
use purrdf_lex::allocation::{Admission, Memory, StorageError};
use purrdf_lex::percent;

use super::{BuildError, invalid};

/// The source family named by an explicit lock source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LockedSourceKind {
    /// A Git repository plus its reference and full selected revision.
    Git,
    /// A registry index.
    Registry,
    /// A sparse registry index, distinct from a Git registry index.
    SparseRegistry,
    /// An explicitly named path source, distinct from an absent local source.
    Path,
    /// A local registry.
    LocalRegistry,
    /// A directory registry.
    Directory,
}

/// The requested Git reference, independent of its selected full revision.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LockedGitReference {
    /// The repository's default branch.
    DefaultBranch,
    /// A named branch, after native form-percent decoding.
    Branch(String),
    /// A named tag.
    Tag(String),
    /// A requested revision expression.
    Revision(String),
}

/// A complete source key whose equality never treats missing precision as a wildcard.
///
/// Repository IRIs retain their validated spelling. This value does not assert
/// correspondence to a supplied checkout or authenticate repository ownership.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LockedSource {
    kind: LockedSourceKind,
    repository: String,
    reference: Option<LockedGitReference>,
    revision: Option<String>,
}

impl LockedSource {
    pub(crate) fn retained<S: Admission + ?Sized>(
        &self,
        memory: &mut Memory<'_, S>,
    ) -> Result<Self, StorageError> {
        let reference = match &self.reference {
            None => None,
            Some(LockedGitReference::DefaultBranch) => Some(LockedGitReference::DefaultBranch),
            Some(LockedGitReference::Branch(text)) => {
                Some(LockedGitReference::Branch(memory.string(text)?))
            }
            Some(LockedGitReference::Tag(text)) => {
                Some(LockedGitReference::Tag(memory.string(text)?))
            }
            Some(LockedGitReference::Revision(text)) => {
                Some(LockedGitReference::Revision(memory.string(text)?))
            }
        };
        Ok(Self {
            kind: self.kind,
            repository: memory.string(&self.repository)?,
            reference,
            revision: self
                .revision
                .as_ref()
                .map(|text| memory.string(text))
                .transpose()?,
        })
    }
    /// The exact source family.
    #[must_use]
    pub const fn kind(&self) -> LockedSourceKind {
        self.kind
    }

    /// The validated repository IRI, without a Git selector or revision fragment.
    #[must_use]
    pub fn repository(&self) -> &str {
        &self.repository
    }

    /// The requested Git reference; absent for other source families.
    #[must_use]
    pub const fn reference(&self) -> Option<&LockedGitReference> {
        self.reference.as_ref()
    }

    /// The full lowercase Git revision, if present, never a wildcard.
    #[must_use]
    pub fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }

    pub(super) fn parse<S: Admission + ?Sized>(
        text: &str,
        memory: &mut Memory<'_, S>,
    ) -> Result<Self, BuildError> {
        let Some((family, body)) = text.split_once('+') else {
            return Err(invalid(
                memory,
                "source",
                format_args!("expected a source family followed by '+' and an absolute IRI"),
            ));
        };
        let kind = match family {
            "git" => LockedSourceKind::Git,
            "registry" => LockedSourceKind::Registry,
            "sparse" => LockedSourceKind::SparseRegistry,
            "path" => LockedSourceKind::Path,
            "local-registry" => LockedSourceKind::LocalRegistry,
            "directory" => LockedSourceKind::Directory,
            _ => {
                return Err(invalid(
                    memory,
                    "source",
                    format_args!("unsupported source family {family:?}"),
                ));
            }
        };
        let iri = purrdf_iri::parse_with_memory(body, memory).map_err(|error| match error {
            IriReadError::Storage(error) => BuildError::Storage(error),
            IriReadError::Lexical(error) => invalid(
                memory,
                "source",
                format_args!("invalid repository IRI: {error}"),
            ),
        })?;
        if !iri.has_scheme() {
            return Err(invalid(
                memory,
                "source",
                format_args!("repository IRI must be absolute"),
            ));
        }
        let (repository, reference, revision) = if kind == LockedSourceKind::Git {
            let suffix = iri.query().map_or(0, |text| text.len() + 1)
                + iri.fragment().map_or(0, |text| text.len() + 1);
            let repository = memory.string(&body[..body.len() - suffix])?;
            let reference = git_reference(iri.query(), memory)?;
            let revision = iri
                .fragment()
                .map(|text| full_revision(text, memory))
                .transpose()?;
            (repository, Some(reference), revision)
        } else {
            if iri.fragment().is_some() {
                return Err(invalid(
                    memory,
                    "source",
                    format_args!("a non-Git source cannot select a revision fragment"),
                ));
            }
            (memory.string(body)?, None, None)
        };
        memory.release_string(iri.into_text())?;
        Ok(Self {
            kind,
            repository,
            reference,
            revision,
        })
    }

    pub(super) fn release<S: Admission + ?Sized>(
        self,
        memory: &mut Memory<'_, S>,
    ) -> Result<(), BuildError> {
        memory.release_string(self.repository)?;
        if let Some(
            LockedGitReference::Branch(text)
            | LockedGitReference::Tag(text)
            | LockedGitReference::Revision(text),
        ) = self.reference
        {
            memory.release_string(text)?;
        }
        if let Some(revision) = self.revision {
            memory.release_string(revision)?;
        }
        Ok(())
    }
}

fn git_reference<S: Admission + ?Sized>(
    query: Option<&str>,
    memory: &mut Memory<'_, S>,
) -> Result<LockedGitReference, BuildError> {
    let Some(query) = query else {
        return Ok(LockedGitReference::DefaultBranch);
    };
    let Some((key, value)) = query.split_once('=') else {
        return Err(invalid(
            memory,
            "source",
            format_args!("Git selector must name branch, tag or rev"),
        ));
    };
    if query.contains('&') || !matches!(key, "branch" | "ref" | "tag" | "rev") {
        return Err(invalid(
            memory,
            "source",
            format_args!("exactly one supported Git selector is required"),
        ));
    }
    let decoded = percent::decode_form(value).map_err(|error| {
        invalid(
            memory,
            "source",
            format_args!("invalid Git selector: {error}"),
        )
    })?;
    if decoded.is_empty() {
        return Err(invalid(
            memory,
            "source",
            format_args!("Git selector cannot be empty"),
        ));
    }
    let value = memory.string(&decoded)?;
    Ok(match key {
        "branch" | "ref" => LockedGitReference::Branch(value),
        "tag" => LockedGitReference::Tag(value),
        "rev" => LockedGitReference::Revision(value),
        _ => unreachable!("the selector was checked before allocation"),
    })
}

fn full_revision<S: Admission + ?Sized>(
    text: &str,
    memory: &mut Memory<'_, S>,
) -> Result<String, BuildError> {
    if !matches!(text.len(), 40 | 64) {
        return Err(invalid(
            memory,
            "source",
            format_args!("Git precision must be a full 40- or 64-digit revision"),
        ));
    }
    let mut bytes = [0; 32];
    let length = hex::decode_to_slice(text, &mut bytes).map_err(|error| {
        invalid(
            memory,
            "source",
            format_args!("invalid full Git revision: {error}"),
        )
    })?;
    Ok(memory.format(&format_args!("{}", hex::Lower(&bytes[..length])))?)
}
