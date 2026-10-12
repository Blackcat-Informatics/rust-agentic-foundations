// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The loaded workspace: packages, targets and the files of every module tree.

use std::collections::BTreeMap;
use std::rc::Rc;

use crate::discovery::discover;
use crate::error::{LoadError, Origin};
use crate::facts::FileFacts;
use crate::modtree::{Reach, TargetId, Walker};
use crate::options::LoadOptions;
use crate::path::RelPath;
use crate::target::{Context, Package, Target, check_crate_paths, derive_targets};
use crate::tree::Tree;

/// A source file of the workspace, parsed once.
#[derive(Debug)]
pub struct SourceFile {
    /// The path, relative to the tree root.
    pub path: RelPath,
    /// The exact text that was parsed.
    pub text: String,
    /// The facts read from it.
    pub facts: FileFacts,
    /// Every way a module tree reaches the file, in the order the walk
    /// found them.
    pub reached: Vec<Reach>,
}

impl SourceFile {
    /// True when every way of reaching the file goes through test code. The
    /// items of such a file are test code whatever their own attributes say.
    #[must_use]
    pub fn reached_only_through_tests(&self) -> bool {
        self.reached.iter().all(|reach| reach.test)
    }
}

/// The packages, targets and source files found under a set of roots.
#[derive(Debug)]
pub struct Workspace {
    packages: Vec<Package>,
    targets: Vec<Target>,
    files: BTreeMap<RelPath, SourceFile>,
}

impl Workspace {
    /// Load the packages under `roots` with the default [`LoadOptions`].
    ///
    /// A root is a directory path relative to the tree root; `"."` is the tree
    /// root itself. There are no default roots: each caller names its own.
    ///
    /// # Errors
    ///
    /// Returns the first [`LoadError`] the walk meets; see
    /// [`Workspace::load_with`].
    pub fn load(tree: &dyn Tree, roots: &[&str]) -> Result<Self, LoadError> {
        Self::load_with(tree, roots, &LoadOptions::default())
    }

    /// Load the packages under `roots`.
    ///
    /// Discovery finds every package, then each package's targets are derived
    /// and each target's module tree is followed. The walk stops at the first
    /// error, which names the file, the line when there is one, and the
    /// module, attribute or manifest key that led to it.
    ///
    /// # Errors
    ///
    /// Returns a [`LoadError`] for a root that is missing or not a directory, a
    /// manifest that cannot be read, a target problem, an unresolved or
    /// ambiguous `mod`, a path that leaves the tree, a module cycle, a file
    /// over `max_file_bytes` or a budget, or a syntax error.
    pub fn load_with(
        tree: &dyn Tree,
        roots: &[&str],
        options: &LoadOptions,
    ) -> Result<Self, LoadError> {
        let mut root_paths = Vec::new();
        for root in roots {
            let path = RelPath::new(root).map_err(|error| {
                LoadError::tree(
                    &RelPath::root(),
                    None,
                    &Origin::Discovery,
                    crate::error::TreeError::from(error),
                )
            })?;
            root_paths.push(path);
        }
        let found = discover(tree, &root_paths, options.max_file_bytes)?;
        let mut packages = Vec::new();
        let mut derived = Vec::new();
        for entry in &found {
            let Some(declared) = &entry.manifest.package else {
                continue;
            };
            let context = Context {
                tree,
                manifest_path: &entry.package.manifest,
                dir: &entry.package.dir,
                manifest: &entry.manifest,
                package: declared,
            };
            for target in derive_targets(&context)? {
                derived.push((&entry.package.manifest, target));
            }
            packages.push(entry.package.clone());
        }
        check_crate_paths(&derived)?;
        let (targets, origins): (Vec<_>, Vec<_>) = derived
            .into_iter()
            .map(|(_, item)| (item.target, item.origin))
            .unzip();
        let mut walker = Walker::new(tree, options);
        for (index, (target, origin)) in targets.iter().zip(&origins).enumerate() {
            walker.root(TargetId(index), &target.root, &target.crate_path, origin)?;
        }
        let files = walker
            .finish()
            .into_iter()
            .map(|(path, visited)| {
                let parsed = Rc::unwrap_or_clone(visited.parsed);
                let file = SourceFile {
                    path: path.clone(),
                    text: parsed.text,
                    facts: parsed.facts,
                    reached: visited.reached,
                };
                (path, file)
            })
            .collect();
        Ok(Self {
            packages,
            targets,
            files,
        })
    }

    /// The packages, sorted by manifest path.
    #[must_use]
    pub fn packages(&self) -> &[Package] {
        &self.packages
    }

    /// The targets, grouped by package: library, binaries, tests, benches,
    /// examples and build script of each.
    pub fn targets(&self) -> impl Iterator<Item = &Target> {
        self.targets.iter()
    }

    /// A target by id.
    #[must_use]
    pub fn target(&self, id: TargetId) -> Option<&Target> {
        self.targets.get(id.index())
    }

    /// The ids of the targets, in the order of [`Workspace::targets`].
    pub fn target_ids(&self) -> impl Iterator<Item = TargetId> {
        (0..self.targets.len()).map(TargetId)
    }

    /// Every source file, once, sorted by path.
    pub fn files(&self) -> impl Iterator<Item = &SourceFile> {
        self.files.values()
    }

    /// One source file.
    #[must_use]
    pub fn file(&self, path: &RelPath) -> Option<&SourceFile> {
        self.files.get(path)
    }

    /// The exact text that was parsed for `path`.
    #[must_use]
    pub fn source(&self, path: &RelPath) -> Option<&str> {
        self.files.get(path).map(|file| file.text.as_str())
    }
}
