---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:source-index
title: "Restate the growth of the settling shapes before and after the two rules"
description: "The Constraints give the growth order of each settling shape in the resolver from before the two rules and now."
changes: [cap:source-index]
affects:
  - cap:source-index#constraints
contract_effect: changes
items:
  - claim: "The Constraints give, for each shape whose settling costs more than linear, its growth before the two rules and now, as measured in instructions for a load and a few paths resolved."
    outcome: implemented
---

## Summary

The Constraints no longer say that the resolver from before the two rules was
linear on every shape; they give its growth and the present growth for each.
