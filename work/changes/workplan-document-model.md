---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-document-model
title: "Implement the workplan document model"
description: "Delivers capability and change documents, the states, PROPOSED blocks, the graph, bindings and the check rules."
changes: [cap:workplan-document-model]
affects:
  - cap:workplan-document-model#contract
  - cap:workplan-document-model#usage
  - cap:workplan-document-model#examples
  - cap:workplan-document-model#verification
contract_effect: changes
items:
  - claim: "Capability and change documents load with the four states and slug identities."
  - claim: "PROPOSED blocks are validated against the change records that name them."
  - claim: "The graph of capabilities, sections, changes, proposals, symbols and tests is built with an acyclic depends check."
  - claim: "Bindings resolve through the source index."
  - claim: "Gate::workspace(...).assert_clean() runs every check rule."
---

## Summary

This change delivers the document model, the graph, binding resolution through the source index, the check rules and the Gate dev-dependency API.
