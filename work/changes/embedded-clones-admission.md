---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:embedded-clones-admission
title: Admit native Source storage and the parser worker entry for embedded matching
description: Establish the worker-entry registry boundary and complete native lexing, parsing and span storage admission under the original resource account.
changes: [cap:census-embedded-clones]
affects:
  - cap:census-embedded-clones#contract
  - cap:census-embedded-clones#constraints
  - cap:census-embedded-clones#verification
contract_effect: changes
items:
  - claim: "Native parser worker entry establishes an owned source-registry boundary before registration; host spawn callbacks cannot silently invalidate registry occupancy, capacity or lifetime admission."
    outcome: implemented
  - claim: "Source lexing, parsing and span storage are admitted under the original resource account for the complete pipeline, with exhaustion refusing actionably rather than returning partial success."
---

## Summary

Discovery, proof, reporting and adoption already charge the original resource
account. Every parser worker now establishes its source-registry boundary at
entry. Native Source storage is still only partially admitted: the syntax tree
itself is not charged. This record owns that residual until the second claim
holds.
