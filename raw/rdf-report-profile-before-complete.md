---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:rdf-report-profile
title: "Shared canonical ground reports"
description: "One ground profile and snapshot identity for census and typed workplan results."
changes: [cap:rdf-report-profile, cap:census-normal-form, cap:workplan-cli, cap:workplan-entity-history, cap:workplan-two-way-coverage]
affects:
  - cap:rdf-report-profile#contract
  - cap:rdf-report-profile#usage
  - cap:rdf-report-profile#examples
  - cap:rdf-report-profile#constraints
  - cap:rdf-report-profile#explanation
  - cap:rdf-report-profile#verification
  - cap:census-normal-form#contract
  - cap:census-normal-form#usage
  - cap:census-normal-form#examples
  - cap:census-normal-form#verification
  - cap:workplan-cli#contract
  - cap:workplan-cli#usage
  - cap:workplan-cli#examples
  - cap:workplan-cli#constraints
  - cap:workplan-cli#verification
  - cap:workplan-entity-history#contract
  - cap:workplan-entity-history#verification
  - cap:workplan-two-way-coverage#contract
  - cap:workplan-two-way-coverage#verification
contract_effect: changes
items:
  - claim: "A shared closed ground profile renders and reads both canonical RDF formats with one snapshot identity."
    outcome: implemented
  - claim: "Census report profile 2 uses the shared ground laws while preserving normal-form identities and census domain invariants."
    outcome: implemented
  - claim: "Typed workplan finding and item-coverage projections preserve ordered fields and named source provenance."
    outcome: implemented
  - claim: "Every workplan command emits typed canonical RDF in either format, with ordered nested results, actual provenance and explicit output, diagnostic and exit semantics."
    outcome: implemented
  - claim: "JSON carriers, dependency paths and obsolete emitters are removed, and dependency and usage refusals have valid neighbours."
    outcome: implemented
  - claim: "Native RDF parser graph pins, refusal neighbors and public documentation hold the shared contract, with comparison provenance stated accurately."
    outcome: implemented
  - claim: "PurRDF supplies shared terms, parsing, traversal, lexical output, vocabularies, exact arithmetic, graph components and hashing; RAF preserves its domain grammar and identity framing."
    outcome: implemented
---

## Summary

The shared report graph owns the closed profile and snapshot identity. Native
libraries supply reusable representation, parsing, lexical output and algorithms.
Census validates its domain index and transfers owned terms through that profile;
workplan projects actual typed findings and coverage with explicit membership
ordinals and reported source-location provenance.
