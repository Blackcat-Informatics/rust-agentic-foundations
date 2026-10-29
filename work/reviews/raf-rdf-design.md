---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Review
id: review:raf-rdf-design
timestamp: 2026-10-28T22:33:34Z
scope: [crate:raf-rdf]
revision: 50422681f2e8de3c6ce258f169e0d3660b5c78c1
kind: design
inputs:
  - kind: symbol
    name: raf_rdf
    digest: 5d738332b96112aa0af5f918e2a2e5b60db537aeae5b7ffe3a681e62006bb8d7
  - kind: symbol
    name: raf_rdf::Format
    digest: 9f9a511ccccee060002e3dca00ac1fd6b6b14ae75942f3908ed3944b0852b11a
  - kind: symbol
    name: raf_rdf::error
    digest: 3a5641c70dbd3bf71aae9bfdaa49ed8acbecd71b9c46aeb72acf6a5e8bed6b40
  - kind: symbol
    name: raf_rdf::error::Error
    digest: 2742ad342df096f5dc4df8a06930b7efa0f10c2b51104dbd7f23ee0a306185ca
  - kind: symbol
    name: raf_rdf::error::Error::fmt
    digest: 686f7d82cc1cac60b2f6f4375bcc7209915d7dc3d99c0f1d75b7cc71c226844c
  - kind: symbol
    name: raf_rdf::example_crate::report
    digest: 0ec9906273ddc232e5c6286ae35ae39b3411c9d7e78cb230777c15a0e6d8b6d6
  - kind: symbol
    name: raf_rdf::example_crate::report::main
    digest: 5399f09970daee7724e4f72869cb7a85957712c6c5e63a28f1701b6f8669f769
  - kind: symbol
    name: raf_rdf::graph
    digest: a51dae69f3cc3c699955a266948b4d6a8da04c1732ee00146fc1d01a36ea42ad
  - kind: symbol
    name: raf_rdf::graph::Builder
    digest: d48fe9c80c2bac988f9a264cc68dec15fbec5f04fcd324dc1dc2335cbdf64dd1
  - kind: symbol
    name: raf_rdf::graph::Builder::contains
    digest: 0c641559a905bcc404c96ab86bb362d74cd8d5e59d2891667f2e0c82f0555ab4
  - kind: symbol
    name: raf_rdf::graph::Builder::default
    digest: 474e5b499dd1c09aaab8be654735027c55ffe6986cf3742669ca0f5362a4fd26
  - kind: symbol
    name: raf_rdf::graph::Builder::finish
    digest: 58e8be68104de35f880230a0bf39f3abcfce89870ab02a780e3ca7ad9bd62746
  - kind: symbol
    name: raf_rdf::graph::Builder::from_graph
    digest: 4a813072f21c2dee09b3aa3376d337076683ca43b0327f4e2d7022a4d9579193
  - kind: symbol
    name: raf_rdf::graph::Builder::insert
    digest: dfee03afc49202cb12069d0a8a299a7931d6ab76d810e4199fb7a4a3d9877398
  - kind: symbol
    name: raf_rdf::graph::Builder::new
    digest: 41e010ebd1b5f0151182b81b30a541a41f2b1eca8ab43555a479dd83d63cfee9
  - kind: symbol
    name: raf_rdf::graph::Builder::remove
    digest: 7581078dd06f4e1daf9e220c6fad4f9a8cc6b0a44d4dd604e0b69d02d7d44d4d
  - kind: symbol
    name: raf_rdf::graph::CONSTRUCTION_OWNER
    digest: 5045681cfe44c8a325b7fdfbfb2398a7d2ca5cadb5fdb28fbc85fc0dfbb55f9f
  - kind: symbol
    name: raf_rdf::graph::Graph
    digest: 632bd5ab9fc5ad8736a8662be43fb06fd8e991f341801239e213f7808eb56dbb
  - kind: symbol
    name: raf_rdf::graph::Graph::assertions
    digest: 4aff348e33da32e4a6c97462f4ed3bd8f0ab8b43140955c5cbf3f98ba050e91b
  - kind: symbol
    name: raf_rdf::graph::Graph::dataset
    digest: c9d587da3214096c8c3dff49eaa51973e48cc7ae8fda6da425881ce44376586f
  - kind: symbol
    name: raf_rdf::graph::Graph::from_text
    digest: e4db7534319269fc5f42764c96a38922119a7f86c98f2a42d231930d008f3cc2
  - kind: symbol
    name: raf_rdf::graph::Graph::get
    digest: ab25f284466a6f9661f952fd1b79c94f6dbdb7ff07231a1bfc9d0ddf1548cef3
  - kind: symbol
    name: raf_rdf::graph::Graph::id
    digest: b85cf7b6e1e72da4365d900975d99a02ca796158fa1d771b7c543886ea5ecafc
  - kind: symbol
    name: raf_rdf::graph::Graph::local
    digest: 5be695615f26ab8838cca12119def93536282bd661bf8c69b4b43392cc6dc041
  - kind: symbol
    name: raf_rdf::graph::Graph::nodes
    digest: 830a9dce7db8ef2a248e114a4dcbb9c0688944443df562b695ea1a8cdc8c8e7a
  - kind: symbol
    name: raf_rdf::graph::Graph::rows
    digest: 69fc879f6f287415c82052777d2d9a246955ecdba6c9b53284838d07ff36c6db
  - kind: symbol
    name: raf_rdf::graph::Graph::snapshot
    digest: 89b26118dd4cddf889264781e4261f84cf1469325529f7d26cfa5db77393f5cf
  - kind: symbol
    name: raf_rdf::graph::Graph::subject_rows
    digest: 6ddc319b3240f31b411391d064682383c04629e8b641333e8991d7a8c7e362a3
  - kind: symbol
    name: raf_rdf::graph::Graph::to_text
    digest: 5b2d3eb32251b7a474e29ca6fb054c55356e3acc625c5f9309915cc9d5fe9395
  - kind: symbol
    name: raf_rdf::graph::Graph::validate
    digest: d83ff38d0cd3ca130f01661f6ae84a2132d6712236ac6806c7c1f5e6682af4f1
  - kind: symbol
    name: raf_rdf::graph::Graph::value
    digest: fbcb8f8377f946d6a8f2c3397a4bfce1445f6af0893e91922bdec2e21d972de6
  - kind: symbol
    name: raf_rdf::graph::Node
    digest: 4b7396100792a51b3da12fa8f0b851ddc2c19b689da1c68db6f768e8e27f6fa6
  - kind: symbol
    name: raf_rdf::graph::Node::objects
    digest: f167d327dc642f7c1face3b9f4c61cdb199afcc7f5d209828f2cc3ddd0dca880
  - kind: symbol
    name: raf_rdf::graph::Node::properties
    digest: 036740dc9e38915f09420c441e38c24b85650c34be3d3d5e0a4cf99c1ace5961
  - kind: symbol
    name: raf_rdf::graph::Node::values
    digest: 38e82f9e9a61be3e663ea8fcdade578ffadaa13321ac4887851e095cc771f8ba
  - kind: symbol
    name: raf_rdf::graph::Value
    digest: eca0286773d60f92e654e707cc8bd4e248cbc9a7c4a88aaeb4e46a7114ddd455
  - kind: symbol
    name: raf_rdf::graph::Value::iri
    digest: 7c5cbef422ce1b84de34ef05548d7103a6559bf9587fffc2d203e0c47fd44697
  - kind: symbol
    name: raf_rdf::graph::Value::literal
    digest: d222f380795e9d35b7a3369d13fe65cf5a5b4ec98e6d1c9b92cd03ec0c6a11c6
  - kind: symbol
    name: raf_rdf::graph::Value::quoted
    digest: 23030c91cc58f6e1bd5ab7daf434e6a5197f97cdcdad31bc36e443f650ee09e0
  - kind: symbol
    name: raf_rdf::graph::Value::term
    digest: 5a27a5ca0e4377e0fce6e4d08d1ffcc4ef3c7bd268d9b9dea2e9439c36e0df2a
  - kind: symbol
    name: raf_rdf::graph::Value::to_owned
    digest: e06a39478528614ee26c08168917f1cc7dba60b6e9e9c4f00d359b5043a60ac3
  - kind: symbol
    name: raf_rdf::graph::tests
    digest: d28ef0fe86297f9272eff820af85ec332f2243d101cde5e16f75973aa2ff8287
  - kind: symbol
    name: raf_rdf::graph::tests::native_physical_statement_overlap_is_refused_without_losing_the_valid_statement
    digest: 9fe11bc37af555935b9d442177d8cb4f9a619fbacd7d8ceb91a3e682e2ca8459
  - kind: symbol
    name: raf_rdf::read
    digest: 671d2d383922d73e8b238314eda77432c0422ec5801f5fa73df982eff3cc58b9
  - kind: symbol
    name: raf_rdf::read::checked_base
    digest: 40c22ccd68aee5161159ebc22b5a0ad0c22625baed8bee458c06fb5d03c890c7
  - kind: symbol
    name: raf_rdf::read::parse
    digest: 1bc2dde389176ce0e1b02b335326c0d2a8d08f6c8b8892138807c8ea39785ef2
  - kind: symbol
    name: raf_rdf::read::root_base
    digest: e83cb50d49e36b319d02ccfc84d85f8e69692533c518c630c6019d2a17ab82a8
  - kind: symbol
    name: raf_rdf::read::tests
    digest: 83d20aac1314bde5aaadbe845c522bb41a8d02c993796c7ad1a28453d9013997
  - kind: symbol
    name: raf_rdf::read::tests::ownership_normalization_checks_the_selected_root_boundary
    digest: f66d2440cacc3ef2da929332be9df38ce82a53b6e202f71a007f24c1f0fb94b6
  - kind: symbol
    name: raf_rdf::term
    digest: 10f0fb20f3b4c5bf8d89f0fb7b45c94bbb4972d9bd4513544238d34249c45466
  - kind: symbol
    name: raf_rdf::term::Triple
    digest: '0b99297f51c8a8de85e14347093e4ac48d365f1e4ba7e987c9d87c9376568098'
  - kind: symbol
    name: raf_rdf::term::Triple::validate
    digest: c02594aa576d4ccbc47a3259bc94db750572413cedd6df6302cf7ce94a106986
  - kind: symbol
    name: raf_rdf::term::atom
    digest: 504eee1752fd1f0256a1f1d766db427d8c008a4ca236466ecb4176bac3a97cfd
  - kind: symbol
    name: raf_rdf::term::expand
    digest: bc6914d53a072fde6fa800859115146712ddcdf22f46442d7b1dd9ed85fbf5fd
  - kind: symbol
    name: raf_rdf::term::iri
    digest: aafae91c22427f4cc13517d075ada0609aa4b75dca951ddac21ad1539b0d40fd
  - kind: symbol
    name: raf_rdf::term::local
    digest: 77c7708a1902007485f979ccf7f4a3f588bb1003e741d2f440f657748c2b1738
  - kind: symbol
    name: raf_rdf::term::normalize
    digest: 36dd4d77cbfb93695d8304b6d76e101a3f91f5ec24ac543a7afdcc9508e45c76
  - kind: symbol
    name: raf_rdf::term::validate
    digest: 436c1875c759a459b29f6e6654801abc447eeb69e01bb2f75ea8376470777c03
  - kind: symbol
    name: raf_rdf::test_crate::profile
    digest: 41d7fdbe54643bd0bef24525e4c2435ad5cc22edf7ef1f491d09d41891238879
  - kind: symbol
    name: raf_rdf::test_crate::profile::abbreviations_blank_nodes_and_quads_are_outside_the_closed_profile
    digest: 2adbfc84237cbdcfa4c9b6f60d74efa87c597207e799e3b9c429faea8ceac365
  - kind: symbol
    name: raf_rdf::test_crate::profile::absolute_iri_and_literal_profile_refusals_keep_valid_neighbors
    digest: e8410eacb79c1d279517aa6edd7481c690efb7bd3f04b79361838d7d535af4cc
  - kind: symbol
    name: raf_rdf::test_crate::profile::absolute_iris_use_component_validation_without_normalizing_identity
    digest: 2ee9efcee5cf2ea4ee387db8e4b98485749a57c71dc5b8b55961ac09f3333ec2
  - kind: symbol
    name: raf_rdf::test_crate::profile::an_external_iri_sharing_a_snapshot_prefix_is_not_local
    digest: 4754963c5911f75cfd525c5bfe181254c67b865c493fad2f98e4a530f0b2ad56
  - kind: symbol
    name: raf_rdf::test_crate::profile::assertions
    digest: ffd7210784b2b9e3f243f01a13d738e86f42de5d405b25875954efa0c428f592
  - kind: symbol
    name: raf_rdf::test_crate::profile::both_readers_preserve_role_language_depth_and_ownership_refusals
    digest: b0d27e95a8e28cd908a49472975fc8ad69212779a231a0b5070624c15c63f6d8
  - kind: symbol
    name: raf_rdf::test_crate::profile::build
    digest: 16fd7b8d1cfb2dc96904daa504956dc8d1621cdc83edc1688f6740e912687ef9
  - kind: symbol
    name: raf_rdf::test_crate::profile::caller_supplied_large_literals_are_validated_without_a_profile_byte_budget
    digest: e4bbe6fff1959c6198793fa8491f9f17b0571aa3c1d330161583c14f168909db
  - kind: symbol
    name: raf_rdf::test_crate::profile::duplicate_native_membership_and_real_subject_queries_have_valid_neighbors
    digest: f76d1608de0724a195a7d0b7392b43846645daab287284473bc06569da9b33a1
  - kind: symbol
    name: raf_rdf::test_crate::profile::exact_bytes_include_cr_final_lf_and_multibyte_diagnostic_neighbors
    digest: fab145664065dec8db8170fb7ac9754d993f4bb574af2abcd4423167ef5a1b12
  - kind: symbol
    name: raf_rdf::test_crate::profile::example
    digest: b0bb2115533b8c46c887f8e378493894bdb03a399f83954bef2afa491b2faec9
  - kind: symbol
    name: raf_rdf::test_crate::profile::ground_validation_does_not_apply_domain_shapes_or_datatype_value_spaces
    digest: 7929be68de79e644fa1618b8aa5be1373ec1710b671b8b4d9115f6c5426d03a5
  - kind: symbol
    name: raf_rdf::test_crate::profile::ingestion_order_does_not_change_either_format_or_snapshot
    digest: 6a6df69ef243235c4e0f024de1f08abda16d795888bca09ffcfcd57a6aaca63f
  - kind: symbol
    name: raf_rdf::test_crate::profile::malformed_escapes_and_predicates_refuse_beside_canonical_neighbors
    digest: 2e0ca83b5068d616c6a84ef9af064bc2472560cbf40a27292b1af90475ed05d3
  - kind: symbol
    name: raf_rdf::test_crate::profile::parsed_statement_duplicates_and_repeated_source_bytes_refuse_with_valid_neighbors
    digest: 3f34f10f7bbcf67f6ee0906ae7fd84e8865c87f8b41ceb4bf336929eb05fbd41
  - kind: symbol
    name: raf_rdf::test_crate::profile::pinned_registry_expansions_remain_distinct_from_native_canonical_bytes
    digest: 5264dd9450027663e105b8d3918e7ca568aeb0093476359eaef25f5e1fa38404
  - kind: symbol
    name: raf_rdf::test_crate::profile::quoted
    digest: 709676b8051f8cbab7549351e723c3327d783e5a9126a90c508d5fb9543a06af
  - kind: symbol
    name: raf_rdf::test_crate::profile::roles_missing_roots_and_dangling_references_refuse_beside_valid_graphs
    digest: f9dd8a194798c52793bceec2451656ac930f0bec5b671a3b72ce96328de4633c
  - kind: symbol
    name: raf_rdf::test_crate::profile::triple
    digest: 220266bc2b6a54dad4642ede60ea34b92935b1920f331c7f0418451fa8446ceb
  - kind: symbol
    name: raf_rdf::vocab
    digest: 64f793136f5732e084717b359cb62fb912c89d53ca155199a13cf0d3ae294d34
  - kind: symbol
    name: raf_rdf::vocab::BASE
    digest: f59b9daf1798975d5d68665e13d0a4ebf0443292ea33f7f1b303b7d3dcf699f4
  - kind: symbol
    name: raf_rdf::vocab::REPORT
    digest: 9954b3230ec2db663b30b5fa8c513cb29b2d7e48c984aaa90aaf3f82a034a871
  - kind: symbol
    name: raf_rdf::write
    digest: 1bbb2403a62bed100a848df8f0219e586c670fc8978f71f74a90d1a3a1f60ee2
  - kind: symbol
    name: raf_rdf::write::Selection
    digest: '0b997083e1f63d2c0668575338d7b8e3e9b7719c0243133273151fc196725630'
  - kind: symbol
    name: raf_rdf::write::Selection::iri
    digest: 6defe8dd0ae9f180cf0dac7d5bde5a3303f3d2a775d81746c8f8cd3a38a699b5
  - kind: symbol
    name: raf_rdf::write::Selection::new
    digest: c8fbb1ee0f43b9bfdf52a6fd893f39fc2bd212cd1a870fc3f933fdca24479251
  - kind: symbol
    name: raf_rdf::write::base
    digest: 7af3ffcc4ec8e277cea4d4514555ce8cf8981f89455d7bccc953c9d5e959a681
  - kind: symbol
    name: raf_rdf::write::body
    digest: 6607860d17f47fc411416674c4b685e4625dfc579d759663b814d2eaf744c7b5
  - kind: symbol
    name: raf_rdf::write::identity
    digest: 54b7d62c22cb21a2d34e2c62c4699f0146e4829908c0a9f586bb8cdcc4d72bf8
  - kind: symbol
    name: raf_rdf::write::normalized
    digest: fdd04d527b2ac7dee4836e21bf937fb92ff5db0c0ff4424ea3e03f1cfc24787b
  - kind: symbol
    name: raf_rdf::write::render
    digest: bae90f7c082343220d78c27f4e1490dab58cc5af7a296694c4f88c8575e0883a
verdict: findings
by: agent
findings: []
findings_ref: docs/reviews/raf-rdf-design.md
---
