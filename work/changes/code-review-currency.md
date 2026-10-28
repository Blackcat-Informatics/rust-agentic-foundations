---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:code-review-currency
title: Code review currency
description: Persist source-bound reviews and expose currentness, findings, coverage and required-review policy through shared records and native RDF.
changes: [cap:workplan-code-reviews, cap:okf-profile, cap:workplan-cli, cap:workplan-document-model, cap:workplan-entity-history, cap:workplan-reconcile, cap:workplan-strict, cap:rdf-report-profile]
affects:
  - cap:workplan-code-reviews#purpose
  - cap:workplan-code-reviews#contract
  - cap:workplan-code-reviews#usage
  - cap:workplan-code-reviews#examples
  - cap:workplan-code-reviews#constraints
  - cap:workplan-code-reviews#explanation
  - cap:workplan-code-reviews#verification
  - cap:okf-profile#contract
  - cap:okf-profile#usage
  - cap:okf-profile#examples
  - cap:workplan-cli#contract
  - cap:workplan-cli#usage
  - cap:workplan-cli#examples
  - cap:workplan-document-model#contract
  - cap:workplan-document-model#usage
  - cap:workplan-document-model#examples
  - cap:workplan-entity-history#contract
  - cap:workplan-entity-history#usage
  - cap:workplan-entity-history#examples
  - cap:workplan-reconcile#contract
  - cap:workplan-reconcile#usage
  - cap:workplan-reconcile#examples
  - cap:workplan-strict#contract
  - cap:workplan-strict#usage
  - cap:workplan-strict#examples
  - cap:rdf-report-profile#contract
  - cap:rdf-report-profile#usage
  - cap:rdf-report-profile#examples
contract_effect: changes
items:
  - claim: Define a closed review record and native authoring projection with source scopes, reviewer categories, verdicts and every finding disposition.
    outcome: implemented
  - claim: Derive recorded inputs from the actual committed source through the shared lazy Git tree and retain per-item currency after the reviewed object is removed.
    outcome: implemented
  - claim: Provide every review query with stable item ordering, exact changed and removed paths, current coverage and native semantic claims.
    outcome: implemented
  - claim: Expose all review operations through the real CLI with text, RDF and JSON-LD carriers, validated arguments and explicit exit classes.
    outcome: implemented
  - claim: Enforce optional required kinds through existing reconciliation and complete checks while preserving default behavior and original typed failures.
    outcome: implemented
  - claim: Integrate review identity, graph relationships, history and shared report projections without an alternative record ledger or serializer.
    outcome: implemented
  - claim: Record a genuine review of one repository crate and retain its open finding and the unreviewed remainder visibly.
    outcome: implemented
  - claim: State the delivered profile, command and policy contracts with runnable usage and current verification bindings.
    outcome: implemented
---

## Summary

A review records the source it examined and the question it answered. Persisted
item inputs distinguish unchanged code from edits, additions and removals while
retaining the original reviewed revision as provenance. Shared native source,
record, obligation and RDF facilities support the same contracts for library and
command callers. Explicit capability requirements select where current reviews
are mandatory.
