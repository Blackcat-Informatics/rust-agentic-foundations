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
> **The command.** `cargo workplan history <id>` takes a capability, change,
> architecture, evidence or assessment id (`cap:`, `change:`, `arch:`, `evidence:` or
> `assessment:`) and prints the commits that shaped it, newest first. Each entry holds
> the commit, its date, its subject, the sources that list it and the paths of the
> document and of the bound files it touched. A commit that several sources list is one entry with all of them.
> Entries are ordered by the instant of their author date, newest first, and then by
> commit id, so the order is the same on every run.
>
> **Document history.** The history of the document comes from
> `git log --follow` on its path, so a move of the file keeps its earlier commits.
> `--name-only` lists the name the document had at each commit. `--cc` lists a merge
> whose resolution changed the document; a merge that changed nothing in it is not
> an entry. `--root` lists the commit that created the document even when a user
> sets `log.showRoot=false`.
>
> **Binding history.** For each resolved code binding, the report reads the span of
> each definition of the symbol (its file and lines) from the
> [source index](./source-index.md) and runs `git log -L` over those lines, so the
> report lists the commits that changed that symbol and not the whole file. Their
> source is the binding's path and their path is the file. A binding that names
> more than one symbol, or that the index cannot decide, is an error and no
> process starts.
>
> A binding that no longer resolves is reported with the commit that removed it
> and that commit's parent as the last commit in which it resolved. The commit is
> the newest one that changed how often the last segment of the binding's path
> occurs in the files of the directory of its crate (`git log -S`); its first
> parent comes from `%P` of the same query, and the first commit of the history
> has none. A
> crate that no package of the workspace provides has no directory, and the report
> says so with no process started.
>
> **Invocation.** Every `git` process is spawned directly with no shell and with its
> arguments as an argv vector, in the workspace root and with `LC_ALL=C`. The
> command first runs `git --version` and exits 2 when Git is older than 2.30, the
> minimum for the suite, and then `git rev-parse --is-shallow-repository`, and exits
> 2 when it answers `true`. The history commands are:
>
> ```text
> git --no-pager log --no-show-signature --follow --cc --root --format=%x1e%H%x1f%aI%x1f%s --name-only "--since=<date> 00:00:00 +0000" -- <doc-path>
> git --no-pager log --no-show-signature -L<start>,<end>:<file> --no-patch --format=%x1e%H%x1f%aI%x1f%s "--since=<date> 00:00:00 +0000" --
> git --no-pager log --no-show-signature --root -S<identifier> -n 1 --format=%H%x1f%P%x1f%aI%x1f%s -- <crate-dir>...
> ```
>
> The quotes mark one argument, not a shell: `--since=<date> 00:00:00 +0000` is one
> element of the argv vector, the start of that day in UTC. It appears only when
> `--since` was given, and it sits before the final `--` so that it is an option. `<doc-path>`, `<file>` and `<crate-dir>` are
> paths relative to the workspace root that the command takes from the loaded
> document set and the [source index](./source-index.md), never from the raw command
> line; `<crate-dir>` is `.` for a package at the root. In the first and third forms
> `--` precedes the paths. In the second the path is embedded after the colon in
> the single `-L` argument, so a path that begins with `-` is a path and not an
> option. `<identifier>` is the last segment of the binding's path without a
> generic argument list, and must match `[A-Za-z_][A-Za-z0-9_]*`. Each commit id
> `git` prints, the parents of the third form included, is accepted only when it
> matches the commit-id pattern. `--no-show-signature` keeps a user's
> `log.showSignature` from adding lines to the output, and `--root` a user's
> `log.showRoot=false` from hiding the first commit. The `<id>` argument is matched against the ids of the loaded documents
> and is never passed to `git`. `--since <date>` must match `YYYY-MM-DD` and name a
> day from 1970-01-01 to 2099-12-31, the years Git's date reader takes. It counts
> from 00:00:00 UTC of that day, so the same date selects the same commits whatever
> the clock and the time zone. `--since` limits the first two forms and not the
> lookup of a removed binding.
>
> Each record of the first two forms opens with `0x1e`, and its first line is split
> with at most two splits at `0x1f` (commit, date, then the rest as the subject; the
> third form has the parents after the commit and is split at most three times),
> because the subject is text an attacker chose and may contain any byte except a
> newline. The subject is data and is never interpreted. The lines after it in the
> first form are the paths, unquoted when `git` quoted them; any other line is an
> error. The command reads at most 16 MiB of each `git` output and fails with an
> error that says so when there is more. It runs against the workspace's
> repository and Git configuration, which is trusted build input (see
> [the security policy](../../SECURITY.md)).
>
> **Output.** The default output is a table: `| Commit | Date | Subject | Source |`,
> the commit abbreviated to seven digits, the day of the author date, the subject
> and the sources, and a second table for the bindings that no longer resolve.
> `--format json` emits the same entries, in the same order, for tools, with the
> full commit id and the author date as `git` printed it. The command is read only
> and writes no file.

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
> - The command reads Git only. It never writes the working tree or the index. In a
>   partial clone Git itself may fetch missing objects from its remote while it
>   answers, which writes into `.git`; that is Git's behaviour for such a clone.
> - A history entry always names its commit, so every line is checkable with plain Git.
> - A shallow clone produces an error that names the missing history, and never
>   a partial list presented as complete.
> - No `git` process receives text from a document or the command line except as a
>   validated path (after `--` or inside the single `-L` argument), as a validated
>   identifier inside the single `-S` argument, or as a `--since` value that matches
>   `YYYY-MM-DD` and reaches `git` as `--since=<date> 00:00:00 +0000`. A path that
>   begins with `-` is treated as a path.
> - `git log -L` follows the lines of a symbol through a rename that Git detects
>   and ends where the lines came from another file, so the history of a symbol
>   moved between files starts at the move. The span is read from the working tree
>   and traced from `HEAD`, so for a file whose working copy differs from `HEAD`
>   the lines are those numbers in `HEAD`. A bound file that is not committed, or
>   is shorter in `HEAD`, makes `git log -L` fail, and the whole history fails with
>   it rather than leaving the binding out.
> - The commit reported as removing a binding is a heuristic. The count is of the
>   text anywhere below the crate's directory, which includes any package nested in
>   it and, for a package at the root, the whole repository with its documents; so
>   a later commit that changes another occurrence of the same word, or of a word
>   that holds it, is reported instead. When the symbol left without changing the
>   count (a rename to a longer name that holds the old one, a move inside the
>   crate) the commit reported can be the one that added it, and the last commit
>   in which it resolved one in which it never existed. A removal inside a merge is
>   not seen, and a symbol whose name never changed its count there has no commit.
> - A table cell writes a control character or a bidirectional formatting
>   character of a subject as its `\u{..}` escape, so a subject cannot reorder or
>   hide the text of a row.
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
>   the report lists that commit and the one that wrote the function, and none of
>   the others.
> - A binding whose symbol was deleted: the report names the commit that removed it
>   and its parent as the last commit in which it resolved.
> - A merge fixture: a merge whose conflict resolution changed the document is in
>   its history, and a merge that changed nothing in it is not.
> - A configuration fixture: with `log.showRoot=false` set in the repository, the
>   commit that created the document, and a symbol added in it, are still found.
> - A time fixture: commits at 23:30 UTC the day before and at 00:30 and 23:30 UTC
>   on the day of `--since` give exactly the two of that day, under several `TZ`
>   values.
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
