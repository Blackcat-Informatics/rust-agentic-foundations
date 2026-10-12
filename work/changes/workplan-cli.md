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
contract_effect: changes
items:
  - claim: "check, status, docs sync, docs preview, change new, impact, reconcile, complete and verify are subcommands."
  - claim: "Exit code 0 means success, 1 findings and 2 a tool failure."
  - claim: "The binary holds no logic beyond calling raf-workplan."
  - claim: "Read-only commands write nothing."
---

## Summary

This change delivers the command line over raf-workplan with clap derive and the 0, 1 and 2 exit codes.
