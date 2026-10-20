// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The loaded census: the units of a workspace and the context of its files.

use raf_source::{LoadOptions, Tree, Workspace};

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
}

/// The units and files of one workspace, under the options of one run.
#[derive(Debug)]
pub struct Census {
    options: CensusOptions,
    units: Vec<Unit>,
    files: Vec<FileCtx>,
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
    /// when a body does not lex or a signature or an initialiser does not
    /// parse. Nothing falls back to a partial result.
    pub fn load(tree: &impl Tree, roots: &[&str], options: &CensusOptions) -> Result<Self, Error> {
        let mut load = LoadOptions::default();
        load.max_file_bytes = options.max_file_bytes;
        let workspace = Workspace::load_with(tree, roots, &load)?;
        let units = units::collect(&workspace, options)?;
        let files = workspace
            .files()
            .map(|file| FileCtx {
                package: file.package.clone(),
                path: file.path.to_string(),
                shipping: file.shipping,
            })
            .collect();
        Ok(Self {
            options: options.clone(),
            units,
            files,
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
}
