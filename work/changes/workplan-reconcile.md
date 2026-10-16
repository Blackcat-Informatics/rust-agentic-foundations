---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-reconcile
title: "Claim, evidence and assessment obligations in change reconcile"
description: "Adds claim-unsupported, evidence-stale and evidence-changed to the obligation list, the first two from the documents and the third from the record at the merge base, and assessment-missing, assessment-stale and assessment-rejected for the sections in affects."
changes: [cap:workplan-reconcile]
affects:
  - cap:workplan-reconcile#contract
  - cap:workplan-reconcile#constraints
  - cap:workplan-reconcile#verification
contract_effect: changes
items:
  - claim: "ObligationKind::ALL lists claim-unsupported, evidence-stale and evidence-changed after tense-mismatch, each with its Needed evidence text, and the contract table of the reconcile capability lists the same kinds in the same order."
    outcome: implemented
  - claim: "Workplan::reconcile raises claim-unsupported for each claim of a listed capability that names an evidence record that does not load, fails, or holds a value over its limit, at the line of the claim marker."
    outcome: implemented
  - claim: "Workplan::reconcile raises evidence-stale for each input of an evidence record that a listed capability's for list or claims reach whose digest now differs or that no longer resolves, naming the input, without Git and without a revision."
    outcome: implemented
  - claim: "Lifecycle::reconcile and Lifecycle::complete raise evidence-changed through one helper, Lifecycle::process_obligations, by reading each record a claim names at the merge base with one git cat-file blob: a pass that now fails, a value over a limit, a limit gone, or a record gone; a record there that does not read is an error."
    outcome: implemented
  - claim: "contract_effect: none leaves the three new kinds in the list."
    outcome: implemented
  - claim: "ObligationKind::ALL lists assessment-missing, assessment-stale and assessment-rejected after evidence-changed, each with its Needed evidence text, and the contract table of the reconcile capability lists the same kinds in the same order."
    outcome: implemented
  - claim: "Workplan::reconcile raises, for each authored section in affects, assessment-missing when no record has the id of the section, assessment-stale once for each input of its assessment that moved, naming the input, and assessment-rejected for a current assessment with the verdict rejected; contract_effect: none leaves the three kinds in the list."
    outcome: implemented
---

## Summary

A claim in a capability document rests on evidence records, and reconciling a
change now says what those claims owe: a claim whose evidence is missing, failing
or over its limit is unsupported; a record whose bound code moved since it was made
is stale and is recorded again; and a record that passed at the merge base and has
since failed, crossed a limit, lost a limit or gone away reopens each claim that
names it. The first two come from the documents alone; the third reads each record
once at the merge base, like the sections of `section-untouched`. Each authored
section the change affects also owes a current, accepted assessment: one that is
missing, one whose inputs moved and one that rejects the section are each listed,
naming the input that moved.
