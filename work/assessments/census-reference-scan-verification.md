---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-reference-scan-verification
subject: cap:census-reference-scan#verification
inputs:
  - kind: section
    name: cap:census-reference-scan#verification
    digest: c43f120c31295910134cb3985ae6587e602072eb73436dbf37426e1713512515
  - kind: symbol
    name: raf_census::ArchiveCache
    digest: e5c3e45db9221f00ace57803096fe0f3a9ecc8a8489cb60eec94c90a2319b937
  - kind: symbol
    name: raf_census::ExternalReference
    digest: dff9ba21af812a6448881bf2fda85215d82d533591e85489c9f8e454bc9ba7ce
  - kind: symbol
    name: raf_census::Gate::check_locked_registry
    digest: fb2fa6b560d7d2425855f40c2e9311b33db35217f36d59b907f047f7a16270f4
  - kind: symbol
    name: raf_census::Gate::check_registry
    digest: dff736d6d6945e7b4c84e2686bd55bebfd589bcf9e85e7d085adfbf7e0d86f74
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
    name: raf_census::Gate::write_debt_registry
    digest: a67a600e75b34406abd13d2f301f0ece26184142f81585d446ea7d9996d44671
  - kind: symbol
    name: raf_census::LockedRegistry
    digest: f50014664e55ffdfd954c1ab91db21e18c7ed61f0d875dd7b1e8d0c1327e5b8e
  - kind: symbol
    name: raf_census::LockedRegistry::require_declared_features
    digest: 5f7974115fb85b850224d4e87f81b0dfe79f0cf17b74625a84ec8aa135bda51e
  - kind: symbol
    name: raf_census::LockedRegistry::seal_reference
    digest: 8630d9bb28400275a0144c776f012de16a769e9ccbbefd679b69bbd488654563
  - kind: symbol
    name: raf_census::ReferenceConstantRow
    digest: 553be507c6f6454a9bd7b7746d27c6f9f79e9461e1c2db7fc0c4c57582e02767
  - kind: symbol
    name: raf_census::ReferenceSource
    digest: 1194fa432612b8cace74775c68a615ba3405ffb79f8126ea16e4bdab700ed9fd
  - kind: symbol
    name: raf_census::ReferenceWitness
    digest: b2c06ca3ba1b22ec1504d2ff6838f1b7052f833014f4fb8932dd2d1068bc8e10
  - kind: symbol
    name: raf_census::gate::Gate::evaluate_reference_tree
    digest: d624f8a5470e951d8dcfcec0269d114e1cfa3bd35397845a2907469e71c9ae00
  - kind: symbol
    name: raf_census::gate::reference
    digest: 48313bbcadf36e610cacfd128c68e0d065b3040b7e8878f5df9ce26ad623804f
  - kind: symbol
    name: raf_census::gate::reference::external
    digest: 483f06fca67b8b2097bceb2e803b7b4a641099ab26cf185f68878384634d610d
  - kind: symbol
    name: raf_census::report::reference
    digest: 32962a6648baff61a37ed57137c4173a3287eedd29605861bd9c061e72cad75b
  - kind: symbol
    name: raf_source::PinnedGitTree
    digest: 3a9d76397494b37297a9f0e1c973cc8a7a1421636e4bf290901c060972d80038
  - kind: symbol
    name: raf_source::PinnedGitTree::open
    digest: a6ae4174ad924bbc05e8125a1b82a020f027558ba45d8064a10bc68c57df8a4e
  - kind: symbol
    name: raf_source::ReferenceObservation
    digest: de8fa7c6e008ee0dbfe4be3e089b77fc3351f0411c6552fd89aeb60a0edacb6d
  - kind: symbol
    name: raf_source::ReferenceRequest
    digest: 866ef48e0fd38b0cf973154ac083374c203f96c6a26983e604aae4eaeb3f15c1
  - kind: symbol
    name: raf_source::Workspace::package_version
    digest: 45f56b0b8440e3433775796a49c801e44862260c98590a3a820e6251ad90cf0a
  - kind: symbol
    name: raf_source::git::Repository::main_work_dir
    digest: e013eb3abf08341a4b162ba445bca36b9c558b121650cd3b18962f6cab61ff73
  - kind: symbol
    name: raf_source::workspace::locked::reference
    digest: fd6c1c386536fcb1b27c99c106b699acb509a02439ac0d4a37c25e7085a8c8c8
verdict: accepted
by: agent
---
