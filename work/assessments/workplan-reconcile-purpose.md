---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:workplan-reconcile-purpose
subject: cap:workplan-reconcile#purpose
inputs:
  - kind: section
    name: cap:workplan-reconcile#purpose
    digest: 8a3306eadf837d1a643c55bf74268e08920fd3cc63918a5ea768b502356c0c4c
  - kind: symbol
    name: raf_workplan::Workplan::impact
    digest: 10268188ad8f1b4edd6b7279bd236e78f8dafb7e853e0bceb339363b2135c59a
  - kind: symbol
    name: raf_workplan::Workplan::new_change
    digest: e98206c0a4765ede28c155cb8575aa631609841c14e02af17e889740bbccc4d5
  - kind: symbol
    name: raf_workplan::Workplan::reconcile
    digest: dc8a8600263b5f9a10aa7cffc3ecbec81a2916b0a29dd22a1b1d161380c1b20f
  - kind: symbol
    name: raf_workplan::git
    digest: e5307f846a142ed700a2454e9310d5413faf0f6847fcc8bb6b53ca9654884cec
  - kind: symbol
    name: raf_workplan::git::MergeBase
    digest: 951ddc94dd150031a671ed2e86ff2c9d0d2b40bf1587af888f53bd09f2ed1dc2
  - kind: symbol
    name: raf_workplan::git::ResolvedCommit
    digest: cc7b374a16746432cb1651e98492f5b6cf4e4bf6ff71ae3c816c7ad7c5231cf7
  - kind: symbol
    name: raf_workplan::git::blob_at
    digest: 8ded1201ebcfb61c97a8d78c8e04fe4e633f23f6e6b467f18d2382c25309f79a
  - kind: symbol
    name: raf_workplan::git::default_merge_base
    digest: 9df74bbca3cefb9301e86cea21d90b79634d54dcd43b33fc0179bef940b86779
  - kind: symbol
    name: raf_workplan::git::resolve_commit
    digest: b169274e3f3a7364de0997b9a3c8b92f440c65030c9ec41c29630841cb9b8462
  - kind: symbol
    name: raf_workplan::git::resolve_commit_id
    digest: 8652aea6e8680fbeea468154619056eb77e992f12bd495399c1c258c8f59d683
  - kind: symbol
    name: raf_workplan::git::show_prefix
    digest: 8faccb5690df217dc2a093f3142b723662621569d56e1274a4c47b3a84d15f52
  - kind: symbol
    name: raf_workplan::lifecycle
    digest: da2bc27fba051157e252084445757f93d595ecb4288e747ae625eb449471b289
  - kind: symbol
    name: raf_workplan::lifecycle::Impact
    digest: a6c21aab0a4b43db01baf0f947be6843cdd501fab1aa1cfed83f8b74ba6a7935
  - kind: symbol
    name: raf_workplan::lifecycle::Lifecycle::open
    digest: 4de1504416926cae0fbca051ba52b61a47422ab8ac8b6f5fcbdc5b67f48d533f
  - kind: symbol
    name: raf_workplan::lifecycle::Lifecycle::reconcile
    digest: 6fa2fd1a30b0e36763f3c3ef091f2b758f1b09b169408df5a1d263f5f1ca9c48
  - kind: symbol
    name: raf_workplan::lifecycle::NewChange
    digest: fbfb102c8819761489ea937ebcd93a201609f7d4378473d0cf2b6ba681a3dcb3
  - kind: symbol
    name: raf_workplan::lifecycle::NewChangeError
    digest: ed0b2ace85b79e37789d5a98ae0a82c91e5067691b47d771a84ac302c6626836
  - kind: symbol
    name: raf_workplan::lifecycle::NewChangeRequest
    digest: 5979427bd8fa150f21c4a959020eb90680dd610e4a4eba487a5d0b118701d2f6
  - kind: symbol
    name: raf_workplan::lifecycle::changed
    digest: 9b63b15bdb3c2e8e1519d4adee37f2e216eff7b571f77e561fce667ef2e6455d
  - kind: symbol
    name: raf_workplan::lifecycle::section_digest
    digest: e06f6932f7a3f48e86afee6bf34ccdd62165d8cf47a430820690bc09696e0bdc
  - kind: symbol
    name: raf_workplan::lifecycle::touched
    digest: 49f2842d828cda3ab00b2e9c3e80bb5ef81b45a0e399405d478af1f43cb351ec
  - kind: symbol
    name: raf_workplan::lifecycle::untouched
    digest: c4bcfb2c0638b52dcd66423977bf61207f5fae27cb5ccc2a1acddc416858af40
  - kind: symbol
    name: raf_workplan::obligation
    digest: 926112966790777aee27d49d7beea889c0c503ae2183769af4908489b0c715d2
  - kind: symbol
    name: raf_workplan::obligation::ClaimProblem
    digest: 326c239924ff76836c168c30cfe1286e5f1095150e492064d5c833d6e6afab29
  - kind: symbol
    name: raf_workplan::obligation::EvidenceChange
    digest: 13280e551b68a94d71a521c3498270aca17e8391edf2e833868b36535e8bd219
  - kind: symbol
    name: raf_workplan::obligation::InputMoved
    digest: 9f63c315dc2333c3179b454424aaa4b908b7a4d759584bd080479c6928716cc4
  - kind: symbol
    name: raf_workplan::obligation::Needed
    digest: 6c9b358c94089cc503112de51941b72ebbd3be5001dcbcf7af37c86e9f2fe847
  - kind: symbol
    name: raf_workplan::obligation::Obligation
    digest: 5b76a0557a9818415e4e2be152ecb36fb6b83bf2b869c8b3d5db97c87a94f667
  - kind: symbol
    name: raf_workplan::obligation::ObligationKind
    digest: 04a1b51d03515d123ff8a3f3dc5f67e9091dd0ab4cdfa2497da216934bd5e467
  - kind: symbol
    name: raf_workplan::obligation::OverLimit
    digest: 4d2c0b145a1dbc098c4edc690b1e6ddf527865bd0e72dff458d582de8f9e9a2c
  - kind: symbol
    name: raf_workplan::obligation::Reconciled
    digest: cf7b88d6c00a315b07a209d7824df8f385ae37c9e379c1c75739eb8ebaccc929
  - kind: symbol
    name: raf_workplan::obligation::suppressed
    digest: 0070cf3d3af9d8dfd83d93fa46b7817ffbdb89588a6635b6dececab6bafcde05
  - kind: symbol
    name: raf_workplan::obligation::tense::PHRASES
    digest: 643aacfe31ef5003891c04cce2603f1ccfe51ed1d9e5db848c657a2c54170b33
verdict: accepted
by: agent
---
