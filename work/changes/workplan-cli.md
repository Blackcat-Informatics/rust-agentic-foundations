---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-cli
title: "Implement the cargo workplan binary"
description: "Delivers the cargo-workplan subcommands, options and exit codes."
changes: [cap:workplan-cli]
affects:
  - cap:workplan-cli#contract
  - cap:workplan-cli#usage
  - cap:workplan-cli#examples
  - cap:workplan-cli#verification
  - cap:workplan-cli#constraints
contract_effect: changes
items:
  - claim: "check, status, docs sync, docs preview, change new, impact, reconcile, complete and verify are subcommands."
    outcome: implemented
  - claim: "Exit code 0 means success, 1 findings and 2 a tool failure."
    outcome: implemented
  - claim: "The binary holds no logic beyond calling raf-workplan."
    outcome: implemented
  - claim: "Read-only commands write nothing."
    outcome: implemented
  - claim: "Root discovery runs cargo metadata --format-version 1 --no-deps --frozen without a shell, and every spawned process takes an argv vector."
    outcome: implemented
---

## Summary

This change delivers the command line over raf-workplan with clap derive and the 0, 1 and 2 exit codes.
