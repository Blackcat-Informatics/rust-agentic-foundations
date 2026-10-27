// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Package discovery under explicit roots.
//!
//! The walk descends through directories that are not packages, with an
//! explicit stack and at most [`Budget::DirectoryDepth`] levels below a root.
//! It stops at a `[package]` manifest that has no `[workspace]` table, so a
//! fixture package nested below another is not found. It skips directories
//! named `target` and dot-directories, and a symlinked directory is an error.

use std::collections::BTreeMap;

use purrdf_lex::allocation::{Memory, OrderedTable};
use purrdf_lex::walk::WorkList;

#[cfg(doc)]
use crate::budget::Budget;
use crate::error::{LoadError, LoadErrorKind, Origin, TargetProblem, TreeError};
use crate::manifest::{self, Manifest, PackageDecl};
use crate::meter::{ItemAdmission, Meter, push_item_work};
use crate::path::{RelPath, SourceAuthorityId};
use crate::target::Package;
use crate::tree::{EntryKind, Tree};

/// A package with its parsed manifest. `declared` is the manifest's `[package]`
/// table, moved out of `manifest` (whose `package` is `None` here).
pub(crate) struct Found {
    pub(crate) package: Package,
    pub(crate) declared: PackageDecl,
    pub(crate) manifest: Manifest,
}

struct Walk<'a> {
    tree: &'a dyn Tree,
    meter: &'a Meter,
    found: BTreeMap<RelPath, Found>,
    /// The manifest of each package name found so far.
    names: OrderedTable<(SourceAuthorityId, String), RelPath>,
    /// Requested layouts already retained under the names table's native account.
    name_bytes: usize,
}

/// A step of the walk: a directory and its depth below the root, or a
/// symlink to classify.
enum Step {
    Dir(RelPath, usize),
    Link(RelPath),
}

/// What discovery found.
pub(crate) struct Discovered {
    pub(crate) packages: Vec<Found>,
}

fn skipped(name: &str) -> bool {
    name == "target" || name.starts_with('.')
}

impl Walk<'_> {
    /// Retain one source-local package name through the original item allowance.
    fn remember_name(&mut self, name: &str, path: &RelPath) -> Result<(), LoadError> {
        let mut admission = ItemAdmission::resume(self.meter, path, self.name_bytes);
        let result = {
            let mut memory = Memory::resume(&mut admission, self.name_bytes);
            let mut owned = String::new();
            let result = memory.push_str(&mut owned, name).and_then(|()| {
                self.names
                    .insert((path.authority(), owned), path.clone(), &mut memory)
            });
            self.name_bytes = memory.admitted_bytes();
            result
        };
        if result.map_err(|error| admission.error(error))?.is_some() {
            return Err(LoadError::new(
                path,
                None,
                LoadErrorKind::Internal {
                    what: "a source-local package name changed during insertion".to_owned(),
                },
            ));
        }
        Ok(())
    }

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
            .read(&path, self.meter.options().max_file_bytes)
            .map_err(|error| LoadError::tree(&path, None, &origin, error))?;
        self.meter
            .read(&path, &text)
            .map_err(|error| error.reached_by(&origin))?;
        let mut parsed = manifest::parse(&text).map_err(|problem| {
            let origin = problem
                .key
                .map_or(Origin::Discovery, |key| Origin::ManifestKey { key });
            LoadError::new(
                &path,
                None,
                LoadErrorKind::Manifest {
                    message: problem.message,
                    source: problem.source,
                },
            )
            .reached_by(&origin)
        })?;
        let descend = parsed.package.is_none() || parsed.has_workspace;
        if let Some(package) = parsed.package.take() {
            match self.names.get_by(|(authority, name)| {
                (path.authority(), package.name.as_str()).cmp(&(*authority, name.as_str()))
            }) {
                Some(other) if *other != path => {
                    return Err(LoadError::new(
                        &path,
                        None,
                        LoadErrorKind::Target(Box::new(TargetProblem::DuplicatePackage {
                            name: package.name.clone(),
                            with: other.clone(),
                        })),
                    )
                    .reached_by(&Origin::ManifestKey {
                        key: "package.name".to_owned(),
                    }));
                }
                Some(_) => {}
                None => {
                    self.remember_name(&package.name, &path)?;
                }
            }
            let entry = Found {
                package: Package {
                    name: package.name.clone(),
                    manifest: path.clone(),
                    dir: dir.clone(),
                },
                declared: package,
                manifest: parsed,
            };
            // The same manifest reached from two overlapping roots is one package.
            self.found.insert(path, entry);
        }
        Ok(descend)
    }

    /// Walk the directories below `root`, depth first and in name order, on
    /// an explicit stack.
    fn visit(&mut self, root: &RelPath) -> Result<(), LoadError> {
        let origin = Origin::Discovery;
        let mut stack = WorkList::<_, 8>::new();
        push_item_work(
            &mut stack,
            Step::Dir(root.clone(), 0),
            self.meter,
            root,
            None,
        )?;
        while let Some(step) = stack.pop() {
            let (dir, depth) = match step {
                Step::Dir(dir, depth) => (dir, depth),
                Step::Link(path) => {
                    self.symlink(&path)?;
                    continue;
                }
            };
            self.meter.directory(depth, &dir)?;
            if !self.manifest(&dir)? {
                continue;
            }
            let entries = self
                .tree
                .list(&dir)
                .map_err(|error| LoadError::tree(&dir, None, &origin, error))?;
            let mut steps = Vec::new();
            for entry in entries {
                if skipped(&entry.name) {
                    continue;
                }
                let child = dir
                    .join(&entry.name)
                    .map_err(|error| LoadError::tree(&dir, None, &origin, error.into()))?;
                match entry.kind {
                    EntryKind::Dir => steps.push(Step::Dir(child, depth + 1)),
                    EntryKind::Symlink => steps.push(Step::Link(child)),
                    EntryKind::File | EntryKind::Other => {}
                }
            }
            // Reversed, so the first entry in name order comes first.
            for step in steps.into_iter().rev() {
                push_item_work(&mut stack, step, self.meter, &dir, None)?;
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
/// or invalid manifest, a symlinked directory, two packages with one name, a
/// directory more than [`Budget::DirectoryDepth`] levels below its root, or a
/// load that passes [`Budget::TotalBytes`].
pub(crate) fn discover(
    tree: &dyn Tree,
    roots: &[RelPath],
    meter: &Meter,
) -> Result<Discovered, LoadError> {
    meter.items(
        size_of::<OrderedTable<(SourceAuthorityId, String), RelPath>>(),
        &RelPath::root(),
        None,
    )?;
    let mut walk = Walk {
        tree,
        meter,
        found: BTreeMap::new(),
        names: OrderedTable::default(),
        name_bytes: 0,
    };
    for root in roots {
        walk.root(root)?;
    }
    Ok(Discovered {
        packages: walk.found.into_values().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Budget, LoadOptions, MemoryTree};

    fn populate_names(limit: usize) -> Result<usize, LoadError> {
        let tree = MemoryTree::default();
        let meter = Meter::new(&LoadOptions {
            max_item_bytes: limit,
            ..LoadOptions::default()
        });
        let mut walk = Walk {
            tree: &tree,
            meter: &meter,
            found: BTreeMap::new(),
            names: OrderedTable::default(),
            name_bytes: 0,
        };
        let path = RelPath::new("Cargo.toml")
            .expect("manifest")
            .in_source(SourceAuthorityId(1));
        for index in 0..40 {
            walk.remember_name(&format!("package-{index:02}"), &path)?;
        }
        let spent = meter.usage().item_bytes;
        for index in 0..40 {
            let name = format!("package-{index:02}");
            assert_eq!(
                walk.names.get_by(|(authority, stored)| {
                    (path.authority(), name.as_str()).cmp(&(*authority, stored.as_str()))
                }),
                Some(&path)
            );
        }
        assert_eq!(meter.usage().item_bytes, spent);
        Ok(spent)
    }

    #[test]
    fn native_discovery_names_resume_retained_storage_and_refuse_the_original_grant() {
        let spent = populate_names(LoadOptions::default().max_item_bytes).expect("populated");
        assert_eq!(populate_names(spent).expect("exact allowance"), spent);
        let error = populate_names(spent - 1).expect_err("one byte too little");
        assert_eq!(error.file.authority(), SourceAuthorityId(1));
        assert!(matches!(
            error.kind,
            LoadErrorKind::Budget {
                budget: Budget::ItemBytes,
                ..
            }
        ));
    }
}
