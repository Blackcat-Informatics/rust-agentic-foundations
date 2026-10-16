---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-prose-assessment
title: "Implement prose assessment"
description: "Delivers assessment records for authored prose and their invalidation."
changes: [cap:workplan-prose-assessment]
affects:
  - cap:workplan-prose-assessment#purpose
  - cap:workplan-prose-assessment#contract
  - cap:workplan-prose-assessment#usage
  - cap:workplan-prose-assessment#examples
  - cap:workplan-prose-assessment#constraints
  - cap:workplan-prose-assessment#verification
contract_effect: changes
items:
  - claim: "assess prints the structured context for a section and records a verdict."
    outcome: implemented
  - claim: "An assessment names the section, its verdict, who assessed by kind only (by: human or agent, never a name) and the digest of each input."
    outcome: implemented
  - claim: "The id of an assessment is assessment:<capability slug>-<section> of its subject, and recording a verdict for a section replaces its record."
    outcome: implemented
  - claim: "An assessment is invalidated when the section, a bound symbol that is one of its inputs or an evidence record a claim of the section names changes, or when an input is gone or new."
    outcome: implemented
  - claim: "Generated blocks are excluded from assessment."
    outcome: implemented
  - claim: "The completion gate requires a current accepted assessment for each affected authored section."
    outcome: implemented
---

## Summary

This change delivers assessment records for authored sections, the input digest that invalidates them and the completion gate rule that requires them.
