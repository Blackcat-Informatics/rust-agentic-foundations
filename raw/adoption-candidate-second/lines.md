| Crate | Counted | Covered | Excepted | Orphan | Shipping orphan | Test orphan | Ceiling | Debt |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cargo-workplan | 10689 | 10285 | 0 | 404 | 7 | 397 | 0 lines | 0 |
| raf-census | 9381 | 9381 | 0 | 0 | 0 | 0 | 0 lines | 0 |
| raf-okf | 18559 | 18491 | 0 | 68 | 4 | 64 | 0 lines | 0 |
| raf-rdf | 1042 | 665 | 0 | 377 | 0 | 377 | 0 lines | 0 |
| raf-source | 48150 | 47747 | 0 | 403 | 32 | 371 | 0 lines | 0 |
| raf-source-fuzz | 128 | 114 | 0 | 14 | 14 | 0 | 0 lines | 0 |
| raf-workplan | 69396 | 68931 | 0 | 465 | 236 | 229 | 0 lines | 0 |
| rust-agentic-foundations | 3287 | 3287 | 0 | 0 | 0 | 0 | 0 lines | 0 |

| Crate | Module | Counted | Orphan |
| --- | --- | --- | --- |
| cargo-workplan | `cargo_workplan::bin_crate::cargo_workplan::command` | 193 | 2 |
| cargo-workplan | `cargo_workplan::bin_crate::cargo_workplan::exit` | 47 | 5 |
| cargo-workplan | `cargo_workplan::bin_crate::cargo_workplan::result_graph::tests` | 54 | 54 |
| cargo-workplan | `cargo_workplan::test_crate::registry` | 342 | 342 |
| cargo-workplan | `cargo_workplan::test_crate::sync` | 256 | 1 |
| raf-okf | `raf_okf::test_crate::bounds` | 99 | 44 |
| raf-okf | `raf_okf::test_crate::index` | 170 | 3 |
| raf-okf | `raf_okf::test_crate::tested_by` | 235 | 1 |
| raf-okf | `raf_okf::work` | 49 | 4 |
| raf-okf | `raf_okf::work::counting` | 51 | 16 |
| raf-rdf | `raf_rdf::example_crate::report` | 47 | 47 |
| raf-rdf | `raf_rdf::read::tests` | 20 | 20 |
| raf-rdf | `raf_rdf::test_crate::profile` | 310 | 310 |
| raf-source | `raf_source::error::tests` | 138 | 2 |
| raf-source | `raf_source::example_crate::registry_scan` | 576 | 345 |
| raf-source | `raf_source::item` | 399 | 15 |
| raf-source | `raf_source::settle` | 2680 | 11 |
| raf-source | `raf_source::test_crate::corpus_replay::support::program` | 909 | 1 |
| raf-source | `raf_source::test_crate::corpus_replay::support::shapes` | 716 | 6 |
| raf-source | `raf_source::test_crate::exported_items` | 1595 | 1 |
| raf-source | `raf_source::test_crate::workspace_seeds::workspace_input` | 75 | 2 |
| raf-source | `raf_source::work` | 72 | 4 |
| raf-source | `raf_source::work::counting` | 51 | 16 |
| raf-source-fuzz | `raf_source_fuzz::bin_crate::census_normalize` | 32 | 11 |
| raf-source-fuzz | `raf_source_fuzz::bin_crate::census_report_read` | 59 | 1 |
| raf-source-fuzz | `raf_source_fuzz::bin_crate::load_workspace` | 11 | 2 |
| raf-workplan | `raf_workplan::generated` | 459 | 2 |
| raf-workplan | `raf_workplan::report` | 409 | 215 |
| raf-workplan | `raf_workplan::test_crate::assessments::support::window` | 204 | 7 |
| raf-workplan | `raf_workplan::test_crate::bindings` | 488 | 1 |
| raf-workplan | `raf_workplan::test_crate::claim_coverage` | 565 | 1 |
| raf-workplan | `raf_workplan::test_crate::complete` | 3048 | 9 |
| raf-workplan | `raf_workplan::test_crate::documents` | 519 | 2 |
| raf-workplan | `raf_workplan::test_crate::evidence` | 785 | 1 |
| raf-workplan | `raf_workplan::test_crate::graph` | 631 | 1 |
| raf-workplan | `raf_workplan::test_crate::history` | 1419 | 1 |
| raf-workplan | `raf_workplan::test_crate::lifecycle` | 1249 | 3 |
| raf-workplan | `raf_workplan::test_crate::obligations` | 1655 | 12 |
| raf-workplan | `raf_workplan::test_crate::rdf_reports` | 173 | 173 |
| raf-workplan | `raf_workplan::test_crate::untouched::stubs` | 239 | 1 |
| raf-workplan | `raf_workplan::test_crate::verify` | 785 | 1 |
| raf-workplan | `raf_workplan::work` | 164 | 19 |
| raf-workplan | `raf_workplan::work::counting` | 51 | 16 |

| Crate | Item | Kind | File | Orphan lines |
| --- | --- | --- | --- | --- |
| cargo-workplan | `cargo_workplan::bin_crate::cargo_workplan::command::LABEL` | const | crates/cargo-workplan/src/command.rs | 127-128 |
| cargo-workplan | `cargo_workplan::bin_crate::cargo_workplan::exit::&lt;impl From&lt;Exit> for ExitCode>` | impl | crates/cargo-workplan/src/exit.rs | 32-32, 36-36 |
| cargo-workplan | `cargo_workplan::bin_crate::cargo_workplan::exit::&lt;impl From&lt;Exit> for ExitCode>::from` | fn | crates/cargo-workplan/src/exit.rs | 33-35 |
| cargo-workplan | `cargo_workplan::bin_crate::cargo_workplan::result_graph::tests::actual_reach_budget_refusal_projects_a_lines_error_without_partial_rows` | fn | crates/cargo-workplan/src/result_graph/tests.rs | 16-69 |
| cargo-workplan | `cargo_workplan::test_crate::registry::terms` | fn | crates/cargo-workplan/tests/registry.rs | 14-38 |
| cargo-workplan | `cargo_workplan::test_crate::registry::observe` | fn | crates/cargo-workplan/tests/registry.rs | 40-66 |
| cargo-workplan | `cargo_workplan::test_crate::registry::roadmap` | fn | crates/cargo-workplan/tests/registry.rs | 68-135 |
| cargo-workplan | `cargo_workplan::test_crate::registry::lifecycle` | fn | crates/cargo-workplan/tests/registry.rs | 137-181 |
| cargo-workplan | `cargo_workplan::test_crate::registry::evidence_and_assessments` | fn | crates/cargo-workplan/tests/registry.rs | 183-282 |
| cargo-workplan | `cargo_workplan::test_crate::registry::line_coverage_and_debt` | fn | crates/cargo-workplan/tests/registry.rs | 284-324 |
| cargo-workplan | `cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms` | fn | crates/cargo-workplan/tests/registry.rs | 326-361 |
| cargo-workplan | `cargo_workplan::test_crate::sync::MATRIX` | const | crates/cargo-workplan/tests/sync.rs | 13-13 |
| raf-okf | `raf_okf::test_crate::bounds::CEILING` | const | crates/raf-okf/tests/bounds.rs | 12-14 |
| raf-okf | `raf_okf::test_crate::bounds::Counting` | struct | crates/raf-okf/tests/bounds.rs | 16-16 |
| raf-okf | `raf_okf::test_crate::bounds::&lt;impl GlobalAlloc for Counting>` | impl | crates/raf-okf/tests/bounds.rs | 23-27, 38-38, 44-44, 60-60 |
| raf-okf | `raf_okf::test_crate::bounds::Counting::alloc` | fn | crates/raf-okf/tests/bounds.rs | 28-37 |
| raf-okf | `raf_okf::test_crate::bounds::Counting::dealloc` | fn | crates/raf-okf/tests/bounds.rs | 39-43 |
| raf-okf | `raf_okf::test_crate::bounds::Counting::realloc` | fn | crates/raf-okf/tests/bounds.rs | 45-59 |
| raf-okf | `raf_okf::test_crate::bounds::ALLOCATOR` | static | crates/raf-okf/tests/bounds.rs | 62-63 |
| raf-okf | `raf_okf::test_crate::index::SPDX_ID` | const | crates/raf-okf/tests/index.rs | 8-9 |
| raf-okf | `raf_okf::test_crate::index::SPDX_COPY` | const | crates/raf-okf/tests/index.rs | 10-10 |
| raf-okf | `raf_okf::test_crate::tested_by::MARKER` | const | crates/raf-okf/tests/tested_by.rs | 38-38 |
| raf-okf | `raf_okf::work::add` | fn | crates/raf-okf/src/work.rs | 132-135 |
| raf-okf | `raf_okf::work::counting::add` | fn | crates/raf-okf/src/work.rs | 78-93 |
| raf-rdf | `raf_rdf::example_crate::report::main` | fn | crates/raf-rdf/examples/report.rs | 5-51 |
| raf-rdf | `raf_rdf::read::tests::ownership_normalization_checks_the_selected_root_boundary` | fn | crates/raf-rdf/src/read.rs | 255-274 |
| raf-rdf | `raf_rdf::test_crate::profile::absolute` | fn | crates/raf-rdf/tests/profile.rs | 8-10 |
| raf-rdf | `raf_rdf::test_crate::profile::local` | fn | crates/raf-rdf/tests/profile.rs | 11-13 |
| raf-rdf | `raf_rdf::test_crate::profile::triple` | fn | crates/raf-rdf/tests/profile.rs | 14-20 |
| raf-rdf | `raf_rdf::test_crate::profile::example` | fn | crates/raf-rdf/tests/profile.rs | 21-53 |
| raf-rdf | `raf_rdf::test_crate::profile::formats_round_trip_the_same_graph_and_snapshot_in_canonical_order` | fn | crates/raf-rdf/tests/profile.rs | 55-86 |
| raf-rdf | `raf_rdf::test_crate::profile::absolute_iris_use_component_validation_without_normalizing_identity` | fn | crates/raf-rdf/tests/profile.rs | 88-113 |
| raf-rdf | `raf_rdf::test_crate::profile::literal_language_metadata_and_relative_datatypes_are_refused_in_both_roles` | fn | crates/raf-rdf/tests/profile.rs | 115-156 |
| raf-rdf | `raf_rdf::test_crate::profile::missing_root_dangling_references_duplicates_and_relative_predicates_refuse` | fn | crates/raf-rdf/tests/profile.rs | 158-206 |
| raf-rdf | `raf_rdf::test_crate::profile::canonical_bytes_include_cr_and_final_lf_and_reject_alternate_escapes` | fn | crates/raf-rdf/tests/profile.rs | 208-232 |
| raf-rdf | `raf_rdf::test_crate::profile::malformed_syntax_digest_and_ownership_have_valid_neighbors` | fn | crates/raf-rdf/tests/profile.rs | 234-265 |
| raf-rdf | `raf_rdf::test_crate::profile::an_external_iri_sharing_a_snapshot_prefix_is_not_local` | fn | crates/raf-rdf/tests/profile.rs | 267-277 |
| raf-rdf | `raf_rdf::test_crate::profile::both_readers_refuse_language_datatypes_literal_subjects_and_nested_terms` | fn | crates/raf-rdf/tests/profile.rs | 279-309 |
| raf-rdf | `raf_rdf::test_crate::profile::independent_parser_pins_preserve_typed_literals_controls_and_triple_terms` | fn | crates/raf-rdf/tests/profile.rs | 311-326 |
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
| raf-source | `raf_source::item::&lt;impl PartialEq&lt;SharedStr> for str>` | impl | crates/raf-source/src/item.rs | 93-93, 97-97 |
| raf-source | `raf_source::item::&lt;impl PartialEq&lt;SharedStr> for str>::eq` | fn | crates/raf-source/src/item.rs | 94-96 |
| raf-source | `raf_source::item::&lt;impl PartialEq&lt;SharedStr> for &str>` | impl | crates/raf-source/src/item.rs | 99-99, 103-103 |
| raf-source | `raf_source::item::&lt;impl PartialEq&lt;SharedStr> for &str>::eq` | fn | crates/raf-source/src/item.rs | 100-102 |
| raf-source | `raf_source::item::&lt;impl PartialEq&lt;SharedStr> for String>` | impl | crates/raf-source/src/item.rs | 105-105, 109-109 |
| raf-source | `raf_source::item::&lt;impl PartialEq&lt;SharedStr> for String>::eq` | fn | crates/raf-source/src/item.rs | 106-108 |
| raf-source | `raf_source::settle::&lt;impl IntoIterator for &'aBound>` | impl | crates/raf-source/src/settle.rs | 192-192, 198-198, 202-202 |
| raf-source | `raf_source::settle::&lt;impl IntoIterator for &'aBound>::Item` | type | crates/raf-source/src/settle.rs | 193-193 |
| raf-source | `raf_source::settle::&lt;impl IntoIterator for &'aBound>::IntoIter` | type | crates/raf-source/src/settle.rs | 194-197 |
| raf-source | `raf_source::settle::&lt;impl IntoIterator for &'aBound>::into_iter` | fn | crates/raf-source/src/settle.rs | 199-201 |
| raf-source | `raf_source::test_crate::corpus_replay::support::program::&lt;impl Emit for F>` | impl | crates/raf-source/tests/support/program.rs | 228-228 |
| raf-source | `raf_source::test_crate::corpus_replay::support::shapes::PAD` | const | crates/raf-source/tests/support/shapes.rs | 389-392 |
| raf-source | `raf_source::test_crate::corpus_replay::support::shapes::REFS` | const | crates/raf-source/tests/support/shapes.rs | 394-395 |
| raf-source | `raf_source::test_crate::exported_items::OUTSIDE` | const | crates/raf-source/tests/exported_items.rs | 779-779 |
| raf-source | `raf_source::test_crate::workspace_seeds::workspace_input::MANIFEST` | const | fuzz/fuzz_targets/workspace_input.rs | 16-17 |
| raf-source | `raf_source::work::add` | fn | crates/raf-source/src/work.rs | 155-158 |
| raf-source | `raf_source::work::counting::add` | fn | crates/raf-source/src/work.rs | 101-116 |
| raf-source-fuzz | `raf_source_fuzz::bin_crate::census_normalize::SELF_CHECK` | static | fuzz/fuzz_targets/census_normalize.rs | 18-18 |
| raf-source-fuzz | `raf_source_fuzz::bin_crate::census_normalize::load` | fn | fuzz/fuzz_targets/census_normalize.rs | 20-29 |
| raf-source-fuzz | `raf_source_fuzz::bin_crate::census_report_read::SELF_CHECK` | static | fuzz/fuzz_targets/census_report_read.rs | 11-11 |
| raf-source-fuzz | `raf_source_fuzz::bin_crate::load_workspace::SELF_CHECK` | static | fuzz/fuzz_targets/load_workspace.rs | 23-24 |
| raf-workplan | `raf_workplan::generated::END_MARKER` | const | crates/raf-workplan/src/generated.rs | 56-57 |
| raf-workplan | `raf_workplan::report::TERMS` | const | crates/raf-workplan/src/report.rs | 18-55 |
| raf-workplan | `raf_workplan::report::COMMAND_TERMS` | const | crates/raf-workplan/src/report.rs | 57-212 |
| raf-workplan | `raf_workplan::report::findings_graph` | fn | crates/raf-workplan/src/report.rs | 249-262 |
| raf-workplan | `raf_workplan::report::findings` | fn | crates/raf-workplan/src/report.rs | 298-304 |
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
| raf-workplan | `raf_workplan::test_crate::lifecycle::COPYRIGHT_TAG` | const | crates/raf-workplan/tests/lifecycle.rs | 1081-1082 |
| raf-workplan | `raf_workplan::test_crate::lifecycle::LICENCE_TAG` | const | crates/raf-workplan/tests/lifecycle.rs | 1083-1083 |
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
| raf-workplan | `raf_workplan::test_crate::rdf_reports::literals` | fn | crates/raf-workplan/tests/rdf_reports.rs | 11-23 |
| raf-workplan | `raf_workplan::test_crate::rdf_reports::both_formats` | fn | crates/raf-workplan/tests/rdf_reports.rs | 24-30 |
| raf-workplan | `raf_workplan::test_crate::rdf_reports::pinned` | fn | crates/raf-workplan/tests/rdf_reports.rs | 32-43 |
| raf-workplan | `raf_workplan::test_crate::rdf_reports::findings_preserve_all_location_variants_and_refuse_unknown_kinds` | fn | crates/raf-workplan/tests/rdf_reports.rs | 44-92 |
| raf-workplan | `raf_workplan::test_crate::rdf_reports::real_coverage_keeps_packages_gaps_exclusions_and_named_source_provenance` | fn | crates/raf-workplan/tests/rdf_reports.rs | 93-139 |
| raf-workplan | `raf_workplan::test_crate::rdf_reports::unreadable_public_surface_retains_its_structured_failure` | fn | crates/raf-workplan/tests/rdf_reports.rs | 140-155 |
| raf-workplan | `raf_workplan::test_crate::rdf_reports::unresolved_reexports_keep_declaration_observations_without_claiming_a_definition` | fn | crates/raf-workplan/tests/rdf_reports.rs | 157-185 |
| raf-workplan | `raf_workplan::test_crate::untouched::stubs::MERGE_BASE` | const | crates/raf-workplan/tests/untouched.rs | 730-730 |
| raf-workplan | `raf_workplan::test_crate::verify::EXAMPLE` | const | crates/raf-workplan/tests/verify.rs | 604-604 |
| raf-workplan | `raf_workplan::work::&lt;impl Holds&lt;Q> for HashSet&lt;T>>` | impl | crates/raf-workplan/src/work.rs | 91-91, 95-95 |
| raf-workplan | `raf_workplan::work::&lt;impl Holds&lt;Q> for HashSet&lt;T>>::holds` | fn | crates/raf-workplan/src/work.rs | 92-94 |
| raf-workplan | `raf_workplan::work::&lt;impl Holds&lt;Q> for BTreeSet&lt;T>>` | impl | crates/raf-workplan/src/work.rs | 97-97, 101-101 |
| raf-workplan | `raf_workplan::work::&lt;impl Holds&lt;Q> for BTreeSet&lt;T>>::holds` | fn | crates/raf-workplan/src/work.rs | 98-100 |
| raf-workplan | `raf_workplan::work::&lt;impl Holds&lt;Q> for &S>` | impl | crates/raf-workplan/src/work.rs | 103-103, 107-107 |
| raf-workplan | `raf_workplan::work::&lt;impl Holds&lt;Q> for &S>::holds` | fn | crates/raf-workplan/src/work.rs | 104-106 |
| raf-workplan | `raf_workplan::work::add` | fn | crates/raf-workplan/src/work.rs | 257-260 |
| raf-workplan | `raf_workplan::work::counting::add` | fn | crates/raf-workplan/src/work.rs | 203-218 |
