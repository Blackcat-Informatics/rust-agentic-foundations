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

The census runs `cargo metadata` as a subprocess with `--frozen`: it does not
update `Cargo.lock`, and it makes no network access, so it cannot fetch a package
that is not already downloaded.

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
- A fixture with a registry reference package that is not downloaded shows the scan
  fails naming the package, with no network access.
