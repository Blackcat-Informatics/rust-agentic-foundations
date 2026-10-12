---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-complete-gate
title: "Implement the completion gate"
description: "Delivers change complete, verify and the Scenario A and Scenario B acceptance fixtures."
changes: [cap:workplan-complete-gate]
affects:
  - cap:workplan-complete-gate#contract
  - cap:workplan-complete-gate#usage
  - cap:workplan-complete-gate#examples
  - cap:workplan-complete-gate#verification
contract_effect: changes
items:
  - claim: "change complete requires zero obligations, an outcome on every item, no PROPOSED blocks, a clean docs sync and passing bound tests and examples."
  - claim: "change complete deletes the change record as its last step."
  - claim: "A change with contract_effect none completes without usage or example edits."
  - claim: "verify runs bound tests and examples and persists no evidence."
  - claim: "Scenario A and Scenario B fixtures pass."
---

## Summary

This change delivers the completion gate, the verify command and the two acceptance scenarios.
