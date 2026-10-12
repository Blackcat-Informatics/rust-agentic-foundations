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

**Root discovery.** When `--root` is absent, the binary spawns exactly this command,
directly with no shell, as an argv vector. The program is the one named by `$CARGO`
when that is set (cargo sets it for a subcommand), and `cargo` from `PATH` otherwise:

```text
cargo metadata --format-version 1 --no-deps --frozen
```

`--no-deps` skips dependency resolution, which is not needed to find the root, and
`--frozen` forbids network access and any change to `Cargo.lock`. No value from a
document is placed on the command line. Even so, cargo reads the workspace's
`.cargo/config.toml` and the rustup proxy reads its `rust-toolchain.toml`, so the
workspace is trusted build input, as it is for `cargo build` (see
[the security policy](../../SECURITY.md)). The binary reads no JSON beyond the
`workspace_root` field and bounds the output at 64 MiB.

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
- Every process the binary starts, directly or through `raf-workplan`, is spawned
  with no shell and an argv vector. Each spec that spawns lists its exact
  invocation, and runs `cargo` with `--frozen`. A value taken from a document
  reaches a process only after resolution and a stated pattern, and then in one of
  three forms: after `--`; inside a single `<rev>:<path>` or `-L<start>,<end>:<path>`
  argument; or as a validated test filter (see
  [the security policy](../../SECURITY.md)).
- A slug starts with a lowercase letter or digit; any other slug is a usage error.
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
- A root-discovery fixture with a stub named by `$CARGO`, and one on `PATH` with
  `$CARGO` unset, records the argv and asserts
  it equals `cargo metadata --format-version 1 --no-deps --frozen`. A second fixture
  has a `.cargo/config.toml` with `build.rustc` set to a script, and asserts that
  root discovery does not run the script.
- A slug argument that begins with `-`, such as `-rf` or `--since=x`, is a usage
  error with exit 2 before any process is spawned, and is never read as an option by
  a spawned process.
- A read-only check: the tree is byte-identical after `check`, `status`,
  `docs preview`, `change impact` and `change reconcile`.
- A `status` fixture with one capability in each state asserting the grouping and the
  gaps listed.
- A `docs preview` fixture asserting that PROPOSED markers are absent from the output
  and the proposed text is present.
- An equivalence test: the violations from `check` equal those of
  `Gate::workspace(...).assert_clean()` on the same fixtures.
