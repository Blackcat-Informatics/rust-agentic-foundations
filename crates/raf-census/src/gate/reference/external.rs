// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Caller-declared reference trees, selected by location and commit rather
//! than through the owning lock graph.

use super::refusal;
use crate::{Error, Ledger, ReferenceSource};
use purrdf_toml::{Bounds, Kind, Profile, Scalar, SourceDocument, Vocabulary};
use raf_source::{DiskTree, LoadOptions, PackageId, PinnedGitTree, RelPath, Tree, Workspace};
use std::path::{Path, PathBuf};

/// The `observation` token of a receipt read from an unpinned directory.
pub(crate) const UNPINNED_OBSERVATION: &str = "unpinned-tree";

/// A reference tree declared by the caller instead of selected through the
/// lock graph. Every package discovered from the tree's root is a reference
/// package, whatever the ledger's `reference` values select.
///
/// A relative location is resolved from the main checkout of the workspace's
/// Git repository, so a linked worktree and the main checkout compare against
/// the same tree. The receipt records the location exactly as declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalReference {
    location: PathBuf,
    pin: Pin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Pin {
    Commit(String),
    File { file: RelPath, key: String },
    Unpinned,
}

impl ExternalReference {
    /// Compare against the committed objects of `commit`, a full 40-character
    /// lowercase hexadecimal SHA-1, in the repository at `location`.
    #[must_use]
    pub fn commit(location: impl Into<PathBuf>, commit: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            pin: Pin::Commit(commit.into()),
        }
    }

    /// Compare against the commit that `key`, a dotted TOML key, names in the
    /// workspace-relative TOML pin file `file`.
    #[must_use]
    pub fn pin_file(location: impl Into<PathBuf>, file: RelPath, key: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            pin: Pin::File {
                file,
                key: key.into(),
            },
        }
    }

    /// Compare against the current files of the directory at `location`.
    /// Every receipt from this mode carries the `unpinned-tree` observation.
    #[must_use]
    pub fn unpinned(location: impl Into<PathBuf>) -> Self {
        Self {
            location: location.into(),
            pin: Pin::Unpinned,
        }
    }

    /// Whether this reference reads a directory rather than a pinned commit.
    #[must_use]
    pub const fn is_unpinned(&self) -> bool {
        matches!(self.pin, Pin::Unpinned)
    }
}

/// One loaded external tree and the receipt of each of its packages.
pub(crate) struct ExternalTree {
    pub(crate) workspace: Workspace,
    pub(crate) packages: Vec<(PackageId, ReferenceSource)>,
}

/// Resolve, read and load one declared reference tree.
pub(crate) fn load(
    reference: &ExternalReference,
    consumer: &dyn Tree,
    host: Option<&Path>,
    ledger: &Ledger,
) -> Result<ExternalTree, Error> {
    let spelling = reference.location.to_str().ok_or_else(|| {
        refusal(format!(
            "reference location {} is not UTF-8",
            reference.location.display()
        ))
    })?;
    let location = resolve(&reference.location, host)?;
    let mut options = LoadOptions::default();
    options.max_file_bytes = ledger.options().max_file_bytes;
    let revision = match &reference.pin {
        Pin::Commit(commit) => Some(commit.clone()),
        Pin::File { file, key } => Some(pin(consumer, file, key, &options)?),
        Pin::Unpinned => None,
    };
    let tree: Box<dyn Tree> = if let Some(commit) = &revision {
        Box::new(
            PinnedGitTree::open(&location, commit, &options).map_err(|error| {
                refusal(format!(
                    "pinned reference {spelling:?} at {commit:?}: {error}; the repository and commit must already be present locally, and nothing is fetched"
                ))
            })?,
        )
    } else {
        Box::new(DiskTree::new(&location).map_err(|source| Error::Tree {
            file: location.display().to_string(),
            source,
        })?)
    };
    let label = revision.as_deref().map_or_else(
        || format!("{spelling:?}"),
        |commit| format!("{spelling:?} at {commit}"),
    );
    let workspace = Workspace::load_with(tree.as_ref(), &["."], &options)
        .map_err(|error| refusal(format!("reference tree {label}: {error}")))?;
    let mut packages = Vec::new();
    for package in workspace.package_ids() {
        let info = workspace
            .package(package)
            .ok_or_else(|| refusal("external reference package is absent"))?;
        let version = workspace
            .package_version(package)
            .map_err(|error| refusal(format!("reference tree {label}: {error}")))?
            .unwrap_or("0.0.0")
            .to_owned();
        packages.push((
            package,
            ReferenceSource {
                package: info.name.clone(),
                version,
                source_kind: if revision.is_some() { "git" } else { "path" }.into(),
                repository: spelling.into(),
                selector: String::new(),
                revision: revision.clone().unwrap_or_default(),
                manifest: info.manifest.to_string(),
                checksum: String::new(),
                observation: if revision.is_some() {
                    "committed-git"
                } else {
                    UNPINNED_OBSERVATION
                }
                .into(),
            },
        ));
    }
    if packages.is_empty() {
        return Err(refusal(format!(
            "reference tree {label} yields no packages; an empty comparison is never reported as clean, so name the repository or directory whose root holds the reference workspace"
        )));
    }
    Ok(ExternalTree {
        workspace,
        packages,
    })
}

/// An absolute location is used as written; a relative one is joined to the
/// main checkout of the workspace's Git repository.
fn resolve(location: &Path, host: Option<&Path>) -> Result<PathBuf, Error> {
    if location.is_absolute() {
        return Ok(location.to_path_buf());
    }
    let shown = location.display();
    let host = host.ok_or_else(|| {
        refusal(format!(
            "relative reference location {shown} is resolved from the workspace's main checkout, which an opaque Tree lacks; use the disk gate or an absolute location"
        ))
    })?;
    let repository = raf_source::git::Repository::discover(host)
        .map_err(|error| refusal(format!("relative reference location {shown}: {error}")))?
        .ok_or_else(|| {
            refusal(format!(
                "relative reference location {shown} is resolved from the main checkout of the workspace's Git repository, and {} is outside a Git repository; use an absolute location",
                host.display()
            ))
        })?;
    let main = repository
        .main_work_dir()
        .map_err(|error| refusal(format!("relative reference location {shown}: {error}")))?
        .ok_or_else(|| {
            refusal(format!(
                "relative reference location {shown} needs a main checkout, and the workspace's main repository is bare; use an absolute location"
            ))
        })?;
    Ok(main.join(location))
}

/// The commit string that the dotted `key` names in the TOML pin file.
fn pin(
    consumer: &dyn Tree,
    file: &RelPath,
    key: &str,
    options: &LoadOptions,
) -> Result<String, Error> {
    let text = consumer
        .read(file, options.max_file_bytes)
        .map_err(|source| Error::Tree {
            file: file.to_string(),
            source,
        })?;
    let failed = |message: String| refusal(format!("pin file {file} key {key:?}: {message}"));
    let profile = Profile::new(
        "raf-reference-pin",
        1,
        Vocabulary::standard().map_err(|error| failed(error.to_string()))?,
        Bounds {
            max_source_bytes: options.max_file_bytes,
            max_depth: 32,
            ..Bounds::standard()
        },
    )
    .map_err(|error| failed(error.to_string()))?;
    let document = purrdf_toml::analyze(
        SourceDocument {
            id: "urn:raf:reference-pin",
            bytes: text.as_bytes(),
        },
        &profile,
    )
    .map_err(|error| failed(error.to_string()))?;
    let nodes = document.nodes();
    let mut at = 0;
    for segment in key.split('.') {
        at = *nodes[at]
            .children
            .iter()
            .find(|child| {
                nodes[**child]
                    .key
                    .as_ref()
                    .is_some_and(|name| name.decoded == segment)
            })
            .ok_or_else(|| failed("the key is absent".into()))?;
    }
    match &nodes[at].kind {
        Kind::Scalar(Scalar::String(commit)) => Ok(commit.clone()),
        _ => Err(failed("the value must be a commit ID string".into())),
    }
}
