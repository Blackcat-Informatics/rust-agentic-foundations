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
  - cap:source-index#constraints
contract_effect: changes
items:
  - claim: "The Tree trait has a disk and a memory implementation."
  - claim: "Package discovery finds library, binary, test, bench and example targets and marks the non-shipping ones."
  - claim: "Module following honours the path attribute and rejects targets outside the workspace root."
  - claim: "Items under cfg(test) are tagged as test code."
  - claim: "The symbol index and use-alias table resolve paths through re-exports with typed failures."
  - claim: "Every path the crate opens is canonicalised and must lie inside the canonical workspace root; non-regular files, symlinks leaving the root and directory-symlink loops are refused."
  - claim: "Files are size-capped by a configurable limit, a token pre-scan bounds delimiter, generic, prefix-operator, closure and else-if recursion, parsing runs on a dedicated stack and a committed fuzz target covers the parser, so a hostile file yields a typed error or, on a residual stack overflow, a failed run and never a pass."
  - claim: "The file type is checked on the opened handle, and a FIFO or other non-regular file is refused."
---

## Summary

This change delivers the raf-source crate with disk and memory trees, module following including path attributes, and the alias-resolving symbol index.
