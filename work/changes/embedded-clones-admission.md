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
    outcome: implemented
---

## Summary

Discovery, proof, reporting and adoption charge the original resource account.
Every parser worker establishes its source-registry boundary at entry, and the
syntax tree that a native parse returns, the value `syn::parse2` builds in
`raf-source`'s parse step, is charged to that account by a bound computed before
the parse and reconciled against the built tree. The capability states the
bound, its measured basis and what it misses.
