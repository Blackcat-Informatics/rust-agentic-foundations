---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:registered-census-check
title: "Run repository-owned census checks through native responses"
description: "Transport complete registered Gate outcomes through a closed native RDF check protocol."
changes: [cap:census-cli, cap:census-rule-api]
affects:
  - cap:census-cli#purpose
  - cap:census-cli#contract
  - cap:census-cli#usage
  - cap:census-cli#examples
  - cap:census-cli#constraints
  - cap:census-cli#verification
  - cap:census-rule-api#constraints
  - cap:census-rule-api#examples
contract_effect: changes
items:
  - claim: "Explicit package and example selection executes the repository-owned registered check through the bounded frozen Cargo invocation in the workspace root."
    outcome: implemented
    reason: "Actual frozen repository targets run at the requested root from another caller directory; the direct Gate shares exact registration source."
  - claim: "Native canonical check responses preserve complete Gate finding order and multiplicity or its evaluation error under a fixed protocol IRI."
    outcome: implemented
    reason: "Native Graph owns the closed canonical response; executable codec controls retain finding order, repeated messages and errors."
  - claim: "Missing or failed targets, invalid or oversized responses, unknown versions and exit disagreement refuse without built-in fallback."
    outcome: implemented
    reason: "Actual absent and broken targets refuse with Cargo causes; executable protocol controls refuse malformed bytes, wrong versions and exit disagreement without fallback."
  - claim: "Actual registered Rule, LedgerExtension and configured native provider fixtures reach the shipped CLI with valid and refusing neighbours."
    outcome: implemented
    reason: "Actual Rule and Extension parity passes; a real DiskTree locked public Hasher target passes and an unlisted competitor is a named finding through the same CLI."
  - claim: "The read-only check response never authorizes a debt write."
    outcome: implemented
    reason: "Every real registered invocation preserves the checked inputs and creates no debt; the fixed check protocol has no write request or write implementation."
---

## Summary

The repository target owns its Gate registration and locked-source inputs.
The command owns bounded execution and closed response validation.
