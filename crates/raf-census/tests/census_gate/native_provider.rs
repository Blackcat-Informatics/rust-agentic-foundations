// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual pinned provider identities and a separately frozen nonempty digest.

use raf_source::{
    CfgAtom, CommittedGitTree, DeclaredCfg, DeclaredConfiguration, GitPackageRoots, LoadOptions,
    LockedCatalog, LockedEdge, LockedManifest, MemoryTree, RelPath, TargetKind,
};

use super::{Format, Gate, Term, Triple, provider_policy, put, vocab};

#[path = "../support/native_hash.rs"]
mod native_hash;

const PIN: &str = "f6220a74255f93b59f4250df2790ee0e66ae51a2";
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

pub(super) fn actual_pinned_hasher() {
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
    assert_eq!(native.source().unwrap().revision(), Some(PIN));
    let demo = catalog
        .packages()
        .find(|row| row.name() == "demo")
        .unwrap()
        .id();
    let repository = purrdf_testkit::paths::workspace_root();
    let committed = CommittedGitTree::open(&catalog, native.id(), &repository, &options)
        .expect("compiled PurRDF dependency must expose its exact pinned Git objects");
    let manifest = format!(
        "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n[dependencies]\npurrdf-hash={{version='3.0.1',git='{}',rev='{PIN}'}}\n",
        native.source().unwrap().repository()
    );
    let ledger = ledger();
    let tree = MemoryTree::from([
        ("Cargo.toml", manifest.as_str()),
        ("src/lib.rs", CONSUMER),
        ("Cargo.lock", lock.as_str()),
        ("helpers-ledger.ttl", ledger.as_str()),
    ]);
    let edges = [LockedEdge::new("dependencies.purrdf-hash", None)];
    let local = [
        LockedManifest::new(demo, RelPath::new("Cargo.toml").unwrap(), &edges)
            .with_configuration(context("demo")),
    ];
    let roots = [LockedManifest::new(
        native.id(),
        RelPath::new("crates/hash/Cargo.toml").unwrap(),
        &[],
    )
    .with_configuration(context("purrdf_hash"))];
    let sources = [GitPackageRoots::new(&committed, &roots)];
    let gate = Gate::workspace(".");
    let report = gate
        .check_locked(&tree, &catalog, &local, &sources)
        .unwrap();
    assert!(report.is_clean(), "{:?}", report.findings());
    let missing = gate.check_locked(&tree, &catalog, &local, &[]).unwrap_err();
    assert!(missing.to_string().contains("source"), "{missing}");
}
