---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:census-detectors
title: "Implement the built-in census detectors"
description: "Delivers the fourteen generic detectors with positive and negative fixtures."
changes: [cap:census-detectors]
affects:
  - cap:census-detectors#purpose
  - cap:census-detectors#contract
  - cap:census-detectors#usage
  - cap:census-detectors#examples
  - cap:census-detectors#constraints
  - cap:census-detectors#verification
contract_effect: changes
items:
  - claim: "The census registers the 14 built-in detector ids, and each reports hits as rule:<id>."
  - claim: "Each detector has a positive fixture it flags and a negative fixture it leaves alone."
  - claim: "A test fails when a registered detector lacks either fixture."
  - claim: "Detector hits are not reported inside the home package of the job that forbids them."
  - claim: "hex-digit-radix, json-pointer-escape, length-prefix-frame, le-slice-int, align-up-mask, div-ceil-by-hand, xor-first-mismatch, home-literal and vocabulary-literal read shipping code only; std-default-hasher, hex-format-loop, hex-pair-radix, hex-table and sole-name also read test, bench and example targets."
---

## Summary

This change delivers the closed set of fourteen built-in detectors together with the fixtures that prove what each does and does not flag.
