// Review probes: impls() shapes and reachability semantics.
#![allow(missing_docs, clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]

use raf_source::{MemoryTree, Namespace, Node, Reachability, Workspace};

fn load_ed(edition: &str, files: &[(&str, &str)]) -> Result<Workspace, String> {
    let manifest =
        format!("[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"{edition}\"\n");
    let mut all = vec![("Cargo.toml", manifest.as_str())];
    all.extend_from_slice(files);
    let tree: MemoryTree = all.into_iter().collect();
    Workspace::load(&tree, &["."]).map_err(|e| format!("{e}"))
}

fn lib(text: &str) -> Workspace {
    load_ed("2021", &[("src/lib.rs", text)]).unwrap_or_else(|e| panic!("{e}"))
}

fn node(ws: &Workspace, path: &str) -> Node {
    for ns in [Namespace::Value, Namespace::Type, Namespace::Macro] {
        if let Ok(r) = ws.resolve_in(path, ns) {
            return Node::Item(r.sites[0]);
        }
    }
    panic!("no {path}")
}

fn name(ws: &Workspace, n: Node) -> String {
    match n {
        Node::Item(id) => {
            let i = ws.item(id).unwrap();
            format!("{}({:?})", i.path, i.kind)
        }
        Node::Impl(id) => {
            let b = ws.impl_block(id).unwrap();
            format!("impl {}:{}", b.self_type, b.line)
        }
    }
}

fn show(ws: &Workspace, r: &Reachability) -> String {
    r.reached
        .iter()
        .map(|x| {
            format!(
                "{} <{:?} from {}>",
                name(ws, x.node),
                x.edge,
                x.from.map(|f| name(ws, f)).unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ")
}

fn go(label: &str, ws: &Workspace, roots: &[&str]) {
    let nodes: Vec<Node> = roots.iter().map(|p| node(ws, p)).collect();
    match ws.reachable(&nodes) {
        Ok(r) => println!("REACH {label}:\n    {}", show(ws, &r)),
        Err(e) => println!("REACH {label}: ERR {e}"),
    }
}

fn impls(label: &str, ws: &Workspace) {
    println!("IMPLS {label}:");
    for b in ws.impls() {
        let keys: Vec<String> = b
            .self_items
            .iter()
            .map(|id| ws.item(*id).unwrap().path.to_string())
            .collect();
        let members: Vec<String> = b
            .members
            .iter()
            .map(|id| ws.item(*id).unwrap().path.to_string())
            .collect();
        println!(
            "    #{} {}:{} lines {:?} bytes {}..{} mod {} self {} trait {:?} test {:?} keys {:?} members {:?}",
            b.id.index(),
            b.file,
            b.line,
            b.span.lines,
            b.span.start_byte,
            b.span.end_byte,
            b.module,
            b.self_type,
            b.trait_path.as_deref(),
            b.test,
            keys,
            members
        );
    }
}

#[test]
fn zz_impl_shapes() {
    let ws = lib("pub struct S;\n\
        pub struct G<T>(T);\n\
        pub type A = S;\n\
        pub trait Tr { fn t(&self) {} }\n\
        unsafe impl Send for S {}\n\
        impl !Sync for G<u8> {}\n\
        impl<T> Tr for G<T> {}\n\
        impl A { pub fn via_alias() {} }\n\
        impl crate::S { pub fn via_crate() {} }\n\
        impl dyn Tr { pub fn on_dyn(&self) {} }\n\
        impl Tr for [S] {}\n\
        impl Tr for (S, S) {}\n\
        impl Tr for fn() {}\n\
        pub fn body() { struct L; impl L { fn local() {} } impl Tr for L {} }\n\
        const _: () = { impl S { pub fn in_const() {} } };\n\
        macro_rules! gen { ($t:ident) => { impl $t { pub fn generated() {} } } }\n\
        gen!(S);\n\
        #[cfg(any())] mod off { impl super::S { pub fn cfg_off() {} } }\n\
        #[cfg(feature = \"x\")] impl S { pub fn cfg_feature() {} }\n\
        #[cfg(not(feature = \"x\"))] impl S { pub fn cfg_feature() {} }\n\
        /// Documented.\n#[allow(dead_code)]\nimpl S { const C: u8 = 1; type X = u8; fn m() {} }\n\
        mod inner { impl super::S { pub fn from_inner() {} } impl Self {} }\n");
    impls("shapes", &ws);
    go("S", &ws, &["demo::S"]);
    go("body", &ws, &["demo::body"]);
}

#[test]
fn zz_reach_variants_fields_and_traits() {
    let ws = lib("pub enum E { V(Helper), W { f: Other } }\n\
        struct Helper;\nstruct Other;\n\
        pub struct S { pub f: Field }\nstruct Field;\n\
        pub trait Tr { fn d(&self) { from_default() } type X; const K: Kc; }\n\
        fn from_default() {}\nstruct Kc;\n\
        pub union U { a: UField }\n#[derive(Clone, Copy)] struct UField;\n");
    go("E", &ws, &["demo::E"]);
    go("S", &ws, &["demo::S"]);
    go("Tr", &ws, &["demo::Tr"]);
    go("U", &ws, &["demo::U"]);
}

#[test]
fn zz_reach_approximations() {
    // a local sharing an item's name
    let ws = lib("pub fn f() { let helper = 1; let _ = helper; }\nfn helper() {}\n\
        pub fn g(shadow: u8) -> u8 { shadow }\nfn shadow() {}\n\
        pub fn h() { struct Local; let _ = Local; }\nstruct Local;\n");
    go("local", &ws, &["demo::f"]);
    go("param", &ws, &["demo::g"]);
    go("local-item-shadow", &ws, &["demo::h"]);
    // macro bodies from the macro's module
    let ws = lib("mod m { macro_rules! mk { () => { helper() } } pub(crate) use mk; pub fn helper() {} }\n\
        mod n { pub fn helper() {} pub fn caller() { crate::m::mk!() } }\n\
        pub use n::caller;\n");
    go("macro-home", &ws, &["demo::n::caller"]);
    // macro_export from crate root
    let ws = lib("mod m { #[macro_export] macro_rules! ex { () => { $crate::target() } } }\n\
        pub fn target() {}\npub fn call() { ex!() }\n");
    go("macro-export", &ws, &["demo::call"]);
    // use inside a fn body
    let ws = lib("pub mod a { pub fn f() { use crate::b::g; g() } }\npub mod b { pub fn g() {} }\n");
    go("use-in-body-2021", &ws, &["demo::a::f"]);
    // nested fn items in bodies
    let ws = lib("pub fn outer() { fn inner() { deep() } inner() }\nfn deep() {}\nfn inner() {}\n");
    go("nested-fn", &ws, &["demo::outer"]);
    // generic named like an item
    let ws = lib("pub fn gen<T>(x: T) -> T { x }\npub struct T;\n");
    go("generic-T", &ws, &["demo::gen"]);
    // a method call on a reached type: S::new reaches S; x.go() only via S
    let ws = lib("pub fn f() { let s = S::new(); s.go(); }\npub struct S;\nimpl S { pub fn new() -> S { S } pub fn go(&self) { inner() } }\nfn inner() {}\n");
    go("method", &ws, &["demo::f"]);
    // a struct literal field shorthand
    let ws = lib("pub fn f(x: u8) -> P { let helper = x; P { helper } }\npub struct P { helper: u8 }\nfn helper() {}\n");
    go("shorthand", &ws, &["demo::f"]);
    // turbofish / qualified paths
    let ws = lib("pub fn f() { let _ = <S as Tr>::k(); let _ = Vec::<Inner>::new(); }\npub struct S; pub trait Tr { fn k(); } impl Tr for S { fn k() {} }\nstruct Inner;\n");
    go("qualified", &ws, &["demo::f"]);
    // attribute and doc paths
    let ws = lib("#[doc = \"x\"]\n/// Uses [`linked`].\n#[cfg_attr(feature = \"x\", derive(Helper))]\npub fn f() {}\nfn linked() {}\nstruct Helper;\n");
    go("attrs", &ws, &["demo::f"]);
}

#[test]
fn zz_reach_edition_2015() {
    let ws = load_ed(
        "2015",
        &[(
            "src/lib.rs",
            "pub mod a { pub fn f() { ::b::g(); } pub fn h() { b::g(); } pub fn k() { use b::g; g(); } pub fn s() { self::sib(); } fn sib() {} }\npub mod b { pub fn g() {} }\n",
        )],
    )
    .unwrap();
    go("2015 ::b::g", &ws, &["demo::a::f"]);
    go("2015 b::g from a (rustc: error, not in scope)", &ws, &["demo::a::h"]);
    go("2015 use b::g in body (rustc: crate::b::g)", &ws, &["demo::a::k"]);
    go("2015 self::sib", &ws, &["demo::a::s"]);
    let ws = load_ed(
        "2018",
        &[(
            "src/lib.rs",
            "pub mod a { pub fn f() { ::b::g(); } pub fn k() { use b::g; g(); } }\npub mod b { pub fn g() {} }\n",
        )],
    )
    .unwrap();
    go("2018 ::b::g (extern crate b)", &ws, &["demo::a::f"]);
    go("2018 use b::g in a (relative, b not in a)", &ws, &["demo::a::k"]);
}

#[test]
fn zz_reach_cfg_modules_and_tests() {
    let ws = lib("pub fn api() {}\n#[cfg(test)] mod tests { use super::*; #[test] fn t() { api(); helper() } fn helper() {} }\n\
        #[cfg(any())] pub mod off { pub fn o() { super::api() } }\n");
    go("off-module", &ws, &["demo::off::o"]);
    go("tests-module", &ws, &["demo::tests"]);
}

#[test]
fn zz_reach_determinism_and_order() {
    let ws = lib("pub fn z() { b() }\npub fn a() { c() }\nfn b() { d() }\nfn c() {}\nfn d() {}\n");
    let r1 = ws.reachable(&[node(&ws, "demo::z"), node(&ws, "demo::a")]).unwrap();
    let r2 = ws.reachable(&[node(&ws, "demo::a"), node(&ws, "demo::z")]).unwrap();
    assert_eq!(r1, r2);
    println!("ORDER:\n    {}", show(&ws, &r1));
}

#[test]
fn zz_scanner_edges() {
    let ws = lib("pub fn f(p: *const Helper, q: *mut Other) {}\nstruct Helper;\nstruct Other;\n");
    go("*const Helper / *mut Other", &ws, &["demo::f"]);
    let ws = lib("pub fn f() { gen(); }\nfn gen() {}\n");
    go("gen() in 2021 (not a keyword before 2024)", &ws, &["demo::f"]);
    let ws = lib("pub fn f() { r#gen(); }
fn gen() {}
");
    go("r#gen() in 2021", &ws, &["demo::f"]);
    let ws = lib("pub struct S;\nimpl S { pub fn api() { Self::helper() } fn helper() { deep() } }\nfn deep() {}\n");
    go("Self::helper from bound method", &ws, &["demo::S::api"]);
    let ws = lib("pub fn f<T>() where Helper: Clone, T: Into<Other> {}\n#[derive(Clone)] struct Helper;\nstruct Other;\n");
    go("where Helper: Clone", &ws, &["demo::f"]);
    let ws = lib("pub fn f() { let r = &raw const VALUE; }\nstatic VALUE: u8 = 1;\n");
    go("&raw const VALUE", &ws, &["demo::f"]);
}

#[test]
fn zz_impl_test_roles() {
    let ws = load_ed("2021", &[
        ("src/lib.rs", "pub struct S;\n#[cfg(test)] mod tests { impl super::S { fn t() {} } }\n"),
        ("tests/it.rs", "mod support;\npub struct T;\nimpl T { fn x() {} }\n"),
        ("tests/support/mod.rs", "pub struct U;\nimpl U { pub fn y() {} }\n"),
    ]).unwrap();
    impls("roles", &ws);
    for i in ws.items() {
        println!("    ITEM {} {:?} {:?}", i.path, i.kind, i.test);
    }
}
