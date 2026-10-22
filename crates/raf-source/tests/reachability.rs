// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `Workspace::reachable`: the items and impl blocks that root items reach
//! over the edges contains, impl-of and references.

use raf_source::{
    Budget, Edge, ImplId, ItemId, LoadErrorKind, LoadOptions, MemoryTree, Namespace, Node,
    ReachError, Reachability, Workspace,
};

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

#[test]
fn qualified_self_suffixes_are_not_member_edges_beside_an_unqualified_self_member() {
    let ws = lib("pub trait Tr { fn trait_member(); } pub struct S; \
         impl S { pub fn qualified() { <Self as Tr>::trait_member(); <S>::helper(); } \
         pub fn ordinary() { Self::helper(); } fn helper() {} }");
    let helper = value(&ws, "demo::S::helper");
    let member = value(&ws, "demo::Tr::trait_member");
    let qualified = reach(&ws, &[value(&ws, "demo::S::qualified")]);
    for (node, owner) in [(helper, ty(&ws, "demo::S")), (member, ty(&ws, "demo::Tr"))] {
        let found = qualified
            .get(node)
            .expect("type containment reaches member");
        assert_eq!((found.edge, found.from), (Edge::Contains, Some(owner)));
    }
    let ordinary = value(&ws, "demo::S::ordinary");
    let reached = reach(&ws, &[ordinary]);
    let found = reached.get(helper).expect("direct Self member");
    assert_eq!((found.edge, found.from), (Edge::References, Some(ordinary)));
}

#[test]
fn an_edition_2015_body_import_is_read_from_its_module_beside_an_explicit_crate_path() {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='demo'\nversion='0.1.0'\nedition='2015'\n",
        ),
        (
            "src/lib.rs",
            "pub fn helper() {} pub mod child { \
         pub fn implicit() { use helper; helper(); } \
         pub fn explicit() { use crate::helper; helper(); } }",
        ),
    ]);
    let ws = Workspace::load(&tree, &["."]).expect("edition 2015 source");
    let helper = value(&ws, "demo::helper");
    assert!(!reach(&ws, &[value(&ws, "demo::child::implicit")]).contains(helper));
    assert!(reach(&ws, &[value(&ws, "demo::child::explicit")]).contains(helper));
}

#[test]
fn nested_impl_self_is_read_in_the_enclosing_members_context() {
    let ws = lib("pub struct S; impl S { pub fn outer() { struct Inner; \
         impl Inner { fn nested() { Self::helper(); } fn helper() {} } } \
         pub fn unrelated() {} fn helper() {} }");
    let outer = value(&ws, "demo::S::outer");
    let helper = value(&ws, "demo::S::helper");
    let reached = reach(&ws, &[outer]);
    let found = reached.get(helper).expect("nested Self uses outer context");
    assert_eq!((found.edge, found.from), (Edge::References, Some(outer)));
    assert!(!reach(&ws, &[value(&ws, "demo::S::unrelated")]).contains(helper));
    assert_eq!(ws.impls().count(), 1);
}

#[test]
fn impls_inside_const_and_function_blocks_are_tokens_beside_an_indexed_top_level_impl() {
    let ws = lib(
        "pub struct S; const _: () = { impl S { fn anonymous() {} } }; \
         const NAMED: () = { impl S { fn named() {} } }; \
         pub fn f() { impl S { fn local() {} } } \
         impl S { pub fn ordinary() {} }",
    );
    let blocks: Vec<_> = ws.impls().collect();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].members.len(), 1);
    assert_eq!(
        blocks[0].members[0],
        site(&ws, "demo::S::ordinary", Namespace::Value)
    );
    for member in ["anonymous", "named", "local"] {
        assert!(
            ws.resolve_in(&format!("demo::S::{member}"), Namespace::Value)
                .is_err()
        );
    }
    assert!(reach(&ws, &[ty(&ws, "demo::S")]).contains(value(&ws, "demo::S::ordinary")));
}

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
         pub mod kinds { pub enum E { V } }\n\
         fn helper() {}\n\
         fn label() {}\n\
         pub fn api<'label>(s: &'label S) -> u8 { let label = 0; let _ = kinds::E::V; s.helper() + label }\n");
    let reached = reach(&ws, &[value(&ws, "demo::api")]);
    // `kinds::E::V` names the enum through its prefix, `S` the struct with its
    // block, and the method call, the field, the lifetime and the local
    // name nothing: `label` is a local, which the index reads as the
    // function, and that is the one name a local shadows.
    assert_eq!(
        sorted(&ws, &reached),
        [
            "demo::S",
            "demo::S::helper",
            "demo::api",
            "demo::kinds",
            "demo::kinds::E",
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
    // A module that a path reaches first, below one that a root opens only
    // after the path has named both, is opened when contains reaches it.
    let ws = lib(
        "pub mod r { pub mod a { pub mod m { pub mod inner { pub fn x() {} pub fn y() {} } } } }\n\
         pub fn f() { r::a::m::inner::x(); }\n",
    );
    let roots = [ty(&ws, "demo::r"), value(&ws, "demo::f")];
    let reached = reach(&ws, &roots);
    let deep = ty(&ws, "demo::r::a::m::inner");
    assert_eq!(
        reached.get(deep).map(|found| found.edge),
        Some(Edge::References)
    );
    assert!(reached.contains(value(&ws, "demo::r::a::m::inner::y")));
    let ws = lib("pub mod outer {\n\
             pub fn first() { super::outer::inner::x(); }\n\
             pub mod inner { pub fn x() {} pub fn y() {} }\n\
         }\n");
    let inner = ty(&ws, "demo::outer::inner");
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
fn a_root_past_the_items_or_the_blocks_of_the_workspace_is_refused_beside_one_in_range() {
    let small = lib("pub fn a() {}\n");
    let big =
        lib("pub struct S;\nimpl S {}\nimpl S {}\npub fn a() {} pub fn b() {} pub fn c() {}\n");
    let far = big.items().map(|item| item.id).max().expect("items");
    let error = small
        .reachable(&[Node::Item(far)])
        .expect_err("no item there");
    assert!(matches!(error, ReachError::UnknownItem(id) if id == far));
    assert!(error.to_string().contains("past the items"), "{error}");
    let block: ImplId = big.impls().last().expect("a block").id;
    let error = small
        .reachable(&[Node::Impl(block)])
        .expect_err("no block here");
    assert!(matches!(error, ReachError::UnknownImpl(id) if id == block));
    assert!(
        error.to_string().contains("past the impl blocks"),
        "{error}"
    );
    assert!(small.reachable(&[value(&small, "demo::a")]).is_ok());
    // An id is a place in the workspace that gave it: an id of another
    // workspace that is in range is read as the item at that place here.
    let near = big.items().map(|item| item.id).min().expect("items");
    let reached = small.reachable(&[Node::Item(near)]).expect("in range");
    assert_eq!(
        names(&small, &reached),
        [small.item(near).expect("an item here").path.to_string()]
    );
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
            "pub fn api() { b::thing() }\n\
             #[cfg(test)]\n\
             mod tests {\n    #[test]\n    fn t() { b::thing() }\n    fn helper() { b::thing() }\n\
             impl super::S { fn check() { b::thing() } }\n}\n\
             pub struct S;\n",
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
    // A function of a `#[cfg(test)]` module that is no test, and a method of
    // a block there, are test code too.
    assert!(reach(&ws, &[value(&ws, "a::tests::helper")]).contains(thing));
    assert!(reach(&ws, &[value(&ws, "a::S::check")]).contains(thing));
    assert!(!reach(&ws, &[value(&ws, "a::api")]).contains(thing));
}

/// A library of `text` in a package of `edition`.
fn lib_of(edition: &str, text: &str) -> Workspace {
    let manifest =
        format!("[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"{edition}\"\n");
    let tree: MemoryTree = [("Cargo.toml", manifest.as_str()), ("src/lib.rs", text)]
        .into_iter()
        .collect();
    Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn gen_is_a_name_before_edition_2024_and_a_keyword_from_it() {
    let text = "pub fn f() { gen() }\npub fn g() { r#gen() }\nfn r#gen() {}\n";
    for edition in ["2015", "2018", "2021"] {
        let ws = lib_of(edition, text);
        let generate = value(&ws, "demo::gen");
        let reached = reach(&ws, &[value(&ws, "demo::f")]);
        assert_eq!(names(&ws, &reached), ["demo::f", "demo::gen"], "{edition}");
        assert!(
            reach(&ws, &[value(&ws, "demo::g")]).contains(generate),
            "{edition}"
        );
    }
    // From edition 2024 `gen` is a keyword: only the raw name is a path.
    let ws = lib_of("2024", text);
    let generate = value(&ws, "demo::gen");
    assert_eq!(
        names(&ws, &reach(&ws, &[value(&ws, "demo::f")])),
        ["demo::f"]
    );
    assert!(reach(&ws, &[value(&ws, "demo::g")]).contains(generate));
}

#[test]
fn a_shared_file_reads_keywords_in_each_reached_packages_edition() {
    for (alpha, zeta) in [("2024", "2021"), ("2021", "2024")] {
        let manifest = |name: &str, edition: &str| {
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"{edition}\"\n")
        };
        let a = manifest("alpha", alpha);
        let z = manifest("zeta", zeta);
        let tree = MemoryTree::from([
            (
                "Cargo.toml",
                "[workspace]\nmembers = [\"alpha\", \"zeta\"]\n",
            ),
            ("alpha/Cargo.toml", a.as_str()),
            ("zeta/Cargo.toml", z.as_str()),
            (
                "alpha/src/lib.rs",
                "#[path = \"../../shared.rs\"] mod shared;\n",
            ),
            (
                "zeta/src/lib.rs",
                "#[path = \"../../shared.rs\"] mod shared;\n",
            ),
            // Both unused macro definitions are valid in either edition;
            // their bodies are tokens, not expanded Rust expressions.
            (
                "shared.rs",
                "macro_rules! probe { () => { gen() }; }\nmacro_rules! raw { () => { r#gen() }; }\nfn r#gen() {}\n",
            ),
        ]);
        let ws = Workspace::load(&tree, &["."]).expect("shared source loads");
        let macro_node = |package: &str, name: &str| {
            Node::Item(site(
                &ws,
                &format!("{package}::shared::{name}"),
                Namespace::Macro,
            ))
        };
        let roots = [macro_node("alpha", "probe"), macro_node("zeta", "probe")];
        let generators = [
            value(&ws, "alpha::shared::gen"),
            value(&ws, "zeta::shared::gen"),
        ];
        let files: Vec<_> = generators
            .iter()
            .map(|node| match node {
                Node::Item(id) => &ws.item(*id).expect("generator").file,
                Node::Impl(_) => unreachable!("generator is a function"),
            })
            .collect();
        assert_eq!(files[0], files[1], "one physical cached source file");
        let mut separate = std::collections::BTreeSet::new();
        for (i, edition) in [alpha, zeta].into_iter().enumerate() {
            let single = reach(&ws, &[roots[i]]);
            assert_eq!(single.contains(generators[i]), edition == "2021");
            separate.extend(single.reached.iter().map(|found| found.node));
            assert!(reach(&ws, &[macro_node(["alpha", "zeta"][i], "raw")]).contains(generators[i]));
        }
        for ordered in [roots, [roots[1], roots[0]]] {
            let combined = reach(&ws, &ordered);
            let nodes: std::collections::BTreeSet<_> =
                combined.reached.iter().map(|found| found.node).collect();
            assert_eq!(nodes, separate, "editions {alpha}/{zeta}");
        }
    }
}

#[test]
fn a_pointer_to_a_type_and_a_raw_borrow_of_a_static_reach_them_and_a_constant_its_type() {
    let ws = lib("pub fn ptr(p: *const Pointee, q: *mut Other) {}\n\
         struct Pointee;\n\
         struct Other;\n\
         pub fn raw() -> *const u8 { &raw const VALUE }\n\
         static VALUE: u8 = 1;\n\
         pub const C: Kind = Kind;\n\
         pub struct Kind;\n\
         pub const fn named() {}\n");
    assert_eq!(
        sorted(&ws, &reach(&ws, &[value(&ws, "demo::ptr")])),
        ["demo::Other", "demo::Pointee", "demo::ptr"]
    );
    assert_eq!(
        sorted(&ws, &reach(&ws, &[value(&ws, "demo::raw")])),
        ["demo::VALUE", "demo::raw"]
    );
    // A constant names its type, and a const fn nothing beside it.
    let reached = reach(&ws, &[value(&ws, "demo::C")]);
    assert!(reached.contains(ty(&ws, "demo::Kind")));
    assert_eq!(
        names(&ws, &reach(&ws, &[value(&ws, "demo::named")])),
        ["demo::named"]
    );
}

#[test]
fn self_paths_and_methods_called_on_self_reach_the_members_of_that_name() {
    let ws = lib("#[derive(Default)]\n\
         pub struct S;\n\
         impl S {\n\
             pub fn api(&self) -> u8 { Self::helper() + self.method() + Self::CONST + Self::default().method() }\n\
             fn helper() -> u8 { 1 }\n\
             const CONST: u8 = 3;\n\
             fn unused() {}\n\
             pub fn make() -> Self { Self }\n\
         }\n\
         impl S { fn method(&self) -> u8 { 2 } }\n\
         pub fn elsewhere(s: S) -> u8 { s.method() }\n\
         fn method() {}\n");
    // `Self::name` and `self.name()` name the members called `name` of the
    // type, in every block of it; `Self::default()`, which a derive makes,
    // names nothing, and the type itself is not reached through `Self`.
    assert_eq!(
        sorted(&ws, &reach(&ws, &[value(&ws, "demo::S::api")])),
        [
            "demo::S::CONST",
            "demo::S::api",
            "demo::S::helper",
            "demo::S::method"
        ]
    );
    assert_eq!(
        names(&ws, &reach(&ws, &[value(&ws, "demo::S::make")])),
        ["demo::S::make"]
    );
    // A method called on any other value is not read by its name; the
    // value's type reaches its methods as their type.
    let reached = reach(&ws, &[value(&ws, "demo::elsewhere")]);
    assert!(reached.contains(ty(&ws, "demo::S")));
    assert!(!reached.contains(value(&ws, "demo::method")));
}

#[test]
fn self_in_a_trait_and_in_its_impl_blocks_reaches_the_members_of_the_trait_and_of_the_block() {
    let ws = lib("pub trait Tr {\n\
             fn req(&self) -> u8;\n\
             fn provided(&self) -> u8 { self.req() + Self::other() }\n\
             fn other() -> u8 { 0 }\n\
             fn lonely() {}\n\
         }\n\
         pub struct S;\n\
         impl Tr for S { fn req(&self) -> u8 { self.provided() } }\n\
         impl Tr for String { fn req(&self) -> u8 { Self::other() } fn other() -> u8 { 1 } }\n");
    // In a trait, the trait's own members.
    assert_eq!(
        sorted(&ws, &reach(&ws, &[value(&ws, "demo::Tr::provided")])),
        ["demo::Tr::other", "demo::Tr::provided", "demo::Tr::req"]
    );
    // In a block of the trait, the trait's members too: `provided` is not
    // the block's.
    assert_eq!(
        sorted(&ws, &reach(&ws, &[value(&ws, "demo::S::req")])),
        [
            "demo::S::req",
            "demo::Tr::other",
            "demo::Tr::provided",
            "demo::Tr::req"
        ]
    );
    // In a block that no type of the workspace keys, its own members.
    let foreign = ws.impls().last().expect("the block on String");
    let (req, other) = (foreign.members[0], foreign.members[1]);
    let reached = reach(&ws, &[Node::Item(req)]);
    assert!(reached.contains(Node::Item(other)));
    assert!(reached.contains(value(&ws, "demo::Tr::other")));
    assert!(!reached.contains(value(&ws, "demo::Tr::lonely")));
    assert!(!reached.contains(Node::Impl(foreign.id)));
}

#[test]
fn a_variant_through_self_names_its_enum_as_a_path_through_the_enum_does() {
    let ws = lib("pub enum E { A, B }\n\
         impl E { pub fn is_a(&self) -> bool { matches!(self, Self::A) } fn unused() {} }\n\
         pub struct S;\n\
         impl S { pub fn get() -> u8 { Self::NOT_HERE } }\n");
    let reached = reach(&ws, &[value(&ws, "demo::E::is_a")]);
    let enumeration = ty(&ws, "demo::E");
    assert_eq!(
        reached.get(enumeration).map(|found| found.edge),
        Some(Edge::References)
    );
    // A struct has no variants: a name that is no member names nothing.
    assert_eq!(
        names(&ws, &reach(&ws, &[value(&ws, "demo::S::get")])),
        ["demo::S::get"]
    );
}

#[test]
fn the_type_a_where_clause_bounds_is_reached_and_a_bound_name_is_not() {
    let ws = lib(
        "pub fn bounded<T>() where Helper: Clone, T: Into<Other> {}\n\
         #[derive(Clone)]\n\
         struct Helper;\n\
         struct Other;\n\
         pub fn param(helper: u8) -> u8 { helper }\n\
         pub fn generic<Unbound: Clone>() {}\n\
         struct Unbound;\n",
    );
    assert_eq!(
        sorted(&ws, &reach(&ws, &[value(&ws, "demo::bounded")])),
        ["demo::Helper", "demo::Other", "demo::bounded"]
    );
    // A parameter and a generic parameter before their `:` are names being
    // bound.
    assert_eq!(
        names(&ws, &reach(&ws, &[value(&ws, "demo::generic")])),
        ["demo::generic"]
    );
}

#[test]
fn a_walk_past_the_item_bytes_budget_is_refused_and_one_within_it_is_given() {
    let text = "pub fn api() { helper() }\nfn helper() {}\n";
    let tree: MemoryTree = [("Cargo.toml", MANIFEST), ("src/lib.rs", text)]
        .into_iter()
        .collect();
    let held = lib(text).usage().item_bytes;
    let with = |limit: usize| {
        let mut options = LoadOptions::default();
        options.max_item_bytes = limit;
        Workspace::load_with(&tree, &["."], &options).unwrap_or_else(|e| panic!("{e}"))
    };
    // The load fits its limit exactly, and the first charge of the walk
    // passes it.
    let ws = with(held);
    let error = ws
        .reachable(&[value(&ws, "demo::api")])
        .expect_err("over the budget");
    let ReachError::Budget(ref load) = error else {
        panic!("{error}");
    };
    assert!(
        matches!(load.kind, LoadErrorKind::Budget { budget: Budget::ItemBytes, limit } if limit == held),
        "{load}"
    );
    assert_eq!(load.file.as_str(), "src/lib.rs");
    assert!(
        error.to_string().starts_with("the walk is over budget: "),
        "{error}"
    );
    let ws = with(held + (1 << 16));
    assert_eq!(
        names(&ws, &reach(&ws, &[value(&ws, "demo::api")])),
        ["demo::api", "demo::helper"]
    );
}
