// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The loaded census: the units of a workspace and the context of its files.

use std::collections::BTreeSet;

use raf_source::{LoadOptions, TargetKind, Tree, Workspace};

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
    options: CensusOptions,
    units: Vec<Unit>,
    files: Vec<FileCtx>,
    grouping: Grouping,
}

impl Census {
    /// Loads the workspace under `roots` of `tree` and collects its units.
    ///
    /// `options.max_file_bytes` is the cap on one source file: a larger file
    /// is a load error that names it.
    ///
    /// # Errors
    ///
    /// [`Error::Load`] when the workspace does not load, and [`Error::Source`]
    /// when a body or an initialiser does not lex or the source index is
    /// inconsistent. Nothing falls back to a partial result.
    pub fn load(tree: &impl Tree, roots: &[&str], options: &CensusOptions) -> Result<Self, Error> {
        let mut load = LoadOptions::default();
        load.max_file_bytes = options.max_file_bytes;
        let workspace = Workspace::load_with(tree, roots, &load)?;
        let units = units::collect(&workspace, options)?;
        let grouping = group::group(&units);
        let files = workspace
            .files()
            .map(|file| {
                let target_kinds = file
                    .reached
                    .iter()
                    .map(|reach| {
                        workspace
                            .target(reach.target)
                            .map(|target| target.kind)
                            .ok_or_else(|| Error::Source {
                                file: file.path.to_string(),
                                line: 1,
                                reason: "a target that reaches the file does not resolve"
                                    .to_owned(),
                            })
                    })
                    .collect::<Result<_, _>>()?;
                Ok(FileCtx {
                    package: file.package.clone(),
                    path: file.path.to_string(),
                    shipping: file.shipping,
                    target_kinds,
                })
            })
            .collect::<Result<_, Error>>()?;
        Ok(Self {
            options: options.clone(),
            units,
            files,
            grouping,
        })
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
