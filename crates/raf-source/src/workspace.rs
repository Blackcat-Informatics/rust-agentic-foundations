// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The loaded workspace: packages, targets and the files of every module tree.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use crate::discovery::discover;
use crate::error::{LoadError, LoadErrorKind, Origin};
use crate::facts::{FileFacts, TestRole};
use crate::index::Index;
use crate::index::PackageInfo;
use crate::item::{Item, ItemId, Namespace};
use crate::manifest::{self, EditionDecl, Manifest, PackageDecl};
use crate::members::members;
use crate::modtree::{Reach, TargetId, Visited, Walker};
use crate::options::LoadOptions;
use crate::path::RelPath;
use crate::resolve::{Resolution, ResolveError};
use crate::target::TargetKind;
use crate::target::{Context, Package, Target, check_crate_paths, derive_targets};
use crate::tree::Tree;

/// A source file of the workspace, parsed once.
#[derive(Debug, PartialEq, Eq)]
pub struct SourceFile {
    /// The path, relative to the tree root.
    pub path: RelPath,
    /// The name of the package of the first target that reaches the file.
    pub package: String,
    /// True when some target that ships (library, binary, build script)
    /// reaches the file other than through test code.
    pub shipping: bool,
    /// True when every way of reaching the file goes through test code.
    pub test: bool,
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
#[derive(Debug, PartialEq, Eq)]
pub struct Workspace {
    packages: Vec<Package>,
    targets: Vec<Target>,
    files: BTreeMap<RelPath, SourceFile>,
    index: Index,
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
        let discovered = discover(tree, &root_paths, options.max_file_bytes)?;
        let found = &discovered.packages;
        let mut infos: BTreeMap<String, PackageInfo> = BTreeMap::new();
        let mut workspace_editions = Caches::default();
        let mut packages = Vec::new();
        let mut derived = Vec::new();
        for entry in found {
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
            infos.insert(
                entry.package.name.clone(),
                package_info(
                    tree,
                    options.max_file_bytes,
                    &entry.package,
                    declared,
                    &entry.manifest,
                    &mut workspace_editions,
                )?,
            );
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
        let (visited, visits) = walker.finish();
        let mut files: BTreeMap<RelPath, SourceFile> = BTreeMap::new();
        for (path, visited) in visited {
            let file = source_file(&path, visited, &targets)?;
            files.insert(path, file);
        }
        let index = Index::build(&targets, &files, &visits, &infos)?;
        Ok(Self {
            packages,
            targets,
            files,
            index,
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

    /// Every item, sorted by path, then name space, file and position, so the
    /// items under a path prefix form a contiguous run.
    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.index
            .order
            .iter()
            .map(|id| &self.index.items[id.index()])
    }

    /// The items whose path starts with `prefix`, in the order of
    /// [`Workspace::items`].
    pub fn items_under<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = &'a Item> {
        let order = &self.index.order;
        let start = order.partition_point(|id| self.index.items[id.index()].path.as_str() < prefix);
        order[start..]
            .iter()
            .map(|id| &self.index.items[id.index()])
            .take_while(move |item| item.path.starts_with(prefix))
    }

    /// One item by id.
    #[must_use]
    pub fn item(&self, id: ItemId) -> Option<&Item> {
        self.index.items.get(id.index())
    }

    /// The items defined in `file`, ordered by the byte offset of their span.
    pub fn items_in<'a>(&'a self, file: &RelPath) -> impl Iterator<Item = &'a Item> {
        self.index
            .by_file
            .get(file)
            .into_iter()
            .flatten()
            .map(|id| &self.index.items[id.index()])
    }

    /// The innermost items of `file` whose span holds the byte `offset`. More
    /// than one item is returned when the file is reached under several paths.
    #[must_use]
    pub fn item_at(&self, file: &RelPath, offset: usize) -> Vec<&Item> {
        let holding: Vec<&Item> = self
            .items_in(file)
            .filter(|item| item.span.start_byte <= offset && offset < item.span.end_byte)
            .collect();
        let smallest = holding
            .iter()
            .map(|item| item.span.end_byte - item.span.start_byte)
            .min();
        holding
            .into_iter()
            .filter(|item| Some(item.span.end_byte - item.span.start_byte) == smallest)
            .collect()
    }

    /// True when the item is in a shipping target and is not test code.
    #[must_use]
    pub fn is_shipping(&self, item: &Item) -> bool {
        item.test == TestRole::None
            && self
                .targets
                .get(item.target.index())
                .is_some_and(|target| target.shipping)
    }

    /// Resolve a canonical path such as `demo::inner::go`, trying the type,
    /// value and macro name spaces in that order.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError::NotFound`] when the workspace defines nothing at
    /// the path (a dependency item and a macro-generated item both end here),
    /// [`ResolveError::Ambiguous`] when it names several symbols,
    /// [`ResolveError::Cyclic`] when the imports on the way form a cycle and
    /// [`ResolveError::Invalid`] when the text is not a path.
    pub fn resolve(&self, path: &str) -> Result<Resolution, ResolveError> {
        self.index.resolve(path, None)
    }

    /// Resolve a canonical path in one name space.
    ///
    /// # Errors
    ///
    /// As [`Workspace::resolve`].
    pub fn resolve_in(&self, path: &str, namespace: Namespace) -> Result<Resolution, ResolveError> {
        self.index.resolve(path, Some(namespace))
    }

    /// The items that share a source item, grouped by its identity: the file
    /// and the byte offset where its span starts. A source item that several
    /// module paths reach appears as several items, and the group holds them
    /// all, sorted by path. Groups come in the order of file and offset.
    pub fn items_by_source(&self) -> impl Iterator<Item = (&RelPath, usize, &[ItemId])> {
        self.index
            .by_source
            .iter()
            .map(|((file, offset), ids)| (file, *offset, ids.as_slice()))
    }

    /// The items that share a source item with `item`, itself included.
    #[must_use]
    pub fn same_source(&self, item: &Item) -> &[ItemId] {
        self.index
            .by_source
            .get(&(item.file.clone(), item.span.start_byte))
            .map_or(&[][..], Vec::as_slice)
    }

    /// The test functions: the data a caller needs to run one with
    /// `cargo test -p <package> --test <target> <name>`, `--bin <target>` or
    /// `--lib`.
    ///
    /// # Errors
    ///
    /// Returns a [`LoadError`] of kind `Internal` if a test function's path
    /// does not start with the crate path of its target, which the index
    /// guarantees it does.
    pub fn test_fns(&self) -> Result<Vec<TestFn>, LoadError> {
        self.items()
            .filter(|item| item.test == TestRole::TestFn)
            .map(|item| test_fn_of(item, &self.targets))
            .collect()
    }
}

/// What is computed once per workspace manifest during a load.
#[derive(Default)]
struct Caches {
    editions: BTreeMap<RelPath, WorkspaceEdition>,
    members: BTreeMap<RelPath, BTreeSet<RelPath>>,
}

/// What the nearest workspace manifest says about the edition of a package.
#[derive(Debug, Clone, PartialEq, Eq)]
enum WorkspaceEdition {
    /// The workspace sets it.
    Set(u16),
    /// The workspace has no `[workspace.package] edition`, or there is no
    /// workspace manifest above the package.
    Unset,
    /// The nearest workspace manifest does not have the package as a member.
    NotMember(RelPath),
}

fn manifest_error(path: &RelPath, key: &str, message: String) -> LoadError {
    LoadError::new(path, None, LoadErrorKind::Manifest { message }).reached_by(
        &Origin::ManifestKey {
            key: key.to_owned(),
        },
    )
}

/// The `[workspace.package] edition` that applies to a package directory:
/// the nearest manifest at or above it that has a `[workspace]` table, found
/// the way cargo finds it, by walking up through the ancestors within the tree.
/// A missing ancestor manifest is skipped; one that cannot be read or parsed is
/// an error. The manifest is not added to the packages that discovery found.
fn workspace_edition(
    tree: &dyn Tree,
    dir: &RelPath,
    max_file_bytes: u64,
    caches: &mut Caches,
) -> Result<WorkspaceEdition, LoadError> {
    let cache = &mut caches.editions;
    if let Some(known) = cache.get(dir) {
        return Ok(known.clone());
    }
    let origin = Origin::Discovery;
    let mut at = Some(dir.clone());
    let mut found = WorkspaceEdition::Unset;
    while let Some(current) = at {
        let path = current
            .join("Cargo.toml")
            .map_err(|error| LoadError::tree(&current, None, &origin, error.into()))?;
        let present = tree
            .entry(&path)
            .map_err(|error| LoadError::tree(&path, None, &origin, error))?;
        if present.is_some() {
            let text = tree
                .read(&path, max_file_bytes)
                .map_err(|error| LoadError::tree(&path, None, &origin, error))?;
            let parsed = manifest::parse(&text).map_err(|problem| {
                let origin = problem
                    .key
                    .map_or(Origin::Discovery, |key| Origin::ManifestKey { key });
                LoadError::new(
                    &path,
                    None,
                    LoadErrorKind::Manifest {
                        message: problem.message,
                    },
                )
                .reached_by(&origin)
            })?;
            if parsed.has_workspace {
                let set = match caches.members.entry(current.clone()) {
                    std::collections::btree_map::Entry::Occupied(known) => known.into_mut(),
                    std::collections::btree_map::Entry::Vacant(slot) => {
                        slot.insert(members(tree, max_file_bytes, &current, &path, &parsed)?)
                    }
                };
                found = if !set.contains(dir) {
                    WorkspaceEdition::NotMember(path)
                } else if let Some(edition) = parsed.workspace_edition {
                    WorkspaceEdition::Set(edition)
                } else {
                    WorkspaceEdition::Unset
                };
                break;
            }
        }
        at = current.parent();
    }
    caches.editions.insert(dir.clone(), found.clone());
    Ok(found)
}

/// What the index needs to know about a package's manifest.
///
/// # Errors
///
/// Returns a manifest error naming `package.edition` when the package inherits
/// its edition and the nearest workspace manifest above it does not have the
/// package as a member or does not set an edition, which cargo rejects as well.
fn package_info(
    tree: &dyn Tree,
    max_file_bytes: u64,
    package: &Package,
    declared: &PackageDecl,
    manifest: &Manifest,
    caches: &mut Caches,
) -> Result<PackageInfo, LoadError> {
    let edition = match &declared.edition {
        EditionDecl::Explicit(edition) => *edition,
        EditionDecl::Unset => 2015,
        EditionDecl::Inherited => {
            match workspace_edition(tree, &package.dir, max_file_bytes, caches)? {
                WorkspaceEdition::Set(edition) => edition,
                WorkspaceEdition::Unset => {
                    return Err(manifest_error(
                        &package.manifest,
                        "package.edition",
                        "`edition.workspace = true` but no `[workspace.package] edition` is set by the nearest workspace manifest above the package"
                            .to_owned(),
                    ));
                }
                WorkspaceEdition::NotMember(workspace) => {
                    return Err(manifest_error(
                        &package.manifest,
                        "package.edition",
                        format!(
                            "`edition.workspace = true` but the package is not a member of the workspace {workspace} (it is not listed in `workspace.members`, or is excluded)"
                        ),
                    ));
                }
            }
        }
    };
    Ok(PackageInfo {
        edition,
        dependencies: manifest.dependencies.clone(),
    })
}

fn internal(file: &RelPath, line: Option<usize>, what: String) -> LoadError {
    LoadError::new(file, line, LoadErrorKind::Internal { what })
}

/// The cargo selection data of one test function.
fn test_fn_of(item: &Item, targets: &[Target]) -> Result<TestFn, LoadError> {
    let target = targets.get(item.target.index()).ok_or_else(|| {
        internal(
            &item.file,
            Some(item.name_line),
            format!(
                "the target {} of `{}` does not exist",
                item.target.index(),
                item.path
            ),
        )
    })?;
    let name = item
        .path
        .strip_prefix(&target.crate_path)
        .and_then(|rest| rest.strip_prefix("::"))
        .ok_or_else(|| {
            internal(
                &item.file,
                Some(item.name_line),
                format!(
                    "the path `{}` does not start with the crate path `{}`",
                    item.path, target.crate_path
                ),
            )
        })?;
    Ok(TestFn {
        id: item.id,
        package: target.package.clone(),
        kind: target.kind,
        target: target.name.clone(),
        name: name.to_owned(),
    })
}

/// A parsed file and the ways it was reached, as a source file.
fn source_file(
    path: &RelPath,
    visited: Visited,
    targets: &[Target],
) -> Result<SourceFile, LoadError> {
    let target_of = |reach: &Reach| {
        targets.get(reach.target.index()).ok_or_else(|| {
            internal(
                path,
                None,
                format!(
                    "the target {} of a module path does not exist",
                    reach.target.index()
                ),
            )
        })
    };
    let Some(first) = visited.reached.first() else {
        return Err(internal(
            path,
            None,
            "a loaded file has no module path".to_owned(),
        ));
    };
    let package = target_of(first)?.package.clone();
    let mut shipping = false;
    let mut test = true;
    for reach in &visited.reached {
        let target = target_of(reach)?;
        if !reach.test {
            test = false;
            shipping |= target.shipping;
        }
    }
    let parsed = Rc::unwrap_or_clone(visited.parsed);
    Ok(SourceFile {
        path: path.clone(),
        package,
        shipping,
        test,
        text: parsed.text,
        facts: parsed.facts,
        reached: visited.reached,
    })
}

/// A function marked `#[test]`, with what cargo needs to select it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TestFn {
    /// The item.
    pub id: ItemId,
    /// The package name (`-p`).
    pub package: String,
    /// The kind of the target that holds it: `Test` for `--test`, `Bin` for
    /// `--bin`, `Lib` for `--lib`.
    pub kind: TargetKind,
    /// The Cargo name of the target.
    pub target: String,
    /// The path of the function inside its crate, such as `inner::case`.
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::MemoryTree;

    #[test]
    fn a_test_function_whose_path_leaves_its_crate_path_is_a_typed_error() {
        let tree = MemoryTree::from([
            ("Cargo.toml", "[package]\nname = \"demo\"\n"),
            ("src/lib.rs", "#[test]\nfn case() {}\n"),
        ]);
        let ws = Workspace::load(&tree, &["."]).expect("loads");
        let fns = ws.test_fns().expect("the invariant holds");
        assert_eq!(fns.len(), 1);
        let mut item = ws.item(fns[0].id).expect("item").clone();
        assert!(test_fn_of(&item, &ws.targets).is_ok());
        item.path = "other::case".to_owned();
        let error = test_fn_of(&item, &ws.targets).expect_err("the path leaves the crate");
        assert!(
            matches!(error.kind, LoadErrorKind::Internal { .. }),
            "{error}"
        );
        assert_eq!(error.line, Some(2));
        item.path = "demo::case".to_owned();
        item.target = TargetId(9);
        assert!(test_fn_of(&item, &ws.targets).is_err());
    }
}
