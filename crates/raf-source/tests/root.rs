// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Workspace-root discovery needs manifests alone, including membership edges.

use raf_source::{MemoryTree, RelPath, discover_root};

const PACKAGE: &str = "[package]\nname='demo'\nversion='0.1.0'\n";

fn found(tree: &MemoryTree, start: &str) -> String {
    discover_root(tree, &RelPath::new(start).unwrap())
        .unwrap()
        .to_string()
}

#[test]
fn member_and_virtual_workspace_directories_resolve_without_sources() {
    let tree = MemoryTree::from([
        ("ws/Cargo.toml", "[workspace]\nmembers=['crates/*']\n"),
        ("ws/crates/demo/Cargo.toml", PACKAGE),
        ("ws/crates/demo/src/deep/note", ""),
    ]);
    for start in [
        "ws",
        "ws/crates",
        "ws/crates/demo",
        "ws/crates/demo/src/deep",
    ] {
        assert_eq!(found(&tree, start), "ws");
    }
}

#[test]
fn standalone_excluded_and_nested_packages_have_their_own_roots() {
    let tree = MemoryTree::from([
        ("Cargo.toml", "[workspace]\nexclude=['excluded']\n"),
        ("excluded/Cargo.toml", PACKAGE),
        ("nested/Cargo.toml", "[workspace]\n"),
        ("nested/notes/text", ""),
    ]);
    assert_eq!(found(&tree, "excluded"), "excluded");
    assert_eq!(found(&tree, "nested/notes"), "nested");
    let standalone = MemoryTree::from([("package/Cargo.toml", PACKAGE)]);
    assert_eq!(found(&standalone, "package"), "package");
    let unlisted = MemoryTree::from([
        ("Cargo.toml", "[workspace]"),
        ("package/Cargo.toml", PACKAGE),
    ]);
    let error = discover_root(&unlisted, &RelPath::new("package").unwrap()).unwrap_err();
    assert!(error.to_string().contains("not a member"), "{error}");
}

#[test]
fn transitive_path_dependencies_are_members() {
    let tree = MemoryTree::from([
        ("Cargo.toml", "[workspace]\nmembers=['a']"),
        (
            "a/Cargo.toml",
            "[package]\nname='a'\n[dependencies]\nb={path='../b'}",
        ),
        ("b/Cargo.toml", PACKAGE),
    ]);
    assert_eq!(found(&tree, "b"), ".");
}

#[test]
fn explicit_workspace_requires_a_safe_path_and_agreed_membership() {
    let package = "[package]\nname='demo'\nworkspace='..'";
    let tree = MemoryTree::from([
        ("Cargo.toml", "[workspace]\nmembers=['package']"),
        ("package/Cargo.toml", package),
    ]);
    assert_eq!(found(&tree, "package"), ".");
    let sibling = MemoryTree::from([
        (
            "workspace/Cargo.toml",
            "[workspace]\nmembers=['../package']",
        ),
        (
            "package/Cargo.toml",
            "[package]\nname='demo'\nworkspace='../workspace'",
        ),
    ]);
    assert_eq!(found(&sibling, "package"), "workspace");
    let unlisted = MemoryTree::from([
        ("workspace/Cargo.toml", "[workspace]"),
        (
            "package/Cargo.toml",
            "[package]\nname='demo'\nworkspace='../workspace'",
        ),
    ]);
    assert!(discover_root(&unlisted, &RelPath::new("package").unwrap()).is_err());
    for workspace in ["[workspace]", "[package]\nname='root'"] {
        let tree = MemoryTree::from([("Cargo.toml", workspace), ("package/Cargo.toml", package)]);
        assert!(discover_root(&tree, &RelPath::new("package").unwrap()).is_err());
    }
    for text in [
        "[package]\nname='demo'\nworkspace='../..'",
        "[package]\nname='demo'\nworkspace='..'\n[workspace]",
    ] {
        let tree = MemoryTree::from([("Cargo.toml", "[workspace]"), ("package/Cargo.toml", text)]);
        assert!(discover_root(&tree, &RelPath::new("package").unwrap()).is_err());
    }
}

#[test]
fn malformed_manifests_and_missing_directories_fail_hard() {
    for text in [
        "[workspace",
        "workspace=1",
        "[workspace]\nmembers=1",
        "[package]\nname='demo'\nworkspace=1",
    ] {
        let tree = MemoryTree::from([("Cargo.toml", text)]);
        assert!(discover_root(&tree, &RelPath::root()).is_err(), "{text}");
    }
    let tree = MemoryTree::from([("notes", "")]);
    assert!(discover_root(&tree, &RelPath::root()).is_err());
    assert!(discover_root(&tree, &RelPath::new("missing").unwrap()).is_err());
    assert!(discover_root(&tree, &RelPath::new("notes").unwrap()).is_err());
}

#[test]
fn manifest_size_bound_is_enforced_beside_the_exact_limit() {
    let limit = usize::try_from(raf_source::DEFAULT_MAX_FILE_BYTES).unwrap();
    let mut text = String::from("[workspace]\n#");
    text.extend(std::iter::repeat_n('x', limit - text.len()));
    let exact = MemoryTree::from([("Cargo.toml", text.as_str())]);
    assert_eq!(found(&exact, "."), ".");
    text.push('x');
    let oversized = MemoryTree::from([("Cargo.toml", text.as_str())]);
    let error = discover_root(&oversized, &RelPath::root()).unwrap_err();
    assert!(
        matches!(&error.kind, raf_source::LoadErrorKind::Tree(tree) if matches!(tree.as_ref(), raf_source::TreeError::TooLarge { .. })),
        "{error}"
    );
}
