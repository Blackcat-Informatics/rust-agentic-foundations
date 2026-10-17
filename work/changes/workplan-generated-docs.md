---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-generated-docs
title: "The docs sync example under a lock"
description: "The test that runs the docs sync example takes the lock the other process tests take."
changes: [cap:workplan-generated-docs]
affects:
  - cap:workplan-generated-docs#verification
contract_effect: changes
items:
  - claim: "The test that runs the docs sync example takes the lock the other process tests of the crate take, so no other test rewrites the executable while it runs."
---

## Summary

The test that runs the `docs_sync` example could fail with "Text file busy"
when another test rebuilt the example at the same time. It now takes the same
lock as the other tests that start processes.
