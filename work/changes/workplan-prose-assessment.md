---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-prose-assessment
title: "Assessments through real directories and the crate-root digest"
description: "An assessment is never written through a linked directory, and the digest of a crate-root binding is pinned."
changes: [cap:workplan-prose-assessment]
affects:
  - cap:workplan-prose-assessment#constraints
  - cap:workplan-prose-assessment#verification
contract_effect: changes
items:
  - claim: "record_assessment refuses a linked work or work/assessments directory, or one that is a file, with nothing written."
    outcome: implemented
  - claim: "The digest of a crate-root binding is unchanged by an edit of a module file below the crate root, and a test pins it."
    outcome: implemented
---

## Summary

Recording an assessment checked the record file through the directories on
the way, following a directory link inside the root, and replaced what it
found. It now refuses a linked `work` or `work/assessments` before anything is
written. A test also pins that a crate-root binding stands for its own span,
not for the files of the modules below it.
