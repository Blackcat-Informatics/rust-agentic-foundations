---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:census-debt-ratchet
title: "Implement the census debt ratchet"
description: "Delivers census-debt.json and the check that lets debt only shrink."
changes: [cap:census-debt-ratchet]
affects:
  - cap:census-debt-ratchet#purpose
  - cap:census-debt-ratchet#contract
  - cap:census-debt-ratchet#usage
  - cap:census-debt-ratchet#examples
  - cap:census-debt-ratchet#constraints
  - cap:census-debt-ratchet#verification
contract_effect: changes
items:
  - claim: "The check fails on a new group, a group whose member set grows to a superset of a recorded group, and a rising copy count."
  - claim: "debt --write refuses to record growth; the fix is to remove the duplication or sanction it with an exact-set distinct entry."
  - claim: "The check fails on a disappearing or shrinking group and on a falling copy count until debt --write re-records it."
  - claim: "A current group whose member set is a strict subset of a recorded group is shrinkage, not a new group."
  - claim: "A missing record means no recorded debt, and debt --write creates it."
---

## Summary

This change delivers the debt record and the two-sided check that keeps recorded debt exactly equal to present debt.
