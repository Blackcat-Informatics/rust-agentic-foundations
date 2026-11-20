// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Admission of signed compiler captures and their committed schema closure.

mod source;

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    str::{FromStr, Utf8Error},
};

use capnp::schema_capnp::{code_generator_request, node};
use purrdf_core::ContentDigest;
use purrdf_ed25519::{
    PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH, Signature, SignatureError, VerifyingKey,
};
use raf_rdf::{Format, Graph, Node};
use raf_source::{RelPath, Tree, TreeError};

use crate::{ReportError, report::shape};

use super::{DescriptorError, TypeId};

const MAX_CAPTURE_BYTES: u64 = 128 * 1024;
const MAX_SCHEMA_BYTES: u64 = 2 * 1024 * 1024;
const MAX_PATH_BYTES: usize = 4096;
const MAX_LITERAL_IMPORTS: usize = 8192;

/// Public key supplied by the caller to verify a compiler capture.
#[derive(Clone, Debug)]
pub struct CaptureKey(VerifyingKey);

/// The supplied text does not encode a supported Ed25519 verification key.
#[derive(Clone, Copy, Debug)]
pub struct InvalidCaptureKey;

impl fmt::Display for InvalidCaptureKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("supply an Ed25519 public key as 64 lowercase hexadecimal digits")
    }
}

impl std::error::Error for InvalidCaptureKey {}

impl FromStr for CaptureKey {
    type Err = InvalidCaptureKey;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0; PUBLIC_KEY_LENGTH];
        purrdf_hash::hex::decode_canonical_to_slice(text, &mut bytes)
            .map_err(|_| InvalidCaptureKey)?;
        VerifyingKey::from_bytes(&bytes)
            .map(Self)
            .map_err(|_| InvalidCaptureKey)
    }
}

/// Committed paths of one canonical capture statement and its detached signature.
#[derive(Clone, Copy, Debug)]
pub struct CaptureFiles<'a> {
    /// Canonical RDF 1.2 N-Triples statement.
    pub statement: &'a RelPath,
    /// Exact 64-byte Ed25519 signature over the statement bytes.
    pub signature: &'a RelPath,
}

/// Verify both snapshots using the same caller-selected public key.
#[derive(Clone, Copy, Debug)]
pub struct CaptureVerification<'a> {
    /// Key selected outside either inspected commit.
    pub key: &'a CaptureKey,
    /// Capture files in the base commit.
    pub base: CaptureFiles<'a>,
    /// Capture files in the current commit.
    pub current: CaptureFiles<'a>,
}

/// Failure to authenticate or corroborate a committed compiler capture.
#[derive(Debug)]
#[non_exhaustive]
pub enum CaptureError {
    /// A committed statement, signature, or source blob was not readable.
    Read(TreeError),
    /// Capture text, schema source, or a name in the descriptor was not UTF-8.
    Encoding(Utf8Error),
    /// The detached signature failed verification with the caller's public key.
    Signature(SignatureError),
    /// The statement failed the canonical RDF or capture profile.
    Profile(ReportError),
    /// A capture did not agree with its committed artifacts or compiler request.
    Mismatch(String),
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(source) => source.fmt(f),
            Self::Encoding(source) => source.fmt(f),
            Self::Signature(source) => write!(f, "capture signature verification failed: {source}"),
            Self::Profile(source) => source.fmt(f),
            Self::Mismatch(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for CaptureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(source) => Some(source),
            Self::Encoding(source) => Some(source),
            Self::Signature(source) => Some(source),
            Self::Profile(source) => Some(source),
            Self::Mismatch(_) => None,
        }
    }
}

impl From<ReportError> for CaptureError {
    fn from(source: ReportError) -> Self {
        Self::Profile(source)
    }
}

type Digest = [u8; 32];

pub(super) struct Capture {
    root: RelPath,
    version: String,
    sources: BTreeMap<RelPath, Digest>,
}

pub(super) fn load(
    tree: &dyn Tree,
    files: CaptureFiles<'_>,
    key: &CaptureKey,
    descriptor: (&RelPath, &[u8]),
) -> Result<Capture, CaptureError> {
    let bytes = tree
        .read_bytes(files.statement, MAX_CAPTURE_BYTES)
        .map_err(CaptureError::Read)?;
    let signature: [u8; SIGNATURE_LENGTH] = tree
        .read_bytes(files.signature, SIGNATURE_LENGTH as u64)
        .map_err(CaptureError::Read)?
        .try_into()
        .map_err(|_| mismatch("capture signature must contain exactly 64 bytes"))?;
    key.0
        .verify_strict(&bytes, &Signature::from_bytes(&signature))
        .map_err(CaptureError::Signature)?;
    let text = str::from_utf8(&bytes).map_err(CaptureError::Encoding)?;
    let graph = Graph::from_text(text, Format::NTriples).map_err(ReportError::Ground)?;
    Capture::from_graph(&graph, descriptor)
}

impl Capture {
    fn from_graph(graph: &Graph, descriptor: (&RelPath, &[u8])) -> Result<Self, CaptureError> {
        let root = graph
            .get("")
            .ok_or_else(|| ReportError::Missing("capture root".into()))?;
        fields(
            &root,
            "CapnpCaptureV1",
            &[
                "compilerSha256",
                "compilerVersion",
                "descriptorFile",
                "descriptorSha256",
                "schemaRoot",
                "schemaSource",
            ],
        )?;
        let descriptor_path = canonical_path(&shape::string(&root, "descriptorFile")?, false)?;
        let descriptor_digest = digest(&root, "descriptorSha256")?;
        if &descriptor_path != descriptor.0
            || &descriptor_digest != ContentDigest::of(descriptor.1).as_bytes()
        {
            return Err(mismatch(
                "capture descriptor path or digest differs from the pinned request",
            ));
        }

        // The signer attests the exact compiler artifact; the request can
        // corroborate its version but does not contain its binary identity.
        let _compiler = digest(&root, "compilerSha256")?;
        let version = shape::string(&root, "compilerVersion")?;
        let schema_root = canonical_path(&shape::string(&root, "schemaRoot")?, true)?;
        let mut sources = BTreeMap::new();
        for reference in root.values(&shape::property("schemaSource")) {
            let id = reference
                .iri()
                .filter(|id| id.starts_with('#'))
                .ok_or_else(|| {
                    ReportError::Value("capture schemaSource must name an owned node".into())
                })?;
            let source = graph
                .get(id)
                .ok_or_else(|| ReportError::Dangling(id.into()))?;
            fields(&source, "CapnpSchemaSource", &["file", "sha256"])?;
            let path = canonical_path(&shape::string(&source, "file")?, false)?;
            if sources
                .insert(path.clone(), digest(&source, "sha256")?)
                .is_some()
            {
                return Err(mismatch(format!("capture repeats schema path {path}")));
            }
        }
        if sources.is_empty() || graph.nodes().count() != sources.len() + 1 {
            return Err(mismatch(
                "capture needs source rows and may contain only its referenced subjects",
            ));
        }
        Ok(Self {
            root: schema_root,
            version,
            sources,
        })
    }

    pub(super) fn verify_request(
        &self,
        tree: &dyn Tree,
        request: code_generator_request::Reader<'_>,
        nodes: &BTreeMap<TypeId, node::Reader<'_>>,
    ) -> Result<(), DescriptorError> {
        let version = request.get_capnp_version()?;
        let actual = format!(
            "{}.{}.{}",
            version.get_major(),
            version.get_minor(),
            version.get_micro()
        );
        if !request.has_capnp_version() || self.version != actual {
            return Err(
                mismatch("capture compiler version differs from the pinned request").into(),
            );
        }

        let mut files = BTreeSet::new();
        for (&id, node) in nodes {
            if matches!(node.which()?, node::File(())) {
                files.insert(id);
            }
        }
        let mut paths = BTreeSet::new();
        let mut requested = BTreeMap::new();
        for row in request.get_requested_files()? {
            let id = TypeId(row.get_id());
            let name = row.get_filename()?;
            let path = canonical_path(name.to_str().map_err(CaptureError::Encoding)?, false)?;
            if !self.sources.contains_key(&path)
                || !paths.insert(path.clone())
                || requested.insert(id, (path.clone(), row)).is_some()
            {
                return Err(mismatch(format!(
                    "duplicate or unrecorded requested schema {path} ({id})"
                ))
                .into());
            }
            if let Some(node) = nodes.get(&id)
                && files.contains(&id)
                && node
                    .get_display_name()?
                    .to_str()
                    .map_err(CaptureError::Encoding)?
                    != path.as_str()
            {
                return Err(mismatch(format!(
                    "file node name differs from requested schema {path} ({id})"
                ))
                .into());
            }
        }
        if paths.len() != self.sources.len() || requested.keys().ne(files.iter()) {
            return Err(mismatch(
                "capture sources, requested schema rows, and file nodes must agree",
            )
            .into());
        }
        let mut remaining = MAX_LITERAL_IMPORTS;
        for (path, row) in requested.values() {
            for import in row.get_imports()? {
                let id = TypeId(import.get_id());
                let name = import.get_name()?;
                let name = name.to_str().map_err(CaptureError::Encoding)?;
                let expected = requested.get(&id).map(|(target, _)| target);
                if expected != Some(&self.resolve_import(tree, path, name)?) {
                    return Err(unresolved_import(path, name).into());
                }
            }
            self.verify_source(tree, path, &mut remaining)?;
        }
        Ok(())
    }

    fn verify_source(
        &self,
        tree: &dyn Tree,
        from: &RelPath,
        remaining: &mut usize,
    ) -> Result<(), CaptureError> {
        let path = join(&self.root, from.as_str())?;
        let bytes = tree
            .read_bytes(&path, MAX_SCHEMA_BYTES)
            .map_err(CaptureError::Read)?;
        if self.sources.get(from) != Some(ContentDigest::of(&bytes).as_bytes()) {
            return Err(mismatch(format!(
                "capture digest differs for schema {path}"
            )));
        }
        source::imports(&bytes, &path, |name| {
            *remaining = remaining.checked_sub(1).ok_or_else(|| {
                mismatch("capture exceeds its 8192 source import allowance; reduce the input set")
            })?;
            if !self
                .sources
                .contains_key(&self.resolve_import(tree, from, name)?)
            {
                return Err(unresolved_import(from, name));
            }
            Ok(())
        })
    }

    fn resolve_import(
        &self,
        tree: &dyn Tree,
        from: &RelPath,
        name: &str,
    ) -> Result<RelPath, CaptureError> {
        let rooted = name.strip_prefix('/');
        if rooted.unwrap_or(name).len() > MAX_PATH_BYTES {
            return Err(mismatch(format!(
                "schema import from {from} exceeds its length bound"
            )));
        }
        let invalid = || unresolved_import(from, name);
        let raw = match rooted {
            Some(rooted) => join(&RelPath::root(), rooted),
            None => join(&from.parent().unwrap_or_else(RelPath::root), name),
        }
        .map_err(|_| invalid())?;
        let mut segments = Vec::new();
        for segment in raw.segments() {
            if segment == ".." {
                segments.pop().ok_or_else(invalid)?;
            } else {
                segments.push(segment);
            }
        }
        let resolved = canonical_path(&segments.join("/"), false).map_err(|_| invalid())?;
        if raw != resolved {
            let original = join(&self.root, raw.as_str())?;
            let canonical = join(&self.root, resolved.as_str())?;
            let actual_id = tree.identity(&original).map_err(CaptureError::Read)?;
            let canonical_id = tree.identity(&canonical).map_err(CaptureError::Read)?;
            if actual_id != canonical_id {
                return Err(invalid());
            }
        }
        Ok(resolved)
    }
}

fn unresolved_import(from: &RelPath, name: &str) -> CaptureError {
    mismatch(format!("unresolved schema import {name:?} from {from}"))
}

fn fields(node: &Node<'_>, class: &str, keys: &[&str]) -> Result<(), ReportError> {
    let found = shape::class(node)?;
    if found != class {
        return Err(ReportError::Kind {
            expected: class.into(),
            found: found.into(),
        });
    }
    if let Some(unexpected) = node.properties().find(|property| {
        *property != raf_rdf::vocab::TYPE
            && !property
                .strip_prefix(raf_rdf::vocab::REPORT)
                .is_some_and(|key| keys.contains(&key))
    }) {
        return Err(ReportError::UnknownPredicate(unexpected.into()));
    }
    Ok(())
}

fn digest(node: &Node<'_>, key: &str) -> Result<Digest, CaptureError> {
    let mut bytes = [0; 32];
    purrdf_hash::hex::decode_canonical_to_slice(&shape::string(node, key)?, &mut bytes)
        .map_err(|_| mismatch(format!("capture {key} must be a lowercase SHA-256 digest")))?;
    Ok(bytes)
}

fn canonical_path(text: &str, root_allowed: bool) -> Result<RelPath, CaptureError> {
    if (!root_allowed && text.is_empty())
        || !RelPath::is_canonical(text)
        || text.split('/').any(|segment| segment == "..")
    {
        return Err(mismatch(format!(
            "capture path {text:?} must be canonical and inside its root"
        )));
    }
    join(&RelPath::root(), text)
}

fn join(path: &RelPath, suffix: &str) -> Result<RelPath, CaptureError> {
    if suffix.len() > MAX_PATH_BYTES || suffix.chars().any(char::is_control) {
        return Err(mismatch(
            "capture path exceeds its length bound or contains a control character",
        ));
    }
    let joined = path
        .join(suffix)
        .map_err(|error| mismatch(error.to_string()))?;
    if joined.as_str().len() > MAX_PATH_BYTES {
        return Err(mismatch("capture path exceeds its length bound"));
    }
    Ok(joined)
}

fn mismatch(message: impl Into<String>) -> CaptureError {
    CaptureError::Mismatch(message.into())
}
