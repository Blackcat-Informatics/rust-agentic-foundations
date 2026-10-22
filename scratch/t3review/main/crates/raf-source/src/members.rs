// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The members of a workspace, as cargo defines them.
//!
//! The member set is the root package of the workspace manifest, if it has
//! one, the directories that match `workspace.members` and not
//! `workspace.exclude`, and, transitively, every `path` dependency of a member
//! that lies inside the workspace directory and is not excluded. Every
//! manifest read goes through the [`Tree`], under the size cap.

use std::collections::{BTreeSet, HashSet, VecDeque};

use crate::budget::Budget;
use crate::error::{LoadError, LoadErrorKind, Origin};
use crate::glob::Pattern;
use crate::manifest::{self, Manifest, PathSource};
use crate::meter::Meter;
use crate::modtree::tidy;
use crate::path::{PathError, RelPath};
use crate::tree::{EntryKind, Tree};
use crate::work::{self, Work};

fn manifest_error(path: &RelPath, key: &str, message: String) -> LoadError {
    LoadError::new(path, None, LoadErrorKind::Manifest { message }).reached_by(
        &Origin::ManifestKey {
            key: key.to_owned(),
        },
    )
}

/// What a walk of the workspace directory may cost: the module-path budget
/// bounds the directories listed and the manifests read.
struct WalkMeter {
    left: usize,
    file: RelPath,
}

impl WalkMeter {
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
    load: &'a Meter,
    workspace_dir: &'a RelPath,
    meter: WalkMeter,
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

    /// The directories under `base` that match the pattern, walked on an
    /// explicit stack, depth first in name order. A directory more than
    /// [`Budget::DirectoryDepth`] levels below the workspace directory is
    /// refused.
    fn expand(
        &mut self,
        pattern: &Pattern,
        base: &RelPath,
        out: &mut BTreeSet<RelPath>,
    ) -> Result<(), LoadError> {
        // A pattern with no wildcard names one directory, which must be a
        // package; a glob skips what it matches that is not.
        let strict = pattern.is_literal();
        let mut stack = vec![(base.clone(), 0_usize)];
        while let Some((base, index)) = stack.pop() {
            self.load.directory(index, &base)?;
            if index == pattern.len() {
                if self.has_manifest(&base)? {
                    out.insert(base);
                } else if strict {
                    return Err(manifest_error(
                        self.manifest_path,
                        "workspace.members",
                        format!("the member {base} has no Cargo.toml"),
                    ));
                }
                continue;
            }
            if let Some(name) = pattern.literal(index) {
                let next = base.join(&name).map_err(|error| {
                    LoadError::tree(&base, None, &Origin::Discovery, error.into())
                })?;
                self.meter.spend()?;
                let kind = self
                    .tree
                    .entry(&next)
                    .map_err(|error| LoadError::tree(&next, None, &Origin::Discovery, error))?;
                if kind == Some(EntryKind::Dir) {
                    stack.push((next, index + 1));
                } else if strict {
                    return Err(manifest_error(
                        self.manifest_path,
                        "workspace.members",
                        format!("the member {next} is not a directory"),
                    ));
                }
                continue;
            }
            self.meter.spend()?;
            let entries = self
                .tree
                .list(&base)
                .map_err(|error| LoadError::tree(&base, None, &Origin::Discovery, error))?;
            let mut matched = Vec::new();
            for entry in entries {
                if entry.kind == EntryKind::Dir && pattern.segment_matches(index, &entry.name) {
                    let next = base.join(&entry.name).map_err(|error| {
                        LoadError::tree(&base, None, &Origin::Discovery, error.into())
                    })?;
                    matched.push((next, index + 1));
                }
            }
            // Reversed, so the first match in name order is walked first.
            stack.extend(matched.into_iter().rev());
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
            .read(&path, self.load.options().max_file_bytes)
            .map_err(|error| LoadError::tree(&path, None, &Origin::Discovery, error))?;
        self.load.read(&path, &text)?;
        manifest::parse(&text).map_err(|problem| fail(format!("{path}: {}", problem.message)))
    }
}

/// Parse each distinct pattern once, in first-seen order.
fn patterns(
    manifest_path: &RelPath,
    key: &str,
    texts: &[String],
) -> Result<Vec<Pattern>, LoadError> {
    let mut seen = HashSet::new();
    let mut parsed = Vec::new();
    for text in texts {
        let pattern = Pattern::parse(text).map_err(|why| {
            manifest_error(
                manifest_path,
                &format!("workspace.{key}"),
                format!("the pattern `{text}` is not supported: {why}"),
            )
        })?;
        if seen.insert(pattern.clone()) {
            parsed.push(pattern);
        }
    }
    Ok(parsed)
}

/// The `workspace.exclude` patterns, compiled once: a pattern without a
/// wildcard becomes an entry of a set, looked up once per ancestor of the
/// path, and only the patterns with a wildcard are matched one by one.
struct Excludes {
    literal: HashSet<String>,
    wildcard: Vec<Pattern>,
}

impl Excludes {
    fn new(patterns: Vec<Pattern>) -> Self {
        let mut literal = HashSet::new();
        let mut wildcard = Vec::new();
        for pattern in patterns {
            match pattern.literal_key() {
                Some(key) => {
                    literal.insert(key);
                }
                None => wildcard.push(pattern),
            }
        }
        Self { literal, wildcard }
    }

    /// True when a pattern matches the path `parts` or a directory above it.
    fn matches(&self, parts: &[&str]) -> bool {
        let mut prefix = String::new();
        for (index, part) in parts.iter().enumerate() {
            if index > 0 {
                prefix.push('/');
            }
            prefix.push_str(part);
            if self.literal.contains(&prefix) {
                return true;
            }
        }
        if self.wildcard.is_empty() {
            return false;
        }
        let chars: Vec<Vec<char>> = parts.iter().map(|part| part.chars().collect()).collect();
        self.wildcard.iter().any(|pattern| {
            work::add(Work::ExcludeTests, 1);
            pattern.matches_prefix(&chars)
        })
    }
}

/// The member directories of the workspace whose manifest `parsed` is at
/// `manifest_path`, relative to the tree root.
///
/// # Errors
///
/// Returns a [`LoadError`] for a pattern outside the supported syntax, an
/// unreadable manifest, a path dependency inside the workspace whose manifest
/// is missing or malformed (naming the dependency key), a walk that passes
/// the module-path budget, a member pattern deeper than
/// [`Budget::DirectoryDepth`], and a read that passes [`Budget::TotalBytes`].
pub(crate) fn members(
    tree: &dyn Tree,
    load: &Meter,
    workspace_dir: &RelPath,
    manifest_path: &RelPath,
    parsed: &Manifest,
) -> Result<BTreeSet<RelPath>, LoadError> {
    let member_patterns = patterns(manifest_path, "members", &parsed.members)?;
    let excluded = Excludes::new(patterns(manifest_path, "exclude", &parsed.exclude)?);
    let mut cx = Context {
        manifest_path,
        tree,
        load,
        workspace_dir,
        meter: WalkMeter {
            left: Budget::ModulePaths.limit(),
            file: manifest_path.clone(),
        },
    };
    let is_excluded = |cx: &Context<'_>, dir: &RelPath| excluded.matches(&cx.relative(dir));
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
            if excluded.matches(&parts) {
                continue;
            }
        }
        let mut matched = BTreeSet::new();
        cx.expand(pattern, workspace_dir, &mut matched)?;
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
        for dependency in &member.dependencies {
            let (base, relative) = match &dependency.path {
                None => continue,
                Some(PathSource::Own(path)) => (dir.clone(), path.as_str()),
                Some(PathSource::Workspace(name)) => match parsed
                    .workspace_dependencies
                    .get(name)
                    .and_then(|entry| entry.path.as_deref())
                {
                    Some(path) => (workspace_dir.clone(), path),
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

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::members;
    use crate::manifest;
    use crate::meter::Meter;
    use crate::options::LoadOptions;
    use crate::path::RelPath;
    use crate::tree::MemoryTree;
    use crate::work::{self, Work};

    const CAP: usize = 4096;

    const PACKAGE: &str = "[package]\nname = \"p\"\nversion = \"0.1.0\"\n";

    /// The directories `members` finds for a workspace manifest that holds the
    /// quoted `members` and `exclude`, beside one package `p`.
    fn find(listed: &[String], exclude: &[String]) -> usize {
        let quoted = |entries: &[String]| {
            let mut text = String::new();
            for entry in entries {
                write!(text, "\"{entry}\",").expect("a string takes the text");
            }
            text
        };
        let root = format!(
            "[workspace]\nmembers = [{}]\nexclude = [{}]\n",
            quoted(listed),
            quoted(exclude)
        );
        let tree: MemoryTree = [
            ("Cargo.toml", root.as_str()),
            ("p/Cargo.toml", PACKAGE),
            ("p/src/lib.rs", ""),
        ]
        .into_iter()
        .collect();
        let parsed = manifest::parse(&root).expect("the manifest parses");
        let meter = Meter::new(&LoadOptions::default());
        let manifest_path = RelPath::new("Cargo.toml").expect("a path");
        members(&tree, &meter, &RelPath::root(), &manifest_path, &parsed)
            .unwrap_or_else(|error| panic!("{error}"))
            .len()
    }

    #[test]
    fn distinct_patterns_at_both_caps_are_tested_a_bounded_number_of_times() {
        // Every literal member is tested against every wildcard exclude, and
        // the only exclude that has one segment, and so is matched against a
        // name at all, comes last.
        let mut members: Vec<String> = (0..CAP - 1).map(|i| format!("m{i}")).collect();
        members.push("p".to_owned());
        let mut exclude: Vec<String> = (0..CAP - 1).map(|i| format!("a{i}/*")).collect();
        exclude.push("m*".to_owned());
        let wildcards = exclude.len();
        // Each literal member is tested once, and the one directory found
        // once more. A name has at most five characters, and `m*` has two
        // pieces, so a test of it takes at most (2 + 1) * (5 + 2) steps.
        let tested = members.len() + 1;
        let steps = tested * 21;
        let (found, tally) =
            work::measure_kind(Work::PatternSteps, steps, || find(&members, &exclude));
        assert_eq!(found, 1);
        assert!(tally.get(Work::ExcludeTests) > 0);
        assert!(tally.get(Work::ExcludeTests) <= tested * wildcards);
        assert!(tally.get(Work::PatternSteps) > 0);
        assert!(tally.get(Work::PatternSteps) <= steps);
    }

    #[test]
    fn duplicate_patterns_are_compiled_and_tested_once() {
        let members = vec!["p".to_owned(); CAP];
        let exclude = vec!["q*".to_owned(); CAP];
        // One member pattern, one wildcard exclude: the literal member is
        // tested against it, and so is the directory it finds.
        let (found, tally) = work::measure_kind(Work::ExcludeTests, 2, || find(&members, &exclude));
        assert_eq!(found, 1);
        assert_eq!(tally.get(Work::ExcludeTests), 2);
    }
}
