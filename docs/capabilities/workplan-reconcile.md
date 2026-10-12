---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-reconcile
title: "Change reconciliation"
description: "change new, change impact and change reconcile: opening a change, listing what it touches, and producing the documentation obligations."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:workplan-document-model, cap:workplan-generated-docs]
---

# Change reconciliation

## Purpose

Reconciling a change means folding it into the current account of the system. The
operation takes an open change and produces a precise list of what the documentation
still owes, each with the evidence that would settle it. A person or an agent then has
a concrete writing task rather than a vague instruction to update the docs.

## Contract

**`change new <slug> --capability cap:<name>`** creates `work/changes/<slug>.md` with
the identity `change:<slug>`, the named capability in `changes`, `contract_effect:
changes`, an empty `items` list and a `## Summary` stub. It fails when the slug is
not kebab case, the file exists, or the capability does not exist. Repeating
`--capability` lists several capabilities.

**`change impact <slug>`** reads the record and the graph and prints, without
modifying anything: the capabilities in `changes`; the sections in `affects`; the
bindings of those capabilities; the capabilities that depend on them and the sections
that link to the affected sections, because those readers are affected too.

**`change reconcile <slug>`** prints the obligation list. An obligation is a
statement of work the documentation owes, with a kind, a location and the evidence
needed. The kinds are:

| Kind | Raised when | Evidence that resolves it |
| --- | --- | --- |
| `unresolved-proposal` | A PROPOSED block names the change | The block is replaced by plain contract text, or removed because the claim was dropped |
| `item-without-outcome` | An item has no `outcome` | An outcome of `implemented`, `dropped` or `moved:change:<slug>` |
| `binding-missing` | A capability the change lists binds nothing where it should, or a binding does not resolve | A binding that resolves through the source index |
| `section-untouched` | A section in `affects` has the same text as at the merge base | A diff of that section against the merge base, or an edit that records why none is needed |
| `example-failing` | A bound example target or documentation test of a capability the change lists fails to build or run | A passing `verify` run for the change, or a binding edit that removes the broken example |
| `generated-stale` | A generated block differs from what `docs sync` writes | A `docs sync` run |
| `tense-mismatch` | Delivered or implementing contract text uses proposal or forward-looking wording, or a delivered capability has a PROPOSED block | Text rewritten as the supported behaviour |

Each obligation line gives the capability, the section, the kind, the evidence, and
for text findings the line number. The list is sorted by capability, section and kind,
so equal inputs print equal output. The exit code is 0 when the list is empty and 1
when it is not.

**Merge base.** `section-untouched` compares each affected section against the
version of the file at the merge base of the current branch and the default branch,
found by running `git`. A section is touched when its heading span differs. A file
that does not exist at the merge base counts as touched.

**Invocation.** `reconcile` starts `git` directly with no shell, with arguments as an
argv vector, and takes no document text onto a command line. It first runs
`git --version` and exits 2 when Git is older than 2.30, the first release
whose `rev-parse` handles `--end-of-options`. The commands are:

```text
git symbolic-ref refs/remotes/origin/HEAD
git rev-parse --verify --end-of-options <default-ref>^{commit}
git merge-base HEAD <default-commit>
git cat-file blob <merge-base>:<path>
```

`<default-ref>` is the full ref name that the first command prints, such as
`refs/remotes/origin/main`, so a local branch named `origin/main` cannot be mistaken
for it. The remote is assumed to be named `origin`. A repository where it fails has no
default branch, which is an error and not a silent pass. The ref is resolved to a
commit id by the second command and every later command receives that id, so a branch
named like an option cannot reach `git` as an option. In the last command the path
follows `<merge-base>:` inside one argument, so it cannot be read as an option.
`<path>` is a repository-relative document path from the loaded document set. The
output of `cat-file blob` is read up to the 1 MiB document bound, and a larger blob is
an error.

`example-failing` builds and runs bound examples with the `--frozen` argv forms, the
patterns, the stdin and timeout rules, and the proof-that-it-ran rule of the
[completion gate](./workplan-complete-gate.md), for bindings that resolve in the
[source index](./source-index.md). The program is `$CARGO` when set and `cargo` from
`PATH` otherwise. The processes run the workspace's cargo and Git configuration,
which is trusted build input (see [the security policy](../../SECURITY.md)).

**`contract_effect: none`.** For a change with this effect, the obligation list is
lighter. `section-untouched` is not raised for `contract`, `usage` and `examples`,
since the change declares they are unaffected. All other kinds apply, because stale
bindings and stale generated blocks are errors regardless of the contract.

## Usage

```sh
cargo workplan change new incremental-parsing --capability cap:streaming-parser
cargo workplan change impact incremental-parsing
cargo workplan change reconcile incremental-parsing
```

An agent receives the output of `reconcile`, writes the prose each line asks for,
runs `docs sync`, and runs `reconcile` again until the list is empty.

## Examples

Output of `reconcile` for a change that renamed an API and has not yet updated
the Usage section:

```text
cap:streaming-parser  usage         section-untouched    diff Usage against the merge base
cap:streaming-parser  contract      unresolved-proposal  line 31: resolve the block for change:incremental-parsing
cap:streaming-parser  verification  binding-missing      raf_parser::Parser::push does not resolve
change:incremental-parsing  item 2  item-without-outcome  record implemented, dropped or moved:change:<slug>
```

The `binding-missing` line shows the shape of every obligation: the capability, the
section and the evidence that settles it. A bound example that stops compiling would
add a line of kind `example-failing` for the capability in the same form.

## Constraints

- `reconcile` and `impact` never write files. Only `change new` writes.
- The merge base comes from `git`; a repository with no default branch reachable from
  `HEAD` makes `section-untouched` an error and not a silent pass.
- Tense findings come from a fixed list of phrases (for example "will", "is planned",
  "proposed") and are reported as findings, not as proof. The list is defined in the
  tool and applies outside code fences and PROPOSED blocks.
- `example-failing` builds and runs bound examples with `cargo`, which is the one
  process `reconcile` starts besides `git`; it still writes no file in the
  repository.
- No `git` or `cargo` process receives document text except a validated path inside
  a `<commit>:<path>` argument, or a binding that resolved in the source index.
- An obligation never says "update the docs". It names the section and the evidence.

## Explanation

The failure this prevents is the reader who must apply amendments mentally. The
current text must state the whole contract that applies, so accepting a change cannot
end at appending "supersedes" to an old statement. An amendment can be an input to
reconciliation, but it must not become a standing instruction that every subsequent
reader has to remember.

A command that accepts a change therefore identifies and updates the affected current
statements. Reconciliation may revise a contract, remove an obsolete constraint,
change an example, adjust a code binding and invalidate some evidence. Those are
consequences of one change, and the tool treats them together.

Mechanical generation and explanatory writing have different jobs. The crate can find
what is wrong, and it can regenerate tables. Explanations and instructional prose need
authorship and assessment. The obligation list is the interface between the two: it is
specific enough to be a writing task, such as "the final API differs from the proposal
in these two places, so update the usage example and the lifecycle explanation".

## Verification

- A `change new` fixture asserts the file's keys, id and Summary, and the failures for
  a bad slug, an existing file and a missing capability.
- A `change impact` fixture asserts the printed capabilities, sections, bindings and
  dependants, and that no file changed.
- One `reconcile` fixture per obligation kind in a temporary Git repository, each
  asserting the kind, location and evidence text, and that fixing it removes the line.
  The `example-failing` fixture binds an example that does not compile, asserts the
  line, and asserts that the line is gone once the example builds and runs.
- A merge-base fixture where an affected section is edited on a branch and is not
  reported, and one where it is unchanged and is reported.
- An invocation fixture with stub `git` and `cargo` on `PATH` records each argv and
  asserts the forms above, and a default branch named `--output=x` is resolved to a
  commit id and never reaches `git` as an option.
- A Git version fixture: a stub `git --version` reporting 2.29 makes `reconcile` exit
  2 before any other process starts, and a repository with no
  `refs/remotes/origin/HEAD` is an error.
- A default-branch fixture with a local branch literally named `origin/main` beside
  `refs/remotes/origin/main` resolves the remote-tracking commit.
- A blob over 1 MiB at the merge base is an error and not a truncated read.
- An `example-failing` fixture binding a function that is not a test shows that zero
  tests run is reported as a failing binding.
- A `contract_effect: none` fixture asserting the lighter list.
- A determinism check: two runs print identical output, and the exit code is 0 only
  for an empty list.
- Scenario A and Scenario B are described in
  [the completion gate](./workplan-complete-gate.md) and use `reconcile` at each step.
