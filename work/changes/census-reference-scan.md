---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:census-reference-scan
title: "Implement the census reference scan"
description: "Delivers reference packages resolved through cargo metadata."
changes: [cap:census-reference-scan]
affects:
  - cap:census-reference-scan#purpose
  - cap:census-reference-scan#contract
  - cap:census-reference-scan#usage
  - cap:census-reference-scan#examples
  - cap:census-reference-scan#constraints
  - cap:census-reference-scan#verification
contract_effect: changes
items:
  - claim: "census reference names packages that are resolved with cargo metadata, so Cargo.lock pins the compared revision."
  - claim: "A job home may be a symbol in a reference package."
  - claim: "A first-party body equal to a reference body is reported as a reference copy."
  - claim: "The scan never updates Cargo.lock and fails on a package outside the dependency graph."
  - claim: "cargo metadata runs with --frozen, so an undownloaded registry reference package fails the scan with no network access."
---

## Summary

This change delivers the single reference model that replaces archive-based and prefix-based reference scans.
