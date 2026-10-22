---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:ledger-rows
title: "Rows carry a stable key"
description: Gives every ledger row a key that survives a rewrite.
tags: [ledger, rows]
changes:
  - cap:ledger-sync
  - cap:ledger-format
affects:
  - cap:ledger-sync#contract
  - cap:ledger-format#purpose # the format page also moves
  - cap:ledger-format#constraints
contract_effect: changes
items:
  - claim: "Rows are keyed by a content digest."
    outcome: implemented
  # the next two have no outcome yet
  - claim: 'The key is shown in "diff" output.'
  - claim: "Old ledgers are upgraded in place."
    outcome: moved:change:ledger-upgrade
  - claim: Keyed rows replace positional rows.
    outcome: dropped
---

## Summary

Rows gain a key, so `sync` no longer depends on row order.

## Notes

The key is a digest of the row's text.
