// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The required containment of module `path` attributes in their package.

use std::collections::{BTreeMap, BTreeSet};

use raf_source::{PackageId, Workspace};

/// One finding for each distinct module `path` attribute, followed by a
/// target of a selected package, whose resolved file that package does not
/// own, in the order of the declaring file, line and resolved file.
pub(super) fn findings(workspace: &Workspace, selected: &BTreeSet<PackageId>) -> Vec<String> {
    let name = |id: PackageId| workspace.package(id).map_or("", |package| &package.name);
    let mut found = BTreeMap::new();
    for foreign in workspace.foreign_module_paths() {
        if !selected.contains(&foreign.package) {
            continue;
        }
        let site = foreign.site;
        let package = name(foreign.package);
        let owned = foreign.owner.map_or_else(
            || "outside every loaded package".to_owned(),
            |owner| format!("in package `{}`", name(owner)),
        );
        found.insert(
            (site.file.clone(), site.line, site.resolved.clone(), package),
            format!(
                "module path attribute: {}:{}: module `{}` of package `{package}` resolves to {}, {owned}; move the file into the directory of `{package}` or use it through a dependency",
                site.file, site.line, foreign.reach.module_path, site.resolved
            ),
        );
    }
    found.into_values().collect()
}
