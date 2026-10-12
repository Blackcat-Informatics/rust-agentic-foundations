---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-prose-assessment
title: "Prose assessment"
description: "Recorded assessments of explanatory prose by humans and agents, invalidated when their inputs change."
state: considered
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:workplan-document-model, cap:workplan-generated-docs, cap:workplan-complete-gate]
---

# Prose assessment

## Purpose

Reference sections, relationship tables, indices and verification matrices can be
generated. Explanations and instructional prose need an author, and they need a
reviewer. This capability records what was assessed, by whom and against which inputs,
and treats the assessment as out of date when any of those inputs change.

Open questions:

- Which verdicts an assessment may carry beyond accepted and rejected.
- Whether an assessment covers a whole section or a span within it.
- Whether an agent assessment satisfies the completion gate by itself or needs a human
  assessment beside it.

## Contract

> [!PROPOSED change:workplan-prose-assessment]
> **Mechanical and authored text.** Text inside a generated block is produced by
> `docs sync` and is never assessed. Every other section is authored and may be
> assessed.
>
> **The record.** An assessment names the section, the assessor and its kind (human or
> agent), the verdict, and the input digest. The input digest covers the section text,
> the bound code and the evidence the section depends on. `cargo workplan assess
> <cap:slug>#<section>` prints the structured context an assessor reads and writes
> the record once a verdict is given.
>
> **Invalidation.** An assessment is current while its input digest equals the digest
> computed now. When the section text, a bound symbol or a depended-on result changes,
> the assessment is invalidated and the section receives an obligation in the
> [reconcile](./workplan-reconcile.md) list.
>
> **The gate.** The [completion gate](./workplan-complete-gate.md) requires current
> assessments for the authored sections that a change lists in `affects`.

## Usage

> [!PROPOSED change:workplan-prose-assessment]
> - `cargo workplan assess cap:workplan-reconcile#explanation --context` prints the
>   section, the bound symbols and the evidence for the assessor.
> - `cargo workplan assess cap:workplan-reconcile#explanation --verdict accepted --by agent`
>   records the assessment.
> - `cargo workplan assess --list` shows each assessment and whether it is current.

## Examples

> [!PROPOSED change:workplan-prose-assessment]
> After reconciliation the tool gives an assessor a concrete task:
>
> | Obligation | Detail |
> | --- | --- |
> | Update the usage example | The final API differs from the stated API in two places. |
> | Update the lifecycle explanation | The same two differences apply. |
> | Support the memory bound | The documented bound has no current evidence. |
>
> Each resolved item ends in an assessment record, and the record stays current only
> until the section, a bound symbol or the evidence changes.

## Constraints

> [!PROPOSED change:workplan-prose-assessment]
> - An assessment never edits the section it assesses.
> - Generated blocks are excluded from assessment, so a regenerated table does not
>   invalidate the assessment of the prose around it.
> - The digest is deterministic: the same inputs always produce the same digest.
> - Humans and agents receive the same context, and the record states which of them
>   assessed.

## Explanation

Mechanical generation and explanatory writing have different jobs. A crate can produce
reference sections, tables, links and indices deterministically, and it can check them.
It cannot decide that an explanation is clear or that an example teaches the right
thing. Humans and agents do that work from the same structured context, and the crate
records what they assessed.

Recording the assessment alone is not enough, because prose and its inputs drift
apart. An assessment that survived a change to the code it describes would certify a
statement nobody has read since. Tying the assessment to a digest of its inputs makes
it expire exactly when it stops being true, and the expiry becomes an explicit
obligation. Documentation work becomes specific and reviewable: the tool names the
sections to rewrite and the evidence still missing, and an assessor resolves them.

## Verification

> [!PROPOSED change:workplan-prose-assessment]
> - An assessment is current for an unchanged section and invalidated after an edit to
>   the section text.
> - Changing a bound symbol invalidates the assessments of the sections that bind it.
> - Replacing evidence that a section depends on invalidates the assessment.
> - A regenerated generated block leaves the neighbouring assessment current.
> - The digest is identical across two runs on the same tree.
> - The completion gate rejects a change whose affected authored section has no current
>   assessment.
