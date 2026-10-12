---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-complete-gate
title: "Completion gate"
description: "change complete and verify: the five completion conditions, lighter obligations for changes that leave the contract alone, deletion of the record, and the acceptance scenarios."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:workplan-reconcile, cap:workplan-generated-docs]
---

# Completion gate

## Purpose

A change is complete when the current documentation says what the system now does,
and not when a status flag is set. `change complete` is the gate that checks it. It
makes documentation a condition of completing the work, in place of a final
checkbox that says "update docs".

## Contract

**Conditions.** `change complete <slug>` succeeds only when all five hold:

1. The obligation list from [reconcile](./workplan-reconcile.md) is empty, with
   zero obligations.
2. Every item in the record has an `outcome` of `implemented`, `dropped` or
   `moved:change:<slug>`, and a `moved` outcome names a change that exists.
3. No PROPOSED block for the change remains in any capability document.
4. `docs sync` would rewrite nothing, so every generated block is current.
5. The bound tests pass: `cargo test` filtered to the tests in `bindings.tests` of
   each listed capability.

When any condition fails, the command prints the failures, changes nothing and exits
with 1. When all hold, it deletes `work/changes/<slug>.md` and exits with 0. The
deletion is the last step and the only write. Git holds the record's history.

**Obligations by effect.** All five conditions apply to every change. The effect only
sets which obligations [reconcile](./workplan-reconcile.md) raises. A change with
`contract_effect: changes` can owe an untouched-section obligation for each section in
`affects`. A change with `contract_effect: none` records that the documented contract
is unaffected, so reconcile raises no `section-untouched` obligation for the contract,
usage and examples sections. Every other obligation kind still applies, and the
obligation list must still be empty. A small internal change does not need a new
tutorial.

**State.** Promotion to a stronger state requires reconciliation, not just a status
change. The gate refuses a listed capability whose `state` the content does not
support: a `delivered` capability needs resolved text, resolving bindings, a test and
no PROPOSED block. A capability in `implementing` may have missing bindings, and
`status`, the catalogue and the reconcile obligations report them as gaps.

**`verify [<slug>]`** runs the bound tests and the bound examples of the named change's
capabilities, or of every capability when no slug is given, and prints pass or fail for
each binding. Examples run as their target or documentation test. Evidence is not
persisted: `verify` reports the run and writes nothing. Exit code 0 means every
binding passed.

## Usage

```sh
cargo workplan change reconcile incremental-parsing
cargo workplan verify incremental-parsing
cargo workplan change complete incremental-parsing
```

`complete` is the final command. Before it, `docs preview` shows the capability as it
reads.

## Examples

A refusal, followed by success after the documentation is fixed:

```text
$ cargo workplan change complete incremental-parsing
refused: 2 obligations, 1 item without outcome, 1 stale generated block
$ cargo workplan docs sync && cargo workplan change complete incremental-parsing
completed: removed work/changes/incremental-parsing.md
```

## Constraints

- `complete` runs the bound tests with `cargo`, so a repository whose tests cannot
  compile cannot complete a change.
- A capability with an empty `bindings.tests` list satisfies the fifth condition only
  while it is `designed` or `considered`.
- The record is deleted and not archived. Recovering it is a Git operation.
- `complete` never edits a capability document; the prose is written by a person or
  an agent before the command runs.

## Explanation

Documentation is part of completing a change because the alternative is a document
that is correct at design time and drifts from the product afterwards. A change that
names the documentation it creates or alters when it is opened, and cannot finish
until each part is current, makes the maintenance step part of the work.

Obligations scale with the change. Requiring a usage example for a refactor teaches
people to write meaningless ones; letting an internal change record that it touches no
contract keeps the gate credible where the contract does move.

Deleting the record keeps the reading path short. After completion, the capability
document is the one account; the claims, the discarded options and the intermediate
states are in Git, where an entity-aware history query can find them. The rationale
that still matters has already been written into the Explanation section.

## Verification

The gate has two flagship acceptance fixtures, built as repositories in temporary
directories.

**Scenario A: several redesigns, one readable present.** One capability passes through
four redesigns by successive changes. Decisions are amended and reversed, and
interfaces and constraints evolve. At the end the fixture asserts: the document
states the active contract directly; every link resolves; no PROPOSED block or change
file is on the reading path; the Explanation section still holds the rationale; and
every earlier state is recoverable with `git show`. The test is whether a reader needs
to reconstruct the changes to understand what applies now.

**Scenario B: planning produces the documentation.** The fixture starts with a
capability in `considered` with open questions. The implementation renames an API and
drops one guarantee. `change complete` must refuse until Usage, Examples and the
bindings match the new API and the dropped claim has the outcome `dropped`. It then
succeeds and leaves the capability `delivered` with a real usage example, current API
references, explicit limitations and a test traceability table, and no proposal text
presented as established behaviour. A document that merely exists does not satisfy
the scenario.

Further checks:

- A refusal fixture per condition: a remaining obligation, an item without outcome, a
  leftover PROPOSED block, a stale generated block and a failing bound test each
  produce exit 1 and leave the record in place.
- A `contract_effect: none` fixture that completes without usage or example edits and
  still refuses when any other obligation remains.
- A fixture asserting the record is gone after success and the working tree has no
  other change.
- A `verify` fixture with one passing and one failing bound test, asserting per-binding
  output and exit codes 0 and 1.
