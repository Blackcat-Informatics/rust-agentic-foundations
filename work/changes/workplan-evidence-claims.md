---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-evidence-claims
title: "Implement evidence and claims"
description: "Delivers persisted evidence records and the dependency of claims on results."
changes: [cap:workplan-evidence-claims]
affects:
  - cap:workplan-evidence-claims#purpose
  - cap:workplan-evidence-claims#contract
  - cap:workplan-evidence-claims#usage
  - cap:workplan-evidence-claims#examples
  - cap:workplan-evidence-claims#constraints
  - cap:workplan-evidence-claims#explanation
  - cap:workplan-evidence-claims#verification
contract_effect: changes
items:
  - claim: "evidence record runs the command after -- or a bound test or example and persists the source of its verdict (the command text or the binding), the revision when one is given, the verdict, the measured values, the limits and the digests of its inputs."
    outcome: implemented
  - claim: "A claim names the evidence it rests on, and the graph holds that edge."
    outcome: implemented
  - claim: "evidence claims lists which current statements depend on a result."
    outcome: implemented
  - claim: "A record that passed at the merge base and now fails, holds a value over a limit, has lost a limit, or is gone gives each claim that names it an evidence-changed obligation."
    outcome: implemented
  - claim: "A claim whose evidence does not exist, fails or holds a value over its limit is reported as unsupported."
    outcome: implemented
---

## Summary

This change delivers persisted evidence records, the claim-to-evidence dependency view and the obligations that follow an evidence change.
