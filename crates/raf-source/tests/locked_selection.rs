// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Declaration selection never substitutes an unrelated or incompletely specified edge.

use raf_source::{
    Budget, DependencyDeclaration, LoadErrorKind, LoadOptions, LockedCatalog, LockedPackageId,
    MemoryTree, RegistryIndex, RelPath, Workspace,
};

const INDEX: &str = "registry+https://example.test/index";
const OTHER: &str = "registry+https://example.test/other";
const SPARSE: &str = "sparse+https://example.test/index";
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn declaration(value: &str) -> DependencyDeclaration {
    let manifest = format!("[package]\nname='app'\nversion='1.0.0'\n[dependencies]\nalias={value}");
    let tree = MemoryTree::from([("Cargo.toml", manifest.as_str()), ("src/lib.rs", "")]);
    let ws = Workspace::load(&tree, &["."]).expect("manifest declaration");
    ws.dependency_declarations(ws.package_ids().next().expect("app"))
        .expect("declarations")[0]
        .clone()
}

fn path_declaration() -> DependencyDeclaration {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='app'\nversion='1.0.0'\n[dependencies]\nalias={package='helper',path='vendor',version='4'}",
        ),
        ("src/lib.rs", ""),
        (
            "vendor/Cargo.toml",
            "[package]\nname='helper'\nversion='4.0.0'",
        ),
        ("vendor/src/lib.rs", ""),
    ]);
    let ws = Workspace::load(&tree, &["."]).expect("path declaration");
    ws.dependency_declarations(ws.package_ids().next().expect("app"))
        .expect("declarations")[0]
        .clone()
}

fn catalog(reverse: bool) -> (LockedCatalog, LockedPackageId) {
    let branch = format!("git+https://example.test/repo?branch=feature%2Ftrees#{REV}");
    let default = format!("git+https://example.test/repo#{REV}");
    let revision = format!("git+https://example.test/repo?rev=abc#{REV}");
    let tag = format!("git+https://example.test/repo?tag=v2#{REV}");
    let sources = [
        Some(INDEX),
        Some(INDEX),
        Some(OTHER),
        Some(SPARSE),
        Some(&branch),
        Some(&default),
        Some(&revision),
        Some(&tag),
        None,
        Some(INDEX),
    ];
    let versions = [
        "1.0.0", "2.0.0", "2.0.0", "2.0.0", "2.0.0", "2.0.0", "3.0.0", "2.0.0", "4.0.0", "7.0.0",
    ];
    let mut rows = Vec::new();
    let mut edges = Vec::new();
    for (index, (source, version)) in sources.into_iter().zip(versions).enumerate() {
        let source_line = source.map_or_else(String::new, |source| format!("source={source:?}\n"));
        rows.push(format!(
            "[[package]]\nname='helper'\nversion={version:?}\n{source_line}"
        ));
        if index != 9 {
            edges.push(source.map_or_else(
                || format!("helper {version}"),
                |source| format!("helper {version} ({source})"),
            ));
        }
    }
    if reverse {
        rows.reverse();
    }
    let text = format!(
        "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies={edges:?}\n{}",
        rows.join("")
    );
    let tree = MemoryTree::from([("Cargo.lock", text.as_str())]);
    let catalog = LockedCatalog::load(
        &tree,
        &RelPath::new("Cargo.lock").expect("path"),
        &LoadOptions::default(),
    )
    .expect("catalog");
    let owner = catalog
        .packages()
        .find(|row| row.name() == "app")
        .expect("app")
        .id();
    (catalog, owner)
}

#[test]
fn rename_version_and_registry_select_only_the_owners_exact_outgoing_row() {
    assert_cargo_version_selection();
    for reverse in [false, true] {
        let (catalog, owner) = catalog(reverse);
        let declaration = declaration("{package='helper',version='=2.0.0',registry='private'}");
        let id = catalog
            .select_dependency(
                owner,
                &declaration,
                Some(RegistryIndex::new(Some("private"), INDEX)),
                &LoadOptions::default(),
            )
            .expect("exact edge");
        let row = catalog.package(id).expect("selected row");
        assert_eq!(row.name(), "helper");
        assert_eq!(row.version(), "2.0.0");
        assert_eq!(
            row.source().expect("registry").repository(),
            "https://example.test/index"
        );
        assert!(
            catalog
                .package(owner)
                .expect("owner")
                .dependencies()
                .contains(&id)
        );
        let mut absent = declaration.clone();
        absent.specification.version = Some("=7.0.0".into());
        let error = catalog
            .select_dependency(
                owner,
                &absent,
                Some(RegistryIndex::new(Some("private"), INDEX)),
                &LoadOptions::default(),
            )
            .expect_err("global row is not outgoing");
        assert!(
            error.to_string().contains("no compatible outgoing"),
            "{error}"
        );
        let mut broad = declaration.clone();
        broad.specification.version = Some(">=1, <3".into());
        let error = catalog
            .select_dependency(
                owner,
                &broad,
                Some(RegistryIndex::new(Some("private"), INDEX)),
                &LoadOptions::default(),
            )
            .expect_err("two matching versions");
        assert!(error.to_string().contains("ambiguous"), "{error}");
    }
}

#[test]
fn git_reference_kind_and_local_source_family_are_constraints_not_tree_proofs() {
    let (catalog, owner) = catalog(false);
    for (value, reference) in [
        (
            "{package='helper',git='https://example.test/repo',branch='feature/trees'}",
            Some("Branch"),
        ),
        (
            "{package='helper',git='https://example.test/repo',tag='v2'}",
            Some("Tag"),
        ),
        (
            "{package='helper',git='https://example.test/repo',rev='abc'}",
            Some("Revision"),
        ),
        (
            "{package='helper',git='https://example.test/repo'}",
            Some("DefaultBranch"),
        ),
        ("{package='helper',path='vendor',version='4'}", None),
    ] {
        let observed = if reference.is_none() {
            path_declaration()
        } else {
            declaration(value)
        };
        let id = catalog
            .select_dependency(owner, &observed, None, &LoadOptions::default())
            .expect(value);
        let row = catalog.package(id).expect("selected");
        if let Some(reference) = reference {
            let source = row.source().expect("Git source");
            assert_eq!(source.revision(), Some(REV));
            assert!(format!("{:?}", source.reference()).contains(reference));
        } else {
            assert!(row.source().is_none());
        }
    }
    let wrong = declaration("{package='helper',git='https://example.test/repo',branch='v2'}");
    assert!(
        catalog
            .select_dependency(owner, &wrong, None, &LoadOptions::default())
            .is_err()
    );
}

#[test]
fn workspace_inheritance_selects_its_package_and_source_without_evaluating_cfg() {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[workspace]\nmembers=['member']\n[workspace.dependencies]\nalias={package='helper',version='2',git='https://example.test/repo',tag='v2'}",
        ),
        (
            "member/Cargo.toml",
            "[package]\nname='member'\nversion='1.0.0'\n[target.'cfg(never_enabled)'.dependencies]\nalias={workspace=true,optional=true,features=['extra']}",
        ),
        ("member/src/lib.rs", ""),
    ]);
    let ws = Workspace::load(&tree, &["."]).expect("inherited declaration");
    let declaration = &ws
        .dependency_declarations(ws.package_ids().next().expect("member"))
        .expect("declarations")[0];
    assert_eq!(declaration.condition.as_deref(), Some("cfg(never_enabled)"));
    assert_eq!(declaration.specification.optional, Some(true));
    let (catalog, owner) = catalog(false);
    let id = catalog
        .select_dependency(owner, declaration, None, &LoadOptions::default())
        .expect("observed constraints only");
    assert_eq!(catalog.package(id).expect("selected").version(), "2.0.0");
    let mut missing = declaration.clone();
    missing.inherited = None;
    assert!(
        catalog
            .select_dependency(owner, &missing, None, &LoadOptions::default())
            .is_err()
    );
}

#[test]
fn absent_mismatched_and_malformed_registry_configuration_refuses_beside_valid_indexes() {
    let (catalog, owner) = catalog(false);
    let declaration = declaration("{package='helper',version='2'}");
    for config in [
        None,
        Some(RegistryIndex::new(Some("wrong"), INDEX)),
        Some(RegistryIndex::new(None, "registry+relative")),
        Some(RegistryIndex::new(None, "git+https://example.test/repo")),
    ] {
        let error = catalog
            .select_dependency(owner, &declaration, config, &LoadOptions::default())
            .expect_err("configuration is required and checked");
        assert!(
            matches!(error.kind, LoadErrorKind::Locked { .. }),
            "{error}"
        );
    }
    for source in [INDEX, OTHER, SPARSE] {
        let id = catalog
            .select_dependency(
                owner,
                &declaration,
                Some(RegistryIndex::new(None, source)),
                &LoadOptions::default(),
            )
            .expect("explicit index");
        let selected = catalog.package(id).expect("row").source().expect("source");
        assert_eq!(
            format!("{:?}", selected.kind()),
            if source == SPARSE {
                "SparseRegistry"
            } else {
                "Registry"
            }
        );
    }
    let mut explicit = declaration.clone();
    explicit.specification.registry_index = Some("sparse+https://example.test/index".into());
    assert!(
        catalog
            .select_dependency(owner, &explicit, None, &LoadOptions::default())
            .is_ok()
    );
    assert!(
        catalog
            .select_dependency(
                owner,
                &explicit,
                Some(RegistryIndex::new(None, INDEX)),
                &LoadOptions::default()
            )
            .is_err()
    );
}

#[test]
fn selection_honors_work_and_retained_catalog_item_admission() {
    assert_qualifier_work_admission();
    let (catalog, owner) = catalog(false);
    let declaration = declaration("{package='helper',version='2'}");
    let config = Some(RegistryIndex::new(None, INDEX));
    let mut options = LoadOptions::default();
    options.max_work = 0;
    let error = catalog
        .select_dependency(owner, &declaration, config, &options)
        .expect_err("no comparison grant");
    assert!(matches!(
        error.kind,
        LoadErrorKind::Budget {
            budget: Budget::Work,
            ..
        }
    ));
    options = LoadOptions::default();
    options.max_item_bytes = catalog.usage().item_bytes;
    let error = catalog
        .select_dependency(owner, &declaration, config, &options)
        .expect_err("no query storage grant");
    assert!(matches!(
        error.kind,
        LoadErrorKind::Budget {
            budget: Budget::ItemBytes,
            ..
        }
    ));
    assert!(
        catalog
            .select_dependency(owner, &declaration, config, &LoadOptions::default())
            .is_ok()
    );
}

#[test]
fn malformed_observations_and_unknown_owners_refuse_without_an_alternate_identity() {
    let (catalog, owner) = catalog(false);
    let valid = declaration("{package='helper',git='https://example.test/repo',tag='v2'}");
    for repository in [
        "relative/repository",
        "https://example.test/repo?branch=other",
    ] {
        let mut malformed = valid.clone();
        malformed.specification.git = Some(repository.into());
        let error = catalog
            .select_dependency(owner, &malformed, None, &LoadOptions::default())
            .expect_err(repository);
        assert!(
            matches!(error.kind, LoadErrorKind::Locked { .. }),
            "{error}"
        );
    }
    let mut malformed = valid.clone();
    malformed.specification.version = Some("not a requirement".into());
    assert!(
        catalog
            .select_dependency(owner, &malformed, None, &LoadOptions::default())
            .is_err()
    );
    let tree = MemoryTree::from([(
        "Cargo.lock",
        "version=4\n[[package]]\nname='app'\nversion='1.0.0'\n",
    )]);
    let smaller = LockedCatalog::load(
        &tree,
        &RelPath::new("Cargo.lock").expect("path"),
        &LoadOptions::default(),
    )
    .expect("small catalog");
    let absent = catalog.packages().last().expect("another row").id();
    assert!(smaller.package(absent).is_none());
    let error = smaller
        .select_dependency(absent, &valid, None, &LoadOptions::default())
        .expect_err("unknown row");
    assert!(
        error.to_string().contains("unknown owning lock row"),
        "{error}"
    );
    assert!(
        catalog
            .select_dependency(owner, &valid, None, &LoadOptions::default())
            .is_ok()
    );
}
fn assert_cargo_version_selection() {
    for (requirement, version, compatible) in [
        ("=1.2.3+expected", "1.2.3+actual.001", true),
        ("=1.2", "1.2.9", true),
        ("=1.2", "1.3.0", false),
        (">1.2", "1.2.9", false),
        (">1.2", "1.3.0", true),
        (">=1.2", "1.2.0", true),
        ("<1.2", "1.2.0", false),
        ("<1.2", "1.1.9", true),
        ("<=1.2", "1.2.9", true),
        ("~1.2.3", "1.2.9", true),
        ("~1.2.3", "1.3.0", false),
        ("^0.0.3", "0.0.4", false),
        ("^0.2.3", "0.2.4", true),
        ("^0.2.3", "0.3.0", false),
        ("^0.0", "0.0.9", true),
        ("^0", "0.9.9", true),
        ("1.*.*", "1.9.9", true),
        ("1.*", "1.2.3-alpha", false),
        ("1.* , >=1.2.3-alpha", "1.2.3-alpha", false),
        ("^1.* , >=1.2.3-alpha", "1.2.3-alpha", true),
        ("~1.* , >=1.2.3-alpha", "1.2.3-alpha", false),
        (" >= 1.2.3-alpha , < 2 ", "1.2.3-beta", true),
        (">=1.2.3-alpha, <2", "1.3.0-alpha", false),
        (">=1.2.3-alpha.2", "1.2.3-alpha.11", true),
        (">=1.2.3-alpha.11", "1.2.3-alpha.2", false),
        (">=1.2.3-alpha.2", "1.2.3-alpha.beta", true),
        (
            ">=1.2.3-18446744073709551616",
            "1.2.3-999999999999999999999999999999",
            true,
        ),
        (">=1.2.3-alpha", "1.2.3-alpha.0", true),
        (">=1.2.3-alpha.0", "1.2.3-alpha", false),
        (
            "^18446744073709551615.0.0",
            "18446744073709551615.9.9",
            true,
        ),
        (
            "~0.18446744073709551615.0",
            "0.18446744073709551615.18446744073709551615",
            true,
        ),
        (">18446744073709551615", "18446744073709551615.9.9", false),
    ] {
        let declaration = declaration(&format!("{{package='helper',version={requirement:?}}}"));
        let (catalog, owner) = version_catalog(version);
        let selected = catalog.select_dependency(
            owner,
            &declaration,
            Some(RegistryIndex::new(None, INDEX)),
            &LoadOptions::default(),
        );
        if compatible {
            let id = selected.unwrap_or_else(|error| {
                panic!("{requirement:?} should select {version:?}: {error}")
            });
            assert_eq!(
                catalog.package(id).expect("selected row").version(),
                version
            );
        } else {
            let error =
                selected.expect_err("incompatible requirement must not select an outgoing row");
            assert!(
                matches!(error.kind, LoadErrorKind::Locked { .. }),
                "{error}"
            );
            assert!(
                error.to_string().contains("no compatible outgoing"),
                "{error}"
            );
        }
    }
}

fn version_catalog(version: &str) -> (LockedCatalog, LockedPackageId) {
    let lock = format!(
        "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=[{edge:?}]\n\
         [[package]]\nname='helper'\nversion={version:?}\nsource={INDEX:?}\n",
        edge = format!("helper {version} ({INDEX})"),
    );
    let tree = MemoryTree::from([("Cargo.lock", lock.as_str())]);
    let catalog = LockedCatalog::load(
        &tree,
        &RelPath::new("Cargo.lock").expect("lock path"),
        &LoadOptions::default(),
    )
    .expect("valid Cargo package version and reference");
    let owner = catalog
        .packages()
        .find(|row| row.name() == "app")
        .expect("owning row")
        .id();
    (catalog, owner)
}

fn assert_qualifier_work_admission() {
    let version = format!("1.2.3-{}", "9".repeat(4096));
    let (catalog, owner) = version_catalog(&version);
    let constrained = declaration(&format!("{{package='helper',version='={version}'}}"));
    let config = Some(RegistryIndex::new(None, INDEX));
    let mut limited = LoadOptions::default();
    limited.max_work = 4 * version.len() + 1024;
    let mut unconstrained = constrained.clone();
    unconstrained.specification.version = None;
    catalog
        .select_dependency(owner, &unconstrained, config, &limited)
        .expect("ordinary selection and full version parsing fit the grant");
    let error = catalog
        .select_dependency(owner, &constrained, config, &limited)
        .expect_err("long qualifier matching needs its original work grant");
    assert!(
        matches!(
            error.kind,
            LoadErrorKind::Budget {
                budget: Budget::Work,
                ..
            }
        ),
        "{error}"
    );
    catalog
        .select_dependency(owner, &constrained, config, &LoadOptions::default())
        .expect("same qualifier comparison succeeds with its work grant");
}
