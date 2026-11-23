// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Committed persistence policies select and protect exact Rust declarations.

use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use purrdf_testkit::TempDir;
use raf_fixtures::git_repo;
use raf_rdf::{Builder, Format, TermValue, vocab};

use super::{assert_refused, put, row, text};

pub(super) const BASE: &[u8] = include_bytes!("../fixtures/capnp/base.bin");
const COMPATIBLE: &[u8] = include_bytes!("../fixtures/capnp/compatible.bin");
pub(super) const CHANGED: &[u8] = include_bytes!("../fixtures/capnp/changed.bin");
pub(super) const PACKET: &str = "0xf1804b9b18ae6505";
const STATE: &str = "0xe70e425d8d63c329";
const PAYLOAD: &str = "0xc5fa75a0c95aa56a";
const TYPES: &str = r"
#[cfg(not(test))]
pub struct Packet { pub value: u64 }
#[cfg(test)]
pub struct Packet { pub test_only: bool }
pub fn Packet() {}
pub use self::Packet as Alias;
pub struct Mirror;
pub enum State { Ready, Active, Retired }
pub enum Backup { Ready }
pub fn run() {}
#[cfg(unix)]
pub struct Ambiguous;
#[cfg(windows)]
pub struct Ambiguous;
";

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
pub(super) fn repository() -> TempDir {
    let scratch = TempDir::new_in(Path::new(env!("CARGO_TARGET_TMPDIR"))).unwrap();
    let root = scratch.path();
    git_repo::init(root);
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"persist-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n",
    )
    .unwrap();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), TYPES).unwrap();
    scratch
}

pub(super) fn policy(rows: &[(&str, &str, &str, &str)]) -> Builder {
    let mut graph = Builder::new();
    put(
        &mut graph,
        "",
        vocab::TYPE,
        TermValue::iri(format!("{}CapnpPersistencePolicyV1", vocab::REPORT)),
    );
    for &(id, package, symbol, root_type) in rows {
        let id = format!("#{id}");
        row(&mut graph, &id, "persistedType", "CapnpPersistedType");
        text(&mut graph, &id, "package", package);
        text(&mut graph, &id, "symbol", symbol);
        text(&mut graph, &id, "rootType", root_type);
    }
    graph
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
pub(super) fn commit(root: &Path, descriptor: &[u8], policy: Builder) -> String {
    fs::write(root.join("schema.bin"), descriptor).unwrap();
    fs::write(
        root.join("persisted.nt"),
        policy.finish().unwrap().to_text(Format::NTriples).unwrap(),
    )
    .unwrap();
    git_repo::commit_all(root)
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
pub(super) fn check(root: &Path, base: &str, current: &str) -> Output {
    command(root, base, current).output().unwrap()
}

fn command(root: &Path, base: &str, current: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-raf-census"));
    command.current_dir(root).args([
        "capnp-persistence",
        "--base",
        base,
        "--current",
        current,
        "--base-descriptor",
        "schema.bin",
        "--current-descriptor",
        "schema.bin",
        "--policy",
        "persisted.nt",
    ]);
    command
}

fn assert_clean(output: &Output) {
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
}

/// Only retained schema closures count. The committed Rust symbol can move.
#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn capnp_persistence_compares_retained_committed_layouts() {
    let scratch = repository();
    let root = scratch.path();
    let base = commit(
        root,
        BASE,
        policy(&[
            (
                "packet",
                "persist-fixture",
                "persist_fixture::Packet",
                PACKET,
            ),
            ("state", "persist-fixture", "persist_fixture::State", STATE),
        ]),
    );
    let moved = [
        (
            "packet",
            "persist-fixture",
            "persist_fixture::relocated::Packet",
            PACKET,
        ),
        ("state", "persist-fixture", "persist_fixture::State", STATE),
    ];
    fs::write(
        root.join("src/lib.rs"),
        "pub mod relocated { pub struct Packet; }\npub enum State { Ready, Active, Retired }",
    )
    .unwrap();
    let compatible = commit(root, COMPATIBLE, policy(&moved));
    let changed = commit(root, CHANGED, policy(&moved));
    fs::write(root.join("persisted.nt"), b"dirty policy").unwrap();
    fs::write(root.join("src/lib.rs"), b"dirty source").unwrap();
    fs::write(root.join("schema.bin"), b"dirty descriptor").unwrap();

    assert_clean(&check(root, &base, &compatible));
    let findings = check(root, &base, &changed);
    assert_eq!(findings.status.code(), Some(1), "{findings:?}");
    assert_eq!(findings.stderr, b"");
    let text = String::from_utf8(findings.stdout).unwrap();
    assert!(
        text.contains(&format!("{PACKET}.title: member renamed to heading")),
        "{text}"
    );
    assert!(
        text.contains(&format!("{STATE}.retired: member removed")),
        "{text}"
    );
    assert!(
        text.contains(&format!("{PAYLOAD}.obsolete: member removed")),
        "{text}"
    );
    assert!(!text.contains("ignored"), "{text}");
}

/// An interface pointer preserves its nominal type without selecting RPC layouts
/// for a retained persisted declaration.
#[test]
fn capnp_persistence_ignores_rpc_endpoint_layouts() {
    let scratch = repository();
    let root = scratch.path();
    for (id, before, after) in [
        (
            "0xeec2000000000006",
            include_bytes!("../fixtures/capnp/rpc-named-base.bin").as_slice(),
            include_bytes!("../fixtures/capnp/rpc-named-changed.bin").as_slice(),
        ),
        (
            "0xeec4000000000006",
            include_bytes!("../fixtures/capnp/rpc-inheritance-base.bin").as_slice(),
            include_bytes!("../fixtures/capnp/rpc-inheritance-changed.bin").as_slice(),
        ),
        (
            "0xeec4400000000003",
            include_bytes!("../fixtures/capnp/rpc-implicit-base.bin").as_slice(),
            include_bytes!("../fixtures/capnp/rpc-implicit-changed.bin").as_slice(),
        ),
    ] {
        let row = || policy(&[("packet", "persist-fixture", "persist_fixture::Packet", id)]);
        let base = commit(root, before, row());
        let current = commit(root, after, row());
        assert_clean(&check(root, &base, &current));
    }
}

/// Stable rows protect prior roots even when a second row selects them.
#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn capnp_persistence_protects_declaration_continuity_after_first_adoption() {
    let scratch = repository();
    let root = scratch.path();
    fs::write(root.join("schema.bin"), BASE).unwrap();
    let missing = git_repo::commit_all(root);
    let empty = commit(root, super::DESCRIPTOR, policy(&[]));
    let adopted = [
        (
            "packet",
            "persist-fixture",
            "persist_fixture::Packet",
            PACKET,
        ),
        (
            "mirror",
            "persist-fixture",
            "persist_fixture::Mirror",
            PACKET,
        ),
        ("state", "persist-fixture", "persist_fixture::State", STATE),
        (
            "backup",
            "persist-fixture",
            "persist_fixture::Backup",
            STATE,
        ),
    ];
    let base = commit(root, BASE, policy(&adopted));
    assert_refused(check(root, &missing, &base), "persisted.nt");
    assert_clean(&check(root, &empty, &base));

    let current = commit(
        root,
        BASE,
        policy(&[
            adopted[1],
            (
                "state",
                "persist-fixture",
                "persist_fixture::State",
                PAYLOAD,
            ),
            adopted[3],
        ]),
    );
    let findings = check(root, &base, &current);
    assert_eq!(findings.status.code(), Some(1), "{findings:?}");
    assert_eq!(findings.stderr, b"");
    assert_eq!(
        String::from_utf8(findings.stdout).unwrap(),
        format!(
            "Cap'n Proto persistence declaration packet: removed protection of {PACKET}\nCap'n Proto persistence declaration state: root changed from {STATE} to {PAYLOAD}\n"
        ),
    );
}

/// Invalid new bindings and roots refuse before providing a partial verdict.
#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn capnp_persistence_refuses_invalid_bindings_and_policy_rows() {
    let scratch = repository();
    let root = scratch.path();
    let base = commit(root, super::DESCRIPTOR, policy(&[]));
    for (package, symbol) in [
        ("persist-fixture", "persist_fixture::Alias"),
        ("persist-fixture", "persist_fixture::Ambiguous"),
        ("persist-fixture", "persist_fixture::Missing"),
        ("persist-fixture", "persist_fixture::run"),
        ("other-package", "persist_fixture::Packet"),
    ] {
        let current = commit(root, BASE, policy(&[("bad", package, symbol, PACKET)]));
        assert_refused(
            check(root, &base, &current),
            "must name one direct first-party",
        );
    }
    let row = (
        "packet",
        "persist-fixture",
        "persist_fixture::Packet",
        PACKET,
    );
    let rpc = include_bytes!("../fixtures/capnp/rpc-base.bin");
    for (descriptor, node) in [
        (super::DESCRIPTOR, row.3),
        (rpc, "0xcad554910d7a67ab"),
        (rpc, "0xd766b858756a10ab"),
    ] {
        let current = commit(root, descriptor, policy(&[(row.0, row.1, row.2, node)]));
        assert_refused(
            check(root, &base, &current),
            "must be a declared schema struct or enum",
        );
    }

    let current = commit(
        root,
        BASE,
        policy(&[(row.0, row.1, row.2, "0x8000000000000001")]),
    );
    assert_refused(
        check(root, &base, &current),
        "missing schema node 0x8000000000000001",
    );

    let mut invalid = policy(&[row]);
    text(&mut invalid, "#packet", "reader", "unrecognized waiver");
    let current = commit(root, BASE, invalid);
    assert_refused(check(root, &base, &current), "reader");

    let current = commit(root, BASE, policy(&[("bad_slug", row.1, row.2, row.3)]));
    assert_refused(check(root, &base, &current), "must have a lowercase slug");

    let ids: Vec<_> = (0..97).map(|index| format!("t{index}")).collect();
    let rows: Vec<_> = ids
        .iter()
        .map(|id| (id.as_str(), "p", "p::T", PACKET))
        .collect();
    let excess = commit(root, BASE, policy(&rows));
    assert_refused(
        check(root, &base, &excess),
        "admits at most 96 declarations",
    );

    fs::write(root.join("persisted.nt"), vec![b' '; 128 * 1024 + 1]).unwrap();
    let oversized = git_repo::commit_all(root);
    assert_refused(
        check(root, &base, &oversized),
        "persisted.nt is larger than the limit of 131072 bytes",
    );
}

/// First adoption validates the new root's layout despite having no retained roots.
#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn capnp_persistence_validates_current_only_layouts() {
    let scratch = repository();
    let root = scratch.path();
    let descriptor = include_bytes!("../fixtures/capnp/required-base.bin");
    let malformed = include_bytes!("../fixtures/capnp/required-invalid-slots.bin");
    let row = (
        "packet",
        "persist-fixture",
        "persist_fixture::Packet",
        PACKET,
    );
    let base = commit(root, descriptor, policy(&[]));
    let valid = commit(root, descriptor, policy(&[row]));
    let invalid = commit(root, malformed, policy(&[row]));
    let check = |current: &str| {
        command(root, &base, current)
            .args(["--required-annotation", "0xe91447cf636b013a"])
            .output()
            .unwrap()
    };
    assert_clean(&check(&valid));
    assert_refused(check(&invalid), "expected one Void application");
}
