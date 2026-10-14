// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The caps on `workspace.members` and `workspace.exclude`, and the cost of
//! matching them. Each cap is shown at its limit, where the load succeeds, and
//! one over, where it is refused with a `Manifest` error that names the key.
//! Large pattern lists, duplicated or distinct, load or are refused in well
//! under a second.

use std::fmt::Write as _;
use std::time::{Duration, Instant};

use raf_source::{LoadError, LoadErrorKind, MemoryTree, Origin, Workspace};

const CAP: usize = 4096;

const PACKAGE: &str = "[package]\nname = \"p\"\nversion = \"0.1.0\"\nedition.workspace = true\n";

/// A workspace manifest whose `members` and `exclude` hold the given quoted
/// entries, and one package `p`.
fn load(members: &[String], exclude: &[String]) -> Result<Workspace, LoadError> {
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
    let tree: MemoryTree = [
        ("Cargo.toml", root.as_str()),
        ("p/Cargo.toml", PACKAGE),
        ("p/src/lib.rs", ""),
    ]
    .into_iter()
    .collect();
    Workspace::load(&tree, &["p"])
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

fn fast<T>(work: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let value = work();
    let took = start.elapsed();
    assert!(took < Duration::from_secs(1), "took {took:?}");
    value
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
    let at = fast(|| load(&["p".to_owned()], &globs(CAP, "q")));
    at.unwrap_or_else(|error| panic!("at the cap: {error}"));
    let over = fast(|| load(&["p".to_owned()], &globs(CAP + 1, "q")));
    refused_key(over, "workspace.exclude", CAP + 1);
}

#[test]
fn members_at_the_cap_loads_and_one_over_is_refused() {
    let at = fast(|| load(&with(globs(CAP - 1, "m"), "p"), &[]));
    at.unwrap_or_else(|error| panic!("at the cap: {error}"));
    let over = fast(|| load(&with(globs(CAP, "m"), "p"), &[]));
    refused_key(over, "workspace.members", CAP + 1);
}

#[test]
fn the_duplicate_pattern_probe_is_refused_at_once() {
    // 20000 members and 200000 excludes: an 880 KB manifest.
    let members = vec!["p".to_owned(); 20_000];
    let exclude = vec!["q".to_owned(); 200_000];
    let result = fast(|| load(&members, &exclude));
    refused_key(result, "workspace.members", 20_000);
}

#[test]
fn duplicate_patterns_within_the_caps_load_at_once() {
    let members = vec!["p".to_owned(); CAP];
    let exclude = vec!["q*".to_owned(); CAP];
    fast(|| load(&members, &exclude)).unwrap_or_else(|error| panic!("loads: {error}"));
}

#[test]
fn distinct_patterns_at_both_caps_load_at_once() {
    // Every literal member is tested against every wildcard exclude, and the
    // only exclude that matches comes last.
    let members = with(names(CAP - 1, "m"), "p");
    let exclude = with(globs(CAP - 1, "a"), "m*");
    fast(|| load(&members, &exclude)).unwrap_or_else(|error| panic!("loads: {error}"));
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
