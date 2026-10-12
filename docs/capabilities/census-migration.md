---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-migration
title: "Census migration"
description: "Move a legacy census baseline and ledger to normal form 1 and the unified ledger schema."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:census-normal-form, cap:census-ledger]
---

# Census migration

## Purpose

Repositories that ran an earlier census hold ledgers whose fingerprints are
meaningless under normal form 1. Migration rebuilds those ledgers: it carries every
row to the new fingerprints and the unified schema, and it states what it could not
carry. A repository adopts the shared census without re-deriving its ledger by hand.

## Contract

`cargo census migrate --old-baseline <clone-baseline.json> --ledger <old.toml> --out <new.toml>`
reads a legacy baseline and a legacy ledger and writes a ledger in the unified
schema to the path given by `--out`, which is required; omitting it is a usage error (see [census-ledger](./census-ledger.md)).

- The old baseline maps each old fingerprint to the member symbols of its group.
- For each old fingerprint named in the old ledger, the command finds the member
  symbols in the current source, computes their normal form 1 fingerprint, and
  writes that fingerprint in place of the old one. When the members disagree on the
  new fingerprint, it writes one fingerprint per distinct value.
- The output sets `normal_form = 1`, keeps `min_tokens`, `roots` and the job and
  variant rows, and writes the unified key names.
- Fields that belong to a dialect, such as specifications, vectors, benchmarks,
  owners and dispositions, are written into `[job.ext]`.
- The command lists every row it could not map, with the reason: the baseline has no
  group for the fingerprint, no member symbol resolves in the current source, or a
  field is unknown. Unmapped rows are written to the output as comments so no row is
  lost.
- The command writes only the output path. It never modifies the old ledger or the
  old baseline.
- The exit code is 0 when every row maps, 1 when any row is unmapped, and 2 on a
  usage error.

## Usage

```text
cargo census migrate --old-baseline clone-baseline.json --ledger old.toml --out helpers-ledger.toml
```

After migration, a ledger with no `[job.ext]` content and no repository rule runs
`cargo census check`, which validates the new ledger, and
`cargo census debt --write` to record the debt that remains (see
[census-debt-ratchet](./census-debt-ratchet.md)).

A dialect ledger, which carries `[job.ext]` content, needs its registered extension,
so `cargo census` exits 2 on it. The repository runs its census test, which builds
the `Gate` with its extensions and rules, and records the remaining debt with its
xtask:

```text
cargo test --test census
cargo xtask census-debt
```

At a `normal_form` bump, run `cargo census migrate` on the ledger, then the census
xtask write (`cargo xtask census-debt`), which replaces the old debt record (see
[census-debt-ratchet](./census-debt-ratchet.md)).

## Examples

A legacy row:

```toml
[[job]]
id = "hex-encode"
home = "demo_util::hex::encode"
forbidden_fingerprints = ["body:0b1c2d3e4f5a6978"]
spec = "RFC 4648"
```

becomes, with the old fingerprint mapped through its members:

```toml
[[job]]
id = "hex-encode"
summary = ""
home = "demo_util::hex::encode"
entry_points = []
enforced = false

[job.forbidden]
names = []
constants = []
fingerprints = ["body:3f9a1c0b7d2e4a65"]
patterns = []

[job.ext]
spec = "RFC 4648"
```

The report ends with the unmapped rows:

```text
unmapped: job iri-parse fingerprint body:aa11bb22cc33dd44: no member symbol resolves
```

## Constraints

- Migration needs the legacy baseline that lists each fingerprint's members. A
  ledger fingerprint absent from the baseline cannot be mapped.
- Migration reads the current source, so a row whose members were renamed or removed
  since the baseline is reported as unmapped.
- The output requires a person to write the `summary` of any row the old ledger did
  not state; the command writes an empty string and the ledger check does not accept
  an empty summary until it is filled in.
- The legacy ledger and baseline are read with the bounds of
  [census-ledger](./census-ledger.md): at most 1 MiB each and 32 levels of nesting,
  with a larger or deeper input refused as a typed error that names the file.
- The command knows the legacy ledger layouts of the earlier census copies and
  rejects any other layout with a usage error.

## Explanation

A fingerprint is a hash, so it cannot be converted by arithmetic. What survives a
change of normal form is the group: the same functions, now named by new hashes.
Mapping through member symbols uses the one thing both versions share. Listing the
unmapped rows instead of dropping them keeps the migration honest, because a row that
silently disappears is a rule that silently stops running.

## Verification

- A fixture legacy baseline and ledger migrate to an expected unified ledger, byte
  for byte.
- The mapped fingerprints equal the fingerprints that `cargo census fingerprint`
  reports for the member symbols.
- A row whose fingerprint is absent from the baseline, and a row whose members no
  longer resolve, each appear in the unmapped list and in the output comments, and
  the exit code is 1.
- Dialect fields land in `[job.ext]`, and a registered extension accepts them.
- The old ledger and baseline are unchanged after the run.
- A legacy ledger of 1 MiB plus one byte and a baseline nested 33 levels deep each
  fail with an error that names the file and write no output.
- The migrated ledger passes the ledger parse with `deny_unknown_fields`, with a
  registered extension for any `[job.ext]` content.
- A fixture shows `cargo census check` exits 2 on a migrated dialect ledger and names
  the Gate path, and that the `Gate` with the registered extension accepts it.
