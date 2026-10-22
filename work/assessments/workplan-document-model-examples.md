---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:workplan-document-model-examples
subject: cap:workplan-document-model#examples
inputs:
  - kind: section
    name: cap:workplan-document-model#examples
    digest: 5bce738c2d3470492bdd9449dc92ba5d78029d2c6c515c66c5d5d5a95e86e064
  - kind: symbol
    name: raf_workplan::ExampleKind
    digest: d45f4c9192e66ec611582276018a52aa3fe49704d6cb5177e4779f9291864673
  - kind: symbol
    name: raf_workplan::Finding
    digest: f43e8c8938a1469f1d0eba8c24854adde16b9f835df2e448ce98a6c83d581555
  - kind: symbol
    name: raf_workplan::Gap
    digest: 121369f0e3dc185003952b3ad941b6850b5f9cc25f86f6e471841569825b06e8
  - kind: symbol
    name: raf_workplan::Gate
    digest: 2e02595a511bef4755a54f5bd679973e44ccbe776c4ea52a7cb42951c50ef807
  - kind: symbol
    name: raf_workplan::Gate::assert_clean
    digest: 16ca22b64d06956c48d1f8df708840f0d406ee752c06890b5e3dc5d27a8d5517
  - kind: symbol
    name: raf_workplan::Gate::workspace
    digest: e6b28d3ee23163591d4a8b7b1dd5ea249b0b087319353ddfae91f8ff8a8ed0a5
  - kind: symbol
    name: raf_workplan::GateError
    digest: defa503729a4a84e5af64aa6c39956b2a1413d039e8675c114f7378f1535cefd
  - kind: symbol
    name: raf_workplan::Graph
    digest: 2d4b2f5a4c5d04e7f082fdd94fb7d84c7fa6b407a4c9b6178f415bfc88280a79
  - kind: symbol
    name: raf_workplan::Graph::assessments_of
    digest: f95b640d68d8ad790e6e074014e8768af5ad6b8101814950470b8fe29bc1a9fb
  - kind: symbol
    name: raf_workplan::Graph::capabilities_of
    digest: 5be52e17d692a2e5ef17a1ab191aa27b892f3299bdc9a9a1ab3128df21f20018
  - kind: symbol
    name: raf_workplan::Graph::claims_on
    digest: 71ef035f60d5440654a0e7eb68cf0e4723e8f0b0fe6a42c59e06fc05e55852b9
  - kind: symbol
    name: raf_workplan::Graph::code_of
    digest: 4b686563804a78708e77f894e35d1d7dc8adc8e998291e058a1391cf7c7de694
  - kind: symbol
    name: raf_workplan::Graph::evidence_of
    digest: 911c042fc7f22f4b373d0f195d7ca0bd0050fe5c3b00abc8dfbb37cf6cb4f0f2
  - kind: symbol
    name: raf_workplan::Graph::inputs_of
    digest: 617821aa3a5505311ad731dea46390687c1e9a8140b85de9a665404f86a6ac05
  - kind: symbol
    name: raf_workplan::Graph::linking_sections
    digest: b4ed88fe223c1a495f50a64f4658df87c1f96a249b352dfd8aa8fb22c44fe794
  - kind: symbol
    name: raf_workplan::Graph::sections_of
    digest: ee6d7f46aef1766be51c67e7b5f40e1b26d56840616a347e0f6c4945b53bd1ed
  - kind: symbol
    name: raf_workplan::Graph::tested_by
    digest: 324ffc331b31008d4e7d586e9563733a5f361b6448d0e9f468d20a4fce3be481
  - kind: symbol
    name: raf_workplan::Graph::tests_for
    digest: c061d0e3321754bcf6f13d065ba48cb96923e266109bf791935fd8920668a5b0
  - kind: symbol
    name: raf_workplan::LinkContext
    digest: d923ec7e0eb27f6a33e86d9baaaaf47685bbf470f9c605371db7cec641ab2172
  - kind: symbol
    name: raf_workplan::Loaded
    digest: 262eb0ded89a0709f5ac4461782a180eb52465ce308d539fd8f04d6324bee6d6
  - kind: symbol
    name: raf_workplan::OpenChange
    digest: 8af4dcb1b4d44404f379b8bec2daca1814b4743f680486131f3c6d67821afd4a
  - kind: symbol
    name: raf_workplan::Opened
    digest: b8bdc09a753eda2b0b9ae7772d111e6b6c588679bcab33ff5ce1132646c2d55e
  - kind: symbol
    name: raf_workplan::Roadmap
    digest: fdb27927871fea87b270dc45eff982f221444db1bcb3b5034221d64c6d51c7f0
  - kind: symbol
    name: raf_workplan::RoadmapEntry
    digest: 486e2d6c6c465bbe9e6dbe925e4524fbdba73babe01cc6396e4a5f1818fb3195
  - kind: symbol
    name: raf_workplan::TestedByEdge
    digest: c251c0aa5163f88d1fdbd5c9a07bb3fdadbfa192cf843854f73d38f31f0077da
  - kind: symbol
    name: raf_workplan::Workplan
    digest: 5fe5ac175bf14fe5ab50a474669be2fced8c90a03f07d4cfe4e7e9772dbb7ee8
  - kind: symbol
    name: raf_workplan::Workplan::assessments
    digest: e5eb77cccb7f921607f45eff8ebb535c9e91b4eece39ac2e0c08e7206c62fa28
  - kind: symbol
    name: raf_workplan::Workplan::check
    digest: 9b674e77e391affd8da9d3515a046247620c55bdbe0ccea501749a94abc324d3
  - kind: symbol
    name: raf_workplan::Workplan::coverage
    digest: e92bf167dc837d99b7cb04e79c432cb7b58cd2bb7accb4d70f3029b18eb9125f
  - kind: symbol
    name: raf_workplan::Workplan::debt
    digest: 9c1f59fc7c7affc79df294a850850037718eac1f1a3c9c3d9ee71f6941ce01e8
  - kind: symbol
    name: raf_workplan::Workplan::evidence
    digest: f452ce5b6ae62ae6280784f0f9107a757ba89d44083adaafc8e845eb58005a49
  - kind: symbol
    name: raf_workplan::Workplan::load
    digest: 1cafbd3bec50d1f1e41fafacec3f4cee4ee05c86facc2a29cf40d29e1e464832
  - kind: symbol
    name: raf_workplan::Workplan::status
    digest: 912d6cf91beb587b6463d8eb3236f6863f6e959f0485e262f74cce308a830ee6
  - kind: symbol
    name: raf_workplan::digest
    digest: 0ac231a23cb2c8888dff7ab83b28a453fba1b266035c9c5faf7e26dca7a3ef5d
  - kind: symbol
    name: raf_workplan::digest::Digest
    digest: 6c7d21b4cba7380d4dcffe0adcc0a59123376b9416b489895f3bbed78b3bba9c
  - kind: symbol
    name: raf_workplan::digest::Digest::record
    digest: 70cb737a4a587a8311a37e3bc94d4bb311afde23eff1496a7565ed594aa98a6b
  - kind: symbol
    name: raf_workplan::digest::Digest::section
    digest: e6533bc6187ff50b72506d15de58ba69145086918b1f87fd5a73923a718a8a50
  - kind: symbol
    name: raf_workplan::digest::Digest::symbol
    digest: 03836b1413fd2dff869e8f5fb030a5b3309023e6a5f8d441a577bcb56b3cb1a8
  - kind: symbol
    name: raf_workplan::git::tracked_paths
    digest: 48690232ec5e646c500e9b63eef189c87af4af7cb84cd73b2af25b80153cc3cf
  - kind: symbol
    name: raf_workplan::graph
    digest: 55379f663cf36fa54c1372514821724e71b42db99cc3d18a7224a6dc58faa3d8
  - kind: symbol
    name: raf_workplan::plain
    digest: 6b773895ebb66d1a71d178eeceedeffff921f45e4c269b24990de65f9111c076
  - kind: symbol
    name: raf_workplan::plain::check
    digest: ca174b83e9a7a962d3264d1ab6102abeddfd5da57d9a5f94fd11a035ac32fcaf
  - kind: symbol
    name: raf_workplan::record
    digest: 86fb9837b3d9789c19ddccaeae427e683596e71dd3edb2358f668f6b1dc355a5
  - kind: symbol
    name: raf_workplan::record::Record
    digest: b4675d6558d5e4ce2325f1e0a4df003079cca409527ebe77bd0450821331af24
  - kind: symbol
    name: raf_workplan::record::create
    digest: 29f87408ae645f6beb6e99beae8362495ea54aaa59341204364a7024987794cb
  - kind: symbol
    name: raf_workplan::record::replace
    digest: 48887deba5c2006961084107c2a7be5d4487228dff46e5a1bd198c7084f25f4c
  - kind: symbol
    name: raf_workplan::refs
    digest: 7fc853361683be08db1a98cb0bf2a69dfc3402e3fdccf2b191fd02d0d13100f3
  - kind: symbol
    name: raf_workplan::refs::banned_in_line
    digest: 6c6e85ce2699d14a54ec35d156b53a1f8aeade4f1b8ab50eaa457d089ae093a4
verdict: accepted
by: agent
---
