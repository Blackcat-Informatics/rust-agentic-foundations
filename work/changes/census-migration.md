---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:census-migration
title: "Implement census migration"
description: "Delivers cargo census migrate from legacy baselines and ledgers."
changes: [cap:census-migration]
affects:
  - cap:census-migration#purpose
  - cap:census-migration#contract
  - cap:census-migration#usage
  - cap:census-migration#examples
  - cap:census-migration#constraints
  - cap:census-migration#verification
contract_effect: changes
items:
  - claim: "migrate maps each old fingerprint to its member symbols and then to the normal form 1 fingerprint."
  - claim: "migrate also reads a current-format ledger at an older normal_form, with the index report that version wrote, remaps only body and shim fingerprints, and rewrites it at the implemented normal_form."
  - claim: "The output is a ledger in the unified schema with dialect fields in job.ext."
  - claim: "Rows that cannot be mapped are listed and kept as comments in the output."
  - claim: "The old ledger and baseline are never modified."
  - claim: "A migrated dialect ledger is validated and its debt recorded through the repository's census test and xtask, not the CLI."
  - claim: "Legacy ledger and baseline reads are bounded in file size and nesting depth."
  - claim: "The index-report read is bounded to admit reports the census writes under its load budgets rather than the ledger's document-size bound, and migration preserves max_file_bytes."
---

## Summary

This change delivers the migration command that moves legacy census ledgers to normal form 1 and the unified schema.
