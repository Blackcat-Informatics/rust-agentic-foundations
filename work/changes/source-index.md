---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:source-index
title: "Settle the glob views by the changes that arrive, and charge the lookups of what the globs bring in"
description: "The settling reads again what the globs of a module bring in only where a companion or a blocked name needs it, the lookups are charged during the load and each query after it, and the text states the steps and growth of each shape."
changes: [cap:source-index]
affects:
  - cap:source-index#contract
  - cap:source-index#usage
  - cap:source-index#constraints
  - cap:source-index#verification
contract_effect: changes
items:
  - claim: "The settling counts each reading again of what the globs of a module bring in under a name, the glob reads it looks at and the lists it copies, the glob reads looked at while the drops are recorded, and the glob reads and routes of the lookups, and tests pin these counts for each generated shape at n, 2n and 3n."
  - claim: "A load settled in the reference mode of the tests gives the answers of a load settled out of it, over the programs of both seed families of the differential test and the committed corpus, in three orders each."
  - claim: "The settling reads again what the globs of a module bring in under a name only for a name that a view holds a companion under, a name that a module was blocked on, a module with a `pub(in path)` visibility, or the first item a class can access arriving under a name beside an item it cannot access; every other arrival is passed on as the change it brings, and a glob that no view of its module can access is given only the names a view of its module holds."
  - claim: "Each generated shape costs at most 1.5 times the instructions of the resolver from before the two visibility rules at n = 800, and none grows faster per doubling of n."
  - claim: "The answers of the index to the programs of both seed families of the differential test in 13 orders and to the committed corpus in 13 orders are those of the index before the change."
  - claim: "The lookups of what the globs of a module bring in are charged to the work budget of the load, and each query after the load is metered against `max_query_work` and refused with a typed budget error past it."
  - claim: "The Contract states the steps and the growth of each generated shape, what a step is and the cost that stays for a name with a companion."
---

## Summary

The settling of the glob views passes each change on as it arrives where no
companion and no blocked name needs the globs of a module read again, so the
shapes whose cost grew with the visibility rules cost what they did before
them. The lookups of what the globs bring in are charged during the load and
for each query after it. Counted tallies, generated shapes, a reference mode
with its equivalence test and the instruction-count harnesses show the cost
and the sameness of the answers, and the capability states the cost of each
shape.
