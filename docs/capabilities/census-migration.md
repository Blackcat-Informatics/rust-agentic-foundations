---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-migration
title: "Census migration"
description: "Move a legacy census baseline and ledger, or a ledger at an older normal form, to the implemented normal form and the unified ledger schema."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:census-normal-form, cap:census-ledger]
---

# Census migration

## Purpose

Repositories that ran an earlier census, or an earlier normal form, hold ledgers whose
fingerprints are meaningless under the implemented normal form. Migration rebuilds
those ledgers: it carries every row to the new fingerprints and the unified schema,
and it states what it could not carry. A repository adopts the shared census without
re-deriving its ledger by hand.

## Contract

`cargo census migrate (--old-baseline <legacy.json> | --old-index <index.ttl>) --ledger <old.toml> --out <new.toml>`
reads exactly one report input and writes only the required output ledger. A
legacy ledger requires its legacy JSON baseline; a current-format ledger requires
an RDF 1.2 index written under its older normal form. A missing, repeated or
mismatched report option is a usage error (exit 2). Format is never inferred.
The baseline report cannot replace an index, since it omits single-member groups.

- The index is read with `Index::from_turtle`, preserving its declared
  `census:normalForm`; migration requires that version to be older than the
  implemented one. Otherwise it exits 2 with nothing to migrate. An unsupported
  `census:reportForm` is a usage error naming both forms.
- The legacy baseline is read with `purrdf_lex::json` only in this import path.

- The old baseline maps each old fingerprint to the member symbols of its group. In
  an index report, the members of a fingerprint are the units whose `census:body` or `census:shim` names it.
- For each old fingerprint named in the old ledger, the command finds the member
  symbols in the current source, computes their fingerprint at the implemented
  `normal_form`, and writes that fingerprint in place of the old one. When the members
  disagree on the new fingerprint, it writes one fingerprint per distinct value.
- The output sets `normal_form` to the implemented version, keeps `min_tokens`,
  `roots` and the job and variant rows, and writes the unified key names.
- For a current-format ledger, only `body:` and `shim:` fingerprints are remapped.
  Everything else carries over unchanged, because it holds names and not hashes: the
  `table:` and `rule:` ids, `[[pattern]]`, `[[distinct]]`, `reference` and
  `max_file_bytes`.
- Fields that belong to a dialect, such as specifications, vectors, benchmarks,
  owners and dispositions, are written into `[job.ext]`.
- The command lists every row it could not map, with the reason: the baseline or
  index report has no entry for the fingerprint, no member symbol resolves in the current source, or a
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

The output is the repository's `helpers-ledger.toml`; it replaces the ledger of the
earlier census.

A current-format ledger at an older `normal_form` is migrated the same way, with the
index report written before the upgrade:

```text
cargo census index > index-before-upgrade.ttl
cargo census migrate --old-index index-before-upgrade.ttl --ledger helpers-ledger.toml --out helpers-ledger.new.toml
```

The output is a new file; move it over `helpers-ledger.toml` to replace the ledger.

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

At a `normal_form` bump, run `cargo census migrate` on the ledger with the index
report written before the upgrade, then the census xtask write
(`cargo xtask census-debt`), which replaces the old debt record (see
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

- The index-report reader admits reports produced under census load budgets,
  rather than imposing the ledger's 1 MiB cap. An oversized input is a typed
  error naming the file. Migration preserves `max_file_bytes`.

- Migration needs the report that lists each fingerprint's members: the legacy
  baseline, or the index report the older version wrote. A ledger fingerprint absent
  from it cannot be mapped.
- Migration reads the current source, so a row whose members were renamed or removed
  since the baseline is reported as unmapped.
- The output requires a person to write the `summary` of any row the old ledger did
  not state; the command writes an empty string and the ledger check does not accept
  an empty summary until it is filled in.
- The input ledger and legacy baseline are read with the bounds of
  [census-ledger](./census-ledger.md): at most 1 MiB each and 32 levels of nesting,
  with a larger or deeper input refused as a typed error that names the file.
- The command knows the legacy ledger layouts of the earlier census copies and the
  current layout at an older `normal_form`, and rejects any other layout with a usage
  error. A current-layout ledger already at the implemented `normal_form` is rejected
  as a usage error: there is nothing to migrate.
- A current-format ledger at an older `normal_form` needs the index report that the
  older version wrote; without it, no fingerprint can be mapped.

## Explanation

A fingerprint is a hash, so it cannot be converted by arithmetic. What survives a
change of normal form is the group: the same functions, now named by new hashes.
Mapping through member symbols uses the one thing both versions share. Listing the
unmapped rows instead of dropping them keeps the migration honest, because a row that
silently disappears is a rule that silently stops running.

## Verification

- A noncanonical index, unknown predicate or BASE that does not name its content
  is refused with the file named and no output written. Both mismatched combinations
  of ledger kind and report option exit 2; the matching neighbours succeed.

- A fixture legacy baseline and ledger migrate to an expected unified ledger, byte
  for byte.
- A fixture current-format ledger at an older `normal_form`, with the index report
  that version wrote, migrates to an expected ledger at the implemented `normal_form`,
  byte for byte, and a current-format ledger already at the implemented `normal_form`
  exits 2.
- A fixture clean job whose forbidden fingerprint only its home carries, a job that
  forbids a `rule:` id, and a ledger with a `[[pattern]]` and a `[[distinct]]` row
  each migrate with no unmapped row, the `rule:` id and the rows kept unchanged and
  the fingerprint mapped through the index report.
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
