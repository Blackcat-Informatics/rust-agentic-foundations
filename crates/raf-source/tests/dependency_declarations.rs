// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Manifest observations retain constraints without inventing active lock edges.

use raf_source::{Budget, DependencyKind, LoadErrorKind, LoadOptions, MemoryTree, Workspace};

fn tree(dependencies: &str) -> MemoryTree {
    let manifest = format!("[package]\nname='demo'\nversion='0.1.0'\n{dependencies}");
    MemoryTree::from([
        ("Cargo.toml", manifest.as_str()),
        ("src/lib.rs", "pub fn local() {}"),
    ])
}

#[test]
fn declarations_retain_sources_flags_aliases_and_unevaluated_conditions() {
    let tree = tree(
        "[dependencies]\nplain='>=1.2, <2'\n\
         renamed={ package='actual', version='^2', registry='private', optional=true, default-features=false, features=['a','a','b'] }\n\
         local={path='other',version='1'}\n\
         indexed={version='1',registry-index='https://example.invalid/index'}\n\
         [dev-dependencies]\nrevision={git='https://example.invalid/repo',rev='abc123',version='3'}\n\
         [build-dependencies]\nbranch={git='https://example.invalid/repo',branch='development'}\n\
         [target.'cfg(unix)'.dependencies]\ntagged={git='https://example.invalid/repo',tag='v1',optional=false}\n\
         [target.x86_64-unknown-linux-gnu.dev-dependencies]\nconditional='4'",
    );
    let ws = Workspace::load(&tree, &["."]).expect("valid declarations");
    let package = ws.package_ids().next().expect("package");
    let declarations = ws.dependency_declarations(package).expect("declarations");
    assert_eq!(declarations.len(), 8);
    let named = |name| {
        declarations
            .iter()
            .find(|entry| entry.name == name)
            .expect("entry")
    };
    assert_eq!(
        named("plain").specification.version.as_deref(),
        Some(">=1.2, <2")
    );
    let alias = &named("renamed").specification;
    assert_eq!(alias.package.as_deref(), Some("actual"));
    assert_eq!(alias.registry.as_deref(), Some("private"));
    assert_eq!(alias.optional, Some(true));
    assert_eq!(alias.default_features, Some(false));
    assert_eq!(alias.features, ["a", "a", "b"]);
    assert_eq!(named("plain").specification.default_features, None);
    assert_eq!(named("local").specification.path.as_deref(), Some("other"));
    assert_eq!(
        named("indexed").specification.registry_index.as_deref(),
        Some("https://example.invalid/index")
    );
    assert_eq!(
        named("revision").specification.rev.as_deref(),
        Some("abc123")
    );
    assert_eq!(named("revision").kind, DependencyKind::Dev);
    assert_eq!(
        named("branch").specification.branch.as_deref(),
        Some("development")
    );
    assert_eq!(named("branch").kind, DependencyKind::Build);
    assert_eq!(named("tagged").specification.tag.as_deref(), Some("v1"));
    assert_eq!(named("tagged").condition.as_deref(), Some("cfg(unix)"));
    assert_eq!(named("tagged").specification.optional, Some(false));
    assert_eq!(
        named("conditional").condition.as_deref(),
        Some("x86_64-unknown-linux-gnu")
    );
    assert!(ws.resolve("demo::local").is_ok());
    assert!(ws.resolve("renamed::anything").is_err());
}

#[test]
fn inheritance_preserves_both_entries_and_the_workspace_path_base() {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[workspace]\nmembers=['member']\n[workspace.dependencies]\nshared={package='actual',path='vendor',version='2',default-features=false,features=['base']}\n",
        ),
        (
            "member/Cargo.toml",
            "[package]\nname='member'\nversion='0.1.0'\n[dependencies]\nshared={workspace=true,optional=true,features=['extra']}\n",
        ),
        ("member/src/lib.rs", "pub fn local() {}"),
        (
            "vendor/Cargo.toml",
            "[package]\nname='actual'\nversion='2.0.0'\n",
        ),
        ("vendor/src/lib.rs", "pub fn supplied() {}"),
    ]);
    let ws = Workspace::load(&tree, &["."]).expect("workspace");
    let id = ws.package_ids().next().expect("member");
    let entry = &ws.dependency_declarations(id).expect("declarations")[0];
    assert!(entry.specification.workspace);
    assert_eq!(entry.specification.optional, Some(true));
    assert_eq!(entry.specification.features, ["extra"]);
    let inherited = entry.inherited.as_ref().expect("inherited entry");
    assert_eq!(inherited.manifest.as_str(), "Cargo.toml");
    assert_eq!(inherited.specification.path.as_deref(), Some("vendor"));
    assert_eq!(inherited.specification.version.as_deref(), Some("2"));
    assert_eq!(inherited.specification.package.as_deref(), Some("actual"));
    assert_eq!(inherited.specification.default_features, Some(false));
    assert_eq!(inherited.specification.features, ["base"]);
    assert!(ws.resolve("member::local").is_ok());
}

#[test]
fn malformed_declarations_refuse_with_valid_neighbors() {
    for declaration in [
        "42",
        "{version=42}",
        "'not a version'",
        "'01.2'",
        "'1.2.3-01'",
        "'1.*.3'",
        "'1.2-pre'",
        "'1.2.*+build'",
        "'1 || 2'",
        "'1, *'",
        "'1.2.3+'",
        "'18446744073709551616'",
        "{path=42}",
        "{optional='yes'}",
        "{default-features=1}",
        "{features='a'}",
        "{features=[1]}",
        "{git=42}",
        "{registry=false}",
        "{rev='abc'}",
        "{git='https://example.invalid/repo',branch='a',tag='b'}",
        "{git='https://example.invalid/repo',path='x'}",
        "{workspace=false}",
        "{workspace=true,version='1'}",
    ] {
        let invalid = tree(&format!("[dependencies]\nbad={declaration}"));
        let error = Workspace::load(&invalid, &["."]).expect_err(declaration);
        assert!(
            matches!(error.kind, LoadErrorKind::Manifest { .. }),
            "{error}"
        );
        assert!(error.to_string().contains("dependencies.bad"), "{error}");
    }
    let valid =
        tree("[dependencies]\ngood={version='1',optional=false,default-features=true,features=[]}");
    assert!(Workspace::load(&valid, &["."]).is_ok());
    for requirement in [
        "  >= 1.2.3-alpha.11+001 , <2 ",
        "1.*.*",
        "^0.0",
        "~18446744073709551615.0.0",
        "*",
    ] {
        Workspace::load(
            &tree(&format!("[dependencies]\ngood={requirement:?}")),
            &["."],
        )
        .expect("complete Cargo requirement neighbor");
    }
    for count in [32, 33] {
        let requirement = vec![">=0"; count].join(", ");
        let input = tree(&format!("[dependencies]\nedge={requirement:?}"));
        let result = Workspace::load(&input, &["."]);
        if count == 32 {
            result.expect("exact comparator ceiling");
        } else {
            let error = result.expect_err("one comparator beyond the ceiling");
            assert!(
                error.to_string().contains("dependencies.edge.version"),
                "{error}"
            );
        }
    }
    for declaration in ["false", "{optional=true}", "{workspace=true}"] {
        let invalid = tree(&format!(
            "[workspace]\n[workspace.dependencies]\nbad={declaration}"
        ));
        let error = Workspace::load(&invalid, &["."]).expect_err(declaration);
        assert!(matches!(error.kind, LoadErrorKind::Manifest { .. }));
        assert!(
            error.to_string().contains("workspace.dependencies.bad"),
            "{error}"
        );
    }
}

#[test]
fn retained_declarations_use_the_original_item_allowance() {
    let tree = tree("[dependencies]\nexternal={version='1',features=['one','two']}");
    let ws = Workspace::load(&tree, &["."]).expect("default load");
    let mut options = LoadOptions::default();
    options.max_item_bytes = ws.usage().item_bytes;
    assert!(Workspace::load_with(&tree, &["."], &options).is_ok());
    options.max_item_bytes -= 1;
    let error = Workspace::load_with(&tree, &["."], &options).expect_err("one byte short");
    assert!(matches!(
        error.kind,
        LoadErrorKind::Budget {
            budget: Budget::ItemBytes,
            ..
        }
    ));
}
