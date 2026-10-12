---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Architecture
id: arch:normal-form
title: "The census normal form, version 1"
description: "The normative definition of how the RAF census turns a function body into a fingerprint."
---

# The census normal form, version 1

The census finds duplicated helpers by comparing what function bodies say, not how
they are spelled. The normal form is the rewrite that makes two spellings of one
algorithm identical, and the fingerprint is the short name of the result. This
page is the normative definition of version 1. The ledger declares it as
`normal_form = 1`, and every fingerprint a ledger names is a version 1
fingerprint.

The capability that applies this definition is
[census-normal-form](../capabilities/census-normal-form.md). The OKF document
shape of this page is stated in [the profile](./okf-profile.md).

## Token source

A body is lexed with `proc_macro2`, so the census sees tokens, not text. Comments
and whitespace are not tokens. The arguments of macro invocations are token trees
too, and the census re-lexes them, so `format!("{x:02x}", x = b)` and a call with
the same shape compare by their tokens like any other code.

## Structural form

The structural form renames what a body names for itself and keeps what it names
for others.

- Local bindings and the function's own parameters are renamed by order of first
  appearance.
- Field names, method names, paths, types, keywords and operators are kept.
- Delimiters and punctuation are kept, with their nesting.

Two bodies that differ only in the names of their locals have the same structural
form.

## Value form

The value form drops the reference and adapter noise that does not change the
value an expression computes. These tokens are removed before fingerprinting:

- borrows (`&x`, `&mut x`, `&&x`), derefs (`*x`) and `ref` bindings;
- a value adapter called with no arguments, from this closed list
  (`VALUE_ADAPTERS`): `as_str`, `as_ref`, `as_slice`, `as_mut`, `as_deref`,
  `borrow`, `borrow_mut`, `clone`, `cloned`, `copied`, `to_owned`;
- a full-range reindex (`x[..]`);
- the type annotation of a `let` binding;
- a turbofish that names types only (`::<T>`); a turbofish with a const argument
  is kept;
- the reference marker inside an index, so `m[&k]` is `m[k]`.

The binary `&`, `&&` and `*` operators and their compound assignments are kept,
because they change the value.

## Literal tier

A body with fewer than 30 structural tokens keeps its literals verbatim. At that
size the constants are most of what the body says: two `match` arms that map the
same variants to different strings are two tables, and two that map them to the
same strings are one table written twice. A body of 30 tokens or more has its
literals abstracted to their kind. Integer literals are compared by value, in any
spelling, so `0xcbf2_9ce4_8422_2325` and its decimal spelling are one constant.

## Shim form

A shim is a short body that forwards to another function. A body with at least
`MIN_SHIM_TOKENS` (6) and fewer than `SHIM_MAX_TOKENS` (30) structural tokens also
has a shim form. In the shim form the function's own parameters are renamed and
the callee and its fixed arguments are kept, so two functions that forward to the
same place with the same fixed arguments are the same shim.

The `shim_key` that groups shims excludes two cases:

- methods in a trait implementation, whose signature a trait fixes, so their
  forwarding is not a free choice;
- forwarders that call through `Self::`, which forward within one type.

Bodies shorter than 6 tokens have no shim form, so `{ f(x) }` never forms a group.

## Thresholds

| Name | Value | Meaning |
| --- | --- | --- |
| `min_tokens` | 20 by default, set by `[census] min_tokens` | A body with fewer structural tokens than this has no body fingerprint. |
| literal tier | 30 | A body with fewer structural tokens keeps its literals verbatim. |
| `MIN_SHIM_TOKENS` | 6 | A body with fewer tokens has no shim form. |
| `SHIM_MAX_TOKENS` | 30 | A body with this many tokens or more has no shim form. |

`min_tokens` is a ledger setting. The other three are fixed by the normal form
version.

## Fingerprint

A fingerprint is `<kind>:` followed by an identifier.

- For the kinds `body` and `shim`, the identifier is the lowercase hexadecimal
  encoding of the first 8 bytes of the BLAKE3 hash of the form's token sequence,
  16 characters.
- For the kinds `table` and `rule`, the identifier is a fixed name, such as
  `table:hex-lower`, `table:hex-upper` or `rule:std-default-hasher`.

The four kinds are:

| Kind | What it names |
| --- | --- |
| `body` | A function body in its value form. |
| `shim` | A forwarder in its shim form. |
| `table` | A recognised table, such as a hex-digit table. |
| `rule` | A hit of a detector, a declarative pattern or a `Rule`. |

## Version-bump rule

Any change that can alter a fingerprint of any input is a new normal form version.
That covers the token source, every form above, every threshold, the kind prefixes,
the hash function and the width of the digest. The change bumps the `normal_form`
number, and the census keeps the old behaviour out of the new version. A change
that cannot alter any fingerprint, such as a faster implementation, does not bump
the version.

## Golden-test rule

The census keeps a fixture set of source snippets together with the fingerprint
each one has under the current version. A golden test recomputes every fingerprint
and compares it to the pinned value. A change to the normal form that moves a
pinned fingerprint fails the test until the version is bumped and the pinned values
are regenerated in the same change. The fixtures cover each form: structural,
value, literal tier, shim, and macro arguments.

## Compatibility

A ledger names the version it was written for with `normal_form`. If the value
differs from the version the census implements, the census fails hard before
reading any job. The message states the ledger's version and the census's version
and tells the user to run `cargo census migrate`, which maps the old fingerprints to
the new ones. The census never reads a ledger under a different version, and never
guesses at a mapping.
