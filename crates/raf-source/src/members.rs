// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The members of a workspace, as cargo defines them.
//!
//! The member set is the root package of the workspace manifest, if it has
//! one, the directories that match `workspace.members` and not
//! `workspace.exclude`, and, transitively, every `path` dependency of a member
//! that lies inside the workspace directory and is not excluded. Every
//! manifest read goes through the [`Tree`], under the size cap.

use std::collections::{BTreeSet, VecDeque};

use crate::budget::Budget;
use crate::error::{LoadError, LoadErrorKind, Origin};
use crate::glob::Pattern;
use crate::manifest::{self, Manifest, PathSource};
use crate::modtree::tidy;
use crate::path::{PathError, RelPath};
use crate::tree::{EntryKind, Tree};

fn manifest_error(path: &RelPath, key: &str, message: String) -> LoadError {
    LoadError::new(path, None, LoadErrorKind::Manifest { message }).reached_by(
        &Origin::ManifestKey {
            key: key.to_owned(),
        },
    )
}

/// What a walk of the workspace directory may cost: the module-path budget
/// bounds the directories listed and the manifests read.
struct Meter {
    left: usize,
    file: RelPath,
}

impl Meter {
    fn spend(&mut self) -> Result<(), LoadError> {
        if self.left == 0 {
            return Err(LoadError::new(
                &self.file,
                None,
                LoadErrorKind::Budget {
                    budget: Budget::ModulePaths,
                    limit: Budget::ModulePaths.limit(),
                },
            ));
        }
        self.left -= 1;
        Ok(())
    }
}

struct Context<'a> {
    manifest_path: &'a RelPath,
    tree: &'a dyn Tree,
    max_file_bytes: u64,
    workspace_dir: &'a RelPath,
    meter: Meter,
}

impl Context<'_> {
    fn relative<'p>(&self, dir: &'p RelPath) -> Vec<&'p str> {
        dir.segments()
            .skip(self.workspace_dir.segments().count())
            .collect()
    }

    fn has_manifest(&mut self, dir: &RelPath) -> Result<bool, LoadError> {
        self.meter.spend()?;
        let path = dir
            .join("Cargo.toml")
            .map_err(|error| LoadError::tree(dir, None, &Origin::Discovery, error.into()))?;
        Ok(self
            .tree
            .entry(&path)
            .map_err(|error| LoadError::tree(&path, None, &Origin::Discovery, error))?
            .is_some())
    }

    /// The directories under `base` that match the pattern from segment
    /// `index` on.
    fn expand(
        &mut self,
        pattern: &Pattern,
        base: &RelPath,
        index: usize,
        out: &mut BTreeSet<RelPath>,
    ) -> Result<(), LoadError> {
        // A pattern with no wildcard names one directory, which must be a
        // package; a glob skips what it matches that is not.
        let strict = pattern.is_literal();
        let missing = |what: &str| {
            manifest_error(
                self.manifest_path,
                "workspace.members",
                format!("the member {base} {what}"),
            )
        };
        if index == pattern.len() {
            if self.has_manifest(base)? {
                out.insert(base.clone());
            } else if strict {
                return Err(missing("has no Cargo.toml"));
            }
            return Ok(());
        }
        if let Some(name) = pattern.literal(index) {
            let next = base
                .join(&name)
                .map_err(|error| LoadError::tree(base, None, &Origin::Discovery, error.into()))?;
            self.meter.spend()?;
            let kind = self
                .tree
                .entry(&next)
                .map_err(|error| LoadError::tree(&next, None, &Origin::Discovery, error))?;
            if kind == Some(EntryKind::Dir) {
                self.expand(pattern, &next, index + 1, out)?;
            } else if strict {
                return Err(manifest_error(
                    self.manifest_path,
                    "workspace.members",
                    format!("the member {next} is not a directory"),
                ));
            }
            return Ok(());
        }
        self.meter.spend()?;
        let entries = self
            .tree
            .list(base)
            .map_err(|error| LoadError::tree(base, None, &Origin::Discovery, error))?;
        for entry in entries {
            if entry.kind == EntryKind::Dir && pattern.segment_matches(index, &entry.name) {
                let next = base.join(&entry.name).map_err(|error| {
                    LoadError::tree(base, None, &Origin::Discovery, error.into())
                })?;
                self.expand(pattern, &next, index + 1, out)?;
            }
        }
        Ok(())
    }

    fn read(
        &mut self,
        dir: &RelPath,
        dependency: Option<(&RelPath, &str)>,
    ) -> Result<Manifest, LoadError> {
        self.meter.spend()?;
        let path = dir
            .join("Cargo.toml")
            .map_err(|error| LoadError::tree(dir, None, &Origin::Discovery, error.into()))?;
        let fail = |message: String| match dependency {
            Some((owner, key)) => manifest_error(owner, key, message),
            None => manifest_error(&path, "workspace.members", message),
        };
        let present = self
            .tree
            .entry(&path)
            .map_err(|error| LoadError::tree(&path, None, &Origin::Discovery, error))?;
        if present.is_none() {
            return Err(fail(format!("the manifest {path} does not exist")));
        }
        let text = self
            .tree
            .read(&path, self.max_file_bytes)
            .map_err(|error| LoadError::tree(&path, None, &Origin::Discovery, error))?;
        manifest::parse(&text).map_err(|problem| fail(format!("{path}: {}", problem.message)))
    }
}

fn patterns(
    manifest_path: &RelPath,
    key: &str,
    texts: &[String],
) -> Result<Vec<Pattern>, LoadError> {
    texts
        .iter()
        .map(|text| {
            Pattern::parse(text).map_err(|why| {
                manifest_error(
                    manifest_path,
                    &format!("workspace.{key}"),
                    format!("the pattern `{text}` is not supported: {why}"),
                )
            })
        })
        .collect()
}

/// The member directories of the workspace whose manifest `parsed` is at
/// `manifest_path`, relative to the tree root.
///
/// # Errors
///
/// Returns a [`LoadError`] for a pattern outside the supported syntax, an
/// unreadable manifest, a path dependency inside the workspace whose manifest
/// is missing or malformed (naming the dependency key), and a walk that passes
/// the budget.
pub(crate) fn members(
    tree: &dyn Tree,
    max_file_bytes: u64,
    workspace_dir: &RelPath,
    manifest_path: &RelPath,
    parsed: &Manifest,
) -> Result<BTreeSet<RelPath>, LoadError> {
    let member_patterns = patterns(manifest_path, "members", &parsed.members)?;
    let excluded = patterns(manifest_path, "exclude", &parsed.exclude)?;
    let mut cx = Context {
        manifest_path,
        tree,
        max_file_bytes,
        workspace_dir,
        meter: Meter {
            left: Budget::ModulePaths.limit(),
            file: manifest_path.clone(),
        },
    };
    let is_excluded = |cx: &Context<'_>, dir: &RelPath| {
        let parts = cx.relative(dir);
        excluded
            .iter()
            .any(|pattern| pattern.matches_prefix(&parts))
    };
    let mut found: BTreeSet<RelPath> = BTreeSet::new();
    if parsed.package.is_some() {
        found.insert(workspace_dir.clone());
    }
    for pattern in &member_patterns {
        // An excluded literal member is ignored, not required to exist.
        if pattern.is_literal() {
            let names: Vec<String> = (0..pattern.len())
                .filter_map(|i| pattern.literal(i))
                .collect();
            let parts: Vec<&str> = names.iter().map(String::as_str).collect();
            if excluded
                .iter()
                .any(|exclude| exclude.matches_prefix(&parts))
            {
                continue;
            }
        }
        let mut matched = BTreeSet::new();
        cx.expand(pattern, workspace_dir, 0, &mut matched)?;
        found.extend(matched.into_iter().filter(|dir| !is_excluded(&cx, dir)));
    }
    // The path dependencies of every member, transitively.
    let mut queue: VecDeque<RelPath> = found.iter().cloned().collect();
    while let Some(dir) = queue.pop_front() {
        let own;
        let member: &Manifest = if dir == *workspace_dir {
            parsed
        } else {
            own = cx.read(&dir, None)?;
            &own
        };
        let owner = dir
            .join("Cargo.toml")
            .map_err(|error| LoadError::tree(&dir, None, &Origin::Discovery, error.into()))?;
        for dependency in &member.path_dependencies {
            let (base, relative) = match &dependency.path {
                PathSource::Own(path) => (dir.clone(), path.as_str()),
                PathSource::Workspace(name) => match parsed.workspace_paths.get(name) {
                    Some(path) => (workspace_dir.clone(), path.as_str()),
                    // A workspace dependency without a path is not a path dependency.
                    None => continue,
                },
            };
            let joined = match base.join(relative) {
                Ok(joined) => joined,
                // A path that leaves the tree, or is absolute, cannot be inside
                // the workspace.
                Err(PathError::AboveRoot { .. } | PathError::Absolute { .. }) => continue,
                Err(other) => {
                    return Err(manifest_error(&owner, &dependency.key, other.to_string()));
                }
            };
            cx.meter.spend()?;
            let target = tidy(tree, &joined)
                .map_err(|error| LoadError::tree(&owner, None, &Origin::Discovery, error))?;
            let inside = target.segments().count() > workspace_dir.segments().count()
                && target
                    .segments()
                    .zip(workspace_dir.segments())
                    .all(|(a, b)| a == b)
                && target.segments().all(|segment| segment != "..");
            if !inside || found.contains(&target) || is_excluded(&cx, &target) {
                continue;
            }
            cx.read(&target, Some((&owner, &dependency.key)))?;
            found.insert(target.clone());
            queue.push_back(target);
        }
    }
    Ok(found)
}
