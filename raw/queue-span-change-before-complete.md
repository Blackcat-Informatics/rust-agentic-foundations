---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:census-span-ownership
title: "Owned census collection spans"
description: "Keep caller spans valid while giving collection lexer storage an owned lifetime."
changes: [cap:census-normal-form]
affects:
  - cap:census-normal-form#contract
  - cap:census-normal-form#usage
  - cap:census-normal-form#constraints
  - cap:census-normal-form#explanation
  - cap:census-normal-form#verification
contract_effect: changes
items:
  - claim: "Collect units on an owned worker without changing the Tree contract."
    outcome: implemented
  - claim: "Propagate worker spawn failures and original panic payloads without partial results."
    outcome: implemented
  - claim: "Verify caller preservation, collection outcomes and source-storage observations."
    outcome: implemented
  - claim: "State the native ownership boundary and preserve normal-form identities."
    outcome: implemented
---

## Summary

Unit collection owns its lexical thread and its span storage. The caller retains
its own spans and Tree implementation, and receives owned units or an actionable
failure. Source-file ordinals observe caller registrations; worker-local teardown
and fixed normal-form fixtures establish their separate lifetime and identity
claims.
