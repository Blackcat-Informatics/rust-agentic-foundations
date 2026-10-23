// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Package discovery under explicit roots, and the targets of each package.

use raf_source::{
    Budget, LoadError, LoadErrorKind, LoadOptions, MemoryTree, Namespace, Origin, PackageId,
    RelPath, ResolveError, SourceTrees, TargetKind, TargetProblem, Workspace,
};

const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";

#[test]
fn relative_queries_select_the_reading_package_without_crossing_source_roots() {
    let first = tree(&[
        ("Cargo.toml", DEMO),
        (
            "src/lib.rs",
            "pub mod provider { pub struct First; fn hidden() {} }\n\
            pub mod reader { pub use crate::provider::First as Routed; }\n\
            pub mod first_only {}",
        ),
    ]);
    let second = tree(&[
        ("Cargo.toml", DEMO),
        (
            "src/lib.rs",
            "pub mod provider { pub struct Second; fn hidden() {} }\n\
            pub mod reader { pub use crate::provider::Second as Routed; }",
        ),
    ]);
    let dependencies: [&dyn raf_source::Tree; 1] = [&second];
    let sources = SourceTrees::new(&first, &dependencies);
    let roots: Vec<_> = sources
        .authorities()
        .map(|authority| sources.root(authority).expect("root"))
        .collect();
    let ws = Workspace::load_from(&sources, &roots, &LoadOptions::default()).expect("loads");
    let ids: Vec<_> = ws.package_ids().collect();
    assert_eq!(ids.len(), 2);
    assert!(matches!(
        ws.resolve_from("demo::reader", "self::Routed"),
        Err(ResolveError::Ambiguous(_))
    ));
    assert!(matches!(
        ws.resolve_from_in("demo::reader", "self::Routed", Namespace::Type),
        Err(ResolveError::Ambiguous(_))
    ));
    for (id, own, foreign) in [(ids[0], "First", "Second"), (ids[1], "Second", "First")] {
        assert_relative_package_route(&ws, id, own, foreign);
    }
    assert!(
        ws.resolve_from_package_id(ids[0], "demo::first_only", "crate::provider::First")
            .is_ok()
    );
    assert_eq!(
        ws.resolve_from_package_id(ids[1], "demo::first_only", "crate::provider::First"),
        Err(ResolveError::UnknownModule("demo::first_only".into()))
    );
    assert_eq!(
        ws.resolve_from_package_id_in(
            ids[1],
            "demo::first_only",
            "crate::provider::First",
            Namespace::Type
        ),
        Err(ResolveError::UnknownModule("demo::first_only".into()))
    );
    let smaller = Workspace::load(&first, &["."]).expect("one package");
    assert!(smaller.package(ids[1]).is_none());
    assert_eq!(
        smaller.resolve_from_package_id(ids[1], "demo::reader", "self::Routed"),
        Err(ResolveError::UnknownPackageId(ids[1]))
    );
    assert_eq!(
        smaller.resolve_from_package_id_in(ids[1], "demo::reader", "self::Routed", Namespace::Type),
        Err(ResolveError::UnknownPackageId(ids[1]))
    );
    assert!(
        smaller
            .resolve_from_package_id(ids[0], "demo::reader", "self::Routed")
            .is_ok()
    );
}

fn assert_relative_package_route(ws: &Workspace, id: PackageId, own: &str, foreign: &str) {
    let resolved = ws
        .resolve_from_package_id(id, "demo::reader", "self::Routed")
        .expect("selected reexport");
    assert_eq!(resolved.path, format!("demo::provider::{own}"));
    assert_eq!(resolved.sites.len(), 1);
    assert_eq!(resolved.namespace, Namespace::Type);
    assert!(
        resolved.chain.len() >= 2,
        "actual reexport route: {:?}",
        resolved.chain
    );
    let item = ws.item(resolved.sites[0]).expect("definition");
    assert_eq!(ws.target(item.target).expect("target").package_id, id);
    assert_eq!(
        item.file.authority(),
        ws.package(id).expect("package").manifest.authority()
    );
    let value = ws
        .resolve_from_package_id_in(id, "demo::reader", "self::Routed", Namespace::Value)
        .expect("unit struct constructor");
    assert_eq!(value.namespace, Namespace::Value);
    assert_eq!(value.sites, resolved.sites);
    for path in [
        format!("crate::provider::{foreign}"),
        "crate::provider::hidden".into(),
    ] {
        assert!(
            matches!(
                ws.resolve_from_package_id(id, "demo::reader", &path),
                Err(ResolveError::NotFound(_))
            ),
            "{path}"
        );
        assert!(
            matches!(
                ws.resolve_from_package_id_in(id, "demo::reader", &path, Namespace::Value),
                Err(ResolveError::NotFound(_))
            ),
            "{path}"
        );
    }
}

#[test]
fn independently_rooted_same_name_packages_resolve_only_by_their_selected_identity() {
    let first = tree(&[("Cargo.toml", DEMO), ("src/lib.rs", "pub fn first() {}")]);
    let second = tree(&[("Cargo.toml", DEMO), ("src/lib.rs", "pub fn second() {}")]);
    let dependencies: [&dyn raf_source::Tree; 1] = [&second];
    let sources = SourceTrees::new(&first, &dependencies);
    let roots: Vec<_> = sources
        .authorities()
        .map(|authority| sources.root(authority).expect("available root"))
        .collect();
    let ws = Workspace::load_from(&sources, &roots, &LoadOptions::default()).expect("loads");
    let ids: Vec<_> = ws.package_ids().collect();
    assert_eq!(ids.len(), 2);
    assert!(matches!(
        ws.resolve_in_package("demo", "demo::first"),
        Err(ResolveError::AmbiguousPackageName(name)) if name == "demo"
    ));
    let mut definitions = Vec::new();
    for (id, own, foreign) in [(ids[0], "first", "second"), (ids[1], "second", "first")] {
        let package = ws.package(id).expect("selected package");
        let resolved = ws
            .resolve_in_package_id(id, &format!("demo::{own}"))
            .expect("actual selected definition");
        assert_eq!(resolved.sites.len(), 1);
        let item = ws.item(resolved.sites[0]).expect("definition");
        assert_eq!(item.file.authority(), package.manifest.authority());
        assert_eq!(item.file.as_str(), "src/lib.rs");
        assert_eq!(
            ws.source(&item.file),
            Some(format!("pub fn {own}() {{}}").as_str())
        );
        assert!(matches!(
            ws.resolve_in_package_id(id, &format!("demo::{foreign}")),
            Err(ResolveError::NotFound(_))
        ));
        definitions.push(item.source_key());
    }
    assert_ne!(definitions[0], definitions[1]);

    let mut exact = LoadOptions::default();
    exact.max_item_bytes = ws.usage().item_bytes;
    let at_limit =
        Workspace::load_from(&sources, &roots, &exact).expect("exact original allowance");
    assert_eq!(ws, at_limit);
    let mut short = exact;
    short.max_item_bytes -= 1;
    let error = Workspace::load_from(&sources, &roots, &short).expect_err("one byte too little");
    assert!(matches!(
        error.kind,
        LoadErrorKind::Budget {
            budget: Budget::ItemBytes,
            ..
        }
    ));
}

#[test]
fn inherited_workspaces_and_renamed_path_dependencies_stay_within_each_source() {
    let manifest = "[workspace]\nmembers = ['app', 'helper']\n\
        [workspace.package]\nedition = '2021'\n\
        [workspace.dependencies]\nshared = { package = 'helper', path = 'helper' }\n";
    let app = "[package]\nname = 'demo'\nversion = '0.1.0'\nedition.workspace = true\n\
        [dependencies]\nshared.workspace = true\n";
    let helper = "[package]\nname = 'helper'\nversion = '0.1.0'\nedition.workspace = true\n";
    let make = |workspace, member, definition| {
        tree(&[
            ("Cargo.toml", workspace),
            ("app/Cargo.toml", app),
            ("app/src/lib.rs", member),
            ("helper/Cargo.toml", helper),
            ("helper/src/lib.rs", definition),
        ])
    };
    let old_edition = manifest.replace("2021", "2015");
    let first = make(
        &old_edition,
        "extern crate shared; pub use shared::First as Selected;",
        "pub struct First;",
    );
    let second = make(
        manifest,
        "pub use shared::Second as Selected;",
        "pub struct Second;",
    );
    let dependencies: [&dyn raf_source::Tree; 1] = [&second];
    let sources = SourceTrees::new(&first, &dependencies);
    let roots: Vec<_> = sources
        .authorities()
        .map(|authority| sources.root(authority).expect("available root"))
        .collect();
    let ws = Workspace::load_from(&sources, &roots, &LoadOptions::default()).expect("loads");
    let apps: Vec<_> = ws
        .package_ids()
        .filter(|id| ws.package(*id).expect("package").name == "demo")
        .collect();
    assert_eq!(apps.len(), 2);
    for (id, definition) in [(apps[0], "First"), (apps[1], "Second")] {
        let resolved = ws
            .resolve_in_package_id(id, "demo::Selected")
            .expect("inherited renamed dependency resolves");
        assert_eq!(resolved.path, format!("helper::{definition}"));
        assert_eq!(resolved.sites.len(), 1);
        let item = ws.item(resolved.sites[0]).expect("helper definition");
        assert_eq!(
            item.file.authority(),
            ws.package(id).expect("app").manifest.authority()
        );
        assert_eq!(item.file.as_str(), "helper/src/lib.rs");
    }
}

fn tree(files: &[(&str, &str)]) -> MemoryTree {
    files.iter().copied().collect()
}

fn load(files: &[(&str, &str)], roots: &[&str]) -> Result<Workspace, LoadError> {
    Workspace::load(&tree(files), roots)
}

fn summary(ws: &Workspace) -> Vec<(String, TargetKind, String, String, bool)> {
    ws.targets()
        .map(|t| {
            (
                t.name.clone(),
                t.kind,
                t.crate_path.clone(),
                t.root.to_string(),
                t.shipping,
            )
        })
        .collect()
}

fn entry(
    name: &str,
    kind: TargetKind,
    crate_path: &str,
    root: &str,
    shipping: bool,
) -> (String, TargetKind, String, String, bool) {
    (
        name.to_owned(),
        kind,
        crate_path.to_owned(),
        root.to_owned(),
        shipping,
    )
}

#[test]
fn each_target_kind_with_default_paths_has_its_name_kind_crate_path_and_shipping_flag() {
    let ws = load(
        &[
            ("Cargo.toml", DEMO),
            ("src/lib.rs", ""),
            ("src/main.rs", ""),
            ("src/bin/x.rs", ""),
            ("src/bin/dir/main.rs", ""),
            ("tests/it.rs", ""),
            ("tests/grouped/main.rs", ""),
            ("benches/b.rs", ""),
            ("benches/grouped/main.rs", ""),
            ("examples/e.rs", ""),
            ("examples/grouped/main.rs", ""),
            ("build.rs", ""),
        ],
        &["."],
    )
    .expect("loads");
    assert_eq!(
        summary(&ws),
        [
            entry("demo", TargetKind::Lib, "demo", "src/lib.rs", true),
            entry(
                "demo",
                TargetKind::Bin,
                "demo::bin_crate::demo",
                "src/main.rs",
                true
            ),
            entry(
                "dir",
                TargetKind::Bin,
                "demo::bin_crate::dir",
                "src/bin/dir/main.rs",
                true
            ),
            entry(
                "x",
                TargetKind::Bin,
                "demo::bin_crate::x",
                "src/bin/x.rs",
                true
            ),
            entry(
                "grouped",
                TargetKind::Test,
                "demo::test_crate::grouped",
                "tests/grouped/main.rs",
                false
            ),
            entry(
                "it",
                TargetKind::Test,
                "demo::test_crate::it",
                "tests/it.rs",
                false
            ),
            entry(
                "b",
                TargetKind::Bench,
                "demo::bench_crate::b",
                "benches/b.rs",
                false
            ),
            entry(
                "grouped",
                TargetKind::Bench,
                "demo::bench_crate::grouped",
                "benches/grouped/main.rs",
                false
            ),
            entry(
                "e",
                TargetKind::Example,
                "demo::example_crate::e",
                "examples/e.rs",
                false
            ),
            entry(
                "grouped",
                TargetKind::Example,
                "demo::example_crate::grouped",
                "examples/grouped/main.rs",
                false
            ),
            entry(
                "build-script-build",
                TargetKind::BuildScript,
                "demo::build_script",
                "build.rs",
                true,
            ),
        ]
    );
}

#[test]
fn each_target_kind_with_explicit_paths_resolves_beside_its_default_neighbour() {
    let manifest = format!(
        "{DEMO}\nautobins = false\nautotests = false\nautobenches = false\nautoexamples = false\n\
         [lib]\nname = \"core-lib\"\npath = \"lib/root.rs\"\n\n\
         [[bin]]\nname = \"x\"\npath = \"tools/x.rs\"\n\n\
         [[test]]\nname = \"it\"\npath = \"checks/it.rs\"\n\n\
         [[bench]]\nname = \"b\"\npath = \"perf/b.rs\"\n\n\
         [[example]]\nname = \"e\"\npath = \"demos/e.rs\"\n"
    );
    let ws = load(
        &[
            ("Cargo.toml", &manifest),
            ("lib/root.rs", ""),
            ("tools/x.rs", ""),
            ("checks/it.rs", ""),
            ("perf/b.rs", ""),
            ("demos/e.rs", ""),
        ],
        &["."],
    )
    .expect("loads");
    assert_eq!(
        summary(&ws),
        [
            entry("core-lib", TargetKind::Lib, "core_lib", "lib/root.rs", true),
            entry(
                "x",
                TargetKind::Bin,
                "demo::bin_crate::x",
                "tools/x.rs",
                true
            ),
            entry(
                "it",
                TargetKind::Test,
                "demo::test_crate::it",
                "checks/it.rs",
                false
            ),
            entry(
                "b",
                TargetKind::Bench,
                "demo::bench_crate::b",
                "perf/b.rs",
                false
            ),
            entry(
                "e",
                TargetKind::Example,
                "demo::example_crate::e",
                "demos/e.rs",
                false
            ),
        ]
    );
}

#[test]
fn the_root_module_of_every_target_is_recorded_under_its_crate_path() {
    let ws = load(
        &[
            ("Cargo.toml", DEMO),
            ("src/lib.rs", "pub fn go() {}"),
            ("tests/it.rs", "fn case() {}"),
        ],
        &["."],
    )
    .expect("loads");
    for target in ws.targets() {
        let file = ws.file(&target.root).expect("the root file is loaded");
        assert!(
            file.reached
                .iter()
                .any(|reach| reach.module_path == target.crate_path && !reach.test),
            "{target:?}"
        );
        assert_eq!(ws.source(&target.root), Some(file.text.as_str()));
    }
    let ids: Vec<_> = ws.target_ids().collect();
    assert_eq!(ids.len(), 2);
    assert_eq!(ws.target(ids[1]).expect("target").name, "it");
}

#[test]
fn a_library_and_a_binary_with_one_name_resolve_under_their_own_crate_paths() {
    let ws = load(
        &[
            ("Cargo.toml", DEMO),
            ("src/lib.rs", ""),
            ("src/main.rs", ""),
        ],
        &["."],
    )
    .expect("loads");
    let paths: Vec<_> = ws.targets().map(|t| t.crate_path.as_str()).collect();
    assert_eq!(paths, ["demo", "demo::bin_crate::demo"]);
}

#[test]
fn a_manifest_with_a_trailing_comment_and_single_quoted_values_is_honoured() {
    let manifest = "[package]\nname = 'demo' # the name\nversion = \"0.1.0\"\n\n\
                    [lib]\npath = 'engine/lib.rs' # moved\n";
    let ws = load(&[("Cargo.toml", manifest), ("engine/lib.rs", "")], &["."]).expect("loads");
    assert_eq!(
        summary(&ws),
        [entry(
            "demo",
            TargetKind::Lib,
            "demo",
            "engine/lib.rs",
            true
        )]
    );
}

#[test]
fn a_virtual_manifest_contributes_nothing_but_its_children_are_found() {
    let virtual_only =
        load(&[("Cargo.toml", "[workspace]\nmembers = []\n")], &["."]).expect("loads");
    assert_eq!(virtual_only.targets().count(), 0);
    assert_eq!(virtual_only.packages().len(), 0);

    let with_member = load(
        &[
            ("Cargo.toml", "[workspace]\nmembers = [\"crates/a\"]\n"),
            ("crates/a/Cargo.toml", DEMO),
            ("crates/a/src/lib.rs", ""),
        ],
        &["."],
    )
    .expect("loads");
    assert_eq!(with_member.packages().len(), 1);
    assert_eq!(
        with_member.packages()[0].manifest.as_str(),
        "crates/a/Cargo.toml"
    );
}

#[test]
fn discovery_target_and_module_failures_precede_later_load_phases() {
    let malformed = load(
        &[
            ("a/Cargo.toml", "not = [ valid TOML"),
            (
                "b/Cargo.toml",
                "[package]\nname = \"b\"\n[[bin]]\nname = \"missing\"\npath = \"missing.rs\"\n",
            ),
        ],
        &["."],
    )
    .expect_err("discovery fails before checking target paths");
    assert!(
        matches!(malformed.kind, LoadErrorKind::Manifest { .. }),
        "{malformed}"
    );
    assert_eq!(malformed.file.as_str(), "a/Cargo.toml");

    let manifest = format!("{DEMO}[[bin]]\nname = \"missing\"\npath = \"missing.rs\"\n");
    let target = load(
        &[("Cargo.toml", &manifest), ("src/lib.rs", "pub fn (")],
        &["."],
    )
    .expect_err("target validation precedes module parsing");
    assert!(
        matches!(target_problem(&target), TargetProblem::MissingFile { .. }),
        "{target}"
    );

    let root = "use super::absent;\nmod broken;\n";
    let module = load(
        &[
            ("Cargo.toml", DEMO),
            ("src/lib.rs", root),
            ("src/broken.rs", "pub fn ("),
        ],
        &["."],
    )
    .expect_err("module parsing precedes index use-path validation");
    assert!(
        matches!(module.kind, LoadErrorKind::Syntax { .. }),
        "{module}"
    );
    assert_eq!(module.file.as_str(), "src/broken.rs");
    let index = load(
        &[
            ("Cargo.toml", DEMO),
            ("src/lib.rs", root),
            ("src/broken.rs", ""),
        ],
        &["."],
    )
    .expect_err("the repaired module permits the later index failure to surface");
    assert!(
        matches!(index.kind, LoadErrorKind::SuperAboveRoot { .. }),
        "{index}"
    );
    assert!(
        load(
            &[
                ("Cargo.toml", DEMO),
                ("src/lib.rs", "mod broken;"),
                ("src/broken.rs", "")
            ],
            &["."]
        )
        .is_ok()
    );
}

#[test]
fn an_empty_root_list_is_a_no_roots_error_beside_dot_which_loads() {
    let files = [("Cargo.toml", DEMO), ("src/lib.rs", "")];
    let error = load(&files, &[]).expect_err("no roots is an error");
    assert!(matches!(error.kind, LoadErrorKind::NoRoots), "{error}");
    assert_eq!(load(&files, &["."]).expect("loads").packages().len(), 1);
}

#[test]
fn a_package_nested_below_a_package_without_a_workspace_is_not_discovered() {
    let files = [
        ("Cargo.toml", DEMO),
        ("src/lib.rs", ""),
        (
            "tests/fixture/Cargo.toml",
            "[package]\nname = \"fixture\"\n",
        ),
        ("tests/fixture/src/lib.rs", ""),
    ];
    let ws = load(&files, &["."]).expect("loads");
    assert_eq!(ws.packages().len(), 1);
    assert_eq!(ws.packages()[0].name, "demo");

    // Beside it: the same layout with a `[workspace]` table goes on below.
    let with_workspace = DEMO.to_owned() + "\n[workspace]\n";
    let files = [
        ("Cargo.toml", with_workspace.as_str()),
        ("src/lib.rs", ""),
        ("crates/a/Cargo.toml", "[package]\nname = \"a\"\n"),
        ("crates/a/src/lib.rs", ""),
    ];
    let ws = load(&files, &["."]).expect("loads");
    let names: Vec<_> = ws.packages().iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["demo", "a"]);
}

#[test]
fn target_and_hidden_directories_are_skipped_beside_a_discovered_sibling() {
    let ws = load(
        &[
            ("crates/a/Cargo.toml", "[package]\nname = \"a\"\n"),
            ("crates/a/src/lib.rs", ""),
            (
                "crates/target/Cargo.toml",
                "[package]\nname = \"skipped-target\"\n",
            ),
            ("crates/target/src/lib.rs", ""),
            (
                "crates/.hidden/Cargo.toml",
                "[package]\nname = \"skipped-hidden\"\n",
            ),
            ("crates/.hidden/src/lib.rs", ""),
        ],
        &["crates"],
    )
    .expect("loads");
    let names: Vec<_> = ws.packages().iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["a"]);
}

#[test]
fn only_the_named_roots_are_searched() {
    let files = [
        ("a/Cargo.toml", "[package]\nname = \"a\"\n"),
        ("a/src/lib.rs", ""),
        ("b/Cargo.toml", "[package]\nname = \"b\"\n"),
        ("b/src/lib.rs", ""),
    ];
    let only_a = load(&files, &["a"]).expect("loads");
    assert_eq!(only_a.packages().len(), 1);
    let both = load(&files, &["a", "b"]).expect("loads");
    assert_eq!(both.packages().len(), 2);
    // Overlapping roots find one package once.
    let overlapping = load(&files, &[".", "a"]).expect("loads");
    assert_eq!(overlapping.packages().len(), 2);
}

#[test]
fn a_missing_root_is_an_error_beside_an_existing_one() {
    let files = [
        ("a/Cargo.toml", "[package]\nname = \"a\"\n"),
        ("a/src/lib.rs", ""),
    ];
    let error = load(&files, &["missing"]).expect_err("missing root");
    assert!(matches!(error.kind, LoadErrorKind::Tree(_)), "{error}");
    assert!(error.to_string().contains("missing"), "{error}");
    assert!(load(&files, &["a"]).is_ok());
}

#[test]
fn autodiscovery_flags_turn_inferred_targets_off_but_explicit_tables_are_still_found() {
    let manifest = "[package]\nname = \"demo\"\nautobins = false\nautotests = false\n\
         autobenches = false\nautoexamples = false\n\n\
         [[test]]\nname = \"kept\"\npath = \"tests/kept.rs\"\n";
    let files = [
        ("Cargo.toml", manifest),
        ("src/main.rs", ""),
        ("src/bin/extra.rs", ""),
        ("tests/kept.rs", ""),
        ("tests/inferred.rs", ""),
        ("benches/inferred.rs", ""),
        ("examples/inferred.rs", ""),
    ];
    let ws = load(&files, &["."]).expect("loads");
    assert_eq!(
        summary(&ws),
        [entry(
            "kept",
            TargetKind::Test,
            "demo::test_crate::kept",
            "tests/kept.rs",
            false
        )]
    );
    // The same package with the flags left alone infers them all.
    let ws = load(
        &[
            ("Cargo.toml", DEMO),
            ("src/main.rs", ""),
            ("src/bin/extra.rs", ""),
            ("tests/inferred.rs", ""),
            ("benches/inferred.rs", ""),
            ("examples/inferred.rs", ""),
        ],
        &["."],
    )
    .expect("loads");
    assert_eq!(ws.targets().count(), 5);
}

#[test]
fn two_packages_with_one_name_are_an_error_naming_both_manifests() {
    let files = [
        ("a/Cargo.toml", "[package]\nname = \"same\"\n"),
        ("a/src/lib.rs", ""),
        ("b/Cargo.toml", "[package]\nname = \"same\"\n"),
        ("b/src/lib.rs", ""),
    ];
    let error = load(&files, &["."]).expect_err("duplicate");
    assert_eq!(error.file.as_str(), "b/Cargo.toml");
    match &error.kind {
        LoadErrorKind::Target(problem) => assert_eq!(
            **problem,
            TargetProblem::DuplicatePackage {
                name: "same".to_owned(),
                with: RelPath::new("a/Cargo.toml").expect("path"),
            }
        ),
        other => panic!("{other:?}"),
    }
    assert!(error.to_string().contains("a/Cargo.toml"), "{error}");
    // With distinct names the same layout loads.
    let distinct = [
        ("a/Cargo.toml", "[package]\nname = \"one\"\n"),
        ("a/src/lib.rs", ""),
        ("b/Cargo.toml", "[package]\nname = \"two\"\n"),
        ("b/src/lib.rs", ""),
    ];
    assert!(load(&distinct, &["."]).is_ok());
}

fn target_problem(error: &LoadError) -> &TargetProblem {
    match &error.kind {
        LoadErrorKind::Target(problem) => problem,
        other => panic!("not a target problem: {other:?}\n{error}"),
    }
}

fn key_of(error: &LoadError) -> &str {
    match error.origin.as_deref() {
        Some(Origin::ManifestKey { key }) => key,
        other => panic!("not a manifest key: {other:?}"),
    }
}

#[test]
fn an_explicit_target_file_that_is_missing_is_an_error_naming_the_manifest_key() {
    let manifest = format!("{DEMO}\n[[bin]]\nname = \"tool\"\npath = \"src/missing.rs\"\n");
    let error = load(&[("Cargo.toml", &manifest)], &["."]).expect_err("missing");
    assert_eq!(error.file.as_str(), "Cargo.toml");
    assert_eq!(key_of(&error), "bin.path");
    assert!(matches!(
        target_problem(&error),
        TargetProblem::MissingFile { .. }
    ));
    // With the file present the same table loads.
    assert!(load(&[("Cargo.toml", &manifest), ("src/missing.rs", "")], &["."]).is_ok());

    let lib = format!("{DEMO}\n[lib]\npath = \"nowhere.rs\"\n");
    let error = load(&[("Cargo.toml", &lib)], &["."]).expect_err("missing lib");
    assert_eq!(key_of(&error), "lib.path");
}

#[test]
fn a_target_table_without_a_name_is_an_error_naming_the_key() {
    for table in ["bin", "test", "bench", "example"] {
        let manifest = format!("{DEMO}\n[[{table}]]\npath = \"x.rs\"\n");
        let error = load(&[("Cargo.toml", &manifest), ("x.rs", "")], &["."]).expect_err(table);
        assert_eq!(key_of(&error), format!("{table}.name"));
        assert_eq!(*target_problem(&error), TargetProblem::NoName);
    }
}

#[test]
fn two_targets_sharing_one_root_are_an_error_naming_the_second() {
    let manifest = format!(
        "{DEMO}\n[[bin]]\nname = \"one\"\npath = \"src/shared.rs\"\n\n\
         [[bin]]\nname = \"two\"\npath = \"src/shared.rs\"\n"
    );
    let error =
        load(&[("Cargo.toml", &manifest), ("src/shared.rs", "")], &["."]).expect_err("shared");
    match target_problem(&error) {
        TargetProblem::SharedRoot { root, other } => {
            assert_eq!(root.as_str(), "src/shared.rs");
            assert_eq!(other, "one");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(key_of(&error), "bin.path");
}

#[test]
fn a_target_path_that_leaves_the_tree_is_refused_and_a_path_that_stays_is_accepted() {
    for path in ["../../outside.rs", "/abs/outside.rs"] {
        let manifest = format!("{DEMO}\n[[bin]]\nname = \"tool\"\npath = \"{path}\"\n");
        let error = load(&[("Cargo.toml", &manifest), ("outside.rs", "")], &["."]).expect_err(path);
        assert!(
            matches!(&error.kind, LoadErrorKind::Tree(e) if e.path() == path),
            "{error}"
        );
        assert_eq!(key_of(&error), "bin.path");
    }
    let inside = format!("{DEMO}\n[[bin]]\nname = \"tool\"\npath = \"../shared/tool.rs\"\n");
    let ws = load(
        &[("pkg/Cargo.toml", &inside), ("shared/tool.rs", "")],
        &["pkg"],
    )
    .expect("a path to a sibling directory stays inside the tree");
    assert_eq!(ws.targets().count(), 1);
}

#[test]
fn a_manifest_that_is_not_toml_or_has_neither_table_is_a_typed_error() {
    let error = load(&[("Cargo.toml", "[package\nname =")], &["."]).expect_err("bad toml");
    assert!(
        matches!(error.kind, LoadErrorKind::Manifest { .. }),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "Cargo.toml");
    let error = load(&[("Cargo.toml", "[dependencies]\n")], &["."]).expect_err("neither");
    assert!(
        matches!(error.kind, LoadErrorKind::Manifest { .. }),
        "{error}"
    );
    let error = load(&[("Cargo.toml", "[package]\nname = 3\n")], &["."]).expect_err("typed");
    assert_eq!(key_of(&error), "package.name");
}

#[test]
fn a_manifest_over_the_cap_is_refused_naming_it_and_one_of_exactly_the_cap_loads() {
    let padding = |len: usize| {
        let head = format!("{DEMO}# ");
        let tail = "\n";
        let fill = len - head.len() - tail.len();
        format!("{head}{}{tail}", "x".repeat(fill))
    };
    let cap = 8 * 1024 * 1024;
    let exact = padding(cap);
    assert_eq!(exact.len(), cap);
    let over = padding(cap + 1);
    assert_eq!(over.len(), cap + 1);

    assert!(load(&[("Cargo.toml", &exact), ("src/lib.rs", "")], &["."]).is_ok());
    let error = load(&[("Cargo.toml", &over), ("src/lib.rs", "")], &["."]).expect_err("over");
    assert!(
        matches!(&error.kind, LoadErrorKind::Tree(e) if matches!(**e, raf_source::TreeError::TooLarge { .. })),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "Cargo.toml");
}

#[test]
fn deeply_nested_inline_tables_are_a_typed_error_on_the_callers_stack_and_a_shallow_one_loads() {
    let nested = |levels: usize| {
        let mut value = "1".to_owned();
        for _ in 0..levels {
            value = format!("{{ a = {value} }}");
        }
        format!("{DEMO}\n[metadata]\nvalue = {value}\n")
    };
    let error =
        load(&[("Cargo.toml", &nested(1000)), ("src/lib.rs", "")], &["."]).expect_err("too deep");
    assert!(
        matches!(error.kind, LoadErrorKind::Manifest { .. }),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "Cargo.toml");
    assert!(load(&[("Cargo.toml", &nested(8)), ("src/lib.rs", "")], &["."]).is_ok());
}

#[test]
fn a_shared_test_helper_directory_is_reached_by_two_targets_and_listed_once() {
    let ws = load(
        &[
            ("Cargo.toml", DEMO),
            ("tests/a.rs", "mod common;"),
            ("tests/b.rs", "mod common;"),
            ("tests/common/mod.rs", "pub fn helper() {}"),
        ],
        &["."],
    )
    .expect("loads");
    // `tests/common` has no `main.rs`, so it is no target.
    let names: Vec<_> = ws.targets().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["a", "b"]);
    let common = ws
        .file(&RelPath::new("tests/common/mod.rs").expect("path"))
        .expect("loaded");
    let mut paths: Vec<_> = common
        .reached
        .iter()
        .map(|r| r.module_path.as_str())
        .collect();
    paths.sort_unstable();
    assert_eq!(
        paths,
        ["demo::test_crate::a::common", "demo::test_crate::b::common"]
    );
    assert_eq!(
        ws.files()
            .filter(|f| f.path.as_str() == "tests/common/mod.rs")
            .count(),
        1
    );
}

#[test]
fn targets_are_sorted_and_two_loads_are_equal() {
    let files = [
        ("b/Cargo.toml", "[package]\nname = \"alpha\"\n"),
        ("b/src/lib.rs", ""),
        ("a/Cargo.toml", "[package]\nname = \"zulu\"\n"),
        ("a/src/lib.rs", ""),
    ];
    let first = load(&files, &["."]).expect("loads");
    let reversed: Vec<_> = files.iter().rev().copied().collect();
    let second = load(&reversed, &["."]).expect("loads in reversed insertion order");
    let reversed_roots = load(&files, &["b", "a"]).expect("reversed roots load");
    let names = |ws: &Workspace| -> Vec<String> { ws.targets().map(|t| t.name.clone()).collect() };
    assert_eq!(names(&first), ["zulu", "alpha"]);
    assert_eq!(names(&first), names(&second));
    let manifests: Vec<_> = first
        .packages()
        .iter()
        .map(|package| (package.manifest.as_str(), package.name.as_str()))
        .collect();
    assert_eq!(
        manifests,
        [("a/Cargo.toml", "zulu"), ("b/Cargo.toml", "alpha")]
    );
    assert_eq!(first.packages(), second.packages());
    let identities = |ws: &Workspace| -> Vec<_> {
        ws.targets()
            .map(|target| (target.package_id, target.package.clone()))
            .collect()
    };
    assert_eq!(identities(&first), identities(&second));
    assert_eq!(identities(&first), identities(&reversed_roots));
}

fn duplicate_path(error: &LoadError) -> (&str, &str, &str) {
    match target_problem(error) {
        TargetProblem::DuplicateCratePath {
            crate_path,
            other,
            with,
        } => (crate_path, other, with.as_str()),
        other => panic!("{other:?}"),
    }
}

#[test]
fn two_targets_with_one_crate_path_are_an_error_naming_both_beside_distinct_names() {
    for (table, directory, segment) in [
        ("test", "tests", "test_crate"),
        ("bench", "benches", "bench_crate"),
        ("example", "examples", "example_crate"),
        ("bin", "src/bin", "bin_crate"),
    ] {
        // Two explicit tables with one name.
        let manifest = format!(
            "{DEMO}\n[[{table}]]\nname = \"a\"\npath = \"{directory}/one.rs\"\n\n\
             [[{table}]]\nname = \"a\"\npath = \"{directory}/two.rs\"\n"
        );
        let one = format!("{directory}/one.rs");
        let two = format!("{directory}/two.rs");
        let files = [
            ("Cargo.toml", manifest.as_str()),
            (one.as_str(), ""),
            (two.as_str(), ""),
        ];
        let error = load(&files, &["."]).expect_err(table);
        let (path, other, with) = duplicate_path(&error);
        assert_eq!(path, format!("demo::{segment}::a"));
        assert_eq!((other, with), ("a", "Cargo.toml"));
        assert_eq!(error.file.as_str(), "Cargo.toml");
        assert_eq!(key_of(&error), format!("{table}.path"));

        // Names that differ only in `-` and `_` have one crate path too.
        let dashed = format!("{directory}/a-b.rs");
        let underscored = format!("{directory}/a_b.rs");
        let files = [
            ("Cargo.toml", DEMO),
            (dashed.as_str(), ""),
            (underscored.as_str(), ""),
        ];
        let error = load(&files, &["."]).expect_err(table);
        assert_eq!(duplicate_path(&error).0, format!("demo::{segment}::a_b"));

        // Distinct names are accepted.
        let distinct = format!("{directory}/a-b.rs");
        let other = format!("{directory}/a_c.rs");
        let files = [
            ("Cargo.toml", DEMO),
            (distinct.as_str(), ""),
            (other.as_str(), ""),
        ];
        assert!(load(&files, &["."]).is_ok(), "{table}");
    }
}

#[test]
fn two_packages_whose_targets_share_crate_paths_load_and_are_told_apart_by_package() {
    let files = [
        ("a/Cargo.toml", "[package]\nname = \"x-y\"\n"),
        ("a/src/lib.rs", "pub fn in_a() {}\n"),
        ("a/src/bin/tool.rs", "fn main() {}\n"),
        ("b/Cargo.toml", "[package]\nname = \"x_y\"\n"),
        ("b/src/lib.rs", "pub fn in_b() {}\n"),
        ("b/src/bin/tool.rs", "fn main() {}\n"),
    ];
    let ws = load(&files, &["."]).expect("the crates are keyed by package");
    let paths: Vec<_> = ws
        .targets()
        .map(|t| (t.package.as_str(), t.crate_path.as_str()))
        .collect();
    assert_eq!(
        paths,
        [
            ("x-y", "x_y"),
            ("x-y", "x_y::bin_crate::tool"),
            ("x_y", "x_y"),
            ("x_y", "x_y::bin_crate::tool"),
        ]
    );
    for path in ["x_y", "x_y::in_a", "x_y::bin_crate::tool"] {
        match ws.resolve(path) {
            Err(ResolveError::Ambiguous(ids)) => assert_eq!(ids.len(), 2, "{path}"),
            other => panic!("{path}: {other:?}"),
        }
    }
    let a = ws
        .resolve_in_package("x-y", "x_y::in_a")
        .expect("in its package");
    assert_eq!(
        ws.item(a.sites[0]).expect("item").file.as_str(),
        "a/src/lib.rs"
    );
    assert!(ws.resolve_in_package("x_y", "x_y::in_a").is_err());
    let tool = ws
        .resolve_in_package("x_y", "x_y::bin_crate::tool")
        .expect("the binary of one package");
    assert_eq!(
        ws.item(tool.sites[0]).expect("item").file.as_str(),
        "b/src/bin/tool.rs"
    );
    // Two targets of one package with one crate path stay an error.
    let one = [
        (
            "Cargo.toml",
            "[package]\nname = \"x-y\"\n\n[[bin]]\nname = \"t\"\npath = \"t1.rs\"\n\n[[bin]]\nname = \"t\"\npath = \"t2.rs\"\n",
        ),
        ("t1.rs", ""),
        ("t2.rs", ""),
    ];
    let error = load(&one, &["."]).expect_err("one package");
    assert_eq!(
        duplicate_path(&error),
        ("x_y::bin_crate::t", "t", "Cargo.toml")
    );
}

#[test]
fn a_first_party_module_named_like_the_test_namespace_loads_beside_a_test_target() {
    let ws = load(
        &[
            ("Cargo.toml", DEMO),
            ("src/lib.rs", "mod test_crate;"),
            ("src/test_crate.rs", ""),
            ("tests/it.rs", ""),
        ],
        &["."],
    )
    .expect("loads");
    let inner = ws
        .file(&RelPath::new("src/test_crate.rs").expect("path"))
        .expect("loaded");
    assert_eq!(inner.reached[0].module_path, "demo::test_crate");
    let root = ws
        .file(&RelPath::new("tests/it.rs").expect("path"))
        .expect("loaded");
    assert_eq!(root.reached[0].module_path, "demo::test_crate::it");
}

/// The test settings of each target: harness, required features, test and
/// doctest.
fn settings(ws: &Workspace) -> Vec<(String, TargetKind, bool, Vec<String>, bool, bool)> {
    ws.targets()
        .map(|t| {
            (
                t.name.clone(),
                t.kind,
                t.harness,
                t.required_features.clone(),
                t.test,
                t.doctest,
            )
        })
        .collect()
}

#[test]
fn harness_required_features_test_and_doctest_are_read_from_each_target_table() {
    let manifest = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\
                    [lib]\nharness = false\ntest = false\ndoctest = false\n\
                    [[bin]]\nname = \"x\"\nrequired-features = [\"cli\"]\ntest = false\n\
                    [[test]]\nname = \"it\"\nharness = false\ntest = false\n\
                    [[bench]]\nname = \"b\"\nharness = false\ntest = true\n\
                    [[example]]\nname = \"e\"\nrequired-features = [\"a\", \"b\"]\ndoctest = true\n";
    let ws = load(
        &[
            ("Cargo.toml", manifest),
            ("src/lib.rs", ""),
            ("src/main.rs", ""),
            ("src/bin/x.rs", ""),
            ("tests/it.rs", ""),
            ("tests/plain.rs", ""),
            ("benches/b.rs", ""),
            ("benches/plain.rs", ""),
            ("examples/e.rs", ""),
            ("examples/plain.rs", ""),
            ("build.rs", ""),
        ],
        &["."],
    )
    .expect("loads");
    let features = |names: &[&str]| names.iter().map(|&n| n.to_owned()).collect::<Vec<_>>();
    let row = |name: &str, kind, harness, required: &[&str], test, doctest| {
        (
            name.to_owned(),
            kind,
            harness,
            features(required),
            test,
            doctest,
        )
    };
    assert_eq!(
        settings(&ws),
        [
            // Each table sets keys away from cargo's defaults.
            row("demo", TargetKind::Lib, false, &[], false, false),
            row("x", TargetKind::Bin, true, &["cli"], false, false),
            // A target with no table takes cargo's defaults for its kind.
            row("demo", TargetKind::Bin, true, &[], true, false),
            row("it", TargetKind::Test, false, &[], false, false),
            row("plain", TargetKind::Test, true, &[], true, false),
            row("b", TargetKind::Bench, false, &[], true, false),
            row("plain", TargetKind::Bench, true, &[], false, false),
            row("e", TargetKind::Example, true, &["a", "b"], false, true),
            row("plain", TargetKind::Example, true, &[], false, false),
            row(
                "build-script-build",
                TargetKind::BuildScript,
                false,
                &[],
                false,
                false
            ),
        ]
    );
    // A package with no tables at all: the library is tested and doctested.
    let plain = load(&[("Cargo.toml", DEMO), ("src/lib.rs", "")], &["."]).expect("loads");
    assert_eq!(
        settings(&plain),
        [row("demo", TargetKind::Lib, true, &[], true, true)]
    );
}

#[test]
fn a_test_setting_of_the_wrong_type_is_a_manifest_error_naming_its_key() {
    for (table, key, value) in [
        ("lib", "harness", "\"no\""),
        ("lib", "doctest", "1"),
        ("bin", "test", "\"yes\""),
        ("test", "required-features", "\"cli\""),
        ("example", "required-features", "[1]"),
    ] {
        let header = if table == "lib" {
            "[lib]".to_owned()
        } else {
            format!("[[{table}]]\nname = \"t\"")
        };
        let manifest = format!("{DEMO}{header}\n{key} = {value}\n");
        let error =
            load(&[("Cargo.toml", &manifest), ("src/lib.rs", "")], &["."]).expect_err(&manifest);
        assert!(
            matches!(error.kind, LoadErrorKind::Manifest { .. }),
            "{manifest}: {error}"
        );
        assert!(
            error.to_string().contains(&format!("{table}.{key}")),
            "{manifest}: {error}"
        );
    }
}

#[test]
fn required_features_on_the_library_is_a_manifest_error_beside_a_binary_that_names_them() {
    let refused = format!("{DEMO}[lib]\nrequired-features = [\"cli\"]\n");
    let error = load(&[("Cargo.toml", &refused), ("src/lib.rs", "")], &["."])
        .expect_err("cargo refuses required-features on a library");
    assert!(
        matches!(error.kind, LoadErrorKind::Manifest { .. }),
        "{error}"
    );
    assert!(
        error.to_string().contains("lib.required-features"),
        "{error}"
    );
    let allowed = format!(
        "{DEMO}[[bin]]\nname = \"t\"\npath = \"src/main.rs\"\nrequired-features = [\"cli\"]\n"
    );
    let ws = load(
        &[
            ("Cargo.toml", &allowed),
            ("src/lib.rs", ""),
            ("src/main.rs", "fn main() {}"),
        ],
        &["."],
    )
    .expect("a binary may name required features");
    assert!(
        ws.targets()
            .any(|target| target.required_features == ["cli"]),
        "the binary keeps its required features"
    );
}
