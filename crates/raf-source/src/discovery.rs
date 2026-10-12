// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Package discovery under explicit roots.
//!
//! The walk recurses through directories that are not packages. It stops at a
//! `[package]` manifest that has no `[workspace]` table, so a fixture package
//! nested below another is not found. It skips directories named `target` and
//! dot-directories, and a symlinked directory is an error.

use std::collections::BTreeMap;

use crate::error::{LoadError, LoadErrorKind, Origin, TargetProblem, TreeError};
use crate::manifest::{self, Manifest};
use crate::path::RelPath;
use crate::target::Package;
use crate::tree::{EntryKind, Tree};

/// A package with its parsed manifest.
pub(crate) struct Found {
    pub(crate) package: Package,
    pub(crate) manifest: Manifest,
}

struct Walk<'a> {
    tree: &'a dyn Tree,
    max_file_bytes: u64,
    found: BTreeMap<RelPath, Found>,
}

/// What discovery found.
pub(crate) struct Discovered {
    pub(crate) packages: Vec<Found>,
}

fn skipped(name: &str) -> bool {
    name == "target" || name.starts_with('.')
}

impl Walk<'_> {
    fn root(&mut self, root: &RelPath) -> Result<(), LoadError> {
        let origin = Origin::Discovery;
        match self
            .tree
            .entry(root)
            .map_err(|error| LoadError::tree(root, None, &origin, error))?
        {
            Some(EntryKind::Dir) => self.visit(root),
            Some(EntryKind::Symlink) => {
                Err(LoadError::new(root, None, LoadErrorKind::DirSymlink).reached_by(&origin))
            }
            Some(EntryKind::File | EntryKind::Other) => Err(LoadError::tree(
                root,
                None,
                &origin,
                TreeError::NotRegular {
                    path: root.to_string(),
                },
            )),
            None => Err(LoadError::tree(
                root,
                None,
                &origin,
                TreeError::NotFound {
                    path: root.to_string(),
                },
            )),
        }
    }

    /// Read the manifest of `dir`, if it has one. Returns whether the walk goes
    /// on into the directory's children.
    fn manifest(&mut self, dir: &RelPath) -> Result<bool, LoadError> {
        let origin = Origin::Discovery;
        let path = dir
            .join("Cargo.toml")
            .map_err(|error| LoadError::tree(dir, None, &origin, error.into()))?;
        let present = self
            .tree
            .entry(&path)
            .map_err(|error| LoadError::tree(&path, None, &origin, error))?;
        if present.is_none() {
            return Ok(true);
        }
        let text = self
            .tree
            .read(&path, self.max_file_bytes)
            .map_err(|error| LoadError::tree(&path, None, &origin, error))?;
        let parsed = manifest::parse(&text).map_err(|problem| {
            let origin = problem
                .key
                .map_or(Origin::Discovery, |key| Origin::ManifestKey { key });
            LoadError::new(
                &path,
                None,
                LoadErrorKind::Manifest {
                    message: problem.message,
                },
            )
            .reached_by(&origin)
        })?;
        let descend = parsed.package.is_none() || parsed.has_workspace;
        if let Some(package) = &parsed.package {
            for other in self.found.values() {
                if other.package.name == package.name && other.package.manifest != path {
                    return Err(LoadError::new(
                        &path,
                        None,
                        LoadErrorKind::Target(Box::new(TargetProblem::DuplicatePackage {
                            name: package.name.clone(),
                            with: other.package.manifest.clone(),
                        })),
                    )
                    .reached_by(&Origin::ManifestKey {
                        key: "package.name".to_owned(),
                    }));
                }
            }
            let entry = Found {
                package: Package {
                    name: package.name.clone(),
                    manifest: path.clone(),
                    dir: dir.clone(),
                },
                manifest: parsed,
            };
            // The same manifest reached from two overlapping roots is one package.
            self.found.insert(path, entry);
        }
        Ok(descend)
    }

    fn visit(&mut self, dir: &RelPath) -> Result<(), LoadError> {
        let origin = Origin::Discovery;
        if !self.manifest(dir)? {
            return Ok(());
        }
        let entries = self
            .tree
            .list(dir)
            .map_err(|error| LoadError::tree(dir, None, &origin, error))?;
        for entry in entries {
            if skipped(&entry.name) {
                continue;
            }
            let child = dir
                .join(&entry.name)
                .map_err(|error| LoadError::tree(dir, None, &origin, error.into()))?;
            match entry.kind {
                EntryKind::Dir => self.visit(&child)?,
                EntryKind::Symlink => self.symlink(&child)?,
                EntryKind::File | EntryKind::Other => {}
            }
        }
        Ok(())
    }

    /// A symlink in a directory the walk enters, classified by what it
    /// resolves to: a regular file is left alone; a directory is a
    /// [`LoadErrorKind::DirSymlink`]; anything else is an error of the tree: a
    /// target outside the root, a loop, a dangling link or a special file.
    fn symlink(&self, path: &RelPath) -> Result<(), LoadError> {
        let origin = Origin::Discovery;
        let fail = |error| LoadError::tree(path, None, &origin, error);
        match self.tree.read(path, 0) {
            Ok(_) | Err(TreeError::TooLarge { .. }) => Ok(()),
            Err(TreeError::NotRegular { .. }) => {
                // Only a directory has a parent that `link/..` can name.
                let parent = path.join("..").map_err(|error| fail(error.into()))?;
                match self.tree.entry(&parent).map_err(fail)? {
                    Some(EntryKind::Dir) => {
                        Err(LoadError::new(path, None, LoadErrorKind::DirSymlink)
                            .reached_by(&origin))
                    }
                    Some(EntryKind::File | EntryKind::Symlink | EntryKind::Other) | None => {
                        Err(fail(TreeError::NotRegular {
                            path: path.to_string(),
                        }))
                    }
                }
            }
            Err(error) => Err(fail(error)),
        }
    }
}

/// Find every package under `roots`, sorted by manifest path.
///
/// # Errors
///
/// Returns the first [`LoadError`]: a missing or symlinked root, an unreadable
/// or invalid manifest, a symlinked directory, or two packages with one name.
pub(crate) fn discover(
    tree: &dyn Tree,
    roots: &[RelPath],
    max_file_bytes: u64,
) -> Result<Discovered, LoadError> {
    let mut walk = Walk {
        tree,
        max_file_bytes,
        found: BTreeMap::new(),
    };
    for root in roots {
        walk.root(root)?;
    }
    Ok(Discovered {
        packages: walk.found.into_values().collect(),
    })
}
