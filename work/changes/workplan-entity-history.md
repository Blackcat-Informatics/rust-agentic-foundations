---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-entity-history
title: "Implement entity history"
description: "Delivers the history command for capabilities, changes and architecture pages."
changes: [cap:workplan-entity-history]
affects:
  - cap:workplan-entity-history#purpose
  - cap:workplan-entity-history#contract
  - cap:workplan-entity-history#usage
  - cap:workplan-entity-history#examples
  - cap:workplan-entity-history#constraints
  - cap:workplan-entity-history#verification
contract_effect: changes
items:
  - claim: "history prints the commits that shaped a document, following renames."
    outcome: implemented
  - claim: "history reports the commits that changed each bound symbol through line-range logs."
    outcome: implemented
  - claim: "A binding that no longer resolves is listed with the commit that removed it, the newest commit that changed how often its last path segment occurs in its crate's directory, and that commit's parent as the last commit in which it resolved."
    outcome: implemented
  - claim: "A shallow clone, whose history is incomplete, produces an error that names the missing history and no partial list."
    outcome: implemented
  - claim: "The table and JSON outputs carry identical entries."
    outcome: implemented
  - claim: "git is spawned without a shell, with a path after -- or inside the single -L argument and a validated since date, so a path or argument that begins with a dash is never read as an option."
    outcome: implemented
  - claim: "git log output is read to a bound, records are split with a bounded split, and the since date sits before the final double dash."
    outcome: implemented
---

## Summary

This change delivers the history command: document history through git log --follow, symbol history through git log -L, and the table and JSON outputs.
