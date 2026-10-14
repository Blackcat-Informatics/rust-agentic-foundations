// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Package targets: the library, binaries, tests, benches, examples and build
//! script of a package, with the crate path each one is indexed under.

use crate::error::{LoadError, LoadErrorKind, Origin, TargetProblem, TreeError};
use crate::manifest::{Build, Manifest, PackageDecl, TargetDecl, TargetSettings};
use crate::modtree::tidy;
use crate::path::RelPath;
use crate::tree::{EntryKind, Tree};

/// What a target is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum TargetKind {
    /// The library.
    Lib,
    /// A binary.
    Bin,
    /// An integration test.
    Test,
    /// A benchmark.
    Bench,
    /// An example.
    Example,
    /// The build script.
    BuildScript,
}

impl TargetKind {
    /// True for code that ships: the library, the binaries and the build
    /// script, which is part of what a package builds with. Tests, benches and
    /// examples do not.
    #[must_use]
    pub const fn is_shipping(self) -> bool {
        matches!(self, Self::Lib | Self::Bin | Self::BuildScript)
    }
}

/// A package found under a root.
///
/// The struct is `#[non_exhaustive]`: a later field is not a breaking change,
/// so a pattern names the fields it reads and ends in `..`.
///
/// ```compile_fail,E0638
/// # fn names(package: &raf_source::Package) {
/// let raf_source::Package { name, manifest, dir } = package;
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Package {
    /// The name in `[package]`.
    pub name: String,
    /// The path of its `Cargo.toml`.
    pub manifest: RelPath,
    /// The directory that holds it.
    pub dir: RelPath,
}

/// One target of a package.
///
/// The test settings (`harness`, `required_features`, `test` and `doctest`)
/// are read from the target table, `[lib]` or `[[bin]]` and the others, and
/// take cargo's default when the table leaves them out or the target has no
/// table. The build script has no table and is never tested: its `harness`,
/// `test` and `doctest` are false and it requires no feature.
///
/// The struct is `#[non_exhaustive]`: a later field is not a breaking change,
/// so a pattern names the fields it reads and ends in `..`.
///
/// ```compile_fail,E0638
/// # fn names(target: &raf_source::Target) {
/// let raf_source::Target {
///     package, name, kind, crate_path, root, shipping,
///     harness, required_features, test, doctest,
/// } = target;
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[allow(
    clippy::struct_excessive_bools,
    reason = "harness, test and doctest are Cargo's boolean target keys, kept one to one"
)]
pub struct Target {
    /// The package name.
    pub package: String,
    /// The Cargo name of the target.
    pub name: String,
    /// What the target is.
    pub kind: TargetKind,
    /// The path items of this target are indexed under: the crate name for the
    /// library (`[lib] name`, or the package name, with `-` as `_`),
    /// `<pkg>::bin_crate::<bin>` for a binary,
    /// `<pkg>::{test,bench,example}_crate::<target>` for the others and
    /// `<pkg>::build_script` for the build script.
    pub crate_path: String,
    /// The root source file.
    pub root: RelPath,
    /// True for the library, the binaries and the build script.
    pub shipping: bool,
    /// `harness`: true when `cargo test` builds the target with the libtest
    /// harness, which is cargo's default for every target table.
    pub harness: bool,
    /// `required-features`: the features the target needs before cargo builds
    /// it, in manifest order; empty by default.
    pub required_features: Vec<String>,
    /// `test`: true when `cargo test` tests the target by default. Cargo's
    /// default is true for the library, the binaries and the tests and false
    /// for benches and examples.
    pub test: bool,
    /// `doctest`: true when `cargo test` runs the documentation tests of the
    /// target. Cargo's default is true for the library and false for every
    /// other target. The value is reported as the manifest writes it; cargo
    /// honours the key only for the library.
    pub doctest: bool,
}

/// A target together with what led to its root file.
pub(crate) struct Derived {
    pub(crate) target: Target,
    pub(crate) origin: Origin,
}

fn ident(name: &str) -> String {
    name.replace('-', "_")
}

/// Everything target derivation needs about one package.
pub(crate) struct Context<'a> {
    pub(crate) tree: &'a dyn Tree,
    pub(crate) manifest_path: &'a RelPath,
    pub(crate) dir: &'a RelPath,
    pub(crate) manifest: &'a Manifest,
    pub(crate) package: &'a PackageDecl,
}

impl Context<'_> {
    fn error(&self, origin: &Origin, kind: LoadErrorKind) -> LoadError {
        LoadError::new(self.manifest_path, None, kind).reached_by(origin)
    }

    fn tree_error(&self, origin: &Origin, error: TreeError) -> LoadError {
        LoadError::tree(self.manifest_path, None, origin, error)
    }

    /// `relative` below the package directory.
    fn join(&self, relative: &str, origin: &Origin) -> Result<RelPath, LoadError> {
        let joined = self
            .dir
            .join(relative)
            .map_err(|error| self.tree_error(origin, error.into()))?;
        tidy(self.tree, &joined).map_err(|error| self.tree_error(origin, error))
    }

    /// True when a file (or a symlink that may lead to one) is at `path`.
    fn is_file(&self, path: &RelPath, origin: &Origin) -> Result<bool, LoadError> {
        match self
            .tree
            .entry(path)
            .map_err(|error| self.tree_error(origin, error))?
        {
            // A special file counts as present so that reading it is refused,
            // not mistaken for a missing target.
            Some(EntryKind::File | EntryKind::Symlink | EntryKind::Other) => Ok(true),
            Some(EntryKind::Dir) | None => Ok(false),
        }
    }

    /// The entries of a package directory such as `tests`, if it is one.
    fn list(&self, relative: &str) -> Result<Vec<crate::tree::Entry>, LoadError> {
        let origin = Origin::Discovery;
        let path = self.join(relative, &origin)?;
        match self
            .tree
            .entry(&path)
            .map_err(|error| self.tree_error(&origin, error))?
        {
            None => Ok(Vec::new()),
            Some(EntryKind::Dir) => self
                .tree
                .list(&path)
                .map_err(|error| LoadError::tree(&path, None, &origin, error)),
            Some(EntryKind::Symlink) => {
                Err(LoadError::new(&path, None, LoadErrorKind::DirSymlink).reached_by(&origin))
            }
            Some(EntryKind::File | EntryKind::Other) => Err(LoadError::tree(
                &path,
                None,
                &origin,
                TreeError::NotRegular {
                    path: path.to_string(),
                },
            )),
        }
    }

    fn missing(&self, path: RelPath, origin: &Origin) -> LoadError {
        self.error(
            origin,
            LoadErrorKind::Target(Box::new(TargetProblem::MissingFile { path })),
        )
    }

    fn derive(
        &self,
        name: &str,
        kind: TargetKind,
        crate_path: String,
        root: RelPath,
        origin: Origin,
        settings: Option<&TargetSettings>,
    ) -> Derived {
        let defaults = TargetSettings::default();
        let settings = settings.unwrap_or(&defaults);
        let tested = kind != TargetKind::BuildScript;
        Derived {
            target: Target {
                package: self.package.name.clone(),
                name: name.to_owned(),
                kind,
                crate_path,
                root,
                shipping: kind.is_shipping(),
                harness: settings.harness.unwrap_or(tested),
                required_features: settings.required_features.clone(),
                test: settings.test.unwrap_or(matches!(
                    kind,
                    TargetKind::Lib | TargetKind::Bin | TargetKind::Test
                )),
                doctest: settings.doctest.unwrap_or(kind == TargetKind::Lib),
            },
            origin,
        }
    }

    fn key(table: &str, field: &str) -> Origin {
        Origin::ManifestKey {
            key: format!("{table}.{field}"),
        }
    }

    /// The root of the first existing candidate, or of the only one that
    /// exists; two candidates that both exist are ambiguous.
    fn pick(&self, candidates: &[String], origin: &Origin) -> Result<Option<RelPath>, LoadError> {
        let mut found: Vec<RelPath> = Vec::new();
        for candidate in candidates {
            let path = self.join(candidate, origin)?;
            if self.is_file(&path, origin)? {
                found.push(path);
            }
        }
        match found.as_slice() {
            [] => Ok(None),
            [only] => Ok(Some(only.clone())),
            [first, second, ..] => Err(self.error(
                origin,
                LoadErrorKind::Target(Box::new(TargetProblem::AmbiguousRoot {
                    first: first.clone(),
                    second: second.clone(),
                })),
            )),
        }
    }

    fn library(&self, out: &mut Vec<Derived>) -> Result<(), LoadError> {
        let explicit = self.manifest.lib.as_ref();
        let written = explicit.and_then(|lib| lib.path.as_deref());
        // The key that led to the root: `lib.path` when the manifest names
        // the file, the `[lib]` table when it only declares the library, and
        // discovery for the default library of a package without `[lib]`.
        let origin = match (written, explicit) {
            (Some(_), _) => Self::key("lib", "path"),
            (None, Some(_)) => Origin::ManifestKey {
                key: "lib".to_owned(),
            },
            (None, None) => Origin::Discovery,
        };
        let relative = written.unwrap_or("src/lib.rs");
        let root = self.join(relative, &origin)?;
        if self.is_file(&root, &origin)? {
            let name = explicit
                .and_then(|lib| lib.name.clone())
                .unwrap_or_else(|| self.package.name.clone());
            let crate_path = ident(&name);
            let settings = explicit.map(|lib| &lib.settings);
            out.push(self.derive(&name, TargetKind::Lib, crate_path, root, origin, settings));
            Ok(())
        } else if explicit.is_some() {
            Err(self.missing(root, &origin))
        } else {
            Ok(())
        }
    }

    fn binaries(&self, out: &mut Vec<Derived>) -> Result<(), LoadError> {
        let package = &self.package.name;
        for declared in &self.manifest.bins {
            let name = declared.name.clone().ok_or_else(|| self.no_name("bin"))?;
            let (origin, root) = if let Some(path) = &declared.path {
                let origin = Self::key("bin", "path");
                let root = self.join(path, &origin)?;
                if !self.is_file(&root, &origin)? {
                    return Err(self.missing(root, &origin));
                }
                (origin, root)
            } else {
                let origin = Self::key("bin", "name");
                let mut candidates = Vec::new();
                if &name == package {
                    candidates.push("src/main.rs".to_owned());
                }
                candidates.push(format!("src/bin/{name}.rs"));
                candidates.push(format!("src/bin/{name}/main.rs"));
                let Some(root) = self.pick(&candidates, &origin)? else {
                    let first = self.join(&candidates[0], &origin)?;
                    return Err(self.missing(first, &origin));
                };
                (origin, root)
            };
            let crate_path = format!("{}::bin_crate::{}", ident(package), ident(&name));
            out.push(self.derive(
                &name,
                TargetKind::Bin,
                crate_path,
                root,
                origin,
                Some(&declared.settings),
            ));
        }
        if !self.package.auto("bin") {
            return Ok(());
        }
        let origin = Origin::Discovery;
        let main = self.join("src/main.rs", &origin)?;
        if self.is_file(&main, &origin)? {
            self.add_inferred(out, package, TargetKind::Bin, main);
        }
        self.infer_from("src/bin", TargetKind::Bin, out)
    }

    fn no_name(&self, table: &str) -> LoadError {
        self.error(
            &Self::key(table, "name"),
            LoadErrorKind::Target(Box::new(TargetProblem::NoName)),
        )
    }

    /// Add an inferred target unless an explicit one has its name or root.
    fn add_inferred(&self, out: &mut Vec<Derived>, name: &str, kind: TargetKind, root: RelPath) {
        let taken = out
            .iter()
            .any(|d| d.target.kind == kind && (d.target.name == name || d.target.root == root));
        if !taken {
            let crate_path = self.crate_path(kind, name);
            out.push(self.derive(name, kind, crate_path, root, Origin::Discovery, None));
        }
    }

    fn crate_path(&self, kind: TargetKind, name: &str) -> String {
        let package = ident(&self.package.name);
        let segment = match kind {
            TargetKind::Bin => "bin_crate",
            TargetKind::Test => "test_crate",
            TargetKind::Bench => "bench_crate",
            TargetKind::Example => "example_crate",
            TargetKind::Lib | TargetKind::BuildScript => return ident(name),
        };
        format!("{package}::{segment}::{}", ident(name))
    }

    /// Inferred targets of a directory: `*.rs` files and `*/main.rs`.
    fn infer_from(
        &self,
        directory: &str,
        kind: TargetKind,
        out: &mut Vec<Derived>,
    ) -> Result<(), LoadError> {
        let origin = Origin::Discovery;
        for entry in self.list(directory)? {
            match entry.kind {
                EntryKind::File | EntryKind::Symlink => {
                    if let Some(stem) = entry.name.strip_suffix(".rs") {
                        let root = self.join(&format!("{directory}/{}", entry.name), &origin)?;
                        self.add_inferred(out, stem, kind, root);
                    }
                }
                EntryKind::Dir => {
                    let root =
                        self.join(&format!("{directory}/{}/main.rs", entry.name), &origin)?;
                    if self.is_file(&root, &origin)? {
                        self.add_inferred(out, &entry.name, kind, root);
                    }
                }
                EntryKind::Other => {}
            }
        }
        Ok(())
    }

    fn examples_like(
        &self,
        table: &str,
        directory: &str,
        kind: TargetKind,
        declared: &[TargetDecl],
        auto: bool,
        out: &mut Vec<Derived>,
    ) -> Result<(), LoadError> {
        for target in declared {
            let name = target.name.clone().ok_or_else(|| self.no_name(table))?;
            let (origin, root) = if let Some(path) = &target.path {
                let origin = Self::key(table, "path");
                let root = self.join(path, &origin)?;
                if !self.is_file(&root, &origin)? {
                    return Err(self.missing(root, &origin));
                }
                (origin, root)
            } else {
                let origin = Self::key(table, "name");
                let candidates = [
                    format!("{directory}/{name}.rs"),
                    format!("{directory}/{name}/main.rs"),
                ];
                let Some(root) = self.pick(&candidates, &origin)? else {
                    let first = self.join(&candidates[0], &origin)?;
                    return Err(self.missing(first, &origin));
                };
                (origin, root)
            };
            let crate_path = self.crate_path(kind, &name);
            out.push(self.derive(
                &name,
                kind,
                crate_path,
                root,
                origin,
                Some(&target.settings),
            ));
        }
        if auto {
            self.infer_from(directory, kind, out)?;
        }
        Ok(())
    }

    fn build_script(&self, out: &mut Vec<Derived>) -> Result<(), LoadError> {
        let (origin, relative) = match &self.package.build {
            Build::Disabled => return Ok(()),
            Build::Path(path) => (Self::key("package", "build"), path.as_str()),
            Build::Default => (Origin::Discovery, "build.rs"),
        };
        let root = self.join(relative, &origin)?;
        if self.is_file(&root, &origin)? {
            let crate_path = format!("{}::build_script", ident(&self.package.name));
            out.push(self.derive(
                "build-script-build",
                TargetKind::BuildScript,
                crate_path,
                root,
                origin,
                None,
            ));
            Ok(())
        } else if matches!(self.package.build, Build::Path(_)) {
            Err(self.missing(root, &origin))
        } else {
            Ok(())
        }
    }
}

/// Derive every target of a package, in a fixed order: library, binaries,
/// tests, benches, examples, build script.
///
/// # Errors
///
/// Returns a [`LoadError`] naming the manifest key at fault when a target has
/// no name, an explicit file is missing, a path leaves the tree, or two
/// targets share a root.
pub(crate) fn derive_targets(context: &Context<'_>) -> Result<Vec<Derived>, LoadError> {
    let mut out = Vec::new();
    context.library(&mut out)?;
    context.binaries(&mut out)?;
    let manifest = context.manifest;
    let package = context.package;
    context.examples_like(
        "test",
        "tests",
        TargetKind::Test,
        &manifest.tests,
        package.auto("test"),
        &mut out,
    )?;
    context.examples_like(
        "bench",
        "benches",
        TargetKind::Bench,
        &manifest.benches,
        package.auto("bench"),
        &mut out,
    )?;
    context.examples_like(
        "example",
        "examples",
        TargetKind::Example,
        &manifest.examples,
        package.auto("example"),
        &mut out,
    )?;
    context.build_script(&mut out)?;
    for (index, derived) in out.iter().enumerate() {
        if let Some(first) = out[..index]
            .iter()
            .find(|earlier| earlier.target.root == derived.target.root)
        {
            return Err(context.error(
                &derived.origin,
                LoadErrorKind::Target(Box::new(TargetProblem::SharedRoot {
                    root: derived.target.root.clone(),
                    other: first.target.name.clone(),
                })),
            ));
        }
    }
    Ok(out)
}

/// Refuse two targets of one package that share a crate path. Targets of
/// different packages may share one: the index keys crates by package.
///
/// # Errors
///
/// Returns a [`LoadError`] on the manifest of the later target, naming the
/// manifest key that declared it and the earlier target.
pub(crate) fn check_crate_paths(derived: &[(&RelPath, Derived)]) -> Result<(), LoadError> {
    for (index, (manifest, later)) in derived.iter().enumerate() {
        if let Some((first_manifest, first)) = derived[..index].iter().find(|(_, earlier)| {
            earlier.target.package == later.target.package
                && earlier.target.crate_path == later.target.crate_path
        }) {
            return Err(LoadError::new(
                manifest,
                None,
                LoadErrorKind::Target(Box::new(TargetProblem::DuplicateCratePath {
                    crate_path: later.target.crate_path.clone(),
                    other: first.target.name.clone(),
                    with: (*first_manifest).clone(),
                })),
            )
            .reached_by(&later.origin));
        }
    }
    Ok(())
}
