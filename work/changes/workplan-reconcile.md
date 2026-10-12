---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-reconcile
title: "Implement change reconciliation"
description: "Delivers change new, change impact and change reconcile with the obligation list."
changes: [cap:workplan-reconcile]
affects:
  - cap:workplan-reconcile#contract
  - cap:workplan-reconcile#usage
  - cap:workplan-reconcile#examples
  - cap:workplan-reconcile#verification
  - cap:workplan-reconcile#constraints
contract_effect: changes
items:
  - claim: "change new creates a valid change record for a capability."
  - claim: "change impact reports the sections, bindings and dependants a change touches without writing."
  - claim: "change reconcile lists obligations of each kind with the evidence that resolves them."
  - claim: "Affected sections untouched since the merge base are found through git."
  - claim: "A change with contract_effect none receives the lighter obligation list."
  - claim: "git and cargo are spawned without a shell, the default branch is resolved to a commit id first, and document text never reaches a command line as an option."
  - claim: "Git runs from version 2.30, the default branch comes from the full ref that symbolic-ref prints, and example-failing requires proof that exactly one test ran."
---

## Summary

This change delivers opening a change, the impact report and the obligation list that reconcile produces.
