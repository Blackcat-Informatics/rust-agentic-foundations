// Review probes: resolve_from across adversarial target layouts.
#![allow(missing_docs, clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]

use raf_source::{MemoryTree, ResolveError, Workspace};

fn load(manifest: &str, files: &[(&str, &str)]) -> Workspace {
    let mut all = vec![("Cargo.toml", manifest)];
    all.extend_from_slice(files);
    let tree: MemoryTree = all.into_iter().collect();
    Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"))
}

const M: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

fn show(ws: &Workspace, module: &str, path: &str) -> String {
    match ws.resolve_from(module, path) {
        Ok(r) => {
            let sites: Vec<String> = r
                .sites
                .iter()
                .map(|id| {
                    let i = ws.item(*id).unwrap();
                    format!("{}@{}", i.path, i.file)
                })
                .collect();
            format!("OK {} {:?}", r.path, sites)
        }
        Err(ResolveError::Ambiguous(ids)) => {
            let sites: Vec<String> = ids
                .iter()
                .map(|id| {
                    let i = ws.item(*id).unwrap();
                    format!("{}@{}", i.path, i.file)
                })
                .collect();
            format!("AMB {sites:?}")
        }
        Err(e) => format!("ERR {e:?}"),
    }
}

fn p(ws: &Workspace, module: &str, path: &str) {
    println!("PROBE {module} | {path} => {}", show(ws, module, path));
}

fn mods(ws: &Workspace) {
    let mut v: Vec<String> = ws
        .items()
        .filter(|i| i.kind == raf_source::SymbolKind::Mod)
        .map(|i| format!("{}@{}", i.path, i.file))
        .collect();
    v.sort();
    for m in v {
        println!("MOD {m}");
    }
}

#[test]
fn zz_l1_support_by_path_and_mod() {
    println!("== L1");
    let ws = load(
        M,
        &[
            ("src/lib.rs", "pub fn api() {}\n"),
            (
                "tests/a.rs",
                "#[path = \"common/util.rs\"]\nmod util;\n#[test]\nfn one() { util::helper() }\n",
            ),
            ("tests/b.rs", "mod common;\n#[test]\nfn two() { common::util::helper() }\n"),
            ("tests/common/mod.rs", "pub mod util;\n"),
            ("tests/common/util.rs", "pub fn helper() { crate::marker() }\n"),
        ],
    );
    mods(&ws);
    p(&ws, "demo::test_crate::a::util", "helper");
    p(&ws, "demo::test_crate::b::common::util", "helper");
    p(&ws, "demo::test_crate::a", "util::helper");
    p(&ws, "demo::test_crate::b", "common::util::helper");
    p(&ws, "demo::test_crate::a", "common::util::helper");
    p(&ws, "demo::test_crate::common::util", "helper");
    p(&ws, "demo::test_crate::a::util", "super::one");
    p(&ws, "demo::test_crate::b::common::util", "super::super::two");
    p(&ws, "demo::test_crate::a::util", "demo::api");
}

#[test]
fn zz_l2_bin_main_nested() {
    println!("== L2");
    let ws = load(
        M,
        &[
            (
                "src/main.rs",
                "mod a { pub mod b { pub mod c { pub fn f() {} } pub fn g() {} } }\nmod x;\nfn main() {}\n",
            ),
            ("src/x.rs", "pub mod y;\npub fn top() {}\n"),
            ("src/x/y.rs", "pub fn deep() { super::top() }\n"),
        ],
    );
    mods(&ws);
    p(&ws, "demo::bin_crate::demo::a::b::c", "self::f");
    p(&ws, "demo::bin_crate::demo::a::b::c", "super::g");
    p(&ws, "demo::bin_crate::demo::a::b::c", "crate::x::y::deep");
    p(&ws, "demo::bin_crate::demo::x::y", "super::top");
    p(&ws, "demo::bin_crate::demo::x::y", "crate::main");
    p(&ws, "demo::bin_crate::demo", "a::b::c::f");
    p(&ws, "demo", "x");
    p(&ws, "demo::x", "top");
}

#[test]
fn zz_l3_two_bins_same_module_name() {
    println!("== L3");
    let ws = load(
        M,
        &[
            ("src/bin/one/main.rs", "mod cli;\nfn main() {}\nfn only_one() {}\n"),
            ("src/bin/one/cli.rs", "pub fn run() { crate::only_one() }\n"),
            ("src/bin/two/main.rs", "mod cli;\nfn main() {}\n"),
            ("src/bin/two/cli.rs", "pub fn run() {}\npub fn two_only() {}\n"),
            (
                "src/bin/three.rs",
                "#[path = \"one/cli.rs\"]\nmod cli;\nfn main() {}\n",
            ),
        ],
    );
    mods(&ws);
    p(&ws, "demo::bin_crate::one::cli", "run");
    p(&ws, "demo::bin_crate::two::cli", "run");
    p(&ws, "demo::bin_crate::three::cli", "run");
    p(&ws, "demo::bin_crate::one::cli", "two_only");
    p(&ws, "demo::bin_crate::two::cli", "two_only");
    p(&ws, "demo::bin_crate::one::cli", "crate::only_one");
    p(&ws, "demo::bin_crate::three::cli", "crate::only_one");
    p(&ws, "demo::bin_crate::one", "cli::run");
    p(&ws, "demo::bin_crate::cli", "run");
}

#[test]
fn zz_l4_example_path_mod() {
    println!("== L4");
    let ws = load(
        M,
        &[
            ("src/lib.rs", "pub fn api() {}\n"),
            (
                "examples/ex.rs",
                "#[path = \"shared/x.rs\"]\nmod x;\nfn main() { x::f() }\n",
            ),
            ("examples/shared/x.rs", "pub fn f() { demo::api() }\nmod inner { }\n"),
            ("examples/multi/main.rs", "mod part;\nfn main() {}\n"),
            ("examples/multi/part.rs", "pub fn piece() {}\n"),
        ],
    );
    mods(&ws);
    p(&ws, "demo::example_crate::ex::x", "f");
    p(&ws, "demo::example_crate::ex::x", "demo::api");
    p(&ws, "demo::example_crate::ex::x::inner", "super::f");
    p(&ws, "demo::example_crate::ex", "x::f");
    p(&ws, "demo::example_crate::multi::part", "piece");
    p(&ws, "demo::example_crate::multi", "part::piece");
    p(&ws, "demo::example_crate::shared::x", "f");
}

#[test]
fn zz_l5_build_scripts() {
    println!("== L5");
    let ws = load(
        M,
        &[
            ("src/lib.rs", "pub fn api() {}\n"),
            ("build.rs", "mod gen;\nfn main() { gen::emit() }\n"),
            ("gen.rs", "pub fn emit() {}\npub mod deeper { pub fn d() { super::emit() } }\n"),
        ],
    );
    mods(&ws);
    p(&ws, "demo::build_script::gen", "emit");
    p(&ws, "demo::build_script::gen::deeper", "super::emit");
    p(&ws, "demo::build_script", "gen::deeper::d");
    p(&ws, "demo::build_script::gen", "demo::api");
    let manifest = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"tools/b.rs\"\n";
    let ws = load(
        manifest,
        &[
            ("src/lib.rs", "pub fn api() {}\n"),
            ("tools/b.rs", "mod helper;\nfn main() {}\n"),
            ("tools/helper.rs", "pub fn h() {}\n"),
        ],
    );
    mods(&ws);
    p(&ws, "demo::build_script::helper", "h");
    p(&ws, "demo::build_script", "helper::h");
}

#[test]
fn zz_l6_test_named_like_lib_module() {
    println!("== L6");
    let ws = load(
        M,
        &[
            (
                "src/lib.rs",
                "pub mod it { pub fn x() {} }\npub mod test_crate { pub mod it { pub mod inner { pub fn z() {} } pub mod libonly { pub fn w() {} } } }\n",
            ),
            ("tests/it.rs", "pub fn x() {}\npub mod inner { pub fn z() {} }\n"),
            ("tests/my-test.rs", "pub fn hy() {}\n"),
            ("tests/dir/main.rs", "mod sub;\n"),
            ("tests/dir/sub.rs", "pub fn s() {}\n"),
        ],
    );
    mods(&ws);
    p(&ws, "demo::it", "self::x");
    p(&ws, "demo::test_crate::it", "self::x");
    p(&ws, "demo::test_crate::it::inner", "self::z");
    p(&ws, "demo::test_crate::it::libonly", "self::w");
    p(&ws, "demo::test_crate::my_test", "self::hy");
    p(&ws, "demo::test_crate::my-test", "self::hy");
    p(&ws, "demo::test_crate::dir::sub", "s");
    p(&ws, "demo::test_crate::dir", "sub::s");
}

#[test]
fn zz_l7_two_packages_same_target_names() {
    println!("== L7");
    let root = "[workspace]\nmembers = [\"a\", \"b\"]\nresolver = \"2\"\n";
    let ma = "[package]\nname = \"a\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
    let mb = "[package]\nname = \"b\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[dev-dependencies]\na = { path = \"../a\" }\n";
    let tree: MemoryTree = [
        ("Cargo.toml", root),
        ("a/Cargo.toml", ma),
        ("a/src/lib.rs", "pub fn fa() {}\n"),
        ("a/tests/it.rs", "fn h() {}\n"),
        ("b/Cargo.toml", mb),
        ("b/src/lib.rs", "pub fn fb() {}\n"),
        ("b/tests/it.rs", "fn h() {}\n"),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"));
    mods(&ws);
    p(&ws, "a::test_crate::it", "h");
    p(&ws, "b::test_crate::it", "h");
    p(&ws, "b::test_crate::it", "a::fa");
    p(&ws, "a::test_crate::it", "b::fb");
    p(&ws, "b", "a::fa");
}
