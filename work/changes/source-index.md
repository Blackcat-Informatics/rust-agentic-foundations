---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:source-index
title: "Implement the raf-source crate"
description: "Delivers the Tree trait, package discovery, mod-tree following and the symbol and use-alias index."
changes: [cap:source-index]
affects:
  - cap:source-index#contract
  - cap:source-index#usage
  - cap:source-index#examples
  - cap:source-index#verification
contract_effect: changes
items:
  - claim: "The Tree trait has a disk and a memory implementation."
  - claim: "Package discovery finds library, binary, test, bench and example targets and marks the non-shipping ones."
  - claim: "Module following honours the path attribute and rejects targets outside the workspace root."
  - claim: "Items under cfg(test) are tagged as test code."
  - claim: "The symbol index and use-alias table resolve paths through re-exports with typed failures."
---

## Summary

This change delivers the raf-source crate with disk and memory trees, module following including path attributes, and the alias-resolving symbol index.
