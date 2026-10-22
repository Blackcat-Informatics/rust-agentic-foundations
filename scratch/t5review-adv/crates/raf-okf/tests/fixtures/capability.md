---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:ledger-sync # trailing note on the id
title: 'Ledger sync: keeps two ledgers equal'
description: "Reads two ledgers and reports the rows that differ."
tags:
  - ledger
  # kept with the tag below
  - sync
timestamp: 2026-03-04T05:06:07+02:00
state: delivered

# bindings are listed in the order code, tests, examples
bindings:
  code: [crates/ledger/src/sync.rs]
  tests:
    - crates/ledger/tests/sync.rs
    - crates/ledger/tests/diff.rs
  examples: []
depends: [cap:ledger-format]
---

# Ledger sync

## Purpose

Two ledgers should say the same thing.

## Contract

`sync` returns the rows that differ, in ledger order.

## Usage

```rust
let rows = ledger::sync(&left, &right);
assert!(rows.is_empty());
```

## Examples

A worked example lives in the test suite.

## Constraints

* A ledger holds at most 1 000 rows.
* Rows compare by value.

## Explanation

The comparison is a merge of two sorted lists.

## Verification

The bound tests cover equal ledgers, a missing row and a changed row.
