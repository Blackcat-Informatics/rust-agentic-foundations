// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `Workspace::reachable`: the items and impl blocks that root items reach
//! over the edges contains, impl-of and references.

use raf_source::{
    Edge, ImplId, ItemId, MemoryTree, Namespace, Node, ReachError, Reachability, Workspace,
};

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

fn load(files: &[(&str, &str)]) -> Workspace {
    let mut all = vec![("Cargo.toml", MANIFEST)];
    all.extend_from_slice(files);
    let tree: MemoryTree = all.into_iter().collect();
    Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"))
}

fn lib(text: &str) -> Workspace {
    load(&[("src/lib.rs", text)])
}

/// The item at a canonical path in one name space.
fn site(ws: &Workspace, path: &str, ns: Namespace) -> ItemId {
    ws.resolve_in(path, ns)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .sites[0]
}

fn value(ws: &Workspace, path: &str) -> Node {
    Node::Item(site(ws, path, Namespace::Value))
}

fn ty(ws: &Workspace, path: &str) -> Node {
    Node::Item(site(ws, path, Namespace::Type))
}

fn reach(ws: &Workspace, roots: &[Node]) -> Reachability {
    ws.reachable(roots).unwrap_or_else(|e| panic!("{e}"))
}

/// The reached nodes as text: an item's path, or `impl <self type>` with the
/// line of a block, in the order of the walk.
fn names(ws: &Workspace, reached: &Reachability) -> Vec<String> {
    reached
        .reached
        .iter()
        .map(|found| name(ws, found.node))
        .collect()
}

fn name(ws: &Workspace, node: Node) -> String {
    match node {
        Node::Item(id) => ws.item(id).expect("an item").path.to_string(),
        Node::Impl(id) => {
            let block = ws.impl_block(id).expect("a block");
            format!("impl {}:{}", block.self_type, block.line)
        }
    }
}

/// The names of the reached nodes, sorted.
fn sorted(ws: &Workspace, reached: &Reachability) -> Vec<String> {
    let mut all = names(ws, reached);
    all.sort();
    all
}

#[test]
fn a_bound_type_covers_its_impl_blocks_and_their_members_and_a_bound_method_nothing_beside_it() {
    let ws = lib("pub struct S;\n\
         impl S { pub fn new() -> u8 { 0 } pub fn other() -> u8 { 1 } }\n\
         impl Clone for S { fn clone(&self) -> S { S } }\n\
         pub struct Apart;\n");
    let reached = reach(&ws, &[ty(&ws, "demo::S")]);
    assert_eq!(
        sorted(&ws, &reached),
        [
            "demo::S",
            "demo::S::clone",
            "demo::S::new",
            "demo::S::other",
            "impl S:2",
            "impl S:3",
        ]
    );
    let block = ws.impls().next().expect("the inherent block").id;
    assert_eq!(
        reached.get(Node::Impl(block)).map(|found| found.edge),
        Some(Edge::ImplOf)
    );
    // The methods are below the type and are first reached as its members.
    let new = value(&ws, "demo::S::new");
    assert_eq!(
        reached.get(new).map(|found| (found.edge, found.from)),
        Some((Edge::Contains, Some(ty(&ws, "demo::S"))))
    );
    assert!(!reached.contains(ty(&ws, "demo::Apart")));
    // A bound method reaches nothing beside it: its tokens name no type.
    let reached = reach(&ws, &[new]);
    assert_eq!(names(&ws, &reached), ["demo::S::new"]);
}

#[test]
fn a_bound_module_covers_its_nested_items_and_impl_blocks_and_a_crate_root_nothing() {
    let ws = lib("pub mod m {\n\
             pub fn a() {}\n\
             struct T;\n\
             impl T { fn t() {} }\n\
             pub mod deeper { pub fn b() {} }\n\
         }\n\
         pub mod outside { pub fn c() {} }\n");
    let reached = reach(&ws, &[ty(&ws, "demo::m")]);
    assert_eq!(
        sorted(&ws, &reached),
        [
            "demo::m",
            "demo::m::T",
            "demo::m::T::t",
            "demo::m::a",
            "demo::m::deeper",
            "demo::m::deeper::b",
            "impl T:4",
        ]
    );
    let root = ty(&ws, "demo");
    assert_eq!(names(&ws, &reach(&ws, &[root])), ["demo"]);
}

#[test]
fn a_private_helper_called_by_a_bound_item_is_reached_and_an_unreferenced_one_is_not() {
    let ws = lib("pub fn api() -> u8 { helper() + inner::deep() }\n\
         fn helper() -> u8 { 1 }\n\
         fn unused() -> u8 { 2 }\n\
         mod inner { pub fn deep() -> u8 { super::chained() } pub fn beside() {} }\n\
         fn chained() -> u8 { 3 }\n");
    let reached = reach(&ws, &[value(&ws, "demo::api")]);
    assert_eq!(
        sorted(&ws, &reached),
        [
            "demo::api",
            "demo::chained",
            "demo::helper",
            "demo::inner",
            "demo::inner::deep",
        ]
    );
    let helper = value(&ws, "demo::helper");
    assert_eq!(
        reached.get(helper).map(|found| (found.edge, found.from)),
        Some((Edge::References, Some(value(&ws, "demo::api"))))
    );
    assert!(!reached.contains(value(&ws, "demo::unused")));
    // A module that a path names reaches nothing below it.
    assert!(reached.contains(ty(&ws, "demo::inner")));
    assert!(!reached.contains(value(&ws, "demo::inner::beside")));
}

#[test]
fn a_method_call_a_field_a_lifetime_and_a_local_name_no_item_and_a_path_prefix_names_its_type() {
    let ws = lib("pub struct S { helper: u8 }\n\
         impl S { pub fn helper(&self) -> u8 { self.helper } }\n\
         pub enum E { V }\n\
         fn helper() {}\n\
         fn label() {}\n\
         pub fn api<'label>(s: &'label S) -> u8 { let label = 0; let _ = E::V; s.helper() + label }\n");
    let reached = reach(&ws, &[value(&ws, "demo::api")]);
    // `E::V` names the enum through its prefix, `S` the struct with its
    // block, and the method call, the field, the lifetime and the local
    // name nothing: `label` is a local, which the index reads as the
    // function, and that is the one name a local shadows.
    assert_eq!(
        sorted(&ws, &reached),
        [
            "demo::E",
            "demo::S",
            "demo::S::helper",
            "demo::api",
            "demo::label",
            "impl S:2",
        ]
    );
    assert!(!reached.contains(value(&ws, "demo::helper")));
}

#[test]
fn a_path_through_a_type_names_the_type_and_the_member_and_a_leading_colon_names_a_crate() {
    let ws = load(&[
        (
            "src/lib.rs",
            "pub struct S;\nimpl S { pub fn make() {} }\npub fn api() {}\n",
        ),
        (
            "tests/it.rs",
            "mod demo { pub fn api() {} }\n#[test]\nfn case() { ::demo::api(); ::demo::S::make(); }\n",
        ),
    ]);
    let case = value(&ws, "demo::test_crate::it::case");
    let reached = reach(&ws, &[case]);
    assert_eq!(
        sorted(&ws, &reached),
        [
            "demo::S",
            "demo::S::make",
            "demo::api",
            "demo::test_crate::it::case",
            "impl S:2",
        ]
    );
}

#[test]
fn paths_inside_tests_and_binaries_reach_their_own_crate_and_the_library() {
    let ws = load(&[
        ("src/lib.rs", "pub fn api() {}\npub fn untouched() {}\n"),
        (
            "src/main.rs",
            "mod cli { pub fn run() { crate::helper(); demo::api(); } }\nfn helper() {}\nfn main() { cli::run() }\n",
        ),
        (
            "tests/a.rs",
            "mod support;\n#[test]\nfn one() { support::helper(); local() }\nfn local() {}\n",
        ),
        ("tests/b.rs", "mod support;\n#[test]\nfn two() {}\n"),
        (
            "tests/support/mod.rs",
            "pub fn helper() { crate::local() }\n",
        ),
    ]);
    let main = value(&ws, "demo::bin_crate::demo::main");
    assert_eq!(
        sorted(&ws, &reach(&ws, &[main])),
        [
            "demo::api",
            "demo::bin_crate::demo::cli",
            "demo::bin_crate::demo::cli::run",
            "demo::bin_crate::demo::helper",
            "demo::bin_crate::demo::main",
        ]
    );
    // The support module of target `a` is reached from `a`, and its copy in
    // `b` is not, though the two are one source item.
    let one = value(&ws, "demo::test_crate::a::one");
    let reached = reach(&ws, &[one]);
    assert_eq!(
        sorted(&ws, &reached),
        [
            "demo::test_crate::a::local",
            "demo::test_crate::a::one",
            "demo::test_crate::a::support",
            "demo::test_crate::a::support::helper",
        ]
    );
    let in_b = value(&ws, "demo::test_crate::b::support::helper");
    assert!(!reached.contains(in_b));
}

#[test]
fn an_item_a_macro_generates_is_not_reached_and_a_macro_named_by_a_reached_item_is() {
    let ws = lib(
        "macro_rules! make { ($n:ident) => { fn $n() { $crate::inside() } } }\n\
         make!(generated);\n\
         fn inside() {}\n\
         pub fn calls_generated() { generated() }\n\
         pub fn invokes() { make!(local); }\n\
         macro_rules! around { () => { lonely() } }\n\
         pub mod holds { around!(); const _: () = { super::unnamed(); }; }\n\
         fn lonely() {}\n\
         fn unnamed() {}\n\
         fn helper() {}\n\
         mod inner { fn helper() {} #[macro_export] macro_rules! exported { () => { helper() } } }\n",
    );
    // A path to an item that a macro generates names nothing of the index.
    let reached = reach(&ws, &[value(&ws, "demo::calls_generated")]);
    assert_eq!(names(&ws, &reached), ["demo::calls_generated"]);
    // A macro that a reached item invokes by name is reached, and the paths
    // of its body are read from its module.
    let make = Node::Item(site(&ws, "demo::make", Namespace::Macro));
    let reached = reach(&ws, &[value(&ws, "demo::invokes")]);
    assert_eq!(
        sorted(&ws, &reached),
        ["demo::inside", "demo::invokes", "demo::make"]
    );
    assert_eq!(
        reached.get(make).map(|found| found.edge),
        Some(Edge::References)
    );
    // A macro invocation and a `const _` at module level are no items: a
    // bound module does not reach what they name.
    let reached = reach(&ws, &[ty(&ws, "demo::holds")]);
    assert_eq!(names(&ws, &reached), ["demo::holds"]);
    // A `#[macro_export]` macro is read from the crate root, where it is
    // indexed, not from the module that holds its text.
    let exported = Node::Item(site(&ws, "demo::exported", Namespace::Macro));
    assert_eq!(
        sorted(&ws, &reach(&ws, &[exported])),
        ["demo::exported", "demo::helper"]
    );
}

#[test]
fn a_cfg_test_module_is_reached_like_any_module_and_its_tests_reach_their_helpers() {
    let ws = lib("pub fn api() {}\n\
         fn helper() {}\n\
         #[cfg(test)]\n\
         mod tests {\n\
             use super::*;\n\
             #[test]\n\
             fn checks() { helper() }\n\
             #[test]\n\
             fn other() {}\n\
             fn unused_test_helper() {}\n\
         }\n");
    // The library's bound item does not reach the test module.
    let reached = reach(&ws, &[value(&ws, "demo::api")]);
    assert_eq!(names(&ws, &reached), ["demo::api"]);
    // A bound test reaches the helper it calls through the glob of its
    // module, and nothing else of its module.
    let reached = reach(&ws, &[value(&ws, "demo::tests::checks")]);
    assert_eq!(
        sorted(&ws, &reached),
        ["demo::helper", "demo::tests::checks"]
    );
    // The test module bound whole covers each of its items.
    let reached = reach(&ws, &[ty(&ws, "demo::tests")]);
    assert_eq!(
        sorted(&ws, &reached),
        [
            "demo::helper",
            "demo::tests",
            "demo::tests::checks",
            "demo::tests::other",
            "demo::tests::unused_test_helper",
        ]
    );
}

#[test]
fn a_module_named_by_a_path_and_then_reached_by_contains_is_opened_then() {
    let ws = lib("pub mod outer {\n\
             pub fn first() { super::outer::inner::x(); }\n\
             pub mod inner { pub fn x() {} pub fn y() {} }\n\
         }\n");
    let outer = ty(&ws, "demo::outer");
    let reached = reach(&ws, &[outer]);
    // `first` names `inner` before the walk reaches it as an item of
    // `outer`; it is opened all the same, and keeps the first edge.
    assert!(reached.contains(value(&ws, "demo::outer::inner::y")));
    let inner = ty(&ws, "demo::outer::inner");
    assert_eq!(
        reached.get(inner).map(|found| found.edge),
        Some(Edge::Contains)
    );
    // Named alone, the module reaches nothing below it.
    let reached = reach(&ws, &[value(&ws, "demo::outer::first")]);
    assert!(reached.contains(inner));
    assert!(reached.contains(value(&ws, "demo::outer::inner::x")));
    assert!(!reached.contains(value(&ws, "demo::outer::inner::y")));
}

#[test]
fn the_answer_does_not_depend_on_the_order_of_the_roots_and_is_the_same_each_run() {
    let ws = lib("pub fn a() { c() }\npub fn b() { c() }\nfn c() {}\n");
    let (a, b) = (value(&ws, "demo::a"), value(&ws, "demo::b"));
    let forward = reach(&ws, &[a, b]);
    let backward = reach(&ws, &[b, a, b]);
    assert_eq!(forward, backward);
    assert_eq!(forward, reach(&ws, &[a, b]));
    assert_eq!(names(&ws, &forward), ["demo::a", "demo::b", "demo::c"]);
    assert_eq!(
        forward.get(a).map(|found| (found.edge, found.from)),
        Some((Edge::Root, None))
    );
    assert_eq!(forward.items().count(), 3);
    assert_eq!(forward.impls().count(), 0);
    // No roots reach nothing.
    assert_eq!(reach(&ws, &[]).reached, []);
}

#[test]
fn a_root_of_another_workspace_is_refused_beside_one_of_this_workspace() {
    let small = lib("pub fn a() {}\n");
    let big =
        lib("pub struct S;\nimpl S {}\nimpl S {}\npub fn a() {} pub fn b() {} pub fn c() {}\n");
    let far = big.items().map(|item| item.id).max().expect("items");
    assert!(matches!(
        small.reachable(&[Node::Item(far)]),
        Err(ReachError::UnknownItem(id)) if id == far
    ));
    let block: ImplId = big.impls().last().expect("a block").id;
    let error = small
        .reachable(&[Node::Impl(block)])
        .expect_err("no block here");
    assert!(matches!(error, ReachError::UnknownImpl(id) if id == block));
    assert!(error.to_string().contains("not an impl block"), "{error}");
    assert!(small.reachable(&[value(&small, "demo::a")]).is_ok());
}

#[test]
fn an_impl_block_root_reaches_its_members_and_what_its_header_names() {
    let ws = lib("pub trait Tr { fn shown(&self) {} }\n\
         pub struct S;\n\
         impl Tr for S {}\n\
         impl S { fn own() {} }\n\
         impl Tr for String { fn shown(&self) { helper() } }\n\
         fn helper() {}\n");
    let block = ws.impls().next().expect("the trait block").id;
    let reached = reach(&ws, &[Node::Impl(block)]);
    assert_eq!(
        sorted(&ws, &reached),
        [
            "demo::S",
            "demo::S::own",
            "demo::Tr",
            "demo::Tr::shown",
            "impl S:3",
            "impl S:4",
        ]
    );
    // A block on a type of a dependency is placed under no type: its members
    // are reached through the block alone.
    let foreign = ws.impls().last().expect("the block on String").id;
    let reached = reach(&ws, &[Node::Impl(foreign)]);
    assert_eq!(
        sorted(&ws, &reached),
        [
            "demo::Tr",
            "demo::Tr::shown",
            "demo::Tr::shown",
            "demo::helper",
            "impl String:5",
        ]
    );
    let member = ws.impl_block(foreign).expect("the block").members[0];
    assert_eq!(
        reached.get(Node::Item(member)).map(|found| found.edge),
        Some(Edge::ImplOf)
    );
}

#[test]
fn a_file_with_a_byte_order_mark_and_crlf_lines_reaches_as_a_plain_one_does() {
    let plain = lib("pub fn api() { helper() }\nfn helper() {}\nfn unused() {}\n");
    let marked = lib("\u{feff}pub fn api() { helper() }\r\nfn helper() {}\r\nfn unused() {}\r\n");
    for ws in [&plain, &marked] {
        let reached = reach(ws, &[value(ws, "demo::api")]);
        assert_eq!(names(ws, &reached), ["demo::api", "demo::helper"]);
    }
}

#[test]
fn a_member_is_read_from_the_module_of_its_block_and_a_trait_member_from_its_trait() {
    let ws = lib("pub struct S;\n\
         fn helper() {}\n\
         mod place { fn helper() {} impl super::S { pub fn go() { helper() } } }\n\
         mod traits { fn inner() {} pub trait T { fn provided() { inner() } } }\n");
    assert_eq!(
        sorted(&ws, &reach(&ws, &[ty(&ws, "demo::S")])),
        [
            "demo::S",
            "demo::S::go",
            "demo::place::helper",
            "impl super::S:3"
        ]
    );
    assert_eq!(
        sorted(&ws, &reach(&ws, &[ty(&ws, "demo::traits::T")])),
        [
            "demo::traits::T",
            "demo::traits::T::provided",
            "demo::traits::inner"
        ]
    );
}

#[test]
fn a_path_the_resolver_refuses_reaches_nothing_beside_one_it_resolves() {
    let ws = lib("mod b { pub fn helper() {} }\n\
         mod broken { pub use self::gone::*; pub use crate::b::*; }\n\
         mod whole { pub use crate::b::*; }\n\
         mod x { pub fn h() {} }\n\
         mod y { pub fn h() {} }\n\
         mod both { pub use crate::x::*; pub use crate::y::*; }\n\
         pub fn refused() { broken::helper(); both::h(); }\n\
         pub fn resolved() { whole::helper(); x::h(); }\n");
    // A lookup that reads a glob whose path names nothing, and a name two
    // globs bring in as different items, name nothing.
    assert_eq!(
        sorted(&ws, &reach(&ws, &[value(&ws, "demo::refused")])),
        ["demo::both", "demo::broken", "demo::refused"]
    );
    assert_eq!(
        sorted(&ws, &reach(&ws, &[value(&ws, "demo::resolved")])),
        [
            "demo::b::helper",
            "demo::resolved",
            "demo::whole",
            "demo::x",
            "demo::x::h"
        ]
    );
}

#[test]
fn test_code_names_the_dev_dependencies_and_other_code_does_not() {
    let tree: MemoryTree = [
        ("Cargo.toml", "[workspace]\nmembers = [\"a\", \"b\"]\n"),
        (
            "a/Cargo.toml",
            "[package]\nname = \"a\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\
             [dev-dependencies]\nb = { path = \"../b\" }\n",
        ),
        (
            "a/src/lib.rs",
            "pub fn api() { b::thing() }\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() { b::thing() }\n}\n",
        ),
        (
            "b/Cargo.toml",
            "[package]\nname = \"b\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        ("b/src/lib.rs", "pub fn thing() {}\n"),
    ]
    .into_iter()
    .collect();
    let ws = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"));
    let thing = value(&ws, "b::thing");
    assert!(reach(&ws, &[value(&ws, "a::tests::t")]).contains(thing));
    assert!(!reach(&ws, &[value(&ws, "a::api")]).contains(thing));
}
