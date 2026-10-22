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

#[test]
fn an_explicit_import_of_a_dependency_trait_shadows_a_glob_of_a_local_one() {
    // rustc reads `Serialize` through the explicit import, not the glob,
    // though the index cannot see the dependency's trait.
    let m = manifest("2021", "\n[dependencies]\nserde = \"1\"\n");
    let local =
        "mod local { pub trait Serialize { fn local(&self); } }\nuse local::*;\npub struct L;\n";
    let explicit = format!(
        "{local}use serde::Serialize;\nimpl Serialize for Vec<L> {{ fn ser(&self) {{}} }}\n"
    );
    let ws = load(&m, &[("src/lib.rs", &explicit)]).expect("loads");
    assert_eq!(
        key_of(&ws, "serde::Serialize::ser"),
        TraitKey::Canonical("serde::Serialize".to_owned())
    );
    // Without the explicit import the glob's local trait is the one.
    let globbed = format!("{local}impl Serialize for Vec<L> {{ fn local(&self) {{}} }}\n");
    let ws = load(&m, &[("src/lib.rs", &globbed)]).expect("loads");
    let keys: Vec<TraitKey> = ws
        .items()
        .filter(|item| item.self_type.as_deref() == Some("Vec<L>"))
        .filter_map(|item| item.trait_key.as_deref().cloned())
        .collect();
    assert_eq!(
        keys,
        [TraitKey::Canonical("demo::local::Serialize".to_owned())]
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

// The step bound of the matcher is held by the unit test
// `glob::tests::a_segment_is_matched_in_at_most_pattern_times_name_steps`, which
// has a ceiling on the steps. This test shows the outcome through the whole
// load; without that bound a matcher that backtracked without limit would make
// it wait, not fail.
#[test]
fn a_star_heavy_member_pattern_that_matches_no_directory_leaves_the_member_alone() {
    let pattern = format!("crates/{}b", "*a".repeat(24));
    let name = "a".repeat(40);
    let manifest = format!("crates/{name}/readme.txt");
    let tree = listed(
        &format!("\"crates/member\", \"{pattern}\""),
        &[(manifest.as_str(), "no manifest here")],
    );
    let ws = Workspace::load(&tree, &["crates/member"]).expect("loads");
    assert_eq!(ws.packages().len(), 1);
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

/// Load `text` as the library of a package of `edition` that depends on
/// `serde`, a crate outside the workspace.
fn with_serde(edition: &str, text: &str) -> Result<Workspace, LoadError> {
    load(
        &manifest(edition, "\n[dependencies]\nserde = \"1\"\n"),
        &[("src/lib.rs", text)],
    )
}

#[test]
fn a_trait_through_a_single_segment_import_of_a_crate_is_keyed_at_the_crate() {
    let display = "pub struct Foo;\nimpl std::fmt::Display for &Foo { fn fmt(&self) {} }\n";
    for edition in ["2018", "2021", "2024"] {
        let ws = lib(edition, &format!("use std;\n{display}"))
            .unwrap_or_else(|e| panic!("{edition}: {e}"));
        assert_eq!(
            key_of(&ws, "std::fmt::Display::fmt"),
            TraitKey::Canonical("std::fmt::Display".to_owned()),
            "{edition}"
        );
    }
    // In edition 2015 a module's `use std;` is the crate root's `std`.
    let ws = lib(
        "2015",
        &format!("pub mod m {{\n    use std;\n    {display}}}\n"),
    )
    .unwrap_or_else(|e| panic!("2015: {e}"));
    assert_eq!(
        key_of(&ws, "std::fmt::Display::fmt"),
        TraitKey::Canonical("std::fmt::Display".to_owned())
    );
    // A single-segment import of a name that is no crate gives no key.
    let error = lib(
        "2021",
        &format!("use nope;\n{}", display.replace("std", "nope")),
    )
    .expect_err("no such crate");
    assert!(
        matches!(&error.kind, LoadErrorKind::UnresolvedTrait { path } if path == "nope::fmt::Display"),
        "{error}"
    );
    // Re-exported, through a module and in a group with `self`, the crate
    // is followed to its own path.
    let ws = with_serde(
        "2021",
        "pub use serde;\npub mod a {\n    use crate::serde::Serialize;\n    \
         impl Serialize for String { fn serialize(&self) {} }\n}\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        key_of(&ws, "serde::Serialize::serialize"),
        TraitKey::Canonical("serde::Serialize".to_owned())
    );
    let ws = with_serde(
        "2021",
        "pub use serde::{self, Serialize};\npub struct L;\n\
         impl Serialize for Vec<L> { fn serialize(&self) {} }\n\
         impl serde::de::Deserialize for Vec<L> { fn deserialize(&self) {} }\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        key_of(&ws, "serde::Serialize::serialize"),
        TraitKey::Canonical("serde::Serialize".to_owned())
    );
    assert_eq!(
        key_of(&ws, "serde::de::Deserialize::deserialize"),
        TraitKey::Canonical("serde::de::Deserialize".to_owned())
    );
}

#[test]
fn a_glob_through_a_re_exported_crate_is_keyed_under_the_crate() {
    let ws = with_serde(
        "2021",
        "pub use serde;\nuse serde::prelude::*;\npub struct L;\n\
         impl Serialize for Vec<L> { fn m(&self) {} }\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        key_of(&ws, "serde::prelude::Serialize::m"),
        TraitKey::Canonical("serde::prelude::Serialize".to_owned())
    );
}

#[test]
fn a_use_path_of_edition_2015_is_followed_from_the_crate_root() {
    // A module's `use serde;` is the root's `extern crate serde;`.
    let ws = with_serde(
        "2015",
        "extern crate serde;\npub mod de {\n    use serde;\n    pub struct D;\n    \
         impl<'de> serde::Deserializer<'de> for &'de mut D { fn f(&self) {} }\n}\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        key_of(&ws, "serde::Deserializer::f"),
        TraitKey::Canonical("serde::Deserializer".to_owned())
    );
    // A module's `use undo_log::Rollback;` names the root's module.
    let ws = lib(
        "2015",
        "pub mod undo_log { pub trait Rollback<U> { fn reverse(&mut self, undo: U); } }\n\
         pub mod snapshot_vec {\n    use undo_log::Rollback;\n    \
         impl Rollback<u8> for Vec<u8> { fn reverse(&mut self, undo: u8) {} }\n}\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        impl_key(&ws, "Vec<u8>"),
        TraitKey::Canonical("demo::undo_log::Rollback".to_owned())
    );
    // A path with a leading `::` starts at the crate root too, so a renamed
    // `extern crate` item is the crate it names.
    let ws = with_serde(
        "2015",
        "extern crate serde as s;\npub struct L;\n\
         impl ::s::Serialize for Vec<L> { fn m(&self) {} }\n\
         impl ::std::fmt::Display for Vec<L> { fn fmt(&self) {} }\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        key_of(&ws, "serde::Serialize::m"),
        TraitKey::Canonical("serde::Serialize".to_owned())
    );
    assert_eq!(
        key_of(&ws, "std::fmt::Display::fmt"),
        TraitKey::Canonical("std::fmt::Display".to_owned())
    );
    // From edition 2018 on, the same `::s` names no crate.
    let error = with_serde(
        "2018",
        "extern crate serde as s;\npub struct L;\nimpl ::s::Serialize for Vec<L> { fn m(&self) {} }\n",
    )
    .expect_err("`s` is no crate of the extern prelude");
    assert!(
        matches!(error.kind, LoadErrorKind::UnresolvedTrait { .. }),
        "{error}"
    );
}

#[test]
fn a_trait_through_an_ambiguous_use_head_keys_every_reading() {
    let ws = with_serde(
        "2021",
        "pub mod local { pub mod serde { pub trait Serialize { fn m(&self); } } }\n\
         use crate::local::*;\nuse serde::Serialize;\npub struct L;\n\
         impl Serialize for Vec<L> { fn m(&self) {} }\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let resolution = ws
        .resolve_in("Serialize::m", Namespace::Value)
        .expect("by its written path");
    let item = ws.item(resolution.sites[0]).expect("item");
    assert_eq!(
        item.trait_key.as_deref(),
        Some(&TraitKey::Unresolved {
            written: "Serialize".to_owned(),
            candidates: vec![
                "demo::local::serde::Serialize".to_owned(),
                "serde::Serialize".to_owned()
            ],
        })
    );
}

/// The key of the one member of a trait impl on `self_type`, which a local
/// trait's own declaration hides from `resolve`.
fn impl_key(ws: &Workspace, self_type: &str) -> TraitKey {
    let keys: Vec<TraitKey> = ws
        .items()
        .filter(|item| item.self_type.as_deref() == Some(self_type))
        .filter_map(|item| item.trait_key.as_deref().cloned())
        .collect();
    match keys.as_slice() {
        [key] => key.clone(),
        other => panic!("{self_type}: {other:?}"),
    }
}

/// Load `text` as the library of `demo` on a thread of 2 MiB, the stack of a
/// test thread.
fn load_on_a_small_stack(text: String) -> Result<Workspace, LoadError> {
    std::thread::Builder::new()
        .stack_size(2 << 20)
        .spawn(move || lib("2021", &text))
        .expect("spawn")
        .join()
        .expect("the load does not abort")
}

#[test]
fn twenty_thousand_chained_local_aliases_load_on_a_two_mib_thread() {
    use std::fmt::Write as _;
    let mut text =
        String::from("pub mod m { pub trait T { fn t(&self); } }\nuse crate::m::T as A0;\n");
    for at in 1..20_000 {
        writeln!(text, "use A{} as A{at};", at - 1).expect("write");
    }
    text.push_str("impl A19999 for X { fn t(&self) {} }\n");
    assert!(!text.contains("use A0 as A0;"));
    let ws = load_on_a_small_stack(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        impl_key(&ws, "X"),
        TraitKey::Canonical("demo::m::T".to_owned())
    );
}

#[test]
fn a_trait_found_through_twenty_thousand_chained_globs_is_keyed_on_a_two_mib_thread() {
    use std::fmt::Write as _;
    let mut text = String::from("pub mod m0 { pub use std::fmt::Display; }\n");
    for at in 1..20_000 {
        writeln!(text, "pub mod m{at} {{ pub use crate::m{}::*; }}", at - 1).expect("write");
    }
    text.push_str("pub mod last {\n    use crate::m19999::*;\n    impl Display for Vec<u8> { fn fmt(&self) {} }\n}\n");
    let ws = load_on_a_small_stack(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        key_of(&ws, "std::fmt::Display::fmt"),
        TraitKey::Canonical("std::fmt::Display".to_owned())
    );
}

#[test]
fn an_import_met_again_through_a_glob_adds_nothing_and_a_pure_cycle_is_refused() {
    // `inner` globs its parent, whose `Serialize` is the import being
    // followed; the other glob of `inner` gives the trait, as rustc has it.
    let ws = with_serde(
        "2021",
        "mod inner { use super::*; pub use serde::*; }\npub use inner::Serialize;\n\
         pub struct Foo;\nimpl Serialize for &Foo { fn m(&self) {} }\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        key_of(&ws, "serde::Serialize::m"),
        TraitKey::Canonical("serde::Serialize".to_owned())
    );
    // Imports that only name each other give nothing, and the trait is
    // refused.
    let error = lib(
        "2021",
        "pub mod a { pub use super::b::T; }\npub mod b { pub use super::a::T; }\n\
         impl a::T for Vec<u8> { fn m(&self) {} }\n",
    )
    .expect_err("a cycle names no trait");
    assert!(
        matches!(&error.kind, LoadErrorKind::UnresolvedTrait { path } if path == "a::T"),
        "{error}"
    );
}

#[test]
fn a_trait_through_a_head_a_glob_brings_in_from_another_crate_keys_both() {
    let ws = with_serde(
        "2021",
        "pub mod m { pub use std as serde; }\nuse m::*;\nuse serde::Serialize;\n\
         impl Serialize for Vec<u8> { fn m(&self) {} }\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let resolution = ws
        .resolve_in("Serialize::m", Namespace::Value)
        .expect("by its written path");
    assert_eq!(
        ws.item(resolution.sites[0])
            .expect("item")
            .trait_key
            .as_deref(),
        Some(&TraitKey::Unresolved {
            written: "Serialize".to_owned(),
            candidates: vec!["serde::Serialize".to_owned(), "std::Serialize".to_owned()],
        })
    );
}

#[test]
fn a_use_path_of_edition_2015_names_a_dependency_only_through_extern_crate() {
    let impl_text =
        "use serde::Serialize;\npub struct L;\nimpl Serialize for Vec<L> { fn m(&self) {} }\n";
    // Without an `extern crate` item at the root, or with one only in a
    // module, rustc refuses the import, and no key is made up.
    for prefix in ["", "pub mod m { extern crate serde; }\n"] {
        let error = with_serde("2015", &format!("{prefix}{impl_text}"))
            .expect_err("no `serde` at the crate root");
        assert!(
            matches!(&error.kind, LoadErrorKind::UnresolvedTrait { path } if path == "Serialize"),
            "{prefix}: {error}"
        );
    }
    let ws = with_serde("2015", &format!("extern crate serde;\n{impl_text}"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        key_of(&ws, "serde::Serialize::m"),
        TraitKey::Canonical("serde::Serialize".to_owned())
    );
    // The crate the compiler puts at the root needs no item: `std`, or
    // `core` under `#![no_std]`.
    for (prefix, krate) in [("", "std"), ("#![no_std]\n", "core")] {
        let text = format!(
            "{prefix}use {krate}::fmt::Display;\npub struct L;\nimpl Display for Vec<L> {{ fn fmt(&self) {{}} }}\n"
        );
        let ws = lib("2015", &text).unwrap_or_else(|e| panic!("{krate}: {e}"));
        assert_eq!(
            key_of(&ws, "std::fmt::Display::fmt"),
            TraitKey::Canonical("std::fmt::Display".to_owned()),
            "{krate}"
        );
    }
    let error = lib(
        "2015",
        "use core::fmt::Display;\npub struct L;\nimpl Display for Vec<L> { fn fmt(&self) {} }\n",
    )
    .expect_err("`core` is not at the root of a crate with `std`");
    assert!(
        matches!(error.kind, LoadErrorKind::UnresolvedTrait { .. }),
        "{error}"
    );
}

#[test]
fn a_trait_name_that_two_globs_bind_keys_each_trait() {
    let ws = lib(
        "2021",
        "pub mod a { pub trait T { fn m(&self); } }\npub mod b { pub trait T { fn m(&self); } }\n\
         use a::*;\nuse b::*;\nimpl T for Vec<u8> { fn m(&self) {} }\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        impl_key(&ws, "Vec<u8>"),
        TraitKey::Unresolved {
            written: "T".to_owned(),
            candidates: vec!["demo::a::T".to_owned(), "demo::b::T".to_owned()],
        }
    );
}

#[test]
fn the_completion_audit_shape_of_a_crate_alias_and_twenty_thousand_imports_loads_on_a_two_mib_thread()
 {
    use std::fmt::Write as _;
    let mut text = String::from(
        "use std;\npub struct Foo;\nimpl std::fmt::Display for &Foo { fn fmt(&self) {} }\n",
    );
    for at in 0..20_000 {
        writeln!(text, "use std::collections::HashMap as H{at};").expect("write");
    }
    let ws = load_on_a_small_stack(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        key_of(&ws, "std::fmt::Display::fmt"),
        TraitKey::Canonical("std::fmt::Display".to_owned())
    );
}

#[test]
fn a_trait_that_several_unknown_globs_could_give_is_refused_naming_one_glob_in_every_order() {
    let blocks = [
        "use sh::*;",
        "use inn::inner::*;",
        "impl T for Vec<u8> { fn m(&self) {} }",
    ];
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for order in orders {
        let text = order.map(|at| blocks[at]).join("\n");
        let error = lib("2021", &text).expect_err("no glob can be looked into");
        let line = order.iter().position(|at| *at == 1).expect("the glob") + 1;
        assert!(
            matches!(&error.kind, LoadErrorKind::UnresolvedTrait { path } if path == "inn::inner::*"),
            "{text}: {error}"
        );
        assert_eq!(error.line, Some(line), "{text}");
    }
}

/// Every order of `blocks`.
fn every_order(blocks: &[&str]) -> Vec<Vec<String>> {
    if blocks.is_empty() {
        return vec![Vec::new()];
    }
    let mut out = Vec::new();
    for at in 0..blocks.len() {
        let mut rest = blocks.to_vec();
        let first = rest.remove(at);
        for mut order in every_order(&rest) {
            order.insert(0, first.to_owned());
            out.push(order);
        }
    }
    out
}

#[test]
fn a_trait_through_a_glob_the_importer_cannot_access_is_keyed_at_its_definition_in_every_order() {
    // `a` imports `b::B`, a function, and globs `c`, whose trait `B` its
    // impl names. `b`'s glob of `a` is private to `b`, so `a`'s import reads
    // nothing of it, as rustc resolves the import: no cycle, and the trait
    // is `c::B`.
    let blocks = [
        "pub mod c { pub trait B { fn m(&self); } }",
        "pub mod a { pub use crate::b::B; pub use crate::c::*; }",
        "pub mod b { #[allow(non_snake_case)] pub fn B() {} use crate::a::*; }",
        "impl a::B for String { fn m(&self) {} }",
    ];
    let orders = every_order(&blocks);
    assert_eq!(orders.len(), 24);
    for order in &orders {
        let text = order.join("\n");
        let ws = lib("2021", &text).unwrap_or_else(|e| panic!("{text}: {e}"));
        let keys: Vec<TraitKey> = ws
            .items()
            .filter(|item| item.self_type.as_deref() == Some("String"))
            .filter_map(|item| item.trait_key.as_deref().cloned())
            .collect();
        assert_eq!(
            keys,
            [TraitKey::Canonical("demo::c::B".to_owned())],
            "{text}"
        );
    }
    // With `b`'s glob public, `a` reads `b::B` back through it: rustc cannot
    // determine the import (E0432), and the trait cannot be named.
    for order in every_order(&[
        blocks[0],
        blocks[1],
        "pub mod b { #[allow(non_snake_case)] pub fn B() {} pub use crate::a::*; }",
        blocks[3],
    ]) {
        let text = order.join("\n");
        let error = lib("2021", &text).expect_err("refused");
        assert!(
            matches!(&error.kind, LoadErrorKind::UnresolvedTrait { path } if path == "a::B"),
            "{text}: {error:?}"
        );
    }
}
