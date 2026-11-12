// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The repository's Census policy, shared by its test and registered target.

use std::error::Error;
use std::path::{Path, PathBuf};

use raf_census::{Gate, GateReport};
use raf_source::{
    CfgAtom, DeclaredCfg, DeclaredConfiguration, LoadOptions, LockedCatalog, LockedEdge,
    LockedManifest, LockedPackageRoots, LockedSourceKind, LockedTree, MemoryTree,
    RegistryArchiveTree, RegistryIndex, RelPath, TargetKind, Tree, Workspace,
};

const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";
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

pub(crate) fn gate(root: impl AsRef<Path>) -> Gate {
    Gate::workspace(root).require_rules(["serde-json", "rdf-text-writer", "rdf-term-model"])
}

/// Where Cargo keeps the original `.crate` archives it downloaded. Only the
/// bytes come from here: each archive is authenticated against the checksum in
/// the repository's lock file before any of it is read.
fn archive_cache() -> Result<PathBuf, Box<dyn Error>> {
    let home = std::env::var_os("CARGO_HOME").map_or_else(
        || {
            std::env::var_os("HOME")
                .map(|home| PathBuf::from(home).join(".cargo"))
                .ok_or("set CARGO_HOME: the registry archives of the locked PurRDF packages are read from its registry/cache")
        },
        |home| Ok(PathBuf::from(home)),
    )?;
    Ok(home.join("registry/cache"))
}

fn archive(cache: &Path, name: &str, version: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let file = format!("{name}-{version}.crate");
    for index in std::fs::read_dir(cache).map_err(|error| {
        format!(
            "read the registry archive cache {}: {error}",
            cache.display()
        )
    })? {
        let path = index?.path().join(&file);
        if path.is_file() {
            return Ok(std::fs::read(path)?);
        }
    }
    Err(format!(
        "{file} is not in {}: build the workspace once so Cargo stores the locked archive",
        cache.display()
    )
    .into())
}

/// The fuzz harness is its own Cargo workspace with its own lock file. Its
/// first-party row joins the root catalog, keeping only the dependencies the
/// root lock also holds; every shared package has the same version in both.
fn with_fuzz_row(root_lock: &str, fuzz_lock: &str) -> Result<String, Box<dyn Error>> {
    let marker = "[[package]]\nname = \"raf-source-fuzz\"\n";
    let start = fuzz_lock
        .find(marker)
        .ok_or("fuzz/Cargo.lock has no raf-source-fuzz row")?;
    let stanza = fuzz_lock[start..]
        .split("\n\n")
        .next()
        .ok_or("fuzz/Cargo.lock row is empty")?;
    let mut merged = format!("{root_lock}\n");
    for line in stanza.lines() {
        let dependency = line
            .trim()
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix("\","));
        if dependency.is_none_or(|name| root_lock.contains(&format!("name = \"{name}\"\n"))) {
            merged.push_str(line);
            merged.push('\n');
        }
    }
    Ok(merged)
}

/// Evaluate the repository through the locked check.
pub(crate) fn locked_report(gate: &Gate, tree: &impl Tree) -> Result<GateReport, Box<dyn Error>> {
    locked(tree, |catalog, local, sources| {
        gate.check_locked(tree, catalog, local, sources)
    })
}

/// Record the repository's admissible debt through the locked evaluation.
#[allow(
    dead_code,
    reason = "the registered example publishes debt; the read-only test never does"
)]
pub(crate) fn write_locked_debt(
    gate: &Gate,
    tree: &impl Tree,
) -> Result<raf_census::DebtWrite, Box<dyn Error>> {
    locked(tree, |catalog, local, sources| {
        gate.write_debt_locked(catalog, local, sources)
    })
}

/// The outgoing edges whose rows have a checked source: contained path packages
/// and the sealed native packages.
fn edges_of(workspace: &Workspace, id: raf_source::PackageId) -> Vec<LockedEdge<'_>> {
    workspace
        .dependency_declarations(id)
        .unwrap_or_default()
        .iter()
        .filter_map(|dependency| {
            let path = dependency.specification.path.is_some()
                || dependency
                    .inherited
                    .as_ref()
                    .is_some_and(|inherited| inherited.specification.path.is_some());
            let package = dependency
                .specification
                .package
                .as_deref()
                .unwrap_or(&dependency.name);
            (path || package.starts_with("purrdf")).then(|| {
                LockedEdge::new(
                    &dependency.key,
                    (!path).then(|| RegistryIndex::new(None, REGISTRY)),
                )
            })
        })
        .collect()
}

/// The declared context of every target of one package. A library is judged in
/// the ordinary mode that its other targets require of their own-library
/// companion; every other target is built by `cargo test`.
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
            DeclaredConfiguration::new(declared, &[], target.kind, &target.name)
        })
        .collect()
}

/// Hand `action` the lock catalog, the first-party manifests with their
/// declared targets, and the `purrdf-*` registry sources sealed from archives
/// authenticated by the lock checksums.
fn locked<T>(
    tree: &impl Tree,
    action: impl FnOnce(
        &LockedCatalog,
        &[LockedManifest<'_>],
        &[LockedPackageRoots<'_>],
    ) -> Result<T, raf_census::Error>,
) -> Result<T, Box<dyn Error>> {
    let options = LoadOptions::default();
    let lock = with_fuzz_row(
        &tree.read(&RelPath::new("Cargo.lock")?, u64::MAX)?,
        &tree.read(&RelPath::new("fuzz/Cargo.lock")?, u64::MAX)?,
    )?;
    let lock_tree = MemoryTree::from([("Cargo.lock", lock.as_str())]);
    let catalog = LockedCatalog::load(&lock_tree, &RelPath::new("Cargo.lock")?, &options)?;
    let workspace = Workspace::load(tree, &[".", "fuzz"])?;
    let row = |name: &str| {
        catalog
            .packages()
            .find(|row| row.name() == name && row.source().is_none())
            .map(raf_source::LockedPackage::id)
            .ok_or_else(|| format!("Cargo.lock has no first-party row for {name}"))
    };
    let cfg = DeclaredCfg::complete("x86_64-unknown-linux-gnu", ORDINARY);
    let testing = DeclaredCfg::complete("x86_64-unknown-linux-gnu", TESTING);
    let edges: Vec<_> = workspace
        .package_ids()
        .map(|id| edges_of(&workspace, id))
        .collect();
    let configs: Vec<_> = workspace
        .package_ids()
        .map(|id| {
            let package = workspace.package(id).ok_or("package vanished")?;
            Ok::<_, Box<dyn Error>>(targets_of(&workspace, &package.name, cfg, testing))
        })
        .collect::<Result<_, _>>()?;
    let mut local = Vec::new();
    for (index, id) in workspace.package_ids().enumerate() {
        let package = workspace.package(id).ok_or("package vanished")?;
        local.push(
            LockedManifest::new(row(&package.name)?, package.manifest.clone(), &edges[index])
                .with_configurations(&configs[index]),
        );
    }

    let cache = archive_cache()?;
    let direct: std::collections::BTreeSet<&str> = workspace
        .package_ids()
        .filter_map(|id| workspace.dependency_declarations(id))
        .flatten()
        .map(|dependency| {
            dependency
                .specification
                .package
                .as_deref()
                .unwrap_or(&dependency.name)
        })
        .collect();
    let natives: Vec<_> = catalog
        .packages()
        .filter(|row| {
            direct.contains(row.name())
                && row.name().starts_with("purrdf")
                && row
                    .source()
                    .is_some_and(|source| source.kind() == LockedSourceKind::Registry)
        })
        .collect();
    let archives = natives
        .iter()
        .map(|row| archive(&cache, row.name(), row.version()))
        .collect::<Result<Vec<_>, _>>()?;
    let trees = natives
        .iter()
        .zip(&archives)
        .map(|(row, bytes)| RegistryArchiveTree::from_bytes(&catalog, row.id(), bytes, &options))
        .collect::<Result<Vec<_>, _>>()?;
    let names: Vec<String> = natives
        .iter()
        .map(|row| row.name().replace('-', "_"))
        .collect();
    let provider_configs: Vec<_> = names
        .iter()
        .map(|name| [DeclaredConfiguration::new(cfg, &[], TargetKind::Lib, name)])
        .collect();
    let manifests: Vec<_> = natives
        .iter()
        .zip(&provider_configs)
        .map(|(row, config)| {
            Ok::<_, Box<dyn Error>>([LockedManifest::new(
                row.id(),
                RelPath::new("Cargo.toml")?,
                &[],
            )
            .with_configurations(config)])
        })
        .collect::<Result<_, _>>()?;
    let sources: Vec<_> = trees
        .iter()
        .zip(&manifests)
        .map(|(tree, manifest)| LockedPackageRoots::new(LockedTree::Registry(tree), manifest))
        .collect();
    Ok(action(&catalog, &local, &sources)?)
}
