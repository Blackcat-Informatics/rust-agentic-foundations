---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-rule-api-constraints
subject: cap:census-rule-api#constraints
inputs:
  - kind: section
    name: cap:census-rule-api#constraints
    digest: e19abe9b5eaf54337be67a49181cf948590c60540eadb4e431997b497cc2f695
  - kind: symbol
    name: raf_census::Gate::assert_clean
    digest: 311c08fb6e81e3c5fab51fa00e901b82571f509110ebf5694df21829a0549107
  - kind: symbol
    name: raf_census::Gate::extensions
    digest: f4325ff16655a45eb8216b2658b9c48879346d2934b729c2f1d4922854c1acf1
  - kind: symbol
    name: raf_census::Gate::require_module_paths_in_package
    digest: c2fc99a581d8c9d066311c7d6915371401ef6a37aa6df5363f59f2c593a4d679
  - kind: symbol
    name: raf_census::Gate::require_rules
    digest: 7d71c7f0aa2e18b4d87e55e6658f72530d98014815dd7756c3e5016c3d5ca267
  - kind: symbol
    name: raf_census::Gate::rules
    digest: d32147afa2dc624555778ac873967a55756ee85a2100657e2751d3981c7cf051
  - kind: symbol
    name: raf_census::Gate::self_test
    digest: f2c6b0abfe9c026c6f57fcd57bc2e4c943f71a5cb8b494d5979cff3a06f4dd86
  - kind: symbol
    name: raf_census::Gate::with_production_lint_run
    digest: 0d8a51a9b223dce2b179a65de00ed2947d9cdf48a199674c2f7e402b9b53355d
  - kind: symbol
    name: raf_census::Gate::without_production_lints
    digest: 620a9a9df0171d6398c862e126dc7e1ae0a0ce1741530b10f07095fd7c60aa86
  - kind: symbol
    name: raf_census::Gate::workspace
    digest: 0503ad24db1afab83d45d098775da0f6d1304ee95c9a905293ccd91d1b73bbd5
  - kind: symbol
    name: raf_census::PRODUCTION_CLIPPY_ARGS
    digest: cada9bb2eaa7eb61ee8ab541538aea42fac4c9d9e33b737d62611753f9179b1b
  - kind: symbol
    name: raf_census::ProductionInputs
    digest: 8ecd97ce4eb3d52bd3fad486fae568941c157347c614c84df7e59b312b6e2a62
  - kind: symbol
    name: raf_census::ProductionInputs::digest
    digest: f2b1e0236c5f75b53c54a1f5b96f24c84643ca8e0fa8600395121079ef9bd6fe
  - kind: symbol
    name: raf_census::ProductionInputs::read
    digest: 601163d6205aaf0ebc7a51acb49874fe4c92718110fbba0dd1828e3b8f9b0ffa
  - kind: symbol
    name: raf_census::ProductionLintRun
    digest: 6761d0070ed689c66286014b4d2869b7685e66ffd85f68276e8b1657908067c2
  - kind: symbol
    name: raf_census::ProductionLintRun::capture
    digest: 316d728d03febd818b98f51da58d46e37a5515d246b270226f25b0532faf5e93
  - kind: symbol
    name: raf_census::ProductionLintRun::from_turtle
    digest: 3254c09f44de54e40f201b3112c5a9e1f722d64290a63c410cda6e45f7f9dcec
  - kind: symbol
    name: raf_census::ProductionLintRun::read_external
    digest: 4d72a1017a537ff1d52abf5ffb497e21cc02117ff1a8377c41b406d39d61cb38
  - kind: symbol
    name: raf_census::ProductionLintRun::to_turtle
    digest: af6c9f264b8fe8d5ba6cf1abfdc4f5d0493cfd7ded0d06213554581f933441d2
  - kind: symbol
    name: raf_census::gate::module_paths
    digest: 555ba10ab2f0f84ce48c296b5d50840bb632b0462be969692e8eef99b25b8fc7
  - kind: symbol
    name: raf_census::gate::rules
    digest: 467fe8bd9c1aba049d11efc0b1569a07e296142e42cacd06c0d5e8acda455f9e
  - kind: symbol
    name: raf_census::rule
    digest: cd623ae43036600b4791b82d9442d7f6305472d959d8329aef8eeeeb80d67c3c
  - kind: symbol
    name: raf_source::CargoLintManifest
    digest: be9eb40e540b21eb67f9f5441cf28d834c7b29cabd754e536b7cab4c18675ecc
  - kind: symbol
    name: raf_source::CargoLintManifest::inherits
    digest: 0697a74f0bbaeb8e4cd753f0f4c5eac541606cd0b4f79f3bd6119d119673efae
  - kind: symbol
    name: raf_source::CargoLintManifest::parse
    digest: '0b3c096375f7f54fd6ce5fab86f204289bbd14ec5eaa1d051ea1577b7676b6a2'
  - kind: symbol
    name: raf_source::CargoLintManifest::parse_config
    digest: 155838c214ecd8fe09ddca09839cb160fd6babb3ee7485b017206f0fefadd8f4
  - kind: symbol
    name: raf_source::CargoLintManifest::production_config_override
    digest: dbcb6a9d1b815d0085f440ea7b3e2182db95e6a7a63d227f2f0c0dd023710726
  - kind: symbol
    name: raf_source::CargoLintManifest::setting
    digest: 96759e3fff87df823709d5553186918929e6d0cbee1ab1058e16e1213c27c6d9
  - kind: symbol
    name: raf_source::Workspace::load_shipping_members
    digest: b24629146be42d59694fa5aff780063c6bbe743471213963d7374c0bc0b08ca9
verdict: accepted
by: agent
---
