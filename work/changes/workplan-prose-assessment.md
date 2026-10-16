---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-prose-assessment
title: "Implement prose assessment"
description: "Delivers assessment records for authored prose and their invalidation."
changes: [cap:workplan-prose-assessment]
affects:
  - cap:workplan-prose-assessment#contract
  - cap:workplan-prose-assessment#usage
  - cap:workplan-prose-assessment#examples
  - cap:workplan-prose-assessment#constraints
  - cap:workplan-prose-assessment#verification
contract_effect: changes
items:
  - claim: "assess prints the structured context for a section and records a verdict."
  - claim: "An assessment names the section, its verdict, who assessed by kind only (by: human or agent, never a name) and the digest of each input."
  - claim: "The id of an assessment is assessment:<capability slug>-<section> of its subject, and recording a verdict for a section replaces its record."
  - claim: "An assessment is invalidated when the section, a bound symbol or depended-on evidence changes."
  - claim: "Generated blocks are excluded from assessment."
  - claim: "The completion gate requires current assessments for the affected authored sections."
---

## Summary

This change delivers assessment records for authored sections, the input digest that invalidates them and the completion gate rule that requires them.
