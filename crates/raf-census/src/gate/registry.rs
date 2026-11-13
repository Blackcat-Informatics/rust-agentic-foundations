// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The locked check over registry dependencies sealed by the lock file.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read as _;
use std::path::{Path, PathBuf};

use raf_source::{
    CfgAtom, DeclaredCfg, DeclaredConfiguration, DependencyDeclaration, DependencySpec,
    LoadOptions, LockedCatalog, LockedEdge, LockedManifest, LockedPackageId, LockedPackageRoots,
    LockedSourceKind, LockedTree, MemoryTree, PackageId, RegistryArchiveTree, RegistryIndex,
    RelPath, TargetKind, Tree, Workspace,
};

use super::{DebtWrite, Gate, GateReport, read_ledger};
use crate::Error;

/// The configuration every package is judged under: the primary Linux target.
const TARGET: &str = "x86_64-unknown-linux-gnu";
const ORDINARY: &[CfgAtom<'_>] = &[
    CfgAtom::Name("unix"),
    CfgAtom::Value("target_os", "linux"),
    CfgAtom::Value("target_family", "unix"),
    CfgAtom::Value("target_arch", "x86_64"),
    CfgAtom::Value("target_endian", "little"),
    CfgAtom::Value("target_pointer_width", "64"),
];
/// The declaration for a target built by `cargo test`, whose `#[test]` functions
/// and `cfg(test)` items it keeps.
const TESTING: &[CfgAtom<'_>] = &[
    CfgAtom::Name("test"),
    CfgAtom::Name("unix"),
    CfgAtom::Value("target_os", "linux"),
    CfgAtom::Value("target_family", "unix"),
    CfgAtom::Value("target_arch", "x86_64"),
    CfgAtom::Value("target_endian", "little"),
    CfgAtom::Value("target_pointer_width", "64"),
];

fn refusal(message: impl Into<String>) -> Error {
    Error::Options(format!("registry reference: {}", message.into()))
}

/// Where Cargo keeps the original `.crate` archives it downloaded, normally
/// `$CARGO_HOME/registry/cache`.
///
/// Only the bytes come from here. Each archive is authenticated against the
/// checksum in the workspace's lock file before any of it is read, so the
/// cache is a locator and never an authority. Nothing is fetched: an archive
/// that Cargo has not stored is a named refusal.
#[derive(Clone, Debug)]
pub struct ArchiveCache {
    directory: PathBuf,
}

impl ArchiveCache {
    /// The cache of the calling user's Cargo home: `CARGO_HOME` when set,
    /// otherwise `$HOME/.cargo`, then `registry/cache`.
    ///
    /// # Errors
    /// Refuses an environment that sets neither `CARGO_HOME` nor `HOME`.
    pub fn cargo_home() -> Result<Self, Error> {
        let home = std::env::var_os("CARGO_HOME").map_or_else(
            || {
                std::env::var_os("HOME")
                    .map(|home| PathBuf::from(home).join(".cargo"))
                    .ok_or_else(|| {
                        refusal(
                            "set CARGO_HOME or HOME: the registry archives are read from its registry/cache",
                        )
                    })
            },
            |home| Ok(PathBuf::from(home)),
        )?;
        Ok(Self::at(home.join("registry/cache")))
    }

    /// An explicit registry cache directory, whose children are the per-index
    /// directories that hold `NAME-VERSION.crate` files.
    #[must_use]
    pub fn at(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    /// The directory the archives are searched in.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// The bytes of the archive whose digest is `checksum`, never more than
    /// `limit` per candidate: the budget that `RegistryArchiveTree` applies to
    /// the same bytes, so an oversized entry is skipped before it is held in
    /// memory. Every index directory is searched in sorted order and the first
    /// candidate that matches the lock checksum wins; when none matches, the
    /// refusal names every candidate and why it was rejected.
    fn read(
        &self,
        name: &str,
        version: &str,
        limit: usize,
        checksum: &[u8],
    ) -> Result<Vec<u8>, Error> {
        let file = format!("{name}-{version}.crate");
        let mut indexes: Vec<_> = std::fs::read_dir(&self.directory)
            .map_err(|error| {
                refusal(format!(
                    "{name} {version}: cannot read the registry archive cache {}: {error}; run `cargo fetch --locked` for the workspace lock file",
                    self.directory.display()
                ))
            })?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<_, _>>()
            .map_err(|error| {
                refusal(format!(
                    "{name} {version}: cannot list {}: {error}",
                    self.directory.display()
                ))
            })?;
        indexes.sort();
        let mut rejected = Vec::new();
        for path in indexes.into_iter().map(|index| index.join(&file)) {
            if !path.is_file() {
                continue;
            }
            match read_candidate(&path, limit) {
                Ok(bytes) if purrdf_core::ContentDigest::of(&bytes).as_bytes() == checksum => {
                    return Ok(bytes);
                }
                Ok(_) => rejected.push(format!(
                    "{} does not match the lock checksum",
                    path.display()
                )),
                Err(reason) => rejected.push(format!("{}: {reason}", path.display())),
            }
        }
        if rejected.is_empty() {
            return Err(refusal(format!(
                "{name} {version}: {file} is not in {}; run `cargo fetch --locked` for the workspace lock file so Cargo stores the locked archives (the check fetches nothing)",
                self.directory.display()
            )));
        }
        Err(refusal(format!(
            "{name} {version}: no cached archive is the locked source ({}); remove the rejected files and run `cargo fetch --locked`",
            rejected.join("; ")
        )))
    }
}

/// One candidate archive, refused when it is larger than `limit`.
fn read_candidate(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let oversized = || format!("larger than the {limit} byte archive budget");
    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let length = file.metadata().map_err(|error| error.to_string())?.len();
    if length > u64::try_from(limit).unwrap_or(u64::MAX) {
        return Err(oversized());
    }
    // The length check is advisory: the bounded read also refuses a file that
    // grows while it is read.
    let mut bytes = Vec::new();
    file.take(length.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > limit {
        return Err(oversized());
    }
    Ok(bytes)
}

/// The registry dependencies that a locked check seals from the archive cache.
///
/// The sealed set is every registry package of the lock file that a workspace
/// package depends on directly and that the ledger's `reference` selectors or
/// the selectors added with [`Self::seal`] name. A selector is an exact package
/// name or a `prefix-*` family.
#[derive(Clone, Debug)]
pub struct LockedRegistry {
    cache: ArchiveCache,
    sealed: Vec<String>,
    features: BTreeMap<String, Vec<String>>,
    lock: Option<String>,
}

impl LockedRegistry {
    /// Seal the ledger's referenced registry packages from `cache`.
    #[must_use]
    pub const fn new(cache: ArchiveCache) -> Self {
        Self {
            cache,
            sealed: Vec::new(),
            features: BTreeMap::new(),
            lock: None,
        }
    }

    /// Also seal the registry packages that `selector` names, such as a
    /// preferred provider's family, which the ledger names by another property.
    #[must_use]
    pub fn seal(mut self, selector: impl Into<String>) -> Self {
        self.sealed.push(selector.into());
        self
    }

    /// Declare the complete, already unified feature set that the workspace
    /// enables on the sealed package `package`, including its `default`
    /// feature and every feature those imply. Cargo's resolver is not run: a
    /// set that omits a requested, default or implied feature refuses.
    #[must_use]
    pub fn features(
        mut self,
        package: impl Into<String>,
        features: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.features.insert(
            package.into(),
            features.into_iter().map(Into::into).collect(),
        );
        self
    }

    /// Use this lock text instead of the workspace root's `Cargo.lock`, for a
    /// workspace that spans several lock files.
    #[must_use]
    pub fn with_lock(mut self, lock: impl Into<String>) -> Self {
        self.lock = Some(lock.into());
        self
    }
}

/// Whether `selector` (an exact name or a `prefix-*` family) names `package`.
fn selects(selector: &str, package: &str) -> bool {
    crate::report::reference_family(selector).map_or_else(
        || selector == package,
        |prefix| {
            package
                .strip_prefix(prefix)
                .is_some_and(|rest| !rest.is_empty())
        },
    )
}

impl Gate {
    /// Check the workspace root `tree` through the locked check, with the
    /// registry dependencies of `registry` sealed from the archive cache.
    ///
    /// A copied body of a sealed registry dependency is a hard `REFERENCE-COPY`
    /// finding exactly as for a path dependency. The lock file is the only
    /// authority for the sealed sources: each archive must match its checksum.
    /// Nothing is fetched and no process is started.
    ///
    /// # Errors
    /// Refuses an archive that is absent from the cache (naming the crate,
    /// version and directory searched), larger than the archive budget or not
    /// matching its lock checksum, and every refusal of [`Self::check_locked`].
    pub fn check_locked_registry(
        &self,
        tree: &impl Tree,
        registry: &LockedRegistry,
    ) -> Result<GateReport, Error> {
        with_registry(tree, registry, |catalog, local, sources| {
            self.check_locked(tree, catalog, local, sources)
        })
    }

    /// [`Self::check_locked_registry`] over this Gate's discovered disk root.
    ///
    /// # Errors
    /// The refusals of [`Self::check_locked_registry`] and of root discovery.
    pub fn check_registry(&self, registry: &LockedRegistry) -> Result<GateReport, Error> {
        let root = self.root()?;
        let tree = raf_source::DiskTree::new(&root).map_err(|source| Error::Tree {
            file: root.display().to_string(),
            source,
        })?;
        self.check_locked_registry(&tree, registry)
    }

    /// Record the present debt through the same registry-sealed evaluation as
    /// [`Self::check_locked_registry`], with the guarantees of
    /// [`Self::write_debt_locked`]. This explicit write belongs in a binary.
    ///
    /// # Errors
    /// The refusals of [`Self::check_locked_registry`] and of
    /// [`Self::write_debt_locked`].
    pub fn write_debt_registry(&self, registry: &LockedRegistry) -> Result<DebtWrite, Error> {
        let root = self.root()?;
        let tree = raf_source::DiskTree::new(&root).map_err(|source| Error::Tree {
            file: root.display().to_string(),
            source,
        })?;
        with_registry(&tree, registry, |catalog, local, sources| {
            self.write_debt_locked(catalog, local, sources)
        })
    }
}

/// A registry declaration matched to its own outgoing lock row.
struct Matched<'a> {
    key: &'a str,
    row: LockedPackageId,
    /// The registry-name configuration and index of the row's source, or
    /// `None` when the declaration writes its own registry index.
    index: Option<(Option<&'a str>, String)>,
}

/// The declaration's effective specification: the workspace entry that an
/// inherited declaration names, otherwise its own.
fn effective(dependency: &DependencyDeclaration) -> &DependencySpec {
    match &dependency.inherited {
        Some(inherited) if dependency.specification.workspace => &inherited.specification,
        _ => &dependency.specification,
    }
}

/// Match each registry declaration of one package, whose effective package name
/// a selector names, to its own outgoing row of `owner`: the row that the
/// locked selector picks for that declaration's version requirement and
/// registry. A declaration that no outgoing row of the owner carries stays unmatched
/// and unsealed; one whose candidate rows the locked selector all refuses is a
/// disagreement between manifest and lock, which refuses.
fn matches_of<'a>(
    catalog: &LockedCatalog,
    workspace: &'a Workspace,
    package: PackageId,
    owner: LockedPackageId,
    selectors: &[&str],
    options: &LoadOptions,
) -> Result<Vec<Matched<'a>>, Error> {
    let Some(owner_row) = catalog.package(owner) else {
        return Ok(Vec::new());
    };
    let mut matched = Vec::new();
    for dependency in workspace
        .dependency_declarations(package)
        .unwrap_or_default()
    {
        let spec = effective(dependency);
        let name = spec.package.as_deref().unwrap_or(&dependency.name);
        if spec.path.is_some()
            || spec.git.is_some()
            || !selectors.iter().any(|selector| selects(selector, name))
        {
            continue;
        }
        let mut refused = None;
        let mut found = false;
        for id in owner_row.dependencies() {
            let Some(row) = catalog.package(*id) else {
                continue;
            };
            let Some(source) = row.source().filter(|source| {
                matches!(
                    source.kind(),
                    LockedSourceKind::Registry | LockedSourceKind::SparseRegistry
                )
            }) else {
                continue;
            };
            if row.name() != name {
                continue;
            }
            let named = spec.registry.is_some() || spec.registry_index.is_some();
            if !named && !is_default_registry(&index_source(source.kind(), source.repository())) {
                continue;
            }
            found = true;
            let index = if spec.registry_index.is_some() {
                None
            } else {
                Some((
                    spec.registry.as_deref(),
                    index_source(source.kind(), source.repository()),
                ))
            };
            let chosen = catalog.select_dependency(
                owner,
                dependency,
                index
                    .as_ref()
                    .map(|(name, source)| RegistryIndex::new(*name, source)),
                options,
            );
            match chosen {
                Ok(chosen) if chosen == *id => {
                    matched.push(Matched {
                        key: &dependency.key,
                        row: *id,
                        index,
                    });
                    refused = None;
                    found = false;
                    break;
                }
                Ok(_) => {}
                Err(error) => refused = Some(error),
            }
        }
        if let (true, Some(error)) = (found, refused) {
            let selector = selectors
                .iter()
                .find(|selector| selects(selector, name))
                .copied()
                .unwrap_or(name);
            return Err(refusal(format!(
                "package {name}: selector {selector:?} selects declaration {}, but the lock file and the manifest disagree: {error}; update Cargo.lock or the declaration",
                dependency.key
            )));
        }
    }
    Ok(matched)
}

/// The outgoing edges whose rows have a checked source: contained path packages
/// and the matched registry declarations, each with the registry configuration
/// of its own lock row.
fn edges_of<'a>(
    workspace: &'a Workspace,
    id: PackageId,
    matched: &'a [Matched<'a>],
) -> Vec<LockedEdge<'a>> {
    workspace
        .dependency_declarations(id)
        .unwrap_or_default()
        .iter()
        .filter_map(|dependency| {
            let spec = effective(dependency);
            if spec.path.is_some() {
                return Some(LockedEdge::new(&dependency.key, None));
            }
            let found = matched.iter().find(|entry| entry.key == dependency.key)?;
            Some(LockedEdge::new(
                &dependency.key,
                found
                    .index
                    .as_ref()
                    .map(|(name, source)| RegistryIndex::new(*name, source)),
            ))
        })
        .collect()
}

/// The declared context of every target of one package. A library is judged in
/// the ordinary mode that its other targets require of their own-library
/// companion; every other target is built by `cargo test`, and a build script
/// also names the ordinary configuration as its host.
fn targets_of<'a>(
    workspace: &'a Workspace,
    package: &str,
    ordinary: DeclaredCfg<'a>,
    testing: DeclaredCfg<'a>,
) -> Vec<DeclaredConfiguration<'a>> {
    workspace
        .targets()
        .filter(|target| target.package == package)
        .map(|target| {
            let declared = if target.kind == TargetKind::Lib {
                ordinary
            } else {
                testing
            };
            let configuration =
                DeclaredConfiguration::new(declared, &[], target.kind, &target.name);
            if target.kind == TargetKind::BuildScript {
                configuration.with_host(ordinary)
            } else {
                configuration
            }
        })
        .collect()
}

/// The lock file's identity text of a registry row's index.
fn index_source(kind: LockedSourceKind, repository: &str) -> String {
    match kind {
        LockedSourceKind::SparseRegistry => format!("sparse+{repository}"),
        _ => format!("registry+{repository}"),
    }
}

/// Hand `action` the lock catalog, the first-party manifests with their
/// declared targets, and the registry sources sealed from archives
/// authenticated by the lock checksums.
fn with_registry<T>(
    tree: &impl Tree,
    registry: &LockedRegistry,
    action: impl FnOnce(
        &LockedCatalog,
        &[LockedManifest<'_>],
        &[LockedPackageRoots<'_>],
    ) -> Result<T, Error>,
) -> Result<T, Error> {
    let ledger = read_ledger(tree)?;
    let selectors: Vec<&str> = ledger
        .reference()
        .iter()
        .chain(&registry.sealed)
        .map(String::as_str)
        .collect();
    if let Some(selector) = registry.sealed.iter().find(|selector| {
        selector.contains('*') && crate::report::reference_family(selector).is_none()
    }) {
        return Err(refusal(format!(
            "selector {selector:?}: a family selector is a package-name prefix followed by `-*`"
        )));
    }
    let options = LoadOptions::default();
    let catalog = load_catalog(tree, registry, &options)?;
    let roots: Vec<&str> = ledger.roots().iter().map(String::as_str).collect();
    let workspace = Workspace::load(tree, &roots)?;

    let owners = owner_rows(&catalog, &workspace)?;
    let matched: Vec<_> = workspace
        .package_ids()
        .zip(&owners)
        .map(|(id, owner)| matches_of(&catalog, &workspace, id, *owner, &selectors, &options))
        .collect::<Result<_, _>>()?;
    let sealed: BTreeSet<LockedPackageId> =
        matched.iter().flatten().map(|entry| entry.row).collect();
    let natives: Vec<_> = sealed
        .iter()
        .filter_map(|id| catalog.package(*id))
        .collect();

    let cfg = DeclaredCfg::complete(TARGET, ORDINARY);
    let testing = DeclaredCfg::complete(TARGET, TESTING);
    let edges: Vec<_> = workspace
        .package_ids()
        .zip(&matched)
        .map(|(id, matched)| edges_of(&workspace, id, matched))
        .collect();
    let configs: Vec<_> = workspace
        .package_ids()
        .map(|id| {
            let package = workspace
                .package(id)
                .ok_or_else(|| refusal("a workspace package vanished"))?;
            Ok::<_, Error>(targets_of(&workspace, &package.name, cfg, testing))
        })
        .collect::<Result<_, _>>()?;
    let mut local = Vec::new();
    for (index, id) in workspace.package_ids().enumerate() {
        let package = workspace
            .package(id)
            .ok_or_else(|| refusal("a workspace package vanished"))?;
        local.push(
            LockedManifest::new(owners[index], package.manifest.clone(), &edges[index])
                .with_configurations(&configs[index]),
        );
    }

    let trees = open_trees(&catalog, &natives, registry, &options)?;
    let names = trees
        .iter()
        .zip(&natives)
        .map(|(tree, row)| library_name(tree, row.name(), &options))
        .collect::<Result<Vec<_>, _>>()?;
    let enabled: Vec<Vec<&str>> = natives
        .iter()
        .map(|row| {
            registry
                .features
                .get(row.name())
                .map_or_else(Vec::new, |set| set.iter().map(String::as_str).collect())
        })
        .collect();
    let provider_configs: Vec<_> = names
        .iter()
        .zip(&enabled)
        .map(|(name, set)| [DeclaredConfiguration::new(cfg, set, TargetKind::Lib, name)])
        .collect();
    let manifests = natives
        .iter()
        .zip(&provider_configs)
        .map(|(row, config)| {
            Ok::<_, Error>([LockedManifest::new(
                row.id(),
                RelPath::new("Cargo.toml").map_err(|error| refusal(error.to_string()))?,
                &[],
            )
            .with_configurations(config)])
        })
        .collect::<Result<Vec<_>, _>>()?;
    let sources: Vec<_> = trees
        .iter()
        .zip(&manifests)
        .map(|(tree, manifest)| LockedPackageRoots::new(LockedTree::Registry(tree), manifest))
        .collect();
    action(&catalog, &local, &sources)
}

/// The library target name that the authenticated manifest of a sealed package
/// declares.
fn library_name(
    tree: &RegistryArchiveTree<'_>,
    package: &str,
    options: &LoadOptions,
) -> Result<String, Error> {
    raf_source::library_target_name(tree, options)?.ok_or_else(|| {
        refusal(format!(
            "{package}: the sealed package has no library target to compare"
        ))
    })
}

fn lock_path() -> Result<RelPath, Error> {
    RelPath::new("Cargo.lock").map_err(|error| refusal(error.to_string()))
}

/// The authenticated tree of every sealed row, read from the archive cache.
fn open_trees<'c>(
    catalog: &'c LockedCatalog,
    rows: &[&raf_source::LockedPackage],
    registry: &LockedRegistry,
    options: &LoadOptions,
) -> Result<Vec<RegistryArchiveTree<'c>>, Error> {
    let archives = rows
        .iter()
        .map(|row| {
            let checksum = row.checksum().ok_or_else(|| {
                refusal(format!(
                    "{} {}: the lock row has no archive checksum",
                    row.name(),
                    row.version()
                ))
            })?;
            registry
                .cache
                .read(row.name(), row.version(), options.max_total_bytes, checksum)
        })
        .collect::<Result<Vec<_>, _>>()?;
    rows.iter()
        .zip(&archives)
        .map(|(row, bytes)| {
            RegistryArchiveTree::from_bytes(catalog, row.id(), bytes, options).map_err(|error| {
                refusal(format!(
                    "{} {}: the cached archive is not the locked source: {error}",
                    row.name(),
                    row.version()
                ))
            })
        })
        .collect()
}

/// The lock catalog of the workspace: the explicit lock text, or the root
/// `Cargo.lock` of `tree`, read within the load's file-size bound.
fn load_catalog(
    tree: &impl Tree,
    registry: &LockedRegistry,
    options: &LoadOptions,
) -> Result<LockedCatalog, Error> {
    let lock = match &registry.lock {
        Some(lock) => lock.clone(),
        None => tree
            .read(&lock_path()?, options.max_file_bytes)
            .map_err(|source| Error::Tree {
                file: "Cargo.lock".into(),
                source,
            })?,
    };
    let lock_tree = MemoryTree::from([("Cargo.lock", lock.as_str())]);
    Ok(LockedCatalog::load(&lock_tree, &lock_path()?, options)?)
}

/// The first-party lock row of every workspace package, in package order.
fn owner_rows(
    catalog: &LockedCatalog,
    workspace: &Workspace,
) -> Result<Vec<LockedPackageId>, Error> {
    workspace
        .package_ids()
        .map(|id| {
            let package = workspace
                .package(id)
                .ok_or_else(|| refusal("a workspace package vanished"))?;
            catalog
                .packages()
                .find(|row| row.name() == package.name && row.source().is_none())
                .map(raf_source::LockedPackage::id)
                .ok_or_else(|| {
                    refusal(format!(
                        "Cargo.lock has no first-party row for {}",
                        package.name
                    ))
                })
        })
        .collect()
}

/// Whether `source` is the default registry, crates.io, which a declaration
/// naming neither a registry nor an index depends on.
fn is_default_registry(source: &str) -> bool {
    matches!(
        source,
        "registry+https://github.com/rust-lang/crates.io-index" | "sparse+https://index.crates.io/"
    )
}
