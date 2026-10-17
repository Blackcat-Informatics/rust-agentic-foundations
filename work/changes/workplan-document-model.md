---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:workplan-document-model
title: "Record writes through real directories only"
description: "The record writer refuses a linked directory on the way to a record, and the roadmap guard accepts a reopened capability with its own open change."
changes: [cap:workplan-document-model]
affects:
  - cap:workplan-document-model#contract
  - cap:workplan-document-model#verification
contract_effect: changes
items:
  - claim: "A record is never written through a linked directory: record::create, record::replace and the checks made before an evidence or assessment record is written refuse a symbolic link or a non-directory at work and at the directory of the record type, with nothing written."
    outcome: implemented
  - claim: "The roadmap guard of this repository accepts each of its built capabilities delivered, or implementing with the open change record named after it, with no gap in either case."
    outcome: implemented
  - claim: "The comment on has_unloaded_documents states that its comparison of file stems with ids relies on the okf-id-file-name rule."
    outcome: implemented
---

## Summary

The record writer judged only the last component of the path it replaced, so
a directory link inside the root let a replacement land on a document. Every
write of a record now judges `work` and the directory of the record type as
directories that are not links, the same judgement for a new record, a
replaced one and the checks made before a run. The roadmap guard of this
repository also accepts a built capability that is reopened under the change
record named after it.
