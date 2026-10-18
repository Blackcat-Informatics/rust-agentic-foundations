---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:source-index
title: "State the runs of the rustc comparison on the final code, its controls and the cost of the supply search"
description: "The text reports only figures measured on the final code for both seed families, states what holds of the unclassified programs, names the controls of the comparison and the cost of the search for an independent supply of an item, and binds the tests added since."
changes: [cap:source-index]
affects:
  - cap:source-index#contract
  - cap:source-index#verification
contract_effect: changes
items:
  - claim: "The runs of the comparison with rustc are reported for both seed families on the final code, and the comparison with the resolver from before the two rules is stated only with figures read from the logs of those runs."
  - claim: "Every program still unclassified in the runs also diverges on the resolver from before the two rules, which is a glob or import cycle that the index settles differently from rustc, and the test fails on the larger runs while those shapes exist."
  - claim: "The comparison is controlled by RAF_ORACLE_SEED_BASE, RAF_ORACLE_PROGRAMS and RAF_ORACLE_LIST, and the default run draws from two seed families."
  - claim: "A chain of globs costs what each link brings in, once per link, and the search for an independent supply of an item is cached and charged, with a test that its steps grow linearly in the chain."
  - claim: "The tests added since are bound: the three every-order fixtures of the glob cycle beside a glob that holds the item from elsewhere, the reading of the seed base, and the count of the supply search."
---

## Summary

The Contract, Constraints and Verification of the capability report the runs of
the comparison with rustc only with figures measured on the final code, in both
seed families, and state what holds of the programs that remain unclassified.
They name the controls of the comparison, state the cost of a chain of globs and
of the search for an independent supply of an item, and bind the tests added
since.
