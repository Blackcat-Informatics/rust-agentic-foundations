// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A two-package workspace that holds a copy of every test-only form, copies
//! under a named directory, and copies in integration-test and example targets.

use raf_source::MemoryTree;

/// A body of 21 value-form tokens over `a`, distinct for each operator.
fn body(op: &str) -> String {
    ["a"; 11].join(&format!(" {op} "))
}

fn function(attributes: &str, name: &str, op: &str) -> String {
    format!(
        "{attributes}pub fn {name}(a: u32) -> u32 {{ {} }}\n",
        body(op)
    )
}

/// One copy of each test-only form, a mixed `any` that is not test-only, and
/// a `#[test]` function, as a library of `package` writes them.
fn library(package: &str) -> String {
    let directory = if package == "one" {
        "mod generated;\n".to_owned()
    } else {
        format!("mod emitted;\n{}", function("", "copy_again", "&"))
    };
    format!(
        "{directory}#[cfg(test)]\nmod tests {{\n{}}}\n{}{}{}#[test]\nfn test_fn() {{ let a = 3u32; let _ = {}; }}\n",
        function("", "helper", "+"),
        function("#[cfg(all(unix, test))]\n", "all_form", "-"),
        function("#[cfg(any(test, test))]\n", "any_form", "*"),
        function("#[cfg(any(test, unix))]\n", "mixed_form", "/"),
        body("%"),
    )
}

/// An integration test; `a` reaches a file under `generated/` whose copy `b`
/// holds itself.
fn integration(name: &str) -> String {
    let made = if name == "a" {
        "mod generated;\n".to_owned()
    } else {
        function("", "made_b", "<<")
    };
    format!(
        "{made}{}#[test]\nfn check() {{ let a = 3u32; let _ = {}; }}\n",
        function("", &format!("shared_{name}"), "^"),
        body("%"),
    )
}

fn example(name: &str) -> String {
    format!(
        "fn main() {{}}\n#[cfg(test)]\nmod t {{\n{}}}\n",
        function("", &format!("example_{name}"), "|"),
    )
}

fn manifest(name: &str) -> String {
    format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
}

pub(crate) fn tree() -> MemoryTree {
    let files = [
        (
            "Cargo.toml".to_owned(),
            "[workspace]\nmembers = [\"one\", \"two\"]\nresolver = \"2\"\n".to_owned(),
        ),
        ("one/Cargo.toml".to_owned(), manifest("one")),
        ("one/src/lib.rs".to_owned(), library("one")),
        (
            "one/src/generated/mod.rs".to_owned(),
            function("", "copy", "&"),
        ),
        ("one/tests/a.rs".to_owned(), integration("a")),
        ("one/tests/b.rs".to_owned(), integration("b")),
        (
            "one/tests/generated/mod.rs".to_owned(),
            function("", "made", "<<"),
        ),
        ("one/examples/x.rs".to_owned(), example("x")),
        ("one/examples/y.rs".to_owned(), example("y")),
        ("two/Cargo.toml".to_owned(), manifest("two")),
        ("two/src/lib.rs".to_owned(), library("two")),
        (
            "two/src/emitted/mod.rs".to_owned(),
            function("", "copy", "&"),
        ),
    ];
    files
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect()
}

/// A forwarder of 6 value tokens, a shim candidate that never body-groups.
fn forwarder(attributes: &str, name: &str, callee: &str) -> String {
    format!("{attributes}pub fn {name}(a: u32) -> u32 {{ {callee}(a, 1) }}\n")
}

/// A two-package workspace in which `one/src/shared.rs` is reached twice: by
/// the library under `#[cfg(test)]` and by an integration test through
/// `#[path]`. Package `two` holds a library copy of its body. Both libraries
/// also hold a test-only forwarder and a plain forwarder, each pair with one
/// shim form.
pub(crate) fn reach_tree() -> MemoryTree {
    let library = |package: &str| {
        let shared = if package == "one" {
            "#[cfg(test)]\nmod shared;\n".to_owned()
        } else {
            function("", "both", "+")
        };
        format!(
            "{shared}#[cfg(test)]\nmod tests {{\n{}}}\n{}",
            forwarder("", "forward", "helper"),
            forwarder("", "relay", "other"),
        )
    };
    let files = [
        (
            "Cargo.toml".to_owned(),
            "[workspace]\nmembers = [\"one\", \"two\"]\nresolver = \"2\"\n".to_owned(),
        ),
        ("one/Cargo.toml".to_owned(), manifest("one")),
        ("one/src/lib.rs".to_owned(), library("one")),
        ("one/src/shared.rs".to_owned(), function("", "both", "+")),
        (
            "one/tests/it.rs".to_owned(),
            "#[path = \"../src/shared.rs\"]\nmod shared;\n".to_owned(),
        ),
        ("two/Cargo.toml".to_owned(), manifest("two")),
        ("two/src/lib.rs".to_owned(), library("two")),
    ];
    files
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect()
}
