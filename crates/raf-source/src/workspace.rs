// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The loaded workspace: packages, targets and the files of every module tree.

mod locked;
pub use locked::{GitPackageRoots, LockedBinding, LockedEdge, LockedManifest};

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;
use std::rc::Rc;

use purrdf_lex::allocation::Memory;
use purrdf_lex::walk::{try_equal_range_by, try_sort_unstable_by};

use crate::discovery::{Found, discover};
use crate::error::{LoadError, LoadErrorKind, Origin};
use crate::exports::{ExportError, ExportOptions, Exports, Surface};
use crate::facts::{FileFacts, TestRole};
use crate::index::{DependencyInfo, Index, LOAD_BASE_BYTES, PackageInfo};
use crate::item::{ImplBlock, ImplId, Item, ItemId, Namespace, UnresolvedImpl};
use crate::manifest::{
    self, Dependency, DependencyDeclaration, EditionDecl, InheritedDependency, Manifest,
    PackageDecl, PathSource, WorkspaceDependency,
};
use crate::members::members;
use crate::meter::{ItemAdmission, Meter, Usage};
use crate::modtree::{Reach, TargetId, Visited, Walked, Walker, tidy};
use crate::options::LoadOptions;
use crate::path::{PathError, RelPath};
use crate::reach::{self, Node, ReachError, Reachability};
use crate::resolve::{Resolution, ResolveError};
use crate::target::TargetKind;
use crate::target::{Context, Package, PackageId, Target, check_crate_paths, derive_targets};
use crate::tree::Tree;

/// A source file of the workspace, parsed once.
///
/// The facts the index is built from stay inside the crate; the items of a
/// file are read through [`Workspace::items_in`] and the other accessors.
///
/// The struct is `#[non_exhaustive]`: adding a field is not a breaking change,
/// so a pattern names the fields it reads and ends in `..`.
///
/// ```compile_fail,E0638
/// # fn names(file: &raf_source::SourceFile) {
/// let raf_source::SourceFile { path, package, shipping, test, text, reached } = file;
/// # }
/// ```
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
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
    pub(crate) facts: FileFacts,
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
///
/// Two workspaces are equal when their packages, targets, files and index are;
/// what the loads spent ([`Workspace::usage`]) can differ between backends
/// that hold the same files, as a disk identity holds no text.
#[derive(Debug)]
pub struct Workspace {
    packages: Vec<Package>,
    dependencies: Vec<Vec<DependencyDeclaration>>,
    locked: Vec<LockedBinding>,
    /// Package IDs sorted by display name, then by their exact load-local ID.
    package_names: Vec<PackageId>,
    targets: Vec<Target>,
    files: BTreeMap<RelPath, SourceFile>,
    index: Index,
    usage: Usage,
    /// The options of the load, whose item-bytes budget also bounds what a
    /// query such as [`Workspace::exported_items`] holds.
    options: LoadOptions,
}

impl PartialEq for Workspace {
    fn eq(&self, other: &Self) -> bool {
        self.packages == other.packages
            && self.dependencies == other.dependencies
            && self.locked == other.locked
            && self.package_names == other.package_names
            && self.targets == other.targets
            && self.files == other.files
            && self.index == other.index
    }
}

impl Eq for Workspace {}

impl Workspace {
    /// Prepare module-level code lookup from actual source owner identities.
    /// The retained reading table is admitted under this load's original limit.
    /// This does not classify lexical shadows or prove provider delegation.
    ///
    /// # Errors
    /// Refuses exhausted original item allowance or native storage admission.
    pub fn code_resolver(&self) -> Result<crate::CodeResolver<'_>, LoadError> {
        crate::CodeResolver::new(&self.index, &self.options, self.usage)
    }

    /// Load the packages under `roots` with the default [`LoadOptions`].
    ///
    /// A root is a directory path relative to the tree root; `"."` is the tree
    /// root itself. There are no default roots: each caller names its own, and
    /// an empty list is a [`LoadErrorKind::NoRoots`] error.
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
    /// and each target's module tree is followed. The load returns the first
    /// error, in the order discovery, targets, module walk, index, and does not
    /// collect errors; the error names the file, the line when there is one,
    /// and the module, attribute or manifest key that led to it.
    ///
    /// # Errors
    ///
    /// Returns a [`LoadError`] for an empty `roots` list, a root that is missing or
    /// not a directory, a manifest that cannot be read, a target problem, an
    /// unresolved or ambiguous `mod`, a path that leaves the tree, a module cycle,
    /// a file over `max_file_bytes`, a load over a [`Budget`](crate::Budget) (the
    /// byte budgets as `options` sets them), or a syntax error.
    ///
    /// Of the items that syn has no node for, a `use` group with a member that
    /// starts with `::`, a function with no body and a type with no definition
    /// are indexed; any other, such as a constant with no value or a type with
    /// bounds and no definition, is a [`LoadErrorKind::UnsupportedItem`]
    /// naming the file and the first line of the item.
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
        Self::load_from(tree, &root_paths, options)
    }

    /// Load explicit roots, retaining each path's selected source authority.
    ///
    /// [`crate::SourceTrees`] dispatches independently rooted sources. This is
    /// physical source selection, not locked dependency or configuration proof.
    /// Ordinary [`Workspace::load_with`] calls this same loader with first-party
    /// roots. Paths returned on packages, targets and items retain their source.
    ///
    /// # Errors
    ///
    /// As [`Workspace::load_with`], including an unavailable source authority.
    pub fn load_from(
        tree: &dyn Tree,
        roots: &[RelPath],
        options: &LoadOptions,
    ) -> Result<Self, LoadError> {
        let meter = Meter::new(options);
        Self::load_meter(tree, roots, options, &meter, None)
    }

    fn load_meter(
        tree: &dyn Tree,
        roots: &[RelPath],
        options: &LoadOptions,
        meter: &Meter,
        binding: Option<&locked::BindingLoad<'_>>,
    ) -> Result<Self, LoadError> {
        let root = RelPath::root();
        if roots.is_empty() {
            return Err(LoadError::new(&root, None, LoadErrorKind::NoRoots));
        }
        meter.items(LOAD_BASE_BYTES, &root, None)?;
        let discovered = discover(tree, roots, meter)?;
        let found = &discovered.packages;
        let dirs = package_directory_index(found, meter)?;
        let mut infos = Vec::new();
        let mut admission = ItemAdmission::new(meter, &root);
        Memory::new(&mut admission)
            .reserve(&mut infos, found.len())
            .map_err(|error| admission.error(error))?;
        let mut caches = Caches::default();
        let reader = Reader {
            tree,
            meter,
            dirs: &dirs,
            found,
        };
        let mut locked = if let Some(binding) = binding {
            locked::bind_found(binding, &reader, &mut caches)?
        } else {
            Vec::new()
        };
        let mut packages = Vec::new();
        let mut dependencies = Vec::new();
        Memory::new(&mut admission)
            .reserve(&mut dependencies, found.len())
            .map_err(|error| admission.error(error))?;
        let mut derived = Vec::new();
        for (package_index, entry) in found.iter().enumerate() {
            let context = Context {
                package_id: PackageId(package_index),
                tree,
                manifest_path: &entry.package.manifest,
                dir: &entry.package.dir,
                manifest: &entry.manifest,
                package: &entry.declared,
            };
            for target in derive_targets(&context)? {
                derived.push((&entry.package.manifest, target));
            }
            infos.push(package_info(
                &reader,
                &entry.package,
                &entry.declared,
                &entry.manifest,
                &mut caches,
            )?);
            dependencies.push(dependency_declarations(
                &reader,
                &entry.package,
                &entry.manifest,
                &mut caches,
            )?);
            packages.push(entry.package.clone());
        }
        if let Some(binding) = binding
            && locked::configure(binding, found, &dependencies, &mut locked, meter)?
        {
            locked::select_targets(&locked, &mut derived, meter)?;
        }
        check_crate_paths(&derived)?;
        let package_names = package_name_index(&packages, meter)?;
        let (targets, origins): (Vec<_>, Vec<_>) = derived
            .into_iter()
            .map(|(_, item)| (item.target, item.origin))
            .unzip();
        let found_files = walk_sources(tree, &targets, &origins, &locked, meter)?;
        let mut files: BTreeMap<RelPath, SourceFile> = BTreeMap::new();
        for (path, visited) in found_files.files {
            let file = source_file(&path, visited, &targets)?;
            files.insert(path, file);
        }
        if let Some(binding) = binding {
            locked::install_edges(binding, found, &dependencies, &locked, &mut infos, meter)?;
        }
        let index = Index::build(
            &targets,
            &files,
            &found_files.visits,
            &infos,
            found_files.paths,
            meter,
        )?;
        Ok(Self {
            packages,
            dependencies,
            locked,
            package_names,
            targets,
            files,
            index,
            usage: meter.usage(),
            options: options.clone(),
        })
    }

    /// The index, for the tests of the crate.
    #[cfg(test)]
    pub(crate) fn index(&self) -> &Index {
        &self.index
    }

    /// What the load spent against its budgets: the bytes it read, kept in the
    /// index and in the facts of its files, the most tokens of one file and
    /// the deepest directory it entered.
    #[must_use]
    pub fn usage(&self) -> Usage {
        self.usage
    }

    /// The packages, sorted by manifest path.
    #[must_use]
    pub fn packages(&self) -> &[Package] {
        &self.packages
    }

    /// Written dependency constraints for an exact load-local package.
    ///
    /// Includes normal, development and build dependencies under all target
    /// conditions. Inheritance retains both declarations and their manifest base;
    /// this does not establish active features, a lock edge or source provenance.
    /// An unknown package identity returns `None` rather than another package's data.
    #[must_use]
    pub fn dependency_declarations(&self, package: PackageId) -> Option<&[DependencyDeclaration]> {
        self.dependencies.get(package.index()).map(Vec::as_slice)
    }

    /// Checked source/manifest identity of a package admitted by a locked load.
    /// This records provenance, not active cfg/features or provider-policy acceptance.
    #[must_use]
    pub fn locked_binding(&self, package: PackageId) -> Option<&LockedBinding> {
        self.locked.get(package.index())
    }

    /// A package by its load-local identity.
    #[must_use]
    pub fn package(&self, id: PackageId) -> Option<&Package> {
        self.packages.get(id.index())
    }

    /// Package identities in the manifest order of [`Workspace::packages`].
    pub fn package_ids(&self) -> impl Iterator<Item = PackageId> {
        (0..self.packages.len()).map(PackageId)
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

    /// Every item, sorted by path, then name space, file and position.
    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.index
            .order
            .iter()
            .map(|id| &self.index.items[id.index()])
    }

    /// The items at `prefix` or below it, in the order of [`Workspace::items`].
    /// The prefix matches whole path segments: `demo::a` selects `demo::a` and
    /// `demo::a::b`, not `demo::ab`. An empty prefix selects every item.
    pub fn items_under<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = &'a Item> {
        let order = &self.index.order;
        let start = order.partition_point(|id| self.index.items[id.index()].path.as_str() < prefix);
        order[start..]
            .iter()
            .map(|id| &self.index.items[id.index()])
            .take_while(move |item| item.path.starts_with(prefix))
            .filter(move |item| {
                prefix.is_empty()
                    || item
                        .path
                        .strip_prefix(prefix)
                        .is_some_and(|rest| rest.is_empty() || rest.starts_with("::"))
            })
    }

    /// One item by id.
    #[must_use]
    pub fn item(&self, id: ItemId) -> Option<&Item> {
        self.index.items.get(id.index())
    }

    /// A conservative platform guard for one indexed item's retained source site.
    ///
    /// This observes source facts without filtering the index or assuming the
    /// compiler's host. Unsupported owner/path ancestry remains unknown.
    ///
    /// # Errors
    /// Returns a typed load error for an invalid item, inconsistent retained
    /// source context, malformed cfg metadata, or the original storage/work limit.
    pub fn test_platform_guard(
        &self,
        id: ItemId,
    ) -> Result<crate::TestPlatformGuard<'_>, LoadError> {
        crate::platform::guard(self, id, &self.index, &self.options, self.usage.item_bytes)
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

    /// The number of indexed items in `file`, without scanning their spans.
    ///
    /// Includes crate roots and every distinct source reach, as
    /// [`Workspace::items_in`] does. An unindexed file has zero items.
    #[must_use]
    pub fn item_count_in(&self, file: &RelPath) -> usize {
        self.index.by_file.get(file).map_or(0, Vec::len)
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
    /// [`ResolveError::UnresolvedSelf`] when it defines nothing there and the
    /// path goes through the self type of an inherent impl that the index
    /// cannot see, [`ResolveError::Ambiguous`] when it names several symbols
    /// or starts with a crate path that targets of several packages share
    /// (see [`Workspace::resolve_in_package`]), [`ResolveError::Cyclic`] when
    /// the imports on the way form a cycle,
    /// [`ResolveError::UnresolvedGlob`] when a lookup on the way reads a glob
    /// whose path names nothing past its first segment, and
    /// [`ResolveError::Invalid`] when the text is not a path, a path keyword
    /// among its segments included.
    pub fn resolve(&self, path: &str) -> Result<Resolution, ResolveError> {
        self.index.resolve(path, None, None)
    }

    /// Resolve a canonical path in one name space.
    ///
    /// # Errors
    ///
    /// As [`Workspace::resolve`].
    pub fn resolve_in(&self, path: &str, namespace: Namespace) -> Result<Resolution, ResolveError> {
        self.index.resolve(path, Some(namespace), None)
    }

    /// Resolve a canonical path among the crates of one package, named by its
    /// Cargo package name: the path starts with the crate path of one of the
    /// package's targets. Two packages whose libraries share a crate name
    /// are told apart this way.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError::UnknownPackage`] when no package of the
    /// workspace has that name, [`ResolveError::AmbiguousPackageName`] when
    /// several have it, and otherwise as [`Workspace::resolve`].
    pub fn resolve_in_package(
        &self,
        package: &str,
        path: &str,
    ) -> Result<Resolution, ResolveError> {
        let matching = try_equal_range_by(&self.package_names, |id| {
            Ok::<_, Infallible>(self.packages[id.index()].name.as_str().cmp(package))
        })
        .unwrap_or_else(|never| match never {});
        if matching.is_empty() {
            return Err(ResolveError::UnknownPackage(package.to_owned()));
        }
        if matching.len() > 1 {
            return Err(ResolveError::AmbiguousPackageName(package.to_owned()));
        }
        self.resolve_in_package_id(self.package_names[matching.start], path)
    }

    /// Resolve a canonical path among the crates of an exact load-local package.
    ///
    /// Displayed crate and symbol paths are unchanged; the identity chooses
    /// which package's roots the ordinary resolver walks.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError::UnknownPackageId`] for an identity absent from
    /// this workspace, and otherwise as [`Workspace::resolve`].
    pub fn resolve_in_package_id(
        &self,
        package: PackageId,
        path: &str,
    ) -> Result<Resolution, ResolveError> {
        if self.package(package).is_none() {
            return Err(ResolveError::UnknownPackageId(package));
        }
        self.index.resolve(path, None, Some(package))
    }

    /// Resolve the path of a `use` declaration written in the module `module`
    /// (a canonical module path such as `demo::inner`), trying the type, value
    /// and macro name spaces in that order. The reading module is the viewer:
    /// the path starts as code of that module starts it (`crate`, `self`,
    /// `super`, a name the module binds, or a crate of the extern prelude)
    /// and each segment counts only the bindings the module can access, as
    /// rustc settles an import.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError::UnknownModule`] when `module` is not a module
    /// of the workspace, and otherwise as [`Workspace::resolve`]: a path
    /// that names nothing the module can access is
    /// [`ResolveError::NotFound`], and one that names several symbols is
    /// [`ResolveError::Ambiguous`]. So is a `module` that starts with a
    /// crate path that targets of several packages share (two libraries with
    /// one crate name), as for [`Workspace::resolve`]: the answer lists the
    /// crate roots of those packages, and a module of any other crate reads
    /// as before. Nothing names the package here, so such a module cannot be
    /// read from.
    ///
    /// # Examples
    ///
    /// ```
    /// use raf_source::{MemoryTree, Workspace};
    ///
    /// let tree: MemoryTree = [
    ///     ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
    ///     ("src/lib.rs", "mod a { pub fn go() {} fn hidden() {} }\nmod b {}\n"),
    /// ]
    /// .into_iter()
    /// .collect();
    /// let ws = Workspace::load(&tree, &["."])?;
    /// assert_eq!(ws.resolve_from("demo::b", "crate::a::go")?.path, "demo::a::go");
    /// assert!(ws.resolve_from("demo::b", "crate::a::hidden").is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn resolve_from(&self, module: &str, path: &str) -> Result<Resolution, ResolveError> {
        self.index.resolve_from(module, path, None, None)
    }

    /// [`Workspace::resolve_from`] in one name space.
    ///
    /// # Errors
    ///
    /// As [`Workspace::resolve_from`].
    pub fn resolve_from_in(
        &self,
        module: &str,
        path: &str,
        namespace: Namespace,
    ) -> Result<Resolution, ResolveError> {
        self.index.resolve_from(module, path, Some(namespace), None)
    }

    /// Resolve a relative import path from a module of an exact load-local package.
    ///
    /// The package selects the reading module's crate roots, even when another
    /// package has the same displayed crate and module paths. Resolution then
    /// uses the ordinary import traversal and the reading module's visibility.
    /// The result can name a dependency of that package; it is not restricted
    /// to definitions in the reading package.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveError::UnknownPackageId`] for an identity absent from
    /// this workspace, [`ResolveError::UnknownModule`] when the module is absent
    /// from the selected package, and otherwise as [`Self::resolve_from`].
    pub fn resolve_from_package_id(
        &self,
        package: PackageId,
        module: &str,
        path: &str,
    ) -> Result<Resolution, ResolveError> {
        if self.package(package).is_none() {
            return Err(ResolveError::UnknownPackageId(package));
        }
        self.index.resolve_from(module, path, None, Some(package))
    }

    /// [`Self::resolve_from_package_id`] in one name space.
    ///
    /// # Errors
    ///
    /// As [`Self::resolve_from_package_id`].
    pub fn resolve_from_package_id_in(
        &self,
        package: PackageId,
        module: &str,
        path: &str,
        namespace: Namespace,
    ) -> Result<Resolution, ResolveError> {
        if self.package(package).is_none() {
            return Err(ResolveError::UnknownPackageId(package));
        }
        self.index
            .resolve_from(module, path, Some(namespace), Some(package))
    }

    /// Resolve a definition only through a declared, cfg-filtered locked load.
    ///
    /// Every returned definition belongs to an actual admitted package with its
    /// retained compilation context. This does not expand macros, observe a
    /// Cargo invocation or decide preferred-provider policy.
    /// Filtering selects modeled declaration headers, not function-body
    /// expressions/statements, fields or enum variants; source bytes are intact.
    ///
    /// # Errors
    /// Refuses an absent package, structural-only context or unconfigured site;
    /// missing/inactive names and ordinary resolver failures remain hard errors.
    pub fn resolve_active_from_package_id_in(
        &self,
        package: PackageId,
        module: &str,
        path: &str,
        namespace: Namespace,
    ) -> Result<Resolution, ResolveError> {
        if self.package(package).is_none() {
            return Err(ResolveError::UnknownPackageId(package));
        }
        if self
            .locked_binding(package)
            .and_then(LockedBinding::configuration)
            .is_none()
        {
            return Err(ResolveError::UnconfiguredPackage(package));
        }
        let resolved = self.resolve_from_package_id_in(package, module, path, namespace)?;
        if resolved.sites.is_empty() {
            return Err(ResolveError::NotFound(path.to_owned()));
        }
        for site in &resolved.sites {
            let valid = self
                .item(*site)
                .and_then(|item| self.target(item.target))
                .and_then(|target| self.locked_binding(target.package_id))
                .and_then(LockedBinding::configuration)
                .is_some();
            if !valid {
                return Err(ResolveError::UnconfiguredDefinition(*site));
            }
        }
        Ok(resolved)
    }

    /// The inherent impl blocks whose self type names no struct, enum or
    /// union the index can see, once for each module path their file is
    /// reached under, sorted by path, file and position. Their members are
    /// items marked [`SelfType::Unresolved`](crate::SelfType::Unresolved), so a
    /// consumer that needs every impl placed can refuse them.
    pub fn unresolved_impls(&self) -> impl Iterator<Item = &UnresolvedImpl> {
        self.index.unresolved.iter()
    }

    /// Every `impl` block of every target, inherent and of a trait alike, once
    /// for each module path its file is reached under, in file order: sorted
    /// by file, the byte offset where the block starts, module path and
    /// target. A block's [`ImplId`] is its place in this order. Each carries
    /// its span from its first attribute or the `impl` keyword to its closing
    /// brace, the module that holds it, its written self type and trait, the
    /// key of its self type (the struct, enum or union the self type path
    /// names from that module, none when it names nothing of the workspace or
    /// a type the index cannot place) and its members; a negative impl keeps
    /// its `!` in its trait path. An impl block inside a
    /// function body or any other block expression, such as
    /// `const _: () = { .. };`, is not an item of a module and is not listed,
    /// nor is one that a macro invocation would generate.
    ///
    /// ```
    /// use raf_source::{MemoryTree, Workspace};
    ///
    /// let tree = MemoryTree::from([
    ///     ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///     ("src/lib.rs", "pub struct S;\nimpl S { pub fn new() {} }\nimpl Clone for S { fn clone(&self) -> S { S } }\n"),
    /// ]);
    /// let ws = Workspace::load(&tree, &["."])?;
    /// let blocks: Vec<_> = ws.impls().collect();
    /// assert_eq!(blocks.len(), 2);
    /// assert_eq!((blocks[0].line, blocks[1].line), (2, 3));
    /// let s = ws.resolve("demo::S")?.sites[0];
    /// assert!(blocks.iter().all(|block| block.self_items == [s]));
    /// assert_eq!(blocks[1].trait_path.as_deref(), Some("Clone"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn impls(&self) -> impl Iterator<Item = &ImplBlock> {
        self.index.impl_blocks.iter()
    }

    /// One impl block by id.
    #[must_use]
    pub fn impl_block(&self, id: ImplId) -> Option<&ImplBlock> {
        self.index.impl_blocks.get(id.index())
    }

    /// The items and impl blocks of every target that `roots` reach over
    /// three edges: contains (a struct, enum or union to its methods and
    /// associated items, a trait to its members, a module to its items and
    /// impl blocks, a crate root to nothing), impl-of (a struct, enum or
    /// union to the impl blocks whose self type it is, and a block to its
    /// members) and references (an item or a block to each item that a path
    /// in its own tokens names from its module). A module is opened only
    /// when it is a root or is reached by contains from one that is opened:
    /// a module that a path names reaches nothing below it.
    ///
    /// A path is an `Ident (:: Ident)*` run of the tokens that no `.`
    /// precedes, read as code of the module reads it: each proper prefix in
    /// the type name space and the whole run in each name space, each walked
    /// as the resolver walks a path, as code reads it where
    /// [`Workspace::resolve_from_in`] reads a `use` path (without the reading
    /// module's visibility, and in edition 2015 from the module), naming the
    /// sites it reaches when they are one symbol; a path that names nothing
    /// of the workspace (a local, a generic, an item of a dependency or one
    /// that a macro generates) reaches nothing. Keywords are those of the
    /// edition of the package of the reached item. A path that `Self` starts
    /// in an impl block or a trait, and a method called on `self` there,
    /// name the members of that name of the block's self type, of the block
    /// and of its trait, or of the trait. The own tokens of an item are those
    /// of its span outside the items and blocks below it; a module's are its
    /// declaration before its body. The tokens of test code are read as test
    /// code, which names the dev-dependencies too.
    ///
    /// Each reached node is listed once, with the edge it was first reached
    /// by, breadth first from the roots in the order of their nodes, so the
    /// answer does not depend on the order of `roots`. Each file is lexed
    /// once per package edition, and each path is resolved at most once for
    /// each module it is read from. What the walk holds is charged to the
    /// item-bytes budget of the load, on from what the load holds.
    ///
    /// ```
    /// use raf_source::{MemoryTree, Node, Workspace};
    ///
    /// let tree = MemoryTree::from([
    ///     ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
    ///     ("src/lib.rs", "pub fn run() { helper() }\nfn helper() {}\nfn unused() {}\n"),
    /// ]);
    /// let ws = Workspace::load(&tree, &["."])?;
    /// let site = |path: &str| ws.resolve(path).map(|found| found.sites[0]);
    /// let reached = ws.reachable(&[Node::Item(site("demo::run")?)])?;
    /// assert!(reached.contains(Node::Item(site("demo::helper")?)));
    /// assert!(!reached.contains(Node::Item(site("demo::unused")?)));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`ReachError::UnknownItem`] or [`ReachError::UnknownImpl`] for
    /// a root past the items or the impl blocks of this workspace (an id is
    /// a place, so an id of another workspace that is in range reads as the
    /// node of this one at that place), [`ReachError::Source`] when a file
    /// does not lex, which a file the load parsed does not do, or the thread
    /// that lexes it cannot start, and [`ReachError::Budget`] when what the
    /// walk holds passes the item-bytes budget.
    pub fn reachable(&self, roots: &[Node]) -> Result<Reachability, ReachError> {
        reach::reachable(
            &self.index,
            &self.files,
            (&self.options, self.usage.item_bytes),
            roots,
        )
    }

    /// The items that share a source item, grouped by its identity: the file
    /// and the byte offset where its span starts. A source item that several
    /// module paths reach appears as several items, and the group holds them
    /// all, sorted by path. Groups come in the order of file and offset. Crate
    /// roots represent whole files and are excluded from these source groups.
    pub fn items_by_source(&self) -> impl Iterator<Item = (&RelPath, usize, &[ItemId])> {
        self.index
            .by_source
            .iter()
            .map(|((file, offset), ids)| (file, *offset, ids.as_slice()))
    }

    /// The items that share a source item with `item`, itself included. A crate
    /// root represents a whole file rather than a source item and returns an
    /// empty slice, including when its first written item also starts at byte 0.
    #[must_use]
    pub fn same_source(&self, item: &Item) -> &[ItemId] {
        if self.index.is_root(item.id) {
            return &[];
        }
        self.index
            .by_source
            .get(&(item.file.clone(), item.span.start_byte))
            .map_or(&[][..], Vec::as_slice)
    }

    /// The public surface of a library or binary target, with the items that
    /// `#[doc(hidden)]` hides left out: see [`Workspace::exported_items_with`].
    ///
    /// ```rust
    /// use raf_source::{MemoryTree, Workspace};
    ///
    /// let tree = MemoryTree::from([
    ///     ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///     ("src/lib.rs", "mod inner;\npub use inner::go as run;\npub(crate) fn helper() {}\n"),
    ///     ("src/inner.rs", "pub fn go() {}\npub fn unused() {}\n"),
    /// ]);
    /// let ws = Workspace::load(&tree, &["."])?;
    /// let lib = ws.target_ids().next().expect("the library");
    /// let exports = ws.exported_items(lib)?;
    /// let paths: Vec<(&str, &str)> = exports
    ///     .items
    ///     .iter()
    ///     .map(|export| (export.path.as_str(), ws.item(export.item).map_or("", |item| item.path.as_str())))
    ///     .collect();
    /// assert_eq!(paths, [("demo::run", "demo::inner::go")]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Workspace::exported_items_with`].
    pub fn exported_items(&self, target: TargetId) -> Result<Exports, ExportError> {
        self.exported_items_with(target, &ExportOptions::default())
    }

    /// The public surface of a library or binary target: the items another
    /// crate can name through its crate root, each under the path it is
    /// named by, and the public re-exports that name nothing the target
    /// defines.
    ///
    /// An item is exported when a chain of `pub` declarations reaches it from
    /// the crate root: `pub` modules, and `pub use` re-exports, named or glob,
    /// renamed or not, through modules of any visibility. A `pub` item of a
    /// private module is exported only through such a re-export, and a
    /// `pub(crate)`, `pub(super)`, `pub(in path)` or private item never is,
    /// nor is one that a re-export that is not `pub` carries on the way.
    /// Test code is never exported. The members of an exported type or trait
    /// are listed under its path: each member of a trait, and each `pub`
    /// member of an inherent impl of a type; the members of trait impls are
    /// not. A binary's surface is read the same way from its crate root; no
    /// other crate can name it. A re-export of another crate, of an enum
    /// variant or of a path the index cannot follow is a
    /// [`Reexport`](crate::Reexport) and not an item, also where a `pub` glob
    /// carries it from a module it reaches.
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::NotLibraryOrBinary`] for a target id that names
    /// no library or binary of the workspace, [`ExportError::Resolve`] when a
    /// public name is ambiguous, goes through a cycle of imports or reads a
    /// glob whose path names nothing in the workspace, and
    /// [`ExportError::Budget`] when what the surface holds passes the
    /// item-bytes budget left after the load, and
    /// [`ExportError::CarryBudget`], naming the re-export, when the work of
    /// deciding whether the public globs carry a re-export passes it.
    pub fn exported_items_with(
        &self,
        target: TargetId,
        options: &ExportOptions,
    ) -> Result<Exports, ExportError> {
        let Some(found) = self
            .targets
            .get(target.index())
            .filter(|found| matches!(found.kind, TargetKind::Lib | TargetKind::Bin))
        else {
            return Err(ExportError::NotLibraryOrBinary(target));
        };
        let meter = Meter::new(&self.options);
        meter
            .items(self.usage.item_bytes, &found.root, None)
            .map_err(ExportError::Budget)?;
        // The modules of the target whose own file has `#![doc(hidden)]`.
        let hidden_files: BTreeSet<&str> = self
            .files
            .values()
            .filter(|file| file.facts.doc_hidden)
            .flat_map(|file| &file.reached)
            .filter(|reach| reach.target == target)
            .map(|reach| reach.module_path.as_str())
            .collect();
        Surface::new(
            &self.index,
            target,
            &found.crate_path,
            options,
            &meter,
            &hidden_files,
        )
        .list()
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
    workspaces: BTreeMap<RelPath, WorkspaceOf>,
    members: BTreeMap<RelPath, BTreeSet<RelPath>>,
}

/// Build the name selector without retaining another copy of package names.
fn package_name_index(packages: &[Package], meter: &Meter) -> Result<Vec<PackageId>, LoadError> {
    package_index_by(packages.len(), meter, |left, right| {
        packages[left.index()]
            .name
            .cmp(&packages[right.index()].name)
    })
}

fn package_directory_index(found: &[Found], meter: &Meter) -> Result<Vec<PackageId>, LoadError> {
    package_index_by(found.len(), meter, |left, right| {
        found[left.index()]
            .package
            .dir
            .cmp(&found[right.index()].package.dir)
    })
}

/// Sort borrowed package keys through native admitted identity storage.
fn package_index_by(
    len: usize,
    meter: &Meter,
    compare: impl Fn(PackageId, PackageId) -> Ordering,
) -> Result<Vec<PackageId>, LoadError> {
    let file = RelPath::root();
    meter.items(size_of::<Vec<PackageId>>(), &file, None)?;
    let mut names = Vec::new();
    let mut admission = ItemAdmission::new(meter, &file);
    Memory::new(&mut admission)
        .reserve(&mut names, len)
        .map_err(|error| admission.error(error))?;
    for index in 0..len {
        names.push(PackageId(index));
    }
    try_sort_unstable_by(&mut names, |left, right| {
        Ok::<_, Infallible>(compare(*left, *right).then_with(|| left.cmp(right)))
    })
    .unwrap_or_else(|never| match never {});
    Ok(names)
}

/// What reading a package's manifest needs: the tree, the meter of the load
/// and the directory of every package discovery found.
struct Reader<'a> {
    tree: &'a dyn Tree,
    meter: &'a Meter,
    dirs: &'a [PackageId],
    found: &'a [Found],
}

/// The nearest workspace manifest above a package, as far as the package can
/// inherit from it.
#[derive(Debug, Clone, PartialEq, Eq)]
enum WorkspaceOf {
    /// The package is a member of the workspace in `dir`.
    Member {
        dir: RelPath,
        /// `[workspace.package] edition`, if it is set.
        edition: Option<u16>,
        version: Option<Rc<String>>,
        /// `[workspace.dependencies]`.
        dependencies: Rc<BTreeMap<String, WorkspaceDependency>>,
    },
    /// The nearest workspace manifest does not have the package as a member.
    NotMember(RelPath),
    /// There is no workspace manifest above the package.
    Absent,
}

fn manifest_error(path: &RelPath, key: &str, message: String) -> LoadError {
    LoadError::new(path, None, LoadErrorKind::Manifest { message }).reached_by(
        &Origin::ManifestKey {
            key: key.to_owned(),
        },
    )
}

/// The workspace that a package directory belongs to: the nearest manifest at
/// or above it that has a `[workspace]` table, found the way cargo finds it,
/// by walking up through the ancestors within the tree. A missing ancestor
/// manifest is skipped; one that cannot be read or parsed is an error. The
/// manifest is not added to the packages that discovery found.
fn workspace_of(
    reader: &Reader<'_>,
    dir: &RelPath,
    caches: &mut Caches,
) -> Result<WorkspaceOf, LoadError> {
    let (tree, meter) = (reader.tree, reader.meter);
    if let Some(known) = caches.workspaces.get(dir) {
        return Ok(known.clone());
    }
    let origin = Origin::Discovery;
    let mut at = Some(dir.clone());
    let mut found = WorkspaceOf::Absent;
    while let Some(current) = at {
        let path = current
            .join("Cargo.toml")
            .map_err(|error| LoadError::tree(&current, None, &origin, error.into()))?;
        let present = tree
            .entry(&path)
            .map_err(|error| LoadError::tree(&path, None, &origin, error))?;
        if present.is_some() {
            let text = tree
                .read(&path, meter.options().max_file_bytes)
                .map_err(|error| LoadError::tree(&path, None, &origin, error))?;
            meter
                .read(&path, &text)
                .map_err(|error| error.reached_by(&origin))?;
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
                        slot.insert(members(tree, meter, &current, &path, &parsed)?)
                    }
                };
                found = if set.contains(dir) {
                    let version = parsed
                        .workspace_version
                        .map(|version| {
                            meter.items(
                                version
                                    .len()
                                    .saturating_add(size_of::<String>())
                                    .saturating_add(2 * size_of::<usize>()),
                                &path,
                                None,
                            )?;
                            Ok::<_, LoadError>(Rc::new(version))
                        })
                        .transpose()?;
                    WorkspaceOf::Member {
                        dir: current,
                        edition: parsed.workspace_edition,
                        version,
                        dependencies: Rc::new(parsed.workspace_dependencies),
                    }
                } else {
                    WorkspaceOf::NotMember(path)
                };
                break;
            }
        }
        at = current.parent();
    }
    caches.workspaces.insert(dir.clone(), found.clone());
    Ok(found)
}

/// What the index needs to know about a package's manifest.
///
/// # Errors
///
/// Returns a manifest error naming `package.edition` when the package inherits
/// its edition and the nearest workspace manifest above it does not have the
/// package as a member or does not set an edition, which cargo rejects as well,
/// and one naming a dependency whose `path` cannot be a path here.
fn package_info(
    reader: &Reader<'_>,
    package: &Package,
    declared: &PackageDecl,
    manifest: &Manifest,
    caches: &mut Caches,
) -> Result<PackageInfo, LoadError> {
    let edition = match &declared.edition {
        EditionDecl::Explicit(edition) => *edition,
        EditionDecl::Unset => 2015,
        EditionDecl::Inherited => match workspace_of(reader, &package.dir, caches)? {
            WorkspaceOf::Member {
                edition: Some(edition),
                ..
            } => edition,
            WorkspaceOf::Member { edition: None, .. } | WorkspaceOf::Absent => {
                return Err(manifest_error(
                        &package.manifest,
                        "package.edition",
                        "`edition.workspace = true` but no `[workspace.package] edition` is set by the nearest workspace manifest above the package"
                            .to_owned(),
                    ));
            }
            WorkspaceOf::NotMember(workspace) => {
                return Err(manifest_error(
                    &package.manifest,
                    "package.edition",
                    format!(
                        "`edition.workspace = true` but the package is not a member of the workspace {workspace} (it is not listed in `workspace.members`, or is excluded)"
                    ),
                ));
            }
        },
    };
    let mut dependencies = Vec::new();
    for dependency in &manifest.dependencies {
        dependencies.push(dependency_info(reader, package, dependency, caches)?);
    }
    Ok(PackageInfo {
        edition,
        dependencies,
    })
}

/// Retain package and inherited declarations under the original item allowance.
/// Membership and inherited keys have already been checked by `package_info`.
fn dependency_declarations(
    reader: &Reader<'_>,
    package: &Package,
    manifest: &Manifest,
    caches: &mut Caches,
) -> Result<Vec<DependencyDeclaration>, LoadError> {
    let mut declarations = Vec::new();
    let mut admission = ItemAdmission::new(reader.meter, &package.manifest);
    Memory::new(&mut admission)
        .reserve(&mut declarations, manifest.dependencies.len())
        .map_err(|error| admission.error(error))?;
    for dependency in &manifest.dependencies {
        let inherited = if dependency.declaration.specification.workspace {
            // package_info has already validated membership and the inherited key.
            let WorkspaceOf::Member {
                dir, dependencies, ..
            } = workspace_of(reader, &package.dir, caches)?
            else {
                return Err(internal(
                    &package.manifest,
                    None,
                    "validated workspace dependency lost its workspace".to_owned(),
                ));
            };
            let entry = dependencies.get(&dependency.name).ok_or_else(|| {
                internal(
                    &package.manifest,
                    None,
                    "validated workspace dependency lost its declaration".to_owned(),
                )
            })?;
            let manifest = dir
                .join("Cargo.toml")
                .map_err(|error| internal(&package.manifest, None, error.to_string()))?;
            reader.meter.items(
                manifest
                    .as_str()
                    .len()
                    .saturating_add(entry.specification.heap_bytes()),
                &package.manifest,
                None,
            )?;
            Some(InheritedDependency {
                manifest,
                specification: entry.specification.clone(),
            })
        } else {
            None
        };
        let original = &dependency.declaration;
        let bytes = original
            .key
            .len()
            .saturating_add(original.name.len())
            .saturating_add(original.condition.as_ref().map_or(0, String::len))
            .saturating_add(original.specification.heap_bytes());
        reader.meter.items(bytes, &package.manifest, None)?;
        let mut declaration = original.clone();
        declaration.inherited = inherited;
        declarations.push(declaration);
    }
    Ok(declarations)
}

/// What the extern prelude needs, including an inherited path's workspace base.
/// Refuses invalid inheritance and malformed paths; external sources remain
/// unbound until a verified lock/source join supplies their package identity.
fn dependency_info(
    reader: &Reader<'_>,
    package: &Package,
    dependency: &Dependency,
    caches: &mut Caches,
) -> Result<DependencyInfo, LoadError> {
    let mut renamed = dependency.package.is_some();
    let place = match &dependency.path {
        None => None,
        Some(PathSource::Own(path)) => Some((package.dir.clone(), path.clone())),
        Some(PathSource::Workspace(name)) => match workspace_of(reader, &package.dir, caches)? {
            WorkspaceOf::Member {
                dir, dependencies, ..
            } => {
                let Some(entry) = dependencies.get(name) else {
                    return Err(manifest_error(
                        &package.manifest,
                        &dependency.key,
                        format!(
                            "`workspace = true` but the workspace in {dir} has no `[workspace.dependencies]` entry `{name}`"
                        ),
                    ));
                };
                renamed |= entry.package.is_some();
                entry.path.clone().map(|path| (dir, path))
            }
            WorkspaceOf::NotMember(workspace) => {
                return Err(manifest_error(
                    &package.manifest,
                    &dependency.key,
                    format!(
                        "`workspace = true` but the package is not a member of the workspace {workspace} (it is not listed in `workspace.members`, or is excluded)"
                    ),
                ));
            }
            WorkspaceOf::Absent => {
                return Err(manifest_error(
                    &package.manifest,
                    &dependency.key,
                    "`workspace = true` but there is no workspace manifest above the package"
                        .to_owned(),
                ));
            }
        },
    };
    let named = match place {
        None => None,
        Some((base, relative)) => match base.join(&relative) {
            Ok(joined) => {
                let target = tidy(reader.tree, &joined).map_err(|error| {
                    LoadError::tree(&package.manifest, None, &Origin::Discovery, error)
                })?;
                let matching = try_equal_range_by(reader.dirs, |id| {
                    Ok::<_, Infallible>(reader.found[id.index()].package.dir.cmp(&target))
                })
                .unwrap_or_else(|never| match never {});
                reader
                    .dirs
                    .get(matching.start)
                    .copied()
                    .filter(|_| !matching.is_empty())
            }
            // A path that leaves the tree, or is absolute, names no package
            // of the workspace.
            Err(PathError::AboveRoot { .. } | PathError::Absolute { .. }) => None,
            Err(other) => {
                return Err(manifest_error(
                    &package.manifest,
                    &dependency.key,
                    other.to_string(),
                ));
            }
        },
    };
    Ok(DependencyInfo {
        kind: dependency.kind,
        name: dependency.name.replace('-', "_"),
        renamed,
        package: named,
    })
}

fn walk_sources(
    tree: &dyn Tree,
    targets: &[Target],
    origins: &[Origin],
    bindings: &[LockedBinding],
    meter: &Meter,
) -> Result<Walked, LoadError> {
    let mut walker = Walker::new(tree, meter);
    for (index, (target, origin)) in targets.iter().zip(origins).enumerate() {
        walker.configure(
            bindings
                .get(target.package_id.index())
                .and_then(|proof| proof.configuration())
                .map(|config| (target.package_id, config)),
        );
        walker.root(TargetId(index), &target.root, &target.crate_path, origin)?;
    }
    walker.finish()
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
    use crate::budget::Budget;
    use crate::tree::MemoryTree;

    #[test]
    fn native_package_name_index_keeps_identity_and_enforces_its_exact_grant() {
        let packages: Vec<_> = (0..24)
            .map(|index| Package {
                name: format!("package-{:02}", 23 - index),
                manifest: RelPath::new(&format!("p{index}/Cargo.toml")).expect("manifest"),
                dir: RelPath::new(&format!("p{index}")).expect("directory"),
            })
            .collect();
        let meter = Meter::new(&LoadOptions::default());
        let names = package_name_index(&packages, &meter).expect("native index");
        assert_eq!(names, (0..24).rev().map(PackageId).collect::<Vec<_>>());
        let charged = meter.usage().item_bytes;
        let exact = Meter::new(&LoadOptions {
            max_item_bytes: charged,
            ..LoadOptions::default()
        });
        assert_eq!(
            package_name_index(&packages, &exact).expect("exact grant"),
            names
        );
        let insufficient = Meter::new(&LoadOptions {
            max_item_bytes: charged - 1,
            ..LoadOptions::default()
        });
        let error = package_name_index(&packages, &insufficient).expect_err("one byte short");
        assert!(matches!(error.kind, LoadErrorKind::Budget {
            budget: Budget::ItemBytes,
            limit,
        } if limit == charged - 1));
        assert_eq!(error.file, RelPath::root());
    }

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
        item.path = "other::case".into();
        let error = test_fn_of(&item, &ws.targets).expect_err("the path leaves the crate");
        assert!(
            matches!(error.kind, LoadErrorKind::Internal { .. }),
            "{error}"
        );
        assert_eq!(error.line, Some(2));
        item.path = "demo::case".into();
        item.target = TargetId(9);
        assert!(test_fn_of(&item, &ws.targets).is_err());
    }
}
