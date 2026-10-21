// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The golden values of normal form 1.
//!
//! A version is fixed by the content of its first merge to the default branch,
//! and from then on ANY CHANGE TO A PINNED VALUE IN THIS FILE REQUIRES A
//! `normal_form` BUMP. A fingerprint names what a body says under one version
//! of the form, and a ledger stores fingerprints, so a pinned value that moves
//! without a new version silently breaks every ledger. When the form changes on
//! purpose, bump `NORMAL_FORM` once the version is fixed, run
//! `cargo run -p raf-census --example goldens`, and paste its output over
//! `MANIFEST` and `TABLE` in the same change.
//!
//! The test fails when a registry fixture has no pinned line, when a pinned
//! line has no fixture, and when a pinned value differs from the computed one.

mod support;

use std::collections::BTreeMap;

use support::fixtures::{registry, table};

/// The manifest of normal form 1: encoding, passes, levels, thresholds, lists.
const MANIFEST: &str = "\
encoding tokens-len-u32le-utf8/blake3/hex64\n\
passes rename_params rename_locals kind_literals erase_held\n\
levels verbatim shim literal-tier structural value\n\
literal_tier 30\n\
min_shim_tokens 6\n\
shim_max_tokens 30\n\
max_nesting 128\n\
value_adapters as_str as_ref as_slice as_mut as_deref borrow borrow_mut clone cloned copied to_owned\n\
weak_keywords gen union raw safe macro_rules\n\
";

/// One line per registry fixture: `id tokens body shim`.
const TABLE: &str = "\
structural_renamed_locals.a 21 body:7eba4ed550accd1d4ed45f2dc46a44626b59f27d0b032e0d3ca9466fb92dea36 -\n\
structural_renamed_locals.b 21 body:7eba4ed550accd1d4ed45f2dc46a44626b59f27d0b032e0d3ca9466fb92dea36 -\n\
structural_renamed_params.a 21 body:7eba4ed550accd1d4ed45f2dc46a44626b59f27d0b032e0d3ca9466fb92dea36 -\n\
structural_renamed_params.b 21 body:7eba4ed550accd1d4ed45f2dc46a44626b59f27d0b032e0d3ca9466fb92dea36 -\n\
structural_renamed_lifetimes.a 23 body:6655792c092e7b92c44992ad9bfa9c9ea3d9eacb662cc5b6267acd45c7d31d7e -\n\
structural_renamed_lifetimes.b 23 body:6655792c092e7b92c44992ad9bfa9c9ea3d9eacb662cc5b6267acd45c7d31d7e -\n\
structural_different_function.a 21 body:7eba4ed550accd1d4ed45f2dc46a44626b59f27d0b032e0d3ca9466fb92dea36 -\n\
structural_different_function.b 21 body:147db542e10d6fdb69c0f73e333a05d628d2d9de6e7f85646449d6c49135dc59 -\n\
structural_different_method.a 12 body:aef8faa355e76e88c0dee747673cc15603e26ac3c8a75754ee9ebf97ae188ab1 -\n\
structural_different_method.b 12 body:119083d53ce8028e534d4054dabd01b2d97f80fed1aae2bcb26ff036846862a3 -\n\
structural_different_field.a 10 body:5af97b733352c98580b5a847ba44ff65eb0cabe623aad67ec2d8fe939a35cf77 -\n\
structural_different_field.b 10 body:9de15726987470056305d6e73f6fec3432dded12f97412bab190f34cfba61656 -\n\
structural_different_type.a 15 body:7d1a4d959447ea281d91dae90ea69c008d14cf3aa0b751f2ff8642cb1ca44082 -\n\
structural_different_type.b 15 body:1c6ca5063215def73fd1d1e529abc6e201dc275b572e3f4018ca8ae253640465 -\n\
structural_local_type_declared.a 13 body:decd3988380fa1a558a7d41ebad7422d4978e63f686699f8ec6ca0f3d2e31fe0 -\n\
structural_local_type_declared.b 13 body:decd3988380fa1a558a7d41ebad7422d4978e63f686699f8ec6ca0f3d2e31fe0 -\n\
structural_tuple_field_kept.a 7 body:e2a9d8c5dbcad44fb20bcaa179bf9e2dd427c179478ef33b797c1c05f58ee609 -\n\
structural_tuple_field_kept.b 7 body:4f345172b382003e665c8370b59ae57177472ce378485ef1c26140fbfef48a3e -\n\
binder_let.a 12 body:95ab577a2b33b1a9ab326429788d45452204f89b9bf12da5811dac25797b617a -\n\
binder_let.b 12 body:95ab577a2b33b1a9ab326429788d45452204f89b9bf12da5811dac25797b617a -\n\
binder_let_tuple.a 18 body:ce48def1369fd476d5c0145b94df238b1f6ef856ed2ff76b9e1f84cca1d40256 -\n\
binder_let_tuple.b 18 body:ce48def1369fd476d5c0145b94df238b1f6ef856ed2ff76b9e1f84cca1d40256 -\n\
binder_let_struct.a 20 body:24afc73f3ccd9ef49863c848165cd8be3aa2aa0ded211eaa079b85e887ee6d12 -\n\
binder_let_struct.b 20 body:24afc73f3ccd9ef49863c848165cd8be3aa2aa0ded211eaa079b85e887ee6d12 -\n\
binder_closure_param.a 11 body:8d22095fbc512e009fbe15844abb56281d2bfd46fc33d477f7fee7c01f1794bc shim:f1f10228fc9a17a3272563a76cc997c0afa167ddea39cb2fdb5e1836ab4e58e3\n\
binder_closure_param.b 11 body:8d22095fbc512e009fbe15844abb56281d2bfd46fc33d477f7fee7c01f1794bc shim:3d04f0ef836626da79dbe628e280458c31c2a36a630b1e491d1f70e792fffc20\n\
binder_for.a 11 body:64e10090f7ac2a2e5b46360fd3bf748d3b913c626dc85e669f28fccfb3715c7a shim:7079b970a584887b1ca96db21bd58d677f3dbbd115024a27dea2d04e86e02258\n\
binder_for.b 11 body:64e10090f7ac2a2e5b46360fd3bf748d3b913c626dc85e669f28fccfb3715c7a shim:9c2a4db9cff8701c64a9156a5063e75b6313c5fad828953501dcb45241d765ef\n\
binder_match_arm.a 19 body:c51942289b8a65e781209b09a473c679e9f5df44e25d63ab0748be177dea5c3e shim:49c20ccc3dd9ed2f02945200fe918e61be39fbc656b419a7fc20cecc313d222f\n\
binder_match_arm.b 19 body:c51942289b8a65e781209b09a473c679e9f5df44e25d63ab0748be177dea5c3e shim:1e231492e23d499b431c991f8dd34f2924c0e33b40248328248b9724ebee6778\n\
binder_if_let.a 18 body:38f212b00c4e9b928d55df8f99e1ae41f8c64453c8c2d69cc68dfeae48d5fd35 -\n\
binder_if_let.b 18 body:38f212b00c4e9b928d55df8f99e1ae41f8c64453c8c2d69cc68dfeae48d5fd35 -\n\
binder_while_let.a 19 body:46ebfa0602c11e7ace0884734af90c0adb7d9a6f0dd8c11cb7daaca30dd30711 -\n\
binder_while_let.b 19 body:46ebfa0602c11e7ace0884734af90c0adb7d9a6f0dd8c11cb7daaca30dd30711 -\n\
binder_at_binding_match_arm.a 22 body:47a8e2d90e7de853773b15325640bd02faae2d7e16d0bffc642694e19fb824e6 shim:bf079faf7d3d7788515672d7250e23d60bbb749f63e2a6c1bdb6813787bd161c\n\
binder_at_binding_match_arm.b 22 body:47a8e2d90e7de853773b15325640bd02faae2d7e16d0bffc642694e19fb824e6 shim:3a22ace1b228da118c53d35b88a40c6f1f866c010b162bb575b5170d7242523f\n\
binder_at_binding_if_let.a 24 body:b8469c5781b9287c6afc004f19b2d5f3295704fd9d1aefb93389d48bcbeaf470 -\n\
binder_at_binding_if_let.b 24 body:b8469c5781b9287c6afc004f19b2d5f3295704fd9d1aefb93389d48bcbeaf470 -\n\
binder_label.a 17 body:b17d7810ac4506ab898e6db6564d3e4660fd9e0d3d2f74faab931d3f6fc1efe6 shim:61ecd3c261e33ff15fa4d1b01ad5f623ee23c89668ab60f65d8ed05d84918581\n\
binder_label.b 17 body:b17d7810ac4506ab898e6db6564d3e4660fd9e0d3d2f74faab931d3f6fc1efe6 shim:6b9712b2ac29106456d16185d888be65dad09f6d7774c5a2c7040fe5b0233a90\n\
binder_lifetime.a 25 body:3498957fe46e7bdb2c560ff558ebda450b93c51ccb2d13bd6903db806d6a433c shim:35075f7d9dbca4ffc40b9aa5beff7d5d6b49936c78cf398c0dcb15703fc5ec17\n\
binder_lifetime.b 25 body:3498957fe46e7bdb2c560ff558ebda450b93c51ccb2d13bd6903db806d6a433c shim:646a83a3d347c20aa7fcb41814dd318b33df834b66686e715889271e93252db8\n\
binder_local_fn.a 19 body:b33834e695b2729538b5b82a7436553e604d2ba4c1191f052db5b271e404b6ad shim:798cd2a2d81dcaea029528d87ec606e0056a45ce2243a085c0f7320708b23c76\n\
binder_local_fn.b 19 body:b33834e695b2729538b5b82a7436553e604d2ba4c1191f052db5b271e404b6ad shim:ad3995d8d8c92cc340e965042a258c4913257055f20cdd90d0ae29875a483c73\n\
binder_local_struct.a 13 body:decd3988380fa1a558a7d41ebad7422d4978e63f686699f8ec6ca0f3d2e31fe0 -\n\
binder_local_struct.b 13 body:decd3988380fa1a558a7d41ebad7422d4978e63f686699f8ec6ca0f3d2e31fe0 -\n\
binder_call_position.a 14 body:67ab1937176da6b7e6a60ca1c8b6fbb49f3417c4a0e83d1083f7cdaa0ef7c3cf -\n\
binder_call_position.b 14 body:67ab1937176da6b7e6a60ca1c8b6fbb49f3417c4a0e83d1083f7cdaa0ef7c3cf -\n\
binder_shadows_an_item.a 12 body:e574c21f9d162ffa1f9081f62cebee05a6a88fc7ef174b29ef477c01914ac0ae -\n\
binder_shadows_an_item.b 12 body:e574c21f9d162ffa1f9081f62cebee05a6a88fc7ef174b29ef477c01914ac0ae -\n\
binder_macro_metavariable.a 18 body:19d5de816f22dba7b2b582370256074ed2bb2457d9ed583486ec5c3f83f962fc -\n\
binder_macro_metavariable.b 18 body:19d5de816f22dba7b2b582370256074ed2bb2457d9ed583486ec5c3f83f962fc -\n\
item_function_passed_by_name.a 14 body:2de5bf3e4a15f6aa928c5e79973fda6cab529f00cb58a74328f2792cf6573305 shim:2de5bf3e4a15f6aa928c5e79973fda6cab529f00cb58a74328f2792cf6573305\n\
item_function_passed_by_name.b 14 body:022271b4904b12f99210899252bcbb31ffe189480e1a5e9a5fbb3ec27b3a76f1 shim:022271b4904b12f99210899252bcbb31ffe189480e1a5e9a5fbb3ec27b3a76f1\n\
item_predicate_passed_by_name.a 14 body:f8737da45c113adbaf7d56e799e9bb003655b78302ae1d5156874139531fae57 shim:f8737da45c113adbaf7d56e799e9bb003655b78302ae1d5156874139531fae57\n\
item_predicate_passed_by_name.b 14 body:5fd82dee390771121ba2e48edde373545af10b7a52ad9b6e2e9360fd2c5fc512 shim:5fd82dee390771121ba2e48edde373545af10b7a52ad9b6e2e9360fd2c5fc512\n\
item_lowercase_static.a 3 body:554875b54e03781463d28f2e4b0f440f9e7026cd520d3f2f079c45bd840e857a -\n\
item_lowercase_static.b 3 body:8e8085da3443f90fcd39bce9a500e5bf2b5765dd60e057508c8ff9b8c25a85a0 -\n\
item_in_struct_shorthand_field.a 4 body:49dac68971098cc17f0bd2a80ec8e818e16873ad18884e7343aaa95d745bb0ba -\n\
item_in_struct_shorthand_field.b 4 body:6e01a576a56490a2c0aa4281f607eb15caffa4b418b268fbebfa8c6a69bb3237 -\n\
item_field_name_kept_beside_a_local.a 11 body:bfbf8fbc1ba0c2c0ddf8d7fa4b67b6e0b35feb54b243d7ccf9d31bc414fa64e2 -\n\
item_field_name_kept_beside_a_local.b 11 body:e6b771da520a1ebe759fdb5246a1d70f732dd6d8c53daf0876467792e19cce8a -\n\
head_block_if.a 26 body:b48bd54506cdefb3d55912590625c4c426e2309c442e3384335cd50f00f3623d -\n\
head_block_if.b 26 body:b48bd54506cdefb3d55912590625c4c426e2309c442e3384335cd50f00f3623d -\n\
head_block_else_if.a 28 body:8f6dadb834bce346109d54d73ea982a375d38d32a3b5777d765c38fc652a4ee0 -\n\
head_block_else_if.b 28 body:8f6dadb834bce346109d54d73ea982a375d38d32a3b5777d765c38fc652a4ee0 -\n\
head_block_while.a 14 body:29c41d6ed732840fa2eeeae150edf35aa3fdd245b260b273468aa1a6dc527eeb -\n\
head_block_while.b 14 body:29c41d6ed732840fa2eeeae150edf35aa3fdd245b260b273468aa1a6dc527eeb -\n\
head_block_match.a 20 body:64524777c9baf3fda9011d4c35e0f4bb181b202ad8a1ec9a06bcb3d6315ed670 -\n\
head_block_match.b 20 body:64524777c9baf3fda9011d4c35e0f4bb181b202ad8a1ec9a06bcb3d6315ed670 -\n\
head_block_for.a 23 body:d1a6109ed96cf2349162d7628f81c2d1ab2f2d7646f625bbcbdfaf7997b3ad68 -\n\
head_block_for.b 23 body:d1a6109ed96cf2349162d7628f81c2d1ab2f2d7646f625bbcbdfaf7997b3ad68 -\n\
threshold_literal_tier_straddle.a 21 body:37da5aac3e618d930835fe916157aa45eabfde986654a5faf9fec53f2cbe8a47 -\n\
threshold_literal_tier_straddle.b 21 body:37da5aac3e618d930835fe916157aa45eabfde986654a5faf9fec53f2cbe8a47 -\n\
threshold_min_tokens_straddle.a 15 body:7ed517820c72f5a530067710e1a9397754f3922796d182d71cbd6e1f5f5942d7 shim:7ed517820c72f5a530067710e1a9397754f3922796d182d71cbd6e1f5f5942d7\n\
threshold_min_tokens_straddle.b 15 body:7ed517820c72f5a530067710e1a9397754f3922796d182d71cbd6e1f5f5942d7 shim:aa42edc601af1b15db6183c201db9d8368770fab7f7b6b78ca8a83705d298fbb\n\
value_borrow.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_borrow.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_borrow_mut.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_borrow_mut.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_double_borrow.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_double_borrow.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_deref.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_deref.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_ref_binding.a 19 body:c51942289b8a65e781209b09a473c679e9f5df44e25d63ab0748be177dea5c3e shim:291cf14185c65d604cb18ad5f8ef9cedb2483ba72a58579f5548caa54213325f\n\
value_ref_binding.b 19 body:c51942289b8a65e781209b09a473c679e9f5df44e25d63ab0748be177dea5c3e shim:47be6280f16d0bb842fd895cf0eb5b5e08212c6e5aa8535f151ff3eb85fc11b4\n\
value_ref_mut_binding.a 19 body:c51942289b8a65e781209b09a473c679e9f5df44e25d63ab0748be177dea5c3e shim:f01dfa44a5b9ec50d248314c3637805b7cd69f3842cb80ed40b32cdf6af98375\n\
value_ref_mut_binding.b 19 body:c51942289b8a65e781209b09a473c679e9f5df44e25d63ab0748be177dea5c3e shim:47be6280f16d0bb842fd895cf0eb5b5e08212c6e5aa8535f151ff3eb85fc11b4\n\
value_adapter_as_str.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_as_str.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_as_ref.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_as_ref.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_as_slice.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_as_slice.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_as_mut.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_as_mut.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_as_deref.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_as_deref.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_borrow.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_borrow.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_borrow_mut.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_borrow_mut.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_clone.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_clone.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_cloned.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_cloned.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_copied.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_copied.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_to_owned.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_to_owned.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_adapter_with_argument_kept.a 9 body:2347a33ac3e3534443013cdec45597d964a41051c4f3f07a00c9f3bd8afd8265 shim:2347a33ac3e3534443013cdec45597d964a41051c4f3f07a00c9f3bd8afd8265\n\
value_adapter_with_argument_kept.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_full_range.a 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_full_range.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_partial_range_kept.a 9 body:94e9d03d52d1a4b5510a6a8736879a51f987012a6d8a95ce5241d629f83307b2 shim:d063387bf91105159101aa0edd840c231a097ed6a6536ff738f4b379cdf5763d\n\
value_partial_range_kept.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_range_to_kept.a 9 body:a6765939ad39f54af47fa6e6eb283029e6d2adbc847d9953da65ca5cac128686 shim:93141a8e3e9bb77a0e9e0db43bfb3365b26fa2a2ae050e816b408a4072538ae3\n\
value_range_to_kept.b 4 body:0f767795a70a02bb2f41642fc65ae92a5b248b4401b8a3754b0823031a67707f -\n\
value_let_annotation.a 9 body:5b76112b24b9e8c336a6e299bbb44a0328b4c745bb9ef0d8830d9b83e0944a8a -\n\
value_let_annotation.b 9 body:5b76112b24b9e8c336a6e299bbb44a0328b4c745bb9ef0d8830d9b83e0944a8a -\n\
value_let_annotation_generic.a 9 body:5b76112b24b9e8c336a6e299bbb44a0328b4c745bb9ef0d8830d9b83e0944a8a -\n\
value_let_annotation_generic.b 9 body:5b76112b24b9e8c336a6e299bbb44a0328b4c745bb9ef0d8830d9b83e0944a8a -\n\
value_erased_names_do_not_shift_numbers.a 17 body:3ba2ecaed0b39ba642e9788325d4a4aba5d2eed3dbdf3d0112e0678af2ab1f71 -\n\
value_erased_names_do_not_shift_numbers.b 17 body:3ba2ecaed0b39ba642e9788325d4a4aba5d2eed3dbdf3d0112e0678af2ab1f71 -\n\
value_type_turbofish.a 5 body:7fb66880a8b3a845eb586c2594e5c4694d23ba6035f9f358f3264774da9a4dec -\n\
value_type_turbofish.b 5 body:7fb66880a8b3a845eb586c2594e5c4694d23ba6035f9f358f3264774da9a4dec -\n\
value_const_turbofish_kept.a 9 body:45026bbba235f1d203cd9b17950f6b127d6d810714490d97b2de01216830a5a6 shim:45026bbba235f1d203cd9b17950f6b127d6d810714490d97b2de01216830a5a6\n\
value_const_turbofish_kept.b 4 body:534ed5ba5aac4f5c535ba8f36508b6af8bc12179688c65104278cb4c35f07479 -\n\
value_bool_turbofish_kept.a 9 body:87ee53773ccf49ef9903d23166e847029472a87e8376986a9910dfb5bec5055c shim:87ee53773ccf49ef9903d23166e847029472a87e8376986a9910dfb5bec5055c\n\
value_bool_turbofish_kept.b 9 body:020a9a61097d48357c0f34ae9987267b4542a9bafc25e05da5cfbd23365a1b03 shim:020a9a61097d48357c0f34ae9987267b4542a9bafc25e05da5cfbd23365a1b03\n\
value_index_reference.a 4 body:b195ad401e59e44475a993c4461bea325f1633c00146acabb07c54127c739d93 -\n\
value_index_reference.b 4 body:b195ad401e59e44475a993c4461bea325f1633c00146acabb07c54127c739d93 -\n\
value_binary_and_kept.a 6 body:4ff5873ae1028c47e427ee6732fd6614db42df8dec89366252eda6baf2f7544e shim:4ff5873ae1028c47e427ee6732fd6614db42df8dec89366252eda6baf2f7544e\n\
value_binary_and_kept.b 5 body:ae705e38df2ce7e69e7f0269862350460623ec1e3bda0ad4806fa4f073d34c1d -\n\
value_binary_mul_kept.a 6 body:cf6fcb46129a87b3688e9b19645be00d1bf8b0cc0bb91c008833ff0187b13f43 shim:cf6fcb46129a87b3688e9b19645be00d1bf8b0cc0bb91c008833ff0187b13f43\n\
value_binary_mul_kept.b 5 body:ae705e38df2ce7e69e7f0269862350460623ec1e3bda0ad4806fa4f073d34c1d -\n\
value_logical_and_kept.a 7 body:17a8159319a7b25444aaf63da1bc5a46f4e637a91443985785ef5f04ce70ec44 shim:17a8159319a7b25444aaf63da1bc5a46f4e637a91443985785ef5f04ce70ec44\n\
value_logical_and_kept.b 5 body:ae705e38df2ce7e69e7f0269862350460623ec1e3bda0ad4806fa4f073d34c1d -\n\
value_compound_assign_kept.a 10 body:a972fadf6c45665f1877d86ba40b465370f0c83709f0d0573989a0688d3157aa -\n\
value_compound_assign_kept.b 6 body:5e73238be60a98f7777ff878152c8549b8f35cc118cd7fe699c46f90277c32ea -\n\
value_binary_after_call_kept.a 6 body:14555388857cf65fd1e5c05e8bb08304979861daacd4340667bf5f1463793684 shim:14555388857cf65fd1e5c05e8bb08304979861daacd4340667bf5f1463793684\n\
value_binary_after_call_kept.b 5 body:dbd2b465e2f37a46873544726707b7b805d55a13159b758513a18e0e8bc39a3c -\n\
tier_25.a 25 body:bae17ce423758aed59ff2128052ad945c9906f5e7990caeb7c3054a2a15914c9 -\n\
tier_25.b 25 body:291a4f75ee3b7d6e239e100a958b008ce91d01da1fa4a123c1ebe1f91d2fa18c -\n\
tier_29.a 29 body:5363862fa58ae6611c24be5b41b6e707df7b88abd6d653a7cd401c85c78c44a1 -\n\
tier_29.b 29 body:147e186d4c12f6b27f8e5b0cfcfe434b25dab3e9f58af57adb0ae1d7402c53e4 -\n\
tier_30.a 30 body:0fe4f26092b867f9079243dc32fae3befeffd38a9940fe8f3c8631f942cc8c35 -\n\
tier_30.b 30 body:0fe4f26092b867f9079243dc32fae3befeffd38a9940fe8f3c8631f942cc8c35 -\n\
tier_35.a 35 body:742087969dfcf3f6d657ef34c0dd306f1ba56a5b7d0db96bc4bab3a7d6c42382 -\n\
tier_35.b 35 body:742087969dfcf3f6d657ef34c0dd306f1ba56a5b7d0db96bc4bab3a7d6c42382 -\n\
tier_30_kinds_differ.a 30 body:0fe4f26092b867f9079243dc32fae3befeffd38a9940fe8f3c8631f942cc8c35 -\n\
tier_30_kinds_differ.b 30 body:7ab910272c01895729672c4183ebd06d9cfdeeaafb842723918f1a4db0cdb1e7 -\n\
tier_30_float_is_not_int.a 30 body:a9c176eabd1d7aa9d9a2f1b0bd5bb8c881edd9c6ab4fc5477851121eb9cb3569 -\n\
tier_30_float_is_not_int.b 30 body:7ab910272c01895729672c4183ebd06d9cfdeeaafb842723918f1a4db0cdb1e7 -\n\
tier_30_big_int_is_int.a 30 body:7ab910272c01895729672c4183ebd06d9cfdeeaafb842723918f1a4db0cdb1e7 -\n\
tier_30_big_int_is_int.b 30 body:7ab910272c01895729672c4183ebd06d9cfdeeaafb842723918f1a4db0cdb1e7 -\n\
tier_29_big_int_kept.a 29 body:2bdd17114b5c62300205b1d95994ac41860672cfec9b44639e65f351521d0788 -\n\
tier_29_big_int_kept.b 29 body:2f5cd06a65147fcde833c495cf5469d4bed28c57671d76641641215930038b8b -\n\
tier_30_suffix_is_part_of_the_kind.a 30 body:7ab910272c01895729672c4183ebd06d9cfdeeaafb842723918f1a4db0cdb1e7 -\n\
tier_30_suffix_is_part_of_the_kind.b 30 body:7ab910272c01895729672c4183ebd06d9cfdeeaafb842723918f1a4db0cdb1e7 -\n\
tier_30_tuple_index_kept.a 30 body:719a04df3958129a4db50331b18127a5ce3b4084c92e6250bc93de8ae906aedf -\n\
tier_30_tuple_index_kept.b 30 body:573b08b98205895a8e1a481731f92f6cc730871e0b58033e200392fb2ff2e143 -\n\
tier_30_range_end_is_a_literal.a 30 body:1e4c474c7db0f5a5fae3c0c50c1c336e531d395a73c06aa24aee46cd513e0fd6 -\n\
tier_30_range_end_is_a_literal.b 30 body:1e4c474c7db0f5a5fae3c0c50c1c336e531d395a73c06aa24aee46cd513e0fd6 -\n\
literal_hex_and_decimal.a 4 body:260e2d80b4f84d52dda59e88fcbdca6c1fcb6ba6fa758fed817c0a4d313beeed -\n\
literal_hex_and_decimal.b 4 body:260e2d80b4f84d52dda59e88fcbdca6c1fcb6ba6fa758fed817c0a4d313beeed -\n\
literal_separators.a 4 body:8b9ed6eb71e560784cc4a2064411e22a648707462f327c4a8e23299cc02a25f4 -\n\
literal_separators.b 4 body:8b9ed6eb71e560784cc4a2064411e22a648707462f327c4a8e23299cc02a25f4 -\n\
literal_octal_binary.a 6 body:a5a021dc7409bb02b5b619ddfca3ec5fe2569676f581b80f17041260b8338ce8 shim:a5a021dc7409bb02b5b619ddfca3ec5fe2569676f581b80f17041260b8338ce8\n\
literal_octal_binary.b 6 body:a5a021dc7409bb02b5b619ddfca3ec5fe2569676f581b80f17041260b8338ce8 shim:a5a021dc7409bb02b5b619ddfca3ec5fe2569676f581b80f17041260b8338ce8\n\
literal_suffix_kept.a 4 body:d1d35c896bca55db8e9303bc0191fa6d8c6ddc7f4988088507d4988ce55f87cd -\n\
literal_suffix_kept.b 4 body:d1d35c896bca55db8e9303bc0191fa6d8c6ddc7f4988088507d4988ce55f87cd -\n\
literal_different_value.a 4 body:260e2d80b4f84d52dda59e88fcbdca6c1fcb6ba6fa758fed817c0a4d313beeed -\n\
literal_different_value.b 4 body:8b99bdb0caa7c4be8bc76a384c5b90ffbf70753ca76fd64bb45f3849aa0bacef -\n\
literal_suffix_differs.a 4 body:d1d35c896bca55db8e9303bc0191fa6d8c6ddc7f4988088507d4988ce55f87cd -\n\
literal_suffix_differs.b 4 body:093bbeb77c74b484ab42f290dae86e6f5c5cbd0794c7fd7773c86c48ddd89e98 -\n\
literal_float_is_not_int.a 4 body:c8af592a4ac2eb3fbd103d5117256fb4a6b4d5a10f8c904c0934caba89b8ed76 -\n\
literal_float_is_not_int.b 4 body:9de2285b669c60b20dc473ef2fc2860bfea208c8bdcf9a421c452ce987d156d1 -\n\
literal_string_spelling_differs.a 4 body:31cb562ee0c5f4b383f1a8d6212b937548d98f49f14eecd28eaf6051f6b60be2 -\n\
literal_string_spelling_differs.b 4 body:cfd65c1c5bc135c1f545b22f8d144bc79faea9b715228ed99bc1aa5f967d0a5c -\n\
shim_5 5 body:072159ca8d2de444481f93b0e01e53080f3d94108903a72342c4b5eaaed27506 -\n\
shim_6 6 body:900b617d60d4ddb6ea0d75aad4149216e2980c83b1a564ca5845488b48226ee1 shim:900b617d60d4ddb6ea0d75aad4149216e2980c83b1a564ca5845488b48226ee1\n\
shim_29 29 body:0b3f0d99a9e75aa1e4f50d373f6673d5af3441264167061a152e28095eabf37f shim:0b3f0d99a9e75aa1e4f50d373f6673d5af3441264167061a152e28095eabf37f\n\
shim_30 30 body:552388bd5c9348b807948e58a543df4d5c5b8b65fe2ec58a55ca6e4e999fa185 -\n\
shim_renamed_params.a 6 body:0bc8930ce972b811e782acbb2b74855bdde83f67186acb79b4113e7771ed8655 shim:0bc8930ce972b811e782acbb2b74855bdde83f67186acb79b4113e7771ed8655\n\
shim_renamed_params.b 6 body:0bc8930ce972b811e782acbb2b74855bdde83f67186acb79b4113e7771ed8655 shim:0bc8930ce972b811e782acbb2b74855bdde83f67186acb79b4113e7771ed8655\n\
shim_only_own_params_renamed.a 6 body:2acd60af59007ce5fb9917950c276e7a1dab44e9763c00deefd4ce06db63e92c shim:2acd60af59007ce5fb9917950c276e7a1dab44e9763c00deefd4ce06db63e92c\n\
shim_only_own_params_renamed.b 6 body:da2914a4c6b3f8e3d37870a588871f5d385689b4407046cd2c3c01f4bcf324d5 shim:da2914a4c6b3f8e3d37870a588871f5d385689b4407046cd2c3c01f4bcf324d5\n\
shim_different_callee.a 6 body:0bc8930ce972b811e782acbb2b74855bdde83f67186acb79b4113e7771ed8655 shim:0bc8930ce972b811e782acbb2b74855bdde83f67186acb79b4113e7771ed8655\n\
shim_different_callee.b 6 body:7af8d8648cdb2db557896c626d24fb61462358b61170b147e346c8fe6d6cc359 shim:7af8d8648cdb2db557896c626d24fb61462358b61170b147e346c8fe6d6cc359\n\
shim_different_fixed_argument.a 6 body:0bc8930ce972b811e782acbb2b74855bdde83f67186acb79b4113e7771ed8655 shim:0bc8930ce972b811e782acbb2b74855bdde83f67186acb79b4113e7771ed8655\n\
shim_different_fixed_argument.b 6 body:69e11e4c9c80a35eb60bdad19c00b835b4901e49fa58146d74eec9a7b77ec8e8 shim:69e11e4c9c80a35eb60bdad19c00b835b4901e49fa58146d74eec9a7b77ec8e8\n\
shim_parameter_order.a 8 body:de92a8b3421d9329287bf5caf4295656880b02131701549a9a82c9671919ce6a shim:de92a8b3421d9329287bf5caf4295656880b02131701549a9a82c9671919ce6a\n\
shim_parameter_order.b 8 body:de92a8b3421d9329287bf5caf4295656880b02131701549a9a82c9671919ce6a shim:9adc3c2762f4273377031b639ee9ec6ca95b32e0fb63c45955a4082bc96ad7c6\n\
shim_let_body 13 body:f343854017fb0ae1ba5a0333a6c0978b4a2413b793d1f2b958b1fd04f2119d22 -\n\
shim_two_statements 13 body:4a7648548b773432501e0de6bdf3d079c7df0c7271125c6695330f867c4a3154 -\n\
shim_trailing_semicolon 9 body:32a80c15e231526ca8bc5e3f09e9505eadde077f8760f2ee666c3fed1eb0fe7d shim:32a80c15e231526ca8bc5e3f09e9505eadde077f8760f2ee666c3fed1eb0fe7d\n\
shim_mentions_self 10 body:21c93a1e0ab84e9e30f7be7e3612e605f9900018ad58d8893d3d2ea5bb8b97ad -\n\
shim_turbofish_forwarder.a 6 body:0bc8930ce972b811e782acbb2b74855bdde83f67186acb79b4113e7771ed8655 shim:0678d206215e13d2d373f1d330f57aa4e2108fb722c48ff64fe55232cf10959b\n\
shim_turbofish_forwarder.b 6 body:0bc8930ce972b811e782acbb2b74855bdde83f67186acb79b4113e7771ed8655 shim:0678d206215e13d2d373f1d330f57aa4e2108fb722c48ff64fe55232cf10959b\n\
shim_closure_body_with_let.a 16 body:0a90b60003db2dc64791d38a22e7f16c9447ee69c75cba69bc97e8a94f292e5f shim:0bf63e12ac9e89601548f37fb9a703357dcb3de1d4cfd9c8640b9f625559c6e6\n\
shim_closure_body_with_let.b 16 body:0a90b60003db2dc64791d38a22e7f16c9447ee69c75cba69bc97e8a94f292e5f shim:0bf63e12ac9e89601548f37fb9a703357dcb3de1d4cfd9c8640b9f625559c6e6\n\
doc_comment_renamed_local.a 16 body:766fd2419fec79338f4a280fbe020ce4efb5d282849f49009a24deafc0f0da6a -\n\
doc_comment_renamed_local.b 16 body:766fd2419fec79338f4a280fbe020ce4efb5d282849f49009a24deafc0f0da6a -\n\
doc_comment_text_differs.a 16 body:766fd2419fec79338f4a280fbe020ce4efb5d282849f49009a24deafc0f0da6a -\n\
doc_comment_text_differs.b 16 body:ee9292675376f425155ee3dae048a141680bfb7642025b146b90180eb70478fc -\n\
shim_method_turbofish 8 body:fddf4bcd3df9ca1e88883b3cc323cd395ce935a989dedb1228bd9f82b1c63a50 shim:9e32ffb9dd6ec68033ac8091ac4d3505239d3cb50b24b64a7e28ca2dfe1b05b1\n\
shim_turbofish_type_differs.a 6 body:0bc8930ce972b811e782acbb2b74855bdde83f67186acb79b4113e7771ed8655 shim:0678d206215e13d2d373f1d330f57aa4e2108fb722c48ff64fe55232cf10959b\n\
shim_turbofish_type_differs.b 6 body:0bc8930ce972b811e782acbb2b74855bdde83f67186acb79b4113e7771ed8655 shim:9ccd131097a9b704d7f1da65577ee9cb1ad61dd91e12e04b610fb5eb4b687822\n\
shim_turbofish_without_call 10 body:75c451cb055aa50383b3111bf6eeb1f19918a1d49eddbcaab50cb3813edd7ec5 -\n\
shim_without_call 9 body:cd8e7181776f4160bac53f222722152a5a16da9b1306d10dfa4781f46475b180 -\n\
shim_macro_call 9 body:d6a18ab1d813e0a5d4970c09859c8c5103252291b829a445e8056bf03b89b25c shim:d6a18ab1d813e0a5d4970c09859c8c5103252291b829a445e8056bf03b89b25c\n\
macro_arguments.a 9 body:2077d1fcd504fa727d23dc7776e2a40fb6f107ba113613cc3165711c9405c0c2 shim:f05e44a0be6bd1a33c3cf86dc1e1435807337464acb13d880adf2081638d63b4\n\
macro_arguments.b 9 body:2077d1fcd504fa727d23dc7776e2a40fb6f107ba113613cc3165711c9405c0c2 shim:a468d3a9dbc58e5644243f8eacc5493d42d0c800eb58aa2395468d5b43b2d846\n\
macro_inline_capture.a 5 body:4add99615ee9bb66095f85901da80513e9a1fea31d825d6e04715b5b6521625c -\n\
macro_inline_capture.b 5 body:4add99615ee9bb66095f85901da80513e9a1fea31d825d6e04715b5b6521625c -\n\
macro_inline_capture_local.a 13 body:50b5364bdaf2e541cbc8de0ac9f62754f548bf46e94dd865e48797b6e0156530 -\n\
macro_inline_capture_local.b 13 body:50b5364bdaf2e541cbc8de0ac9f62754f548bf46e94dd865e48797b6e0156530 -\n\
macro_unknown_name_kept.a 7 body:2b20a1eef898aae0331c6fc3cf0ff4f221e793cef69d2e08229080ea8ee844cf shim:2b20a1eef898aae0331c6fc3cf0ff4f221e793cef69d2e08229080ea8ee844cf\n\
macro_unknown_name_kept.b 7 body:cc7b50db97a93c132817c268bb5f84a500f4a8d1242cc819c47ab4d4a7a590e7 shim:cc7b50db97a93c132817c268bb5f84a500f4a8d1242cc819c47ab4d4a7a590e7\n\
macro_positional_kept.a 7 body:5f39f17eae70196bb1e2be5d69db7fa56b1ee91f328eb3a417d18f0cc0b25b5b shim:5f39f17eae70196bb1e2be5d69db7fa56b1ee91f328eb3a417d18f0cc0b25b5b\n\
macro_positional_kept.b 7 body:bbde246f89d6ae9030e07018ecea0a0a9502680845d5a98922d644b4ed3b721c shim:bbde246f89d6ae9030e07018ecea0a0a9502680845d5a98922d644b4ed3b721c\n\
macro_escaped_brace_kept.a 9 body:5d0c64128f4c5f2c17111eb08f10ee8f6e8559a680956aac956f3808ff1bc824 shim:5d0c64128f4c5f2c17111eb08f10ee8f6e8559a680956aac956f3808ff1bc824\n\
macro_escaped_brace_kept.b 9 body:e2b9a82f27480b9bc57f3ab63f1d40bf886d2e9069eabbb09b5e05cad5793942 shim:e2b9a82f27480b9bc57f3ab63f1d40bf886d2e9069eabbb09b5e05cad5793942\n\
macro_unicode_escape_is_not_a_placeholder.a 5 body:149fb6d70b94c8d544fd48dd3db56f088a5c22997da0d33c5b9f3098e16f176e -\n\
macro_unicode_escape_is_not_a_placeholder.b 5 body:149fb6d70b94c8d544fd48dd3db56f088a5c22997da0d33c5b9f3098e16f176e -\n\
macro_arguments_above_tier.a 50 body:7f12262ec5c73d6d18a30637587282d939a9fbed08fe7dd7645c6f9209341314 -\n\
macro_arguments_above_tier.b 50 body:7f12262ec5c73d6d18a30637587282d939a9fbed08fe7dd7645c6f9209341314 -\n\
macro_inline_capture_above_tier.a 46 body:e09a3dbbb102073016e4409990aaf0804f99ff55e4549eef725c1d49cf07b453 -\n\
macro_inline_capture_above_tier.b 46 body:e09a3dbbb102073016e4409990aaf0804f99ff55e4549eef725c1d49cf07b453 -\n\
numbering_unused_trailing_parameter.a 6 body:9aad3e89f6d6af9b22a6c79837d59bad03dc979a45acccc923fd42df76140ee3 -\n\
numbering_unused_trailing_parameter.b 6 body:9aad3e89f6d6af9b22a6c79837d59bad03dc979a45acccc923fd42df76140ee3 -\n\
numbering_used_trailing_parameter.a 6 body:9aad3e89f6d6af9b22a6c79837d59bad03dc979a45acccc923fd42df76140ee3 -\n\
numbering_used_trailing_parameter.b 6 body:9aad3e89f6d6af9b22a6c79837d59bad03dc979a45acccc923fd42df76140ee3 -\n\
numbering_parameter_order.a 8 body:31a5488b1691eb8c2aabde440342a4c87efe78e47e89cddd13ef5796eb0977c4 -\n\
numbering_parameter_order.b 8 body:31a5488b1691eb8c2aabde440342a4c87efe78e47e89cddd13ef5796eb0977c4 -\n\
numbering_placeholder_parameter.a 10 body:37ed09ed5fd20c2af5b46edb9107c88f4d02f2d66ec03b0d38f81e8772c4a223 -\n\
numbering_placeholder_parameter.b 10 body:2027574a377e6303405a645e38a1ee229aac2e0612fff97b3aef7652bec64ace -\n\
numbering_placeholder_parameter_above_tier.a 51 body:edb8aec3a585bf743f7989dde6effa7dfc84ba8375407c8e3c0e421354578f9e -\n\
numbering_placeholder_parameter_above_tier.b 51 body:edb8aec3a585bf743f7989dde6effa7dfc84ba8375407c8e3c0e421354578f9e -\n\
numbering_placeholder_first_appearance.a 10 body:cd08fc57ab4da17664535d1855536c2f56e404782056183fff383ed76d78aa58 -\n\
numbering_placeholder_first_appearance.b 10 body:cd08fc57ab4da17664535d1855536c2f56e404782056183fff383ed76d78aa58 -\n\
numbering_placeholder_versus_argument.a 10 body:37ed09ed5fd20c2af5b46edb9107c88f4d02f2d66ec03b0d38f81e8772c4a223 -\n\
numbering_placeholder_versus_argument.b 12 body:f75be63fa5375e955f20df61bbb5e17639d7c59f10ae24451dab32faa7818228 -\n\
lexical_gen_kept.a 9 body:0d185a5e08a042756a17efe75e4d573bdf1dc6cc540694cb740acf6bab8f1eeb -\n\
lexical_gen_kept.b 9 body:5b76112b24b9e8c336a6e299bbb44a0328b4c745bb9ef0d8830d9b83e0944a8a -\n\
lexical_union_kept.a 9 body:21953eda92b7411201c6fd638ce79b2ddddbfa0129d8a53ab69b59b06e96061e -\n\
lexical_union_kept.b 9 body:5b76112b24b9e8c336a6e299bbb44a0328b4c745bb9ef0d8830d9b83e0944a8a -\n\
lexical_raw_kept.a 9 body:1c8f2dac62d70929b57d4e7bc4e1c47abed6334572220477c6f365fd806b7d24 -\n\
lexical_raw_kept.b 9 body:5b76112b24b9e8c336a6e299bbb44a0328b4c745bb9ef0d8830d9b83e0944a8a -\n\
lexical_safe_kept.a 9 body:4bcd02328941de9054517685e91856f69b6a4dfd75e69dd923de387bf59bcb01 -\n\
lexical_safe_kept.b 9 body:5b76112b24b9e8c336a6e299bbb44a0328b4c745bb9ef0d8830d9b83e0944a8a -\n\
lexical_macro_rules_kept.a 9 body:1ab877ecb3fa6f35632db94023632239cc97f4652adecf58e4351f7558403360 -\n\
lexical_macro_rules_kept.b 9 body:5b76112b24b9e8c336a6e299bbb44a0328b4c745bb9ef0d8830d9b83e0944a8a -\n\
lexical_macro_rules_definition.a 18 body:19d5de816f22dba7b2b582370256074ed2bb2457d9ed583486ec5c3f83f962fc -\n\
lexical_macro_rules_definition.b 18 body:19d5de816f22dba7b2b582370256074ed2bb2457d9ed583486ec5c3f83f962fc -\n\
lexical_wildcard_kept.a 8 body:c53227ea2077fa9c9da031fa6f56e925f439c20f080dc550bd8471afe8552e2b -\n\
lexical_wildcard_kept.b 8 body:b90ccc00dcd1aa8151d5c60394f9e6600f9aeb1e0dda358c97d60a4a43ec7d82 -\n\
lexical_nesting_128.a 257 body:577e8c5e2bdea44001e725898204971e75df7decdda85f725ebfc4fd0bc2d5cb -\n\
lexical_nesting_128.b 257 body:577e8c5e2bdea44001e725898204971e75df7decdda85f725ebfc4fd0bc2d5cb -\n\
empty_body 0 - -\n\
";

#[test]
fn the_manifest_is_pinned() {
    assert_eq!(raf_census::manifest(), MANIFEST);
}

#[test]
fn the_pinned_version_is_the_implemented_one() {
    assert_eq!(raf_census::NORMAL_FORM, 1);
}

/// Every way `pinned` fails to be the table of `computed`: a line without a
/// pin, a pin without a line, a moved value and an id that appears twice.
fn pin_problems(computed: &str, pinned: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let mut seen = |text: &str, side: &str| {
        let mut ids = BTreeMap::new();
        for line in text.lines() {
            let (id, value) = line.split_once(' ').expect("a line has an id");
            if ids.insert(id.to_owned(), value.to_owned()).is_some() {
                problems.push(format!("{id}: {side} twice"));
            }
        }
        ids
    };
    let (computed, pinned) = (seen(computed, "computed"), seen(pinned, "pinned"));
    for (id, value) in &computed {
        match pinned.get(id) {
            None => problems.push(format!("{id}: unpinned")),
            Some(pin) if pin != value => problems.push(format!(
                "{id}: moved from `{pin}` to `{value}`; a changed pin requires a normal_form bump"
            )),
            Some(_) => {}
        }
    }
    for id in pinned.keys().filter(|id| !computed.contains_key(*id)) {
        problems.push(format!("{id}: orphan pin"));
    }
    problems
}

#[test]
fn every_fixture_fingerprint_is_pinned() {
    let computed = table(&registry());
    let problems = pin_problems(&computed, TABLE);
    assert!(
        problems.is_empty(),
        "run the goldens example to see the table: {problems:?}"
    );
}

#[test]
fn the_pin_check_names_an_unpinned_orphan_moved_and_duplicated_entry() {
    let pinned = "a 1 body:0000000000000001 -\nb 2 - -\n";
    assert!(
        pin_problems(pinned, pinned).is_empty(),
        "the neighbour passes"
    );
    let one = |computed: &str, pinned: &str| pin_problems(computed, pinned);
    assert_eq!(one("a 1 x -\nc 3 - -\n", "a 1 x -\n"), ["c: unpinned"]);
    assert_eq!(one("a 1 x -\n", "a 1 x -\nd 4 - -\n"), ["d: orphan pin"]);
    let moved = one("a 1 y -\n", "a 1 x -\n");
    assert_eq!(moved.len(), 1);
    assert!(moved[0].starts_with("a: moved") && moved[0].contains("normal_form bump"));
    assert_eq!(one("a 1 x -\n", "a 1 x -\na 1 x -\n"), ["a: pinned twice"]);
    assert_eq!(
        one("a 1 x -\na 1 x -\n", "a 1 x -\n"),
        ["a: computed twice"]
    );
}
