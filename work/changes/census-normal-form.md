---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:census-normal-form
title: "Implement the census normal form and fingerprints"
description: "Delivers normal form 1, BLAKE3 fingerprints, grouping and the baseline and index reports."
changes: [cap:census-normal-form]
affects:
  - cap:census-normal-form#purpose
  - cap:census-normal-form#contract
  - cap:census-normal-form#usage
  - cap:census-normal-form#examples
  - cap:census-normal-form#constraints
  - cap:census-normal-form#verification
contract_effect: changes
items:
  - claim: "The census lexes bodies with proc_macro2, macro arguments included, and produces the structural and value forms of normal form 1."
  - claim: "Bodies under 30 tokens keep literals, and a body that is one expression with a call, no self and 6 to 29 tokens also gets a shim form that excludes trait-impl methods and Self:: forwarders."
  - claim: "A fingerprint is the kind prefix plus the first 8 bytes of the BLAKE3 hash in hex, for the kinds body, shim, table and rule."
  - claim: "Units are collected once per source item, whatever number of targets reach it, and grouping builds a sorted BTreeMap of groups with two or more members and compares integer constants by value."
  - claim: "Golden tests pin every fixture fingerprint."
  - claim: "The census implements normal_form 1 and check_normal_form refuses any other version, naming both versions and cargo census migrate."
  - claim: "Matches are classified as HOME, VARIANT or COPY and counted per job."
  - claim: "Baseline and index JSON reports are deterministic."
---

## Summary

This change delivers the census normal form as defined in the normal-form architecture page: the normaliser, the fingerprint function, grouping, the two JSON reports and the golden tests that pin the version.
