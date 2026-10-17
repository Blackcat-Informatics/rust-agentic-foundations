---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-two-way-coverage
title: "The super:: re-export limit of coverage"
description: "The Constraints state that a re-export through super:: is not normalised, which gives a false gap row and never hides a gap."
changes: [cap:workplan-two-way-coverage]
affects:
  - cap:workplan-two-way-coverage#constraints
contract_effect: changes
items:
  - claim: "The Constraints state that a re-export named through super:: is not normalised to the item it names, so it can show a gap row that is not one, and never hides a gap."
---

## Summary

Coverage strips a leading `self::` from a re-export path but not `super::`.
The effect is one-sided: such a re-export can show as a gap row, and no gap is
ever hidden by it. The Constraints now state it.
