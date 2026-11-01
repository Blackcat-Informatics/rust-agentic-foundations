---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:workplan-reconcile-explanation
subject: cap:workplan-reconcile#explanation
inputs:
  - kind: section
    name: cap:workplan-reconcile#explanation
    digest: 04afaa90dc079b77bb5ca4ca26560f40ec886df0761db8050740b3eac3b088bc
  - kind: symbol
    name: raf_workplan::Workplan::impact
    digest: 10268188ad8f1b4edd6b7279bd236e78f8dafb7e853e0bceb339363b2135c59a
  - kind: symbol
    name: raf_workplan::Workplan::new_change
    digest: e98206c0a4765ede28c155cb8575aa631609841c14e02af17e889740bbccc4d5
  - kind: symbol
    name: raf_workplan::Workplan::reconcile
    digest: 0c3f46f0e8b0d0f38ee66eccb226f909ed9bf78bde660f9c52b1555edb3722ad
  - kind: symbol
    name: raf_workplan::git
    digest: d75fb2d1f2c72a897b5a82eadc3be06bcf7591993062024ed91fa1946bbb3c6a
  - kind: symbol
    name: raf_workplan::git::MergeBase
    digest: b464e2a5bc868fc8d438506a08d75a783c881f733c5dc4d7ff0b37a37c822489
  - kind: symbol
    name: raf_workplan::git::ResolvedCommit
    digest: 7619e6480dc1d726f34a651c15581123946acbde019d51ed1c2e98dbc92e3d96
  - kind: symbol
    name: raf_workplan::git::blob_at
    digest: b8323e86845081824cd7ef8f61af5ec41d4057c24c48c1ca7e8e2eadacdaa41d
  - kind: symbol
    name: raf_workplan::git::default_merge_base
    digest: c46081c6cb447bb9513eb34831cf28654382026218aba81fe36543b915b4df2b
  - kind: symbol
    name: raf_workplan::git::require_version
    digest: 507c48b88168458d9c1a173099f53a446597492189f96dd137d83c30b55e48de
  - kind: symbol
    name: raf_workplan::git::resolve_commit
    digest: fe54098521262c24cef768174dfe37ba17238322be8e739acf64e4b73ec9e4ba
  - kind: symbol
    name: raf_workplan::git::resolve_commit_id
    digest: cb97dc67d0fad54b814f87d29d6a3a23246891f967f1494aa997c22b14e1dc53
  - kind: symbol
    name: raf_workplan::git::show_prefix
    digest: c76430b86eec0d5f69a21cab58487a83f02da7c7f14416f4e30ce4afac0b7a95
  - kind: symbol
    name: raf_workplan::lifecycle
    digest: cc7e641b03d4eae28456fcc917c274560ab7fe6777edc92cb7c5ab600d5cced8
  - kind: symbol
    name: raf_workplan::lifecycle::Impact
    digest: a6c21aab0a4b43db01baf0f947be6843cdd501fab1aa1cfed83f8b74ba6a7935
  - kind: symbol
    name: raf_workplan::lifecycle::Lifecycle::open
    digest: 59ebcd148d2408ad3f0833018540b2453c6120c10594cb09b6ebf5676a5acf56
  - kind: symbol
    name: raf_workplan::lifecycle::Lifecycle::reconcile
    digest: 8b63f107a3c0212ff28e09abd4c0e9538ea3b331281a47a59ca8a498e671478f
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
    name: raf_workplan::lifecycle::section_digest
    digest: e06f6932f7a3f48e86afee6bf34ccdd62165d8cf47a430820690bc09696e0bdc
  - kind: symbol
    name: raf_workplan::obligation
    digest: 7114128cd55e565a61877c0e773152490d082898e947581dfbb7364d178133e6
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
    digest: b046f614798185197ee13d374406d253cae84e079e63b8bf1c50a269acf951b3
  - kind: symbol
    name: raf_workplan::obligation::Obligation
    digest: 5b76a0557a9818415e4e2be152ecb36fb6b83bf2b869c8b3d5db97c87a94f667
  - kind: symbol
    name: raf_workplan::obligation::ObligationKind
    digest: 60192c1f9b311df6a307f0d9cd7adc7c642c9f93e3ad206f6b1c1ef3ddff1312
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
