| Capability | Section | Claim | Line | Tested by |
| --- | --- | --- | --- | --- |
| cap:census-normal-form | constraints | c-10bb63bfd306f0cc | 710 | `raf_census::test_crate::units::written_units_exclude_generated_items_and_initializer_metadata_excludes_attributes` |
| cap:census-normal-form | constraints | c-ba8838079710399a | 716 | `raf_census::test_crate::groups::renamed_locals_inside_macro_arguments_land_in_one_body_group`, `raf_census::test_crate::forms::an_unlexable_slice_is_a_source_error_with_a_valid_neighbour` |
| cap:census-normal-form | constraints | c-596da69bc827d39d | 724 | `raf_census::test_crate::forms::doc_comments_are_attribute_tokens_while_ordinary_comments_are_erased`, `raf_census::test_crate::forms::the_names_of_an_attribute_are_kept_beside_a_local_of_the_same_name` |
| cap:census-normal-form | constraints | c-58b3e3b821ca7f18 | 730 | `raf_census::test_crate::units::min_tokens_decides_the_body_fingerprint_at_the_default_and_at_ten`, `raf_census::test_crate::forms::shim_forms_exist_for_six_to_twenty_nine_tokens` |
| cap:census-normal-form | constraints | c-a0a1dfeb5df066a7 | 736 | `raf_census::test_crate::forms::the_digest_is_blake3_of_length_prefixed_tokens`, `raf_census::test_crate::forms::human_abbreviations_do_not_replace_complete_identity` |
| cap:census-normal-form | constraints | c-2dd3b23c236efd9b | 744 | `raf_census::test_crate::forms::a_fingerprint_displays_and_parses_back` |
| cap:census-normal-form | constraints | c-50a26b6e3f06ea86 | 749 |  |
| cap:census-normal-form | constraints | c-6ee9d57d89ede10d | 756 | `raf_census::test_crate::units::loading_a_tree_with_caller_local_state_needs_no_sync_bound`, `raf_census::test_crate::units::a_held_standalone_tower_keeps_its_forms_and_print_across_census_loads` |
| cap:census-normal-form | constraints | c-acddbe54571dcf7d | 765 | `raf_census::test_crate::units::a_module_declared_by_a_library_and_a_binary_is_one_unit`, `raf_census::test_crate::units::a_support_module_shared_by_two_test_binaries_is_one_unit_that_does_not_ship` |
| cap:census-normal-form | constraints | c-0c415e5ce74e1681 | 771 | `raf_census::test_crate::groups::a_forwarder_at_min_tokens_joins_a_shim_group_and_never_a_body_group` |
| cap:census-normal-form | constraints | c-cb57a0e2e44e4e3b | 777 | `raf_census::test_crate::forms::an_unlexable_slice_is_a_source_error_with_a_valid_neighbour` |
| cap:census-normal-form | constraints | c-63aa410472c51651 | 784 | `raf_census::test_crate::units::written_units_exclude_generated_items_and_initializer_metadata_excludes_attributes`, `raf_census::test_crate::units::a_constant_whose_type_names_a_number_does_not_hold_it`, `raf_census::test_crate::units::each_kind_yields_one_unit_with_every_field` |
| cap:census-normal-form | constraints | c-5e912b4c2d7432f9 | 791 | `raf_census::test_crate::units::byte_strings_and_c_strings_are_strings` |
| cap:census-normal-form | constraints | c-8ef8e5f03196d7b3 | 796 | `raf_census::test_crate::forms::local_binding_names_apply_to_the_whole_body_beside_preserved_unbound_items`, `raf_census::test_crate::forms::every_registered_pair_compares_as_registered` |
| cap:census-normal-form | constraints | c-48eed51405ffbf48 | 804 | `raf_census::test_crate::units::a_deep_signature_loads_on_a_small_stack_and_beyond_the_budget_is_refused`, `raf_census::test_crate::units::a_deep_constant_type_loads_on_a_small_stack` |
| cap:census-normal-form | constraints | c-86b2278b819487dc | 814 | `raf_census::test_crate::forms::nesting_at_the_budget_lexes_and_one_deeper_is_refused` |
| cap:census-normal-form | constraints | c-46d4dfdf6a63a3ab | 821 | `raf_census::units::tests::collection_worker_tears_down_on_normalization_success_and_error_beside_a_preserved_caller_span`, `raf_census::units::tests::collection_worker_panic_keeps_its_payload_and_tears_down_without_invalidating_caller_spans`, `raf_census::test_crate::units::loading_one_tree_twice_preserves_the_callers_span_storage_and_unit_results` |
| cap:census-normal-form | contract | normalization-pipeline | 300 | `raf_census::test_crate::forms::every_registered_pair_compares_as_registered`, `raf_census::test_crate::golden::every_fixture_fingerprint_is_pinned`, `raf_census::test_crate::golden::the_manifest_is_pinned` |
| cap:census-normal-form | contract | value-erasures | 309 | `raf_census::test_crate::forms::the_value_form_keeps_binary_operators_and_drops_borrows`, `raf_census::test_crate::forms::the_value_adapters_are_the_eleven_of_the_page`, `raf_census::test_crate::forms::every_registered_pair_compares_as_registered` |
| cap:census-normal-form | contract | bound-names-and-item-names | 317 | `raf_census::test_crate::forms::parameters_and_locals_are_numbered_together_by_first_appearance`, `raf_census::test_crate::forms::self_is_not_a_parameter_and_a_field_named_like_one_is_kept`, `raf_census::test_crate::forms::every_registered_pair_compares_as_registered` |
| cap:census-normal-form | contract | literal-selection | 330 | `raf_census::test_crate::forms::the_literal_tier_boundary_is_thirty_value_tokens`, `raf_census::test_crate::forms::below_the_tier_the_final_sequence_keeps_literals_and_above_it_abstracts_them`, `raf_census::test_crate::forms::every_threshold_counts_the_value_form` |
| cap:census-normal-form | contract | shim-eligibility-and-grouping | 339 | `raf_census::test_crate::forms::shim_forms_exist_for_six_to_twenty_nine_tokens`, `raf_census::test_crate::forms::self_callee_query_distinguishes_generic_types_from_calls`, `raf_census::test_crate::groups::a_forwarder_at_min_tokens_joins_a_shim_group_and_never_a_body_group`, `raf_census::test_crate::units::shim_keys_are_absent_for_trait_impl_methods_and_self_forwarders`, `raf_census::test_crate::units::self_type_arguments_keep_shim_keys_beside_generic_self_callees` |
| cap:census-normal-form | contract | minimum-body-size | 354 | `raf_census::test_crate::forms::the_body_print_needs_min_tokens`, `raf_census::test_crate::units::min_tokens_decides_the_body_fingerprint_at_the_default_and_at_ten`, `raf_census::test_crate::groups::bodies_under_min_tokens_never_group` |
| cap:census-normal-form | contract | complete-fingerprint-identities | 361 | `raf_census::test_crate::forms::the_digest_is_blake3_of_length_prefixed_tokens`, `raf_census::test_crate::forms::a_fingerprint_displays_and_parses_back`, `raf_census::test_crate::forms::human_abbreviations_do_not_replace_complete_identity`, `raf_census::group::tests::digest_suffixes_are_distinct_group_keys` |
| cap:census-normal-form | contract | portable-token-encoding | 373 | `raf_census::test_crate::forms::the_digest_is_blake3_of_length_prefixed_tokens` |
| cap:census-normal-form | contract | explicit-normal-form-gate | 382 | `raf_census::version::tests::the_implemented_version_is_one`, `raf_census::version::tests::the_implemented_version_passes`, `raf_census::version::tests::other_versions_are_refused_naming_both_and_the_migration` |
| cap:census-normal-form | contract | load-options | 390 | `raf_census::options::tests::defaults_are_twenty_tokens_and_eight_mebibytes`, `raf_census::options::tests::zero_min_tokens_is_refused_and_one_is_accepted`, `raf_census::test_crate::units::max_file_bytes_caps_a_file_and_the_cap_itself_loads` |
| cap:census-normal-form | contract | actionable-errors-and-panic-ownership | 398 | `raf_census::test_crate::forms::an_unlexable_slice_is_a_source_error_with_a_valid_neighbour`, `raf_census::error::tests::each_variant_renders_its_facts`, `raf_census::units::tests::collection_worker_panic_keeps_its_payload_and_tears_down_without_invalidating_caller_spans` |
| cap:census-normal-form | contract | file-contexts | 410 | `raf_census::test_crate::units::file_context_names_package_path_and_shipping`, `raf_census::test_crate::units::file_context_names_the_target_kinds_that_reach_the_file`, `raf_census::test_crate::units::shipping_is_true_when_any_reach_ships_and_the_shipping_path_is_the_symbol` |
| cap:census-normal-form | contract | source-units | 421 | `raf_census::test_crate::units::each_kind_yields_one_unit_with_every_field`, `raf_census::test_crate::units::a_module_declared_by_a_library_and_a_binary_is_one_unit`, `raf_census::test_crate::units::a_support_module_shared_by_two_test_binaries_is_one_unit_that_does_not_ship`, `raf_census::test_crate::units::shipping_is_true_when_any_reach_ships_and_the_shipping_path_is_the_symbol`, `raf_census::test_crate::units::byte_strings_and_c_strings_are_strings` |
| cap:census-normal-form | contract | deterministic-groups | 435 | `raf_census::test_crate::groups::groups_of_one_are_dropped_and_members_follow_symbol_order`, `raf_census::test_crate::groups::groups_order_by_count_then_tokens_then_fingerprint`, `raf_census::group::tests::members_order_by_symbol_then_file_then_line`, `raf_census::group::tests::digest_suffixes_are_distinct_group_keys` |
| cap:census-normal-form | contract | tables-and-shared-constants | 444 | `raf_census::test_crate::groups::every_form_of_a_hex_table_is_reported_in_both_cases`, `raf_census::test_crate::groups::near_misses_of_a_hex_table_are_not_reported`, `raf_census::test_crate::groups::constants_above_ffff_in_two_packages_are_shared`, `raf_census::test_crate::groups::a_constant_in_one_package_is_not_shared`, `raf_census::test_crate::groups::shared_constants_run_by_value_ascending` |
| cap:census-normal-form | contract | job-classification | 453 | `raf_census::test_crate::groups::a_job_classifies_home_entry_point_variant_and_copies`, `raf_census::test_crate::groups::jobs_match_shim_and_table_fingerprints`, `raf_census::test_crate::groups::a_job_that_matches_nothing_has_an_empty_list` |
| cap:census-normal-form | contract | closed-canonical-reports | 465 | `raf_census::test_crate::reports::both_reports_read_back_what_they_write`, `raf_census::test_crate::reports::two_cfg_alternatives_on_one_line_have_distinct_unit_nodes`, `raf_census::test_crate::reports::the_index_renders_the_classified_matches_of_every_job`, `raf_census::test_crate::reports::a_report_of_another_report_form_is_refused_naming_both`, `raf_census::test_crate::reports::an_index_of_another_normal_form_reads_and_keeps_its_version`, `raf_census::test_crate::reports::unknown_terms_are_not_a_report`, `raf_census::test_crate::reports::missing_repeated_and_mistyped_properties_are_not_a_report`, `raf_census::test_crate::reports::names_that_disagree_with_their_node_are_not_a_report`, `raf_census::test_crate::reports::orphan_units_and_matches_without_a_named_reifier_are_not_a_report`, `raf_census::test_crate::reports::an_edited_or_respaced_text_is_not_a_report`, `raf_census::test_crate::reports::the_writer_refuses_a_value_outside_the_shape` |
| cap:census-normal-form | contract | version-stability | 481 | `raf_census::test_crate::golden::every_fixture_fingerprint_is_pinned`, `raf_census::test_crate::golden::the_manifest_is_pinned` |
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
| cap:rdf-report-profile | constraints | c-89dd853abe0c7f51 | 275 | `raf_rdf::test_crate::profile::abbreviations_blank_nodes_and_quads_are_outside_the_closed_profile` |
| cap:rdf-report-profile | constraints | c-c5fcacf0a880b3ed | 278 |  |
| cap:rdf-report-profile | constraints | c-ef165e1400f188f6 | 281 |  |
| cap:rdf-report-profile | constraints | c-063584c0957b6c59 | 284 | `raf_rdf::test_crate::profile::ground_validation_does_not_apply_domain_shapes_or_datatype_value_spaces` |
| cap:rdf-report-profile | contract | ground-graph-invariants | 64 | `raf_rdf::test_crate::profile::missing_root_dangling_references_duplicates_and_relative_predicates_refuse` |
| cap:rdf-report-profile | contract | term-validation | 75 | `raf_rdf::test_crate::profile::absolute_iris_use_component_validation_without_normalizing_identity`, `raf_rdf::test_crate::profile::literal_language_metadata_and_relative_datatypes_are_refused_in_both_roles`, `raf_rdf::test_crate::profile::both_readers_refuse_language_datatypes_literal_subjects_and_nested_terms` |
| cap:rdf-report-profile | contract | vocabulary-ownership | 86 | `raf_workplan::coverage::report_projection_tests::emitted_application_iris_belong_to_the_documented_registry`, `cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms` |
| cap:rdf-report-profile | contract | snapshot-identity | 93 | `raf_rdf::test_crate::profile::formats_round_trip_the_same_graph_and_snapshot_in_canonical_order`, `raf_rdf::test_crate::profile::independent_parser_pins_preserve_typed_literals_controls_and_triple_terms` |
| cap:rdf-report-profile | contract | equivalent-formats-and-owned-names | 106 | `raf_rdf::test_crate::profile::formats_round_trip_the_same_graph_and_snapshot_in_canonical_order`, `raf_rdf::read::tests::ownership_normalization_checks_the_selected_root_boundary`, `raf_rdf::test_crate::profile::an_external_iri_sharing_a_snapshot_prefix_is_not_local` |
| cap:rdf-report-profile | contract | exact-canonical-acceptance | 117 | `raf_rdf::test_crate::profile::canonical_bytes_include_cr_and_final_lf_and_reject_alternate_escapes`, `raf_rdf::test_crate::profile::malformed_syntax_digest_and_ownership_have_valid_neighbors` |
| cap:rdf-report-profile | contract | typed-observations | 126 | `raf_workplan::test_crate::rdf_reports::findings_preserve_all_location_variants_and_refuse_unknown_kinds`, `raf_workplan::test_crate::rdf_reports::real_coverage_keeps_packages_gaps_exclusions_and_named_source_provenance`, `raf_workplan::test_crate::rdf_reports::unresolved_reexports_keep_declaration_observations_without_claiming_a_definition`, `raf_workplan::test_crate::rdf_reports::unreadable_public_surface_retains_its_structured_failure` |
| cap:rdf-report-profile | contract | shared-finding-append | 140 | `cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms` |
| cap:rdf-report-profile | contract | workplan-projection-registry | 147 | `raf_workplan::coverage::report_projection_tests::emitted_application_iris_belong_to_the_documented_registry`, `raf_workplan::coverage::report_projection_tests::surface_failure_projection_preserves_optional_paths_and_direction` |
| cap:rdf-report-profile | contract | typed-command-registry | 168 | `cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms`, `cargo_workplan::bin_crate::cargo_workplan::result_graph::tests::actual_reach_budget_refusal_projects_a_lines_error_without_partial_rows` |
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
| cap:workplan-cli | contract | debt-record | 410 |  |
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
| cap:workplan-document-model | constraints | c-534176e4bbc87b92 | 676 |  |
| cap:workplan-document-model | constraints | c-5f5b093d83296864 | 680 |  |
| cap:workplan-document-model | constraints | c-bafd423da3329096 | 681 |  |
| cap:workplan-document-model | constraints | c-b87dc9d491b3515c | 684 |  |
| cap:workplan-document-model | constraints | c-a2560bf0d56f300b | 685 |  |
| cap:workplan-document-model | constraints | c-fef2950bd6ee0954 | 688 |  |
| cap:workplan-document-model | contract | capability-identity-and-shape | 349 | `raf_workplan::test_crate::documents::the_model_holds_typed_values_and_bodies`, `raf_workplan::test_crate::references::a_slug_id_that_differs_from_the_file_name_is_refused`, `raf_workplan::test_crate::references::a_slug_that_is_not_kebab_case_is_refused`, `raf_workplan::test_crate::references::a_matching_kebab_slug_is_accepted` |
| cap:workplan-document-model | contract | capability-lifetime-identity | 358 |  |
| cap:workplan-document-model | contract | state-and-open-record-consistency | 364 | `raf_workplan::test_crate::states::accepts_each_state_with_matching_content`, `raf_workplan::test_crate::states::rejects_each_non_delivered_state_without_an_open_record`, `raf_workplan::test_crate::states::rejects_delivered_with_open_change_record` |
| cap:workplan-document-model | contract | state-transition-policy | 377 |  |
| cap:workplan-document-model | contract | proposal-ownership-and-placement | 384 | `raf_workplan::test_crate::states::rejects_orphan_proposed_marker`, `raf_workplan::test_crate::states::rejects_proposed_section_missing_from_affects`, `raf_workplan::test_crate::states::rejects_proposed_change_that_does_not_list_the_capability`, `raf_workplan::test_crate::states::rejects_considered_without_proposed_block`, `raf_workplan::test_crate::states::rejects_delivered_with_proposed_block`, `raf_workplan::test_crate::states::rejects_designed_with_proposed_block`, `raf_workplan::test_crate::documents::rejects_proposed_marker_in_architecture`, `raf_workplan::test_crate::documents::rejects_proposed_marker_in_change` |
| cap:workplan-document-model | contract | change-documents | 405 |  |
| cap:workplan-document-model | contract | slug-references | 422 | `raf_workplan::test_crate::references::a_slug_id_that_differs_from_the_file_name_is_refused`, `raf_workplan::test_crate::references::a_slug_that_is_not_kebab_case_is_refused`, `raf_workplan::test_crate::references::a_matching_kebab_slug_is_accepted` |
| cap:workplan-document-model | contract | typed-loading-and-invalid-document-exclusion | 429 | `raf_workplan::test_crate::documents::accepts_valid_fixture`, `raf_workplan::test_crate::documents::the_model_holds_typed_values_and_bodies`, `raf_workplan::test_crate::documents::a_broken_document_is_a_finding_and_not_in_the_model`, `raf_workplan::test_crate::documents::an_oversize_file_is_reported_by_name_and_not_skipped` |
| cap:workplan-document-model | contract | finding-vocabulary-and-diagnostics | 440 | `raf_workplan::test_crate::kinds::kinds_are_unique_kebab_case_names`, `raf_workplan::test_crate::kinds::every_rule_and_every_reader_error_has_its_own_kind`, `raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind`, `raf_workplan::test_crate::kinds::every_finding_names_a_fix_and_a_message`, `raf_workplan::test_crate::documents::a_finding_renders_with_its_place_and_its_fix` |
| cap:workplan-document-model | contract | typed-records-and-absent-directories | 447 | `raf_workplan::test_crate::records::a_repository_without_record_directories_has_no_records_and_no_new_findings`, `raf_workplan::test_crate::records::each_record_type_loads_from_its_directory_into_the_model`, `raf_workplan::test_crate::records::work_coverage_holds_only_the_coverage_record`, `raf_workplan::test_crate::debt::a_file_other_than_the_one_record_is_a_finding` |
| cap:workplan-document-model | contract | complete-policy-and-finding-order | 463 | `raf_workplan::test_crate::strict::content_and_binding_findings_follow_load_findings`, `raf_workplan::test_crate::strict::the_removed_policy_key_is_rejected_even_for_its_former_values`, `raf_workplan::test_crate::strict::the_default_gate_judges_the_complete_policy_in_its_opened_workplan` |
| cap:workplan-document-model | contract | the-graph | 474 |  |
| cap:workplan-document-model | contract | binding-roles-and-resolution | 508 | `raf_workplan::test_crate::bindings::a_binding_through_a_re_export_resolves_to_its_definition`, `raf_workplan::test_crate::bindings::the_role_is_judged_on_the_definition_a_re_export_leads_to`, `raf_workplan::test_crate::bindings::a_tests_binding_must_name_a_function_marked_test`, `raf_workplan::test_crate::bindings::a_code_binding_must_name_a_shipping_item`, `raf_workplan::test_crate::bindings::an_examples_binding_is_an_example_target_or_a_documented_library_item`, `raf_workplan::test_crate::bindings::a_binding_to_a_missing_path_is_reported_with_its_capability`, `raf_workplan::test_crate::bindings::a_path_two_globs_both_bring_in_is_ambiguous_and_names_both_definitions`, `raf_workplan::test_crate::bindings::a_binding_to_a_macro_generated_function_is_not_found` |
| cap:workplan-document-model | contract | roadmap-grouping-and-binding-gaps | 525 | `raf_workplan::test_crate::roadmap::an_implementing_capability_lists_its_gaps_exactly`, `raf_workplan::test_crate::roadmap::only_an_implementing_capability_has_gaps`, `raf_workplan::test_crate::roadmap::a_resolving_binding_of_the_wrong_role_is_not_a_gap`, `raf_workplan::test_crate::roadmap::open_changes_are_the_changes_that_list_the_capability`, `raf_workplan::test_crate::roadmap::the_roadmap_does_not_depend_on_insertion_order_and_repeats`, `raf_workplan::test_crate::roadmap::slug_order_is_not_file_name_order` |
| cap:workplan-document-model | contract | checks | 534 |  |
| cap:workplan-document-model | contract | canonical-input-digests | 584 | `raf_workplan::test_crate::digest::each_kind_hashes_its_documented_encoding`, `raf_workplan::test_crate::digest::a_symbol_and_a_record_with_equal_bytes_differ`, `raf_workplan::test_crate::digest::crlf_and_lf_give_the_same_digest`, `raf_workplan::test_crate::digest::a_regenerated_block_leaves_the_section_digest_and_an_edit_moves_it`, `raf_workplan::test_crate::digest::every_definition_of_a_path_counts_once_in_file_order`, `raf_workplan::test_crate::digest::a_module_stands_for_its_files_and_the_modules_below_it_and_a_sibling_does_not_move_it`, `raf_workplan::test_crate::digest::a_digest_is_written_and_read_as_sixty_four_lowercase_hex_digits` |
| cap:workplan-document-model | contract | writing-records | 601 |  |
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
| cap:workplan-generated-docs | constraints | c-fe3d15372f715d32 | 300 | `raf_workplan::test_crate::generated::a_matrix_and_a_relationship_table_sit_in_a_section_of_a_capability_document`, `raf_workplan::test_crate::generated::a_generated_block_does_not_make_a_section_non_empty` |
| cap:workplan-generated-docs | constraints | c-c39f99f700afd7c8 | 303 | `raf_workplan::test_crate::generated::an_edited_removed_or_added_matrix_row_is_stale_and_sync_repairs_it`, `raf_workplan::test_crate::generated::an_edited_removed_or_added_relationship_row_is_stale_and_sync_repairs_it`, `raf_workplan::test_crate::generated::an_edited_removed_or_added_catalogue_row_is_stale_and_sync_repairs_it` |
| cap:workplan-generated-docs | constraints | c-59d930a4e54d0a36 | 306 | `raf_workplan::test_crate::sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal`, `raf_workplan::test_crate::sync::write_puts_every_edit_on_disk_and_a_second_run_writes_nothing` |
| cap:workplan-generated-docs | constraints | c-ff13dea40f3cea9c | 310 | `raf_workplan::test_crate::sync::write_refuses_a_target_that_is_not_an_existing_regular_file_or_the_new_index`, `raf_workplan::test_crate::sync::write_refuses_a_target_behind_a_directory_link_inside_the_root` |
| cap:workplan-generated-docs | constraints | c-977c8ef2778e523f | 317 | `raf_workplan::test_crate::sync::write_changes_no_target_when_a_later_file_cannot_be_written` |
| cap:workplan-generated-docs | constraints | c-185a366823b57d53 | 321 | `raf_workplan::sync::tests::a_late_rename_failure_preserves_published_files_and_discards_the_tail` |
| cap:workplan-generated-docs | constraints | c-92348930d5749f17 | 326 | `raf_workplan::test_crate::preview::a_disk_fixture_is_byte_identical_afterwards` |
| cap:workplan-generated-docs | constraints | c-e50ac913ff6ee4bb | 328 | `raf_workplan::test_crate::preview::a_fenced_code_block_in_a_block_of_the_change_is_refused_with_its_line`, `raf_workplan::test_crate::preview::a_continuation_that_is_not_indented_as_the_marker_is_refused_with_its_line`, `raf_workplan::test_crate::preview::a_block_in_a_list_item_keeps_its_indentation` |
| cap:workplan-generated-docs | contract | generated-block-syntax | 126 | `raf_workplan::test_crate::generated::a_marker_fixture_for_each_error_nested_unclosed_orphan_end_and_unknown_kind` |
| cap:workplan-generated-docs | contract | generated-kinds | 138 |  |
| cap:workplan-generated-docs | contract | verification-matrix-rows | 142 | `raf_workplan::test_crate::generated::a_verification_matrix_lists_each_test_and_example_by_path_with_its_sections`, `raf_workplan::test_crate::generated::rows_are_sorted_by_path_whatever_the_order_of_the_lists`, `raf_workplan::test_crate::generated::a_tested_by_marker_substantiates_its_section_and_an_unmarked_binding_keeps_its_row` |
| cap:workplan-generated-docs | contract | relationship-table-rows | 158 | `raf_workplan::test_crate::generated::a_relationship_table_lists_each_relation_sorted_by_slug_with_links`, `raf_workplan::test_crate::generated::a_capability_with_no_relation_has_a_table_of_the_header_only` |
| cap:workplan-generated-docs | contract | catalogue-states-and-gaps | 167 | `raf_workplan::test_crate::generated::the_catalogue_is_a_table_per_state_in_order_with_gaps_for_implementing_only`, `raf_workplan::test_crate::generated::a_gap_names_an_unresolved_binding_and_an_empty_list`, `raf_workplan::test_crate::generated::groups_with_no_capability_are_omitted_and_each_group_is_sorted_by_slug` |
| cap:workplan-generated-docs | contract | placement-and-escaped-cells | 177 | `raf_workplan::test_crate::generated::a_matrix_and_a_relationship_table_sit_in_a_section_of_a_capability_document`, `raf_workplan::test_crate::generated::a_catalogue_block_is_valid_only_in_the_capability_index`, `raf_workplan::test_crate::generated::a_matrix_or_a_relationship_table_in_the_index_is_misplaced`, `raf_workplan::test_crate::generated::blocks_in_a_change_record_or_an_architecture_page_are_misplaced_and_never_rewritten`, `raf_workplan::test_crate::generated::cell_text_is_escaped_the_same_way_every_time` |
| cap:workplan-generated-docs | contract | index-creation-and-refusal | 187 | `raf_workplan::test_crate::generated::a_missing_index_is_a_finding_and_sync_creates_it_with_the_spdx_lines`, `raf_workplan::test_crate::generated::the_spdx_text_of_a_new_index_is_required_and_must_fit_one_comment_line`, `raf_workplan::test_crate::generated::an_index_with_text_outside_its_blocks_is_rejected_whatever_else_it_holds` |
| cap:workplan-generated-docs | contract | pure-sync-report-and-stable-output | 197 | `raf_workplan::test_crate::sync::the_report_is_a_pure_function_of_the_documents_and_is_sorted_by_path`, `raf_workplan::test_crate::sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal`, `raf_workplan::test_crate::generated::text_outside_a_block_is_never_changed_and_a_file_without_blocks_is_untouched`, `raf_workplan::test_crate::generated::the_frontmatter_of_a_rewritten_document_is_kept_byte_for_byte`, `raf_workplan::test_crate::generated::a_crlf_document_stays_all_crlf_and_an_lf_document_all_lf` |
| cap:workplan-generated-docs | contract | unloaded-documents-stop-synchronization | 211 | `raf_workplan::test_crate::sync::a_document_that_does_not_load_stops_the_sync_since_the_tables_would_miss_it` |
| cap:workplan-generated-docs | contract | generated-staleness-and-repair | 217 | `raf_workplan::test_crate::generated::the_stale_finding_points_at_the_opening_marker_of_the_block`, `raf_workplan::test_crate::sync::generated_checks_are_clean_exactly_when_sync_has_nothing_to_write`, `raf_workplan::test_crate::sync::check_and_sync_agree_after_each_repair` |
| cap:workplan-generated-docs | contract | preview-selection-and-preserved-text | 226 | `raf_workplan::test_crate::preview::a_change_with_two_blocks_in_a_capability_reads_reconciled_and_other_blocks_stay`, `raf_workplan::test_crate::preview::another_change_reconciles_only_its_own_blocks`, `raf_workplan::test_crate::preview::the_slug_is_taken_with_or_without_the_prefix`, `raf_workplan::test_crate::preview::preview_is_deterministic`, `raf_workplan::test_crate::preview::a_crlf_document_keeps_its_line_endings`, `raf_workplan::test_crate::preview::a_block_in_a_list_item_keeps_its_indentation`, `raf_workplan::test_crate::preview::a_tab_after_the_quote_marker_is_kept_as_written`, `raf_workplan::test_crate::preview::a_nested_quote_loses_one_level_only`, `raf_workplan::test_crate::preview::an_unknown_slug_names_it_and_lists_the_open_changes`, `raf_workplan::test_crate::preview::a_change_record_that_did_not_load_says_why`, `raf_workplan::test_crate::preview::a_listed_capability_that_did_not_load_is_named_never_skipped` |
| cap:workplan-generated-docs | contract | authored-text-preservation | 239 | `raf_workplan::test_crate::generated::text_outside_a_block_is_never_changed_and_a_file_without_blocks_is_untouched` |
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
| cap:workplan-strict | constraints | c-86c2efe28ebd9c53 | 567 |  |
| cap:workplan-strict | constraints | c-4c7ba1e2f9ca0c5e | 571 |  |
| cap:workplan-strict | constraints | c-9584b39f4e727e13 | 574 |  |
| cap:workplan-strict | constraints | c-3ffbb8e2f6af5935 | 579 |  |
| cap:workplan-strict | constraints | c-0c3346ff1e6026a5 | 582 |  |
| cap:workplan-strict | constraints | c-93a07c400a9bce3e | 587 |  |
| cap:workplan-strict | constraints | c-e335e52278b6923e | 589 |  |
| cap:workplan-strict | constraints | c-b367ad2a4ee118d0 | 593 |  |
| cap:workplan-strict | constraints | c-b2c848cd083954a7 | 595 |  |
| cap:workplan-strict | constraints | c-2ccd01162abcfb57 | 598 |  |
| cap:workplan-strict | constraints | c-0bef61b03de76a8d | 599 | `raf_workplan::test_crate::lines::a_directory_that_is_a_workspace_of_its_own_has_a_row_when_the_source_roots_reach_it_and_none_when_they_do_not` |
| cap:workplan-strict | constraints | c-cedd6208f499775b | 603 | `raf_workplan::test_crate::lines::a_macro_invocation_is_orphan_with_its_range_and_covered_once_a_bound_function_calls_what_it_generates`, `raf_workplan::test_crate::lines::a_cfg_test_module_with_one_bound_and_one_unbound_test_reports_the_unbound_one_and_its_helper_as_test_lines` |
| cap:workplan-strict | constraints | c-b6969c7e85bc2644 | 608 |  |
| cap:workplan-strict | constraints | c-964bdd0a8cd28f7d | 611 |  |
| cap:workplan-strict | constraints | c-5cbb80b7aac4870d | 613 |  |
| cap:workplan-strict | constraints | c-82032a0ba291b68e | 616 |  |
| cap:workplan-strict | constraints | c-f2a23ab762094367 | 618 |  |
| cap:workplan-strict | contract | one-policy | 205 |  |
| cap:workplan-strict | contract | examples-and-the-matrix | 213 |  |
| cap:workplan-strict | contract | claims | 222 |  |
| cap:workplan-strict | contract | explained-items | 281 |  |
| cap:workplan-strict | contract | completion | 296 |  |
| cap:workplan-strict | contract | sections | 352 |  |
| cap:workplan-strict | contract | reasons | 364 |  |
| cap:workplan-strict | contract | untouched-sections | 373 |  |
| cap:workplan-strict | contract | lines | 378 |  |
| cap:workplan-strict | contract | ceilings-and-exceptions | 438 |  |
| cap:workplan-strict | contract | debt | 476 |  |
| cap:workplan-strict | contract | the-kinds | 529 |  |
| cap:workplan-two-way-coverage | constraints | c-1f4a08dd18482a5d | 240 |  |
| cap:workplan-two-way-coverage | constraints | c-03b4823ef7d8df60 | 241 |  |
| cap:workplan-two-way-coverage | constraints | c-d820f6a1186854f1 | 242 |  |
| cap:workplan-two-way-coverage | constraints | c-6a1ef9632b546e6c | 243 |  |
| cap:workplan-two-way-coverage | constraints | c-47593046790e2245 | 245 |  |
| cap:workplan-two-way-coverage | constraints | c-383f8f25c38e4028 | 252 |  |
| cap:workplan-two-way-coverage | constraints | c-08175d66a60d1ab6 | 257 |  |
| cap:workplan-two-way-coverage | constraints | c-e710cc15944e7529 | 259 |  |
| cap:workplan-two-way-coverage | contract | the-direction-table | 68 |  |
| cap:workplan-two-way-coverage | contract | exported-public-surface | 79 | `raf_workplan::test_crate::coverage::the_code_report_lists_every_public_item_and_no_private_or_hidden_one` |
| cap:workplan-two-way-coverage | contract | unresolved-re-export-observations | 95 | `raf_workplan::test_crate::coverage::a_re_export_the_index_cannot_follow_is_reported_and_enforced_and_its_module_covers_it`, `raf_workplan::test_crate::coverage::a_re_export_written_through_super_names_the_path_super_reaches` |
| cap:workplan-two-way-coverage | contract | definition-based-item-and-test-coverage | 112 | `raf_workplan::test_crate::coverage::a_bound_item_is_covered_and_its_unbound_neighbour_is_a_gap`, `raf_workplan::test_crate::coverage::binding_an_item_covers_what_is_below_it_and_nothing_beside_it`, `raf_workplan::test_crate::coverage::an_item_bound_through_a_re_export_or_its_definition_is_covered_under_its_exported_path`, `raf_workplan::test_crate::coverage::an_item_bound_only_in_examples_is_a_gap`, `raf_workplan::test_crate::coverage::a_bound_test_is_covered_and_every_unbound_test_unit_tests_included_is_a_gap` |
| cap:workplan-two-way-coverage | contract | public-item-report | 130 | `raf_workplan::test_crate::coverage::the_table_and_rdf_preserve_the_same_entries_and_escape_their_carriers`, `raf_workplan::test_crate::coverage::the_code_report_lists_every_public_item_and_no_private_or_hidden_one` |
| cap:workplan-two-way-coverage | contract | test-report | 138 | `raf_workplan::test_crate::coverage::a_bound_test_is_covered_and_every_unbound_test_unit_tests_included_is_a_gap`, `raf_workplan::test_crate::coverage::a_unit_test_module_is_excluded_by_its_prefix_with_the_reason_shown` |
| cap:workplan-two-way-coverage | contract | scope-and-exclusion-matching | 144 | `raf_workplan::test_crate::coverage::a_scope_entry_is_a_package_name_and_a_crate_name_says_which_package_it_means`, `raf_workplan::test_crate::coverage::an_exclusion_without_a_reason_or_that_matches_nothing_fails_naming_the_file`, `raf_workplan::test_crate::coverage::an_item_is_excluded_by_its_exported_or_its_defining_path`, `raf_workplan::test_crate::coverage::a_prefix_matches_below_its_module_and_never_a_sibling_that_shares_its_text`, `raf_workplan::test_crate::coverage::the_first_matching_exclusion_gives_the_reason` |
| cap:workplan-two-way-coverage | contract | scoped-enforcement-and-invalid-document-suppression | 163 | `raf_workplan::test_crate::coverage::the_report_of_check_and_its_findings_follow_the_scope`, `raf_workplan::test_crate::coverage::a_capability_that_does_not_load_gives_its_own_findings_and_no_cascade_of_gaps`, `raf_workplan::test_crate::coverage::without_a_record_check_has_exactly_one_finding_that_says_to_add_it`, `raf_workplan::test_crate::coverage::a_surface_that_cannot_be_listed_is_an_error_of_the_report_and_a_finding_in_scope` |
| cap:workplan-two-way-coverage | contract | binding-reached-package-enforcement | 183 | `raf_workplan::test_crate::strict::a_binding_reached_package_is_enforced_and_an_unreached_one_is_not`, `raf_workplan::test_crate::strict::a_scoped_package_is_enforced_once_with_a_binding_present`, `raf_workplan::test_crate::strict::an_exclusion_of_the_record_takes_a_gap_of_a_bound_package_out` |
| cap:workplan-two-way-coverage | contract | complete-workspace-account | 195 | `raf_workplan::test_crate::coverage::the_code_report_lists_every_public_item_and_no_private_or_hidden_one`, `raf_workplan::test_crate::coverage::a_bound_test_is_covered_and_every_unbound_test_unit_tests_included_is_a_gap`, `raf_workplan::test_crate::coverage::the_report_and_its_two_forms_are_the_same_on_every_run` |
