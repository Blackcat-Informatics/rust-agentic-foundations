---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-complete-gate
title: "A current accepted assessment of each affected section in change complete"
description: "Adds condition 6 to the completion gate: each authored section in affects has a current assessment with the verdict accepted, judged with the static conditions."
changes: [cap:workplan-complete-gate]
affects:
  - cap:workplan-complete-gate#contract
  - cap:workplan-complete-gate#examples
  - cap:workplan-complete-gate#verification
contract_effect: changes
items:
  - claim: "Lifecycle::complete refuses a change while an authored section in its affects has no assessment, has an assessment whose inputs moved, or has a current assessment with the verdict rejected, as Condition::Assessments (number 6) with one Failure::Assessment for each, whether the assessor was a human or an agent."
  - claim: "Every affected section of a capability that loads needs the assessment, since the profile refuses a section with only generated blocks as okf-section-empty, and contract_effect: none asks for the assessments as changes does."
  - claim: "Condition 6 is judged with the static conditions, before any bound test or example runs, and again before the writes; the refusal counts it as sections without a current accepted assessment and prints each fault once, under its obligation."
  - claim: "complete leaves the assessment records as they are: its writes are still the rewrite of the generated blocks the removal stales and the removal of the record."
---

## Summary

A change is complete only when every authored section it affects has been read
and accepted against the text, the bound code and the evidence it describes. The
completion gate gains a sixth condition: a current assessment with the verdict
`accepted` for each authored section in `affects`, judged with the other static
conditions, so a section that was edited after its assessment, or rejected, keeps
the change open.
