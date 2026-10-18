---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:source-index
title: "State the cost of settling the glob views as measured and the runs of the comparison on the final code"
description: "The text names the shapes whose settling costs more than linear with their measured growth, keeps the supply search linear and charged, and reports every run of the comparison from the final code."
changes: [cap:source-index]
affects:
  - cap:source-index#contract
  - cap:source-index#constraints
  - cap:source-index#verification
contract_effect: changes
items:
  - claim: "Settling the glob views costs more than linear on three shapes, a fan-in of private globs, the prelude pattern and a hub that siblings glob and that globs them back, with the measured growth, and no budget bounds the time of these loads."
    outcome: implemented
  - claim: "The Contract does not imply that settling is linear in general; the supply search is linear and charged."
    outcome: implemented
  - claim: "Every row of the comparison with rustc is from the final code."
    outcome: implemented
---

## Summary

The Constraints state the shapes whose settling costs more than linear and how
the cost grows, the Contract stops short of saying that settling is linear, and
the Verification reports every run of the comparison from the final code.
