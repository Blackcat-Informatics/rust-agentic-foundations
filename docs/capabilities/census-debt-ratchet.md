---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-debt-ratchet
title: "Census debt ratchet"
description: "A recorded set of existing duplication that can only shrink, stored in census-debt.ttl."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:census-normal-form, cap:census-ledger]
---

# Census debt ratchet

## Purpose

A repository adopting the census usually already holds duplication it cannot remove
at once. The debt ratchet records that duplication so the gate can start failing on
anything new, and it forces the record to shrink as the duplication is removed. The
repository gets a clean gate immediately, and the debt can only go down.

## Contract

The recorded debt lives in `census-debt.ttl` at the workspace root. It names the
`normal_form` it was written for, the census groups that are accepted as debt, and
the number of copies each unenforced job carries:

```turtle
VERSION "1.2"
BASE <https://blackcatinformatics.ca/.well-known/genid/raf-census/f520ad2c373757f914c2940564d088ccd565646b5cfac051d50de557d9c107f3>
PREFIX census: <https://blackcatinformatics.ca/raf/census#>

<> a census:DebtRecord ;
    census:normalForm 1 ;
    census:reportForm 1 .

<#debt/body/demo_a::len,demo_b::len> a census:BodyDebt ;
    census:memberSymbol "demo_a::len" ,
        "demo_b::len" .

<#job/hex-encode> a census:JobDebt ;
    census:copies 3 ;
    census:jobId "hex-encode" .
```

The check compares the debt the workspace carries now with the recorded debt.
A current group whose member set is a strict subset of a recorded group's set is
shrinkage of that recorded group, not a new group.

Growth always fails:

- a group appears that is not recorded and is not a strict subset of a recorded
  group;
- a current group's member set is a superset of a recorded group's set;
- a job's copy count rises above its recorded count.

Writing the record has two entry points with one behaviour.
`cargo census debt --write` writes it for a ledger that needs no registered code.
`Gate::write_debt` writes it for every ledger, including one that needs registered
rules or extensions (see [census-rule-api](./census-rule-api.md)); it is the only
write path for those, and the CLI exits 2 naming it. The write refuses to record growth. The way out is to
remove the duplication, or to sanction it with an exact-set `[[distinct]]` entry
(see [census-ledger](./census-ledger.md)).

Shrinkage fails until it is re-recorded:

- a recorded group no longer exists, because it was resolved;
- a recorded group's member set has shrunk to a strict subset;
- a job's copy count falls below its recorded count.

The message names `cargo census debt --write` and `Gate::write_debt` as the ways to
lower the record. The write rewrites `census-debt.ttl` to exactly the debt the
workspace carries now. Recorded debt never carries slack: the record always equals the
present debt. Two cases let `debt --write` record a debt that has no earlier record. A
workspace with no `census-debt.ttl` has the file created from the present debt, which
is how a repository adopts the ratchet. A file whose `normal_form` differs from the
implemented version is replaced by the present debt, as the bump path below states.

A missing `census-debt.ttl` records no debt, so every existing group is growth until
the file is created. A file whose `normal_form` differs from the implemented version
fails hard. The bump path: after a `normal_form` bump, `cargo census migrate`, given
the index report written before the upgrade, rebuilds the ledger (see
[census-migration](./census-migration.md)), and the write, which refuses while any
non-debt finding exists, replaces the record whose `normal_form` differs with the
present debt. The check itself keeps failing hard until that write has run.

The write computes the record from the present workspace, so deleting
`census-debt.ttl` and writing again records any growth. The control is review: every
creation of `census-debt.ttl`, and every change of its `normal_form`, is read in
the diff as a statement that the listed duplication is accepted.

## Usage

`cargo census check` and `Gate::assert_clean` read `census-debt.ttl` as part of
every run. For a ledger that needs no registered code, `cargo census debt --write`
updates it (see [census-cli](./census-cli.md)). For a ledger that needs registered
code, the repository calls `Gate::write_debt` from its own xtask, which it runs as:

```text
cargo xtask census-debt
```

The file is committed, so a reduction of debt appears in review as a smaller diff.

## Examples

A workspace records the group `body [demo_a::len, demo_b::len, demo_c::len]`. A
developer removes one clone. The current group `[demo_a::len, demo_b::len]` is a
strict subset of the recorded one, so the next `check` fails with a finding that the
recorded group has shrunk, and the developer runs:

```text
$ cargo census debt --write
census-debt.ttl: 1 group, 0 jobs with copies
```

The next `check` passes. If a developer instead adds a fourth clone to the recorded
group, the current member set is a superset of the recorded one. The check fails, and
`cargo census debt --write` refuses with the message that growth cannot be recorded.
The developer removes the clone or adds a `[[distinct]]` entry for the exact set.

## Constraints

- The ratchet covers census groups and per-job copy counts. It does not cover
  detector hits that a job forbids; those are findings, not debt.
- The write refuses while the workspace has a finding that is not debt, such as a
  stale ledger row, in both entry points.
- `Gate::write_debt` is called from the repository's own binary or xtask, never from
  a `#[test]`.
- The file is RDF 1.2 Turtle in the census canonical form, read through the same
  strict closed-profile reader. Equal debt gives byte-equal files; a hand edit
  without recomputing the unkeyed content digest is refused.

## Explanation

The content-addressed BASE changes with the debt. A review diff shows the changed
base beside the changed debt entries.

A ratchet that only blocks growth leaves slack: the record says three copies, the
code has one, and two new copies can appear unnoticed. Failing on shrinkage closes
that gap. Refusing to record growth keeps the write from admitting new duplication
into an existing record; adopting the ratchet or replacing a record after a version
bump records the present debt, and review of that creation diff is what stops it
being a re-baseline of growth. Each improvement is recorded the moment it happens,
so the record is always the exact present debt, and every regression, however small,
is a visible failure.

The rule is not the census's alone. [The debt ratchet](../architecture/debt-ratchet.md)
states it once for every RAF tool that records debt, and the strict mode of the
workplan follows it with its own record and keys; this capability states the census
keys, the census file and its version bump.

## Verification

- A member added without recomputing the digest is refused as an invalid debt
  record, naming the file; an unchanged canonical record reads.

- Growth fixtures: a new group, a group whose member set is a superset of a
  recorded one, and a job copy count above the record each fail the check, and each
  still fails after `debt --write`, which refuses.
- Shrink fixtures: a resolved group, a group shrunk to a strict subset, and a job
  copy count below the record each fail the check with a message that names
  `cargo census debt --write`, and each passes after `debt --write`, with the written
  file equal to the present debt, through `cargo census debt --write` and through
  `Gate::write_debt`.
- A fixture shows a strict-subset group is treated as shrinkage and not as a new
  group.
- A fixture shows `debt --write` and `Gate::write_debt` refuse while a stale ledger
  row exists, and write nothing.
- A fixture whose ledger has a registered rule shows `Gate::write_debt` writes the
  record and `cargo census debt --write` exits 2 naming the Gate path.
- A fixture shows a record with a different `normal_form` fails the check hard, and
  is replaced by the write once the ledger is at the implemented version.
- A fixture shows `debt --write` creates the file in a workspace that has none.
- Two runs of `debt --write` produce byte-identical files.
