---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-two-way-coverage
title: "Implement two-way coverage"
description: "Delivers the code-to-documentation and tests-to-claims coverage reports."
changes: [cap:workplan-two-way-coverage]
affects:
  - cap:workplan-two-way-coverage#purpose
  - cap:workplan-two-way-coverage#contract
  - cap:workplan-two-way-coverage#usage
  - cap:workplan-two-way-coverage#examples
  - cap:workplan-two-way-coverage#constraints
  - cap:workplan-two-way-coverage#verification
contract_effect: changes
items:
  - claim: "coverage code lists public items that no capability binds."
    outcome: implemented
  - claim: "coverage tests lists tests that no capability binds."
    outcome: implemented
  - claim: "Exclusions carry a reason, and an exclusion with an empty reason, of a whole crate, or that matches nothing is an error."
    outcome: implemented
  - claim: "Visibility is read from the source index."
    outcome: implemented
  - claim: "check enforces both directions for the crates the coverage record lists in scope and fails on a gap there, and a repository with no coverage record has one coverage-record-missing finding."
    outcome: implemented
---

## Summary

This change delivers the code-to-documentation and tests-to-claims reports, the exclusion file and the direction table as checks.
