// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Authenticated reader acceptance claims for exact committed payloads.

use std::{collections::BTreeMap, error, fmt, path::Path};

use capnp::message::{Reader, ReaderOptions, SegmentArray};
use purrdf_core::ContentDigest;
use raf_rdf::Graph;
use raf_source::{LoadError, LoadOptions, PinnedGitTree, RelPath, Tree, TreeError};

use crate::{ReportError, report::shape};

use super::{
    CaptureError, CaptureFiles, CaptureKey, TypeId,
    capture::{canonical_path, digest, fields, owned_node, parse_digest, signed_graph},
};

pub(super) const MAX_PAYLOADS: usize = 128;
const MAX_PAYLOAD_BYTES: u64 = 2 * 1024 * 1024;
const MAX_ENTRY_POINT_BYTES: usize = 1024;

/// The externally selected subject of the acceptance claim.
#[derive(Clone, Copy, Debug)]
pub struct Expected<'a> {
    /// Full commit ID holding the original payloads.
    pub base: &'a str,
    /// Full commit ID of the reader revision.
    pub current: &'a str,
    /// Nominal root type identified by the signer.
    pub root: TypeId,
    /// Exact reader entry point identified by the caller.
    pub entry_point: &'a str,
    /// Expected SHA-256 of the reader executable artifact.
    pub artifact: ContentDigest,
    /// Nonempty, exact set of repository-relative payload paths in the base.
    pub payloads: &'a [RelPath],
}

/// The trusted key and explicitly pinned evidence input.
#[derive(Clone, Copy, Debug)]
pub struct Evidence<'a> {
    /// Full commit ID holding the statement and its detached signature.
    pub commit: &'a str,
    /// Statement and signature paths in the evidence commit.
    pub files: CaptureFiles<'a>,
    /// Verification authority selected by the caller.
    pub key: &'a CaptureKey,
}

/// A refusal to establish an authenticated acceptance claim.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A caller-selected input was invalid.
    Selection(&'static str),
    /// A required commit could not be admitted.
    Revision {
        /// Which of the three revisions was refused.
        role: &'static str,
        /// Reason from the native Git tree.
        source: LoadError,
    },
    /// The evidence failed authentication, profile validation, or subject matching.
    Evidence(CaptureError),
    /// A committed payload did not agree with the evidence.
    Payload {
        /// Path of the payload in the pinned base.
        path: RelPath,
        /// Reason its identity or framing could not be established.
        source: PayloadError,
    },
}

/// Failure to admit a selected committed payload.
#[derive(Debug)]
#[non_exhaustive]
pub enum PayloadError {
    /// Native pinned blob read failure.
    Read(TreeError),
    /// The committed bytes differ from the signed digest.
    Digest,
    /// The bytes do not form a canonical raw Cap'n Proto segment.
    Framing,
    /// The bounded Cap'n Proto reader refused the message.
    Wire(capnp::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Selection(reason) => f.write_str(reason),
            Self::Revision { role, source } => write!(f, "{role} revision: {source}"),
            Self::Evidence(source) => write!(f, "reader evidence: {source}"),
            Self::Payload { path, source } => write!(f, "base payload {path}: {source}"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Selection(_) => None,
            Self::Revision { source, .. } => Some(source),
            Self::Evidence(source) => Some(source),
            Self::Payload { source, .. } => Some(source),
        }
    }
}

impl fmt::Display for PayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(source) => source.fmt(f),
            Self::Digest => f.write_str("SHA-256 differs from the signed payload digest"),
            Self::Framing => f.write_str("expected a canonical raw single Cap'n Proto segment"),
            Self::Wire(source) => write!(f, "cannot read canonical Cap'n Proto payload: {source}"),
        }
    }
}

impl error::Error for PayloadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Read(source) => Some(source),
            Self::Wire(source) => Some(source),
            Self::Digest | Self::Framing => None,
        }
    }
}

/// Parse a canonical nonempty evidence or payload path relative to the repository.
///
/// # Errors
/// Refuses noncanonical, escaping, or overlong paths.
pub fn path(text: &str) -> Result<RelPath, CaptureError> {
    canonical_path(text, false)
}

/// Parse a canonical lowercase SHA-256 of an artifact.
///
/// # Errors
/// Refuses invalid or noncanonical SHA-256 text.
pub fn artifact_digest(text: &str) -> Result<ContentDigest, Error> {
    parse_digest(text)
        .map(ContentDigest::from_raw)
        .ok_or(Error::Selection(
            "supply the artifact SHA-256 as 64 lowercase hexadecimal digits",
        ))
}

/// Verify an exact caller-selected set and return its paths in repository order.
///
/// Census corroborates the commits, payload identities and raw single-segment
/// canonicality. The caller's trusted signer attests the type identity,
/// executable artifact, entry point and application-level acceptance.
///
/// # Errors
/// Refuses invalid selections, unadmitted revisions, unauthenticated or
/// mismatched claims, and unreadable, changed, or noncanonical payloads.
pub fn verify(
    repository: impl AsRef<Path>,
    expected: Expected<'_>,
    evidence: Evidence<'_>,
) -> Result<Vec<RelPath>, Error> {
    check_evidence_inputs(expected.base, expected.current, evidence)?;
    let mut selected = selection(&expected)?;
    selected.sort_unstable();
    if selected.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(Error::Selection("select each payload path once"));
    }
    let repository = repository.as_ref();
    pinned(repository, expected.current, "current")?;
    let reported = {
        let graph = authenticate(repository, evidence)?;
        statement(&graph, &expected).map_err(Error::Evidence)?
    };
    match_specimens(&selected, &reported)?;
    let base = pinned(repository, expected.base, "base")?;
    verify_specimens(&base, reported)
}

pub(super) fn match_specimens(
    selected: &[&RelPath],
    reported: &BTreeMap<RelPath, [u8; 32]>,
) -> Result<(), Error> {
    if selected.iter().copied().ne(reported.keys()) {
        return Err(Error::Evidence(CaptureError::Mismatch(
            "signed payload paths differ from the exact caller selection".into(),
        )));
    }
    Ok(())
}

pub(super) fn verify_specimens(
    base: &dyn Tree,
    reported: BTreeMap<RelPath, [u8; 32]>,
) -> Result<Vec<RelPath>, Error> {
    for (path, digest) in &reported {
        verify_payload(base, path, digest).map_err(|source| Error::Payload {
            path: path.clone(),
            source,
        })?;
    }
    Ok(reported.into_keys().collect())
}

pub(super) fn check_evidence_inputs(
    base: &str,
    current: &str,
    evidence: Evidence<'_>,
) -> Result<(), Error> {
    check_evidence_selection(
        base,
        current,
        evidence,
        "select a separate commit for the reader evidence",
    )
}

pub(super) fn check_evidence_selection(
    base: &str,
    current: &str,
    evidence: Evidence<'_>,
    separate_commit: &'static str,
) -> Result<(), Error> {
    if evidence.commit == base || evidence.commit == current {
        return Err(Error::Selection(separate_commit));
    }
    for file in [evidence.files.statement, evidence.files.signature] {
        path(file.as_str()).map_err(Error::Evidence)?;
    }
    Ok(())
}

pub(super) fn authenticate(repository: &Path, evidence: Evidence<'_>) -> Result<Graph, Error> {
    let signed = pinned(repository, evidence.commit, "evidence")?;
    signed_graph(&signed, evidence.files, evidence.key).map_err(Error::Evidence)
}

pub(super) fn pinned(
    repository: &Path,
    commit: &str,
    role: &'static str,
) -> Result<PinnedGitTree<'static>, Error> {
    PinnedGitTree::open(repository, commit, &LoadOptions::default())
        .map_err(|source| Error::Revision { role, source })
}

fn selection<'a>(expected: &Expected<'a>) -> Result<Vec<&'a RelPath>, Error> {
    if expected.payloads.is_empty() || expected.payloads.len() > MAX_PAYLOADS {
        return Err(Error::Selection(
            "select one to 128 exact base payload paths",
        ));
    }
    let entry = expected.entry_point;
    if entry.is_empty()
        || entry.len() > MAX_ENTRY_POINT_BYTES
        || entry.chars().any(char::is_control)
    {
        return Err(Error::Selection(
            "supply a nonempty reader entry point of at most 1024 bytes without control characters",
        ));
    }
    for file in expected.payloads {
        path(file.as_str()).map_err(Error::Evidence)?;
    }
    Ok(expected.payloads.iter().collect())
}

fn statement(
    graph: &Graph,
    expected: &Expected<'_>,
) -> Result<BTreeMap<RelPath, [u8; 32]>, CaptureError> {
    let root = graph
        .get("")
        .ok_or_else(|| ReportError::Missing("reader evidence root".into()))?;
    fields(
        &root,
        "CapnpReaderEvidenceV1",
        &[
            "baseCommit",
            "currentCommit",
            "rootType",
            "readerEntry",
            "readerSha256",
            "accepted",
            "payload",
        ],
    )?;
    match_reader(&root, expected)?;
    if shape::boolean_value(shape::one(&root, "accepted")?) != Some(true) {
        return Err(CaptureError::Mismatch(
            "signed evidence must affirm acceptance of every payload".into(),
        ));
    }
    payloads(graph, &root)
}

pub(super) fn match_reader(
    root: &raf_rdf::Node<'_>,
    expected: &Expected<'_>,
) -> Result<(), CaptureError> {
    for (key, value) in [
        ("baseCommit", expected.base),
        ("currentCommit", expected.current),
        ("rootType", &expected.root.to_string()),
    ] {
        require_text(root, key, value)?;
    }
    match_artifact(
        root,
        ("readerEntry", expected.entry_point),
        ("readerSha256", expected.artifact),
        "reader",
    )
}

pub(super) fn match_artifact(
    root: &raf_rdf::Node<'_>,
    (entry_key, entry): (&str, &str),
    (digest_key, expected): (&str, ContentDigest),
    role: &str,
) -> Result<(), CaptureError> {
    require_text(root, entry_key, entry)?;
    if &digest(root, digest_key)? != expected.as_bytes() {
        return Err(CaptureError::Mismatch(format!(
            "signed {role} SHA-256 differs from the caller selection"
        )));
    }
    Ok(())
}

pub(super) fn require_text(
    root: &raf_rdf::Node<'_>,
    key: &str,
    value: &str,
) -> Result<(), CaptureError> {
    if shape::string(root, key)? != value {
        return Err(CaptureError::Mismatch(format!(
            "signed {key} differs from the caller selection"
        )));
    }
    Ok(())
}

pub(super) fn payloads(
    graph: &Graph,
    root: &raf_rdf::Node<'_>,
) -> Result<BTreeMap<RelPath, [u8; 32]>, CaptureError> {
    let mut payloads = BTreeMap::new();
    for reference in root.values(&shape::property("payload")) {
        if payloads.len() == MAX_PAYLOADS {
            return Err(CaptureError::Mismatch(
                "reader evidence admits at most 128 payloads".into(),
            ));
        }
        let row = owned_node(graph, reference, "payload")?;
        fields(&row, "CapnpAcceptedPayload", &["file", "sha256"])?;
        let file = path(&shape::string(&row, "file")?)?;
        if payloads.insert(file, digest(&row, "sha256")?).is_some() {
            return Err(CaptureError::Mismatch(
                "reader evidence repeats a payload path".into(),
            ));
        }
    }
    if graph.nodes().count() != 1 + payloads.len() {
        return Err(CaptureError::Mismatch(
            "reader evidence may contain only its referenced subjects".into(),
        ));
    }
    Ok(payloads)
}

pub(super) fn verify_payload(
    tree: &dyn Tree,
    path: &RelPath,
    digest: &[u8; 32],
) -> Result<(), PayloadError> {
    let bytes = tree
        .read_bytes(path, MAX_PAYLOAD_BYTES)
        .map_err(PayloadError::Read)?;
    if ContentDigest::of(&bytes).as_bytes() != digest {
        return Err(PayloadError::Digest);
    }
    if bytes.is_empty() || !bytes.len().is_multiple_of(8) {
        return Err(PayloadError::Framing);
    }
    let segments = [bytes.as_slice()];
    let mut options = ReaderOptions::new();
    options.traversal_limit_in_words(Some(1024 * 1024));
    options.nesting_limit(64);
    let message = Reader::new(SegmentArray::new(&segments), options);
    if !message.is_canonical().map_err(PayloadError::Wire)? {
        return Err(PayloadError::Framing);
    }
    Ok(())
}
