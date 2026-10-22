// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `resolve_from` read from a module of every kind of target: an integration
//! test, a binary and its modules, an example, a bench and the build script,
//! and a support module that several test targets include.

use raf_source::{MemoryTree, ResolveError, Workspace};

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

fn load(files: &[(&str, &str)]) -> Workspace {
    let mut all = vec![("Cargo.toml", MANIFEST)];
    all.extend_from_slice(files);
    let tree: MemoryTree = all.into_iter().collect();
    Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"))
}

/// The canonical path a `use` of `path` in `module` names.
fn from(ws: &Workspace, module: &str, path: &str) -> Result<String, ResolveError> {
    ws.resolve_from(module, path).map(|found| found.path)
}

/// Every module of every kind of target, with a library beside them.
fn every_target() -> Workspace {
    load(&[
        ("src/lib.rs", "pub fn api() {}\nfn private() {}\n"),
        (
            "src/main.rs",
            "mod cli { pub fn parse() {} pub mod nested { pub fn deep() { super::parse() } } }\nfn main() {}\n",
        ),
        (
            "src/bin/tool/main.rs",
            "mod cli;\nfn helper() {}\nfn main() {}\n",
        ),
        (
            "src/bin/tool/cli.rs",
            "pub fn run() {}\npub mod nested { pub fn deep() {} }\n",
        ),
        (
            "tests/it.rs",
            "fn helper() {}\nmod inner { pub fn local() {} }\n#[test]\nfn case() { helper() }\n",
        ),
        ("examples/ex.rs", "fn shown() {}\nfn main() {}\n"),
        ("benches/b.rs", "fn measured() {}\n"),
        ("build.rs", "fn configure() {}\nfn main() {}\n"),
    ])
}

#[test]
fn a_helper_resolves_from_an_integration_test_root_and_from_a_module_below_it() {
    let ws = every_target();
    let root = "demo::test_crate::it";
    assert_eq!(
        from(&ws, root, "helper"),
        Ok("demo::test_crate::it::helper".to_owned())
    );
    assert_eq!(
        from(&ws, root, "crate::inner::local"),
        Ok("demo::test_crate::it::inner::local".to_owned())
    );
    assert_eq!(
        from(&ws, "demo::test_crate::it::inner", "super::helper"),
        Ok("demo::test_crate::it::helper".to_owned())
    );
    // The library of the package is a crate of the test's extern prelude,
    // and a private item of it is not visible from the test.
    assert_eq!(from(&ws, root, "demo::api"), Ok("demo::api".to_owned()));
    assert_eq!(
        ws.resolve_from(root, "demo::private"),
        Err(ResolveError::NotFound("demo::private".to_owned()))
    );
    // `crate` is the test target's root, not the library's.
    assert_eq!(
        ws.resolve_from(root, "crate::api"),
        Err(ResolveError::NotFound("crate::api".to_owned()))
    );
}

#[test]
fn a_helper_resolves_from_a_nested_module_of_a_binary() {
    let ws = every_target();
    assert_eq!(
        from(&ws, "demo::bin_crate::tool::cli::nested", "super::run"),
        Ok("demo::bin_crate::tool::cli::run".to_owned())
    );
    assert_eq!(
        from(&ws, "demo::bin_crate::tool::cli::nested", "crate::helper"),
        Ok("demo::bin_crate::tool::helper".to_owned())
    );
    assert_eq!(
        from(&ws, "demo::bin_crate::tool::cli", "demo::api"),
        Ok("demo::api".to_owned())
    );
    // The binary of `src/main.rs` and its inline modules.
    assert_eq!(
        from(&ws, "demo::bin_crate::demo::cli::nested", "super::parse"),
        Ok("demo::bin_crate::demo::cli::parse".to_owned())
    );
}

#[test]
fn an_example_a_bench_and_the_build_script_are_read_from_as_well() {
    let ws = every_target();
    assert_eq!(
        from(&ws, "demo::example_crate::ex", "shown"),
        Ok("demo::example_crate::ex::shown".to_owned())
    );
    assert_eq!(
        from(&ws, "demo::example_crate::ex", "demo::api"),
        Ok("demo::api".to_owned())
    );
    assert_eq!(
        from(&ws, "demo::bench_crate::b", "self::measured"),
        Ok("demo::bench_crate::b::measured".to_owned())
    );
    assert_eq!(
        from(&ws, "demo::build_script", "configure"),
        Ok("demo::build_script::configure".to_owned())
    );
    // The build script does not name the library of its package.
    assert_eq!(
        ws.resolve_from("demo::build_script", "demo::api"),
        Err(ResolveError::NotFound("demo::api".to_owned()))
    );
}

#[test]
fn a_module_path_that_no_target_holds_is_unknown_whichever_kind_of_target_it_names() {
    let ws = every_target();
    for module in [
        "demo::test_crate::it::gone",
        "demo::test_crate::other",
        "demo::bin_crate::tool::gone",
        "demo::bin_crate::tool::cli::nested::gone",
        "demo::example_crate::ex::gone",
        "demo::bench_crate::b::gone",
        "demo::build_script::gone",
        "demo::test_crate",
        "demo::gone",
    ] {
        assert_eq!(
            ws.resolve_from(module, "self::x"),
            Err(ResolveError::UnknownModule(module.to_owned())),
            "{module}"
        );
    }
    // Each neighbour that a target holds reads.
    for module in [
        "demo::test_crate::it::inner",
        "demo::bin_crate::tool::cli::nested",
        "demo::example_crate::ex",
        "demo::bench_crate::b",
        "demo::build_script",
        "demo",
    ] {
        assert!(
            !matches!(
                ws.resolve_from(module, "self::x"),
                Err(ResolveError::UnknownModule(_))
            ),
            "{module}"
        );
    }
}

#[test]
fn a_support_module_that_two_test_targets_include_resolves_in_each_target_alone() {
    let ws = load(&[
        ("src/lib.rs", ""),
        (
            "tests/a.rs",
            "mod support;\n#[test]\nfn one() { support::helper() }\n",
        ),
        (
            "tests/b.rs",
            "mod support;\n#[test]\nfn two() { support::helper() }\n",
        ),
        ("tests/support/mod.rs", "pub fn helper() {}\n"),
    ]);
    let in_a = ws
        .resolve_from("demo::test_crate::a", "support::helper")
        .expect("from a");
    let in_b = ws
        .resolve_from("demo::test_crate::b", "support::helper")
        .expect("from b");
    assert_eq!(in_a.path, "demo::test_crate::a::support::helper");
    assert_eq!(in_b.path, "demo::test_crate::b::support::helper");
    // One site in each target, and the two are one source item.
    assert_eq!(in_a.sites.len(), 1);
    assert_eq!(in_b.sites.len(), 1);
    assert_ne!(in_a.sites, in_b.sites);
    let item = ws.item(in_a.sites[0]).expect("an item");
    let mut both = vec![in_a.sites[0], in_b.sites[0]];
    both.sort_by_key(|id| ws.item(*id).map(|item| item.path.to_string()));
    assert_eq!(ws.same_source(item), both.as_slice());
    // Each copy reads from its own target: `crate` there is the test's root.
    assert_eq!(
        from(&ws, "demo::test_crate::a::support", "crate::one"),
        Ok("demo::test_crate::a::one".to_owned())
    );
    assert_eq!(
        ws.resolve_from("demo::test_crate::a::support", "crate::two"),
        Err(ResolveError::NotFound("crate::two".to_owned()))
    );
}

#[test]
fn a_module_of_the_library_at_the_crate_path_of_a_test_target_is_ambiguous_with_it() {
    let ws = load(&[
        (
            "src/lib.rs",
            "pub mod test_crate { pub mod it { pub fn x() {} } pub mod lib_only { pub fn y() {} } }\n",
        ),
        ("tests/it.rs", "pub fn x() {}\n"),
    ]);
    let Err(ResolveError::Ambiguous(ids)) = ws.resolve_from("demo::test_crate::it", "self::x")
    else {
        panic!("the module path names a module of each crate");
    };
    let paths: Vec<&str> = ids
        .iter()
        .map(|id| ws.item(*id).map_or("", |item| item.path.as_str()))
        .collect();
    assert_eq!(paths, ["demo::test_crate::it", "demo::test_crate::it"]);
    // A module that only the library has reads.
    assert_eq!(
        from(&ws, "demo::test_crate::lib_only", "self::y"),
        Ok("demo::test_crate::lib_only::y".to_owned())
    );
}
