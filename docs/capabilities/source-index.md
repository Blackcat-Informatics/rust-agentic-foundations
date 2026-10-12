---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:source-index
title: "Source index"
description: "The raf-source crate: a Tree abstraction, package discovery, mod-tree following with #[path], and a symbol and use-alias index."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: []
---

# Source index

## Purpose

`raf-source` answers one question for the other crates: does this Rust path name a
real item, and where is it defined. The census resolves ledger homes with it and the
workplan resolves code and test bindings with it, so one walker and one index serve
both and cannot disagree.

## Contract

**Tree.** All file access goes through the `Tree` trait: list a directory, read a
file, test existence. `DiskTree` reads a directory on disk. `MemoryTree` holds a map
of path to text and exists so that fixtures build repositories without touching the
disk. Nothing else in the crate opens a file.

**Package discovery.** Given a set of roots, the crate finds every package: each
directory holding a `Cargo.toml` with a `[package]` table. For each package it reads
the library, binary, test, bench and example targets, honouring explicit target
paths and the default `src/`, `tests/`, `benches/` and `examples/` layout. Test,
bench and example targets are marked non-shipping; library and binary targets are
shipping.

**Mod-tree following.** From each target root the crate follows `mod name;`
declarations to `name.rs` or `name/mod.rs` and parses each file with `syn`. An
attribute `#[path = "..."]` on a module redirects to the named file, relative to the
directory the module would otherwise use. A resolved path that leaves the workspace
root, by `..` segments or by an absolute path, is an error naming the module and the
attribute; the file is not read. Inline modules are followed in place. An unresolved
`mod` declaration is an error that names the file and line.

**`cfg(test)` tagging.** Items under `#[cfg(test)]`, and modules reached only through
such an item, are tagged as test code. They stay in the index with that tag so that
a caller decides whether they count.

**Symbol index.** Every named item, which is a function, struct, enum, union, trait,
type alias, constant, static, module and macro definition, is recorded with its full
crate path, its kind, its file, its line, its visibility and its test tag. Methods
are recorded under their impl type path.

**Use-alias index.** Every `use` declaration, including renames, nested groups and
globs, contributes entries to an alias table. `resolve(path)` follows the table
through re-exports, with a cycle guard, until it reaches a defined item, and returns
that item or a typed failure: not found, ambiguous, or cyclic. A path that resolves
through a glob is reported as such. A symbol that a macro generates is not indexed,
and the documentation of the capabilities that rely on the index says so.

## Usage

```rust
use raf_source::{DiskTree, Workspace};

let ws = Workspace::load(&DiskTree::new("."), &["crates"])?;
let item = ws.resolve("raf_okf::Document")?;
println!("{} at {}:{}", item.path, item.file.display(), item.line);
for t in ws.targets().filter(|t| !t.shipping) {
    println!("{} is a non-shipping target", t.name);
}
```

## Examples

With `MemoryTree` a fixture needs no disk:

```rust
use raf_source::{MemoryTree, Workspace};

let tree = MemoryTree::from([
    ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ("src/lib.rs", "mod inner;\npub use inner::go as run;\n"),
    ("src/inner.rs", "pub fn go() {}\n"),
]);
let ws = Workspace::load(&tree, &["."])?;
assert_eq!(ws.resolve("demo::run")?.path, "demo::inner::go");
```

## Constraints

- Parsing is syntactic. The crate does not expand macros, evaluate `cfg` other than
  the `test` tag, or run type inference.
- A `#[path]` target must stay inside the workspace root.
- Resolution covers items the source defines. Items from dependencies are outside the
  index and resolve to a not-found failure.
- The index is built once per `Workspace::load`; callers reload to see changed files.
- Output order is sorted by path, so two loads of the same tree give equal results.

## Explanation

A census that compares function bodies, and a workplan that checks that a documented
symbol exists, both fail in the same way when they trust text: a re-export makes the
public path differ from the defining path, and a module declared with `#[path]` is
invisible to a directory walk. Both tools need the compiler's view of the module
tree, and `syn` gives a close enough one without the cost of invoking the compiler.

The `Tree` trait is the seam that keeps the crate testable. Acceptance fixtures
for both consumers are repositories built in memory, and a disk-backed run uses the
same code path, so a fixture that passes says something about the real workspace.

Keeping test and example targets in the index, tagged non-shipping, lets one
consumer ignore them and another bind to them. Workplan bindings name tests and
examples, so those must resolve; the census must not count test helpers as shipped
duplicates.

## Verification

- A `MemoryTree` fixture per discovery rule: a library, a binary, and each of the
  test, bench and example targets with default and explicit paths, asserting the
  shipping flag.
- Mod-tree fixtures for `name.rs`, `name/mod.rs`, inline modules, nested modules and
  an unresolved `mod` error.
- `#[path]` fixtures: a redirect inside the root resolves; `..` out of the root and
  an absolute path outside it both fail without reading the file.
- A `cfg(test)` fixture asserting the tag on an item and on a module reached only
  through it.
- Alias fixtures: a rename, a nested group, a glob, a re-export chain of three
  hops, and a cycle that yields the cyclic failure.
- A determinism check: two loads over the same tree give equal indexes.
- A `DiskTree` run over a temporary directory gives the same index as the
  equivalent `MemoryTree`.
