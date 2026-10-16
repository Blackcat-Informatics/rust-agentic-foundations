---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-generated-docs
title: "Bind the generated-block, sync and preview tests"
description: "Lists every test of the generated blocks, of sync and of preview, unit tests included, and the change lookup error, in the bindings of the generated documentation capability."
changes: [cap:workplan-generated-docs]
affects:
  - cap:workplan-generated-docs#verification
contract_effect: none
items:
  - claim: "The bindings.code of the generated documentation capability list raf_workplan::ChangeLookupError, the error a preview of an unknown or unloaded change returns."
  - claim: "The bindings.tests of the generated documentation capability list every test of the generated blocks, of sync and of preview, unit tests included, so check finds no unbound test of them."
---

## Summary

The coverage record enforces two-way coverage for raf-workplan and cargo-workplan, so every public item and test of those crates is bound by the capability that explains it. The tests of the generated blocks, sync and preview belong to this capability, and its bindings now list them; its verification matrix grows with them.
