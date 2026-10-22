---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:census-patterns
title: "Implement declarative census patterns"
description: "Delivers the [[pattern]] tables and their six matcher forms."
changes: [cap:census-patterns]
affects:
  - cap:census-patterns#purpose
  - cap:census-patterns#contract
  - cap:census-patterns#usage
  - cap:census-patterns#examples
  - cap:census-patterns#constraints
  - cap:census-patterns#verification
contract_effect: changes
items:
  - claim: "A pattern has an id, a summary and exactly one matcher among call, path, macro, type, literal_regex and tokens."
  - claim: "The tokens matcher compares normalized tokens, with $_ matching one token or one balanced group."
  - claim: "Pattern hits report as rule:<id> and are referenced from forbidden.patterns."
  - claim: "Unknown, unreferenced and duplicate pattern ids fail the ledger check."
---

## Summary

This change delivers ledger-declared patterns, their matchers and the checks that keep pattern ids honest.
