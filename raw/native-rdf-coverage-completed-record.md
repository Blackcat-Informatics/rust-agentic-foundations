---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:native-report-coverage
title: "Selected native report coverage"
description: "Repository coverage includes Workplan, its CLI and native RDF reports without exclusions."
changes: [cap:workplan-two-way-coverage]
affects:
  - cap:workplan-two-way-coverage#verification
contract_effect: none
items:
  - claim: "The repository coverage proof pins all three selected packages and requires complete reports without exclusions."
    outcome: implemented
  - claim: "The verification text names the actual selected scope and preserves outside-scope gap reporting."
    outcome: implemented
---

## Summary

The repository selects the native report crate alongside Workplan and its CLI.
Its actual coverage proof requires the exact package scope, empty exclusions,
complete reports in both directions and no coverage finding. The tool's public
coverage contract is unchanged; its verification text states this concrete
repository configuration.
