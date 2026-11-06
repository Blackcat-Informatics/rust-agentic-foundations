// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Explicit trusted Cargo documentation; static commands never enter this module.
//!
//! Trust authorizes build scripts and proc macros, which can have arbitrary
//! effects. This operation is not a sandbox and does not supervise descendants.

mod capture;
mod clock;
mod configuration;

use crate::exit::Failure;
use raf_workplan::diagram::cargo::{self, CargoDocOptions};
use std::path::{Component, Path, PathBuf};

const STREAM_BYTES: usize = 16 * 1024 * 1024;

/// One invocation's explicit tools, artifact root and selected library scope.
pub(crate) struct BuildRequest {
    pub(crate) root: PathBuf,
    pub(crate) toolchain_directory: PathBuf,
    pub(crate) target_directory: PathBuf,
    pub(crate) packages: Vec<String>,
    pub(crate) features: Vec<String>,
    pub(crate) no_default_features: bool,
    pub(crate) trust_docs: bool,
}

/// Bounded documentation outputs; no test execution or full-environment receipt.
pub(crate) struct BuiltDocumentation {
    pub(crate) metadata_before: Vec<u8>,
    pub(crate) metadata_after: Vec<u8>,
    pub(crate) messages: Vec<u8>,
    pub(crate) diagnostics: Vec<u8>,
    pub(crate) cargo_version: String,
    pub(crate) rustc_version: String,
    pub(crate) rustdoc_version: String,
}

struct Tools {
    cargo: PathBuf,
    rustc: PathBuf,
    rustdoc: PathBuf,
}
struct Checked {
    root: PathBuf,
    target: PathBuf,
    tools: Tools,
    options: CargoDocOptions,
}

#[derive(Clone, Copy)]
enum Operation {
    CargoVersion,
    RustcVersion,
    RustdocVersion,
    Metadata,
    Documentation,
}

impl Operation {
    const fn label(self) -> &'static str {
        match self {
            Self::CargoVersion => "Cargo version",
            Self::RustcVersion => "Rust compiler version",
            Self::RustdocVersion => "Rustdoc version",
            Self::Metadata => "Cargo metadata",
            Self::Documentation => "selected-library Cargo documentation",
        }
    }
    const fn is_build(self) -> bool {
        matches!(self, Self::Documentation)
    }
    const fn output_limit(self) -> usize {
        match self {
            Self::Metadata | Self::Documentation => STREAM_BYTES,
            _ => 64 * 1024,
        }
    }
}

/// Build only after explicit invocation-scoped trust, with fixed native tools.
pub(crate) fn build(request: &BuildRequest) -> Result<BuiltDocumentation, Failure> {
    if !request.trust_docs {
        return Err(Failure::new(
            "documentation compilation requires explicit --trust-docs; build scripts and proc macros can execute arbitrary code",
        ));
    }
    let checked = Checked::new(request)?;
    configuration::inspect(&checked.root)?;
    std::fs::create_dir_all(&checked.target)?;
    if checked.target.canonicalize()? != checked.target {
        return Err(Failure::new(
            "authorized target directory changed while it was created",
        ));
    }
    let mut diagnostics = Vec::new();
    let cargo_version = version(&checked, Operation::CargoVersion, &mut diagnostics)?;
    let rustc_version = version(&checked, Operation::RustcVersion, &mut diagnostics)?;
    let rustdoc_version = version(&checked, Operation::RustdocVersion, &mut diagnostics)?;
    let before = capture::run(&checked, Operation::Metadata)?;
    append(&mut diagnostics, &before.stderr)?;
    // Selection must be admitted before Cargo can build the requested libraries.
    cargo::read_metadata(
        &checked.root,
        &checked.target,
        &checked.options,
        &before.stdout,
    )?;
    let documented = capture::run(&checked, Operation::Documentation)?;
    append(&mut diagnostics, &documented.stderr)?;
    let after = capture::run(&checked, Operation::Metadata)?;
    append(&mut diagnostics, &after.stderr)?;
    cargo::read_metadata(
        &checked.root,
        &checked.target,
        &checked.options,
        &after.stdout,
    )?;
    if before.stdout != after.stdout {
        return Err(Failure::new(
            "Cargo metadata changed during trusted documentation; no stable documentation result is admitted",
        ));
    }
    Ok(BuiltDocumentation {
        metadata_before: before.stdout,
        metadata_after: after.stdout,
        messages: documented.stdout,
        diagnostics,
        cargo_version,
        rustc_version,
        rustdoc_version,
    })
}

fn append(destination: &mut Vec<u8>, bytes: &[u8]) -> Result<(), Failure> {
    let size = destination
        .len()
        .checked_add(bytes.len())
        .filter(|size| *size <= STREAM_BYTES)
        .ok_or_else(|| {
            Failure::new("trusted documentation diagnostics exceed their cumulative 16 MiB bound")
        })?;
    destination.try_reserve_exact(size - destination.len())?;
    destination.extend_from_slice(bytes);
    Ok(())
}

fn version(
    checked: &Checked,
    operation: Operation,
    diagnostics: &mut Vec<u8>,
) -> Result<String, Failure> {
    let output = capture::run(checked, operation)?;
    append(diagnostics, &output.stderr)?;
    let text = std::str::from_utf8(&output.stdout)?;
    if text.trim().is_empty() {
        return Err(Failure::new(format!(
            "{} returned no version",
            operation.label()
        )));
    }
    Ok(text.trim_end().to_owned())
}

fn spelling(path: &Path, name: &str) -> Result<(), Failure> {
    let text = path
        .to_str()
        .ok_or_else(|| Failure::new(format!("{name} must be UTF-8")))?;
    if !path.is_absolute()
        || text.len() > 4096
        || text.chars().any(char::is_control)
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        return Err(Failure::new(format!(
            "{name} must be an absolute path of at most 4096 bytes without controls or dot components"
        )));
    }
    Ok(())
}

fn planned_directory(path: &Path) -> Result<PathBuf, Failure> {
    let mut missing = Vec::new();
    let mut existing = path;
    loop {
        match std::fs::metadata(existing) {
            Ok(metadata) if metadata.is_dir() => break,
            Ok(_) => {
                return Err(Failure::new(
                    "target directory has a non-directory ancestor",
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                missing.push(
                    existing
                        .file_name()
                        .ok_or_else(|| Failure::new("target directory has no existing ancestor"))?
                        .to_os_string(),
                );
                existing = existing
                    .parent()
                    .ok_or_else(|| Failure::new("target directory has no existing ancestor"))?;
            }
            Err(error) => return Err(error.into()),
        }
    }
    let mut resolved = existing.canonicalize()?;
    for component in missing.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn overlaps(left: &Path, right: &Path) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

impl Checked {
    fn new(request: &BuildRequest) -> Result<Self, Failure> {
        for (path, name) in [
            (&request.root, "workspace root"),
            (&request.toolchain_directory, "toolchain directory"),
            (&request.target_directory, "target directory"),
        ] {
            spelling(path, name)?;
        }
        let root = request.root.canonicalize()?;
        let toolchain = request.toolchain_directory.canonicalize()?;
        if !root.is_dir() || !toolchain.is_dir() {
            return Err(Failure::new(
                "workspace and toolchain roots must be existing directories",
            ));
        }
        let target = planned_directory(&request.target_directory)?;
        if overlaps(&target, &root)
            || overlaps(&target, &toolchain)
            || target.components().any(|part| part.as_os_str() == ".git")
        {
            return Err(Failure::new(
                "documentation target must not overlap source, toolchain or Git administration directories",
            ));
        }
        reject_git_administration(&root, &target)?;
        let packages: Vec<_> = request.packages.iter().map(String::as_str).collect();
        let features: Vec<_> = request.features.iter().map(String::as_str).collect();
        let options = CargoDocOptions::new(
            &packages,
            &features,
            request.no_default_features,
            Some(&target),
        )?;
        let tools = Tools::open(&toolchain)?;
        Ok(Self {
            root,
            target,
            tools,
            options,
        })
    }
}

fn reject_git_administration(root: &Path, target: &Path) -> Result<(), Failure> {
    if let Some(repository) = raf_source::git::Repository::discover(root)? {
        for directory in [repository.git_dir(), repository.common_dir()] {
            if overlaps(target, &directory.canonicalize()?) {
                return Err(Failure::new(
                    "documentation target overlaps Git administration",
                ));
            }
        }
    }
    let mut ancestor = target;
    while !ancestor.exists() {
        ancestor = ancestor
            .parent()
            .ok_or_else(|| Failure::new("target has no existing parent"))?;
    }
    if let Some(repository) = raf_source::git::Repository::discover(ancestor)?
        && (overlaps(target, &repository.git_dir().canonicalize()?)
            || overlaps(target, &repository.common_dir().canonicalize()?)
            || repository
                .work_dir()
                .is_some_and(|work| target.starts_with(work)))
    {
        return Err(Failure::new(
            "documentation target must be outside repository source and Git administration",
        ));
    }
    Ok(())
}

impl Tools {
    fn open(directory: &Path) -> Result<Self, Failure> {
        let tool = |name: &str| -> Result<PathBuf, Failure> {
            let path = directory.join(name).canonicalize()?;
            if path.parent() != Some(directory)
                || path.file_name() != Some(std::ffi::OsStr::new(name))
                || !path.is_file()
            {
                return Err(Failure::new(format!(
                    "{name} must be its fixed native executable inside the explicit toolchain directory"
                )));
            }
            Ok(path)
        };
        Ok(Self {
            cargo: tool("cargo")?,
            rustc: tool("rustc")?,
            rustdoc: tool("rustdoc")?,
        })
    }
}
