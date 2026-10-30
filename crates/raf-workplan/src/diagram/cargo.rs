// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Selected Cargo documentation through the existing typed process runner.

use super::DiagramError;
use crate::exec::{CargoDocOptions, DEFAULT_OUTPUT_BOUND, Invocation, Limits, Output, Runner};
use purrdf_core::ContentDigest;
use purrdf_lex::{
    json::{self, Value},
    walk::try_sort_unstable_by,
};
use std::fmt;
use std::path::{Path, PathBuf};

mod artifact;
pub use artifact::LibraryArtifact;

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

/// A captured Cargo selection. Construction requires an actual runner response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    options: CargoDocOptions,
    root: PathBuf,
    target_directory: PathBuf,
    libraries: Vec<Library>,
    metadata_sha256: ContentDigest,
    metadata_output: Output,
}

impl Selection {
    /// The exact validated request retained for both Cargo operations.
    #[must_use]
    pub fn options(&self) -> &CargoDocOptions {
        &self.options
    }
    /// Canonical workspace directory in which the metadata command ran.
    #[must_use]
    pub fn workspace_root(&self) -> &Path {
        &self.root
    }
    /// Effective absolute target directory reported by Cargo itself.
    ///
    /// This is not a guessed documentation subdirectory. Target-specific output
    /// layout and actual generated HTML still require artifact evidence.
    #[must_use]
    pub fn target_directory(&self) -> &Path {
        &self.target_directory
    }
    /// Required selected workspace libraries, in request order.
    #[must_use]
    pub fn libraries(&self) -> &[Library] {
        &self.libraries
    }
    /// Native unframed SHA-256 of the exact bounded metadata response bytes.
    #[must_use]
    pub const fn metadata_sha256(&self) -> ContentDigest {
        self.metadata_sha256
    }
    /// Complete bounded metadata stdout and diagnostic stderr from the runner.
    #[must_use]
    pub fn metadata_output(&self) -> &Output {
        &self.metadata_output
    }
}

/// Cargo exited unsuccessfully; its bounded stdout diagnostics remain available.
#[derive(Debug)]
pub struct CargoFailure {
    command: String,
    output: Output,
}

impl CargoFailure {
    /// Original bounded streams and exit status, without success-protocol parsing.
    #[must_use]
    pub fn output(&self) -> &Output {
        &self.output
    }
}

impl fmt::Display for CargoFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "`{}` exited {:?}\nstdout:\n{}\nstderr:\n{}",
            self.command,
            self.output.code,
            String::from_utf8_lossy(&self.output.stdout),
            String::from_utf8_lossy(&self.output.stderr)
        )
    }
}
impl std::error::Error for CargoFailure {}

/// A successful Cargo operation and its reported artifact observations.
///
/// These observations do not attest effective compiler configuration, source
/// expansion, generated inputs, atomic reads or complete HTML link validity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentationRun {
    selection: Selection,
    recaptured_metadata_output: Output,
    invocation: Invocation,
    output: Output,
    messages: Vec<Value>,
    artifacts: Vec<LibraryArtifact>,
}

impl DocumentationRun {
    /// Original metadata selection whose scope remained equal after Cargo completed.
    #[must_use]
    pub fn selection(&self) -> &Selection {
        &self.selection
    }
    /// Complete bounded metadata output recaptured after documentation completed.
    /// Its stdout equals the original response; diagnostic stderr may differ.
    #[must_use]
    pub fn recaptured_metadata_output(&self) -> &Output {
        &self.recaptured_metadata_output
    }
    /// Exact typed Cargo documentation operation.
    #[must_use]
    pub fn invocation(&self) -> &Invocation {
        &self.invocation
    }
    /// Original bounded process output, including diagnostics.
    #[must_use]
    pub fn output(&self) -> &Output {
        &self.output
    }
    /// Complete native JSON messages emitted by Cargo, in order.
    ///
    /// Hosts use actual artifact messages for output admission; this adapter
    /// does not manufacture HTML paths or infer cfg from requested features.
    #[must_use]
    pub fn messages(&self) -> &[Value] {
        &self.messages
    }
    /// Reported HTML entries admitted through the contained native disk reader.
    ///
    /// One entry corresponds to each selected library, in selection order.
    /// These receipts observe output bytes; they do not attest effective cfg,
    /// atomic subprocess reads, HTML provenance or every embedded link.
    #[must_use]
    pub fn artifacts(&self) -> &[LibraryArtifact] {
        &self.artifacts
    }
}

/// Capture Cargo's effective target directory and exact selected workspace libraries.
///
/// Uses the native bounded JSON reader and the existing Runner. Metadata and
/// documentation receive identical explicit feature/output-directory requests.
/// Non-library targets are not selected for this documentation operation.
///
/// # Errors
/// Refuses process failures, malformed/bounded metadata, a mismatched workspace
/// root, non-workspace or ambiguous packages, missing/disabled/ambiguous library
/// targets, absent feature-resolution rows and non-absolute Cargo path locators.
pub fn capture(
    runner: &dyn Runner,
    root: &Path,
    options: &CargoDocOptions,
) -> Result<Selection, DiagramError> {
    let root = root.canonicalize().map_err(DiagramError::data)?;
    let invocation = Invocation::cargo_doc_metadata(options).at_directory(&root);
    let output = execute(runner, &invocation)?;
    let metadata = read(&output.stdout)?;
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
    let target_directory = absolute(&metadata, "target_directory")?;
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
        metadata_sha256: ContentDigest::of(&output.stdout),
        metadata_output: output,
    })
}

/// Run selected Cargo library documentation and observe reported output bytes.
///
/// Cargo receives `--locked --no-deps --lib` and the exact
/// retained feature/directory request. It owns cumulative documentation output;
/// this function never cleans, replaces or publishes documentation files.
///
/// Metadata feature names and source locators do not prove effective compiler
/// cfg, build-script behavior or complete link validity. HTML observations admit
/// Cargo's reported identity and bounded contained bytes. They do not publish
/// an assembly or authenticate its compiler inputs.
/// Both bounded Cargo metadata responses must be byte-identical. This
/// conservative temporal check includes dependency resolution and also refuses
/// formatting-only changes; it does not attest intervening state or agreement
/// with source bytes before the initial metadata capture.
///
/// # Errors
/// Refuses process or JSON protocol failures, incomplete Cargo completion,
/// reported broken intra-doc link diagnostics, changed metadata,
/// and missing, ambiguous, mismatched or unreadable required HTML artifacts.
pub fn document(
    runner: &dyn Runner,
    selection: &Selection,
) -> Result<DocumentationRun, DiagramError> {
    let invocation = Invocation::cargo_doc(&selection.options).at_directory(&selection.root);
    let output = execute(runner, &invocation)?;
    let messages = messages(&output.stdout)?;
    let after = capture(runner, &selection.root, &selection.options)?;
    if after.root != selection.root
        || after.target_directory != selection.target_directory
        || after.libraries != selection.libraries
        || after.metadata_output.stdout != selection.metadata_output.stdout
    {
        return Err(refused(
            "selected Cargo documentation scope changed during the run; regenerate against the current metadata",
        ));
    }
    let artifacts = artifact::admit(&after, &messages)?;
    Ok(DocumentationRun {
        selection: selection.clone(),
        recaptured_metadata_output: after.metadata_output,
        invocation,
        output,
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

fn execute(runner: &dyn Runner, invocation: &Invocation) -> Result<Output, DiagramError> {
    let output = runner
        .run(invocation, &Limits::build())
        .map_err(DiagramError::data)?;
    if output.stdout.len() > DEFAULT_OUTPUT_BOUND || output.stderr.len() > DEFAULT_OUTPUT_BOUND {
        return Err(refused(
            "Cargo response exceeds the runner's 16 MiB stream bound",
        ));
    }
    if !output.success() {
        return Err(DiagramError::data(CargoFailure {
            command: invocation.to_string(),
            output,
        }));
    }
    Ok(output)
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
