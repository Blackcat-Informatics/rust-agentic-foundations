// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Workspace-root lookup over the bounded tree and membership model.

use crate::error::{LoadError, LoadErrorKind, Origin};
use crate::manifest::{self, Manifest};
use crate::members::members;
use crate::meter::Meter;
use crate::modtree::tidy;
use crate::{DEFAULT_MAX_FILE_BYTES, EntryKind, LoadOptions, RelPath, Tree};

fn finding(path: &RelPath, message: impl Into<String>) -> LoadError {
    LoadError::new(
        path,
        None,
        LoadErrorKind::Manifest {
            message: message.into(),
            source: None,
        },
    )
}

fn read(tree: &dyn Tree, dir: &RelPath) -> Result<Option<Manifest>, LoadError> {
    let path = dir.join("Cargo.toml").expect("a literal child");
    if tree
        .entry(&path)
        .map_err(|error| LoadError::tree(&path, None, &Origin::Discovery, error))?
        .is_none()
    {
        return Ok(None);
    }
    let text = tree
        .read(&path, DEFAULT_MAX_FILE_BYTES)
        .map_err(|error| LoadError::tree(&path, None, &Origin::Discovery, error))?;
    manifest::parse(&text).map(Some).map_err(|error| {
        let origin = error
            .key
            .map_or(Origin::Discovery, |key| Origin::ManifestKey { key });
        LoadError::new(
            &path,
            None,
            LoadErrorKind::Manifest {
                message: error.message,
                source: error.source,
            },
        )
        .reached_by(&origin)
    })
}

fn owns(
    tree: &dyn Tree,
    root: &RelPath,
    parsed: &Manifest,
    package: &RelPath,
) -> Result<bool, LoadError> {
    let path = root.join("Cargo.toml").expect("a literal child");
    let found = members(
        tree,
        &Meter::new(&LoadOptions::default()),
        root,
        &path,
        parsed,
    )?;
    if found.contains(package) {
        return Ok(true);
    }
    for member in found {
        if directory(tree, &member)? == *package {
            return Ok(true);
        }
    }
    Ok(false)
}

fn directory(tree: &dyn Tree, path: &RelPath) -> Result<RelPath, LoadError> {
    let path =
        tidy(tree, path).map_err(|error| LoadError::tree(path, None, &Origin::Discovery, error))?;
    let mut prefix = path.clone();
    loop {
        if prefix.segments().any(|segment| segment == "..")
            || tree
                .entry(&prefix)
                .map_err(|error| LoadError::tree(&prefix, None, &Origin::Discovery, error))?
                != Some(EntryKind::Dir)
        {
            return Err(finding(
                &prefix,
                "root discovery requires real directories without directory symlinks; canonicalize the starting directory first",
            ));
        }
        let Some(parent) = prefix.parent() else {
            break;
        };
        prefix = parent;
    }
    Ok(path)
}

/// Find the workspace root containing `start`, relative to the supplied tree.
///
/// The nearest `[workspace]` wins. A package belongs to it only when the
/// bounded member walk includes it (including transitive path dependencies).
/// An excluded package or a package without an ancestor workspace is its own
/// root. An unlisted package is refused rather than assigned to an unrelated
/// workspace. `package.workspace` selects an explicit workspace inside the
/// tree, whose membership must agree. No source files or processes are needed.
///
/// # Errors
///
/// Returns a [`LoadError`] for an invalid starting directory, missing manifest,
/// malformed or oversized manifest, inaccessible tree, conflicting workspace
/// declarations, unlisted package, or membership walk exceeding its bounds.
/// Discovery never walks above the caller's tree boundary.
/// Directory symlinks are refused because their lexical ancestors need not be
/// the ancestors of the directory they reach.
pub fn discover_root(tree: &dyn Tree, start: &RelPath) -> Result<RelPath, LoadError> {
    let start = directory(tree, start)?;
    let mut current = Some(start.clone());
    let mut package = None;
    while let Some(dir) = current {
        if let Some(parsed) = read(tree, &dir)? {
            if package.is_none() && parsed.package.is_some() {
                if let Some(target) = &parsed.package_workspace {
                    if parsed.has_workspace {
                        return Err(finding(
                            &dir,
                            "package.workspace conflicts with [workspace]",
                        ));
                    }
                    let joined = dir
                        .join(target)
                        .map_err(|error| finding(&dir, error.to_string()))?;
                    let root = directory(tree, &joined)?;
                    let workspace = read(tree, &root)?
                        .ok_or_else(|| finding(&root, "package.workspace has no Cargo.toml"))?;
                    if !workspace.has_workspace || !owns(tree, &root, &workspace, &dir)? {
                        return Err(finding(
                            &dir,
                            "package.workspace must name a workspace that includes this package",
                        ));
                    }
                    return Ok(root);
                }
                package = Some(dir.clone());
            }
            if parsed.has_workspace {
                let Some(package) = &package else {
                    return Ok(dir);
                };
                if owns(tree, &dir, &parsed, package)? {
                    return Ok(dir);
                }
                let parts: Vec<Vec<char>> = package
                    .segments()
                    .skip(dir.segments().count())
                    .map(|part| part.chars().collect())
                    .collect();
                for text in &parsed.exclude {
                    let pattern =
                        crate::glob::Pattern::parse(text).map_err(|error| finding(&dir, error))?;
                    if pattern.matches_prefix(&parts) {
                        return Ok(package.clone());
                    }
                }
                return Err(finding(
                    package,
                    "package is below a workspace but is not a member; add it to workspace.members, exclude it, or give it its own [workspace]",
                ));
            }
        }
        current = dir.parent();
    }
    package.ok_or_else(|| {
        finding(
            &start,
            "no Cargo.toml package or workspace found in this directory or its ancestors",
        )
    })
}
