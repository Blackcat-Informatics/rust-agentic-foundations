---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-strict
title: "Strict mode"
description: "An opt-in strict mode of the workplan gate: examples, claim coverage, explained items and lines, fuller sections, stated reasons and a debt record that can only shrink."
changes: [cap:workplan-strict, cap:workplan-two-way-coverage, cap:workplan-complete-gate, cap:workplan-reconcile, cap:workplan-cli, cap:okf-profile, cap:workplan-generated-docs, cap:workplan-document-model, cap:source-index, cap:census-debt-ratchet]
affects:
  - cap:workplan-strict#purpose
  - cap:workplan-strict#contract
  - cap:workplan-strict#usage
  - cap:workplan-strict#examples
  - cap:workplan-strict#constraints
  - cap:workplan-strict#explanation
  - cap:workplan-strict#verification
  - cap:workplan-two-way-coverage#contract
  - cap:workplan-two-way-coverage#constraints
  - cap:workplan-complete-gate#contract
  - cap:workplan-reconcile#contract
  - cap:workplan-cli#contract
  - cap:workplan-cli#usage
  - cap:okf-profile#contract
  - cap:okf-profile#verification
  - cap:workplan-generated-docs#contract
  - cap:workplan-document-model#contract
  - cap:source-index#contract
  - cap:source-index#verification
  - cap:census-debt-ratchet#explanation
contract_effect: changes
items:
  - claim: "A coverage record with strictness: strict, a Gate::strict() caller or --strict judges the repository under strict mode, and nothing lowers it."
  - claim: "Under strict, a delivered capability binds an example and holds a verification matrix."
  - claim: "Under strict, every Contract clause and Constraints item of a delivered capability names a bound test or example in a raf:tested-by marker, and coverage claims reports each claim."
  - claim: "Under strict, every public item and test of a package that a capability binds is bound or excluded."
  - claim: "Under strict, a contract_effect: none change that alters any token of a bound public item does not complete."
  - claim: "Under strict, every section of a delivered capability holds 20 words, Usage names a bound example and Constraints lists a limitation."
  - claim: "Under strict, a dropped or moved item states its reason."
  - claim: "Under strict, Usage and Examples in affects of a contract_effect: none change must be edited."
  - claim: "coverage lines, and coverage --reverse with the same output, reports orphan lines per crate, module and item, and strict holds each package to its ceilings."
  - claim: "The strict debt in work/debt/default.md can only shrink and always equals the present debt."
  - claim: "A repository whose coverage record says strictness: strict is judged strict without a flag."
  - claim: "The source index resolves paths from the modules of every target, lists impl blocks and gives the reachability of items."
  - claim: "The RAF profile page states the strictness key, the tested-by marker, the reason of an item, the ceilings and exceptions of the coverage record and the Debt type with its seven types."
---

## Summary

This change adds strict mode to the workplan gate: a repository-level setting with a
caller floor, the strict rules for examples, claims, explained items, sections,
reasons, untouched sections and lines, the reverse coverage report with its ceilings
and exceptions, and the strict debt record that follows the debt ratchet.
