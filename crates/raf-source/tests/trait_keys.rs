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
        .clone()
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
