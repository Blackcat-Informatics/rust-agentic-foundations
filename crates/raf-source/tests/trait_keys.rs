// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! How the trait of an impl on a type that is not local is keyed.

use raf_source::{LoadError, LoadErrorKind, MemoryTree, Namespace, TraitKey, Workspace};

fn manifest(edition: &str, extra: &str) -> String {
    format!("[package]\nname = \"demo\"\nedition = \"{edition}\"\n{extra}")
}

fn load(manifest: &str, files: &[(&str, &str)]) -> Result<Workspace, LoadError> {
    let mut all = vec![("Cargo.toml", manifest)];
    all.extend_from_slice(files);
    let tree: MemoryTree = all.into_iter().collect();
    Workspace::load(&tree, &["."])
}

fn lib(edition: &str, text: &str) -> Result<Workspace, LoadError> {
    load(&manifest(edition, ""), &[("src/lib.rs", text)])
}

fn key_of(ws: &Workspace, path: &str) -> TraitKey {
    let resolution = ws
        .resolve_in(path, Namespace::Value)
        .unwrap_or_else(|e| panic!("{path}: {e}"));
    ws.item(resolution.sites[0])
        .expect("item")
        .trait_key
        .as_deref()
        .cloned()
        .expect("a trait impl on a foreign type has a key")
}

#[test]
fn each_prelude_trait_loads_in_the_edition_that_has_it_and_is_refused_in_the_one_before() {
    // (name, first edition, canonical path)
    let cases = [
        ("TryFrom<L>", 2021, "std::convert::TryFrom"),
        ("TryInto<L>", 2021, "std::convert::TryInto"),
        ("FromIterator<L>", 2021, "std::iter::FromIterator"),
        ("Future", 2024, "std::future::Future"),
        ("IntoFuture", 2024, "std::future::IntoFuture"),
        ("Clone", 2015, "std::clone::Clone"),
        ("Iterator", 2015, "std::iter::Iterator"),
        ("Unpin", 2015, "std::marker::Unpin"),
    ];
    for (written, first, canonical) in cases {
        let name = written.split('<').next().expect("name");
        let text = format!("pub struct L;\nimpl {written} for u8 {{ fn m(&self) {{}} }}\n");
        for edition in [2015, 2018, 2021, 2024] {
            let result = lib(&edition.to_string(), &text);
            if edition >= first {
                let ws = result.unwrap_or_else(|e| panic!("{name} in {edition}: {e}"));
                assert_eq!(
                    key_of(&ws, &format!("{canonical}::m")),
                    TraitKey::Canonical(canonical.to_owned()),
                    "{name} in {edition}"
                );
            } else {
                let error = result.expect_err(name);
                assert!(
                    matches!(error.kind, LoadErrorKind::UnresolvedTrait { .. }),
                    "{name} in {edition}: {error}"
                );
                assert_eq!(error.line, Some(2));
            }
        }
    }
}

#[test]
fn an_inherited_edition_comes_from_the_workspace_and_a_missing_one_is_a_manifest_error() {
    let root = "[workspace]\nmembers = [\"member\"]\n\n[workspace.package]\nedition = \"2021\"\n";
    let member = "[package]\nname = \"member\"\nedition.workspace = true\n";
    let text = "pub struct L;\nimpl TryFrom<L> for u8 { fn m(&self) {} }\n";
    let tree: MemoryTree = [
        ("Cargo.toml", root),
        ("member/Cargo.toml", member),
        ("member/src/lib.rs", text),
    ]
    .into_iter()
    .collect();
    // 2021 has `TryFrom`, and not `Future`.
    assert!(Workspace::load(&tree, &["."]).is_ok());
    let with_future = format!("{text}impl Future for u8 {{ fn n(&self) {{}} }}\n");
    let tree: MemoryTree = [
        ("Cargo.toml", root),
        ("member/Cargo.toml", member),
        ("member/src/lib.rs", with_future.as_str()),
    ]
    .into_iter()
    .collect();
    let error = Workspace::load(&tree, &["."]).expect_err("Future is 2024");
    assert!(
        matches!(error.kind, LoadErrorKind::UnresolvedTrait { .. }),
        "{error}"
    );
    // A workspace manifest with no edition leaves nothing to inherit.
    let bare = "[workspace]\nmembers = [\"member\"]\n";
    let tree: MemoryTree = [
        ("Cargo.toml", bare),
        ("member/Cargo.toml", member),
        ("member/src/lib.rs", text),
    ]
    .into_iter()
    .collect();
    let error = Workspace::load(&tree, &["."]).expect_err("nothing to inherit");
    assert!(
        matches!(error.kind, LoadErrorKind::Manifest { .. }),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "member/Cargo.toml");
    assert!(
        matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key }) if key == "package.edition"),
        "{error}"
    );
    // The member alone, with its workspace manifest outside the roots.
    let tree: MemoryTree = [("member/Cargo.toml", member), ("member/src/lib.rs", text)]
        .into_iter()
        .collect();
    assert!(Workspace::load(&tree, &["member"]).is_err());
}

#[test]
fn a_trait_through_one_dependency_glob_is_keyed_under_that_glob() {
    let ws = lib(
        "2021",
        "use std::fmt::*;\npub struct L;\nimpl Display for Vec<L> { fn fmt(&self) {} }\n",
    )
    .expect("loads");
    assert_eq!(
        key_of(&ws, "std::fmt::Display::fmt"),
        TraitKey::Canonical("std::fmt::Display".to_owned())
    );
    // Through `use super::*` where the parent imported the trait.
    let ws = lib(
        "2021",
        "use std::fmt::Display;\npub struct L;\nmod t { use super::*; impl Display for Vec<L> { fn fmt(&self) {} } }\n",
    )
    .expect("loads");
    assert_eq!(
        key_of(&ws, "std::fmt::Display::fmt"),
        TraitKey::Canonical("std::fmt::Display".to_owned())
    );
}

#[test]
fn several_dependency_globs_leave_the_trait_unresolved_and_the_members_are_still_found() {
    let ws = lib(
        "2021",
        "use std::fmt::*;\nuse std::io::*;\npub struct L;\nimpl Write for Vec<L> { fn m(&self) {} }\n",
    )
    .expect("compiling code loads");
    let key = key_of(&ws, "Write::m");
    assert_eq!(
        key,
        TraitKey::Unresolved {
            written: "Write".to_owned(),
            candidates: vec!["std::fmt::Write".to_owned(), "std::io::Write".to_owned()],
        }
    );
    // A prelude name beside a dependency glob could be either.
    let ws = lib(
        "2021",
        "use std::fmt::*;\npub struct L;\nimpl Clone for Vec<L> { fn clone(&self) {} }\n",
    )
    .expect("loads");
    match key_of(&ws, "Clone::clone") {
        TraitKey::Unresolved { candidates, .. } => {
            assert_eq!(candidates, ["std::clone::Clone", "std::fmt::Clone"]);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_head_that_is_not_a_known_crate_is_an_error_and_a_declared_dependency_loads() {
    let error = lib(
        "2021",
        "pub struct L;\n\nimpl bogus::Tr for Vec<L> { fn m(&self) {} }\n",
    )
    .expect_err("bogus is no crate");
    assert!(
        matches!(error.kind, LoadErrorKind::UnresolvedTrait { .. }),
        "{error}"
    );
    assert_eq!(error.line, Some(3));
    let with_dep = manifest(
        "2021",
        "\n[dependencies]\nbogus = \"1\"\nsome-crate = \"1\"\n",
    );
    let text = "pub struct L;\nimpl bogus::Tr for Vec<L> { fn m(&self) {} }\nimpl some_crate::Tr for u8 { fn n(&self) {} }\n";
    let ws = load(&with_dep, &[("src/lib.rs", text)]).expect("declared");
    assert_eq!(
        key_of(&ws, "bogus::Tr::m"),
        TraitKey::Canonical("bogus::Tr".to_owned())
    );
    assert_eq!(
        key_of(&ws, "some_crate::Tr::n"),
        TraitKey::Canonical("some_crate::Tr".to_owned())
    );
}

#[test]
fn dependency_tables_are_honoured_by_target_kind_and_renames_use_the_name_in_code() {
    let extra = "\n[dependencies]\nnormal = \"1\"\nreal-name = { package = \"original\", version = \"1\" }\n\n\
                 [dev-dependencies]\nhelper = \"1\"\n\n[build-dependencies]\nbuilder = \"1\"\n\n\
                 [target.'cfg(unix)'.dependencies]\nunixy = \"1\"\n";
    let m = manifest("2021", extra);
    let case =
        |head: &str| format!("pub struct L;\nimpl {head}::Tr for Vec<L> {{ fn m(&self) {{}} }}\n");
    // Library: dependencies (also the target-specific ones, and the rename as written).
    for head in ["normal", "real_name", "unixy"] {
        assert!(load(&m, &[("src/lib.rs", &case(head))]).is_ok(), "{head}");
    }
    for head in ["helper", "builder", "original"] {
        assert!(load(&m, &[("src/lib.rs", &case(head))]).is_err(), "{head}");
    }
    // A dev-dependency is nameable from test code of the library and from a test target.
    let in_tests = format!("#[cfg(test)]\nmod t {{ {} }}\n", case("helper"));
    assert!(load(&m, &[("src/lib.rs", &in_tests)]).is_ok());
    assert!(load(&m, &[("src/lib.rs", ""), ("tests/it.rs", &case("helper"))]).is_ok());
    assert!(load(&m, &[("src/lib.rs", ""), ("tests/it.rs", &case("builder"))]).is_err());
    // A build script names build-dependencies only.
    assert!(load(&m, &[("src/lib.rs", ""), ("build.rs", &case("builder"))]).is_ok());
    assert!(load(&m, &[("src/lib.rs", ""), ("build.rs", &case("normal"))]).is_err());
}

#[test]
fn a_path_through_a_local_re_export_of_a_dependency_has_one_key_with_the_direct_spelling() {
    let m = manifest("2021", "\n[dependencies]\nserde = \"1\"\n");
    let text = "pub struct L;\n\
        pub mod traits { pub use serde::Serialize; }\n\
        impl traits::Serialize for Vec<L> { fn m(&self) {} }\n\
        impl serde::Serialize for Option<L> { fn m(&self) {} }\n";
    let ws = load(&m, &[("src/lib.rs", text)]).expect("loads");
    let resolution = ws.resolve("serde::Serialize::m").expect("one key");
    assert_eq!(resolution.sites.len(), 2);
    assert_eq!(
        key_of(&ws, "serde::Serialize::m"),
        TraitKey::Canonical("serde::Serialize".to_owned())
    );
    assert!(ws.resolve("demo::traits::Serialize::m").is_err());
}

#[test]
fn local_traits_still_key_under_their_definition_beside_dependency_ones() {
    let ws = lib(
        "2021",
        "pub trait Mine { fn run(&self); }\npub mod m { pub use super::Mine as Alias; }\npub struct L;\n\
         impl m::Alias for Vec<L> { fn run(&self) {} }\n",
    )
    .expect("loads");
    let resolution = ws.resolve("demo::Mine::run").expect("resolves");
    // The trait's own declaration wins the path; the impl is an item.
    assert_eq!(resolution.sites.len(), 1);
    assert!(
        ws.items()
            .any(|item| item.self_type.as_deref() == Some("Vec<L>"))
    );
}

#[test]
fn an_edition_that_is_not_a_year_is_a_manifest_error_naming_the_key() {
    let error = lib("next", "pub fn f() {}\n").expect_err("not a year");
    assert!(
        matches!(error.kind, LoadErrorKind::Manifest { .. }),
        "{error}"
    );
    assert!(
        matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key }) if key == "package.edition"),
        "{error}"
    );
    assert!(lib("2018", "pub fn f() {}\n").is_ok());
}

#[test]
fn a_glob_of_something_that_is_no_crate_or_module_is_an_error_beside_a_declared_one() {
    let text = "use bogus::*;\npub struct L;\nimpl Thing for Vec<L> { fn m(&self) {} }\n";
    let error = lib("2021", text).expect_err("the glob names no crate");
    assert!(
        matches!(error.kind, LoadErrorKind::UnresolvedTrait { .. }),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "src/lib.rs");
    assert_eq!(error.line, Some(1));
    // A glob that is never consulted for a trait key does no harm.
    assert!(lib("2021", "use bogus::*;\npub fn f() {}\n").is_ok());
    // The same glob of a declared dependency loads and keys under it.
    let with_dep = manifest("2021", "\n[dependencies]\nbogus = \"1\"\n");
    let ws = load(&with_dep, &[("src/lib.rs", text)]).expect("declared");
    assert_eq!(
        key_of(&ws, "bogus::Thing::m"),
        TraitKey::Canonical("bogus::Thing".to_owned())
    );
}

#[test]
fn the_same_trait_through_a_glob_and_through_a_named_import_can_have_two_keys() {
    let m = manifest("2021", "\n[dependencies]\nserde = \"1\"\n");
    let glob = "use serde::*;\npub struct L;\nimpl Serialize for Vec<L> { fn m(&self) {} }\n";
    let named =
        "use serde::ser::Serialize;\npub struct L;\nimpl Serialize for Vec<L> { fn n(&self) {} }\n";
    let by_glob = load(&m, &[("src/lib.rs", glob)]).expect("loads");
    let by_name = load(&m, &[("src/lib.rs", named)]).expect("loads");
    assert_eq!(
        key_of(&by_glob, "serde::Serialize::m"),
        TraitKey::Canonical("serde::Serialize".to_owned())
    );
    assert_eq!(
        key_of(&by_name, "serde::ser::Serialize::n"),
        TraitKey::Canonical("serde::ser::Serialize".to_owned())
    );
}

/// A tree with a workspace manifest at the root and one member under `crates/`.
fn inherited(root: &str, extra: &[(&str, &str)]) -> MemoryTree {
    let member = "[package]\nname = \"member\"\nedition.workspace = true\n";
    let text = "pub struct L;\nimpl TryFrom<L> for u8 { fn m(&self) {} }\n";
    let mut files = vec![
        ("Cargo.toml", root),
        ("crates/member/Cargo.toml", member),
        ("crates/member/src/lib.rs", text),
    ];
    files.extend_from_slice(extra);
    files.into_iter().collect()
}

const WORKSPACE_2021: &str = "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"3\"\n\n[workspace.package]\nedition = \"2021\"\n";

#[test]
fn the_workspace_manifest_is_found_above_the_roots_and_adds_no_package() {
    let ws = Workspace::load(&inherited(WORKSPACE_2021, &[]), &["crates"])
        .expect("the edition is inherited");
    let names: Vec<_> = ws.packages().iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["member"]);
    // The inherited edition decides the prelude: 2021 has `TryFrom`, not `Future`.
    let with_future = "pub struct L;\nimpl TryFrom<L> for u8 { fn m(&self) {} }\nimpl Future for u8 { fn n(&self) {} }\n";
    let tree: MemoryTree = [
        ("Cargo.toml", WORKSPACE_2021),
        (
            "crates/member/Cargo.toml",
            "[package]\nname = \"member\"\nedition.workspace = true\n",
        ),
        ("crates/member/src/lib.rs", with_future),
    ]
    .into_iter()
    .collect();
    let error = Workspace::load(&tree, &["crates"]).expect_err("Future is 2024");
    assert!(
        matches!(error.kind, LoadErrorKind::UnresolvedTrait { .. }),
        "{error}"
    );
}

#[test]
fn without_a_workspace_above_or_an_edition_in_it_the_inheritance_is_a_manifest_error() {
    for (label, root) in [
        (
            "no workspace table",
            "[package]\nname = \"top\"\nedition = \"2021\"\n",
        ),
        ("no edition", "[workspace]\nmembers = [\"crates/*\"]\n"),
    ] {
        let extra = [("src/lib.rs", "")];
        let error = Workspace::load(&inherited(root, &extra), &["crates"]).expect_err(label);
        assert!(
            matches!(error.kind, LoadErrorKind::Manifest { .. }),
            "{label}: {error}"
        );
        assert_eq!(error.file.as_str(), "crates/member/Cargo.toml", "{label}");
        assert!(
            matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key }) if key == "package.edition"),
            "{label}: {error}"
        );
    }
    // Nothing above the member at all.
    let tree: MemoryTree = [
        (
            "crates/member/Cargo.toml",
            "[package]\nname = \"member\"\nedition.workspace = true\n",
        ),
        ("crates/member/src/lib.rs", ""),
    ]
    .into_iter()
    .collect();
    assert!(Workspace::load(&tree, &["crates"]).is_err());
}

#[test]
fn the_nearest_ancestor_with_a_workspace_table_decides_and_others_are_skipped() {
    // A package manifest between the member and the workspace is skipped.
    let skipped = [(
        "crates/Cargo.toml",
        "[package]\nname = \"mid\"\nedition = \"2018\"\n",
    )];
    assert!(Workspace::load(&inherited(WORKSPACE_2021, &skipped), &["crates/member"]).is_ok());
    // A nearer workspace without an edition does not defer to a farther one.
    let nearer = [("crates/Cargo.toml", "[workspace]\nmembers = [\"member\"]\n")];
    let error = Workspace::load(&inherited(WORKSPACE_2021, &nearer), &["crates/member"])
        .expect_err("the nearest workspace has none");
    assert!(
        matches!(error.kind, LoadErrorKind::Manifest { .. }),
        "{error}"
    );
    // A nearer workspace with one wins: 2015 has no `TryFrom` in its prelude.
    let nearer = [(
        "crates/Cargo.toml",
        "[workspace]\nmembers = [\"member\"]\n\n[workspace.package]\nedition = \"2015\"\n",
    )];
    let error = Workspace::load(&inherited(WORKSPACE_2021, &nearer), &["crates/member"])
        .expect_err("the nearer 2015 edition applies");
    assert!(
        matches!(error.kind, LoadErrorKind::UnresolvedTrait { .. }),
        "{error}"
    );
}

#[test]
fn a_malformed_ancestor_manifest_is_an_error_and_a_missing_one_is_not() {
    let error =
        Workspace::load(&inherited("[workspace\n", &[]), &["crates"]).expect_err("malformed");
    assert!(
        matches!(error.kind, LoadErrorKind::Manifest { .. }),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "Cargo.toml");
    // No manifest between `crates` and the root is fine.
    assert!(Workspace::load(&inherited(WORKSPACE_2021, &[]), &["crates/member"]).is_ok());
}

#[test]
fn an_ancestor_manifest_over_the_size_cap_is_a_typed_error() {
    let padded = format!("{WORKSPACE_2021}# {}\n", "x".repeat(300));
    let tree = inherited(&padded, &[]);
    let mut options = raf_source::LoadOptions::default();
    options.max_file_bytes = 250;
    let error = Workspace::load_with(&tree, &["crates"], &options).expect_err("over the cap");
    assert!(
        matches!(&error.kind, LoadErrorKind::Tree(e) if matches!(**e, raf_source::TreeError::TooLarge { .. })),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "Cargo.toml");
    // Under the cap the same tree loads.
    options.max_file_bytes = 1000;
    assert!(Workspace::load_with(&tree, &["crates"], &options).is_ok());
}

fn not_a_member(root: &str) -> LoadError {
    Workspace::load(&inherited(root, &[]), &["crates"]).expect_err("not a member")
}

#[test]
fn inheriting_requires_membership_by_the_members_and_exclude_lists() {
    // Listed by a glob, by name, and by a character class.
    for members in [
        "crates/*",
        "crates/member",
        "crates/mem?er",
        "crates/[a-z]*",
        "crates/[!x]ember",
    ] {
        let root = format!(
            "[workspace]\nmembers = [\"{members}\"]\n\n[workspace.package]\nedition = \"2021\"\n"
        );
        assert!(
            Workspace::load(&inherited(&root, &[]), &["crates"]).is_ok(),
            "{members}"
        );
    }
    // Excluded, by name and below an excluded directory.
    for exclude in ["crates/member", "crates"] {
        let root = format!(
            "[workspace]\nmembers = [\"crates/*\"]\nexclude = [\"{exclude}\"]\n\n[workspace.package]\nedition = \"2021\"\n"
        );
        let error = not_a_member(&root);
        assert!(
            matches!(error.kind, LoadErrorKind::Manifest { .. }),
            "{exclude}: {error}"
        );
        assert!(error.to_string().contains("not a member"), "{error}");
    }
    // Not listed.
    for members in ["other/*", "crates/memberx*", "crates/[!m]ember"] {
        let root = format!(
            "[workspace]\nmembers = [\"{members}\"]\n\n[workspace.package]\nedition = \"2021\"\n"
        );
        let error = not_a_member(&root);
        assert_eq!(error.file.as_str(), "crates/member/Cargo.toml", "{members}");
        assert!(error.to_string().contains("Cargo.toml"), "{error}");
        assert!(
            matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key }) if key == "package.edition"),
            "{members}: {error}"
        );
    }
}

#[test]
fn the_workspace_root_package_inherits_without_being_listed() {
    let root = "[package]\nname = \"top\"\nedition.workspace = true\n\n[workspace]\nmembers = []\n\n[workspace.package]\nedition = \"2021\"\n";
    let tree: MemoryTree = [
        ("Cargo.toml", root),
        (
            "src/lib.rs",
            "pub struct L;\nimpl TryFrom<L> for u8 { fn m(&self) {} }\n",
        ),
    ]
    .into_iter()
    .collect();
    assert!(Workspace::load(&tree, &["."]).is_ok());
}

#[test]
fn a_pattern_outside_the_supported_syntax_is_a_manifest_error_naming_the_list() {
    for (list, pattern) in [
        ("members", "crates/**"),
        ("members", "crates/[a"),
        ("members", "crates\\\\x"),
        ("exclude", "crates/**"),
    ] {
        let (members, exclude) = if list == "members" {
            (format!("\"{pattern}\""), String::new())
        } else {
            ("\"crates/*\"".to_owned(), format!("\"{pattern}\""))
        };
        let root = format!(
            "[workspace]\nmembers = [{members}]\nexclude = [{exclude}]\n\n[workspace.package]\nedition = \"2021\"\n"
        );
        let error = Workspace::load(&inherited(&root, &[]), &["crates"]).expect_err(pattern);
        assert!(
            matches!(error.kind, LoadErrorKind::Manifest { .. }),
            "{pattern}: {error}"
        );
        assert_eq!(error.file.as_str(), "Cargo.toml", "{pattern}");
        assert!(
            matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key }) if *key == format!("workspace.{list}")),
            "{pattern}: {error}"
        );
    }
}

const HELPER: &str = "[package]\nname = \"helper\"\nedition.workspace = true\n";

/// A workspace whose root package depends on `crates/helper` by path and lists no members.
fn implied(root_extra: &str, helper: &str, files: &[(&str, &str)]) -> MemoryTree {
    let root = format!(
        "[package]\nname = \"top\"\nedition = \"2021\"\n\n[dependencies]\nhelper = {{ path = \"crates/helper\" }}\n{root_extra}\n\
         [workspace]\nmembers = []\n\n[workspace.package]\nedition = \"2021\"\n"
    );
    let mut all: Vec<(String, String)> = vec![
        ("Cargo.toml".to_owned(), root),
        ("src/lib.rs".to_owned(), String::new()),
        ("crates/helper/Cargo.toml".to_owned(), helper.to_owned()),
        ("crates/helper/src/lib.rs".to_owned(), String::new()),
    ];
    all.extend(
        files
            .iter()
            .map(|(a, b)| ((*a).to_owned(), (*b).to_owned())),
    );
    all.into_iter().collect()
}

#[test]
fn a_package_that_is_only_a_path_dependency_of_the_root_package_is_a_member() {
    let ws = Workspace::load(&implied("", HELPER, &[]), &["crates"])
        .expect("an implied member inherits");
    assert_eq!(ws.packages().len(), 1);
    // Through `[workspace.dependencies]`, the form `helper.workspace = true`.
    let root = "[package]\nname = \"top\"\nedition = \"2021\"\n\n[dependencies]\nhelper = { workspace = true }\n\n\
                [workspace]\nmembers = []\n\n[workspace.package]\nedition = \"2021\"\n\n\
                [workspace.dependencies]\nhelper = { path = \"crates/helper\" }\n";
    let tree: MemoryTree = [
        ("Cargo.toml", root),
        ("src/lib.rs", ""),
        ("crates/helper/Cargo.toml", HELPER),
        ("crates/helper/src/lib.rs", ""),
    ]
    .into_iter()
    .collect();
    assert!(Workspace::load(&tree, &["crates"]).is_ok());
    // Without the dependency the same package is not a member.
    let alone = root.replace("helper = { workspace = true }\n", "");
    let tree: MemoryTree = [
        ("Cargo.toml", alone.as_str()),
        ("src/lib.rs", ""),
        ("crates/helper/Cargo.toml", HELPER),
        ("crates/helper/src/lib.rs", ""),
    ]
    .into_iter()
    .collect();
    let error = Workspace::load(&tree, &["crates"]).expect_err("not implied");
    assert!(error.to_string().contains("not a member"), "{error}");
}

#[test]
fn path_dependencies_of_members_are_members_transitively_in_every_dependency_table() {
    for table in [
        "dependencies",
        "dev-dependencies",
        "build-dependencies",
        "target.'cfg(unix)'.dependencies",
    ] {
        let helper = format!("{HELPER}\n[{table}]\ndeep = {{ path = \"../deep\" }}\n");
        let tree = implied(
            "",
            helper.as_str(),
            &[
                (
                    "crates/deep/Cargo.toml",
                    "[package]\nname = \"deep\"\nedition.workspace = true\n",
                ),
                ("crates/deep/src/lib.rs", ""),
            ],
        );
        let ws = Workspace::load(&tree, &["crates"]).unwrap_or_else(|e| panic!("{table}: {e}"));
        assert_eq!(ws.packages().len(), 2, "{table}");
    }
}

#[test]
fn a_path_dependency_outside_the_workspace_directory_or_excluded_is_not_a_member() {
    // Outside: the package below another workspace is not a member of that one.
    let outer = "[workspace]\nmembers = []\n\n[workspace.package]\nedition = \"2021\"\n";
    let inner = "[workspace]\nmembers = [\"m\"]\n\n[workspace.package]\nedition = \"2021\"\n";
    let m = "[package]\nname = \"m\"\nedition.workspace = true\n\n[dependencies]\ndep = { path = \"../../elsewhere/dep\" }\n";
    let dep = "[package]\nname = \"dep\"\nedition.workspace = true\n";
    let tree: MemoryTree = [
        ("Cargo.toml", outer),
        ("ws/Cargo.toml", inner),
        ("ws/m/Cargo.toml", m),
        ("ws/m/src/lib.rs", ""),
        ("elsewhere/dep/Cargo.toml", dep),
        ("elsewhere/dep/src/lib.rs", ""),
    ]
    .into_iter()
    .collect();
    assert!(Workspace::load(&tree, &["ws/m"]).is_ok());
    let error =
        Workspace::load(&tree, &["elsewhere"]).expect_err("not a member of the outer workspace");
    assert!(error.to_string().contains("not a member"), "{error}");
    // Excluded.
    let root = "[package]\nname = \"top\"\nedition = \"2021\"\n\n[dependencies]\nhelper = { path = \"crates/helper\" }\n\n\
                [workspace]\nmembers = []\nexclude = [\"crates/helper\"]\n\n[workspace.package]\nedition = \"2021\"\n";
    let tree: MemoryTree = [
        ("Cargo.toml", root),
        ("src/lib.rs", ""),
        ("crates/helper/Cargo.toml", HELPER),
        ("crates/helper/src/lib.rs", ""),
    ]
    .into_iter()
    .collect();
    let error = Workspace::load(&tree, &["crates"]).expect_err("excluded");
    assert!(error.to_string().contains("not a member"), "{error}");
}

#[test]
fn a_path_dependency_inside_the_workspace_with_no_manifest_is_an_error_naming_the_key() {
    let tree = implied("gone = { path = \"crates/gone\" }\n", HELPER, &[]);
    let error = Workspace::load(&tree, &["crates"]).expect_err("no manifest");
    assert!(
        matches!(error.kind, LoadErrorKind::Manifest { .. }),
        "{error}"
    );
    assert!(
        matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key }) if key == "dependencies.gone"),
        "{error}"
    );
    let broken = implied(
        "gone = { path = \"crates/gone\" }\n",
        HELPER,
        &[("crates/gone/Cargo.toml", "[package\n")],
    );
    let error = Workspace::load(&broken, &["crates/helper"]).expect_err("malformed");
    assert!(
        matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key }) if key == "dependencies.gone"),
        "{error}"
    );
}

/// A workspace with members listed by `members`, plus the extra files.
fn listed(members: &str, files: &[(&str, &str)]) -> MemoryTree {
    let root =
        format!("[workspace]\nmembers = [{members}]\n\n[workspace.package]\nedition = \"2021\"\n");
    let mut all: Vec<(String, String)> = vec![
        ("Cargo.toml".to_owned(), root),
        (
            "crates/member/Cargo.toml".to_owned(),
            "[package]\nname = \"member\"\nedition.workspace = true\n".to_owned(),
        ),
        ("crates/member/src/lib.rs".to_owned(), String::new()),
    ];
    all.extend(
        files
            .iter()
            .map(|(a, b)| ((*a).to_owned(), (*b).to_owned())),
    );
    all.into_iter().collect()
}

#[test]
fn matching_a_pattern_takes_time_in_proportion_to_pattern_and_name() {
    let pattern = format!("crates/{}b", "*a".repeat(24));
    let name = "a".repeat(40);
    let manifest = format!("crates/{name}/readme.txt");
    let tree = listed(
        &format!("\"crates/member\", \"{pattern}\""),
        &[(manifest.as_str(), "no manifest here")],
    );
    let started = std::time::Instant::now();
    let ws = Workspace::load(&tree, &["crates/member"]).expect("loads");
    assert_eq!(ws.packages().len(), 1);
    assert!(
        started.elapsed() < std::time::Duration::from_secs(1),
        "{:?}",
        started.elapsed()
    );
}

#[test]
fn a_path_dependency_with_an_unusable_path_is_an_error_beside_one_that_is_outside_the_tree() {
    for bad in ["crates\\\\x", "C:x"] {
        let extra = format!("gone = {{ path = \"{bad}\" }}\n");
        let error = Workspace::load(&implied(&extra, HELPER, &[]), &["crates"]).expect_err(bad);
        assert!(
            matches!(error.kind, LoadErrorKind::Manifest { .. }),
            "{bad}: {error}"
        );
        assert!(
            matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key }) if key == "dependencies.gone"),
            "{bad}: {error}"
        );
    }
    // Above the root or absolute: simply not a member.
    for outside in ["../../up", "/absolute/elsewhere"] {
        let extra = format!("far = {{ path = \"{outside}\" }}\n");
        assert!(
            Workspace::load(&implied(&extra, HELPER, &[]), &["crates"]).is_ok(),
            "{outside}"
        );
    }
}

#[test]
fn a_literal_member_without_a_manifest_is_an_error_and_a_glob_skips_such_a_directory() {
    let docs = [("crates/docs/readme.txt", "text")];
    for literal in ["crates/nope", "crates/docs"] {
        let tree = listed(&format!("\"crates/member\", \"{literal}\""), &docs);
        let error = Workspace::load(&tree, &["crates/member"]).expect_err(literal);
        assert!(
            matches!(error.kind, LoadErrorKind::Manifest { .. }),
            "{literal}: {error}"
        );
        assert!(
            matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key }) if key == "workspace.members"),
            "{literal}: {error}"
        );
    }
    let tree = listed("\"crates/*\"", &docs);
    assert!(Workspace::load(&tree, &["crates/member"]).is_ok());
}

#[test]
fn a_workspace_dependency_whose_path_is_not_a_string_is_an_error_naming_the_key() {
    let root = |path: &str| {
        format!(
            "[workspace]\nmembers = [\"crates/member\"]\n\n[workspace.package]\nedition = \"2021\"\n\n[workspace.dependencies]\nhelper = {{ path = {path} }}\n"
        )
    };
    let files = |root: &str| -> MemoryTree {
        [
            ("Cargo.toml", root),
            (
                "crates/member/Cargo.toml",
                "[package]\nname = \"member\"\nedition.workspace = true\n",
            ),
            ("crates/member/src/lib.rs", ""),
        ]
        .into_iter()
        .collect()
    };
    let bad = root("3");
    let error = Workspace::load(&files(&bad), &["crates"]).expect_err("not a string");
    assert!(
        matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key }) if key == "workspace.dependencies.helper"),
        "{error}"
    );
    let good = root("\"crates/other\"");
    assert!(Workspace::load(&files(&good), &["crates"]).is_ok());
}

#[test]
fn an_empty_pattern_segment_is_a_pattern_error_beside_a_plain_pattern() {
    for pattern in ["crates//member", "crates/member//", "//", "/crates/member"] {
        let tree = listed(&format!("\"{pattern}\""), &[]);
        let error = Workspace::load(&tree, &["crates"]).expect_err(pattern);
        assert!(
            matches!(error.kind, LoadErrorKind::Manifest { .. }),
            "{pattern}: {error}"
        );
        assert!(
            matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key }) if key == "workspace.members"),
            "{pattern}: {error}"
        );
    }
    assert!(Workspace::load(&listed("\"./crates/member\"", &[]), &["crates"]).is_ok());
    // Exactly one trailing `/` is the same pattern.
    for accepted in ["crates/member/", "crates/*/", "./crates/m*/"] {
        assert!(
            Workspace::load(&listed(&format!("\"{accepted}\""), &[]), &["crates"]).is_ok(),
            "{accepted}"
        );
    }
}

#[test]
fn an_excluded_literal_member_is_ignored_and_not_required_to_exist() {
    let root = |exclude: &str| {
        format!(
            "[workspace]\nmembers = [\"crates/member\", \"crates/gone\"]\nexclude = [{exclude}]\n\n[workspace.package]\nedition = \"2021\"\n"
        )
    };
    let files = |root: &str| -> MemoryTree {
        [
            ("Cargo.toml", root),
            (
                "crates/member/Cargo.toml",
                "[package]\nname = \"member\"\nedition.workspace = true\n",
            ),
            ("crates/member/src/lib.rs", ""),
        ]
        .into_iter()
        .collect()
    };
    let excluded = root("\"crates/gone\"");
    assert!(Workspace::load(&files(&excluded), &["crates/member"]).is_ok());
    // Not excluded, the same missing member is an error.
    let kept = root("");
    let error = Workspace::load(&files(&kept), &["crates/member"]).expect_err("missing member");
    assert!(
        matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key }) if key == "workspace.members"),
        "{error}"
    );
}

#[test]
fn a_macro_call_in_a_trait_body_is_skipped() {
    let text = "macro_rules! decl { () => { fn generated(&self); } }\n\
                pub trait T {\n    decl!();\n    fn written(&self);\n}\n\
                pub struct S;\n\
                impl T for S {\n    decl!();\n    fn written(&self) {}\n}\n\
                impl T for u8 { fn written(&self) {} }\n";
    let ws = lib("2021", text).unwrap_or_else(|e| panic!("{e}"));
    let trait_item = ws
        .resolve("demo::T")
        .map(|found| ws.item(found.sites[0]).expect("item").clone())
        .expect("the trait");
    assert_eq!(trait_item.kind, raf_source::SymbolKind::Trait);
    // The written member is indexed; what the macro would generate is not.
    assert!(ws.resolve("demo::T::written").is_ok());
    assert!(matches!(
        ws.resolve("demo::T::generated"),
        Err(raf_source::ResolveError::NotFound(_))
    ));
    assert!(ws.resolve("demo::S::written").is_ok());
    let on_u8: Vec<_> = ws
        .items()
        .filter(|item| item.path == "demo::T::written" && item.trait_key.is_some())
        .map(|item| item.trait_key.as_deref().cloned())
        .collect();
    assert_eq!(on_u8, [Some(TraitKey::Canonical("demo::T".to_owned()))]);
}

#[test]
fn a_dependency_package_that_is_not_a_string_is_an_error_naming_the_key() {
    for (extra, key) in [
        (
            "\n[dependencies]\nrenamed = { package = 3, version = \"1\" }\n",
            "dependencies.renamed",
        ),
        (
            "\n[target.'cfg(unix)'.dev-dependencies]\nrenamed = { package = [\"x\"], path = \"x\" }\n",
            "target.cfg(unix).dev-dependencies.renamed",
        ),
    ] {
        let m = manifest("2021", extra);
        let error = load(&m, &[("src/lib.rs", "")]).expect_err(key);
        assert!(
            matches!(&error.kind, LoadErrorKind::Manifest { message } if message.contains("`package` must be a string")),
            "{error}"
        );
        assert!(
            matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key: found }) if found == key),
            "{error}"
        );
    }
    let good = manifest(
        "2021",
        "\n[dependencies]\nrenamed = { package = \"original\", version = \"1\" }\n",
    );
    assert!(load(&good, &[("src/lib.rs", "")]).is_ok());
    // The same refusal in `[workspace.dependencies]`.
    let root = |package: &str| -> MemoryTree {
        let text = format!(
            "[workspace]\nmembers = [\"m\"]\n\n[workspace.dependencies]\nhelper = {{ package = {package}, version = \"1\" }}\n"
        );
        [
            ("Cargo.toml", text.as_str()),
            ("m/Cargo.toml", "[package]\nname = \"m\"\nedition = \"2021\"\n\n[dependencies]\nhelper = { workspace = true }\n"),
            ("m/src/lib.rs", ""),
        ]
        .into_iter()
        .collect()
    };
    let error = Workspace::load(&root("1"), &["m"]).expect_err("not a string");
    assert!(
        matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key }) if key == "workspace.dependencies.helper"),
        "{error}"
    );
    assert!(Workspace::load(&root("\"h\""), &["m"]).is_ok());
}

#[test]
fn an_inherited_dependency_the_workspace_cannot_give_is_an_error_naming_the_key() {
    let member = "[package]\nname = \"m\"\nedition = \"2021\"\n\n[dependencies]\nhelper = { workspace = true }\n";
    let cases: [(&str, Option<&str>, &str); 3] = [
        // No workspace manifest above the package.
        ("", None, "no workspace manifest above the package"),
        // A workspace the package is not a member of.
        (
            "[workspace]\nmembers = []\n\n[workspace.dependencies]\nhelper = \"1\"\n",
            Some("Cargo.toml"),
            "not a member of the workspace",
        ),
        // A workspace without the entry.
        (
            "[workspace]\nmembers = [\"m\"]\n\n[workspace.dependencies]\nother = \"1\"\n",
            Some("Cargo.toml"),
            "has no `[workspace.dependencies]` entry `helper`",
        ),
    ];
    for (root, at, why) in cases {
        let mut files = vec![("m/Cargo.toml", member), ("m/src/lib.rs", "")];
        if let Some(path) = at {
            files.push((path, root));
        }
        let tree: MemoryTree = files.into_iter().collect();
        let error = Workspace::load(&tree, &["m"]).expect_err(why);
        assert_eq!(error.file.as_str(), "m/Cargo.toml", "{error}");
        assert!(
            matches!(&error.kind, LoadErrorKind::Manifest { message } if message.contains(why)),
            "{error}"
        );
        assert!(
            matches!(error.origin.as_deref(), Some(raf_source::Origin::ManifestKey { key }) if key == "dependencies.helper"),
            "{error}"
        );
    }
    // The workspace that has the package as a member and the entry loads.
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"m\"]\n\n[workspace.dependencies]\nhelper = \"1\"\n",
        ),
        ("m/Cargo.toml", member),
        ("m/src/lib.rs", ""),
    ]
    .into_iter()
    .collect();
    assert!(Workspace::load(&tree, &["m"]).is_ok());
}
