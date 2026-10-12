---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-entity-history
title: "Entity history"
description: "A history command for a capability or decision and its bindings, built on git log --follow and -L."
state: considered
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:workplan-document-model, cap:source-index]
---

# Entity history

## Purpose

The current documentation states what applies now and carries no chronology. Some
readers still need to know how a capability or a decision came to be what it is. This
capability answers that question on request, from Git, so that the answer costs the
asking reader and nobody else.

Open questions:

- Whether a capability's history merges the history of its document with the history of
  its bound symbols into one timeline, or reports them as separate lists.
- How a rename of a capability slug is followed, since the identity is carried by the
  file name and the `id` key together.
- How much of each commit the report shows by default.

## Contract

> [!PROPOSED change:workplan-entity-history]
> **The command.** `cargo workplan history <id>` takes a capability, change or
> architecture id and prints the commits that shaped it, newest first. Each entry holds
> the commit, its date, its subject and the paths it touched.
>
> **Document history.** The history of the document comes from
> `git log --follow` on its path, so a move of the file keeps its earlier commits.
>
> **Binding history.** For each resolved code binding, the report reads the span of the
> symbol from the [source index](./source-index.md) and runs `git log -L` over
> those lines, so the report lists the commits that changed that symbol and not the
> whole file. A binding that no longer resolves is listed with the last commit in which
> it did.
>
> **Output.** The default output is a table. `--format json` emits the same entries
> for tools. The command is read only and writes no file.

## Usage

> [!PROPOSED change:workplan-entity-history]
> A maintainer asks how a design reached its present form:
>
> - `cargo workplan history cap:workplan-reconcile` lists the document commits and the
>   commits of each bound symbol.
> - `cargo workplan history cap:workplan-reconcile --since 2026-01-01` limits the range.
> - `cargo workplan history arch:normal-form` reports the history of a shared decision.
>
> An agent that needs the reasoning behind a contract runs the command in its own
> investigation. Nothing in the capability document needs to change for this.

## Examples

> [!PROPOSED change:workplan-entity-history]
> A table for one capability with one bound function:
>
> | Commit | Date | Subject | Source |
> | --- | --- | --- | --- |
> | 3f9c2a1 | 2026-05-02 | Reject duplicate obligations | document |
> | b7d014e | 2026-04-18 | Group obligations by section | `raf_workplan::reconcile` |
> | 91ac5d0 | 2026-03-30 | Add the reconcile capability | document |

## Constraints

> [!PROPOSED change:workplan-entity-history]
> - The command reads Git only. It never writes the working tree or the index.
> - A history entry always names its commit, so every line is checkable with plain Git.
> - A shallow clone that cannot answer a query produces an error that says so, and never
>   a partial list presented as complete.
> - The command does not alter what the capability documents say. History is reached on
>   request and is never embedded in a document.

## Explanation

Current documentation and history have different readers. The reader who wants to
change or use a capability needs the complete contract that applies and does not need
the proposals that preceded it. The reader who is investigating a surprising decision
needs the sequence. Keeping the sequence in Git, and giving that reader a command to
reach it, lets the first reader work without carrying the second reader's context.

`git log --follow` suits the document because a capability keeps one permanent path,
and the flag keeps the trail through a move. `git log -L` suits the bindings because
the question about a symbol is which commits changed it, and a whole-file log buries
that answer in unrelated edits. Both are commands that a maintainer can repeat by hand,
so the report is a convenience and not a second source of truth.

The distinction between current rationale and historical deliberation (see
[current rationale](./workplan-current-rationale.md)) is what this command completes:
the document keeps the reasons that still govern, and the history command reaches the
sequence of proposals, discarded names and reversals that Git retains.

## Verification

> [!PROPOSED change:workplan-entity-history]
> - A fixture repository where a document is renamed: the report includes the commits
>   from before the rename.
> - A fixture where a function changes in one commit among several that touch its file:
>   the report lists that commit only.
> - A binding whose symbol was deleted: the report names the last commit in which it
>   resolved.
> - A shallow clone: the command fails with an error that names the missing history.
> - The JSON output and the table carry the same entries in the same order.
