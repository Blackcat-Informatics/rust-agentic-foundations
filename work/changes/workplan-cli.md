---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-cli
title: "Unloaded capabilities and escaped text in the claim and context output"
description: "evidence claims fails hard while a capability does not load, and the text output of evidence claims and assess --context escapes control and bidi characters."
changes: [cap:workplan-cli]
affects:
  - cap:workplan-cli#contract
  - cap:workplan-cli#constraints
  - cap:workplan-cli#verification
contract_effect: changes
items:
  - claim: "evidence claims exits 2 naming each capability document that does not load, and exits 0 while only a change or evidence record fails to load."
  - claim: "The text output of evidence claims and assess --context shows each control and bidirectional formatting character of a document escaped, as the history and coverage tables do."
---

## Summary

`evidence claims` answered "no claim" with exit 0 while a capability document
did not load, unlike `evidence for` and `history`. It now exits 2 naming the
document. The text output of `evidence claims` and `assess --context` printed
document text raw, so a control sequence in a document reached the terminal;
it is now escaped as in the history and coverage tables.
