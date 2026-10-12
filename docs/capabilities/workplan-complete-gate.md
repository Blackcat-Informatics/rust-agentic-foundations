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
5. The bound tests and examples pass: `cargo test` filtered to the tests in
   `bindings.tests` of each listed capability, and the targets and documentation
   tests in `bindings.examples` built and run, through the invocations below.

A failing bound example also stands as an `example-failing` obligation, so conditions
1 and 5 both report it. Condition 5 is the execution and condition 1 the record of it;
the refusal lists the fault under each.

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
support: a `delivered` capability needs resolved text, no PROPOSED block, and
bindings that resolve; it binds at least one code path and one test. A capability
in `implementing` may have missing bindings, and `status`, the catalogue and the
reconcile obligations report them as gaps.

**Invocation.** Bound tests and examples run only through the argv forms below. Each
process is spawned directly with no shell and with its arguments as an argv vector.
The program is the one named by `$CARGO` when that is set, and `cargo` from `PATH`
otherwise. Every call carries `--frozen`, so completion never reaches the network or
rewrites `Cargo.lock`. The placeholders are filled only from the source index, never
from raw document text:

| Binding | Argv |
| --- | --- |
| Test, list pass | `cargo test --frozen --package=<pkg> <target> -- --list --exact -- <name>` |
| Test, run pass | `cargo test --frozen --package=<pkg> <target> -- --exact -- <name>` |
| Doc test, list pass | `cargo test --frozen --package=<pkg> --doc -- --list` |
| Doc test, run pass | `cargo test --frozen --package=<pkg> --doc -- <item-path>` |
| Example target | `cargo run --frozen --package=<pkg> --example=<example>` |

`<target>` is the resolved target of the test, written `--lib`, `--bin=<bin>` or
`--test=<test>`, so that the same name in another target is never run. `<name>` is
the resolved item's path within its crate. The `--` before `<name>` is a backstop:
libtest reads a value after it as a filter even if validation regressed.

Four patterns validate the placeholders, and a value that fails its pattern is
reported as an unresolved binding and is not passed to `cargo`:

- `<name>`: `[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*`;
- `<pkg>`, `<bin>`, `<test>` and `<example>`: `[A-Za-z0-9_][A-Za-z0-9_-]*`, which
  allows the hyphens that target names carry and never begins with `-`;
- `<item-path>`: the `<name>` pattern, widened so that an impl item written by rustdoc as `Wrapper<T>::get` can be bound. Each `::`-separated segment is `[A-Za-z_][A-Za-z0-9_]*` followed by an optional generic argument list matching `<[A-Za-z0-9_,':&]*>`, with no whitespace, and the filter never begins with `-`;
- the names in the list output, which the gate parses and never places on a command line, have the form `<relative-source-path> - <item-path> (line <n>)`, where `<relative-source-path>` matches `[A-Za-z0-9_][A-Za-z0-9_./-]*`.

**Proof that the test ran (test bindings).** Exit status 0 from `cargo test` is not proof, because
libtest reports success when a filter matches nothing. A test binding counts as passed
only when the list pass shows exactly one `<name>: test` entry (an `#[ignore]`d test
is listed as a test, so the run pass must also report it ran) and the run pass reports
`1 passed; 0 failed; 0 ignored` for that target. Zero entries, more than one entry, or
`0 passed` is an unresolved-binding failure and refuses completion, so a resolved item
that is not a `#[test]`, a test behind a disabled `cfg(feature)`, and a name that
differs from the libtest name all refuse. A doc-test binding is run by listing the doc
tests of the package, selecting the entries whose item part (the text between ` - `
and ` (line `) equals the bound item path within the crate, and then running the doc
tests with the bare item path as a whitespace-free substring filter, without
`--exact`. rustdoc splits every test argument on whitespace, so a full name such as
`src/lib.rs - foo (line 4)` can never act as one exact filter. The run output is
parsed for `test <name> ... ok` lines, and every entry selected by the list pass must
appear as `... ok`, matched strictly as the line `test <name> ... ok` with nothing
between the name and `...`. A `no_run` doc test, reported as `<name> - compile ...
ok`, was built and not run, and a `compile_fail` doc test is reported as `<name> -
compile fail ... ok`. Neither satisfies built and run, so a binding that selects one
is refused with a message that tells the author to bind a runnable doc test. No
selected entry is a refusal. A selected entry that fails, is ignored or does not
appear is a refusal. Any other doc test that the substring also matches (for example
`foo_more` for the item `foo`) is ignored by the gate and neither required to pass nor
counted, but its failure still fails the `cargo test` exit status, so the binding is
refused.

**Examples.** `cargo run` for an example target runs with its standard input closed
(null) and a timeout of 300 seconds, after which the process is killed and the binding
counts as failing. The example's own exit status decides pass or fail. The run
executes the workspace's cargo configuration as `cargo test` would, so the workspace
is trusted build input (see [the security policy](../../SECURITY.md)).

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

A refusal on the fifth condition, where a bound example no longer builds:

```text
$ cargo workplan change complete incremental-parsing
refused: 1 obligation, 1 bound example failing
cap:streaming-parser  examples  example-failing  examples/chunked.rs does not build
```

## Constraints

- `complete` runs only bindings that resolve in the source index and match their
  patterns, and counts a binding as passed only by the proof its kind requires: for a
  test binding, that exactly one test ran; for a doc-test binding, that every doc test
  the list pass selected reported `... ok`; for an example binding, that the example
  exited with status 0 within the timeout. It never passes document text to `cargo` as
  a shell string or as an option.
- `complete` runs the bound tests and examples with `cargo`, so a repository whose
  tests or examples cannot compile cannot complete a change.
- A capability with empty `bindings.tests` and `bindings.examples` lists satisfies
  the fifth condition only while it is `designed` or `considered`.
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
succeeds and leaves the capability `delivered` with an accurate capability
explanation, a real usage example, current API references, explicit limitations and
a test traceability table, and no proposal text presented as established
behaviour. A document that merely exists does not satisfy
the scenario.

Further checks:

- A refusal fixture per condition: a remaining obligation, an item without outcome, a
  leftover PROPOSED block, a stale generated block, a failing bound test and a bound
  example that fails to build each produce exit 1 and leave the record in place.
  The example refusal prints an `example-failing` obligation for the capability.
- A `contract_effect: none` fixture that completes without usage or example edits and
  still refuses when any other obligation remains.
- A fixture asserting the record is gone after success and the working tree has no
  other change.
- A binding fixture with a stub named by `$CARGO` (and one on `PATH` when `$CARGO` is
  unset) records each argv and asserts it equals the matching row of the invocation
  table with `--frozen`, `--package=` and the pinned target, and that no shell is
  involved.
- A hostile binding fixture: a `bindings.tests` entry `-Zunstable-options`, one
  `--include-ignored`, one `--exact`, and a path with a `;` or a space each yield an
  unresolved-binding finding, make `complete` refuse and `verify` exit 1, and are
  never passed to `cargo`.
- A zero-test fixture: a binding that resolves in the source index to a function that
  is not a `#[test]` runs zero tests, so `complete` refuses and `verify` exits 1; so
  does an `#[ignore]`d test, and a test whose name matches in two targets runs only
  in the pinned target.
- A doc-test fixture: a binding to a documented function with a passing doc test runs
  that doc test with the bare item path as the filter and passes only when its
  `test <full name> ... ok` line appears; the same fixture with a failing doc test
  makes `complete` refuse; a doc-test binding that matches no doc test refuses; a
  selected entry missing from the run output refuses; an item whose substring also
  matches a failing `foo_more` doc test refuses; an item `Wrapper<T>::get` binds and
  passes; a `no_run` doc test and a `compile_fail` doc test, each bound as a doc test,
  are refused with the message that asks for a runnable doc test; and an item path
  that begins with `-` is rejected by its pattern.
- An example fixture: a hyphenated example target runs, an example that reads stdin
  sees it closed, and an example that does not exit within the timeout fails the
  binding.
- A `verify` fixture with one passing and one failing bound test, asserting per-binding
  output and exit codes 0 and 1.
