---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-normal-form
title: "Census normal form, fingerprints and grouping"
description: "How the RAF census normalises function bodies, fingerprints them with BLAKE3 and groups the duplicates."
state: implementing
bindings:
  code: []
  tests: []
  examples: []
depends: []
---

# Census normal form, fingerprints and grouping

## Purpose

The census finds helpers that a workspace implements more than once. Textual
comparison misses most duplicates, because two copies of one algorithm differ in
local names, borrows and clones. The normal form rewrites a function body into a
canonical token sequence, and the fingerprint names that sequence, so equal
algorithms get equal fingerprints. This capability is the foundation every other
census capability stands on.

> [!PROPOSED change:census-normal-form]
> The census is a library: a caller loads a workspace through `raf-source`, and
> the census reports which units share a fingerprint and which of them a ledger job
> forbids outside its home. The fingerprint is only comparable inside one normal-form
> version, so the version is a value the library exposes and a gate it applies to
> every version a caller declares.

## Contract

The normative definition of the normal form is
[the architecture page](../architecture/normal-form.md). This capability
implements it as follows.

- The census implements `normal_form = 1`. A body is lexed with `proc_macro2`,
  including the arguments of macro invocations, and rewritten into its structural
  form and its value form.
- The value form drops borrows, derefs, `ref` bindings, the value adapters listed
  in `VALUE_ADAPTERS` (`as_str`, `as_ref`, `as_slice`, `as_mut`, `as_deref`,
  `borrow`, `borrow_mut`, `clone`, `cloned`, `copied`, `to_owned`), full-range
  reindexing, `let` type annotations and type-only turbofish.
- A body with fewer than 30 structural tokens keeps its literals verbatim (the
  literal tier). A longer body has its literals abstracted.
- A body with 6 to 29 tokens also has a shim form, in which its own parameters are
  renamed. `MIN_SHIM_TOKENS` is 6 and `SHIM_MAX_TOKENS` is 30. The `shim_key`
  excludes methods of trait implementations and `Self::` forwarders.
- A body with fewer than `min_tokens` structural tokens has no body fingerprint.
  `min_tokens` defaults to 20 and `[census] min_tokens` sets it.
- A fingerprint is `<kind>:` followed by an identifier. For `body` and `shim` the
  identifier is the lowercase hex of the first 8 bytes of the BLAKE3 hash of the
  form. For `table` and `rule` it is a fixed name. The kinds are `body`, `shim`,
  `table` and `rule`.
- The census groups units by fingerprint in a `BTreeMap` from fingerprint to
  members. A group has two or more members. Groups and members are sorted, so the
  same source always produces the same output.
- Integer constants are compared by value in any spelling. The census recognises
  hex-digit tables and reports them as `table:hex-lower` and `table:hex-upper`.
- The census renders two JSON reports. The baseline report lists every group. The
  index report lists every unit with its fingerprints.
- Any change that can alter a fingerprint bumps `normal_form`. A ledger whose
  `normal_form` differs from the implemented version fails hard, and the message
  names `cargo census migrate`.

> [!PROPOSED change:census-normal-form]
> - `raf_census::NORMAL_FORM` is 1. `raf_census::check_normal_form(declared)`
>   accepts 1 and refuses every other version with `Error::Version`, whose message
>   names the declared version, the implemented version and `cargo census migrate`.
>   Reading a version out of a ledger file is the ledger's.
> - `raf_census::CensusOptions` is `#[non_exhaustive]` and has `Default` and two
>   setters. `min_tokens` defaults to 20, and the setter refuses 0 with
>   `Error::Options`. `max_file_bytes` defaults to 8 MiB and is the per-file read cap
>   of the workspace load; a file above it is a load error that names the file.
> - `raf_census::Error` is the one error type: `Source` (file, line and reason of a
>   slice that does not lex or a signature or initialiser that does not parse),
>   `Options`, `Collision` (two different forms with one fingerprint, naming both
>   symbols), `Version` and a load error of `raf-source`. Nothing falls back to a
>   partial result.
> - Units are collected once per source item, whatever number of targets reach it,
>   so a module declared by both a library and a binary yields one unit. A unit is
>   a function, a method (inherent, trait implementation or trait default), a
>   constant or a static; its identity is its file, line and symbol, its symbol is
>   chosen deterministically (a shipping reach over a test reach, then the smallest
>   path), and it is shipping when any target that reaches it ships.
> - A body is a shim candidate when it is one expression (no `let`, no `;` before
>   its tail), contains a call, does not mention `self` and has 6 to 29 structural
>   tokens. A shim candidate keeps its `body` fingerprint when it reaches
>   `min_tokens`, which the index lists, but it never joins a body group: only its
>   `shim` fingerprint groups, and only when its shim key is present. The shim key
>   is absent for a method of a trait implementation and for a forwarder that calls
>   through `Self::`.
> - A job (`raf_census::Job`) has an id, a home symbol, entry-point symbols, variant
>   symbols and a forbidden set of fingerprints and integer constants.
>   `Census::match_jobs` returns one report per job. A unit matches when its `body`
>   or `shim` fingerprint or its `table:` id is a forbidden fingerprint, or when one
>   of its integer constants equals a forbidden constant by value. A match is `Home`
>   when the unit's symbol is the home or an entry point, `Variant` when it is a
>   listed variant, and `Copy` otherwise, and the job counts its copies. A `rule:`
>   fingerprint matches no unit here.
> - The baseline report has the members `normal_form`, `min_tokens`, `units` (the
>   count), `body_groups`, `shim_groups`, `hex_tables` and `shared_constants`. The
>   index report has `normal_form`, `units` (symbol, package, file, line, shipping,
>   tokens, body and shim), `groups` (kind, fingerprint and members) and `jobs`
>   (id, home, matches with symbol and class, and copies). Both reports are closed
>   schemas that read back what they write, and they render with sorted keys and a
>   trailing newline.
> - Groups sort by member count descending, then tokens descending, then fingerprint
>   ascending, and their members by symbol, file and line.

## Usage

The census runs as part of `raf_census::Gate` and through `cargo census`. The
interactive commands that print what this capability computes are `baseline`,
`index` and `fingerprint`, described in [census-cli](./census-cli.md).

A fingerprint is used in two places. A ledger job lists it under
`forbidden.fingerprints` so that a copy outside the job's home fails the gate (see
[census-ledger](./census-ledger.md)). A reader compares fingerprints in a baseline
to see which functions the census considers the same.

> [!PROPOSED change:census-normal-form]
> A caller loads the workspace with `raf_census::Census::load(&tree, roots,
> &options)`, normalises single sources with `raf_census::Normalizer`, and asks
> `Census::match_jobs` which units a set of `Job` values forbids. `check_normal_form`
> is the gate a ledger reader applies to the version it finds. `cargo census` and the
> rule API are the designed consumers of this library.

## Examples

These two functions are one algorithm in the value form, and the census puts them
in one `body` group:

```rust
fn total(items: &[u64]) -> u64 {
    let mut sum = 0;
    for item in items.iter() {
        sum += *item;
        if sum > 1_000_000 { sum = 1_000_000; }
    }
    sum
}

fn add_up(values: &[u64]) -> u64 {
    let mut acc: u64 = 0;
    for v in values.iter() {
        acc += *v;
        if acc > 1_000_000 { acc = 1_000_000; }
    }
    acc
}
```

The designed baseline report has this shape:

```json
{
  "normal_form": 1,
  "min_tokens": 20,
  "groups": [
    {
      "fingerprint": "body:3f9a1c0b7d2e4a65",
      "members": [
        { "symbol": "demo::total", "file": "src/a.rs", "line": 1 },
        { "symbol": "demo::add_up", "file": "src/b.rs", "line": 1 }
      ]
    }
  ]
}
```

The designed index report lists every unit:

```json
{
  "normal_form": 1,
  "units": [
    { "symbol": "demo::total", "file": "src/a.rs", "line": 1, "tokens": 31,
      "body": "body:3f9a1c0b7d2e4a65", "shim": null }
  ]
}
```

> [!PROPOSED change:census-normal-form]
> The baseline report of a workspace with one body group and one shim group names
> `normal_form`, `min_tokens`, a unit count, the two group lists, the hex tables and the
> shared constants; the index report of the same workspace lists each unit with its
> `body` and `shim` fingerprints, each group with its members, and each job with its
> classified matches. The fingerprints and keys are those printed by the
> `baseline` example of the crate.

## Constraints

- The census reads Rust source only. Generated code and code behind macros that
  expand to items are compared as written, not as expanded.
- Macro arguments are re-lexed as token streams. A macro whose arguments are not
  valid Rust tokens is compared by the tokens `proc_macro2` yields.
- A body shorter than `min_tokens` is never a `body` group, however many copies
  exist. Forwarders of 6 to 29 tokens are caught by the shim form instead.
- Fingerprints are 64 bits. The census treats a collision between different
  forms as a possibility the normal form version does not exclude, and a ledger
  that names a fingerprint still lists member symbols that a reader can check.
- The `table` and `rule` kinds carry names, not hashes.

> [!PROPOSED change:census-normal-form]
> - A unit that several targets reach is one unit; the census never counts a shared
>   module once per target.
> - A shim candidate never joins a body group, so a 20-token forwarder is judged by
>   its shim form alone.
> - A slice that does not lex, a signature that does not parse and an initialiser that
>   does not parse are errors that name the file and line; the census does not skip
>   them.
> - Integer constants and the fingerprints of a constant or static come from its
>   initialiser expression only, never from its type or attributes.
> - Nesting depth is bounded by the delimiter budget of `raf-source`.

## Explanation

Seven copies of one helper census had diverged across the Blackcat repositories,
each with its own normal form. A fingerprint recorded in one repository meant
nothing in another, and a change to a normaliser silently moved every ledger row.
Version 1 unifies them from the strongest parts of each: the structural form,
value form and literal tier of the largest copy, and the `shim_key` refinement that
stops trait-implementation methods and `Self::` forwarders from forming false
groups.

Versioning the form is what makes the ledger trustworthy. A fingerprint is only
comparable inside one version, so the version is written in the ledger and the
census refuses a mismatch instead of reporting spurious drift. Golden tests turn
the version-bump rule from a convention into a check: a normaliser change that
moves a fingerprint fails until the version moves with it.

Literals are kept below 30 tokens because short bodies are mostly constants, and
abstracting them would merge unrelated tables. Longer bodies abstract literals
because there the algorithm, not the constants, is what two copies share.

## Verification

- Golden tests pin the fingerprint of every fixture snippet under
  `normal_form = 1`, with at least one fixture per form: structural, value, literal
  tier, shim and macro arguments. A change that moves a pinned value fails.
- A fixture pair differing only in local names, borrows, clones and a `let` type
  annotation fingerprints equal; a pair differing in a method name or a field name
  fingerprints different.
- A fixture of two 25-token bodies that differ only in a string literal
  fingerprints different, and the same pair at 35 tokens fingerprints equal.
- Shim fixtures cover the 5, 6, 29 and 30 token boundaries, and show that trait
  implementation methods and `Self::` forwarders produce no shim group.
- A fixture with a `0x`-spelled and a decimal-spelled constant yields one constant
  value.
- A grouping test shows groups of one are dropped, and that two runs over the same
  source give byte-identical baseline and index reports.
- A ledger fixture with `normal_form = 2` fails with a message that names
  `cargo census migrate`.

> [!PROPOSED change:census-normal-form]
> - `check_normal_form` is tested with 0 and 2 (refused, the message naming both
>   versions and `cargo census migrate`) and with 1 (accepted).
> - `CensusOptions` is tested with `min_tokens` 0 (refused) and 1 (accepted), with the
>   defaults, and with a file above and a file at `max_file_bytes`.
> - Golden tests pin the fingerprint of every fixture, and fail when a fixture has no
>   pinned value or a pinned value has no fixture.
> - A job fixture over one workspace yields a `Home` for the home, a `Home` for an
>   entry point, a `Variant`, a `Copy`, and a `Copy` for a unit holding the forbidden
>   constant by value, with a copy count of 2, and a job that matches nothing has no
>   matches.
> - Shim fixtures cover the 5, 6, 29 and 30 token boundaries, a two-statement and a
>   `let` body without a shim form, a trait-implementation method and a `Self::`
>   forwarder each beside a neighbour that does group, and a module reached by two
>   targets as one unit.
> - The reports are tested for byte-identical output over two fresh loads, for a
>   round trip through their JSON, and for refusal of an unknown key.
