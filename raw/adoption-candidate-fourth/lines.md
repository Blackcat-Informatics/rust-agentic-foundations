| Crate | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cargo-workplan | 10979 | 10978 | 0 | 1 | 0 | 1 | 0 lines | 0 |
| raf-census | 9421 | 9421 | 0 | 0 | 0 | 0 | 0 lines | 0 |
| raf-okf | 18569 | 18565 | 0 | 4 | 0 | 4 | 0 lines | 0 |
| raf-rdf | 1042 | 1042 | 0 | 0 | 0 | 0 | 0 lines | 0 |
| raf-source | 48150 | 47793 | 0 | 357 | 2 | 355 | 0 lines | 0 |
| raf-source-fuzz | 128 | 114 | 0 | 14 | 14 | 0 | 0 lines | 0 |
| raf-workplan | 69466 | 69426 | 0 | 40 | 0 | 40 | 0 lines | 0 |
| rust-agentic-foundations | 3287 | 3287 | 0 | 0 | 0 | 0 | 0 lines | 0 |

| Crate | Module | Counted | Orphan |
| --- | --- | --- | --- |
| cargo-workplan | `cargo_workplan::test_crate::sync` | 256 | 1 |
| raf-okf | `raf_okf::test_crate::index` | 170 | 3 |
| raf-okf | `raf_okf::test_crate::tested_by` | 235 | 1 |
| raf-source | `raf_source::error::tests` | 138 | 2 |
| raf-source | `raf_source::example_crate::registry_scan` | 576 | 345 |
| raf-source | `raf_source::test_crate::corpus_replay::support::program` | 909 | 1 |
| raf-source | `raf_source::test_crate::corpus_replay::support::shapes` | 716 | 6 |
| raf-source | `raf_source::test_crate::exported_items` | 1595 | 1 |
| raf-source | `raf_source::test_crate::workspace_seeds::workspace_input` | 75 | 2 |
| raf-source-fuzz | `raf_source_fuzz::bin_crate::census_normalize` | 32 | 11 |
| raf-source-fuzz | `raf_source_fuzz::bin_crate::census_report_read` | 59 | 1 |
| raf-source-fuzz | `raf_source_fuzz::bin_crate::load_workspace` | 11 | 2 |
| raf-workplan | `raf_workplan::test_crate::assessments::support::window` | 204 | 7 |
| raf-workplan | `raf_workplan::test_crate::bindings` | 488 | 1 |
| raf-workplan | `raf_workplan::test_crate::claim_coverage` | 565 | 1 |
| raf-workplan | `raf_workplan::test_crate::complete` | 3048 | 9 |
| raf-workplan | `raf_workplan::test_crate::documents` | 519 | 2 |
| raf-workplan | `raf_workplan::test_crate::evidence` | 785 | 1 |
| raf-workplan | `raf_workplan::test_crate::graph` | 631 | 1 |
| raf-workplan | `raf_workplan::test_crate::history` | 1419 | 1 |
| raf-workplan | `raf_workplan::test_crate::lifecycle` | 1255 | 3 |
| raf-workplan | `raf_workplan::test_crate::obligations` | 1655 | 12 |
| raf-workplan | `raf_workplan::test_crate::untouched::stubs` | 239 | 1 |
| raf-workplan | `raf_workplan::test_crate::verify` | 785 | 1 |

| Crate | Item | Kind | File | Orphan lines |
| --- | --- | --- | --- | --- |
| cargo-workplan | `cargo_workplan::test_crate::sync::MATRIX` | const | crates/cargo-workplan/tests/sync.rs | 13-13 |
| raf-okf | `raf_okf::test_crate::index::SPDX_ID` | const | crates/raf-okf/tests/index.rs | 8-9 |
| raf-okf | `raf_okf::test_crate::index::SPDX_COPY` | const | crates/raf-okf/tests/index.rs | 10-10 |
| raf-okf | `raf_okf::test_crate::tested_by::MARKER` | const | crates/raf-okf/tests/tested_by.rs | 38-38 |
| raf-source | `raf_source::error::tests::HOSTILE_PATH` | const | crates/raf-source/src/error.rs | 559-560 |
| raf-source | `raf_source::example_crate::registry_scan::RESULT` | const | crates/raf-source/examples/registry_scan.rs | 83-84 |
| raf-source | `raf_source::example_crate::registry_scan::DEFAULT_MAX_MEMORY` | const | crates/raf-source/examples/registry_scan.rs | 86-87 |
| raf-source | `raf_source::example_crate::registry_scan::ALLOCATION_FAILED` | const | crates/raf-source/examples/registry_scan.rs | 89-90 |
| raf-source | `raf_source::example_crate::registry_scan::main` | fn | crates/raf-source/examples/registry_scan.rs | 104-120 |
| raf-source | `raf_source::example_crate::registry_scan::Options` | struct | crates/raf-source/examples/registry_scan.rs | 133-139 |
| raf-source | `raf_source::example_crate::registry_scan::parse_bytes` | fn | crates/raf-source/examples/registry_scan.rs | 152-167 |
| raf-source | `raf_source::example_crate::registry_scan::parse_args` | fn | crates/raf-source/examples/registry_scan.rs | 169-231 |
| raf-source | `raf_source::example_crate::registry_scan::machine_memory` | fn | crates/raf-source/examples/registry_scan.rs | 233-245 |
| raf-source | `raf_source::example_crate::registry_scan::Version` | struct | crates/raf-source/examples/registry_scan.rs | 249-254 |
| raf-source | `raf_source::example_crate::registry_scan::&lt;impl Version>` | impl | crates/raf-source/examples/registry_scan.rs | 256-256, 282-282 |
| raf-source | `raf_source::example_crate::registry_scan::Version::parse` | fn | crates/raf-source/examples/registry_scan.rs | 257-281 |
| raf-source | `raf_source::example_crate::registry_scan::&lt;impl Ord for Version>` | impl | crates/raf-source/examples/registry_scan.rs | 284-284, 310-310 |
| raf-source | `raf_source::example_crate::registry_scan::Version::cmp` | fn | crates/raf-source/examples/registry_scan.rs | 285-309 |
| raf-source | `raf_source::example_crate::registry_scan::&lt;impl PartialOrd for Version>` | impl | crates/raf-source/examples/registry_scan.rs | 312-312, 316-316 |
| raf-source | `raf_source::example_crate::registry_scan::Version::partial_cmp` | fn | crates/raf-source/examples/registry_scan.rs | 313-315 |
| raf-source | `raf_source::example_crate::registry_scan::split_name` | fn | crates/raf-source/examples/registry_scan.rs | 318-329 |
| raf-source | `raf_source::example_crate::registry_scan::Crate` | struct | crates/raf-source/examples/registry_scan.rs | 331-335 |
| raf-source | `raf_source::example_crate::registry_scan::newest` | fn | crates/raf-source/examples/registry_scan.rs | 337-371 |
| raf-source | `raf_source::example_crate::registry_scan::run_child` | fn | crates/raf-source/examples/registry_scan.rs | 542-585 |
| raf-source | `raf_source::example_crate::registry_scan::drain` | fn | crates/raf-source/examples/registry_scan.rs | 587-599 |
| raf-source | `raf_source::example_crate::registry_scan::scan` | fn | crates/raf-source/examples/registry_scan.rs | 648-696 |
| raf-source | `raf_source::test_crate::corpus_replay::support::program::&lt;impl Emit for F>` | impl | crates/raf-source/tests/support/program.rs | 228-228 |
| raf-source | `raf_source::test_crate::corpus_replay::support::shapes::PAD` | const | crates/raf-source/tests/support/shapes.rs | 389-392 |
| raf-source | `raf_source::test_crate::corpus_replay::support::shapes::REFS` | const | crates/raf-source/tests/support/shapes.rs | 394-395 |
| raf-source | `raf_source::test_crate::exported_items::OUTSIDE` | const | crates/raf-source/tests/exported_items.rs | 779-779 |
| raf-source | `raf_source::test_crate::workspace_seeds::workspace_input::MANIFEST` | const | fuzz/fuzz_targets/workspace_input.rs | 16-17 |
| raf-source-fuzz | `raf_source_fuzz::bin_crate::census_normalize::SELF_CHECK` | static | fuzz/fuzz_targets/census_normalize.rs | 18-18 |
| raf-source-fuzz | `raf_source_fuzz::bin_crate::census_normalize::load` | fn | fuzz/fuzz_targets/census_normalize.rs | 20-29 |
| raf-source-fuzz | `raf_source_fuzz::bin_crate::census_report_read::SELF_CHECK` | static | fuzz/fuzz_targets/census_report_read.rs | 11-11 |
| raf-source-fuzz | `raf_source_fuzz::bin_crate::load_workspace::SELF_CHECK` | static | fuzz/fuzz_targets/load_workspace.rs | 23-24 |
| raf-workplan | `raf_workplan::test_crate::assessments::support::window::ACCURATE` | const | crates/raf-workplan/tests/support/window.rs | 243-249 |
| raf-workplan | `raf_workplan::test_crate::bindings::MAKE` | const | crates/raf-workplan/tests/bindings.rs | 415-415 |
| raf-workplan | `raf_workplan::test_crate::claim_coverage::MARKER` | const | crates/raf-workplan/tests/claim_coverage.rs | 19-19 |
| raf-workplan | `raf_workplan::test_crate::complete::EXCLUSIONS` | const | crates/raf-workplan/tests/complete.rs | 264-267 |
| raf-workplan | `raf_workplan::test_crate::complete::MUL` | const | crates/raf-workplan/tests/complete.rs | 2826-2826 |
| raf-workplan | `raf_workplan::test_crate::complete::FRAME` | const | crates/raf-workplan/tests/complete.rs | 2828-2829 |
| raf-workplan | `raf_workplan::test_crate::complete::FRAME_LIB` | const | crates/raf-workplan/tests/complete.rs | 2924-2925 |
| raf-workplan | `raf_workplan::test_crate::documents::SPDX_ID` | const | crates/raf-workplan/tests/documents.rs | 291-292 |
| raf-workplan | `raf_workplan::test_crate::evidence::REVIEW` | const | crates/raf-workplan/tests/evidence.rs | 616-616 |
| raf-workplan | `raf_workplan::test_crate::graph::DIGEST` | const | crates/raf-workplan/tests/graph.rs | 119-119 |
| raf-workplan | `raf_workplan::test_crate::history::P` | const | crates/raf-workplan/tests/history.rs | 35-35 |
| raf-workplan | `raf_workplan::test_crate::lifecycle::COPYRIGHT_TAG` | const | crates/raf-workplan/tests/lifecycle.rs | 1087-1088 |
| raf-workplan | `raf_workplan::test_crate::lifecycle::LICENCE_TAG` | const | crates/raf-workplan/tests/lifecycle.rs | 1089-1089 |
| raf-workplan | `raf_workplan::test_crate::obligations::REASON_NOT_FOUND` | const | crates/raf-workplan/tests/obligations.rs | 92-92 |
| raf-workplan | `raf_workplan::test_crate::obligations::PROPOSAL_EVIDENCE` | const | crates/raf-workplan/tests/obligations.rs | 94-95 |
| raf-workplan | `raf_workplan::test_crate::obligations::BINDING_EVIDENCE` | const | crates/raf-workplan/tests/obligations.rs | 97-97 |
| raf-workplan | `raf_workplan::test_crate::obligations::SYNC_EVIDENCE` | const | crates/raf-workplan/tests/obligations.rs | 98-98 |
| raf-workplan | `raf_workplan::test_crate::obligations::TENSE_EVIDENCE` | const | crates/raf-workplan/tests/obligations.rs | 99-99 |
| raf-workplan | `raf_workplan::test_crate::obligations::CLAIM_EVIDENCE` | const | crates/raf-workplan/tests/obligations.rs | 100-100 |
| raf-workplan | `raf_workplan::test_crate::obligations::STALE_EVIDENCE` | const | crates/raf-workplan/tests/obligations.rs | 101-101 |
| raf-workplan | `raf_workplan::test_crate::obligations::ASSESSED_EVIDENCE` | const | crates/raf-workplan/tests/obligations.rs | 797-797 |
| raf-workplan | `raf_workplan::test_crate::obligations::ASSESS_AGAIN_EVIDENCE` | const | crates/raf-workplan/tests/obligations.rs | 798-798 |
| raf-workplan | `raf_workplan::test_crate::obligations::ACCEPTED_EVIDENCE` | const | crates/raf-workplan/tests/obligations.rs | 799-800 |
| raf-workplan | `raf_workplan::test_crate::untouched::stubs::MERGE_BASE` | const | crates/raf-workplan/tests/untouched.rs | 743-743 |
| raf-workplan | `raf_workplan::test_crate::verify::EXAMPLE` | const | crates/raf-workplan/tests/verify.rs | 604-604 |
