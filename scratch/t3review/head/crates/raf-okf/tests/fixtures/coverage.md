---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Coverage
id: coverage:default
scope: [ledger, ledger-cli]
exclusions:
  - item: ledger::sync::tests::*
    reason: "Unit tests that guard the internals of the sync."
  - item: ledger::Row::debug_dump
    reason: Kept for debugging sessions, with no contract.
---
