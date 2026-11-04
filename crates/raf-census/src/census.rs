// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The loaded census: the units of a workspace and the context of its files.

use std::collections::BTreeSet;

use purrdf_lex::allocation::{Memory, OrderedTable, Resident};

use raf_source::{LoadOptions, PackageId, RelPath, TargetKind, Tree, Workspace};

use crate::group::{self, Group, GroupKind, Grouping, HexTable, SharedConstant};
use crate::units::{self, Unit};
use crate::{CensusOptions, Error};

/// What a rule needs to know about the file it checks.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FileCtx {
    /// The package of the first target that reaches the file.
    pub package: String,
    /// The path, relative to the workspace root.
    pub path: String,
    /// True when some target that ships reaches the file other than through
    /// test code.
    pub shipping: bool,
    /// The kinds of the targets whose module trees reach the file (a library,
    /// a test, an example, ...), by any reach, test code included.
    pub target_kinds: BTreeSet<TargetKind>,
}

/// The units and files of one workspace, under the options of one run.
#[derive(Debug)]
pub struct Census {
    pub(crate) reference_raw_units: Vec<Unit>,
    pub(crate) reference_units: Vec<crate::IndexUnit>,
    pub(crate) reference_sources: Vec<crate::ReferenceSource>,
    pub(crate) reference_witnesses: Vec<crate::ReferenceWitness>,
    options: CensusOptions,
    units: Vec<Unit>,
    files: Vec<FileCtx>,
    unexpanded: Vec<raf_source::Unexpanded>,
    unexpanded_targets: OrderedTable<raf_source::TargetId, raf_source::Target>,
    unexpanded_packages: OrderedTable<PackageId, raf_source::Package>,
    source_files: OrderedTable<RelPath, usize>,
    grouping: Grouping,
}

impl Census {
    /// Loads the workspace under `roots` of `tree` and collects its units.
    ///
    /// `options.max_file_bytes` is the cap on one source file: a larger file
    /// is a load error that names it.
    ///
    /// The tree is read on the caller's thread and need not be Send or Sync.
    /// Unit collection borrows the loaded workspace on a fresh owned thread;
    /// only plain unit data returns. Caller-owned proc-macro spans remain valid,
    /// and the worker's thread-local span storage ends with that worker.
    ///
    /// # Errors
    ///
    /// [`Error::Load`] when the workspace does not load, and [`Error::Source`]
    /// when a body or an initialiser does not lex or the source index is
    /// inconsistent. Nothing falls back to a partial result.
    /// [`Error::CollectionSpawn`] when the collection worker cannot start.
    /// A worker panic resumes on the caller with its original payload.
    pub fn load(tree: &impl Tree, roots: &[&str], options: &CensusOptions) -> Result<Self, Error> {
        let mut load = LoadOptions::default();
        load.max_file_bytes = options.max_file_bytes;
        let workspace = Workspace::load_with(tree, roots, &load)?;
        Self::from_workspace(&workspace, options)
    }

    pub(crate) fn from_workspace(
        workspace: &Workspace,
        options: &CensusOptions,
    ) -> Result<Self, Error> {
        Self::from_packages(workspace, options, &workspace.package_ids().collect())
    }

    pub(crate) fn from_packages(
        workspace: &Workspace,
        options: &CensusOptions,
        packages: &BTreeSet<PackageId>,
    ) -> Result<Self, Error> {
        let units = units::collect_packages(workspace, options, packages)?;
        let mut unexpanded = Vec::new();
        let mut unexpanded_targets = OrderedTable::default();
        let mut unexpanded_packages = OrderedTable::default();
        let mut resident = Resident;
        let mut memory = Memory::new(&mut resident);
        for occurrence in workspace.unexpanded() {
            let target = workspace
                .target(occurrence.target)
                .ok_or_else(|| Error::Source {
                    file: occurrence.file.to_string(),
                    line: occurrence.span.lines.start,
                    reason: "an unexpanded occurrence has no owning target".into(),
                })?;
            if packages.contains(&target.package_id) {
                let package =
                    workspace
                        .package(target.package_id)
                        .ok_or_else(|| Error::Source {
                            file: occurrence.file.to_string(),
                            line: occurrence.span.lines.start,
                            reason: "an unexpanded occurrence's target has no owning package"
                                .into(),
                        })?;
                if unexpanded_targets.get(&occurrence.target).is_none() {
                    unexpanded_targets
                        .insert(occurrence.target, target.clone(), &mut memory)
                        .map_err(|source| Error::SourceIndexStorage { source })?;
                }
                if unexpanded_packages.get(&target.package_id).is_none() {
                    unexpanded_packages
                        .insert(target.package_id, package.clone(), &mut memory)
                        .map_err(|source| Error::SourceIndexStorage { source })?;
                }
                unexpanded.push(occurrence.clone());
            }
        }
        let grouping = group::group(&units);
        let mut files = Vec::new();
        let mut source_files = OrderedTable::default();
        for file in workspace.files() {
            let mut selected = Vec::new();
            for reach in &file.reached {
                let target = workspace
                    .target(reach.target)
                    .ok_or_else(|| Error::Source {
                        file: file.path.to_string(),
                        line: 1,
                        reason: "a target that reaches the file does not resolve".into(),
                    })?;
                if packages.contains(&target.package_id) {
                    selected.push((reach, target));
                }
            }
            let Some((_, first)) = selected.first() else {
                continue;
            };
            source_files
                .insert(file.path.clone(), files.len(), &mut memory)
                .map_err(|source| Error::SourceIndexStorage { source })?;
            files.push(FileCtx {
                package: first.package.clone(),
                path: file.path.to_string(),
                shipping: selected
                    .iter()
                    .any(|(reach, target)| target.shipping && !reach.test),
                target_kinds: selected.iter().map(|(_, target)| target.kind).collect(),
            });
        }
        Ok(Self {
            reference_raw_units: Vec::new(),
            reference_units: Vec::new(),
            reference_sources: Vec::new(),
            reference_witnesses: Vec::new(),
            options: options.clone(),
            units,
            files,
            unexpanded,
            unexpanded_targets,
            unexpanded_packages,
            source_files,
            grouping,
        })
    }

    pub(crate) fn source_context(&self, path: &RelPath) -> Option<&FileCtx> {
        self.source_files.get(path).map(|index| &self.files[*index])
    }

    /// The options of the run.
    #[must_use]
    pub const fn options(&self) -> &CensusOptions {
        &self.options
    }

    /// Every unit, in the order of file and position.
    #[must_use]
    pub fn units(&self) -> &[Unit] {
        &self.units
    }

    /// Unexpanded module macro occurrences in the selected packages, in Source
    /// order. They are syntax evidence, separate from fingerprinted units, and
    /// are not included in the canonical RDF unit index.
    #[must_use]
    pub fn unexpanded(&self) -> &[raf_source::Unexpanded] {
        &self.unexpanded
    }

    /// Original target metadata for an unexpanded occurrence owned by this
    /// Census. IDs belong to this load; targets without occurrences are absent.
    #[must_use]
    pub fn unexpanded_target(&self, id: raf_source::TargetId) -> Option<&raf_source::Target> {
        self.unexpanded_targets.get(&id)
    }

    /// Original package metadata for a retained occurrence target. Package IDs
    /// belong to this load; packages without occurrences are absent.
    #[must_use]
    pub fn unexpanded_package(&self, id: PackageId) -> Option<&raf_source::Package> {
        self.unexpanded_packages.get(&id)
    }

    /// The first unit with `symbol`, in the order of file and position. Two
    /// units share a symbol only when `cfg` alternatives define one path
    /// twice; [`Census::units_of`] lists them all.
    #[must_use]
    pub fn unit(&self, symbol: &str) -> Option<&Unit> {
        self.units.iter().find(|unit| unit.symbol == symbol)
    }

    /// Every unit with `symbol`.
    pub fn units_of<'a>(&'a self, symbol: &'a str) -> impl Iterator<Item = &'a Unit> {
        self.units.iter().filter(move |unit| unit.symbol == symbol)
    }

    /// Every file of the workspace, in path order.
    #[must_use]
    pub fn files(&self) -> &[FileCtx] {
        &self.files
    }

    /// The context of the file at `path`.
    #[must_use]
    pub fn file(&self, path: &str) -> Option<&FileCtx> {
        self.files.iter().find(|file| file.path == path)
    }

    /// Every group of two or more units, body and shim, ordered by member
    /// count descending, then tokens descending, then fingerprint ascending.
    #[must_use]
    pub fn groups(&self) -> &[Group] {
        &self.grouping.groups
    }

    /// The body groups, in the order of [`Census::groups`].
    pub fn body_groups(&self) -> impl Iterator<Item = &Group> {
        self.groups_of(GroupKind::Body)
    }

    /// The shim groups, in the order of [`Census::groups`].
    pub fn shim_groups(&self) -> impl Iterator<Item = &Group> {
        self.groups_of(GroupKind::Shim)
    }

    fn groups_of(&self, kind: GroupKind) -> impl Iterator<Item = &Group> {
        self.grouping.groups.iter().filter(move |g| g.kind == kind)
    }

    /// The units that hold a hexadecimal digit table, by unit and case.
    #[must_use]
    pub fn hex_tables(&self) -> &[HexTable] {
        &self.grouping.hex_tables
    }

    /// The integer constants above `0xffff` that units of two or more
    /// packages hold, by value ascending.
    #[must_use]
    pub fn shared_constants(&self) -> &[SharedConstant] {
        &self.grouping.shared_constants
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raf_source::MemoryTree;

    #[test]
    fn unexpanded_inventory_retains_only_selected_packages_and_stays_separate_from_units() {
        let tree = MemoryTree::from([
            ("Cargo.toml", "[workspace]\nmembers=['one','two']"),
            ("one/Cargo.toml", "[package]\nname='one'\nversion='0.1.0'"),
            ("one/src/lib.rs", "make_one!(Generated);\npub fn kept() {}"),
            ("two/Cargo.toml", "[package]\nname='two'\nversion='0.1.0'"),
            ("two/src/lib.rs", "make_two!(Other);"),
        ]);
        let options = CensusOptions::default();
        let all = Census::load(&tree, &["."], &options).expect("public load");
        assert_eq!(all.unexpanded().len(), 2);
        for occurrence in all.unexpanded() {
            let target = all
                .unexpanded_target(occurrence.target)
                .expect("retained original target");
            let package = all
                .unexpanded_package(target.package_id)
                .expect("retained original package");
            assert_eq!(target.package, package.name);
            assert_eq!(target.kind, TargetKind::Lib);
            assert_eq!(
                package.manifest.as_str(),
                format!("{}/Cargo.toml", package.name)
            );
        }
        assert!(
            all.units()
                .iter()
                .all(|unit| !unit.symbol.contains("Generated"))
        );
        let source = Workspace::load(&tree, &["."]).expect("original source");
        let package = source
            .package_ids()
            .find(|id| source.package(*id).expect("package").name == "one")
            .expect("selected package");
        let selected = Census::from_packages(&source, &options, &BTreeSet::from([package]))
            .expect("actual package-selected collection");
        assert_eq!(selected.unexpanded().len(), 1);
        let target = selected
            .unexpanded_target(selected.unexpanded()[0].target)
            .expect("selected owner");
        assert_eq!(
            selected
                .unexpanded_package(target.package_id)
                .expect("package owner")
                .name,
            "one"
        );
        let omitted = source
            .target_ids()
            .find(|id| source.target(*id).expect("target").package == "two")
            .expect("omitted owner");
        assert!(selected.unexpanded_target(omitted).is_none());
        assert!(
            selected
                .unexpanded_package(source.target(omitted).expect("target").package_id)
                .is_none()
        );
        assert_eq!(selected.unexpanded()[0].invoked.as_str(), "make_one");
        assert_eq!(selected.unexpanded()[0].span.lines, 1..2);
        assert_eq!(
            selected.unexpanded()[0].identifiers[0].as_str(),
            "Generated"
        );
        assert!(
            selected
                .units()
                .iter()
                .all(|unit| unit.symbol.starts_with("one::"))
        );
    }
}
