// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Complete locked identities and unique outgoing references through the public API.

use raf_source::{
    Budget, DiskTree, LoadError, LoadErrorKind, LoadOptions, LockedCatalog, LockedGitReference,
    LockedSourceKind, MemoryTree, RelPath,
};

const REV_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const REV_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn load(text: &str, options: &LoadOptions) -> Result<LockedCatalog, LoadError> {
    let tree = MemoryTree::from([("Cargo.lock", text)]);
    LockedCatalog::load(
        &tree,
        &RelPath::new("Cargo.lock").expect("lock path"),
        options,
    )
}

fn row(name: &str, version: &str, source: Option<&str>, dependencies: &[&str]) -> String {
    let source = source.map_or_else(String::new, |text| format!("source = {text:?}\n"));
    format!(
        "[[package]]\nname = {name:?}\nversion = {version:?}\n{source}dependencies = {dependencies:?}\n"
    )
}

fn graph(reference: &str, reverse: bool) -> String {
    let first = format!("git+https://example.test/first?rev={REV_A}#{REV_A}");
    let second = format!("git+https://example.test/second?rev={REV_B}#{REV_B}");
    let a = row("helper", "1.0.0", Some(&first), &[]);
    let b = row("helper", "1.0.0", Some(&second), &[]);
    let root = row("app", "0.1.0", None, &[reference]);
    if reverse {
        format!("version = 4\n{b}{root}{a}")
    } else {
        format!("version = 4\n{a}{root}{b}")
    }
}

#[test]
fn same_name_version_sources_require_unique_complete_selection_in_either_row_order() {
    let reference = format!("helper 1.0.0 (git+https://example.test/second?rev={REV_B}#{REV_B})");
    for reverse in [false, true] {
        for abbreviated in ["helper", "helper 1.0.0"] {
            let error = load(&graph(abbreviated, reverse), &LoadOptions::default())
                .expect_err("neither first row nor version alone proves a source");
            assert!(matches!(error.kind, LoadErrorKind::Locked { .. }));
            assert!(error.to_string().contains("ambiguous"), "{error}");
        }
        let catalog =
            load(&graph(&reference, reverse), &LoadOptions::default()).expect("exact source");
        let app = catalog
            .packages()
            .find(|package| package.name() == "app")
            .expect("app");
        assert_eq!(app.dependencies().len(), 1);
        let helper = catalog
            .package(app.dependencies()[0])
            .expect("selected helper");
        assert_eq!(helper.name(), "helper");
        assert_eq!(helper.version(), "1.0.0");
        let source = helper.source().expect("external source");
        assert_eq!(source.repository(), "https://example.test/second");
        assert_eq!(source.revision(), Some(REV_B));
    }
}

#[test]
fn omitted_git_precision_is_not_a_wildcard_beside_the_full_revision() {
    let partial = format!("helper 1.0.0 (git+https://example.test/second?rev={REV_B})");
    let error =
        load(&graph(&partial, false), &LoadOptions::default()).expect_err("precision required");
    assert!(
        error
            .to_string()
            .contains("selects no complete locked package"),
        "{error}"
    );
    let complete = format!("helper 1.0.0 (git+https://example.test/second?rev={REV_B}#{REV_B})");
    load(&graph(&complete, false), &LoadOptions::default()).expect("full neighboring identity");
}

#[test]
fn encoded_git_references_keep_native_decoded_identity_and_reference_kind() {
    let source = format!("git+https://example.test/helpers?branch=feature%2Ftrees#{REV_A}");
    let catalog = load(
        &format!(
            "version = 4\n{}",
            row("helper", "1.0.0", Some(&source), &[])
        ),
        &LoadOptions::default(),
    )
    .expect("encoded branch");
    let package = catalog.packages().next().expect("helper");
    let source = package.source().expect("source");
    assert_eq!(source.kind(), LockedSourceKind::Git);
    assert!(
        matches!(source.reference(), Some(LockedGitReference::Branch(value)) if value == "feature/trees")
    );
    assert_eq!(source.revision(), Some(REV_A));
    let duplicate = format!("git+https://example.test/helpers?branch=feature/trees#{REV_A}");
    let error = load(&format!("version = 4\n{}{}", row("helper", "1.0.0", Some(&duplicate), &[]), row("helper", "1.0.0", Some("git+https://example.test/helpers?branch=feature%2Ftrees#aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), &[])), &LoadOptions::default()).expect_err("same decoded source is a duplicate");
    assert!(
        error
            .to_string()
            .contains("duplicate complete package identity")
    );
}

#[test]
fn unicode_checksum_refuses_without_panicking_beside_native_ascii_hex() {
    let package = row(
        "helper",
        "1.0.0",
        Some("registry+https://example.test/index"),
        &[],
    );
    let invalid = format!("version = 4\n{package}checksum = {:?}\n", "é".repeat(32));
    let error = load(&invalid, &LoadOptions::default()).expect_err("64-byte non-ASCII is not hex");
    assert!(matches!(
        error.kind,
        LoadErrorKind::Locked {
            field: "checksum",
            ..
        }
    ));
    let valid = format!("version = 4\n{package}checksum = {:?}\n", "a".repeat(64));
    let catalog = load(&valid, &LoadOptions::default()).expect("native checksum decoder");
    assert_eq!(
        catalog.packages().next().expect("helper").checksum(),
        Some(&[0xaa; 32])
    );
}

#[test]
fn native_catalog_storage_and_comparison_work_obey_exact_original_grants() {
    let reference = format!("helper 1.0.0 (git+https://example.test/second?rev={REV_B}#{REV_B})");
    let text = graph(&reference, true);
    let catalog = load(&text, &LoadOptions::default()).expect("complete catalog");
    let mut exact = LoadOptions::default();
    exact.max_item_bytes = catalog.usage().item_bytes;
    exact.max_work = catalog.usage().work;
    assert_eq!(load(&text, &exact).expect("both exact allowances"), catalog);
    for budget in [Budget::ItemBytes, Budget::Work] {
        let mut short = exact.clone();
        match budget {
            Budget::ItemBytes => short.max_item_bytes -= 1,
            Budget::Work => short.max_work -= 1,
            _ => unreachable!("only the two exact grants are varied"),
        }
        let error = load(&text, &short).expect_err("one unit too little");
        assert!(
            matches!(error.kind, LoadErrorKind::Budget { budget: actual, .. } if actual == budget),
            "{error}"
        );
        assert_eq!(error.file.as_str(), "Cargo.lock");
    }
}

#[test]
fn malformed_unsupported_and_stale_evidence_has_a_valid_profile_neighbor() {
    let grammar = load("version = ", &LoadOptions::default()).unwrap_err();
    assert!(
        std::error::Error::source(&grammar)
            .is_some_and(<dyn std::error::Error>::is::<purrdf_toml::TomlError>)
    );
    let valid = format!(
        "version = 4\n{}",
        row("app", "1.0.0-alpha.1+build.7", None, &[])
    );
    load(&valid, &LoadOptions::default()).expect("maintained SemVer prerelease/build parsing");
    assert_cargo_version_identities();
    for text in [
        valid.replace("version = 4", "version = 2"),
        valid.replace("version = 4", "version = 9223372036854775808"),
        valid.replace("version = 4", "version = -9223372036854775809"),
        valid.replace("version = 4", "version = 0x8000_0000_0000_0000"),
        valid.replace("version = 4", "version = 1e999"),
        valid.replace("version = 4", "version = 4.0"),
        valid.replace("1.0.0-alpha.1+build.7", "01.0.0"),
        format!("{valid}unknown = true\n"),
        format!("{valid}replace = 'other 1.0.0'\n"),
        format!("version = 4\n{}", row("app", "1.0.0", None, &["missing"])),
        format!(
            "version = 4\n{}{}",
            row("app", "1.0.0", None, &["helper", "helper"]),
            row("helper", "1.0.0", None, &[])
        ),
        format!(
            "version = 4\n{}",
            row(
                "helper",
                "1.0.0",
                Some("git+https://example.test/helpers#abc"),
                &[]
            )
        ),
        format!(
            "version = 4\n{}",
            row(
                "helper",
                "1.0.0",
                Some("git+https://example.test/helpers"),
                &[]
            )
        ),
        format!(
            "version = 4\n{}",
            row(
                "helper",
                "1.0.0",
                Some(
                    "git+https://example.test/helpers?branch=a&tag=b#aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                ),
                &[]
            )
        ),
    ] {
        let error = load(&text, &LoadOptions::default()).expect_err(&text);
        assert!(
            matches!(error.kind, LoadErrorKind::Locked { .. }),
            "{error}"
        );
    }
    load(
        &valid.replace("version = 4", "version = 3"),
        &LoadOptions::default(),
    )
    .expect("supported version three neighbor");
    for version in ["+4", "0x4", "0o4", "0b100"] {
        load(
            &valid.replace("version = 4", &format!("version = {version}")),
            &LoadOptions::default(),
        )
        .expect("supported integer spelling retains version identity");
    }
}

#[test]
fn the_actual_repository_lock_resolves_all_outgoing_edges_and_native_source_precision() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tree = DiskTree::new(&root).expect("actual repository tree");
    let catalog = LockedCatalog::load(
        &tree,
        &RelPath::new("Cargo.lock").expect("lock"),
        &LoadOptions::default(),
    )
    .expect("actual complete package graph");
    let native = catalog
        .packages()
        .find(|package| package.name() == "purrdf-hash")
        .expect("actual native dependency");
    let source = native.source().expect("external native source");
    assert_eq!(source.kind(), LockedSourceKind::Git);
    assert_eq!(
        source.repository(),
        "https://github.com/Blackcat-Informatics/purrdf.git"
    );
    assert_eq!(source.revision().expect("full precision").len(), 40);
    for package in catalog.packages() {
        for id in package.dependencies() {
            assert!(
                catalog.package(*id).is_some(),
                "every published edge names an actual row"
            );
        }
    }
}

fn assert_cargo_version_identities() {
    for version in [
        "0.0.0+00.001",
        "18446744073709551615.18446744073709551615.18446744073709551615",
        "1.2.3-999999999999999999999999999999999999999+0001",
    ] {
        let text = format!(
            "version=4\n{}{}",
            row("app", "1.0.0", None, &[&format!("helper {version}")]),
            row("helper", version, None, &[])
        );
        let catalog = load(&text, &LoadOptions::default()).expect("full version neighbor");
        let helper = catalog
            .packages()
            .find(|row| row.name() == "helper")
            .expect("helper row");
        assert_eq!(
            helper.version(),
            version,
            "canonical build and prerelease identity"
        );
    }
    for version in [
        "1.2",
        "1.2.3-01",
        "1.2.3+",
        "1.2.3-a..b",
        "18446744073709551616.0.0",
        "1.2.3-α",
    ] {
        let text = format!("version=4\n{}", row("helper", version, None, &[]));
        let error = load(&text, &LoadOptions::default()).expect_err("invalid full package version");
        assert!(
            error.to_string().contains("invalid package version"),
            "{error}"
        );
        let text = format!(
            "version=4\n{}{}",
            row("app", "1.0.0", None, &[&format!("helper {version}")]),
            row("helper", "1.2.3", None, &[])
        );
        let error = load(&text, &LoadOptions::default()).expect_err("invalid outgoing version");
        assert!(
            error.to_string().contains("invalid dependency version"),
            "{error}"
        );
    }
}
