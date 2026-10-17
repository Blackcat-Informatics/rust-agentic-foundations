---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-complete-gate
title: "Sections in the refusal headline, the time of an output kill and the record removal"
description: "The completion refusal counts sections for the sixth condition, a kill for too much output records the time it ran, and the record is removed through real directories only."
changes: [cap:workplan-complete-gate]
affects:
  - cap:workplan-complete-gate#contract
  - cap:workplan-complete-gate#examples
  - cap:workplan-complete-gate#constraints
  - cap:workplan-complete-gate#verification
contract_effect: changes
items:
  - claim: "The refusal headline counts sections: a section without a current accepted assessment is counted once, however many assessment obligations it has."
    outcome: implemented
  - claim: "A kill for too much output records the time it ran and the output limit in the reason of the failing binding, as a kill at the timeout records the time it ran and the time limit."
    outcome: implemented
  - claim: "change complete refuses a linked work directory before it runs or writes anything, and a linked work/changes holds no change record, so the record it removes is the file at work/changes/<slug>.md."
    outcome: implemented
---

## Summary

The refusal headline of `change complete` counted the assessment obligations
of the sixth condition as sections, so one section with several moved inputs
was counted several times. The headline now counts distinct sections, and the
Contract says how one section relates to its obligations. A bound test killed
for too much output now records the time it ran, as a timeout does. The record
is removed only through directories that are not links.
