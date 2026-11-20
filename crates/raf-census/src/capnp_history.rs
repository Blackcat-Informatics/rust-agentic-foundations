// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Strict layout history for explicitly selected types in committed Cap'n Proto descriptors.

mod capture;
mod compare;
mod descriptor;
mod occupancy;
pub mod persistence;
pub mod reader_evidence;
mod shape;
#[cfg(test)]
mod tests;

use std::{collections::BTreeSet, fmt, path::Path, str::FromStr};

use raf_source::{LoadError, LoadOptions, PinnedGitTree, RelPath, Tree, TreeError};

pub use capture::{CaptureError, CaptureFiles, CaptureKey, CaptureVerification, InvalidCaptureKey};
pub use compare::{Change, ChangeKind};

/// Schema node ID in the compiler's 64-bit identity space.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TypeId(u64);

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

impl FromStr for TypeId {
    type Err = InvalidTypeId;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let digits = text.strip_prefix("0x").ok_or(InvalidTypeId)?;
        let mut bytes = [0; 8];
        purrdf_hash::hex::decode_canonical_to_slice(digits, &mut bytes)
            .map_err(|_| InvalidTypeId)?;
        let id = u64::from_be_bytes(bytes);
        if id & (1 << 63) == 0 {
            return Err(InvalidTypeId);
        }
        Ok(Self(id))
    }
}

/// The argument was not a canonical Cap'n Proto schema ID.
#[derive(Clone, Copy, Debug)]
pub struct InvalidTypeId;

impl fmt::Display for InvalidTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("use 0x followed by 16 lowercase hexadecimal digits with the high bit set")
    }
}

impl std::error::Error for InvalidTypeId {}

/// One committed descriptor location in a local Git object store.
#[derive(Clone, Copy, Debug)]
pub struct Snapshot<'a> {
    /// Full lowercase commit ID.
    pub commit: &'a str,
    /// Repository-relative path to an unpacked compiler descriptor.
    pub descriptor: &'a RelPath,
}

/// Selected schema roots and optional policies for one comparison.
#[derive(Clone, Copy, Debug)]
pub struct HistoryOptions<'a> {
    /// Struct or enum IDs whose data references define the comparison closure.
    pub roots: &'a [TypeId],
    /// Void field annotation identifying required pointer fields, if selected.
    pub required_annotation: Option<TypeId>,
    /// Authenticate and corroborate both committed compiler captures, if selected.
    pub capture: Option<CaptureVerification<'a>>,
}

/// An input that prevented a complete strict history verdict.
#[derive(Debug)]
pub enum Error {
    /// No root types were declared, or a root was selected more than once.
    Roots,
    /// A complete verdict would exceed the retained finding allowance.
    FindingLimit,
    /// The explicit Git commit could not be admitted.
    Revision {
        /// Supplied object ID.
        commit: String,
        /// Native Git admission error.
        source: LoadError,
    },
    /// The pinned descriptor blob could not be read.
    Read {
        /// Supplied object ID.
        commit: String,
        /// The blob read error, including its path.
        source: TreeError,
    },
    /// A capture or one of its recorded blobs could not be admitted.
    Capture {
        /// Supplied object ID.
        commit: String,
        /// Repository-relative capture statement.
        path: RelPath,
        /// Authentication or artifact failure.
        source: CaptureError,
    },
    /// The descriptor could not establish its selected layout.
    Descriptor {
        /// Supplied object ID.
        commit: String,
        /// Repository-relative descriptor path.
        path: RelPath,
        /// Reason the selected layout could not be established.
        source: DescriptorError,
    },
}

/// Failure decoding or validating a selected descriptor layout.
#[derive(Debug)]
pub enum DescriptorError {
    /// Cap'n Proto refused framing, pointers, traversal, or nesting.
    Wire(capnp::Error),
    /// The descriptor uses an unknown schema discriminant.
    Tag(capnp::NotInSchema),
    /// The selected closure contains inconsistent or unsupported information.
    Layout(String),
    /// A descriptor does not agree with its signed compiler capture.
    Capture(CaptureError),
}

impl From<CaptureError> for DescriptorError {
    fn from(source: CaptureError) -> Self {
        Self::Capture(source)
    }
}

impl From<capnp::Error> for DescriptorError {
    fn from(source: capnp::Error) -> Self {
        Self::Wire(source)
    }
}

impl From<capnp::NotInSchema> for DescriptorError {
    fn from(source: capnp::NotInSchema) -> Self {
        Self::Tag(source)
    }
}

impl fmt::Display for DescriptorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wire(source) => source.fmt(f),
            Self::Tag(source) => source.fmt(f),
            Self::Layout(message) => f.write_str(message),
            Self::Capture(source) => source.fmt(f),
        }
    }
}

impl std::error::Error for DescriptorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Wire(source) => Some(source),
            Self::Tag(source) => Some(source),
            Self::Layout(_) => None,
            Self::Capture(source) => Some(source),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Roots => {
                f.write_str("declare at least one Cap'n Proto root type ID without repeats")
            }
            Self::FindingLimit => f.write_str(
                "Cap'n Proto history findings exceed the two MiB allowance; compare fewer selected roots",
            ),
            Self::Revision { commit, source } => {
                write!(f, "Cap'n Proto revision {commit}: {source}")
            }
            Self::Read { commit, source } => {
                write!(f, "Cap'n Proto read at {commit}: {source}")
            }
            Self::Capture { commit, path, source } => write!(
                f,
                "Cap'n Proto capture {path} at {commit}: {source}; check the trusted key and committed capture inputs"
            ),
            Self::Descriptor {
                commit,
                path,
                source,
            } => write!(
                f,
                "Cap'n Proto descriptor {path} at {commit}: {source}; regenerate and commit the complete descriptor"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Roots | Self::FindingLimit => None,
            Self::Revision { source, .. } => Some(source),
            Self::Read { source, .. } => Some(source),
            Self::Capture { source, .. } => Some(source),
            Self::Descriptor { source, .. } => Some(source),
        }
    }
}

/// Compare strict schema history across two explicit commits in one repository.
///
/// Both descriptors are normative compiled inputs. This check reads no working
/// tree files and runs no compiler, generator, consumer program, or migration.
/// A change describes a history violation, including renames that the wire
/// protocol permits.
/// The optional marker must name a Void annotation targeting fields in both
/// snapshots. Optional capture verification admits both compiled inputs
/// against the caller's trusted key before projecting the selected layout.
///
/// # Errors
///
/// Refuses missing or repeated roots, unavailable commits or blobs, and
/// malformed, incomplete, or unsupported layout or capture inputs.
pub fn check(
    repository: impl AsRef<Path>,
    base: Snapshot<'_>,
    current: Snapshot<'_>,
    options: HistoryOptions<'_>,
) -> Result<Vec<Change>, Error> {
    let selected: BTreeSet<_> = options.roots.iter().copied().collect();
    if selected.is_empty() || selected.len() != options.roots.len() {
        return Err(Error::Roots);
    }
    let base = load(
        repository.as_ref(),
        base,
        &selected,
        options.required_annotation,
        descriptor::Roots::Require,
        options.capture.map(|capture| (capture.key, capture.base)),
    )?;
    let current = load(
        repository.as_ref(),
        current,
        &selected,
        options.required_annotation,
        descriptor::Roots::AllowMissing,
        options
            .capture
            .map(|capture| (capture.key, capture.current)),
    )?;
    compare::changes(&base, &current)
}

fn load(
    repository: &Path,
    snapshot: Snapshot<'_>,
    roots: &BTreeSet<TypeId>,
    required_annotation: Option<TypeId>,
    mode: descriptor::Roots<'_>,
    verification: Option<(&CaptureKey, CaptureFiles<'_>)>,
) -> Result<descriptor::Schema, Error> {
    let tree = open_tree(repository, snapshot.commit)?;
    load_descriptor(
        &tree,
        snapshot,
        roots,
        required_annotation,
        mode,
        verification,
    )
}

fn open_tree(repository: &Path, commit: &str) -> Result<PinnedGitTree<'static>, Error> {
    PinnedGitTree::open(repository, commit, &LoadOptions::default()).map_err(|source| {
        Error::Revision {
            commit: commit.into(),
            source,
        }
    })
}

fn load_descriptor(
    tree: &dyn Tree,
    snapshot: Snapshot<'_>,
    roots: &BTreeSet<TypeId>,
    required_annotation: Option<TypeId>,
    mode: descriptor::Roots<'_>,
    verification: Option<(&CaptureKey, CaptureFiles<'_>)>,
) -> Result<descriptor::Schema, Error> {
    decode_descriptor(
        tree,
        snapshot,
        roots,
        required_annotation,
        mode,
        verification,
        |_| (),
    )
    .map(|(schema, ())| schema)
}

fn decode_descriptor<T>(
    tree: &dyn Tree,
    snapshot: Snapshot<'_>,
    roots: &BTreeSet<TypeId>,
    required_annotation: Option<TypeId>,
    mode: descriptor::Roots<'_>,
    verification: Option<(&CaptureKey, CaptureFiles<'_>)>,
    identify: impl FnOnce(&[u8]) -> T,
) -> Result<(descriptor::Schema, T), Error> {
    let bytes = tree
        .read_bytes(snapshot.descriptor, descriptor::MAX_BYTES)
        .map_err(|source| Error::Read {
            commit: snapshot.commit.into(),
            source,
        })?;
    let capture = verification
        .map(|(key, files)| {
            capture::load(tree, files, key, (snapshot.descriptor, &bytes)).map_err(|source| {
                Error::Capture {
                    commit: snapshot.commit.into(),
                    path: files.statement.clone(),
                    source,
                }
            })
        })
        .transpose()?;
    let schema = descriptor::read(
        &bytes,
        roots,
        required_annotation,
        mode,
        capture.as_ref().map(|capture| (capture, tree)),
    )
    .map_err(|source| match (source, verification) {
        (DescriptorError::Capture(source), Some((_, files))) => Error::Capture {
            commit: snapshot.commit.into(),
            path: files.statement.clone(),
            source,
        },
        (source, _) => Error::Descriptor {
            commit: snapshot.commit.into(),
            path: snapshot.descriptor.clone(),
            source,
        },
    })?;
    Ok((schema, identify(&bytes)))
}
