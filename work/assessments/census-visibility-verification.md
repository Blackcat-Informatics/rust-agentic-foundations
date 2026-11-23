---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-visibility-verification
subject: cap:census-visibility#verification
inputs:
  - kind: section
    name: cap:census-visibility#verification
    digest: 240b8793592829375d2393129f5dc218f8c44d8a4a705fb95883e5c9b295fd3d
  - kind: symbol
    name: cargo_raf_census::command::visibility
    digest: 194e2393fe8176980627cbef871f4cd82154ffbe10e1d9ac5063a9e888389742
  - kind: symbol
    name: raf_census::VisibilityError
    digest: 5d8132dde3b1d357b87122175833d2d664e24332c8fec6567d49732a0b8ca6d9
  - kind: symbol
    name: raf_census::visibility
    digest: fdba8e824cce7e735c67843d27f8ecb2aee2c9d59ae22f20c74fe109d8092b44
  - kind: symbol
    name: raf_census::visibility_demand
    digest: d1c4097b20a9460eb3a865e27d46b327a5af477af4be4ef1cc5550067ba2186e
  - kind: symbol
    name: raf_source::CodeResolver::indexed_imports
    digest: a6eb6bf9a5edfdc00f82208fb0cefb3840d2c824b139b4a1805e68d85a325c5c
  - kind: symbol
    name: raf_source::CodeResolver::resolve_test_from_item_at
    digest: 0265ad5890b7e20f03b95a3877825919ab0bbdcf65e9a8d1ca6bc23b29cf43d6
  - kind: symbol
    name: raf_source::IndexedImport::is_conditional
    digest: f244dc24d73319a1b199cc6fef23c3fd99258cfccdca94eb99a6bac981766761
  - kind: symbol
    name: raf_source::IndexedImport::is_test
    digest: 0299517d48e2ad705bd547e7364ecb5517d129da956fdc67b766ad804500d28c
  - kind: symbol
    name: raf_source::Workspace::library_prelude_dependencies
    digest: 98d682acec3215279757af68226903dc7c596ad761278775312ca660dd72f47f
  - kind: symbol
    name: raf_source::Workspace::load_members
    digest: bdb0156a1e0187ce80f223afbd1a780d48d6b4a78cc583c939ed5899845132b6
  - kind: symbol
    name: raf_source::Workspace::target_prelude_dependencies
    digest: 0aaf0554a74bd7e8cd963b4319f6c27502db3a386cd8873efb3e7eeb58200b06
  - kind: symbol
    name: raf_source::cfg_requires_test
    digest: 7dc09ae92de5f3e670d4cf9bb0da252399fef6456499fb93b7a99843d7901c80
  - kind: symbol
    name: raf_source::visit_grouped_import
    digest: 53918660e10aa7de9d8bdc878718f3d449887c9f5fed3df485af9315346c255d
verdict: accepted
by: agent
---
