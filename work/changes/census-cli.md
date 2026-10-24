---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:census-cli
title: "Implement the cargo census command line"
description: "Delivers the cargo census subcommands and exit codes."
changes: [cap:census-cli]
affects:
  - cap:census-cli#purpose
  - cap:census-cli#contract
  - cap:census-cli#usage
  - cap:census-cli#examples
  - cap:census-cli#constraints
  - cap:census-cli#verification
contract_effect: changes
items:
  - claim: "The subcommands check, baseline, index, job, count, fingerprint, debt --write, migrate and dump-ledger exist."
  - claim: "Exit code 0 means clean, 1 means findings, and 2 means a usage error."
  - claim: "The command uses clap derive; native Gate and report APIs own Census policy."
  - claim: "For a ledger that needs no registered code, check reports the same findings as Gate::assert_clean."
  - claim: "check invokes the repository target for registered policy; direct debt --write exits 2 for policy requiring registrations and preserves the record."
  - claim: "Gate::write_debt is the only write path for a ledger that needs registered code."
---

## Summary

This change delivers the command-line front end of the census as a thin layer over the library.
