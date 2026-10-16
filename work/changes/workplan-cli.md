---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-cli
title: "The history, evidence, assess and coverage commands, and the bindings of the command-line tool"
description: "Adds history, evidence record|claims|for, assess and coverage code|tests to cargo workplan, with their exit codes, JSON kinds and the reading of the command after --, and lists every test of cargo-workplan and of the root discovery in the bindings of the command-line capability."
changes: [cap:workplan-cli]
affects:
  - cap:workplan-cli#contract
  - cap:workplan-cli#usage
  - cap:workplan-cli#examples
  - cap:workplan-cli#constraints
  - cap:workplan-cli#verification
contract_effect: changes
items:
  - claim: "The bindings.code of the command-line capability list the module raf_workplan::root, which finds the workspace root every command runs at."
    outcome: implemented
  - claim: "The bindings.tests of the command-line capability list every test of cargo-workplan, its unit tests included, and every test of the root discovery in raf-workplan, its unit tests included, so check finds no unbound test of them."
    outcome: implemented
  - claim: "history <id> [--since <day>] prints Workplan::history as the table and the JSON array of raf_workplan::history::render, exits 0 with an answer and 2 without one, and refuses an id that is not a cap:, change:, arch:, evidence: or assessment: id and a day that is not one when the arguments are parsed, before any process starts."
    outcome: implemented
  - claim: "evidence record runs the command after the first -- (the TrustedArgv of the command line of the process, refused when it differs from the trailing words the parser read) or the bound test or example of --from-binding, writes exactly one record, and exits 0 for a pass record, 1 for a fail record and 2 when it writes none; an option whose value is --, a program name of -- and --from-binding beside a -- are refused before any process starts, the options after -- are words of the command, and a replacement holds only the SPDX lines its own run gives."
    outcome: implemented
  - claim: "evidence claims <evidence-id> and evidence for <cap:slug> print the statements that rest on a record and the records that support a capability, write nothing and exit 0 with an answer."
    outcome: implemented
  - claim: "assess takes exactly one of --context, which prints the same context for --by human and --by agent and writes nothing, --verdict accepted|rejected with --by human|agent, which writes exactly one record and exits 0 for either verdict, and --list, which prints each assessment, or the one of a section that must exist, with its currency, writes nothing and exits 0."
    outcome: implemented
  - claim: "coverage code|tests [--crate <package>] prints one direction of Workplan::coverage_report for the whole workspace or for one package named in either form, and exits 1 when the report names a gap, or in the code direction a surface that cannot be listed, and 0 otherwise."
    outcome: implemented
  - claim: "The JSON kinds history-entry, history-removed, evidence-recorded, evidence-claim, evidence-for, assess-context, assessment-recorded, assessment, coverage-gap, coverage-excluded and coverage-error are documented with their members in order, none equals the kind of a rule, and -q hides the outcomes and keeps a fail record and a coverage gap."
    outcome: implemented
---

## Summary

The coverage record enforces two-way coverage for raf-workplan and cargo-workplan, so every public item and test of those crates is bound by the capability that explains it. The command-line tool's tests and the root discovery belong to this capability, and its bindings now list them; its verification matrix grows with them.

The command line also gains the commands of the record documents: `history` reads the commits of a document and its bound code, `evidence record` runs a command or a bound test and keeps its verdict, `evidence claims` and `evidence for` show what rests on a result, `assess` gives an assessor the context of a section and records the verdict, and `coverage` prints the public items and tests no capability binds. Each is a thin call into `raf-workplan` with its own exit codes and JSON kinds.
