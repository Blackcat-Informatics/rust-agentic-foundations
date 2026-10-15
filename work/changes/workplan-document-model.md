---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-document-model
title: "Records in the workplan document model"
description: "Loads the evidence, assessment and coverage records, relates them in the graph, and reports their findings."
changes: [cap:workplan-document-model]
affects:
  - cap:workplan-document-model#contract
contract_effect: changes
items:
  - claim: "Finding::KINDS holds one okf kind for each rule and each reader error of the record types and the claim markers, and each kind is produced by a named fixture."
  - claim: "Workplan::load reads the Evidence, Assessment and Coverage records from work/evidence, work/assessments and work/coverage, and reads the coverage record only at work/coverage/default.md."
  - claim: "The graph holds the records and their edges to the capabilities, sections, symbols and evidence they name."
  - claim: "Workplan::check reports the findings of the records with the file and location of each, and a repository without records reports none of them."
---

## Summary

The document model reads the three record types of the profile beside the
capability, change and architecture documents. Each record is validated through
`raf-okf`, related to what it names in the graph, and reported through the same
finding vocabulary, so the checks over evidence, assessments and coverage have one
model to read.
