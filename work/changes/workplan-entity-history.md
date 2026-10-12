---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-entity-history
title: "Implement entity history"
description: "Delivers the history command for capabilities, changes and architecture pages."
changes: [cap:workplan-entity-history]
affects:
  - cap:workplan-entity-history#contract
  - cap:workplan-entity-history#usage
  - cap:workplan-entity-history#examples
  - cap:workplan-entity-history#constraints
  - cap:workplan-entity-history#verification
contract_effect: changes
items:
  - claim: "history prints the commits that shaped a document, following renames."
  - claim: "history reports the commits that changed each bound symbol through line-range logs."
  - claim: "A binding that no longer resolves is listed with the last commit in which it resolved."
  - claim: "A repository whose history is incomplete produces an error and no partial list."
  - claim: "The table and JSON outputs carry identical entries."
  - claim: "git is spawned without a shell, with a path after -- and a validated since date, so a path or argument that begins with a dash is never read as an option."
  - claim: "git log output is read to a bound, records are split with a bounded split, and the since date sits before the final double dash."
---

## Summary

This change delivers the history command: document history through git log --follow, symbol history through git log -L, and the table and JSON outputs.
