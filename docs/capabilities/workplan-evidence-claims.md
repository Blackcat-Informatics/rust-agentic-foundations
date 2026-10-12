---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-evidence-claims
title: "Evidence and claims"
description: "Persisted execution evidence, the claims that depend on each result, and invalidation when the evidence changes."
state: considered
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:workplan-document-model, cap:workplan-reconcile]
---

# Evidence and claims

## Purpose

A statement such as "the parser stays under a fixed memory bound" is true only while
some execution shows it. This capability records executions as evidence and records
which current statements depend on which result, so that when a result changes, the
statements that rested on it are known and are reopened.

Open questions:

- Where evidence is stored: beside the capability, or in one directory for the
  workspace.
- How much of an execution is kept: the verdict only, or the verdict with the measured
  values.
- Whether a statement may depend on several results, and whether any one failing result
  invalidates it.

## Contract

> [!PROPOSED change:workplan-evidence-claims]
> **Evidence.** An evidence record states one execution: the command, the revision, the
> verdict and the measured values. Records are persisted as files in the repository and
> are written by `cargo workplan evidence record`.
>
> **Claims.** A claim is a statement in a capability document that depends on
> evidence. The document names the evidence a claim rests on, so the dependency is
> stated in the text and not inferred.
>
> **The dependency view.** The graph holds an edge from each claim to the evidence it
> rests on. `cargo workplan evidence claims <evidence-id>` answers which current
> statements depend on this result. `cargo workplan evidence for <cap:slug>` answers
> which results support a capability.
>
> **Invalidation.** When a record is replaced by one with a different verdict or a
> value outside the stated bound, or when the record is removed, each dependent claim
> receives an obligation in the [reconcile](./workplan-reconcile.md) list. A claim
> with no evidence is reported as unsupported.

## Usage

> [!PROPOSED change:workplan-evidence-claims]
> - `cargo workplan evidence record --id memory-bound -- cargo bench parse` runs the
>   command and persists the evidence.
> - `cargo workplan evidence claims memory-bound` lists the statements that depend on it.
> - `cargo workplan change reconcile <slug>` includes an obligation for every claim
>   whose evidence changed.

## Examples

> [!PROPOSED change:workplan-evidence-claims]
> A claim in a capability document names its evidence, and the evidence record holds
> the result:
>
> | Part | Content |
> | --- | --- |
> | Statement | Parsing a 10 MiB input stays under 64 MiB resident. |
> | Evidence id | `memory-bound` |
> | Record | command `cargo bench parse`, revision `b499578`, verdict pass, peak 41 MiB |
>
> Re-recording the evidence at a newer revision with a peak of 80 MiB makes the
> statement an obligation: the bound is still documented and no longer supported.

## Constraints

> [!PROPOSED change:workplan-evidence-claims]
> - An evidence record is a result and not a judgement. It never carries prose that
>   interprets the result.
> - A claim names its evidence explicitly. No dependency is guessed from wording.
> - Recording evidence never edits a capability document. The document changes only
>   through an authored edit that resolves the obligation.
> - Evidence for a revision other than the current one is reported as stale.

## Explanation

The documented state of a system and the supported state differ by the evidence. A
design is accepted, code exists, tests address the criteria, and executions provide
evidence, and each of these becomes true at a different time. A statement about a
bound, a latency or a limit moves from planned to supported only when an execution
shows it, and moves back when the execution stops showing it.

Persisting the evidence makes the relationship inspectable. A reader asking which
current statements depend on a result gets an answer from the graph and does not have
to search the prose. A maintainer who changes a benchmark learns which statements need
review before the change merges. This is the evidence-to-claims direction of the
[two-way coverage](./workplan-two-way-coverage.md) table.

## Verification

> [!PROPOSED change:workplan-evidence-claims]
> - A fixture with one claim and one record: the dependency view lists the claim for
>   the record and the record for the capability.
> - Replacing a passing record with a failing one produces an obligation for the claim.
> - Removing a record reports the claim as unsupported.
> - A record made at another revision is reported as stale.
> - A claim that names an evidence id that does not exist is an error naming the file.
