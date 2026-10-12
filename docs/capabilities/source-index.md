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

**Path containment.** Every path the crate opens is checked against the workspace
root, so a hostile tree cannot make the index read a file outside it. This covers
files reached by `mod name;` with or without `#[path]`, explicit `path` values of
`Cargo.toml` targets, and every discovered `Cargo.toml`. `DiskTree::new` canonicalises
the root once. For each candidate it canonicalises the resolved path, which follows
every symlink and applies every `..` against the real directories, and requires the
canonical root as a prefix of the result. A candidate outside the root is an error
that names the module, attribute or manifest key, and it is neither opened nor read.
A lexical check of `..` segments is not enough, because `link/..` leaves the root when
`link` is a directory symlink. `DiskTree` refuses any entry that is not a regular
file, such as a FIFO or a device, and it does not descend into a directory symlink,
so a symlink loop ends at the first link. The file type is checked on the opened
handle and not by a stat before the open: the crate opens the canonical path
non-blocking, so opening a FIFO cannot hang, and then calls `fstat` on the handle and
refuses anything that is not a regular file. No check separates from the open it
guards.

**Resource bounds.** Source is parsed in-process. Each read is capped at
`max_file_bytes`, which defaults to 8 MiB and is set by `LoadOptions::max_file_bytes`
or, for the census, by `max_file_bytes` in `[census]` (see
[census-ledger](./census-ledger.md)). A larger file is an error that names it. `syn`
recurses on more than delimiters, so before any file is parsed the crate runs a token
pre-scan that skips strings and comments and refuses, with an error that names the
file, any file that exceeds one of these budgets: delimiter nesting of 128 levels;
nested `<` of 64 levels; a run of prefix operators (`!`, `-`, `*`, `&` and `&&`) of
256 tokens; a chain of closure bars (`||` or `|…|` with no body between) of 128; an
`else if` chain of 256. Parsing then runs on a dedicated thread with a stack of 64
MiB, which is a stated constant and not the stack of the caller. A committed fuzz
target, `parse_source` in `fuzz/fuzz_targets/parse_source.rs`, feeds arbitrary bytes
through the pre-scan and the parse. Its corpus, `fuzz/corpus/parse_source/`, is
committed and holds the composite and single-shape fixtures of Verification below,
files at and just over each budget, and every input that ever aborted a run. These
measures make a stack overflow unlikely. They do not prove its absence: the budgets
multiply across nesting (a run of prefix operators is legal inside every delimiter
level), some recursive shapes are counted by no budget, and a debug build costs about
16 KiB of stack per level. The crate does not claim that an admitted file cannot
overflow. A residual overflow aborts the process, so the run fails and is never
reported as a pass. A parser overflow is a robustness bug, and an input that causes
one is added to the fuzz corpus and to the budgets. `Cargo.toml` is read with the same
size cap, and the `toml` parser's own recursion limit turns deep nesting into an error
and not a crash. Every refusal the pre-scan makes is a typed error and not a panic.
The crate sets no time bound.

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
root, whether by `..` segments, an absolute path or a symlink, is an error naming the
module and the attribute; the file is not read. Inline modules are followed in place.
An unresolved `mod` declaration is an error that names the file and line.

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
- Every opened path must be, after canonicalisation, inside the canonical workspace
  root and a regular file; the same rule bounds explicit `Cargo.toml` target paths.
- A file larger than `max_file_bytes` (default 8 MiB), or one that exceeds a
  recursion budget of the pre-scan (128 delimiter levels, 64 nested `<`, 256 prefix
  operators, 128 closure bars, 256 `else if` links), is refused with a typed error.
  Parsing runs on a dedicated 64 MiB stack. A residual overflow aborts the run and
  fails the gate; it is never a pass.
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
- Bound fixtures: a file over `max_file_bytes` fails with an error that names the
  file, a lowered `max_file_bytes` is honoured, and a file of 8 MiB parses at the
  default. A file at the delimiter limit (128 levels) parses and one at 129 levels
  fails. Each of these shapes just over its budget, and the shape sizes below, fails
  with a typed error that names the file, and runs in debug and in release on the
  default 2 MiB test-thread stack of the caller, so the dedicated stack and the
  pre-scan are what keep the process alive:
  - 500 nested `A<A<...>>` generics (delimiter depth 0);
  - `!` repeated 4000 times before an expression;
  - `&` repeated 20000 times before a type;
  - `||` repeated 10000 times as a closure chain;
  - an `else if` chain of 20000 links.

  A file at each budget (64 nested `<`, 256 `!`, 128 closure bars, 256 `else if`
  links) parses on the same stack.
- Composite and uncounted-shape fixtures: `!` repeated 256 times before `(` repeated
  across 127 levels; `&` repeated 256 times before `(` across 127 levels, in a type;
  `&&` repeated 256 times before `(` across 127 levels; `A<` followed by 256 `&`,
  repeated across 64 levels; `||` repeated 128 times before `(` across 127 levels;
  and 20000 repetitions each of `return `, `&mut `, `*const `, `fn() -> `, `a = ` and
  `move || `. A harness runs each in a child process, in debug and in release, and
  asserts that it ends in exactly one of three outcomes: the pre-scan refuses it with
  a typed error that names the file and the budget; the process exits 0 with a
  correct, complete index (the symbols of the fixture, compared with the expected
  list); or the process aborts with a non-zero status. Exit 0 with a wrong or partial
  index, or with an index produced after an abort, fails the fixture.
- The parse thread is named after the file it parses, so a stack-overflow message
  from the runtime names the culprit. A fixture asserts the thread name for a file
  and that the abort message of the overflow fixtures contains that name.
- Corpus replay under `cargo test`: a test parses every file of
  `fuzz/corpus/parse_source/` in a child process and asserts that each ends in one of
  the three outcomes above, so a regression is caught without running the fuzzer.
- The fuzz target `parse_source` builds and runs for a bounded number of iterations
  over its corpus without a crash.
- A `cfg(test)` fixture asserting the tag on an item and on a module reached only
  through it.
- Alias fixtures: a rename, a nested group, a glob, a re-export chain of three
  hops, and a cycle that yields the cyclic failure.
- A determinism check: two loads over the same tree give equal indexes.
- A `DiskTree` run over a temporary directory gives the same index as the
  equivalent `MemoryTree` for trees of regular files. `MemoryTree` has no symlinks or
  special files, so this equivalence does not cover the containment cases, which have
  their own `DiskTree` fixtures below.
- `DiskTree` containment fixtures, each built in a temporary directory beside a
  sentinel file outside the root, asserting an error and that the sentinel is not
  read:
  - `src/inner.rs` is a symlink to a file outside the root, reached by a plain
    `mod inner;`;
  - `#[path = "vendored/../x.rs"]` where `vendored` is a symlink to a directory
    outside the root, so the `..` leaves the root only on disk;
  - a `Cargo.toml` target with `path = "../outside.rs"` and one with an absolute
    path outside the root;
  - a link to a symlink that itself points outside the root;
  - a directory-symlink loop, which ends with an error and does not recurse;
  - a FIFO named as a module file, which is refused without blocking on a read.
