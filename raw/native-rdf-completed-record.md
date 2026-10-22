---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:native-rdf-reports
title: "Native canonical RDF reports"
description: "Ground report storage and canonical RDF 1.2 formats use native PurRDF with closed Census validation."
changes: [cap:rdf-report-profile, cap:census-normal-form]
affects:
  - cap:rdf-report-profile#purpose
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
contract_effect: changes
items:
  - claim: "The closed ground profile retains native parsed datasets, constructs through native mutable membership, and exposes native-backed subject and term views."
    outcome: implemented
  - claim: "Both canonical RDF 1.2 formats share one normalized Turtle-body identity, exact owner normalization and actionable canonical-byte refusals."
    outcome: implemented
  - claim: "Census report layout 2 uses native graph views for its complete closed shapes and semantic checks, preserving normal-form 1 fingerprint bytes and start-byte identities."
    outcome: implemented
  - claim: "Canonical fixtures, registry-revision graph pins, actual report examples and the report-read fuzz target exercise the native consumer path and refusal neighbors."
    outcome: implemented
  - claim: "The public native report contract, workspace inventory, dependency closure and license policy agree with the implemented APIs."
    outcome: implemented
---

## Summary

The native report kernel owns the closed ground profile while PurRDF owns the
storage, set membership, parsing, terms, traversal, lexical output, hashing,
standard vocabularies and byte/line primitives. Census retains domain shape and
cross-row validation over the same native storage. Its report layout is version
2; normal-form 1 fingerprint bytes are unchanged.
