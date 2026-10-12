---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:census-ledger
title: "Implement the census ledger"
description: "Delivers the helpers-ledger.toml reader, job classification and the staleness checks."
changes: [cap:census-ledger]
affects:
  - cap:census-ledger#purpose
  - cap:census-ledger#contract
  - cap:census-ledger#usage
  - cap:census-ledger#examples
  - cap:census-ledger#constraints
  - cap:census-ledger#verification
contract_effect: changes
items:
  - claim: "The ledger is read with toml and serde under deny_unknown_fields, with the census, job, variant and distinct tables."
  - claim: "A registered LedgerExtension validates the job.ext table."
  - claim: "Matches are classified as HOME, VARIANT or COPY and counted per job."
  - claim: "Staleness findings cover missing members, unmatched fingerprints, unenforced jobs with zero copies, gone variant symbols, distinct rows that equal no group and unreferenced patterns."
  - claim: "A forbidden rule id that is not a built-in detector, a pattern or a registered rule is a hard ledger error in the Gate and the CLI."
  - claim: "A normal_form mismatch fails before any job is read."
---

## Summary

This change delivers the unified ledger schema, its strict reader, the extension hook and the staleness checks.
