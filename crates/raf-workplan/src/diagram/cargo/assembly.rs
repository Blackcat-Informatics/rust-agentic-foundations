// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Compiler-grounded include preparation before delegated HTML execution.
//!
//! Real Rustdoc consumes prepared includes with required-link lint enforcement.
//! Admitted Cargo HTML artifacts select the shared publication roots. Explorer
//! API href materialization and compiler-expanded inputs remain separate claims.

mod configuration;
mod plan;
mod prepare;
mod receiver;
mod record;
mod seed;
mod source;

pub use plan::{AssemblyPlan, IncludeRequest};
pub use prepare::Preparation;

use super::{DiagramError, Selection, observation};
use crate::exec::{Runner, RustdocObservationOptions};
use purrdf_core::ContentDigest;
use raf_rdf::{Format, Graph};
use raf_source::{DiskTree, Opened, RelPath, Tree};
use std::path::Path;
use std::path::PathBuf;

/// Observed compiler execution and owned shared-bundle publication results.
/// Required links were consumed by the actual Rustdoc compiler. This does not
/// materialize their hrefs for explorer navigation or admit generated definitions.
#[derive(Debug, Clone)]
pub struct AssemblyDocumentation {
    observed: observation::ObservedDocumentation,
    publications: Vec<PathBuf>,
}

impl AssemblyDocumentation {
    /// Private instrumentation evidence; never copied into public diagram files.
    #[must_use]
    pub const fn observations(&self) -> &observation::ObservedDocumentation {
        &self.observed
    }
    /// Complete immutable bundles beneath actual selected Cargo artifact roots.
    #[must_use]
    pub fn publications(&self) -> &[PathBuf] {
        &self.publications
    }
}

const MAX_ITEMS: usize = 4096;
const MAX_BYTES: usize = 16 * 1024 * 1024;

fn refused(reason: impl Into<String>) -> DiagramError {
    DiagramError::refused(reason)
}

fn digest(text: &str) -> Result<ContentDigest, DiagramError> {
    ContentDigest::from_hex(text)
        .filter(|digest| digest.to_hex() == text)
        .ok_or_else(|| refused("assembly digest must be canonical lowercase SHA-256"))
}

/// Generate admitted includes inside actual compiler invocations, then document.
///
/// The plan is created exclusively at the caller's absolute locator. It selects
/// exactly the previously captured libraries. Each driver generates its owned
/// includes after its native configuration query and before delegated HTML.
/// Required intra-doc links are admitted by actual compiler consumption under
/// forbid-level link lints. Shared bundles are then published in the documentation
/// root beside Cargo's replaceable crate directories. No explorer href is inferred
/// from a source symbol.
/// All original static publication operations stay separate.
/// The caller's seed model supplies complete includes for ordinary dependency
/// compilation and names the complete document-model package roots in the plan.
/// The driver reloads a declared-source model for diagrams and independently
/// resolves required API spellings in its exact compiler-configured source.
/// The caller model supplies no compiler configuration or final link evidence.
///
/// # Errors
/// Refuses mismatched selection, existing plan paths, unavailable exact source
/// definitions/includes, failed compiler operations and closed-record failures.
pub fn document(
    runner: &dyn Runner,
    selection: &Selection,
    instrumentation: &RustdocObservationOptions,
    plan: &AssemblyPlan,
    plan_path: &Path,
    seed_model: &crate::Workplan<'_>,
) -> Result<AssemblyDocumentation, DiagramError> {
    use std::io::Write as _;
    if instrumentation.assembly().is_some() {
        return Err(refused(
            "assembly operation owns its one exact plan selection",
        ));
    }
    plan.matches_selection(selection)?;
    let parent = plan_path
        .parent()
        .ok_or_else(|| refused("assembly plan has no parent"))?;
    if parent.canonicalize().map_err(DiagramError::data)? != parent {
        return Err(refused(
            "assembly plan parent must be an explicit canonical directory",
        ));
    }
    let (concrete, seeds) = seed::prepare(plan, seed_model, plan_path, instrumentation)?;
    let text = concrete.to_ntriples()?;
    let controls = instrumentation
        .clone()
        .with_assembly(plan_path, ContentDigest::of(text.as_bytes()))
        .map_err(DiagramError::data)?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    options
        .open(plan_path)
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .map_err(DiagramError::data)?;
    seed::write(plan, &seeds)?;
    let observed = observation::document(runner, selection, &controls)?;
    let publications = prepare::publish(&controls, &concrete, &observed)?;
    Ok(AssemblyDocumentation {
        observed,
        publications,
    })
}

pub(crate) fn read_plan(
    path: &Path,
    digest: &str,
) -> Result<(AssemblyPlan, raf_source::FileId), DiagramError> {
    let parent = path
        .parent()
        .ok_or_else(|| refused("assembly plan has no parent"))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| refused("assembly plan name must be UTF-8"))?;
    let tree = DiskTree::new(parent).map_err(DiagramError::data)?;
    let path = RelPath::new(name).map_err(DiagramError::data)?;
    let Opened::Read { id, text } = tree
        .read_unseen(&path, MAX_BYTES as u64, &|_| false)
        .map_err(DiagramError::data)?
    else {
        return Err(refused("assembly plan was not read"));
    };
    if ContentDigest::of(text.as_bytes()).to_string() != digest {
        return Err(refused(
            "assembly plan digest disagrees with its exact bytes",
        ));
    }
    let graph = Graph::from_text(&text, Format::NTriples).map_err(DiagramError::data)?;
    let plan = AssemblyPlan::read(&graph)?;
    if plan.to_ntriples()? != text {
        return Err(refused("assembly plan must use canonical native N-Triples"));
    }
    Ok((plan, id))
}

pub(crate) use prepare::Pending;
pub(in crate::diagram::cargo) use prepare::html_arguments;
