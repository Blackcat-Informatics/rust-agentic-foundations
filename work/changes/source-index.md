---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:source-index
title: "The first carrying glob of a binding site"
description: "A fixture pins that the binding site of an item reached through several globs is the first glob that carries it."
changes: [cap:source-index]
affects:
  - cap:source-index#verification
contract_effect: changes
items:
  - claim: "A fixture pins that the binding site of an item reached through more than one glob re-export is the first glob that carries it."
    outcome: implemented
  - claim: "The rustdoc of the data structs and options of raf-source words its non-exhaustive note as the Contract words it: adding a field or an option is not a breaking change."
    outcome: implemented
---

## Summary

No test pinned which glob re-export the source index names as the binding
site of an item that more than one glob carries. A fixture now pins the first
one. The rustdoc of the data structs and options of `raf-source` words its
non-exhaustive note as the Contract does.
