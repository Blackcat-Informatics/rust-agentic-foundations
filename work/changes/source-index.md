---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:source-index
title: "State what the order-independence fixtures and the rustc comparison assert"
description: "The source index gives one answer in every order of the declarations for the shapes with a companion item, a bare first segment and cfg alternatives; the text states what is asserted and the limits that remain."
changes: [cap:source-index]
affects:
  - cap:source-index#contract
  - cap:source-index#constraints
  - cap:source-index#verification
contract_effect: changes
items:
  - claim: "A name that two globs bring in as different items is ambiguous; where an explicit import or a glob of a module that defines the item supplies the item that another glob carries as a companion, the index gives that item in every order, where rustc's own answer depends on the order."
  - claim: "A path whose first segment is a bare name that two globs bring in as different modules is ambiguous, the cfg alternatives of one module are one candidate, and resolve_from answers Ambiguous for a reading module of a crate name that several packages share."
  - claim: "The constraints keep the limits of glob cycles in both directions and state that rustc's answer depends on the order for some glob ambiguities where the index gives one fixed answer; the companion order limit is gone."
  - claim: "The comparison with rustc probes bare first segments, checks the index against the order of the declarations, and reports the runs of 5,522, 11,044 and 22,088 programs and the classes that remain."
---

## Summary

The Contract, Constraints and Verification of the capability state exactly what
the every-order fixtures and the comparison with rustc assert: the order
independence of the companion shapes, the bare first segment, the cfg
alternatives of one module, the shared crate name for a reading module, the
limits of glob cycles that remain, and the runs of the comparison.
