---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-cli
title: "cargo workplan command line"
description: "The cargo workplan subcommands, options and exit codes."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:workplan-complete-gate, cap:workplan-reconcile, cap:workplan-generated-docs]
---

# cargo workplan command line

## Purpose

`cargo workplan` is the interactive surface of the workplan. The library checks
everything in a test; the binary is how a person or an agent opens a change, reads
what the documentation owes, and completes the work.

## Contract

The binary is `cargo-workplan`, a thin wrapper over `raf-workplan`, parsed with clap
derive. It runs from any directory inside the workspace and finds the root from
`cargo metadata`.

| Command | Effect |
| --- | --- |
| `check` | Run every rule of the [document model](./workplan-document-model.md); print each violation |
| `status` | Print the roadmap: planned, implementing with gaps, delivered |
| `docs sync` | Rewrite the generated blocks ([generated documents](./workplan-generated-docs.md)) |
| `docs preview <slug>` | Print the capabilities of the change as they read once reconciled |
| `change new <slug> --capability cap:<name>` | Create a change record |
| `change impact <slug>` | Print what the change touches |
| `change reconcile <slug>` | Print the obligation list ([reconcile](./workplan-reconcile.md)) |
| `change complete <slug>` | Run the [completion gate](./workplan-complete-gate.md) and delete the record |
| `verify [<slug>]` | Run the bound tests and examples |

`status` groups capabilities by state, lists for each `implementing` capability the
bindings that are missing or do not resolve, as gaps, and names the open changes. `docs preview` applies the
PROPOSED blocks of the named change to a copy of the text, removes the markers,
and prints the result; it never writes a file.

**Exit codes.** 0 means success: no violations, an empty obligation list, a completed
change, or every binding passing. 1 means findings: violations, obligations, a refused
completion or a failing binding. 2 means the tool could not run: a usage error, an
unreadable file, a missing `git`, a slug that does not exist.

**Options.** `--root <dir>` sets the workspace root, `--format text|json` selects the
output, and `-q` suppresses everything but findings. Findings are printed to stdout in
a stable order and diagnostics to stderr.

## Usage

```sh
cargo workplan check
cargo workplan status
cargo workplan change new incremental-parsing --capability cap:streaming-parser
cargo workplan docs preview incremental-parsing
cargo workplan change complete incremental-parsing
```

## Examples

```text
$ cargo workplan status
delivered     2  alpha-store, beta-index
implementing  1  streaming-parser (gap: bindings.tests is empty)
designed      3  ...
$ echo $?
0
```

`--format json` prints one object per finding with `kind`, `path`, `section` and
`message`, for scripts and agents.

## Constraints

- The binary holds no logic of its own. Each command calls a function in
  `raf-workplan`, so the dev-dependency `Gate` and the CLI agree.
- `check`, `status`, `docs preview`, `change impact` and `change reconcile` write
  nothing. `docs sync` writes only generated blocks. `change new` writes one file.
  `change complete` deletes one file.
- A command that takes a slug fails with exit 2 when `work/changes/<slug>.md` does
  not exist, except `change new`.

## Explanation

Library and binary are separate so that the check runs where it cannot be skipped, in
`cargo test`, and the lifecycle commands are available where a person is working. The
exit codes distinguish "the repository has findings" from "the tool failed", which a
script and a hook need in order to react correctly.

The command set mirrors the lifecycle. `change new` opens, `impact` and `reconcile`
read, `docs preview` shows the outcome, `verify` runs the evidence available, and
`complete` closes. `status` is the roadmap, derived from the states of the documents
rather than kept by hand.

## Verification

- An argument-parsing test per command and option, including `--capability`
  repetition and `--format`.
- A fixture repository per command asserting stdout, files written and the exit code,
  with exit 0, 1 and 2 each exercised: a clean check, a finding, and a missing slug.
- A read-only check: the tree is byte-identical after `check`, `status`,
  `docs preview`, `change impact` and `change reconcile`.
- A `status` fixture with one capability in each state asserting the grouping and the
  gaps listed.
- A `docs preview` fixture asserting that PROPOSED markers are absent from the output
  and the proposed text is present.
- An equivalence test: the violations from `check` equal those of
  `Gate::workspace(...).assert_clean()` on the same fixtures.
