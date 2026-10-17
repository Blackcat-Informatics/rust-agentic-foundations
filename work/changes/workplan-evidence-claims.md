---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-evidence-claims
title: "Evidence through real directories, unloaded capabilities and the trusted command limits"
description: "Evidence is never written through a linked directory, evidence claims refuses while a capability document does not load, and the limits of the trusted command are stated."
changes: [cap:workplan-evidence-claims]
affects:
  - cap:workplan-evidence-claims#contract
  - cap:workplan-evidence-claims#constraints
  - cap:workplan-evidence-claims#verification
contract_effect: changes
items:
  - claim: "evidence record refuses a linked work or work/evidence directory before the command or the binding runs, and writes nothing."
    outcome: implemented
  - claim: "evidence claims exits 2 naming each capability document that does not load, and an unloaded change or evidence record leaves its answer as it is."
    outcome: implemented
  - claim: "The Constraints state that an interrupt of cargo-workplan does not reach the process group of the trusted command, that a descendant that starts a session of its own escapes the kill, and that a relative program resolves against the workspace root while the record keeps the command as typed."
    outcome: implemented
  - claim: "An error message about a trusted command quotes its words as the record stores them."
    outcome: implemented
  - claim: "The Constraints state that a hand-written evidence record may hold more than 1024 values, bounded by the document size bound, and that the write checks judge paths, not open handles."
    outcome: implemented
---

## Summary

An evidence record could be written through a directory link inside the root,
so `evidence record --update` could put a record in place of a document. The
check before the run and the write now refuse a linked `work` or
`work/evidence`. `evidence claims` answered "no claim" while a capability
document did not load; it now fails hard as `evidence for` does. The limits of
the trusted command that hold today are stated where its contract is.
