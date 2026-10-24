// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Target-scoped locked contexts preserve actual dependency and syntax selection.
use raf_source::{
    Budget, CfgAtom, DeclaredCfg, DeclaredConfiguration, LoadError, LoadErrorKind, LoadOptions,
    LockedCatalog, LockedEdge, LockedManifest, MemoryTree, Namespace, RelPath, TargetKind,
    Workspace,
};

const ORDINARY: &[CfgAtom<'_>] = &[CfgAtom::Name("unix")];
const TEST: &[CfgAtom<'_>] = &[CfgAtom::Name("unix"), CfgAtom::Name("test")];
fn context(kind: TargetKind, name: &str, test: bool) -> DeclaredConfiguration<'_> {
    DeclaredConfiguration::new(
        DeclaredCfg::complete("declared-target", if test { TEST } else { ORDINARY }),
        &[],
        kind,
        name,
    )
}
fn tree(dependency: &str, shared: bool, required: bool) -> MemoryTree {
    tree_with_options(dependency, shared, required, "")
}
fn tree_with_options(dependency: &str, shared: bool, required: bool, options: &str) -> MemoryTree {
    let manifest = format!(
        "[package]\nname='consumer'\nversion='1.0.0'\nedition='2024'\n[lib]\nname='consumer_library'\n[[bin]]\nname='worker'\npath='src/main.rs'\n[[test]]\nname='integration'\npath='tests/integration.rs'\n[[bench]]\nname='measure'\npath='benches/measure.rs'\nharness=false\n[[example]]\nname='sample'\npath='examples/sample.rs'\n{}[features]\nextra=[]\n[{dependency}]\nrenamed={{package='provider',path='../provider',version='1'{options}}}\n",
        if required {
            "required-features=['extra']\n"
        } else {
            ""
        }
    );
    let source = if shared {
        "#[path=\"../shared.rs\"] mod shared; pub fn reader() {}"
    } else {
        "pub fn reader() {}"
    };
    let library = if shared {
        "#[path=\"../shared.rs\"] pub mod shared; pub struct Local; pub fn lib_reader() {}"
    } else {
        "pub struct Local; pub fn lib_reader() {}"
    };
    MemoryTree::from([
        ("Cargo.toml", "[workspace]\nmembers=['consumer','provider']"),
        ("consumer/Cargo.toml", manifest.as_str()),
        ("consumer/src/lib.rs", library),
        ("consumer/src/main.rs", source),
        (
            "consumer/shared.rs",
            "#[cfg(test)] pub struct OnlyTest; pub struct Shared;",
        ),
        ("consumer/tests/integration.rs", "pub fn reader() {}"),
        ("consumer/benches/measure.rs", "pub fn reader() {}"),
        ("consumer/examples/sample.rs", "pub fn reader() {}"),
        (
            "provider/Cargo.toml",
            "[package]\nname='provider'\nversion='1.0.0'\nedition='2024'",
        ),
        (
            "provider/src/lib.rs",
            "pub struct Native; #[cfg(test)] pub struct OnlyTest;",
        ),
        (
            "Cargo.lock",
            "version=4\n[[package]]\nname='consumer'\nversion='1.0.0'\ndependencies=['provider']\n[[package]]\nname='provider'\nversion='1.0.0'",
        ),
    ])
}
fn load(
    tree: &MemoryTree,
    configs: &[DeclaredConfiguration<'_>],
    dependency: &str,
) -> Result<Workspace, LoadError> {
    load_under(tree, configs, dependency, &LoadOptions::default())
}
fn load_under(
    tree: &MemoryTree,
    configs: &[DeclaredConfiguration<'_>],
    dependency: &str,
    options: &LoadOptions,
) -> Result<Workspace, LoadError> {
    let catalog = LockedCatalog::load(tree, &RelPath::new("Cargo.lock").unwrap(), options)?;
    let row = |name| {
        catalog
            .packages()
            .find(|row| row.name() == name)
            .unwrap()
            .id()
    };
    let edges = [LockedEdge::new(dependency, None)];
    let hints = [
        LockedManifest::new(
            row("consumer"),
            RelPath::new("consumer/Cargo.toml").unwrap(),
            &edges,
        )
        .with_configurations(configs),
        LockedManifest::new(
            row("provider"),
            RelPath::new("provider/Cargo.toml").unwrap(),
            &[],
        )
        .with_configuration(context(TargetKind::Lib, "provider", false)),
    ];
    Workspace::load_locked_sources(&catalog, tree, &hints, &[], options)
}
fn reader(ws: &Workspace, kind: TargetKind) -> raf_source::ItemId {
    ws.items()
        .find(|item| ws.target(item.target).unwrap().kind == kind && item.name == "reader")
        .unwrap()
        .id
}
fn query(
    ws: &Workspace,
    owner: raf_source::ItemId,
    path: &str,
) -> Result<raf_source::Resolution, raf_source::CodeResolveError> {
    let item = ws.item(owner).unwrap();
    ws.code_resolver().unwrap().resolve_from_item_at(
        owner,
        item.body.as_ref().unwrap().start_byte + 1,
        path,
        Namespace::Type,
    )
}

#[test]
fn all_selected_target_kinds_keep_their_native_and_own_library_routes() {
    for (kind, name, test, dependency) in [
        (TargetKind::Bin, "worker", false, "dependencies"),
        (TargetKind::Bin, "worker", true, "dev-dependencies"),
        (TargetKind::Test, "integration", true, "dev-dependencies"),
        (TargetKind::Bench, "measure", false, "dev-dependencies"),
        (TargetKind::Example, "sample", false, "dev-dependencies"),
    ] {
        let tree = tree(dependency, false, false);
        let configs = [
            context(TargetKind::Lib, "consumer_library", false),
            context(kind, name, test),
        ];
        let ws = load(&tree, &configs, &format!("{dependency}.renamed")).unwrap();
        let owner = reader(&ws, kind);
        assert_eq!(
            query(&ws, owner, "renamed::Native").unwrap().path,
            "provider::Native"
        );
        assert_eq!(
            query(&ws, owner, "consumer_library::Local").unwrap().path,
            "consumer_library::Local"
        );
        assert!(query(&ws, owner, "renamed::OnlyTest").is_err());
        let item = ws.item(owner).unwrap();
        let binding = ws
            .locked_binding(ws.target(item.target).unwrap().package_id)
            .unwrap();
        assert!(binding.configuration().is_none());
        assert_eq!(binding.configurations().len(), 2);
        assert!(binding.configuration_for(kind, name).is_some());
        ws.code_resolver()
            .unwrap()
            .with_configured_syntax(item.target, &item.file, |_, syntax| {
                assert!(!syntax.items.is_empty());
            })
            .unwrap();
        let root = ws
            .items()
            .find(|root| {
                root.target == item.target
                    && root.path == ws.target(item.target).unwrap().crate_path
            })
            .unwrap();
        assert_eq!(root.test != raf_source::TestRole::None, test);
        if dependency == "dev-dependencies" {
            let ordinary = ws.items().find(|item| item.name == "lib_reader").unwrap();
            assert!(query(&ws, ordinary.id, "renamed::Native").is_err());
        }
    }
}

#[test]
fn missing_duplicate_and_feature_ineligible_targets_refuse_beside_complete_selection() {
    let tree = tree("dependencies", false, true);
    let lib = context(TargetKind::Lib, "consumer_library", false);
    let sample = context(TargetKind::Example, "sample", false);
    for configs in [
        &[][..],
        &[sample][..],
        &[lib, lib][..],
        &[lib, sample][..],
        &[lib, context(TargetKind::Bin, "missing", false)][..],
    ] {
        assert!(load(&tree, configs, "dependencies.renamed").is_err());
    }
    let enabled = DeclaredConfiguration::new(
        DeclaredCfg::complete("declared-target", ORDINARY),
        &["extra"],
        TargetKind::Example,
        "sample",
    );
    let companion = DeclaredConfiguration::new(
        DeclaredCfg::complete("declared-target", ORDINARY),
        &["extra"],
        TargetKind::Lib,
        "consumer_library",
    );
    load(&tree, &[companion, enabled], "dependencies.renamed").unwrap();
    assert!(load(&tree, &[lib, enabled], "dependencies.renamed").is_err());
}

#[test]
fn shared_files_reuse_only_the_same_actual_syntax_selection() {
    let tree = tree("dependencies", true, false);
    let lib = context(TargetKind::Lib, "consumer_library", false);
    load(
        &tree,
        &[lib, context(TargetKind::Bin, "worker", false)],
        "dependencies.renamed",
    )
    .unwrap();
    let error = load(
        &tree,
        &[lib, context(TargetKind::Bin, "worker", true)],
        "dependencies.renamed",
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("physical source file"),
        "{error}"
    );
}

#[test]
fn development_edges_require_a_real_development_or_declared_test_context() {
    let tree = tree("dev-dependencies", false, false);
    let lib = context(TargetKind::Lib, "consumer_library", false);
    assert!(
        load(
            &tree,
            &[lib, context(TargetKind::Bin, "worker", false)],
            "dev-dependencies.renamed"
        )
        .is_err()
    );
    load(
        &tree,
        &[context(TargetKind::Lib, "consumer_library", true)],
        "dev-dependencies.renamed",
    )
    .unwrap();
}

#[test]
fn conditional_and_optional_edges_keep_each_targets_actual_activation() {
    let conditional = tree("target.'cfg(custom)'.dependencies", false, false);
    let lib = context(TargetKind::Lib, "consumer_library", false);
    let custom = [CfgAtom::Name("unix"), CfgAtom::Name("custom")];
    let worker = DeclaredConfiguration::new(
        DeclaredCfg::complete("declared-target", &custom),
        &[],
        TargetKind::Bin,
        "worker",
    );
    let key = "target.cfg(custom).dependencies.renamed";
    let ws = load(&conditional, &[lib, worker], key).unwrap();
    query(&ws, reader(&ws, TargetKind::Bin), "renamed::Native").unwrap();
    let owner = ws
        .items()
        .find(|item| item.name == "lib_reader")
        .unwrap()
        .id;
    assert!(query(&ws, owner, "renamed::Native").is_err());
    assert!(load(&conditional, &[lib], key).is_err());

    let optional = tree_with_options("dependencies", false, false, ",optional=true");
    let enabled = |kind, name| {
        DeclaredConfiguration::new(
            DeclaredCfg::complete("declared-target", ORDINARY),
            &["renamed"],
            kind,
            name,
        )
    };
    let ws = load(
        &optional,
        &[
            enabled(TargetKind::Lib, "consumer_library"),
            enabled(TargetKind::Bin, "worker"),
        ],
        "dependencies.renamed",
    )
    .unwrap();
    query(&ws, reader(&ws, TargetKind::Bin), "renamed::Native").unwrap();
    assert!(
        load(
            &optional,
            &[lib, context(TargetKind::Bin, "worker", false)],
            "dependencies.renamed"
        )
        .is_err()
    );
}

#[test]
fn retained_target_lists_and_configuration_queries_spend_the_original_grants() {
    let tree = tree("dev-dependencies", true, false);
    let configs = [
        context(TargetKind::Lib, "consumer_library", false),
        context(TargetKind::Test, "integration", true),
        context(TargetKind::Bench, "measure", false),
        context(TargetKind::Example, "sample", false),
        context(TargetKind::Bin, "worker", false),
    ];
    let baseline = load(&tree, &configs, "dev-dependencies.renamed").unwrap();
    let mut options = LoadOptions::default();
    options.max_work = baseline.usage().work;
    options.max_item_bytes = baseline.usage().item_bytes;
    load_under(&tree, &configs, "dev-dependencies.renamed", &options).unwrap();
    for budget in [Budget::Work, Budget::ItemBytes] {
        let mut short = options.clone();
        match budget {
            Budget::Work => short.max_work -= 1,
            Budget::ItemBytes => short.max_item_bytes -= 1,
            _ => unreachable!(),
        }
        let error = load_under(&tree, &configs, "dev-dependencies.renamed", &short).unwrap_err();
        assert!(
            matches!(error.kind, LoadErrorKind::Budget { budget: actual, .. } if actual == budget),
            "{error}"
        );
    }
}

#[test]
fn configured_syntax_queries_select_the_actual_target_and_preserve_budget_causes() {
    let tree = tree("dependencies", true, false);
    let file = RelPath::new("consumer/shared.rs").unwrap();
    for test in [false, true] {
        let configs = [context(TargetKind::Lib, "consumer_library", test)];
        let workspace = load(&tree, &configs, "dependencies.renamed").unwrap();
        let code = workspace.code_resolver().unwrap();
        let target = workspace
            .items()
            .find(|item| item.name == "lib_reader")
            .unwrap()
            .target;
        let found = code
            .with_configured_syntax(target, &file, |_, syntax| {
                syntax
                    .items
                    .iter()
                    .any(|item| matches!(item, syn::Item::Struct(item) if item.ident == "OnlyTest"))
            })
            .unwrap();
        assert_eq!(found, test);
        assert!(matches!(
            code.with_configured_syntax(
                target,
                &RelPath::new("provider/src/lib.rs").unwrap(),
                |_, _| ()
            ),
            Err(raf_source::CodeResolveError::SyntaxContext { .. })
        ));
        let mut options = LoadOptions::default();
        options.max_work = workspace.usage().work;
        let restricted = load_under(&tree, &configs, "dependencies.renamed", &options).unwrap();
        let error = restricted
            .code_resolver()
            .unwrap()
            .with_configured_syntax(target, &file, |_, _| ())
            .unwrap_err();
        assert!(
            matches!(error, raf_source::CodeResolveError::ConfiguredSyntax(ref cause) if matches!(cause.kind, LoadErrorKind::Budget { budget: Budget::Work, .. })),
            "{error}"
        );
    }
}
