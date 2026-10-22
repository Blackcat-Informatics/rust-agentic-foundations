| Capability | Section | Claim | Line | Tested by |
| --- | --- | --- | --- | --- |
| cap:census-normal-form | constraints | c-10bb63bfd306f0cc | 708 | `raf_census::test_crate::units::written_units_exclude_generated_items_and_initializer_metadata_excludes_attributes` |
| cap:census-normal-form | constraints | c-ba8838079710399a | 714 | `raf_census::test_crate::groups::renamed_locals_inside_macro_arguments_land_in_one_body_group`, `raf_census::test_crate::forms::an_unlexable_slice_is_a_source_error_with_a_valid_neighbour` |
| cap:census-normal-form | constraints | c-596da69bc827d39d | 722 |  |
| cap:census-normal-form | constraints | c-58b3e3b821ca7f18 | 727 | `raf_census::test_crate::units::min_tokens_decides_the_body_fingerprint_at_the_default_and_at_ten`, `raf_census::test_crate::forms::shim_forms_exist_for_six_to_twenty_nine_tokens` |
| cap:census-normal-form | constraints | c-a0a1dfeb5df066a7 | 733 | `raf_census::test_crate::forms::the_digest_is_blake3_of_length_prefixed_tokens`, `raf_census::test_crate::forms::human_abbreviations_do_not_replace_complete_identity` |
| cap:census-normal-form | constraints | c-2dd3b23c236efd9b | 741 | `raf_census::test_crate::forms::a_fingerprint_displays_and_parses_back` |
| cap:census-normal-form | constraints | c-50a26b6e3f06ea86 | 746 |  |
| cap:census-normal-form | constraints | c-6ee9d57d89ede10d | 753 | `raf_census::test_crate::units::loading_a_tree_with_caller_local_state_needs_no_sync_bound`, `raf_census::test_crate::units::a_held_standalone_tower_keeps_its_forms_and_print_across_census_loads` |
| cap:census-normal-form | constraints | c-acddbe54571dcf7d | 762 | `raf_census::test_crate::units::a_module_declared_by_a_library_and_a_binary_is_one_unit`, `raf_census::test_crate::units::a_support_module_shared_by_two_test_binaries_is_one_unit_that_does_not_ship` |
| cap:census-normal-form | constraints | c-0c415e5ce74e1681 | 768 | `raf_census::test_crate::groups::a_forwarder_at_min_tokens_joins_a_shim_group_and_never_a_body_group` |
| cap:census-normal-form | constraints | c-cb57a0e2e44e4e3b | 774 | `raf_census::test_crate::forms::an_unlexable_slice_is_a_source_error_with_a_valid_neighbour` |
| cap:census-normal-form | constraints | c-63aa410472c51651 | 781 | `raf_census::test_crate::units::written_units_exclude_generated_items_and_initializer_metadata_excludes_attributes`, `raf_census::test_crate::units::a_constant_whose_type_names_a_number_does_not_hold_it`, `raf_census::test_crate::units::each_kind_yields_one_unit_with_every_field` |
| cap:census-normal-form | constraints | c-5e912b4c2d7432f9 | 788 | `raf_census::test_crate::units::byte_strings_and_c_strings_are_strings` |
| cap:census-normal-form | constraints | c-8ef8e5f03196d7b3 | 793 |  |
| cap:census-normal-form | constraints | c-48eed51405ffbf48 | 800 | `raf_census::test_crate::units::a_deep_signature_loads_on_a_small_stack_and_beyond_the_budget_is_refused`, `raf_census::test_crate::units::a_deep_constant_type_loads_on_a_small_stack` |
| cap:census-normal-form | constraints | c-86b2278b819487dc | 810 | `raf_census::test_crate::forms::nesting_at_the_budget_lexes_and_one_deeper_is_refused` |
| cap:census-normal-form | constraints | c-46d4dfdf6a63a3ab | 817 | `raf_census::units::tests::collection_worker_tears_down_on_normalization_success_and_error_beside_a_preserved_caller_span`, `raf_census::units::tests::collection_worker_panic_keeps_its_payload_and_tears_down_without_invalidating_caller_spans`, `raf_census::test_crate::units::loading_one_tree_twice_preserves_the_callers_span_storage_and_unit_results` |
| cap:census-normal-form | contract | normalization-pipeline | 298 | `raf_census::test_crate::forms::every_registered_pair_compares_as_registered`, `raf_census::test_crate::golden::every_fixture_fingerprint_is_pinned`, `raf_census::test_crate::golden::the_manifest_is_pinned` |
| cap:census-normal-form | contract | value-erasures | 307 | `raf_census::test_crate::forms::the_value_form_keeps_binary_operators_and_drops_borrows`, `raf_census::test_crate::forms::the_value_adapters_are_the_eleven_of_the_page`, `raf_census::test_crate::forms::every_registered_pair_compares_as_registered` |
| cap:census-normal-form | contract | bound-names-and-item-names | 315 | `raf_census::test_crate::forms::parameters_and_locals_are_numbered_together_by_first_appearance`, `raf_census::test_crate::forms::self_is_not_a_parameter_and_a_field_named_like_one_is_kept`, `raf_census::test_crate::forms::every_registered_pair_compares_as_registered` |
| cap:census-normal-form | contract | literal-selection | 328 | `raf_census::test_crate::forms::the_literal_tier_boundary_is_thirty_value_tokens`, `raf_census::test_crate::forms::below_the_tier_the_final_sequence_keeps_literals_and_above_it_abstracts_them`, `raf_census::test_crate::forms::every_threshold_counts_the_value_form` |
| cap:census-normal-form | contract | shim-eligibility-and-grouping | 337 | `raf_census::test_crate::forms::shim_forms_exist_for_six_to_twenty_nine_tokens`, `raf_census::test_crate::forms::self_callee_query_distinguishes_generic_types_from_calls`, `raf_census::test_crate::groups::a_forwarder_at_min_tokens_joins_a_shim_group_and_never_a_body_group`, `raf_census::test_crate::units::shim_keys_are_absent_for_trait_impl_methods_and_self_forwarders`, `raf_census::test_crate::units::self_type_arguments_keep_shim_keys_beside_generic_self_callees` |
| cap:census-normal-form | contract | minimum-body-size | 352 | `raf_census::test_crate::forms::the_body_print_needs_min_tokens`, `raf_census::test_crate::units::min_tokens_decides_the_body_fingerprint_at_the_default_and_at_ten`, `raf_census::test_crate::groups::bodies_under_min_tokens_never_group` |
| cap:census-normal-form | contract | complete-fingerprint-identities | 359 | `raf_census::test_crate::forms::the_digest_is_blake3_of_length_prefixed_tokens`, `raf_census::test_crate::forms::a_fingerprint_displays_and_parses_back`, `raf_census::test_crate::forms::human_abbreviations_do_not_replace_complete_identity`, `raf_census::group::tests::digest_suffixes_are_distinct_group_keys` |
| cap:census-normal-form | contract | portable-token-encoding | 371 | `raf_census::test_crate::forms::the_digest_is_blake3_of_length_prefixed_tokens` |
| cap:census-normal-form | contract | explicit-normal-form-gate | 380 | `raf_census::version::tests::the_implemented_version_is_one`, `raf_census::version::tests::the_implemented_version_passes`, `raf_census::version::tests::other_versions_are_refused_naming_both_and_the_migration` |
| cap:census-normal-form | contract | load-options | 388 | `raf_census::options::tests::defaults_are_twenty_tokens_and_eight_mebibytes`, `raf_census::options::tests::zero_min_tokens_is_refused_and_one_is_accepted`, `raf_census::test_crate::units::max_file_bytes_caps_a_file_and_the_cap_itself_loads` |
| cap:census-normal-form | contract | actionable-errors-and-panic-ownership | 396 | `raf_census::test_crate::forms::an_unlexable_slice_is_a_source_error_with_a_valid_neighbour`, `raf_census::error::tests::each_variant_renders_its_facts`, `raf_census::units::tests::collection_worker_panic_keeps_its_payload_and_tears_down_without_invalidating_caller_spans` |
| cap:census-normal-form | contract | file-contexts | 408 | `raf_census::test_crate::units::file_context_names_package_path_and_shipping`, `raf_census::test_crate::units::file_context_names_the_target_kinds_that_reach_the_file`, `raf_census::test_crate::units::shipping_is_true_when_any_reach_ships_and_the_shipping_path_is_the_symbol` |
| cap:census-normal-form | contract | source-units | 419 | `raf_census::test_crate::units::each_kind_yields_one_unit_with_every_field`, `raf_census::test_crate::units::a_module_declared_by_a_library_and_a_binary_is_one_unit`, `raf_census::test_crate::units::a_support_module_shared_by_two_test_binaries_is_one_unit_that_does_not_ship`, `raf_census::test_crate::units::shipping_is_true_when_any_reach_ships_and_the_shipping_path_is_the_symbol`, `raf_census::test_crate::units::byte_strings_and_c_strings_are_strings` |
| cap:census-normal-form | contract | deterministic-groups | 433 | `raf_census::test_crate::groups::groups_of_one_are_dropped_and_members_follow_symbol_order`, `raf_census::test_crate::groups::groups_order_by_count_then_tokens_then_fingerprint`, `raf_census::group::tests::members_order_by_symbol_then_file_then_line`, `raf_census::group::tests::digest_suffixes_are_distinct_group_keys` |
| cap:census-normal-form | contract | tables-and-shared-constants | 442 | `raf_census::test_crate::groups::every_form_of_a_hex_table_is_reported_in_both_cases`, `raf_census::test_crate::groups::near_misses_of_a_hex_table_are_not_reported`, `raf_census::test_crate::groups::constants_above_ffff_in_two_packages_are_shared`, `raf_census::test_crate::groups::a_constant_in_one_package_is_not_shared`, `raf_census::test_crate::groups::shared_constants_run_by_value_ascending` |
| cap:census-normal-form | contract | job-classification | 451 | `raf_census::test_crate::groups::a_job_classifies_home_entry_point_variant_and_copies`, `raf_census::test_crate::groups::jobs_match_shim_and_table_fingerprints`, `raf_census::test_crate::groups::a_job_that_matches_nothing_has_an_empty_list` |
| cap:census-normal-form | contract | closed-canonical-reports | 463 | `raf_census::test_crate::reports::both_reports_read_back_what_they_write`, `raf_census::test_crate::reports::two_cfg_alternatives_on_one_line_have_distinct_unit_nodes`, `raf_census::test_crate::reports::the_index_renders_the_classified_matches_of_every_job`, `raf_census::test_crate::reports::a_report_of_another_report_form_is_refused_naming_both`, `raf_census::test_crate::reports::an_index_of_another_normal_form_reads_and_keeps_its_version`, `raf_census::test_crate::reports::unknown_terms_are_not_a_report`, `raf_census::test_crate::reports::missing_repeated_and_mistyped_properties_are_not_a_report`, `raf_census::test_crate::reports::names_that_disagree_with_their_node_are_not_a_report`, `raf_census::test_crate::reports::orphan_units_and_matches_without_a_named_reifier_are_not_a_report`, `raf_census::test_crate::reports::an_edited_or_respaced_text_is_not_a_report`, `raf_census::test_crate::reports::the_writer_refuses_a_value_outside_the_shape` |
| cap:census-normal-form | contract | version-stability | 479 | `raf_census::test_crate::golden::every_fixture_fingerprint_is_pinned`, `raf_census::test_crate::golden::the_manifest_is_pinned` |
| cap:okf-profile | constraints | c-97685f63478a83ad | 797 |  |
| cap:okf-profile | constraints | c-4a564cab3edb0571 | 799 |  |
| cap:okf-profile | constraints | c-0c9ef2a1dd9bf1fb | 802 |  |
| cap:okf-profile | constraints | c-240cafd991cac922 | 805 |  |
| cap:okf-profile | constraints | c-1b83297ba6bddb41 | 806 |  |
| cap:okf-profile | constraints | c-0feec640b058b65a | 807 |  |
| cap:okf-profile | constraints | c-d91d86529f996060 | 808 |  |
| cap:okf-profile | contract | opening | 396 |  |
| cap:okf-profile | contract | parse-framing | 403 | `raf_okf::test_crate::refusals::a_document_without_a_fence_on_line_one_is_refused`, `raf_okf::test_crate::refusals::an_unclosed_fence_is_refused_at_the_opening_fence`, `raf_okf::test_crate::refusals::a_frontmatter_that_is_not_a_block_mapping_is_refused` |
| cap:okf-profile | contract | comment-attachment | 410 | `raf_okf::test_crate::render::comments_attach_to_the_next_key_and_trailing_comments_stay_on_their_line` |
| cap:okf-profile | contract | frontmatter-subset | 418 | `raf_okf::test_crate::refusals::numbers_booleans_and_null_are_read_as_strings`, `raf_okf::test_crate::refusals::anchors_aliases_tags_and_merge_keys_are_refused_naming_the_token`, `raf_okf::test_crate::refusals::a_duplicate_key_is_refused_at_every_level`, `raf_okf::test_crate::refusals::block_scalars_are_refused_naming_the_header` |
| cap:okf-profile | contract | bounded-parsing | 425 |  |
| cap:okf-profile | contract | whole-document-byte-limit | 431 | `raf_okf::test_crate::refusals::a_text_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused` |
| cap:okf-profile | contract | yaml-depth-limit | 436 | `raf_okf::test_crate::refusals::nesting_over_thirty_two_levels_is_refused_before_descent` |
| cap:okf-profile | contract | alias-expansion-refusal | 442 | `raf_okf::test_crate::refusals::a_billion_laughs_document_is_refused_at_the_first_anchor`, `raf_okf::test_crate::refusals::anchors_aliases_tags_and_merge_keys_are_refused_naming_the_token` |
| cap:okf-profile | contract | markdown-depth-limit | 448 | `raf_okf::test_crate::body_structure::block_nesting_of_sixty_four_levels_parses_and_sixty_five_is_refused` |
| cap:okf-profile | contract | link-part-byte-limit | 453 | `raf_okf::test_crate::links::a_destination_or_title_of_exactly_4096_bytes_is_read_and_one_more_is_refused` |
| cap:okf-profile | contract | document-types | 461 | `raf_okf::test_crate::schema::each_type_lists_its_keys_in_the_order_of_the_architecture_page`, `raf_okf::test_crate::schema::only_coverage_and_debt_are_singletons_and_only_the_four_records_are_records` |
| cap:okf-profile | contract | closed-coverage-policy | 469 | `raf_okf::test_crate::records::a_coverage_record_rejects_the_removed_policy_key`, `raf_okf::test_crate::record_writer::a_coverage_record_writes_only_its_closed_policy_independent_keys` |
| cap:okf-profile | contract | ceiling-values-and-classes | 476 | `raf_okf::test_crate::records::a_ceiling_names_a_crate_one_whole_number_of_lines_or_a_percent_and_an_optional_class`, `raf_okf::test_crate::records::a_ceiling_of_another_shape_is_refused_with_its_rule` |
| cap:okf-profile | contract | exception-targets-and-reasons | 494 | `raf_okf::test_crate::records::an_exception_names_one_item_prefix_file_or_directory_and_its_reason`, `raf_okf::test_crate::records::an_exception_of_another_shape_or_without_a_reason_is_refused_with_its_rule` |
| cap:okf-profile | contract | coverage-extension-writer | 511 | `raf_okf::test_crate::record_writer::ceilings_and_exceptions_are_written_after_the_exclusions_and_read_back_equal` |
| cap:okf-profile | contract | typed-debt-rows | 521 | `raf_okf::test_crate::records::a_valid_debt_record_validates_to_its_typed_value_and_an_empty_one_opts_out_visibly`, `raf_okf::test_crate::records::a_row_of_findings_holds_exactly_a_kind_and_a_subject_each_pair_once`, `raf_okf::test_crate::records::a_row_of_lines_holds_exactly_an_item_and_a_whole_number_of_lines_each_item_once`, `raf_okf::test_crate::records::both_lists_of_a_debt_record_are_required`, `raf_okf::test_crate::record_writer::a_debt_record_is_written_sorted_and_read_back_equal` |
| cap:okf-profile | contract | positioned-validation-results | 537 | `raf_okf::test_crate::profile::a_document_with_three_faults_returns_three_violations` |
| cap:okf-profile | contract | body-model | 619 |  |
| cap:okf-profile | contract | deterministic-document-rendering | 693 | `raf_okf::test_crate::render::keys_out_of_order_render_in_table_order_and_then_stay_put`, `raf_okf::test_crate::render::keys_outside_the_table_follow_in_input_order`, `raf_okf::test_crate::render::a_document_of_an_unknown_type_keeps_input_order`, `raf_okf::test_crate::render::tags_render_sorted_and_stay_sorted`, `raf_okf::test_crate::render::a_parsed_document_equals_its_rendering_read_back`, `raf_okf::test_crate::render::equality_compares_keys_as_a_map_with_comments_and_body_bytes` |
| cap:okf-profile | contract | typed-record-construction-and-writing | 708 | `raf_okf::test_crate::record_writer::a_built_evidence_record_renders_to_a_record_that_reads_back_equal`, `raf_okf::test_crate::record_writer::a_built_assessment_and_coverage_record_read_back_equal`, `raf_okf::test_crate::record_writer::the_keys_are_written_in_the_order_of_the_key_table`, `raf_okf::test_crate::record_writer::a_value_the_profile_refuses_is_written_and_refused_when_it_is_read_back`, `raf_okf::test_crate::record_writer::a_value_the_frontmatter_cannot_hold_is_refused_by_the_writer`, `raf_okf::test_crate::record_writer::verdict_and_subject_names_are_the_values_of_their_keys` |
| cap:okf-profile | contract | generated-only-index | 724 | `raf_okf::test_crate::index::a_generated_only_index_is_accepted_with_either_header_form`, `raf_okf::test_crate::index::prose_outside_the_blocks_is_refused_with_its_line`, `raf_okf::test_crate::index::only_the_capability_index_path_is_the_index`, `raf_okf::test_crate::index::document_parse_of_index_text_fails_at_line_one`, `raf_okf::test_crate::index::indented_and_mid_line_markers_are_refused`, `raf_okf::test_crate::index::a_header_with_unknown_tags_or_in_the_wrong_place_is_refused`, `raf_okf::test_crate::index::an_empty_header_tag_is_refused`, `raf_okf::test_crate::index::an_index_with_no_block_or_an_unclosed_block_is_refused`, `raf_okf::test_crate::splice::the_catalogue_of_an_index_is_replaced_through_the_index` |
| cap:rdf-report-profile | constraints | c-89dd853abe0c7f51 | 273 |  |
| cap:rdf-report-profile | constraints | c-c5fcacf0a880b3ed | 275 |  |
| cap:rdf-report-profile | constraints | c-ef165e1400f188f6 | 278 |  |
| cap:rdf-report-profile | constraints | c-063584c0957b6c59 | 281 |  |
| cap:rdf-report-profile | contract | ground-graph-invariants | 62 | `raf_rdf::test_crate::profile::missing_root_dangling_references_duplicates_and_relative_predicates_refuse` |
| cap:rdf-report-profile | contract | term-validation | 73 | `raf_rdf::test_crate::profile::absolute_iris_use_component_validation_without_normalizing_identity`, `raf_rdf::test_crate::profile::literal_language_metadata_and_relative_datatypes_are_refused_in_both_roles`, `raf_rdf::test_crate::profile::both_readers_refuse_language_datatypes_literal_subjects_and_nested_terms` |
| cap:rdf-report-profile | contract | vocabulary-ownership | 84 | `raf_workplan::coverage::report_projection_tests::emitted_application_iris_belong_to_the_documented_registry`, `cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms` |
| cap:rdf-report-profile | contract | snapshot-identity | 91 | `raf_rdf::test_crate::profile::formats_round_trip_the_same_graph_and_snapshot_in_canonical_order`, `raf_rdf::test_crate::profile::independent_parser_pins_preserve_typed_literals_controls_and_triple_terms` |
| cap:rdf-report-profile | contract | equivalent-formats-and-owned-names | 104 | `raf_rdf::test_crate::profile::formats_round_trip_the_same_graph_and_snapshot_in_canonical_order`, `raf_rdf::read::tests::ownership_normalization_checks_the_selected_root_boundary`, `raf_rdf::test_crate::profile::an_external_iri_sharing_a_snapshot_prefix_is_not_local` |
| cap:rdf-report-profile | contract | exact-canonical-acceptance | 115 | `raf_rdf::test_crate::profile::canonical_bytes_include_cr_and_final_lf_and_reject_alternate_escapes`, `raf_rdf::test_crate::profile::malformed_syntax_digest_and_ownership_have_valid_neighbors` |
| cap:rdf-report-profile | contract | typed-observations | 124 | `raf_workplan::test_crate::rdf_reports::findings_preserve_all_location_variants_and_refuse_unknown_kinds`, `raf_workplan::test_crate::rdf_reports::real_coverage_keeps_packages_gaps_exclusions_and_named_source_provenance`, `raf_workplan::test_crate::rdf_reports::unresolved_reexports_keep_declaration_observations_without_claiming_a_definition`, `raf_workplan::test_crate::rdf_reports::unreadable_public_surface_retains_its_structured_failure` |
| cap:rdf-report-profile | contract | shared-finding-append | 138 | `cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms` |
| cap:rdf-report-profile | contract | workplan-projection-registry | 145 | `raf_workplan::coverage::report_projection_tests::emitted_application_iris_belong_to_the_documented_registry`, `raf_workplan::coverage::report_projection_tests::surface_failure_projection_preserves_optional_paths_and_direction` |
| cap:rdf-report-profile | contract | typed-command-registry | 166 | `cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms`, `cargo_workplan::bin_crate::cargo_workplan::result_graph::tests::actual_reach_budget_refusal_projects_a_lines_error_without_partial_rows` |
| cap:repository-contract | constraints | c-b328a0f5c8165618 | 184 | `rust_agentic_foundations::test_crate::docs_contract::refuses_non_regular_files_instead_of_reading_them`, `rust_agentic_foundations::test_crate::docs_contract::rejects_a_tracked_symlink_that_nothing_links_to` |
| cap:repository-contract | constraints | c-86cdd10f36870a79 | 188 | `rust_agentic_foundations::test_crate::repo_gates::issue_refs_flags_a_token_in_a_latin_1_file`, `rust_agentic_foundations::test_crate::repo_gates::issue_refs_treats_only_a_nul_byte_as_binary`, `rust_agentic_foundations::test_crate::repo_gates::issue_refs_skips_a_nul_containing_file_holding_a_token` |
| cap:repository-contract | contract | tracked-inventory | 132 | `rust_agentic_foundations::test_crate::docs_contract::inventory_accepts_a_matching_tree`, `rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_state_change`, `rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_change_record_that_lists_other_capabilities`, `rust_agentic_foundations::test_crate::docs_contract::inventory_rejects_a_document_the_table_does_not_list` |
| cap:repository-contract | contract | document-contract | 138 | `rust_agentic_foundations::test_crate::docs_contract::repository_documents_satisfy_the_profile`, `rust_agentic_foundations::test_crate::docs_contract::repository_bindings_resolve_in_the_source_index`, `rust_agentic_foundations::test_crate::docs_contract::the_checker_judges_links_against_the_tracked_paths_and_symlinks`, `rust_agentic_foundations::test_crate::docs_contract::a_block_that_differs_by_one_line_is_reported`, `rust_agentic_foundations::test_crate::docs_contract::usage_and_examples_blocks_are_the_crate_doctests` |
| cap:repository-contract | contract | licence-and-version-integrity | 144 | `rust_agentic_foundations::test_crate::repo_gates::licenses_accepts_bytes_that_hash_to_the_pin`, `rust_agentic_foundations::test_crate::repo_gates::licenses_rejects_an_altered_mulan_text`, `rust_agentic_foundations::test_crate::repo_gates::licenses_rejects_a_record_without_a_fingerprint_line`, `rust_agentic_foundations::test_crate::repo_gates::citation_accepts_matching_versions`, `rust_agentic_foundations::test_crate::repo_gates::citation_rejects_a_manifest_bump_without_a_matching_citation` |
| cap:repository-contract | contract | manifest-policy | 149 | `rust_agentic_foundations::test_crate::repo_gates::lints_every_workspace_manifest_inherits_the_workspace_lints`, `rust_agentic_foundations::test_crate::repo_gates::lints_rejects_a_weakened_level_in_the_fuzz_tables`, `rust_agentic_foundations::test_crate::repo_gates::deny_rejects_a_missing_or_widened_or_repeated_exception`, `rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_dependency_refusal_covers_aliases_targets_workspace_and_every_dependency_class`, `rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_dependency_scan_refuses_malformed_manifests_instead_of_skipping_them` |
| cap:repository-contract | contract | source-discipline | 156 | `rust_agentic_foundations::test_crate::repo_gates::issue_refs_reports_every_hit_as_file_and_line`, `rust_agentic_foundations::test_crate::repo_gates::issue_refs_spares_near_miss_prose`, `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_rejects_each_source_spelling_in_the_library_and_the_tests`, `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_passes_the_clean_neighbour`, `rust_agentic_foundations::test_crate::repo_gates::time_in_crates_judges_a_crate_added_under_the_crates_directory` |
| cap:repository-contract | contract | current-assessments | 162 | `rust_agentic_foundations::test_crate::self_workplan::repository_workplan_is_clean_and_assessed` |
| cap:source-index | constraints | c-bac07ae380e4d2bb | 2138 |  |
| cap:source-index | constraints | c-1d70e9f4892e05a5 | 2143 |  |
| cap:source-index | constraints | c-7b4198654f540132 | 2148 |  |
| cap:source-index | constraints | c-332daad70a30a7ca | 2152 |  |
| cap:source-index | constraints | c-0823f5551a22065f | 2164 |  |
| cap:source-index | constraints | c-0264ca28848e601f | 2167 |  |
| cap:source-index | constraints | c-69fe1308083f199b | 2168 |  |
| cap:source-index | constraints | c-22eab2502ad332d1 | 2170 |  |
| cap:source-index | constraints | c-1d5689dcbb17f358 | 2179 |  |
| cap:source-index | constraints | c-8f821fd6e5124a5a | 2183 |  |
| cap:source-index | constraints | c-dc07ba4a0ef1a2e5 | 2186 |  |
| cap:source-index | constraints | c-ecec00ed8d66d466 | 2189 |  |
| cap:source-index | constraints | c-59add39b57d93370 | 2193 |  |
| cap:source-index | constraints | c-462762f4d250337b | 2194 |  |
| cap:source-index | constraints | c-0009c0aa5c784d90 | 2195 |  |
| cap:source-index | constraints | c-a5912ee87b6b6814 | 2196 |  |
| cap:source-index | constraints | c-270d3ad6b692efd3 | 2197 |  |
| cap:source-index | constraints | c-ec20ceb9608f870b | 2200 |  |
| cap:source-index | constraints | c-0ae06feb91e20158 | 2201 |  |
| cap:source-index | constraints | c-a143a123f029975c | 2203 |  |
| cap:source-index | constraints | c-5e3dc3de75927231 | 2205 |  |
| cap:source-index | constraints | c-f896647143217993 | 2206 |  |
| cap:source-index | constraints | c-40279a43fad15d61 | 2225 |  |
| cap:source-index | contract | root-discovery | 754 | `raf_source::test_crate::root::member_and_virtual_workspace_directories_resolve_without_sources`, `raf_source::test_crate::root::standalone_excluded_and_nested_packages_have_their_own_roots`, `raf_source::test_crate::root::transitive_path_dependencies_are_members`, `raf_source::test_crate::root::explicit_workspace_requires_a_safe_path_and_agreed_membership`, `raf_source::test_crate::root::malformed_manifests_and_missing_directories_fail_hard`, `raf_source::test_crate::root::manifest_size_bound_is_enforced_beside_the_exact_limit` |
| cap:source-index | contract | tree | 769 |  |
| cap:source-index | contract | file-identity-and-seen-reads | 783 | `raf_source::test_crate::memory_tree::the_identity_of_a_file_is_its_normalised_key_and_refuses_what_read_refuses`, `raf_source::test_crate::memory_tree::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen`, `raf_source::test_crate::disk_tree_containment::the_identity_of_a_file_is_the_device_and_inode_of_its_verified_handle`, `raf_source::test_crate::disk_tree_containment::one_open_gives_the_identity_and_the_text_unless_the_identity_is_seen` |
| cap:source-index | contract | tree-errors | 798 | `raf_source::test_crate::tree_error::every_variant_names_its_path_in_display_and_in_path`, `raf_source::test_crate::tree_error::only_io_has_a_source_and_only_not_found_is_absence` |
| cap:source-index | contract | path-containment | 818 |  |
| cap:source-index | contract | resource-bounds | 927 |  |
| cap:source-index | contract | work-and-memory-bounds | 1053 |  |
| cap:source-index | contract | package-discovery | 1206 |  |
| cap:source-index | contract | mod-tree-following | 1263 |  |
| cap:source-index | contract | cfg-test-tagging | 1306 |  |
| cap:source-index | contract | symbol-index | 1321 |  |
| cap:source-index | contract | byte-and-line-spans | 1331 | `raf_source::test_crate::symbol_index::items_on_one_long_line_of_wide_characters_get_exact_byte_offsets`, `raf_source::test_crate::source_text::a_one_line_item_has_the_half_open_lines_n_to_n_plus_one` |
| cap:source-index | contract | parameter-bindings | 1339 | `raf_source::test_crate::symbol_index::parameter_bindings_are_every_bound_name_in_signature_order_beside_a_function_with_none` |
| cap:source-index | contract | initializer-ranges | 1349 | `raf_source::test_crate::symbol_index::an_initializer_is_the_text_between_the_equals_and_the_semicolon_beside_a_required_constant` |
| cap:source-index | contract | written-bodiless-declarations | 1372 | `raf_source::test_crate::symbol_index::bodiless_functions_and_types_of_attribute_macros_are_indexed_where_they_are_written` |
| cap:source-index | contract | source-identity-and-groups | 1530 | `raf_source::test_crate::symbol_index::items_that_share_a_source_item_are_grouped_by_its_identity` |
| cap:source-index | contract | use-alias-index | 1558 |  |
| cap:source-index | contract | module-relative-resolution | 1835 | `raf_source::test_crate::resolve_paths::a_use_path_counts_only_what_the_reading_module_can_access`, `raf_source::test_crate::resolve_paths::a_use_path_is_read_in_one_name_space_or_in_each_in_turn`, `raf_source::test_crate::resolve_paths::a_module_that_is_not_in_the_workspace_and_a_text_that_is_no_path_are_refused` |
| cap:source-index | contract | resolution-from-every-target | 1840 |  |
| cap:source-index | contract | impl-records | 1842 | `raf_source::test_crate::symbol_index::every_impl_block_is_listed_in_file_order_with_its_span_module_self_type_key_and_members`, `raf_source::test_crate::symbol_index::a_negative_impl_keeps_its_bang_in_the_trait_path_beside_a_positive_one` |
| cap:source-index | contract | reachability-edges | 1847 | `raf_source::test_crate::reachability::a_private_helper_called_by_a_bound_item_is_reached_and_an_unreferenced_one_is_not`, `raf_source::test_crate::reachability::a_method_call_a_field_a_lifetime_and_a_local_name_no_item_and_a_path_prefix_names_its_type` |
| cap:source-index | contract | edition-sensitive-references | 1852 | `raf_source::test_crate::reachability::gen_is_a_name_before_edition_2024_and_a_keyword_from_it`, `raf_source::test_crate::reachability::a_shared_file_reads_keywords_in_each_reached_packages_edition` |
| cap:source-index | contract | self-member-references | 1857 | `raf_source::test_crate::reachability::self_paths_and_methods_called_on_self_reach_the_members_of_that_name`, `raf_source::test_crate::reachability::self_in_a_trait_and_in_its_impl_blocks_reaches_the_members_of_the_trait_and_of_the_block`, `raf_source::test_crate::reachability::a_variant_through_self_names_its_enum_as_a_path_through_the_enum_does` |
| cap:source-index | contract | reachability-result-and-identity-scope | 1862 | `raf_source::test_crate::reachability::the_answer_does_not_depend_on_the_order_of_the_roots_and_is_the_same_each_run`, `raf_source::test_crate::reachability::a_root_past_the_items_or_the_blocks_of_the_workspace_is_refused_beside_one_in_range` |
| cap:source-index | contract | counted-reachability-work | 1867 | `raf_source::reach::tests::the_work_of_a_walk_is_counted_exactly`, `raf_source::reach::tests::the_work_grows_with_the_items_and_the_references_and_not_their_product`, `raf_source::reach::tests::a_path_of_fifty_thousand_segments_is_walked_once_and_not_once_for_each_prefix` |
| cap:source-index | contract | public-surface-visibility | 1872 | `raf_source::test_crate::exported_items::a_pub_function_is_exported_beside_private_and_restricted_ones_which_are_not`, `raf_source::test_crate::exported_items::a_pub_item_of_a_private_module_is_exported_only_through_a_pub_use_and_under_its_rename`, `raf_source::test_crate::exported_items::a_glob_re_export_brings_in_the_pub_items_of_its_module_only` |
| cap:source-index | contract | api-surface | 2015 |  |
| cap:workplan-cli | constraints | c-0eb1378a42fee4b8 | 499 |  |
| cap:workplan-cli | constraints | c-62689747d603683a | 502 |  |
| cap:workplan-cli | constraints | c-21a964430dda7d6b | 504 |  |
| cap:workplan-cli | constraints | c-80e2172de96d4a6a | 509 |  |
| cap:workplan-cli | constraints | c-ad29653d62298ea2 | 514 |  |
| cap:workplan-cli | constraints | c-0b4354336fbd2d78 | 516 |  |
| cap:workplan-cli | constraints | c-4100f19a45803695 | 524 |  |
| cap:workplan-cli | constraints | c-7660ff65bb3afa6d | 528 |  |
| cap:workplan-cli | constraints | c-2efe54f5ca6d77be | 531 |  |
| cap:workplan-cli | constraints | c-c93b532b2df7e835 | 535 |  |
| cap:workplan-cli | contract | opening | 250 |  |
| cap:workplan-cli | contract | root-discovery | 257 |  |
| cap:workplan-cli | contract | values-of-the-record-commands | 310 |  |
| cap:workplan-cli | contract | the-command-of-evidence-record | 320 |  |
| cap:workplan-cli | contract | exit-codes | 335 |  |
| cap:workplan-cli | contract | exit-codes-of-the-record-commands | 343 |  |
| cap:workplan-cli | contract | options | 370 |  |
| cap:workplan-cli | contract | machine-reports | 382 |  |
| cap:workplan-cli | contract | quiet-reports | 402 |  |
| cap:workplan-cli | contract | strict-mode | 410 |  |
| cap:workplan-complete-gate | constraints | c-273d711ebad03ff6 | 541 |  |
| cap:workplan-complete-gate | constraints | c-350b9a4b0ff36862 | 547 |  |
| cap:workplan-complete-gate | constraints | c-6e0031740999275e | 552 |  |
| cap:workplan-complete-gate | constraints | c-d1dad6f8506a0478 | 555 |  |
| cap:workplan-complete-gate | constraints | c-c161c246c47de7d7 | 560 |  |
| cap:workplan-complete-gate | constraints | c-3a9495b8c1730ea6 | 561 |  |
| cap:workplan-complete-gate | constraints | c-79afa74e024ef58f | 564 |  |
| cap:workplan-complete-gate | contract | conditions | 263 |  |
| cap:workplan-complete-gate | contract | order | 303 |  |
| cap:workplan-complete-gate | contract | the-repository-once-the-record-is-removed | 317 |  |
| cap:workplan-complete-gate | contract | obligations-by-effect | 347 |  |
| cap:workplan-complete-gate | contract | complete-policy | 358 |  |
| cap:workplan-complete-gate | contract | completion-conditions | 363 |  |
| cap:workplan-complete-gate | contract | state | 381 |  |
| cap:workplan-complete-gate | contract | invocation | 388 |  |
| cap:workplan-complete-gate | contract | proof-that-the-test-ran-test-bindings | 425 |  |
| cap:workplan-complete-gate | contract | examples | 452 |  |
| cap:workplan-complete-gate | contract | bounds | 459 |  |
| cap:workplan-complete-gate | contract | verify-slug | 474 |  |
| cap:workplan-current-rationale | constraints | c-593380acc97fa0d2 | 140 | `raf_workplan::test_crate::rationale::an_empty_conditions_list_is_reported_and_a_list_item_with_text_passes` |
| cap:workplan-current-rationale | constraints | c-f63a9dcdbfde6097 | 143 | `raf_workplan::test_crate::rationale::headings_match_exactly_at_their_level` |
| cap:workplan-current-rationale | contract | architecture-pages | 49 | `raf_workplan::test_crate::rationale::a_decision_page_linked_from_a_capability_with_its_conditions_passes`, `raf_workplan::test_crate::rationale::an_unlinked_page_is_reported_by_name_and_a_link_to_itself_does_not_count` |
| cap:workplan-current-rationale | contract | conditions-to-revisit | 55 | `raf_workplan::test_crate::rationale::a_decision_page_without_conditions_to_revisit_is_reported_by_name` |
| cap:workplan-current-rationale | contract | decision-pages | 61 | `raf_workplan::test_crate::rationale::an_explanation_decision_without_conditions_is_reported_and_with_them_passes`, `raf_workplan::test_crate::rationale::a_decision_page_without_conditions_to_revisit_is_reported_by_name`, `raf_workplan::test_crate::rationale::an_empty_conditions_list_is_reported_and_a_list_item_with_text_passes` |
| cap:workplan-current-rationale | contract | headings-and-lists | 69 | `raf_workplan::test_crate::rationale::headings_match_exactly_at_their_level`, `raf_workplan::test_crate::rationale::headings_and_list_items_in_fenced_code_or_proposed_blocks_do_not_count`, `raf_workplan::test_crate::rationale::an_empty_conditions_list_is_reported_and_a_list_item_with_text_passes` |
| cap:workplan-current-rationale | contract | conditions-are-prose | 85 | `raf_workplan::test_crate::rationale::an_empty_conditions_list_is_reported_and_a_list_item_with_text_passes` |
| cap:workplan-current-rationale | contract | checks | 90 | `raf_workplan::test_crate::rationale::a_link_from_a_capability_a_change_or_another_page_links_the_page`, `raf_workplan::test_crate::rationale::no_page_is_reported_unlinked_while_a_document_that_may_link_it_does_not_load`, `raf_workplan::test_crate::rationale::a_capability_link_to_a_missing_page_or_heading_fails_the_link_check`, `raf_workplan::test_crate::rationale::the_findings_are_the_same_on_every_run_in_file_and_line_order` |
| cap:workplan-document-model | constraints | c-534176e4bbc87b92 | 637 |  |
| cap:workplan-document-model | constraints | c-5f5b093d83296864 | 641 |  |
| cap:workplan-document-model | constraints | c-bafd423da3329096 | 642 |  |
| cap:workplan-document-model | constraints | c-b87dc9d491b3515c | 645 |  |
| cap:workplan-document-model | constraints | c-a2560bf0d56f300b | 646 |  |
| cap:workplan-document-model | constraints | c-fef2950bd6ee0954 | 649 |  |
| cap:workplan-document-model | contract | capability-documents | 349 |  |
| cap:workplan-document-model | contract | the-four-states | 357 |  |
| cap:workplan-document-model | contract | proposed-blocks | 372 |  |
| cap:workplan-document-model | contract | change-documents | 390 |  |
| cap:workplan-document-model | contract | slug-identities | 407 |  |
| cap:workplan-document-model | contract | loading | 411 |  |
| cap:workplan-document-model | contract | records | 421 |  |
| cap:workplan-document-model | contract | complete-policy | 434 |  |
| cap:workplan-document-model | contract | the-graph | 442 |  |
| cap:workplan-document-model | contract | binding-resolution | 476 |  |
| cap:workplan-document-model | contract | status | 490 |  |
| cap:workplan-document-model | contract | checks | 496 |  |
| cap:workplan-document-model | contract | digests | 548 |  |
| cap:workplan-document-model | contract | writing-records | 562 |  |
| cap:workplan-entity-history | constraints | c-0331b445250dd0d4 | 217 |  |
| cap:workplan-entity-history | constraints | c-151b7713cbb47d43 | 220 |  |
| cap:workplan-entity-history | constraints | c-34fabb77324d09cd | 221 |  |
| cap:workplan-entity-history | constraints | c-08efa6ae6027c051 | 227 |  |
| cap:workplan-entity-history | constraints | c-7cd06aee8135337c | 229 |  |
| cap:workplan-entity-history | constraints | c-16fbfd60feebb7d3 | 233 |  |
| cap:workplan-entity-history | constraints | c-81d98201a1efade0 | 236 |  |
| cap:workplan-entity-history | constraints | c-83631d3c9a2335f1 | 246 |  |
| cap:workplan-entity-history | constraints | c-c22608697fdb670a | 254 |  |
| cap:workplan-entity-history | constraints | c-8e706cfc1dd83acb | 263 |  |
| cap:workplan-entity-history | constraints | c-c7c2a5ce26096aaa | 266 |  |
| cap:workplan-entity-history | contract | the-command | 70 |  |
| cap:workplan-entity-history | contract | document-history | 80 |  |
| cap:workplan-entity-history | contract | binding-history | 87 |  |
| cap:workplan-entity-history | contract | invocation | 125 |  |
| cap:workplan-entity-history | contract | output | 174 |  |
| cap:workplan-evidence-claims | constraints | c-6643c6fd9cf61956 | 191 |  |
| cap:workplan-evidence-claims | constraints | c-a80197c324f9da08 | 193 |  |
| cap:workplan-evidence-claims | constraints | c-e880a4cd587cb09d | 194 |  |
| cap:workplan-evidence-claims | constraints | c-ad715c07c2d8ebcd | 196 |  |
| cap:workplan-evidence-claims | constraints | c-f043aaa8ad21db01 | 198 |  |
| cap:workplan-evidence-claims | constraints | c-66732b789ac8745e | 202 |  |
| cap:workplan-evidence-claims | constraints | c-21008a8da6b853f7 | 205 |  |
| cap:workplan-evidence-claims | constraints | c-61539077042a1b02 | 209 |  |
| cap:workplan-evidence-claims | constraints | c-e017993e6d9a492e | 216 |  |
| cap:workplan-evidence-claims | constraints | c-e898a937fe8594a7 | 222 |  |
| cap:workplan-evidence-claims | constraints | c-b854c3612feb75e3 | 226 |  |
| cap:workplan-evidence-claims | constraints | c-2240bc15eaee2de6 | 229 |  |
| cap:workplan-evidence-claims | contract | evidence | 71 |  |
| cap:workplan-evidence-claims | contract | claims | 113 |  |
| cap:workplan-evidence-claims | contract | the-dependency-view | 127 |  |
| cap:workplan-evidence-claims | contract | invalidation | 137 |  |
| cap:workplan-generated-docs | constraints | c-fe3d15372f715d32 | 260 |  |
| cap:workplan-generated-docs | constraints | c-c39f99f700afd7c8 | 262 |  |
| cap:workplan-generated-docs | constraints | c-59d930a4e54d0a36 | 264 |  |
| cap:workplan-generated-docs | constraints | c-dbcd65f09e97917b | 267 |  |
| cap:workplan-generated-docs | constraints | c-92348930d5749f17 | 275 |  |
| cap:workplan-generated-docs | constraints | c-e50ac913ff6ee4bb | 276 |  |
| cap:workplan-generated-docs | contract | block-syntax | 125 |  |
| cap:workplan-generated-docs | contract | kinds | 134 |  |
| cap:workplan-generated-docs | contract | the-index | 167 |  |
| cap:workplan-generated-docs | contract | sync | 174 |  |
| cap:workplan-generated-docs | contract | staleness | 187 |  |
| cap:workplan-generated-docs | contract | preview | 192 |  |
| cap:workplan-generated-docs | contract | mechanical-versus-explanatory | 202 |  |
| cap:workplan-prose-assessment | constraints | c-9643991eb5d200d9 | 186 |  |
| cap:workplan-prose-assessment | constraints | c-227de316034bae99 | 188 |  |
| cap:workplan-prose-assessment | constraints | c-4937993569d5da7c | 193 |  |
| cap:workplan-prose-assessment | constraints | c-0a6a71bd675e6c15 | 195 |  |
| cap:workplan-prose-assessment | constraints | c-07243650cd64a0a2 | 197 |  |
| cap:workplan-prose-assessment | constraints | c-f6a03bba71c6b329 | 199 |  |
| cap:workplan-prose-assessment | constraints | c-98fa0c7ea3403a95 | 205 |  |
| cap:workplan-prose-assessment | constraints | c-bd53fd0a24f4f02f | 213 |  |
| cap:workplan-prose-assessment | contract | authored-sections | 65 | `raf_workplan::test_crate::assessments::a_capability_that_does_not_load_is_refused_and_a_section_of_generated_blocks_only_does_not_load`, `raf_workplan::test_crate::assessments::a_regenerated_block_leaves_the_assessment_of_its_section_current`, `raf_workplan::test_crate::assessments::each_refusal_writes_nothing_and_its_neighbour_records` |
| cap:workplan-prose-assessment | contract | canonical-assessment-records | 78 | `raf_workplan::test_crate::assessments::an_unchanged_section_is_current_and_a_second_record_is_byte_identical_and_replaces_the_first`, `raf_workplan::test_crate::assessments::a_record_named_otherwise_than_its_subject_is_a_finding_and_not_the_assessment_of_it`, `raf_workplan::test_crate::assessments::an_evidence_record_a_claim_of_the_section_names_is_an_input`, `raf_workplan::test_crate::assessments::each_refusal_writes_nothing_and_its_neighbour_records`, `raf_workplan::test_crate::assessments::a_huge_section_and_ten_thousand_named_symbols_are_recorded_and_judged` |
| cap:workplan-prose-assessment | contract | named-symbol-inputs | 93 | `raf_workplan::test_crate::assessments::a_symbol_the_section_names_is_an_input_and_a_bound_symbol_it_does_not_name_is_not`, `raf_workplan::test_crate::assessments::a_bound_symbol_named_by_its_last_segment_is_an_input_beside_one_named_in_full`, `raf_workplan::test_crate::assessments::a_common_word_that_is_the_last_segment_of_a_binding_does_not_narrow_the_inputs`, `raf_workplan::test_crate::assessments::a_method_named_by_its_type_and_name_is_an_input_and_its_sibling_is_not`, `raf_workplan::test_crate::assessments::a_suffix_two_bindings_end_in_names_both`, `raf_workplan::test_crate::assessments::with_no_bound_symbol_named_every_bound_symbol_is_an_input` |
| cap:workplan-prose-assessment | contract | assessor-context | 111 | `raf_workplan::test_crate::assessments::the_context_is_the_same_for_a_human_and_an_agent_and_holds_what_the_assessor_reads` |
| cap:workplan-prose-assessment | contract | input-currency-and-reconcile-obligations | 120 | `raf_workplan::test_crate::assessments::an_edit_of_the_section_makes_it_stale_naming_the_section_and_an_edit_of_another_does_not`, `raf_workplan::test_crate::assessments::a_symbol_the_section_names_is_an_input_and_a_bound_symbol_it_does_not_name_is_not`, `raf_workplan::test_crate::assessments::an_evidence_record_a_claim_of_the_section_names_is_an_input`, `raf_workplan::test_crate::assessments::a_rejected_assessment_is_current_and_owes_an_acceptance_whoever_gave_it`, `raf_workplan::test_crate::assessments::a_regenerated_block_leaves_the_assessment_of_its_section_current`, `raf_workplan::assessment::tests::a_moved_input_names_its_kind_its_name_and_how_it_moved` |
| cap:workplan-prose-assessment | contract | completion-acceptance | 135 | `raf_workplan::test_crate::complete::an_agent_assessment_satisfies_condition_six_and_a_rejected_one_by_either_never_does` |
| cap:workplan-reconcile | constraints | c-f8cb8a1eb05b4a81 | 496 |  |
| cap:workplan-reconcile | constraints | c-43f1ad34bd372822 | 499 |  |
| cap:workplan-reconcile | constraints | c-72a8d88a83862c90 | 502 |  |
| cap:workplan-reconcile | constraints | c-0ce23cf8dd99bed2 | 507 |  |
| cap:workplan-reconcile | constraints | c-dfc1fc23a443b3c3 | 510 |  |
| cap:workplan-reconcile | constraints | c-9717c3b8c127e74e | 512 |  |
| cap:workplan-reconcile | contract | change-new-slug-capability-cap-name | 249 |  |
| cap:workplan-reconcile | contract | change-impact-slug | 280 |  |
| cap:workplan-reconcile | contract | change-reconcile-slug | 285 |  |
| cap:workplan-reconcile | contract | merge-base | 348 |  |
| cap:workplan-reconcile | contract | invocation | 359 |  |
| cap:workplan-reconcile | contract | contract-effect-none | 415 |  |
| cap:workplan-reconcile | contract | complete-policy | 423 |  |
| cap:workplan-strict | constraints | c-86c2efe28ebd9c53 | 568 |  |
| cap:workplan-strict | constraints | c-4c7ba1e2f9ca0c5e | 572 |  |
| cap:workplan-strict | constraints | c-9584b39f4e727e13 | 575 |  |
| cap:workplan-strict | constraints | c-3ffbb8e2f6af5935 | 580 |  |
| cap:workplan-strict | constraints | c-0c3346ff1e6026a5 | 583 |  |
| cap:workplan-strict | constraints | c-93a07c400a9bce3e | 588 |  |
| cap:workplan-strict | constraints | c-4c23dda116c82b3a | 590 |  |
| cap:workplan-strict | constraints | c-b367ad2a4ee118d0 | 594 |  |
| cap:workplan-strict | constraints | c-988404ad9de51b94 | 596 |  |
| cap:workplan-strict | constraints | c-2ccd01162abcfb57 | 599 |  |
| cap:workplan-strict | constraints | c-0bef61b03de76a8d | 600 | `raf_workplan::test_crate::lines::a_directory_that_is_a_workspace_of_its_own_has_a_row_when_the_source_roots_reach_it_and_none_when_they_do_not` |
| cap:workplan-strict | constraints | c-cedd6208f499775b | 604 | `raf_workplan::test_crate::lines::a_macro_invocation_is_orphan_with_its_range_and_covered_once_a_bound_function_calls_what_it_generates`, `raf_workplan::test_crate::lines::a_cfg_test_module_with_one_bound_and_one_unbound_test_reports_the_unbound_one_and_its_helper_as_test_lines` |
| cap:workplan-strict | constraints | c-b6969c7e85bc2644 | 609 |  |
| cap:workplan-strict | constraints | c-964bdd0a8cd28f7d | 612 |  |
| cap:workplan-strict | constraints | c-5cbb80b7aac4870d | 614 |  |
| cap:workplan-strict | constraints | c-82032a0ba291b68e | 617 |  |
| cap:workplan-strict | constraints | c-f2a23ab762094367 | 619 |  |
| cap:workplan-strict | contract | one-policy | 205 |  |
| cap:workplan-strict | contract | examples-and-the-matrix | 213 |  |
| cap:workplan-strict | contract | claims | 222 |  |
| cap:workplan-strict | contract | explained-items | 281 |  |
| cap:workplan-strict | contract | completion | 296 |  |
| cap:workplan-strict | contract | sections | 353 |  |
| cap:workplan-strict | contract | reasons | 365 |  |
| cap:workplan-strict | contract | untouched-sections | 374 |  |
| cap:workplan-strict | contract | lines | 379 |  |
| cap:workplan-strict | contract | ceilings-and-exceptions | 439 |  |
| cap:workplan-strict | contract | debt | 477 |  |
| cap:workplan-strict | contract | the-kinds | 530 |  |
| cap:workplan-two-way-coverage | constraints | c-1f4a08dd18482a5d | 213 |  |
| cap:workplan-two-way-coverage | constraints | c-03b4823ef7d8df60 | 214 |  |
| cap:workplan-two-way-coverage | constraints | c-d820f6a1186854f1 | 215 |  |
| cap:workplan-two-way-coverage | constraints | c-6a1ef9632b546e6c | 216 |  |
| cap:workplan-two-way-coverage | constraints | c-47593046790e2245 | 218 |  |
| cap:workplan-two-way-coverage | constraints | c-383f8f25c38e4028 | 225 |  |
| cap:workplan-two-way-coverage | constraints | c-d274ceea8df28f20 | 230 |  |
| cap:workplan-two-way-coverage | constraints | c-a941dbe32a0d1358 | 232 |  |
| cap:workplan-two-way-coverage | contract | the-direction-table | 68 |  |
| cap:workplan-two-way-coverage | contract | the-public-surface | 79 |  |
| cap:workplan-two-way-coverage | contract | unresolved-re-exports | 92 |  |
| cap:workplan-two-way-coverage | contract | covered | 106 |  |
| cap:workplan-two-way-coverage | contract | code-to-documentation | 121 |  |
| cap:workplan-two-way-coverage | contract | tests-to-claims | 126 |  |
| cap:workplan-two-way-coverage | contract | the-coverage-record | 129 |  |
| cap:workplan-two-way-coverage | contract | what-check-enforces | 145 |  |
| cap:workplan-two-way-coverage | contract | binding-reached-packages | 162 |  |
| cap:workplan-two-way-coverage | contract | the-account | 171 |  |
