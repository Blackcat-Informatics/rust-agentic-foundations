---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-reference-scan
title: "Census reference scan"
description: "Compare first-party code with the source of named dependency packages, resolved through cargo metadata."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:census-ledger]
---

# Census reference scan

## Purpose

A helper can be duplicated across a repository boundary: a project re-implements
something that a library it already depends on provides. The reference scan treats
named dependency packages as a reference tree, so the census can home a job in a
dependency and report first-party code that copies it.

## Contract

`[census] reference = ["pkg", ...]` in the ledger names the reference packages.

- The census resolves each name with `cargo metadata`, so the source of a reference
  package is the exact revision that `Cargo.lock` pins. A name that is not in the
  resolved dependency graph is an error.
- The census walks the source of each reference package with the same normal form
  as first-party code (see [census-normal-form](./census-normal-form.md)).
- A job's `home` may be a symbol in a reference package. The home then resolves in
  the reference tree, and the job's `HOME` role is assigned there.
- A first-party body or shim whose fingerprint equals a reference body or shim is
  reported as a reference copy, with the reference symbol it matches.
- Reference code is read-only context. It never produces findings against itself,
  and it never appears in groups of first-party code, only in reference copy
  reports.
- A reference copy is a `COPY` for a job homed in that reference package, and an
  unclassified finding otherwise.

**Invocation.** The census runs exactly this command, spawned directly with no shell
and with the arguments as an argv vector, in the workspace root. The program is the
one named by `$CARGO` when that is set, and `cargo` from `PATH` otherwise:

```text
cargo metadata --format-version 1 --frozen --manifest-path <root>/Cargo.toml
```

`--frozen` means `--locked` and `--offline`: the command does not update
`Cargo.lock` and makes no network access, so it cannot fetch a package that is not
already downloaded. (`--locked` alone still downloads missing packages.) The
`<root>` is the canonical workspace root and is the value of `--manifest-path`, so it
cannot be read as an option. No ledger value, including a reference package name, is
ever placed on the command line. Names are matched against the parsed JSON output.
Resolution needs the full dependency graph, so `--no-deps` is not used here.

**Trust.** This step executes the workspace's cargo configuration exactly as
`cargo build` would: `build.rustc` and `build.rustc-wrapper` from `.cargo/config.toml`
are spawned, and the rustup proxy honours the workspace's `rust-toolchain.toml`. The
workspace is trusted build input (see [the security policy](../../SECURITY.md)). The
output is parsed hardened: it is bounded to 64 MiB and 128 levels of JSON nesting,
and a larger or deeper output is an error.

**Path containment.** The source of a reference package is read from the directory of
the `manifest_path` that `cargo metadata` reports. That directory is the containment
root for the package, in place of the workspace root. The census reads it through the
same `DiskTree` rules as first-party code (see [source index](./source-index.md)):
each file's canonical path must lie inside the canonical package directory, a
directory symlink is not descended into so a walk through a link to an ancestor ends
at the first link, a file that is not a regular file is refused, and that type is
checked on the opened handle. A refusal is an error that names the package and the
file, and the file is not read. The same per-file `max_file_bytes` cap (default 8 MiB,
set in `[census]`), the pre-scan budgets, the 64 MiB parse stack and the
`parse-source` fuzz target apply to reference source. They make a stack overflow
unlikely and do not prove its absence, and a residual overflow aborts the run, which
fails the scan and never passes it. A file ends in one of three outcomes: refused with
a typed error, indexed correctly and completely, or an abort with a non-zero status;
the census never reports success with a wrong or partial index.

## Usage

Name the reference packages in the ledger and home a job in one of them:

```toml
[census]
normal_form = 1
reference = ["acme-core"]

[[job]]
id = "iri-parse"
summary = "IRI parsing."
home = "acme_core::iri::parse"
entry_points = []
enforced = true

[job.forbidden]
names = []
constants = []
fingerprints = []
patterns = []
```

`cargo census check` prints each reference copy with the first-party symbol and the
reference symbol.

## Examples

A first-party function with the body of `acme_core::iri::parse` is reported as:

```text
REFERENCE-COPY  src/iri.rs:12  demo::iri::parse_iri  matches acme_core::iri::parse  body:3f9a1c0b7d2e4a65
```

## Constraints

- The reference packages must be dependencies of the workspace; the scan does not
  clone repositories or download archives.
- Only Rust source is compared.
- A reference package whose source is not available locally, such as a registry
  package that was not downloaded, makes the scan fail with the package name, with
  no network access.
- Reference packages are not subject to `min_tokens` overrides of their own.

## Explanation

Two earlier mechanisms did this job. One compared against pinned archives of a
sibling repository's source, fetched by a script; another walked every dependency
under a package-name prefix. Both needed their own pinning. Resolving reference
packages through `cargo metadata` makes `Cargo.lock` the single pin, so the compared
revision is the one the code is built against, and no archive step exists to go
stale.

## Verification

- A fixture workspace with a path dependency listed in `reference` reports a
  reference copy for a first-party body equal to a body in that dependency.
- A fixture shows a job homed in the reference package resolves its `home`.
- A fixture naming a package outside the dependency graph fails with that name.
- A fixture shows reference code produces no groups among first-party code.
- A fixture shows the scan does not modify `Cargo.lock`.
- A fixture with a stub named by `$CARGO` (and one with a stub on `PATH` when
  `$CARGO` is unset) records the argv of the spawned process and asserts
  it equals `cargo metadata --format-version 1 --frozen --manifest-path <root>/Cargo.toml`
  with no shell involved, and that no reference package name appears on it.
- A fixture whose reference package contains a symlink to a file outside the package,
  and one with a directory-symlink loop, each fail naming the package and the file
  without reading the target; a FIFO in the package is refused without blocking.
- A fixture whose reference source file exceeds `max_file_bytes` or exceeds a
  pre-scan recursion budget (for example 500 nested `A<A<...>>`) fails with an error
  that names the file, and does not abort the process.
- A fixture with a registry reference package that is not downloaded shows the scan
  fails naming the package, with no network access.
