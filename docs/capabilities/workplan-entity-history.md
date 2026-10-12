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
> **Invocation.** Every `git` process is spawned directly with no shell and with its
> arguments as an argv vector. The command first runs `git --version` and exits 2 when
> Git is older than 2.30, the minimum for the suite. The history commands are:
>
> ```text
> git --no-pager log --no-show-signature --follow --format=%H%x1f%aI%x1f%s --since=<date> -- <doc-path>
> git --no-pager log --no-show-signature -L<start>,<end>:<file> --no-patch --since=<date> --format=%H%x1f%aI%x1f%s --
> ```
>
> `--since=<date>` appears only when `--since` was given, and it sits before the final
> `--` so that it is an option. `<doc-path>` and `<file>` are repository-relative
> paths that the command takes from the loaded document set and the
> [source index](./source-index.md), never from the raw command line. In the first
> form `--` precedes the path. In the second the path is embedded after the colon in
> the single `-L` argument, so a path that begins with `-` is a path and not an
> option. `--no-show-signature` keeps a user's `log.showSignature` from adding lines
> to the output. The `<id>` argument is matched against the ids of the loaded
> documents and is never passed to `git`. `--since <date>` must match `YYYY-MM-DD`.
>
> Each output line is split with at most two splits at `0x1f` (commit, date, then the
> rest as the subject), because the subject is text an attacker chose and may contain
> any byte except a newline. The subject is data and is never interpreted. The
> command reads at most 16 MiB of `git` output and fails with an error that says so
> when there is more. It runs against the workspace's repository and Git
> configuration, which is trusted build input (see
> [the security policy](../../SECURITY.md)).
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
> - No `git` process receives text from a document or the command line except as a
>   validated path (after `--` or inside the single `-L` argument), or as a
>   `--since=<date>` value that matches `YYYY-MM-DD`. A path that begins with `-` is
>   treated as a path.
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
> - A path fixture: a document and a bound symbol whose file is named `-rf.rs` (and
>   one named `--output=x.rs`) produce their history, create no file named `x.rs`,
>   and a stub `git` on `PATH` records `log --follow ... -- -rf.rs` for the document
>   form and `-L` carrying the path after the colon for the symbol form.
> - A record fixture: a commit subject that contains `0x1f` bytes and text that looks
>   like a hash is reported whole as the subject, and output past 16 MiB is an error.
> - A Git version fixture: a stub `git --version` reporting 2.29 exits 2 before any
>   history is read.
> - An argument fixture: an `<id>` or `--since` value that begins with `-` or does not
>   match its pattern is a usage error with exit 2, and no `git` process starts.
> - A shallow clone: the command fails with an error that names the missing history.
> - The JSON output and the table carry the same entries in the same order.
