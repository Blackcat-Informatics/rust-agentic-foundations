// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual registry provider identities and a separately frozen nonempty digest.

use raf_source::{
    CfgAtom, DeclaredCfg, DeclaredConfiguration, LoadOptions, LockedCatalog, LockedEdge,
    LockedManifest, LockedPackageRoots, LockedSourceKind, LockedTree, MemoryTree,
    RegistryArchiveTree, RegistryIndex, RelPath, TargetKind,
};

use super::{Format, Gate, Term, Triple, provider_policy, put, vocab};

#[path = "../support/native_hash.rs"]
mod native_hash;

const INDEX: &str = "registry+https://github.com/rust-lang/crates.io-index";
const CONSUMER: &str = include_str!("../support/native_hash.rs");

fn context(name: &str) -> DeclaredConfiguration<'_> {
    const ATOMS: &[CfgAtom<'_>] = &[
        CfgAtom::Name("unix"),
        CfgAtom::Value("target_os", "linux"),
        CfgAtom::Value("target_family", "unix"),
        CfgAtom::Value("target_arch", std::env::consts::ARCH),
        CfgAtom::Value("target_endian", "little"),
        CfgAtom::Value("target_pointer_width", "64"),
    ];
    DeclaredConfiguration::new(
        DeclaredCfg::complete("native-linux-provider", ATOMS),
        &[],
        TargetKind::Lib,
        name,
    )
}

fn ledger() -> String {
    let mut graph = provider_policy("purrdf-hash", "purrdf_hash::blake3::Hasher::new");
    for (subject, predicate, old) in [
        ("#provider", "consumer", "demo::run"),
        ("#job", "homeSymbol", "demo::alias"),
    ] {
        assert!(
            graph
                .remove(Triple {
                    subject: Term::iri(subject),
                    predicate: Term::iri(format!("{}{predicate}", vocab::REPORT)),
                    object: Term::simple_literal(old),
                })
                .unwrap()
        );
        put(
            &mut graph,
            subject,
            predicate,
            Term::simple_literal("demo::digest"),
        );
    }
    for api in [
        "purrdf_hash::blake3::Hasher::update",
        "purrdf_hash::blake3::Hasher::finalize",
        "purrdf_hash::blake3::Hash::as_bytes",
    ] {
        put(
            &mut graph,
            "#provider",
            "entryPoint",
            Term::simple_literal(api),
        );
    }
    graph.finish().unwrap().to_text(Format::Turtle).unwrap()
}

pub(super) fn actual_registry_hasher() {
    // BLAKE3's official unkeyed vector for input bytes 0, 1, 2. The expected
    // bytes are fixed independently of every implementation exercised here.
    let expected = [
        0xe1, 0xbe, 0x4d, 0x7a, 0x8a, 0xb5, 0x56, 0x0a, 0xa4, 0x19, 0x9e, 0xea, 0x33, 0x98, 0x49,
        0xba, 0x8e, 0x29, 0x3d, 0x55, 0xca, 0x0a, 0x81, 0x00, 0x67, 0x26, 0xd1, 0x84, 0x51, 0x9e,
        0x64, 0x7f,
    ];
    assert_eq!(native_hash::digest(&[0, 1, 2]).as_bytes(), &expected);

    let lock = format!(
        "{}\n[[package]]\nname='demo'\nversion='1.0.0'\ndependencies=['purrdf-hash']\n",
        include_str!("../../../../Cargo.lock")
    );
    let lock_tree = MemoryTree::from([("Cargo.lock", lock.as_str())]);
    let options = LoadOptions::default();
    let catalog =
        LockedCatalog::load(&lock_tree, &RelPath::new("Cargo.lock").unwrap(), &options).unwrap();
    let native = catalog
        .packages()
        .find(|row| row.name() == "purrdf-hash")
        .unwrap();
    assert_eq!(native.source().unwrap().kind(), LockedSourceKind::Registry);
    assert_eq!(
        native.source().unwrap().repository(),
        INDEX.strip_prefix("registry+").unwrap()
    );
    assert!(native.version().starts_with("4."), "{}", native.version());
    let demo = catalog
        .packages()
        .find(|row| row.name() == "demo")
        .unwrap()
        .id();
    let archive = registry_archive(native.name(), native.version());
    let registry = RegistryArchiveTree::from_bytes(&catalog, native.id(), &archive, &options)
        .expect("compiled registry dependency must expose its checksum-authenticated archive");
    let mut altered = archive.clone();
    altered[0] ^= 1;
    let invalid = RegistryArchiveTree::from_bytes(&catalog, native.id(), &altered, &options)
        .err()
        .expect("changed archive bytes must fail authentication");
    assert!(invalid.to_string().contains("checksum"), "{invalid}");
    let manifest = "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n[dependencies]\npurrdf-hash={version='=4'}\n";
    let ledger = ledger();
    let tree = MemoryTree::from([
        ("Cargo.toml", manifest),
        ("src/lib.rs", CONSUMER),
        ("Cargo.lock", lock.as_str()),
        ("helpers-ledger.ttl", ledger.as_str()),
    ]);
    let edges = [LockedEdge::new(
        "dependencies.purrdf-hash",
        Some(RegistryIndex::new(None, INDEX)),
    )];
    let local = [
        LockedManifest::new(demo, RelPath::new("Cargo.toml").unwrap(), &edges)
            .with_configuration(context("demo")),
    ];
    let roots = [
        LockedManifest::new(native.id(), RelPath::new("Cargo.toml").unwrap(), &[])
            .with_configuration(context("purrdf_hash")),
    ];
    let sources = [LockedPackageRoots::new(
        LockedTree::Registry(&registry),
        &roots,
    )];
    let gate = Gate::workspace(".");
    let report = gate
        .check_locked(&tree, &catalog, &local, &sources)
        .unwrap();
    assert!(report.is_clean(), "{:?}", report.findings());
    // A source already authenticated under one catalog cannot be relabeled
    // by another catalog carrying a different archive checksum.
    let checksum = purrdf_hash::hex::Lower(native.checksum().unwrap()).to_string();
    let altered_lock = lock.replacen(&checksum, &"0".repeat(64), 1);
    let altered_tree = MemoryTree::from([("Cargo.lock", altered_lock.as_str())]);
    let altered_catalog = LockedCatalog::load(
        &altered_tree,
        &RelPath::new("Cargo.lock").unwrap(),
        &options,
    )
    .unwrap();
    let relabeled = gate
        .check_locked(&tree, &altered_catalog, &local, &sources)
        .unwrap_err();
    assert!(relabeled.to_string().contains("checksum"), "{relabeled}");
    let missing = gate.check_locked(&tree, &catalog, &local, &[]).unwrap_err();
    assert!(missing.to_string().contains("source"), "{missing}");
}

fn registry_archive(name: &str, version: &str) -> Vec<u8> {
    // Cache names only locate candidate bytes. The production constructor
    // authenticates their identity against the actual selected lock checksum.
    let home = std::env::var_os("CARGO_HOME").map_or_else(
        || {
            std::path::PathBuf::from(
                std::env::var_os("HOME").expect("native fixture needs a Cargo cache locator"),
            )
            .join(".cargo")
        },
        std::path::PathBuf::from,
    );
    let cache = home.join("registry/cache");
    let mut indexes: Vec<_> = std::fs::read_dir(&cache)
        .expect("compiled registry packages require their Cargo archive cache")
        .map(|entry| entry.expect("read Cargo cache index").path())
        .collect();
    indexes.sort();
    let archive = format!("{name}-{version}.crate");
    let path = indexes
        .into_iter()
        .map(|index| index.join(&archive))
        .find(|path| path.is_file())
        .expect("compiled registry package needs its original archive; restore the Cargo cache");
    std::fs::read(path).expect("read the actual registry package archive")
}
