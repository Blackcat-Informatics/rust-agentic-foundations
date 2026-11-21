---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-rule-api-verification
subject: cap:census-rule-api#verification
inputs:
  - kind: section
    name: cap:census-rule-api#verification
    digest: 6671e242e82ce331ceb96aab1c201f365ead3b1e94fe3e6179e3095ae94599f1
  - kind: symbol
    name: raf_census::Gate::assert_clean
    digest: 45cf504b0f3a9b33934d1581934a738fd056a845e17d60f3751a2b4d7c5e8a25
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
    digest: 3e66378dece710f9cd38e9254f2fefe53f8a30108a45b8050d4ab355c5e05f16
  - kind: symbol
    name: raf_census::ProductionInputs
    digest: 8ecd97ce4eb3d52bd3fad486fae568941c157347c614c84df7e59b312b6e2a62
  - kind: symbol
    name: raf_census::ProductionInputs::digest
    digest: f2b1e0236c5f75b53c54a1f5b96f24c84643ca8e0fa8600395121079ef9bd6fe
  - kind: symbol
    name: raf_census::ProductionInputs::read
    digest: 24492a2d7df5b66b0b4ae7a1a536b4052bd1a2d0409cc0a71bd15a433e562185
  - kind: symbol
    name: raf_census::ProductionLintRun
    digest: 6761d0070ed689c66286014b4d2869b7685e66ffd85f68276e8b1657908067c2
  - kind: symbol
    name: raf_census::ProductionLintRun::capture
    digest: 316d728d03febd818b98f51da58d46e37a5515d246b270226f25b0532faf5e93
  - kind: symbol
    name: raf_census::ProductionLintRun::from_turtle
    digest: dbfd611f0689f5dba6f8d499b5e5b4e9d45ce7b04119d85e03bdb8af20e177a8
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
    digest: 25a1ce6ed6de9dcc776c326ae9d343656a72d8a00a659fdcf10aed319308e9f7
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
    name: raf_source::CargoLintManifest::production_config_override
    digest: 337c10d0a4f89fb44f40b414a2b0df9e18afdba936dabf0618f48a6484a1243d
  - kind: symbol
    name: raf_source::CargoLintManifest::setting
    digest: 96759e3fff87df823709d5553186918929e6d0cbee1ab1058e16e1213c27c6d9
verdict: accepted
by: agent
---
