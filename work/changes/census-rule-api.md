---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:census-rule-api
title: "Implement the census Rule trait and Gate"
description: "Delivers the Rule trait, the Gate builder and the negative-fixture requirement."
changes: [cap:census-rule-api]
affects:
  - cap:census-rule-api#purpose
  - cap:census-rule-api#contract
  - cap:census-rule-api#usage
  - cap:census-rule-api#examples
  - cap:census-rule-api#constraints
  - cap:census-rule-api#verification
contract_effect: changes
items:
  - claim: "The Rule trait has id, check and fixtures methods with the documented signatures."
  - claim: "Gate::workspace builds a gate with rules, extensions and assert_clean."
  - claim: "Gate::write_debt writes census-debt.ttl beside the ledger, refuses while a non-debt finding exists, and is called from a binary or xtask, not a test."
  - claim: "A forbidden rule id that is not a built-in detector, a pattern or a registered rule fails the gate."
  - claim: "Before running a rule the gate runs its fixtures and fails, naming the rule, on an empty positive set, an empty negative set, a negative fixture with a hit, or a positive fixture with none."
  - claim: "Duplicate rule ids fail the gate."
  - claim: "Dev-dependency usage in a #[test] compiles and runs without building the code under test."
---

## Summary

This change delivers the Rust extension point of the census and the dev-dependency gate that runs it.
