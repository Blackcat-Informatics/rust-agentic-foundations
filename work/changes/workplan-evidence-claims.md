---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-evidence-claims
title: "Implement evidence and claims"
description: "Delivers persisted evidence records and the dependency of claims on results."
changes: [cap:workplan-evidence-claims]
affects:
  - cap:workplan-evidence-claims#contract
  - cap:workplan-evidence-claims#usage
  - cap:workplan-evidence-claims#examples
  - cap:workplan-evidence-claims#constraints
  - cap:workplan-evidence-claims#verification
contract_effect: changes
items:
  - claim: "evidence record persists the command, revision, verdict and values of an execution."
  - claim: "A claim names the evidence it rests on, and the graph holds that edge."
  - claim: "evidence claims lists which current statements depend on a result."
  - claim: "A changed or removed record gives each dependent claim an obligation."
  - claim: "A claim without evidence is reported as unsupported."
---

## Summary

This change delivers persisted evidence records, the claim-to-evidence dependency view and the obligations that follow an evidence change.
