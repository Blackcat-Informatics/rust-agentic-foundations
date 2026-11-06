// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Bounded explicit Cargo metadata and documentation artifact readers.
//! These APIs never invoke Cargo, compile source or grant execution authority.

use super::DiagramError;
use purrdf_core::ContentDigest;
use purrdf_lex::{
    json::{self, Value},
    walk::try_sort_unstable_by,
};
use std::path::{Path, PathBuf};

mod artifact;
mod options;
pub use artifact::LibraryArtifact;
pub use options::{CargoDocOptions, CargoDocOptionsError};

/// Maximum bytes in one imported metadata or documentation message carrier.
pub const MAX_METADATA_BYTES: usize = 16 * 1024 * 1024;
const DEFAULT_OUTPUT_BOUND: usize = MAX_METADATA_BYTES;

/// One exact workspace library selected from Cargo's format-v1 metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    /// Opaque Cargo package ID, never reconstructed from its name.
    pub package_id: String,
    /// Cargo package name.
    pub package: String,
    /// Cargo package version.
    pub version: String,
    /// Cargo's absolute manifest locator.
    pub manifest: PathBuf,
    /// Cargo's actual library target name.
    pub name: String,
    /// Canonical Cargo target-kind set.
    pub kind: Vec<String>,
    /// Canonical Cargo crate-type set.
    pub crate_types: Vec<String>,
    /// Cargo's absolute library source locator.
    pub source: PathBuf,
    /// Cargo metadata's workspace-resolved feature union, separate from artifact
    /// features and compiler cfg proof.
    pub features: Vec<String>,
}

/// Selected libraries described by explicitly supplied metadata.
/// The metadata is imported data, not evidence of current compiler execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    options: CargoDocOptions,
    root: PathBuf,
    target_directory: PathBuf,
    libraries: Vec<Library>,
    metadata_sha256: ContentDigest,
    metadata_bytes: Vec<u8>,
}

impl Selection {
    /// The declared documentation request.
    #[must_use]
    pub fn options(&self) -> &CargoDocOptions {
        &self.options
    }
    /// The caller-selected canonical workspace root.
    #[must_use]
    pub fn workspace_root(&self) -> &Path {
        &self.root
    }
    /// The caller-authorized canonical artifact root.
    #[must_use]
    pub fn target_directory(&self) -> &Path {
        &self.target_directory
    }
    /// Selected library declarations from the imported metadata.
    #[must_use]
    pub fn libraries(&self) -> &[Library] {
        &self.libraries
    }
    /// Content identity of the exact imported metadata bytes.
    #[must_use]
    pub const fn metadata_sha256(&self) -> ContentDigest {
        self.metadata_sha256
    }
    /// The bounded imported metadata carrier.
    #[must_use]
    pub fn metadata_bytes(&self) -> &[u8] {
        &self.metadata_bytes
    }
}

/// Imported message records and their contained documentation files.
/// Successful records in the carrier do not authenticate a compiler run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentationArtifacts {
    selection: Selection,
    messages: Vec<Value>,
    artifacts: Vec<LibraryArtifact>,
}

impl DocumentationArtifacts {
    /// The imported library selection.
    #[must_use]
    pub fn selection(&self) -> &Selection {
        &self.selection
    }
    /// Bounded message values, retaining their supplied order.
    #[must_use]
    pub fn messages(&self) -> &[Value] {
        &self.messages
    }
    /// Files admitted beneath the caller-authorized artifact root.
    #[must_use]
    pub fn artifacts(&self) -> &[LibraryArtifact] {
        &self.artifacts
    }
}

/// Read imported metadata without launching tools or writing files.
/// The output directory in the data must match the explicit artifact authority.
///
/// # Errors
/// Refuses malformed or oversized input, root mismatches and invalid selections.
pub fn read_metadata(
    root: &Path,
    artifact_root: &Path,
    options: &CargoDocOptions,
    metadata_bytes: &[u8],
) -> Result<Selection, DiagramError> {
    let root = root.canonicalize().map_err(DiagramError::data)?;
    let metadata = read(metadata_bytes)?;
    if metadata.get("version").and_then(Value::as_u64) != Some(1) {
        return Err(refused("Cargo metadata must report format version 1"));
    }
    if absolute(&metadata, "workspace_root")?
        .canonicalize()
        .map_err(DiagramError::data)?
        != root
    {
        return Err(refused(
            "Cargo metadata workspace_root disagrees with the selected directory",
        ));
    }
    let target_directory = artifact_root.canonicalize().map_err(DiagramError::data)?;
    let reported = absolute(&metadata, "target_directory")?
        .canonicalize()
        .map_err(DiagramError::data)?;
    if reported != target_directory {
        return Err(refused(
            "metadata output directory differs from the caller-authorized artifact root",
        ));
    }
    let members = strings(required(&metadata, "workspace_members")?)?;
    let packages = array(required(&metadata, "packages")?, "packages")?;
    let nodes = array(
        required(required(&metadata, "resolve")?, "nodes")?,
        "resolve.nodes",
    )?;
    let mut libraries = Vec::new();
    for package in options.packages() {
        let mut matching = packages.iter().filter(|row| {
            row.get("name").and_then(Value::as_str) == Some(package.as_str())
                && row
                    .get("id")
                    .and_then(Value::as_str)
                    .is_some_and(|id| members.iter().any(|member| member == id))
        });
        let row = matching
            .next()
            .ok_or_else(|| refused(&format!("workspace package `{package}` is absent")))?;
        if matching.next().is_some() {
            return Err(refused(&format!(
                "workspace package `{package}` is ambiguous; select a unique package name"
            )));
        }
        libraries.push(library(row, nodes)?);
    }
    Ok(Selection {
        options: options.clone(),
        root,
        target_directory,
        libraries,
        metadata_sha256: ContentDigest::of(metadata_bytes),
        metadata_bytes: metadata_bytes.to_vec(),
    })
}

/// Read message data and existing HTML files without compiling or publishing.
///
/// # Errors
/// Refuses oversized or incomplete carriers, mismatched library identities and
/// files outside the authorized artifact root. Original file failures are retained.
pub fn read_artifacts(
    selection: &Selection,
    message_bytes: &[u8],
) -> Result<DocumentationArtifacts, DiagramError> {
    if message_bytes.len() > MAX_METADATA_BYTES {
        return Err(refused("documentation messages exceed their byte bound"));
    }
    let messages = messages(message_bytes)?;
    let artifacts = artifact::admit(selection, &messages)?;
    Ok(DocumentationArtifacts {
        selection: selection.clone(),
        messages,
        artifacts,
    })
}

fn library(package: &Value, nodes: &[Value]) -> Result<Library, DiagramError> {
    if package.get("source") != Some(&Value::Null) {
        return Err(refused(
            "selected workspace libraries must have local manifest source identities",
        ));
    }
    let package_id = text(package, "id")?;
    let targets = array(required(package, "targets")?, "targets")?;
    let mut matching = targets.iter().filter(|target| {
        target
            .get("kind")
            .and_then(Value::as_array)
            .is_some_and(|kinds| {
                kinds.iter().any(|kind| {
                    matches!(
                        kind.as_str(),
                        Some("lib" | "rlib" | "dylib" | "cdylib" | "staticlib" | "proc-macro")
                    )
                })
            })
    });
    let target = matching
        .next()
        .ok_or_else(|| refused("selected package has no library target"))?;
    if matching.next().is_some() || target.get("doc") != Some(&Value::Bool(true)) {
        return Err(refused(
            "selected package must have exactly one documented library target",
        ));
    }
    let mut matching = nodes
        .iter()
        .filter(|node| node.get("id").and_then(Value::as_str) == Some(package_id));
    let node = matching
        .next()
        .ok_or_else(|| refused("selected package has no Cargo feature-resolution receipt"))?;
    if matching.next().is_some() {
        return Err(refused(
            "selected package has duplicate feature-resolution receipts",
        ));
    }
    let crate_types = strings(required(target, "crate_types")?)?;
    if crate_types.is_empty() {
        return Err(refused(
            "selected library must report a nonempty crate-type set",
        ));
    }
    Ok(Library {
        package_id: package_id.into(),
        package: text(package, "name")?.into(),
        version: text(package, "version")?.into(),
        manifest: absolute(package, "manifest_path")?,
        name: text(target, "name")?.into(),
        kind: strings(required(target, "kind")?)?,
        crate_types,
        source: absolute(target, "src_path")?,
        features: strings(required(node, "features")?)?,
    })
}

fn messages(bytes: &[u8]) -> Result<Vec<Value>, DiagramError> {
    let mut messages = Vec::new();
    let mut complete = false;
    for line in bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.iter().all(u8::is_ascii_whitespace))
    {
        let message = read(line)?;
        let reason = text(&message, "reason")?;
        if reason == "build-finished" {
            if complete || message.get("success") != Some(&Value::Bool(true)) {
                return Err(refused(
                    "Cargo documentation has a duplicate or unsuccessful completion receipt",
                ));
            }
            complete = true;
        } else if complete {
            return Err(refused(
                "Cargo emitted messages after its documentation completion receipt",
            ));
        }
        if reason == "compiler-message" {
            let diagnostic = required(&message, "message")?;
            let code = diagnostic
                .get("code")
                .and_then(|code| code.get("code"))
                .and_then(Value::as_str);
            if matches!(
                code,
                Some("rustdoc::broken_intra_doc_links" | "broken_intra_doc_links")
            ) {
                return Err(refused(&format!(
                    "required Rustdoc link is unresolved: {}",
                    text(diagnostic, "message")?
                )));
            }
        }
        messages.push(message);
    }
    if !complete {
        return Err(refused(
            "Cargo documentation emitted no successful build-finished receipt",
        ));
    }
    Ok(messages)
}
fn read(bytes: &[u8]) -> Result<Value, DiagramError> {
    if bytes.len() > DEFAULT_OUTPUT_BOUND {
        return Err(refused("Cargo JSON exceeds 16 MiB"));
    }
    json::read_slice(
        bytes,
        json::Limits {
            max_depth: 64,
            max_values: 1_000_000,
            max_string_bytes: DEFAULT_OUTPUT_BOUND,
            unique_members: true,
        },
    )
    .map_err(DiagramError::data)
}

fn required<'a>(value: &'a Value, key: &'static str) -> Result<&'a Value, DiagramError> {
    json::record::Record::new(value, "Cargo response")
        .and_then(|mut record| record.required_with(key, Ok::<_, json::record::DecodeError>))
        .map_err(DiagramError::data)
}
fn text<'a>(value: &'a Value, key: &'static str) -> Result<&'a str, DiagramError> {
    required(value, key)?
        .as_str()
        .filter(|text| !text.is_empty())
        .ok_or_else(|| refused(&format!("Cargo `{key}` must be a nonempty string")))
}
fn array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value], DiagramError> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| refused(&format!("Cargo `{key}` must be an array")))
}
fn absolute(value: &Value, key: &'static str) -> Result<PathBuf, DiagramError> {
    let text = text(value, key)?;
    let path = PathBuf::from(text);
    if !path.is_absolute() || text.chars().any(char::is_control) {
        return Err(refused(&format!(
            "Cargo `{key}` must be an absolute path without controls"
        )));
    }
    Ok(path)
}
fn strings(value: &Value) -> Result<Vec<String>, DiagramError> {
    let mut strings = array(value, "string set")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|text| !text.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| refused("Cargo string sets must contain nonempty strings"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    try_sort_unstable_by(&mut strings, |left, right| {
        Ok::<_, std::convert::Infallible>(left.cmp(right))
    })
    .unwrap_or_else(|never| match never {});
    if strings.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(refused("Cargo string set contains duplicate values"));
    }
    Ok(strings)
}
fn refused(reason: &str) -> DiagramError {
    DiagramError::Refused {
        reason: format!("Cargo documentation: {reason}"),
    }
}
