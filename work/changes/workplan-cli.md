---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-cli
title: "Bind the command-line tool's tests and root discovery"
description: "Lists every test of cargo-workplan and of the root discovery, unit tests included, and the raf_workplan::root module, in the bindings of the command-line capability."
changes: [cap:workplan-cli]
affects:
  - cap:workplan-cli#verification
contract_effect: none
items:
  - claim: "The bindings.code of the command-line capability list the module raf_workplan::root, which finds the workspace root every command runs at."
  - claim: "The bindings.tests of the command-line capability list every test of cargo-workplan, its unit tests included, and every test of the root discovery in raf-workplan, its unit tests included, so check finds no unbound test of them."
---

## Summary

The coverage record enforces two-way coverage for raf-workplan and cargo-workplan, so every public item and test of those crates is bound by the capability that explains it. The command-line tool's tests and the root discovery belong to this capability, and its bindings now list them; its verification matrix grows with them.
