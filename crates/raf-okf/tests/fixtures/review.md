---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Review
id: review:ledger-design
tags: [ledger]
timestamp: 2030-01-01T00:00:00Z
scope: [crate:ledger]
revision: 0123456789abcdef0123456789abcdef01234567
kind: design
inputs:
  - kind: symbol
    name: ledger::sync_rows
    digest: 2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae
verdict: findings
by: human
findings:
  - id: accepted
    severity: low
    location: item:ledger::sync_rows
    status: accepted
    reason: The documented boundary is intentional.
  - id: fixed
    severity: high
    location: file:src/lib.rs#L1-L3
    status: fixed
    revision: abcdef0123456789abcdef0123456789abcdef01
  - id: open
    severity: medium
    location: item:ledger::sync_rows
    status: open
  - id: spurious
    severity: informational
    location: item:ledger::sync_rows
    status: false-positive
    reason: The native precondition excludes this input.
findings_ref: docs/reviews/ledger.md
---
