---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-entity-history
title: "Module history and an exact since"
description: "The history of a module binding covers the files its digest reads, and --since selects by author date."
changes: [cap:workplan-entity-history]
affects:
  - cap:workplan-entity-history#contract
  - cap:workplan-entity-history#usage
  - cap:workplan-entity-history#constraints
  - cap:workplan-entity-history#verification
contract_effect: changes
items:
  - claim: "The history of a code binding that resolves to a module below its crate root lists the commits that changed any file the digest of the binding reads: the module's own file and the file of every module below it, each through the document log of that file, besides the line log of the module's own span; a crate root and every other item keep the line log of their span, and a file of such a module that has no commit is an error that names the binding and the file."
    outcome: implemented
  - claim: "--since selects by author date: every entry shown has an author date at or after 00:00:00 UTC of the day, because no date reaches git and the logs are filtered in process by the date each entry shows."
    outcome: implemented
---

## Summary

The history of a module binding listed only the commits of its declaration
line, while its digest, and so the evidence and assessments that rest on it,
reads every file of the module. This change makes the history read the same
files. It also makes `--since` select by the author date each entry shows and
orders by, where Git's own `--since` selects by the committer date: the date
is no longer given to `git`, and every log is filtered in process.
