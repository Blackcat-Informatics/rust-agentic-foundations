// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual declared contexts gate dependency edges and definition syntax together.
use raf_source::{
    ActiveConfiguration, Budget, CfgAtom, DeclaredCfg, DeclaredConfiguration, LoadError,
    LoadErrorKind, LoadOptions, LockedCatalog, LockedEdge, LockedManifest, MemoryTree, Namespace,
    PackageId, RelPath, ResolveError, TargetKind, Workspace,
};
const LINUX: &[CfgAtom<'_>] = &[CfgAtom::Name("unix"), CfgAtom::Value("target_os", "linux")];
const WINDOWS: &[CfgAtom<'_>] = &[
    CfgAtom::Name("windows"),
    CfgAtom::Value("target_os", "windows"),
];
fn context<'a>(features: &'a [&'a str], name: &'a str) -> DeclaredConfiguration<'a> {
    DeclaredConfiguration::new(
        DeclaredCfg::complete("linux-target", LINUX),
        features,
        TargetKind::Lib,
        name,
    )
}
fn fixture(manifest: &str, source: &str) -> MemoryTree {
    fixture_workspace(
        "[workspace]\nmembers=['consumer','provider']",
        manifest,
        source,
    )
}
fn fixture_workspace(workspace: &str, manifest: &str, source: &str) -> MemoryTree {
    let manifest = manifest.replacen("[package]\n", "[package]\nedition='2024'\n", 1);
    MemoryTree::from([
        ("Cargo.toml", workspace),
        ("consumer/Cargo.toml", manifest.as_str()),
        ("consumer/src/lib.rs", source),
        ("consumer/src/chosen.rs", "pub struct Chosen;"),
        ("consumer/src/platform.rs", "pub struct Windows;"),
        ("consumer/src/linux.rs", "pub struct Linux;"),
        (
            "consumer/build.rs",
            "pub use renamed::Native as Alias; fn main() {}",
        ),
        (
            "provider/Cargo.toml",
            "[package]\nname='provider'\nversion='1.0.0'\n[features]\ndefault=['api']\napi=[]\nextra=[]",
        ),
        (
            "provider/src/lib.rs",
            "#[cfg(feature=\"api\")] pub struct Native;\n#[cfg(not(feature=\"api\"))] pub struct Native;\n#[cfg(feature=\"api\")] pub struct OnlyApi;\n#[cfg_attr(feature=\"api\",cfg(not(feature=\"api\")))] pub struct AppliedInactive;\npub trait Trait { #[cfg(feature=\"api\")] fn on(); #[cfg(not(feature=\"api\"))] fn off(); }\nimpl Native { #[cfg(feature=\"api\")] pub fn on() {} #[cfg(not(feature=\"api\"))] pub fn off() {} }\n#[cfg(test)] pub struct TestsOnly;\n#[test] fn unit() {}",
        ),
    ])
}

#[test]
fn nested_cfg_attr_boolean_guards_escaped_values_and_test_builds_select_actual_declarations() {
    let source = "#[cfg(all(unix,any(target_os=\"linux\",not(unix))))] pub struct Active;\n#[cfg_attr(unix,cfg_attr(target_os=\"linux\",cfg(not(unix))))] pub struct Hidden;\n#[cfg(test)] pub struct TestsOnly;\n#[test] fn unit() {}\n#[cfg(custom=\"a\\\"b\")] pub struct Escaped;";
    let tree = fixture(MANIFEST, source);
    let atoms = [
        CfgAtom::Name("unix"),
        CfgAtom::Value("target_os", "linux"),
        CfgAtom::Value("custom", "a\"b"),
    ];
    let own = DeclaredConfiguration::new(
        DeclaredCfg::complete("linux-target", &atoms),
        &[],
        TargetKind::Lib,
        "consumer",
    );
    let ws = load(
        &tree,
        Some(own),
        Some(context(&[], "provider")),
        "",
        &LoadOptions::default(),
    )
    .expect("ordinary active syntax");
    assert!(ws.resolve("consumer::Active").is_ok());
    assert!(ws.resolve("consumer::Escaped").is_ok());
    assert!(ws.resolve("consumer::Hidden").is_err());
    assert!(ws.resolve("consumer::TestsOnly").is_err());
    assert!(ws.resolve("consumer::unit").is_err());
    let atoms = [
        CfgAtom::Name("unix"),
        CfgAtom::Name("test"),
        CfgAtom::Value("target_os", "linux"),
    ];
    let own = DeclaredConfiguration::new(
        DeclaredCfg::complete("linux-target", &atoms),
        &[],
        TargetKind::Lib,
        "consumer",
    );
    let ws = load(
        &tree,
        Some(own),
        Some(context(&[], "provider")),
        "",
        &LoadOptions::default(),
    )
    .expect("declared test build");
    ws.resolve_active_from_package_id_in(
        package(&ws, "consumer"),
        "consumer",
        "TestsOnly",
        Namespace::Type,
    )
    .expect("actual active test declaration");
    assert!(ws.resolve("consumer::unit").is_ok());
    for source in [
        "#[cfg(unknown_operator(unix))] pub struct Bad;",
        "#[cfg_attr(unix)] pub struct Bad;",
    ] {
        let tree = fixture(MANIFEST, source);
        assert!(
            load(
                &tree,
                Some(own),
                Some(context(&[], "provider")),
                "",
                &LoadOptions::default()
            )
            .is_err()
        );
    }
}

#[test]
fn forwarding_default_and_inherited_feature_requirements_are_validated_at_the_actual_destination() {
    let manifest = MANIFEST.replace("features=['api']", "features=[]").replace(
        "native=['dep:renamed']",
        "native=['dep:renamed','renamed/api']",
    );
    let tree = fixture(&manifest, "pub use renamed::Native;");
    assert!(
        load(
            &tree,
            Some(context(&["native"], "consumer")),
            Some(context(&[], "provider")),
            "dependencies.renamed",
            &LoadOptions::default()
        )
        .is_err()
    );
    load(
        &tree,
        Some(context(&["native"], "consumer")),
        Some(context(&["api"], "provider")),
        "dependencies.renamed",
        &LoadOptions::default(),
    )
    .expect("actual forwarded api");
    assert!(
        load(
            &tree,
            Some(context(&["native", "renamed"], "consumer")),
            Some(context(&["api"], "provider")),
            "dependencies.renamed",
            &LoadOptions::default()
        )
        .is_err()
    );
    let weak = manifest.replace(
        "native=['dep:renamed','renamed/api']",
        "native=['renamed?/api']",
    );
    let tree = fixture(&weak, "pub use renamed::Native;");
    assert!(
        load(
            &tree,
            Some(context(&["native"], "consumer")),
            Some(context(&["api"], "provider")),
            "dependencies.renamed",
            &LoadOptions::default()
        )
        .is_err()
    );
    load(
        &tree,
        Some(context(&["native", "renamed"], "consumer")),
        Some(context(&["api"], "provider")),
        "dependencies.renamed",
        &LoadOptions::default(),
    )
    .expect("weak forwarding after implicit activation");
    let defaults = MANIFEST.replace("default-features=false,", "");
    let tree = fixture(&defaults, "pub use renamed::Native;");
    assert!(
        load(
            &tree,
            Some(context(&["native"], "consumer")),
            Some(context(&["api"], "provider")),
            "dependencies.renamed",
            &LoadOptions::default()
        )
        .is_err()
    );
    load(
        &tree,
        Some(context(&["native"], "consumer")),
        Some(context(&["api", "default"], "provider")),
        "dependencies.renamed",
        &LoadOptions::default(),
    )
    .expect("actual destination defaults");
    let workspace = "[workspace]\nmembers=['consumer','provider']\n[workspace.dependencies]\nrenamed={package='provider',path='provider',version='1',default-features=false,features=['api']}";
    let manifest = "[package]\nname='consumer'\nversion='1.0.0'\n[dependencies]\nrenamed={workspace=true,optional=true,features=['extra']}\n[features]\nnative=['dep:renamed']";
    let tree = fixture_workspace(workspace, manifest, "pub use renamed::Native;");
    for selected in [&["api"][..], &["extra"][..]] {
        assert!(
            load(
                &tree,
                Some(context(&["native"], "consumer")),
                Some(context(selected, "provider")),
                "dependencies.renamed",
                &LoadOptions::default()
            )
            .is_err()
        );
    }
    load(
        &tree,
        Some(context(&["native"], "consumer")),
        Some(context(&["api", "extra"], "provider")),
        "dependencies.renamed",
        &LoadOptions::default(),
    )
    .expect("workspace and member feature union");
}
const MANIFEST: &str = "[package]\nname='consumer'\nversion='1.0.0'\n[dependencies]\nrenamed={package='provider',path='../provider',version='1',optional=true,default-features=false,features=['api']}\n[features]\ndefault=['native']\nnative=['dep:renamed']\n";
const SOURCE: &str = "pub mod reader {#[cfg(feature=\"native\")] pub use renamed::Native as Alias;}\n#[cfg(feature=\"native\")] pub mod chosen;\n#[cfg(not(feature=\"native\"))] mod never_opened;\n#[cfg_attr(target_os=\"linux\",path=\"linux.rs\")] pub mod platform;";
fn load(
    tree: &MemoryTree,
    consumer: Option<DeclaredConfiguration<'_>>,
    provider: Option<DeclaredConfiguration<'_>>,
    requested: &str,
    options: &LoadOptions,
) -> Result<Workspace, LoadError> {
    let catalog = LockedCatalog::load(
        &MemoryTree::from([(
            "Cargo.lock",
            "version=4\n[[package]]\nname='consumer'\nversion='1.0.0'\ndependencies=['provider']\n[[package]]\nname='provider'\nversion='1.0.0'\n",
        )]),
        &RelPath::new("Cargo.lock").expect("path"),
        &LoadOptions::default(),
    )?;
    let row = |name| {
        catalog
            .packages()
            .find(|row| row.name() == name)
            .expect("row")
            .id()
    };
    let edges = [LockedEdge::new(requested, None)];
    let requested = if requested.is_empty() {
        &[][..]
    } else {
        &edges[..]
    };
    let own = LockedManifest::new(
        row("consumer"),
        RelPath::new("consumer/Cargo.toml").expect("path"),
        requested,
    );
    let own = match consumer {
        Some(config) => own.with_configuration(config),
        None => own,
    };
    let native = LockedManifest::new(
        row("provider"),
        RelPath::new("provider/Cargo.toml").expect("path"),
        &[],
    );
    let native = match provider {
        Some(config) => native.with_configuration(config),
        None => native,
    };
    Workspace::load_locked_sources(&catalog, tree, &[own, native], &[], options)
}
fn package(ws: &Workspace, name: &str) -> PackageId {
    ws.package_ids()
        .find(|id| ws.package(*id).expect("package").name == name)
        .expect("package")
}
fn active<'a>(ws: &'a Workspace, name: &str) -> &'a ActiveConfiguration {
    ws.locked_binding(package(ws, name))
        .and_then(raf_source::LockedBinding::configuration)
        .expect("opaque active context")
}

#[test]
fn configured_aliases_resolve_only_active_real_definitions_and_paths() {
    exact_target_keeps_unobserved_packages_and_targets_out();
    let tree = fixture(MANIFEST, SOURCE);
    let ws = load(
        &tree,
        Some(context(&["native"], "consumer")),
        Some(context(&["api"], "provider")),
        "dependencies.renamed",
        &LoadOptions::default(),
    )
    .expect("active configured edge");
    let resolved = ws
        .resolve_active_from_package_id_in(
            package(&ws, "consumer"),
            "consumer::reader",
            "Alias",
            Namespace::Type,
        )
        .expect("actual definition");
    assert_eq!(resolved.path, "provider::Native");
    assert_eq!(resolved.sites.len(), 1);
    let item = ws.item(resolved.sites[0]).expect("site");
    assert_eq!(item.name_line, 1);
    assert_eq!(
        ws.target(item.target).expect("target").package_id,
        package(&ws, "provider")
    );
    assert_eq!(active(&ws, "consumer").features(), ["native"]);
    assert_eq!(active(&ws, "consumer").optional_dependencies(), ["renamed"]);
    assert_eq!(active(&ws, "provider").target().triple(), "linux-target");
    assert!(ws.resolve("consumer::platform::Linux").is_ok());
    assert!(ws.resolve("consumer::platform::Windows").is_err());
    assert!(ws.resolve("provider::AppliedInactive").is_err());
    assert!(ws.resolve("provider::Native::on").is_ok());
    assert!(ws.resolve("provider::Native::off").is_err());
    assert!(
        ws.resolve_active_from_package_id_in(
            package(&ws, "provider"),
            "provider",
            "absent",
            Namespace::Type
        )
        .is_err()
    );
}

fn exact_target_keeps_unobserved_packages_and_targets_out() {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='chosen'\nversion='1.0.0'\nedition.workspace=true\n[workspace]\nmembers=['unobserved']\n[workspace.package]\nedition='2024'\n[lib]\nname='named_api'\n[[bin]]\nname='unobserved-bin'\npath='src/not-read.rs'\n[[bin]]\nname='requires-shown'\npath='src/requires.rs'\nrequired-features=['shown']\n[features]\nshown=[]\n[dependencies]\nsibling={path='unobserved'}\n",
        ),
        (
            "src/lib.rs",
            "mod inner { #[cfg(feature=\"shown\")] pub struct Selected; #[cfg(not(feature=\"shown\"))] pub struct Other; #[doc(hidden)] pub struct Hidden; pub(crate) struct Private; } #[cfg(feature=\"shown\")] pub use inner::Selected as Alias; #[cfg(not(feature=\"shown\"))] pub use inner::Other as Alias; pub use inner::Hidden; #[cfg(doc)] pub struct DocumentationOnly;",
        ),
        (
            "src/not-read.rs",
            "this unselected target must not be parsed",
        ),
        (
            "src/requires.rs",
            "#[cfg(feature=\"shown\")] pub fn chosen() {} fn main() {}",
        ),
        (
            "unobserved/Cargo.toml",
            "[package]\nname='sibling'\nversion='1.0.0'\nedition='2024'\n",
        ),
        (
            "unobserved/src/lib.rs",
            "this unobserved sibling must not be parsed",
        ),
    ]);
    let manifest = RelPath::new("Cargo.toml").unwrap();
    let atoms = [CfgAtom::Name("unix"), CfgAtom::Name("doc")];
    for (features, expected) in [(&["shown"][..], "Selected"), (&[][..], "Other")] {
        let selection = DeclaredConfiguration::new(
            DeclaredCfg::complete("linux-target", &atoms),
            features,
            TargetKind::Lib,
            "named_api",
        );
        let ws =
            Workspace::load_configured_target(&tree, &manifest, selection, &LoadOptions::default())
                .expect("exact target retains inherited edition and declared selection");
        assert_eq!(ws.packages().len(), 1);
        let targets: Vec<_> = ws.targets().collect();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].name, "named_api");
        assert!(ws.files().all(|file| file.path.as_str() == "src/lib.rs"));
        let package = ws.package_ids().next().unwrap();
        assert!(ws.locked_binding(package).is_none());
        assert!(matches!(
            ws.resolve_active_from_package_id_in(package, "named_api", "Alias", Namespace::Type),
            Err(ResolveError::UnconfiguredPackage(_))
        ));
        let resolved = ws.resolve("named_api::Alias").unwrap();
        assert!(resolved.path.ends_with(expected));
        let target = ws.item(resolved.sites[0]).unwrap().target;
        let exports = ws.exported_items(target).unwrap();
        assert!(
            exports
                .items
                .iter()
                .any(|item| item.path == "named_api::Alias")
        );
        assert!(
            !exports
                .items
                .iter()
                .any(|item| item.path.ends_with("Hidden") || item.path.ends_with("Private"))
        );
        assert!(ws.resolve_from("named_api", "sibling::Anything").is_err());
        ws.code_resolver()
            .unwrap()
            .with_configured_syntax(target, &RelPath::new("src/lib.rs").unwrap(), |_, syntax| {
                assert!(syntax.items.iter().any(|item| matches!(item,
                    syn::Item::Struct(item) if item.ident == "DocumentationOnly")));
            })
            .expect("configured syntax reuses the exact target's original declaration");
    }
    exact_target_required_features(&tree, &manifest, &atoms);
    assert!(
        Workspace::load_configured_target(
            &tree,
            &manifest,
            context(&[], "missing"),
            &LoadOptions::default(),
        )
        .is_err()
    );
    assert!(
        Workspace::load_configured_target(
            &tree,
            &RelPath::new("not-a-manifest").unwrap(),
            context(&[], "named_api"),
            &LoadOptions::default(),
        )
        .is_err()
    );
}

fn exact_target_required_features(tree: &MemoryTree, manifest: &RelPath, atoms: &[CfgAtom<'_>]) {
    for features in [&[][..], &["shown"][..]] {
        let selection = DeclaredConfiguration::new(
            DeclaredCfg::complete("linux-target", atoms),
            features,
            TargetKind::Bin,
            "requires-shown",
        );
        let selected =
            Workspace::load_configured_target(tree, manifest, selection, &LoadOptions::default());
        if features.is_empty() {
            let error = selected.expect_err("required feature is inactive");
            assert!(
                error.to_string().contains("requires an inactive feature"),
                "{error}"
            );
        } else {
            let ws = selected.expect("enabled required feature admits the same target");
            let targets: Vec<_> = ws.targets().collect();
            assert_eq!(targets.len(), 1);
            assert_eq!(targets[0].name, "requires-shown");
            assert!(
                ws.files()
                    .all(|file| file.path.as_str() == "src/requires.rs")
            );
        }
    }
}

#[test]
fn distinct_declared_sets_change_definitions_and_keep_their_exact_receipt() {
    let tree = fixture(MANIFEST, "pub struct Local;");
    let on = load(
        &tree,
        Some(context(&[], "consumer")),
        Some(context(&["api"], "provider")),
        "",
        &LoadOptions::default(),
    )
    .expect("api context");
    let off = load(
        &tree,
        Some(context(&[], "consumer")),
        Some(context(&[], "provider")),
        "",
        &LoadOptions::default(),
    )
    .expect("no api context");
    let resolved = off
        .resolve_active_from_package_id_in(
            package(&off, "provider"),
            "provider",
            "Native",
            Namespace::Type,
        )
        .expect("other actual declaration");
    assert_eq!(off.item(resolved.sites[0]).expect("site").name_line, 2);
    assert!(on.resolve("provider::OnlyApi").is_ok());
    assert!(matches!(
        off.resolve_active_from_package_id_in(
            package(&off, "provider"),
            "provider",
            "OnlyApi",
            Namespace::Type
        ),
        Err(ResolveError::NotFound(_))
    ));
    assert_eq!(active(&off, "provider").features().len(), 0);
    assert_eq!(active(&on, "provider").features(), ["api"]);
    let structural = Workspace::load(&tree, &["provider"]).expect("structural inventory");
    assert!(matches!(
        structural.resolve_active_from_package_id_in(
            package(&structural, "provider"),
            "provider",
            "Native",
            Namespace::Type
        ),
        Err(ResolveError::UnconfiguredPackage(_))
    ));
}

#[test]
fn missing_unknown_inactive_or_incomplete_configuration_refuses_beside_valid_selection() {
    let tree = fixture(MANIFEST, "pub use renamed::Native;");
    for (own, provider) in [
        (Some(context(&["native"], "consumer")), None),
        (
            Some(context(&["missing"], "consumer")),
            Some(context(&["api"], "provider")),
        ),
        (
            Some(context(&["default"], "consumer")),
            Some(context(&["api"], "provider")),
        ),
        (
            Some(context(&[], "consumer")),
            Some(context(&["api"], "provider")),
        ),
        (
            Some(context(&["native"], "consumer")),
            Some(context(&[], "provider")),
        ),
        (
            Some(context(&["native"], "unknown_target")),
            Some(context(&["api"], "provider")),
        ),
    ] {
        assert!(
            load(
                &tree,
                own,
                provider,
                "dependencies.renamed",
                &LoadOptions::default()
            )
            .is_err()
        );
    }
    load(
        &tree,
        Some(context(&["native"], "consumer")),
        Some(context(&["api"], "provider")),
        "dependencies.renamed",
        &LoadOptions::default(),
    )
    .expect("valid complete feature evidence");
}

#[test]
fn target_and_host_conditions_use_the_selected_compilation_role() {
    let manifest = "[package]\nname='consumer'\nversion='1.0.0'\n[target.'cfg(unix)'.build-dependencies]\nrenamed={package='provider',path='../provider',version='1',default-features=false,features=['api']}";
    let tree = fixture(manifest, "pub struct Local;");
    let cross = DeclaredCfg::complete("windows-target", WINDOWS);
    let own = DeclaredConfiguration::new(cross, &[], TargetKind::BuildScript, "build-script-build");
    assert!(
        load(
            &tree,
            Some(own),
            Some(context(&["api"], "provider")),
            "target.cfg(unix).build-dependencies.renamed",
            &LoadOptions::default()
        )
        .is_err()
    );
    let own = own.with_host(DeclaredCfg::complete("linux-target", LINUX));
    let ws = load(
        &tree,
        Some(own),
        Some(context(&["api"], "provider")),
        "target.cfg(unix).build-dependencies.renamed",
        &LoadOptions::default(),
    )
    .expect("host eligible build edge");
    assert_eq!(active(&ws, "consumer").target().triple(), "windows-target");
    assert_eq!(
        active(&ws, "consumer").host().expect("host").triple(),
        "linux-target"
    );
    ws.resolve_active_from_package_id_in(
        package(&ws, "consumer"),
        "consumer::build_script",
        "Alias",
        Namespace::Type,
    )
    .expect("host eligible alias definition");
    for condition in [
        "cfg(feature=\"native\")",
        "cfg(r#feature=\"native\")",
        "cfg(test)",
        "cfg(r#test)",
        "cfg(debug_assertions)",
        "cfg(proc_macro)",
    ] {
        let manifest = manifest.replace("cfg(unix)", condition);
        let key = format!("target.{condition}.build-dependencies.renamed");
        let error = load(
            &fixture(&manifest, "pub struct Local;"),
            Some(own),
            Some(context(&["api"], "provider")),
            &key,
            &LoadOptions::default(),
        )
        .expect_err("Cargo does not activate dependency tables using these predicates");
        assert!(
            error
                .to_string()
                .contains("Cargo target dependency predicates"),
            "{error}"
        );
    }
    let manifest = manifest.replace(".build-dependencies]", ".dependencies]");
    let tree = fixture(&manifest, "pub use renamed::Native;");
    let own = DeclaredConfiguration::new(cross, &[], TargetKind::Lib, "consumer")
        .with_host(DeclaredCfg::complete("linux-target", LINUX));
    assert!(
        load(
            &tree,
            Some(own),
            Some(context(&["api"], "provider")),
            "target.cfg(unix).dependencies.renamed",
            &LoadOptions::default()
        )
        .is_err()
    );
}

#[test]
fn configured_syntax_and_feature_work_spend_the_original_combined_grant() {
    let tree = fixture(MANIFEST, SOURCE);
    let own = Some(context(&["native"], "consumer"));
    let native = Some(context(&["api"], "provider"));
    let baseline = load(
        &tree,
        own,
        native,
        "dependencies.renamed",
        &LoadOptions::default(),
    )
    .expect("baseline");
    let usage = baseline.usage();
    let mut options = LoadOptions::default();
    options.max_work = usage.work;
    options.max_item_bytes = usage.item_bytes;
    options.max_total_bytes = usage.total_bytes;
    load(&tree, own, native, "dependencies.renamed", &options)
        .expect("exact original combined grant");
    for budget in [Budget::Work, Budget::ItemBytes, Budget::TotalBytes] {
        let mut short = options.clone();
        match budget {
            Budget::Work => short.max_work -= 1,
            Budget::ItemBytes => short.max_item_bytes -= 1,
            Budget::TotalBytes => short.max_total_bytes -= 1,
            _ => unreachable!(),
        }
        let error =
            load(&tree, own, native, "dependencies.renamed", &short).expect_err("one short");
        assert!(
            matches!(error.kind,LoadErrorKind::Budget {budget:actual,..} if actual==budget),
            "{error}"
        );
    }
}

#[test]
fn cargo_feature_names_use_rust_unicode_xid_with_actual_cargo_punctuation() {
    let manifest = format!("{MANIFEST}\n\"δelta+api\"=[]\n\"0feature\"=[]\n");
    let tree = fixture(
        &manifest,
        "#[cfg(all(feature=\"δelta+api\",feature=\"0feature\"))] pub struct Selected;",
    );
    let ws = load(
        &tree,
        Some(context(&["δelta+api", "0feature"], "consumer")),
        Some(context(&[], "provider")),
        "",
        &LoadOptions::default(),
    )
    .expect("actual Cargo feature names");
    assert!(
        ws.resolve_active_from_package_id_in(
            package(&ws, "consumer"),
            "consumer",
            "Selected",
            Namespace::Type
        )
        .is_ok()
    );
    let bad = manifest.replace("\"δelta+api\"=[]", "\"bad@feature\"=[]");
    assert!(
        load(
            &fixture(&bad, "pub struct Local;"),
            Some(context(&[], "consumer")),
            Some(context(&[], "provider")),
            "",
            &LoadOptions::default()
        )
        .is_err()
    );
    let duplicate = [CfgAtom::Name("unix"), CfgAtom::Name("unix")];
    let duplicate = DeclaredConfiguration::new(
        DeclaredCfg::complete("linux-target", &duplicate),
        &[],
        TargetKind::Lib,
        "consumer",
    );
    assert!(
        load(
            &tree,
            Some(duplicate),
            Some(context(&[], "provider")),
            "",
            &LoadOptions::default()
        )
        .is_err()
    );
}

#[test]
fn a_physical_file_cannot_silently_merge_different_active_package_contexts() {
    let build = |root: &str| {
        let provider = format!(
            "[package]\nname='provider'\nversion='1.0.0'\nedition='2024'\n[lib]\npath={root:?}"
        );
        MemoryTree::from([
            ("Cargo.toml", "[workspace]\nmembers=['consumer','provider']"),
            (
                "consumer/Cargo.toml",
                "[package]\nname='consumer'\nversion='1.0.0'\nedition='2024'",
            ),
            ("provider/Cargo.toml", provider.as_str()),
            ("consumer/src/lib.rs", "pub struct Shared;"),
            ("provider/src/lib.rs", "pub struct Shared;"),
        ])
    };
    let other = DeclaredConfiguration::new(
        DeclaredCfg::complete("windows-target", WINDOWS),
        &[],
        TargetKind::Lib,
        "provider",
    );
    let error = load(
        &build("../consumer/src/lib.rs"),
        Some(context(&[], "consumer")),
        Some(other),
        "",
        &LoadOptions::default(),
    )
    .expect_err("distinct contexts cannot share one facts cache");
    assert!(
        error
            .to_string()
            .contains("different configured package contexts"),
        "{error}"
    );
    load(
        &build("src/lib.rs"),
        Some(context(&[], "consumer")),
        Some(other),
        "",
        &LoadOptions::default(),
    )
    .expect("equal bytes at distinct physical files remain separate");
}

#[test]
fn raw_cfg_identifiers_have_one_logical_identity_and_cannot_bypass_feature_separation() {
    let source = "#[r#cfg(r#custom)] pub struct RawFlag;\n#[cfg(r#custom_value=\"yes\")] pub struct RawValue;\n#[cfg(r#feature=\"native\")] pub struct RawFeature;\n#[cfg_attr(r#custom,r#path=\"linux.rs\")] pub mod platform;";
    let tree = fixture(MANIFEST, source);
    let normal = [
        CfgAtom::Name("custom"),
        CfgAtom::Value("custom_value", "yes"),
    ];
    let raw = [
        CfgAtom::Name("r#custom"),
        CfgAtom::Value("r#custom_value", "yes"),
    ];
    for atoms in [&normal[..], &raw[..]] {
        let own = DeclaredConfiguration::new(
            DeclaredCfg::complete("linux-target", atoms),
            &["native"],
            TargetKind::Lib,
            "consumer",
        );
        let ws = load(
            &tree,
            Some(own),
            Some(context(&[], "provider")),
            "",
            &LoadOptions::default(),
        )
        .expect("raw and ordinary names are one cfg identity");
        for name in ["RawFlag", "RawValue", "RawFeature", "platform::Linux"] {
            ws.resolve_active_from_package_id_in(
                package(&ws, "consumer"),
                "consumer",
                name,
                Namespace::Type,
            )
            .expect("actual active declaration");
        }
        assert_eq!(
            active(&ws, "consumer").target().predicates(),
            ["custom", "custom_value = \"yes\""]
        );
    }
    for atom in [
        CfgAtom::Name("r#feature"),
        CfgAtom::Value("r#feature", "native"),
    ] {
        let atoms = [atom];
        let own = DeclaredConfiguration::new(
            DeclaredCfg::complete("linux-target", &atoms),
            &[],
            TargetKind::Lib,
            "consumer",
        );
        let error = load(
            &tree,
            Some(own),
            Some(context(&[], "provider")),
            "",
            &LoadOptions::default(),
        )
        .expect_err("raw spelling cannot smuggle feature cfg into target predicates");
        assert!(
            error.to_string().contains("feature values belong"),
            "{error}"
        );
    }
    let atoms = [CfgAtom::Name("custom"), CfgAtom::Name("r#custom")];
    let own = DeclaredConfiguration::new(
        DeclaredCfg::complete("linux-target", &atoms),
        &[],
        TargetKind::Lib,
        "consumer",
    );
    let error = load(
        &tree,
        Some(own),
        Some(context(&[], "provider")),
        "",
        &LoadOptions::default(),
    )
    .expect_err("duplicate logical cfg identity");
    assert!(
        error
            .to_string()
            .contains("duplicate declared configuration"),
        "{error}"
    );
}

#[test]
fn configured_predicates_refuse_qualified_paths_beside_unqualified_names() {
    for predicate in [
        "::custom",
        "custom::flag",
        "::key=\"yes\"",
        "::all(custom)",
        "all(::custom)",
        "any(custom::flag)",
    ] {
        let source = format!("#[cfg({predicate})] pub struct Selected;");
        let error = load(
            &fixture(MANIFEST, &source),
            Some(context(&[], "consumer")),
            Some(context(&[], "provider")),
            "",
            &LoadOptions::default(),
        )
        .expect_err("qualified predicates cannot lose their path qualification");
        assert!(
            error.to_string().contains("unqualified identifiers"),
            "{error}"
        );
    }
    let atoms = [CfgAtom::Name("custom")];
    let own = DeclaredConfiguration::new(
        DeclaredCfg::complete("linux-target", &atoms),
        &[],
        TargetKind::Lib,
        "consumer",
    );
    let ws = load(
        &fixture(MANIFEST, "#[cfg(custom)] pub struct Selected;"),
        Some(own),
        Some(context(&[], "provider")),
        "",
        &LoadOptions::default(),
    )
    .expect("ordinary unqualified cfg remains valid");
    ws.resolve_active_from_package_id_in(
        package(&ws, "consumer"),
        "consumer",
        "Selected",
        Namespace::Type,
    )
    .expect("active unqualified neighbor");
}

#[test]
fn empty_cfg_attr_requires_its_comma_and_boolean_literals_are_not_raw_flags() {
    let source = "#[cfg_attr(true,)] pub struct EmptyTrue;\n#[cfg_attr(false,)] pub struct EmptyFalse;\n#[cfg(true)] pub struct LiteralTrue;\n#[cfg(false)] pub struct LiteralFalse;\n#[cfg(r#true)] pub struct RawTrue;\n#[cfg(r#false)] pub struct RawFalse;";
    for enabled in [false, true] {
        let atoms = [CfgAtom::Name("r#true"), CfgAtom::Name("r#false")];
        let own = DeclaredConfiguration::new(
            DeclaredCfg::complete("linux-target", if enabled { &atoms } else { &[] }),
            &[],
            TargetKind::Lib,
            "consumer",
        );
        let ws = load(
            &fixture(MANIFEST, source),
            Some(own),
            Some(context(&[], "provider")),
            "",
            &LoadOptions::default(),
        )
        .expect("empty applied attribute list with required comma is valid");
        for (name, present) in [
            ("EmptyTrue", true),
            ("EmptyFalse", true),
            ("LiteralTrue", true),
            ("LiteralFalse", false),
            ("RawTrue", enabled),
            ("RawFalse", enabled),
        ] {
            let result = ws.resolve_active_from_package_id_in(
                package(&ws, "consumer"),
                "consumer",
                name,
                Namespace::Type,
            );
            assert_eq!(result.is_ok(), present, "{name}: {result:?}");
        }
    }
    for attr in ["cfg_attr(true)", "cfg_attr()", "cfg_attr(,)"] {
        let source = format!("#[{attr}] pub struct Missing;");
        load(
            &fixture(MANIFEST, &source),
            Some(context(&[], "consumer")),
            Some(context(&[], "provider")),
            "",
            &LoadOptions::default(),
        )
        .expect_err("missing predicate or comma is refused");
    }
}
