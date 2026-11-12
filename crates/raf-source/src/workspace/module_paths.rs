// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Module `path` attributes that lead a target outside its own package.

use crate::modtree::{PathSite, Reach};
use crate::path::RelPath;
use crate::target::PackageId;

use super::Workspace;

/// An arrival through a module `path` attribute at a file that the package of
/// the arriving target does not own.
///
/// The struct is `#[non_exhaustive]`: adding a field is not a breaking change,
/// so a pattern names the fields it reads and ends in `..`.
///
/// ```compile_fail,E0638
/// # fn names(found: &raf_source::ForeignModulePath<'_>) {
/// let raf_source::ForeignModulePath { file, reach, site, package, owner } = found;
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct ForeignModulePath<'a> {
    /// The reached file's path, as the workspace keeps it.
    pub file: &'a RelPath,
    /// The arrival, whose target followed the attribute.
    pub reach: &'a Reach,
    /// The attribute of that arrival.
    pub site: &'a PathSite,
    /// The package of the target that followed the attribute.
    pub package: PackageId,
    /// The package whose directory holds the resolved spelling, the deepest
    /// one when directories nest; `None` when no loaded package holds it.
    pub owner: Option<PackageId>,
}

impl Workspace {
    /// Every arrival through a module `path` attribute, plain or inside
    /// `cfg_attr`, whose resolved spelling the arriving target's package does
    /// not own, in the order of [`Workspace::files`] and then of each file's
    /// reaches.
    ///
    /// A package owns a spelling when its directory is the deepest package
    /// directory that lexically holds it in the same source, so a package
    /// nested inside another one owns its own directory. A spelling that
    /// keeps a `..` below the package directory, which the walk keeps only
    /// after a symlink or other non-directory entry, is not owned by that
    /// package. Every target kind is covered, since each target's module
    /// tree is walked from its root. Structural loads follow every `cfg_attr`
    /// alternative; configured loads follow only the active ones.
    #[must_use]
    pub fn foreign_module_paths(&self) -> Vec<ForeignModulePath<'_>> {
        let mut found = Vec::new();
        for file in self.files.values() {
            for reach in &file.reached {
                let Some(site) = &reach.path_attribute else {
                    continue;
                };
                // INVARIANT: every reach names a target of the same load.
                let target = &self.targets[reach.target.index()];
                let owner = self.owning_package(&site.resolved);
                if owner != Some(target.package_id) {
                    found.push(ForeignModulePath {
                        file: &file.path,
                        reach,
                        site,
                        package: target.package_id,
                        owner,
                    });
                }
            }
        }
        found
    }

    /// The package with the deepest directory that lexically holds `path`.
    fn owning_package(&self, path: &RelPath) -> Option<PackageId> {
        self.package_ids()
            .filter(|id| holds(&self.packages[id.index()].dir, path))
            .max_by_key(|id| self.packages[id.index()].dir.segments().count())
    }
}

/// True when `path` lies below `dir` in the same source, without a `..`
/// segment after the directory's own segments.
fn holds(dir: &RelPath, path: &RelPath) -> bool {
    if dir.authority() != path.authority() {
        return false;
    }
    let mut rest = path.segments();
    dir.segments().all(|segment| rest.next() == Some(segment))
        && rest.all(|segment| segment != "..")
}
