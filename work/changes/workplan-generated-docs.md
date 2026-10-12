---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-generated-docs
title: "Implement generated documentation blocks"
description: "Delivers the verification matrix, relationship table and catalogue blocks and docs sync."
changes: [cap:workplan-generated-docs]
affects:
  - cap:workplan-generated-docs#contract
  - cap:workplan-generated-docs#usage
  - cap:workplan-generated-docs#examples
  - cap:workplan-generated-docs#verification
contract_effect: changes
items:
  - claim: "docs sync rewrites verification matrix, relationship and catalogue blocks deterministically."
  - claim: "Sync is idempotent and leaves text outside blocks unchanged."
  - claim: "check fails on a stale generated block."
  - claim: "The catalogue groups capabilities by state and shows the gaps of implementing ones."
---

## Summary

This change delivers the three generated block kinds and a deterministic docs sync with stale-block detection in check.
