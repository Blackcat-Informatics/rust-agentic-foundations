---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-generated-docs
title: "Writes below real directories and the docs sync example under a lock"
description: "Workplan::write refuses a target below a linked directory, and the test that runs the docs sync example takes the lock the other process tests take."
changes: [cap:workplan-generated-docs]
affects:
  - cap:workplan-generated-docs#constraints
  - cap:workplan-generated-docs#verification
contract_effect: changes
items:
  - claim: "Workplan::write refuses a target below a directory on its way that is a symbolic link, inside the root or out of it, before anything is written."
    outcome: implemented
  - claim: "The four tests that run the docs_sync example hold one lock of their test binary from their first process to their last, as the other process tests of the crate hold the lock of theirs, so no test of that binary rewrites the example while another starts it."
    outcome: implemented
---

## Summary

`Workplan::write` judged only each target itself, so a directory link inside the
root on the way to it let a write land in the directory the link names. Each
directory on the way is now judged as every write of the crate judges it.
The test that runs the `docs_sync` example could fail with "Text file busy"
when another test rebuilt the example at the same time. It now takes the same
lock as the other tests that start processes.
