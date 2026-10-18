---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:source-index
title: "State what the order fixtures assert, the cost of the supply search and the limit of a large fan-in"
description: "The text says what the every-order fixtures assert and where rustc varies with the order, states the dominator search and its charge, the limit of a fan-in of globs reached back by a chain, and binds the tests added since."
changes: [cap:source-index]
affects:
  - cap:source-index#contract
  - cap:source-index#constraints
  - cap:source-index#verification
contract_effect: changes
items:
  - claim: "For a name two globs bring in as different items the reader gets Ambiguous in every order; where a definition, an explicit import or a glob of a module that defines or imports the item, reached other than through the reading module, supplies an item another glob carries as a companion, the index gives that item in every order, and where rustc varies with the order the index gives one answer, the ambiguity for some shapes and the item for others."
    outcome: implemented
  - claim: "The search that decides whether a companion may be dropped is one dominator tree for each item, name and name space, each step charged to the budget of the load before it is made."
    outcome: implemented
  - claim: "A fan-in of very many globs onto one module that a chain of globs also reaches back costs more than linear in the number of globs to settle."
    outcome: implemented
  - claim: "The tests added since are bound: the read-back fixture, the linear-step tests, the companion guard and the dominator tests."
    outcome: implemented
---

## Summary

The Contract states what the every-order fixtures assert and where rustc varies
with the order, and the cost of the search for the holders of an item. The
Constraints state the limit of a fan-in of globs that a chain reaches back. The
Verification binds the tests added since and reports the runs repeated on the
latest code.
