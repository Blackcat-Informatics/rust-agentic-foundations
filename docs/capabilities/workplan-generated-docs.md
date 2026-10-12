---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-generated-docs
title: "Generated documentation blocks"
description: "Generated blocks for the verification matrix, relationship table and capability catalogue, and the deterministic docs sync."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:workplan-document-model]
---

# Generated documentation blocks

## Purpose

Some documentation is mechanical: tables that restate bindings, links between
capabilities, and the catalogue of what exists and in what state. A tool produces
those deterministically so that they are always correct, and people spend their time
on the prose that needs judgement.

## Contract

**Block syntax.** A generated block is delimited as follows. The kind is lowercase
kebab case. Each block is opened once, closed once and never nested.

```markdown
<!-- raf:generated:verification-matrix -->
...
<!-- raf:generated:end -->
```

**Kinds.** `raf-workplan` owns three kinds:

- `verification-matrix`: in a capability document, one row per binding in
  `bindings.tests` and `bindings.examples`, giving the bound item, its kind and the
  section of the document that it substantiates. Rows are sorted by path.
- `relationships`: in a capability document, a table of the capabilities it depends
  on, the capabilities that depend on it, and the open changes that list it, each as
  a link.
- `catalogue`: in `docs/capabilities/index.md`, the catalogue and roadmap. It groups
  capabilities by state, in the order `delivered`, `implementing`, `designed`,
  `considered`, and sorts each group by slug. An `implementing` row shows the gaps:
  bindings that are empty or do not resolve. A row links to the document and carries
  its description.

`docs/capabilities/index.md` is the only index. It has no frontmatter and carries only
generated blocks written by `docs sync`; a hand-written index is not allowed.

**Sync.** `docs sync` rewrites every generated block in the repository from the
current documents and source index. The output is a pure function of its inputs: the
same repository always yields the same bytes, with `\n` line endings, no timestamps
and stable ordering. A file with no generated block is not touched, and text outside
a block is never changed. A block of an unknown kind is an error.

**Staleness.** A block whose content differs from what `docs sync` would write is
stale. `check` fails on a stale block and names the file and the kind. `docs sync`
makes it pass.

**Mechanical versus explanatory.** Generation covers reference sections,
relationship tables, status summaries, links, indices and verification matrices.
Explanation and instructional prose are written by a person or an agent and are never
generated. The generator does not write into the seven sections outside a block.

## Usage

```sh
cargo workplan docs sync            # rewrite all generated blocks
cargo workplan check                # fails when any block is stale
cargo workplan docs preview <slug>  # print the capability as it reads once reconciled
```

A capability document that wants a verification matrix includes an empty block of
that kind in its Verification section. The first `docs sync` fills it.

## Examples

A relationship table after `docs sync`:

```markdown
<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-document-model](./workplan-document-model.md) |
| Depended on by | [workplan-reconcile](./workplan-reconcile.md) |
<!-- raf:generated:end -->
```

## Constraints

- Generated blocks sit inside a section. They do not replace a required section and do
  not count toward its being non-empty.
- Editing inside a block by hand produces a stale block, so the edit is overwritten by
  the next sync and reported by `check` first.
- Sync is idempotent: running it twice changes nothing the second time.
- Preview output is printed and never written to a file.

## Explanation

Generation is limited to what a program can get right. A table that restates the
bindings is correct by construction, and it cannot drift from the bindings it
restates. The reasons behind a design, and the instruction for using it, cannot be
derived from structure, and a generator that fabricated them would make documents
that look complete and say nothing.

Failing `check` on a stale block is what makes the tables trustworthy: a reader who
sees a verification matrix knows it was produced from the bindings at this revision.
Determinism makes that cheap to maintain, because a sync on an unchanged repository
produces no diff.

The catalogue is the roadmap. A planned capability appears as upcoming work, an
implementing one appears with its gaps, and a delivered one appears in the catalogue,
so the question "what is built and what is not" has one answer that is derived rather
than written.

## Verification

- A fixture repository synced twice: the second sync reports no change and the bytes
  are equal.
- A staleness fixture per kind: an edited row, a removed row and an added binding each
  make `check` fail naming the kind, and `docs sync` repairs it.
- A catalogue fixture with one capability in each state asserts the group order, the
  slug order inside a group and the gaps shown for an `implementing` row.
- A marker fixture for each error: nested blocks, an unclosed block, an end marker
  without a start and an unknown kind.
- A fixture where text outside a block carries a distinctive string, asserting sync
  leaves it unchanged.
- A fixture with a hand-written `index.md` containing no generated block, asserting it
  is rejected.
