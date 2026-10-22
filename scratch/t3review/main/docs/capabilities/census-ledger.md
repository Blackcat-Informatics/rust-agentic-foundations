---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-ledger
title: "Census ledger"
description: "The helpers-ledger.toml schema: jobs, variants, distinct sets, extensions and the staleness checks."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:census-normal-form, cap:source-index]
---

# Census ledger

## Purpose

The ledger is the record of intent. It states, for each piece of functionality a
workspace implements exactly once, where that single implementation lives, what may
not appear elsewhere, and which second implementations are sanctioned and why. The
census compares the workspace against the ledger on every run, so the record cannot
drift from the code.

## Contract

The ledger is the file `helpers-ledger.toml` at the workspace root. It is read with
the `toml` crate into `serde` structures marked `deny_unknown_fields`, so an unknown
key at any level is an error that names the key and its table.

The read is bounded, because a ledger may arrive in a repository the reader does not
control: the file is at most 1 MiB, TOML nesting is at most 32 levels, and a larger
or deeper file is an error that names the file and never a panic or a stack overflow.

`[census]` holds the settings:

- `normal_form`: required; the normal form version the ledger was written for. A
  value other than the implemented version fails hard (see
  [census-normal-form](./census-normal-form.md)).
- `min_tokens`: optional; the smallest body the census fingerprints. It defaults to
  20.
- `roots`: the repository-relative directories the census walks. It defaults to the
  workspace root.
- `max_file_bytes`: optional; the largest Rust source or `Cargo.toml` file the census
  reads, for first-party and reference code alike (see
  [source-index](./source-index.md)). It defaults to 8 MiB, and a larger file is an
  error that names it.
- `reference`: the packages scanned as reference code (see
  [census-reference-scan](./census-reference-scan.md)). It defaults to none.

Each `[[job]]` is one piece of functionality. Every key except `[job.ext]` is
required, and an empty array states "none":

- `id`: the job identifier, unique in the ledger.
- `summary`: required and non-empty; what the job is, in one line. A job with an
  empty summary fails the ledger check.
- `home`: the single implementation, as `crate::path::item`. It must resolve,
  through re-exports, to a defined item.
- `entry_points`: further public paths to the same implementation. Each must
  resolve.
- `enforced`: when true, a forbidden match outside the home fails the gate. When
  false, the census still reports the copies.
- `forbidden`: a table with `names`, `constants`, `fingerprints` and `patterns`.
  `names` are regular expressions over function and constant names. `constants`
  are integer literals compared by value. `fingerprints` are census ids of kinds
  `body`, `shim`, `table` and `rule`. `patterns` are ids of `[[pattern]]` entries
  (see [census-patterns](./census-patterns.md)).
- `[[job.variant]]`: a sanctioned second implementation, with `symbol`, `file`,
  `detector`, `criterion`, `anchor` and `reason`. The symbol must resolve in the
  file, and the anchor must resolve to a documented item.
- `[job.ext]`: optional; dialect fields that belong to one repository.

`[[distinct]]` sanctions one group of items that share a shape but no job. Its keys
are `members` (two or more symbols), `kind` (`body` or `shim`), `criterion`,
`anchor` and `reason`. A row matches the census group of its kind whose member set
equals `members` exactly, so a new clone of a sanctioned shape breaks the match and
is reported.

A `LedgerExtension` validates `[job.ext]`. The census core accepts any `[job.ext]`
table only when an extension registered on the gate claims it, and the extension
returns an error for a table it rejects. With no extension registered, a non-empty
`[job.ext]` is an unknown-key error. `cargo census` registers no extension, so it
exits 2 on a non-empty `[job.ext]` and names the Gate path (see
[census-cli](./census-cli.md)).

The ledger is invalid, in the Gate and in `cargo census` alike, when a job forbids a
`rule:<id>` that is not the id of a built-in detector, of a `[[pattern]]` or of a
`Rule` registered on the gate. An unknown `rule:` id is a hard error that names the id
and the job, not a rule that silently does not run.

For each job the census classifies every matching item as one of three roles:

- `HOME`: the job's home or an entry point.
- `VARIANT`: a symbol named by a `[[job.variant]]`.
- `COPY`: any other match of the job's `forbidden` entries outside the home.

`copies` for a job is the number of `COPY` items.

The census fails with a staleness finding when:

- a member, home, entry point or variant symbol no longer resolves;
- a `body` or `shim` fingerprint matches no unit;
- a job with `enforced = false` has zero copies, because it must become enforced;
- a `[[job.variant]]` symbol is gone, or its detector no longer matches anything;
- a `[[distinct]]` row equals no current group;
- a `[[pattern]]` is listed in no job's `forbidden.patterns` (see
  [census-patterns](./census-patterns.md)).

## Usage

The ledger is edited by hand and read by `raf_census::Gate` and `cargo census`. A
ledger that needs registered code is run through the `Gate` only.
`cargo census job <ID>` prints one job's roles, `cargo census count <ID>` prints its
copy count, and `cargo census dump-ledger` prints the parsed ledger (see
[census-cli](./census-cli.md)).

## Examples

A designed ledger with one enforced job, one sanctioned variant and one distinct
set:

```toml
[census]
normal_form = 1
min_tokens = 20
roots = ["crates"]

[[job]]
id = "hex-encode"
summary = "Encode bytes as lowercase hexadecimal."
home = "demo_util::hex::encode"
entry_points = ["demo_util::encode_hex"]
enforced = true

[job.forbidden]
names = ["^hex_encode.*"]
constants = []
fingerprints = ["table:hex-lower", "rule:hex-format-loop"]
patterns = []

[[job.variant]]
symbol = "demo_wasm::hex::encode"
file = "crates/demo_wasm/src/hex.rs"
detector = "isomorphic"
criterion = "layering"
anchor = "demo_wasm::hex"
reason = "The wasm crate cannot depend on demo_util."

[[distinct]]
members = ["demo_a::len", "demo_b::len"]
kind = "shim"
criterion = "a"
anchor = "demo_a::len"
reason = "Each forwards to a different standard-library type."
```

## Constraints

- Only TOML is read. The census has no hand-written parser.
- The ledger is at most 1 MiB and 32 levels of nesting deep.
- A key not listed above is an error; there is no pass-through for unknown keys
  outside `[job.ext]`.
- `home` and variant symbols resolve through the source index, so a symbol that a
  macro generates cannot be a home.
- `[[distinct]]` covers `body` and `shim` groups only.
- Ids in `forbidden.fingerprints` carry the `normal_form` of the ledger and are
  meaningless under another version.

## Explanation

The ledger started as a dialect in each repository. One repository added
specification and benchmark fields to jobs, another added a disposition and an owner.
The unified schema keeps the core strict, so every repository reads the same file
the same way, and moves the dialect fields into `[job.ext]` where a registered
extension validates them. Strictness is the point of `deny_unknown_fields`: a
misspelled key that is silently ignored is a rule that silently does not run.

The staleness checks exist because a ledger that can only grow is a ledger that
stops being true. Every sanction is re-checked on every run, and a sanction that
matches nothing fails, so retired code takes its exemptions with it.
The exact-set rule for `[[distinct]]` makes a sanction cover precisely what a
reviewer saw, and nothing that was copied from it afterwards.

## Verification

- A parse test per table feeds an unknown key and expects an error that names the
  key and table.
- Fixtures cover each required key missing from `[[job]]`, `[[job.variant]]` and
  `[[distinct]]`.
- A staleness fixture exists for each finding: a missing member symbol, an unmatched
  `body` fingerprint, an unenforced job with zero copies, a variant symbol that is
  gone, and a `[[distinct]]` row whose member set has gained a member.
- A classification fixture shows one `HOME`, one `VARIANT` and one `COPY` for a job
  and the resulting `copies` count.
- An extension fixture shows `[job.ext]` accepted by a registered `LedgerExtension`,
  rejected by it with its message, and rejected as unknown when none is registered.
- A bound fixture: a ledger of 1 MiB plus one byte and a ledger nested 33 levels deep
  each fail with an error that names the file, and a ledger at both limits parses.
  TOML has no aliases, so no alias fixture applies.
- A `max_file_bytes` fixture: a source file above a lowered value fails naming the
  file, and the same file passes under a raised value; the key is accepted only in
  `[census]`.
- A ledger with a mismatched `normal_form` fails before any job is read.
- A fixture whose job forbids `rule:nope`, which is not a built-in detector, a
  pattern or a registered rule, fails ledger validation in the Gate and in
  `cargo census`, naming the id and the job; the same ledger passes once a `Rule`
  with that id is registered on the gate.
- A staleness fixture shows a `[[pattern]]` that no job references fails the check.
- A fixture with a `[job.ext]` table and no `[job.ext]` on another job shows the
  other job's keys are still all required.
