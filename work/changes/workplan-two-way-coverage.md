---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-two-way-coverage
title: "Implement two-way coverage"
description: "Delivers the code-to-documentation and tests-to-claims coverage reports."
changes: [cap:workplan-two-way-coverage]
affects:
  - cap:workplan-two-way-coverage#contract
  - cap:workplan-two-way-coverage#usage
  - cap:workplan-two-way-coverage#examples
  - cap:workplan-two-way-coverage#verification
contract_effect: changes
items:
  - claim: "coverage code lists public items that no capability binds."
  - claim: "coverage tests lists tests that no capability binds."
  - claim: "Exclusions carry a reason, and stale or empty exclusions are errors."
  - claim: "Visibility is read from the source index."
  - claim: "check includes both reports and fails on a gap."
---

## Summary

This change delivers the code-to-documentation and tests-to-claims reports, the exclusion file and the direction table as checks.
