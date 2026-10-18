// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The caps on `workspace.members` and `workspace.exclude`, and the tree
//! calls their patterns cost. Each cap is shown at its limit, where the load
//! succeeds, and one over, where it is refused with a `Manifest` error that
//! names the key before any pattern is expanded. The cost of a load is the
//! calls it makes to a counting tree, compared with the calls of a load that
//! holds one of each pattern.

mod support;

use std::fmt::Write as _;

use raf_source::{LoadError, LoadErrorKind, MemoryTree, Origin, Workspace};
use support::counting::{Counting, Op};

const CAP: usize = 4096;

const PACKAGE: &str = "[package]\nname = \"p\"\nversion = \"0.1.0\"\nedition.workspace = true\n";

/// The most tree calls any load here may make: the largest one expands 4095
/// member globs, one call each.
const CEILING: usize = CAP + 200;

/// A load that was counted, and the counting tree it ran against.
struct Counted {
    result: Result<Workspace, LoadError>,
    tree: Counting<MemoryTree>,
}

impl Counted {
    /// The calls to the tree, by kind.
    fn ops(&self) -> [usize; 5] {
        [Op::Read, Op::Open, Op::Identity, Op::List, Op::Entry].map(|op| self.tree.count(op))
    }
}

/// A workspace manifest whose `members` and `exclude` hold the given quoted
/// entries, and one package `p`, loaded through a counting tree.
fn load(members: &[String], exclude: &[String]) -> Counted {
    let quoted = |entries: &[String]| {
        let mut text = String::new();
        for entry in entries {
            write!(text, "\"{entry}\",").expect("writes to a string");
        }
        text
    };
    let root = format!(
        "[workspace]\nmembers = [{}]\nexclude = [{}]\n[workspace.package]\nedition = \"2021\"\n",
        quoted(members),
        quoted(exclude)
    );
    counted(&root)
}

/// The workspace whose root manifest is `root`, beside one package `p`.
fn counted(root: &str) -> Counted {
    let tree: MemoryTree = [
        ("Cargo.toml", root),
        ("p/Cargo.toml", PACKAGE),
        ("p/src/lib.rs", ""),
    ]
    .into_iter()
    .collect();
    let tree = Counting::new(tree).with_ceiling(CEILING);
    let result = Workspace::load(&tree, &["p"]);
    Counted { result, tree }
}

/// A load that holds one member and no exclude.
fn plain() -> Counted {
    load(&["p".to_owned()], &[])
}

/// A load refused while the root manifest is read, before any pattern could
/// be looked at.
fn refused_at_the_manifest() -> Counted {
    let counted = counted("[workspace]\nmembers = \"p\"\n");
    assert!(counted.result.is_err());
    counted
}

fn names(count: usize, stem: &str) -> Vec<String> {
    (0..count).map(|i| format!("{stem}{i}")).collect()
}

fn globs(count: usize, stem: &str) -> Vec<String> {
    (0..count).map(|i| format!("{stem}{i}/*")).collect()
}

fn with(mut list: Vec<String>, entry: &str) -> Vec<String> {
    list.push(entry.to_owned());
    list
}

fn refused_key(result: Result<Workspace, LoadError>, key: &str, count: usize) {
    let error = result.expect_err("over the cap");
    assert!(
        matches!(error.kind, LoadErrorKind::Manifest { .. }),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "Cargo.toml", "{error}");
    let origin = error.origin.as_deref().expect("names the key");
    assert_eq!(
        origin,
        &Origin::ManifestKey {
            key: key.to_owned()
        },
        "{error}"
    );
    let text = error.to_string();
    assert!(
        text.contains(&format!("has {count} entries; at most {CAP} are allowed")),
        "{text}"
    );
}

#[test]
fn exclude_at_the_cap_loads_and_one_over_is_refused() {
    let at = load(&["p".to_owned()], &globs(CAP, "q"));
    at.result
        .as_ref()
        .unwrap_or_else(|error| panic!("at the cap: {error}"));
    // The excludes are matched in memory, never expanded through the tree.
    assert_eq!(at.ops(), plain().ops());
    assert!(at.tree.count(Op::Entry) > 0);
    let over = load(&["p".to_owned()], &globs(CAP + 1, "q"));
    assert_eq!(over.ops(), refused_at_the_manifest().ops());
    refused_key(over.result, "workspace.exclude", CAP + 1);
}

#[test]
fn members_at_the_cap_loads_and_one_over_is_refused() {
    let reference = plain();
    let at = load(&with(globs(CAP - 1, "m"), "p"), &[]);
    at.result
        .as_ref()
        .unwrap_or_else(|error| panic!("at the cap: {error}"));
    // A glob whose first segment is a literal asks the tree about that one
    // name and lists nothing.
    assert_eq!(at.tree.count(Op::List), 0);
    assert_eq!(
        at.tree.count(Op::Entry),
        reference.tree.count(Op::Entry) + CAP - 1
    );
    assert_eq!(at.tree.count(Op::Read), reference.tree.count(Op::Read));
    let over = load(&with(globs(CAP, "m"), "p"), &[]);
    assert_eq!(over.ops(), refused_at_the_manifest().ops());
    refused_key(over.result, "workspace.members", CAP + 1);
}

#[test]
fn the_duplicate_pattern_probe_is_refused_before_any_pattern_is_expanded() {
    // 20000 members and 200000 excludes: an 880 KB manifest.
    let members = vec!["p".to_owned(); 20_000];
    let exclude = vec!["q".to_owned(); 200_000];
    let probe = load(&members, &exclude);
    let reference = refused_at_the_manifest();
    assert_eq!(probe.ops(), reference.ops());
    assert_eq!(probe.tree.read_log(), reference.tree.read_log());
    refused_key(probe.result, "workspace.members", 20_000);
}

#[test]
fn duplicate_patterns_within_the_caps_load_with_the_tree_calls_of_one_of_each() {
    let members = vec!["p".to_owned(); CAP];
    let exclude = vec!["q*".to_owned(); CAP];
    let many = load(&members, &exclude);
    many.result
        .as_ref()
        .unwrap_or_else(|error| panic!("loads: {error}"));
    let one = load(&["p".to_owned()], &["q*".to_owned()]);
    one.result.as_ref().expect("loads");
    assert_eq!(many.ops(), one.ops());
    assert!(many.tree.count(Op::Entry) > 0);
}

#[test]
fn distinct_patterns_at_both_caps_load_with_no_tree_calls_for_the_excludes() {
    // Every literal member is tested against every wildcard exclude, and the
    // only exclude that matches comes last. The tests are made in memory, so
    // the tree sees the calls of the same members and the one exclude that
    // matches.
    let members = with(names(CAP - 1, "m"), "p");
    let many = load(&members, &with(globs(CAP - 1, "a"), "m*"));
    many.result
        .as_ref()
        .unwrap_or_else(|error| panic!("loads: {error}"));
    let one = load(&members, &["m*".to_owned()]);
    one.result.as_ref().expect("loads");
    assert_eq!(many.ops(), one.ops());
    assert!(many.tree.count(Op::Entry) > 0);
}

#[test]
fn exclude_still_removes_literal_wildcard_and_nested_members() {
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"a\", \"b/*\", \"c\", \"d\"]\nexclude = [\"a\", \"b/x*\", \"c/sub\", \"d\", \"d\"]\n[workspace.package]\nedition = \"2021\"\n",
        ),
        ("a/Cargo.toml", PACKAGE),
        ("b/xy/Cargo.toml", "[package]\nname = \"xy\"\nversion = \"0.1.0\"\nedition.workspace = true\n"),
        ("b/ok/Cargo.toml", "[package]\nname = \"ok\"\nversion = \"0.1.0\"\nedition.workspace = true\n"),
        ("b/ok/src/lib.rs", ""),
        ("c/Cargo.toml", "[package]\nname = \"c\"\nversion = \"0.1.0\"\nedition.workspace = true\n"),
        ("c/src/lib.rs", ""),
    ]
    .into_iter()
    .collect();
    for member in ["b/ok", "c"] {
        Workspace::load(&tree, &[member]).unwrap_or_else(|e| panic!("{member}: {e}"));
    }
    for excluded in ["a", "b/xy"] {
        let error = Workspace::load(&tree, &[excluded]).expect_err("excluded");
        assert!(error.to_string().contains("member"), "{excluded}: {error}");
    }
}
