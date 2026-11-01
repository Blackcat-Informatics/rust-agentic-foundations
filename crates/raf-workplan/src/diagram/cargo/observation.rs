// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Explicit Rustdoc selection and native invocation receipt admission.

use super::{DiagramError, DocumentationRun, Library, LibraryArtifact, Selection, refused};
use crate::exec::{DEFAULT_OUTPUT_BOUND, Invocation, Output, Runner, RustdocObservationOptions};
use purrdf_core::ContentDigest;
use purrdf_lex::allocation::{Memory, Resident};
use raf_rdf::{Format, Graph};
use raf_source::{DiskTree, EntryKind, FileId, Opened, RelPath, Tree};
use std::path::Path;

pub(super) mod driver;
pub(super) mod receipt;

/// One selected HTML artifact joined to successful native compiler observations.
#[derive(Debug, Clone)]
pub struct InvocationObservation {
    artifact: LibraryArtifact,
    observed: receipt::Observation,
    graph: Graph,
    file_id: FileId,
}

impl InvocationObservation {
    /// Exact selected Cargo library and admitted HTML entry.
    #[must_use]
    pub fn artifact(&self) -> &LibraryArtifact {
        &self.artifact
    }
    /// Original ordered arguments Cargo supplied to the observing driver.
    #[must_use]
    pub fn arguments(&self) -> &[String] {
        &self.observed.arguments
    }
    /// Actual working directory received by the selected Rustdoc driver.
    #[must_use]
    pub fn directory(&self) -> &str {
        &self.observed.directory
    }
    /// Separate information-query arguments for the same compiler and environment.
    #[must_use]
    pub fn configuration_arguments(&self) -> &[String] {
        &self.observed.query_arguments
    }
    /// Actual bounded native configuration query output, including `doc`.
    #[must_use]
    pub fn configuration_output(&self) -> &Output {
        &self.observed.configuration
    }
    /// Actual bounded HTML compiler output and successful exit status.
    #[must_use]
    pub fn html_output(&self) -> &Output {
        &self.observed.html
    }
    /// Optional pre-HTML include evidence, separate from emitted href admission.
    #[must_use]
    pub fn assembly_preparation(&self) -> Option<&super::assembly::Preparation> {
        self.observed.assembly.as_ref()
    }
    /// Cargo's environment as received by the observing driver, in name order.
    #[must_use]
    pub fn cargo_environment(&self) -> &[(String, String)] {
        &self.observed.cargo_environment
    }
    /// Shared compiler environment after removing observer controls.
    #[must_use]
    pub fn compiler_environment(&self) -> &[(String, String)] {
        &self.observed.compiler_environment
    }
    /// Complete closed native RDF receipt; it does not attest atomic source reads.
    #[must_use]
    pub fn graph(&self) -> &Graph {
        &self.graph
    }
    /// Physical identity from the same contained handle as the receipt bytes.
    #[must_use]
    pub fn file_id(&self) -> &FileId {
        &self.file_id
    }
}

/// Successful Cargo artifact capture with one compiler receipt per selected library.
#[derive(Debug, Clone)]
pub struct ObservedDocumentation {
    run: DocumentationRun,
    invocations: Vec<InvocationObservation>,
}

impl ObservedDocumentation {
    /// Original metadata, Cargo protocol, output and HTML observations.
    #[must_use]
    pub fn documentation(&self) -> &DocumentationRun {
        &self.run
    }
    /// Compiler observations in the metadata selection's library order.
    #[must_use]
    pub fn invocations(&self) -> &[InvocationObservation] {
        &self.invocations
    }
}

/// Document with deliberately selected driver and compiler executables.
///
/// This operation creates a new receipt directory and shares the ordinary Cargo
/// artifact-admission path. Native information queries observe configuration
/// separately from HTML execution. Neither claims expanded inputs, atomic reads,
/// authenticated binaries, complete links or a published documentation assembly.
///
/// # Errors
/// Refuses recursive/missing executables, existing receipt directories, failed
/// Cargo operations, cached output without a new invocation, malformed/duplicate
/// or unmatched native receipts, and changed executable file identities.
pub fn document(
    runner: &dyn Runner,
    selection: &Selection,
    options: &RustdocObservationOptions,
) -> Result<ObservedDocumentation, DiagramError> {
    let options = canonical_options(options)?;
    let driver = executable_identity(options.driver())?;
    let delegate = executable_identity(options.delegate())?;
    if driver == delegate {
        return Err(refused(
            "Rustdoc driver and delegate name the same executable",
        ));
    }
    private_directory(options.receipts())?;
    let invocation = Invocation::cargo_doc_observed(selection.options(), &options)
        .at_directory(selection.workspace_root());
    let run = super::document_invocation(runner, selection, invocation)?;
    let invocations = admit(&run, &options)?;
    if executable_identity(options.driver())? != driver
        || executable_identity(options.delegate())? != delegate
    {
        return Err(refused(
            "selected Rustdoc executable identity changed during documentation",
        ));
    }
    Ok(ObservedDocumentation { run, invocations })
}

/// Dispatch only the scoped Cargo observing-driver mode before ordinary CLI parsing.
///
/// `None` leaves ordinary command handling untouched. No process-wide environment
/// is modified. Driver calls use the same bounded native Runner as other commands.
///
/// # Errors
/// A selected driver returns an actionable error for invalid controls, recursive
/// delegation, unsupported arguments, failed queries or receipt publication errors.
pub fn dispatch_driver() -> Option<Result<i32, DiagramError>> {
    std::env::var_os(RustdocObservationOptions::MARKER).map(|marker| {
        if marker != "1" {
            return Err(refused("unsupported Rustdoc observer mode"));
        }
        driver::run()
    })
}

fn canonical_options(
    options: &RustdocObservationOptions,
) -> Result<RustdocObservationOptions, DiagramError> {
    let receipts = options.receipts();
    let parent = receipts
        .parent()
        .ok_or_else(|| refused("Rustdoc receipts need a parent directory"))?
        .canonicalize()
        .map_err(DiagramError::data)?;
    let name = receipts
        .file_name()
        .ok_or_else(|| refused("Rustdoc receipts need a new directory name"))?;
    let canonical =
        RustdocObservationOptions::new(options.driver(), options.delegate(), &parent.join(name))
            .map_err(DiagramError::data)?;
    if let Some((plan, digest)) = options.assembly() {
        canonical
            .with_assembly(
                plan,
                ContentDigest::from_hex(digest)
                    .ok_or_else(|| refused("assembly plan digest is invalid"))?,
            )
            .map_err(DiagramError::data)
    } else {
        Ok(canonical)
    }
}

#[cfg(unix)]
fn private_directory(path: &Path) -> Result<(), DiagramError> {
    use std::os::unix::fs::DirBuilderExt as _;
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(path)
        .map_err(DiagramError::data)
}

#[cfg(not(unix))]
fn private_directory(_path: &Path) -> Result<(), DiagramError> {
    Err(refused(
        "private Rustdoc environment receipts require native Unix directory permissions",
    ))
}

fn executable_identity(path: &Path) -> Result<FileId, DiagramError> {
    let resolved = path.canonicalize().map_err(DiagramError::data)?;
    let path = resolved.as_path();
    let parent = path
        .parent()
        .ok_or_else(|| refused("Rustdoc executable has no parent"))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| refused("Rustdoc executable name must be UTF-8"))?;
    let disk = DiskTree::new(parent).map_err(DiagramError::data)?;
    disk.identity(&RelPath::new(name).map_err(DiagramError::data)?)
        .map_err(DiagramError::data)
}

fn admit(
    run: &DocumentationRun,
    options: &RustdocObservationOptions,
) -> Result<Vec<InvocationObservation>, DiagramError> {
    let disk = DiskTree::new(options.receipts()).map_err(DiagramError::data)?;
    let entries = disk.list(&RelPath::root()).map_err(DiagramError::data)?;
    if entries.len() != run.artifacts().len() {
        return Err(refused(
            "every selected library requires exactly one new Rustdoc receipt; cached or incomplete output is unsupported",
        ));
    }
    let mut pending = Vec::new();
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut remaining = DEFAULT_OUTPUT_BOUND;
    for entry in entries {
        if entry.kind != EntryKind::File {
            return Err(refused(
                "Rustdoc receipt directory contains a non-file entry",
            ));
        }
        let path = RelPath::new(&entry.name).map_err(DiagramError::data)?;
        let Opened::Read { id, text } = disk
            .read_unseen(&path, remaining as u64, &|_| false)
            .map_err(DiagramError::data)?
        else {
            return Err(refused("Rustdoc receipt was not read"));
        };
        remaining = remaining
            .checked_sub(text.len())
            .ok_or_else(|| refused("Rustdoc receipts exceed their cumulative byte bound"))?;
        let graph = Graph::from_text(&text, Format::NTriples).map_err(DiagramError::data)?;
        let observed = receipt::Observation::read(&graph)?;
        let expected = format!("{}.nt", ContentDigest::of(text.as_bytes()));
        if entry.name != expected {
            return Err(refused(
                "Rustdoc receipt filename disagrees with its native content",
            ));
        }
        validate_observation(&observed, options)?;
        memory
            .push(&mut pending, (observed, graph, id))
            .map_err(DiagramError::data)?;
    }
    let mut invocations = Vec::new();
    for artifact in run.artifacts() {
        let selected = matching_receipt(&pending, artifact.library())?;
        let (observed, graph, file_id) = pending.remove(selected);
        memory
            .push(
                &mut invocations,
                InvocationObservation {
                    artifact: artifact.clone(),
                    observed,
                    graph,
                    file_id,
                },
            )
            .map_err(DiagramError::data)?;
    }
    Ok(invocations)
}

fn validate_observation(
    observed: &receipt::Observation,
    options: &RustdocObservationOptions,
) -> Result<(), DiagramError> {
    if Path::new(&observed.driver) != options.driver()
        || Path::new(&observed.delegate) != options.delegate()
        || !Path::new(&observed.directory).is_absolute()
    {
        return Err(refused(
            "Rustdoc receipt disagrees with the selected driver, delegate or absolute working directory",
        ));
    }
    let expected = driver::query_arguments(&observed.arguments)?;
    if expected != observed.query_arguments
        || driver::compiler_environment(&observed.cargo_environment, options)?
            != observed.compiler_environment
    {
        return Err(refused(
            "Rustdoc query does not preserve its original arguments and compiler environment",
        ));
    }
    let cfg = std::str::from_utf8(&observed.configuration.stdout).map_err(DiagramError::data)?;
    if !cfg.lines().any(|line| line == "doc") {
        return Err(refused(
            "native Rustdoc configuration query did not report doc",
        ));
    }
    match (options.assembly(), &observed.assembly) {
        (None, None) => {}
        (Some((_, digest)), Some(prepared)) if prepared.plan == digest => {
            if prepared.html_arguments != super::assembly::html_arguments(&observed.arguments)? {
                return Err(refused(
                    "assembly receipt changes the required effective HTML invocation",
                ));
            }
        }
        _ => {
            return Err(refused(
                "assembly receipt and exact scoped plan selection disagree",
            ));
        }
    }
    Ok(())
}

fn matching_receipt(
    pending: &[(receipt::Observation, Graph, FileId)],
    library: &Library,
) -> Result<usize, DiagramError> {
    let mut found = None;
    for (index, (observed, _, _)) in pending.iter().enumerate() {
        if !driver::matches_library(observed, library)? {
            continue;
        }
        if found.replace(index).is_some() {
            return Err(refused(
                "selected library has duplicate Rustdoc invocation receipts",
            ));
        }
    }
    found.ok_or_else(|| {
        refused(&format!(
            "selected library {} has no matching Rustdoc compiler receipt",
            library.package
        ))
    })
}
