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
  - cap:workplan-complete-gate#constraints
contract_effect: changes
items:
  - claim: "change complete requires zero obligations, an outcome on every item, no PROPOSED blocks, a clean docs sync and passing bound tests and examples."
    outcome: implemented
  - claim: "change complete deletes the change record as its last step."
    outcome: implemented
  - claim: "A change with contract_effect none completes without usage or example edits."
    outcome: implemented
  - claim: "verify runs bound tests and examples and persists no evidence."
    outcome: implemented
  - claim: "Scenario A and Scenario B fixtures pass."
    outcome: implemented
  - claim: "Bound tests and examples run only when they resolve in the source index, through cargo invocations that carry --frozen, use no shell, pin the target and keep document text off the option positions."
    outcome: implemented
  - claim: "A bound test counts as passed only when exactly one test ran."
    outcome: implemented
  - claim: "Doc tests are listed and run by their full libtest name."
    outcome: dropped
---

## Summary

This change delivers the completion gate, the verify command and the two acceptance scenarios.

The claim that doc tests are listed and run by their full libtest name is dropped:
rustdoc splits every test argument on whitespace, so a full name can never act as one
filter. A doc test is selected from the list pass by its item part, run with the bare
item path as a substring filter, and passes only when its whole `test <name> ... ok`
line appears.
