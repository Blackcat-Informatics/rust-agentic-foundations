---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-entity-history
title: "The message of an unreadable line history"
description: "The error for a line history git cannot read says what failed and suggests a commit only as one cause."
changes: [cap:workplan-entity-history]
affects:
  - cap:workplan-entity-history#contract
  - cap:workplan-entity-history#constraints
  - cap:workplan-entity-history#verification
contract_effect: changes
items:
  - claim: "The error of a line history git cannot read says that git log -L could not read the lines from HEAD, and names committing the file as one way to fix it, not as the cause."
    outcome: implemented
---

## Summary

The error for a bound symbol whose line history git cannot read said the
symbol had no history and told the reader to commit the file, whatever the
cause. It now says what failed and offers the commit as one possible fix.
