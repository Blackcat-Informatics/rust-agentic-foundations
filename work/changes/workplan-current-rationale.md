---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-current-rationale
title: "Implement current rationale"
description: "Delivers architecture page conventions and conditions to revisit."
changes: [cap:workplan-current-rationale]
affects:
  - cap:workplan-current-rationale#purpose
  - cap:workplan-current-rationale#contract
  - cap:workplan-current-rationale#usage
  - cap:workplan-current-rationale#examples
  - cap:workplan-current-rationale#constraints
  - cap:workplan-current-rationale#verification
contract_effect: changes
items:
  - claim: "A shared decision has one architecture page that states the resolved current decision."
    outcome: implemented
  - claim: "Capabilities link to the architecture pages their decisions depend on."
    outcome: implemented
  - claim: "A decision page, an architecture page with a ## Decision heading, has a ## Conditions to revisit heading whose list holds at least one item."
    outcome: implemented
  - claim: "Unlinked architecture pages are reported."
    outcome: implemented
  - claim: "A decision page without conditions to revisit is reported."
    outcome: implemented
---

## Summary

This change delivers the architecture page conventions, the conditions to revisit structure and the checks for unlinked pages and decision pages without conditions.
