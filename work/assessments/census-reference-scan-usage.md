---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-reference-scan-usage
subject: cap:census-reference-scan#usage
inputs:
  - kind: section
    name: cap:census-reference-scan#usage
    digest: 0f388161b5b206bfc13897475e1549a82bcf5d97b9e6b52759ef92ca8c895a74
  - kind: symbol
    name: raf_census::Gate::check_registry
    digest: 1c1a37f53db239e0be6f3436ace2ed25b59e6027988f470371c11e4b626bd8a6
  - kind: symbol
    name: raf_census::Gate::check_tree_with_references
    digest: c5087ae773dfc4b1246c1a9498aa45fbe698686a15afef9ecc6786beb2ff8ec8
  - kind: symbol
    name: raf_census::Gate::external_references
    digest: 203e097a440d87e226a19737629540596262176ef6dba84291ce2bcd060f3adb
  - kind: symbol
    name: raf_census::Gate::reference_roots
    digest: fb7552389a703dcbb4da65f26224530d5b67448fa60350c87a0f9f48e66a93b5
  - kind: symbol
    name: raf_census::LockedRegistry
    digest: f509748b4d8a963817c7c30c1567acb43145aba79046fa435ff2b7cff85223ef
  - kind: symbol
    name: raf_census::gate::reference
    digest: 48313bbcadf36e610cacfd128c68e0d065b3040b7e8878f5df9ce26ad623804f
  - kind: symbol
    name: raf_census::report::reference
    digest: 32962a6648baff61a37ed57137c4173a3287eedd29605861bd9c061e72cad75b
  - kind: symbol
    name: raf_source::workspace::locked::reference
    digest: fd6c1c386536fcb1b27c99c106b699acb509a02439ac0d4a37c25e7085a8c8c8
verdict: accepted
by: agent
---
