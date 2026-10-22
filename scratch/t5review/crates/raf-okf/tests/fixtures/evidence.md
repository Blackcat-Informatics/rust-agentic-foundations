---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Evidence
id: evidence:ledger-sync-bound
timestamp: 2026-03-01T12:00:00Z
subject: command
inputs:
  - kind: symbol
    name: ledger::sync::apply
    digest: 2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae
  # the row type is read as well
  - kind: symbol
    name: ledger::Row
    digest: fcde2b2edba56bf408601fb721fe9b5c338d10ee429ea04fae5511b68fbf8fb9
verdict: pass
by: agent
for: [cap:ledger-sync]
command: "cargo bench --bench sync"
revision: 0123456789abcdef0123456789abcdef01234567
values:
  rows_per_second: 120000
  peak_mib: 41.5 # resident
limits:
  peak_mib: 64
---
