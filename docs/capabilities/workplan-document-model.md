---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-document-model
title: "Workplan document model"
description: "Capability and change documents, the four states, PROPOSED blocks, the graph, binding resolution and the check rules."
state: implementing
bindings:
  code:
    - raf_workplan::Workplan
    - raf_workplan::Workplan::load
    - raf_workplan::Workplan::check
    - raf_workplan::Workplan::status
    - raf_workplan::Finding
    - raf_workplan::Graph
    - raf_workplan::Graph::code_of
    - raf_workplan::Graph::capabilities_of
    - raf_workplan::Graph::tests_for
    - raf_workplan::Graph::sections_of
    - raf_workplan::Graph::linking_sections
    - raf_workplan::Roadmap
    - raf_workplan::LinkContext
    - raf_workplan::Gate
    - raf_workplan::Gate::workspace
    - raf_workplan::Gate::assert_clean
    - raf_workplan::plain::check
    - raf_workplan::refs::banned_in_line
    - raf_workplan::Workplan::evidence
    - raf_workplan::Workplan::assessments
    - raf_workplan::Workplan::coverage
    - raf_workplan::Graph::evidence_of
    - raf_workplan::Graph::claims_on
    - raf_workplan::Graph::tested_by
    - raf_workplan::TestedByEdge
    - raf_workplan::Graph::assessments_of
    - raf_workplan::Graph::inputs_of
    - raf_workplan::digest::Digest
    - raf_workplan::digest::Digest::section
    - raf_workplan::digest::Digest::symbol
    - raf_workplan::digest::Digest::record
    - raf_workplan::record::Record
    - raf_workplan::record::create
    - raf_workplan::record::replace
    - raf_workplan::graph
    - raf_workplan::Loaded
    - raf_workplan::Opened
    - raf_workplan::GateError
    - raf_workplan::ExampleKind
    - raf_workplan::RoadmapEntry
    - raf_workplan::OpenChange
    - raf_workplan::Gap
    - raf_workplan::record
    - raf_workplan::digest
    - raf_workplan::refs
    - raf_workplan::plain
    - raf_workplan::git::tracked_paths
  tests:
    - raf_workplan::test_crate::documents::accepts_valid_fixture
    - raf_workplan::test_crate::documents::the_model_holds_typed_values_and_bodies
    - raf_workplan::test_crate::documents::a_broken_document_is_a_finding_and_not_in_the_model
    - raf_workplan::test_crate::documents::rejects_type_in_the_wrong_directory
    - raf_workplan::test_crate::documents::rejects_nested_documents
    - raf_workplan::test_crate::documents::rejects_capability_without_frontmatter
    - raf_workplan::test_crate::documents::rejects_proposed_marker_in_architecture
    - raf_workplan::test_crate::rules::every_rule_has_a_fixture_reported_with_file_and_location
    - raf_workplan::test_crate::parse_errors::every_reader_error_has_a_fixture_reported_with_file_and_line
    - raf_workplan::test_crate::kinds::every_rule_and_every_reader_error_has_its_own_kind
    - raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind
    - raf_workplan::test_crate::states::accepts_each_state_with_matching_content
    - raf_workplan::test_crate::states::rejects_considered_without_proposed_block
    - raf_workplan::test_crate::states::rejects_designed_with_proposed_block
    - raf_workplan::test_crate::states::rejects_delivered_with_proposed_block
    - raf_workplan::test_crate::states::rejects_delivered_with_open_change_record
    - raf_workplan::test_crate::states::rejects_each_non_delivered_state_without_an_open_record
    - raf_workplan::test_crate::states::rejects_orphan_proposed_marker
    - raf_workplan::test_crate::states::rejects_proposed_change_that_does_not_list_the_capability
    - raf_workplan::test_crate::states::rejects_proposed_section_missing_from_affects
    - raf_workplan::test_crate::states::a_marker_in_a_code_fence_is_not_read_as_a_block
    - raf_workplan::test_crate::states::surfaces_the_single_document_state_rules_from_the_profile
    - raf_workplan::test_crate::references::accepts_a_depends_on_an_existing_capability_and_an_acyclic_graph
    - raf_workplan::test_crate::references::rejects_depends_missing
    - raf_workplan::test_crate::references::rejects_changes_missing
    - raf_workplan::test_crate::references::rejects_moved_missing
    - raf_workplan::test_crate::references::rejects_depends_cycles_once_for_each_group_naming_its_members
    - raf_workplan::test_crate::references::a_slug_id_that_differs_from_the_file_name_is_refused
    - raf_workplan::test_crate::graph::the_graph_has_every_kind_of_node
    - raf_workplan::test_crate::graph::the_graph_has_every_kind_of_edge
    - raf_workplan::test_crate::graph::links_run_from_the_section_that_holds_them_to_the_section_they_name
    - raf_workplan::test_crate::graph::code_of_lists_the_items_that_realise_a_capability_in_path_order
    - raf_workplan::test_crate::graph::capabilities_of_names_every_capability_bound_to_a_symbol
    - raf_workplan::test_crate::graph::tests_for_a_binding_named_in_two_sections_substantiates_both
    - raf_workplan::test_crate::graph::sections_of_a_change_are_its_affected_sections_sorted
    - raf_workplan::test_crate::graph::linking_sections_are_section_granular
    - raf_workplan::test_crate::links::rejects_broken_relative_link
    - raf_workplan::test_crate::links::rejects_broken_anchor
    - raf_workplan::test_crate::links::rejects_link_to_a_tracked_symlink
    - raf_workplan::test_crate::links::rejects_link_leaving_the_repository
    - raf_workplan::test_crate::links::anchors_follow_github_rules
    - raf_workplan::test_crate::links::the_oracle_is_the_tracked_set_and_not_the_tree
    - raf_workplan::test_crate::refs::the_matcher_bans_references_and_process_wording
    - raf_workplan::test_crate::refs::the_matcher_spares_colours_prose_numbers_and_near_misses
    - raf_workplan::test_crate::refs::every_document_type_and_the_frontmatter_are_scanned
    - raf_workplan::test_crate::bindings::bindings_that_name_real_symbols_resolve
    - raf_workplan::test_crate::bindings::a_binding_through_a_re_export_resolves_to_its_definition
    - raf_workplan::test_crate::bindings::a_binding_to_a_missing_path_is_reported_with_its_capability
    - raf_workplan::test_crate::bindings::a_binding_to_a_macro_generated_function_is_not_found
    - raf_workplan::test_crate::bindings::a_path_two_globs_both_bring_in_is_ambiguous_and_names_both_definitions
    - raf_workplan::test_crate::bindings::a_tests_binding_must_name_a_function_marked_test
    - raf_workplan::test_crate::bindings::a_code_binding_must_name_a_shipping_item
    - raf_workplan::test_crate::bindings::an_examples_binding_is_an_example_target_or_a_documented_library_item
    - raf_workplan::test_crate::bindings::a_missing_example_target_is_reported_with_its_capability_list_and_binding
    - raf_workplan::test_crate::roadmap::the_groups_are_in_catalogue_order_and_sorted_by_slug
    - raf_workplan::test_crate::roadmap::an_implementing_capability_lists_its_gaps_exactly
    - raf_workplan::test_crate::roadmap::this_repository_lists_every_capability_once_in_its_state_group
    - raf_workplan::test_crate::roadmap::a_built_capability_is_delivered_or_reopened_under_an_open_change_and_has_no_gap
    - raf_workplan::test_crate::records::a_replacement_through_a_link_to_the_documents_never_overwrites_a_document
    - raf_workplan::test_crate::records::a_directory_link_inside_the_root_on_the_way_to_a_record_is_refused_and_a_real_directory_is_not
    - raf_workplan::test_crate::gate::a_clean_repository_passes
    - raf_workplan::test_crate::gate::a_broken_repository_panics_listing_every_finding
    - raf_workplan::test_crate::gate::a_missing_capabilities_directory_is_an_error
    - raf_workplan::test_crate::gate::a_capabilities_directory_without_a_capability_document_is_an_error
    - raf_workplan::test_crate::gate::a_missing_changes_directory_is_the_empty_set_of_changes
    - raf_workplan::test_crate::gate::inside_a_git_checkout_the_tracked_paths_come_from_the_index
    - raf_workplan::test_crate::plain::reads_markdown_outside_the_document_directories_with_or_without_frontmatter
    - raf_workplan::test_crate::plain::a_broken_link_and_a_split_link_are_reported_together_for_a_plain_file
    - raf_workplan::test_crate::generated::a_body_that_mixes_line_endings_is_refused_by_sync_and_reported_by_check
    - raf_workplan::test_crate::records::each_record_type_loads_from_its_directory_into_the_model
    - raf_workplan::test_crate::records::a_repository_without_record_directories_has_no_records_and_no_new_findings
    - raf_workplan::test_crate::records::work_coverage_holds_only_the_coverage_record
    - raf_workplan::test_crate::records::a_record_that_names_what_does_not_exist_is_a_finding_and_its_neighbour_is_clean
    - raf_workplan::test_crate::records::a_claim_on_unknown_evidence_names_the_file_and_the_line
    - raf_workplan::test_crate::records::a_claim_on_evidence_for_another_capability_is_a_finding
    - raf_workplan::test_crate::records::claims_are_section_granular_and_one_edge_per_named_record
    - raf_workplan::test_crate::records::the_records_are_nodes_with_their_edges
    - raf_workplan::test_crate::records::each_input_of_a_record_that_resolves_is_an_edge_and_one_that_does_not_is_no_finding
    - raf_workplan::test_crate::records::replace_renames_a_new_file_over_the_target_and_never_writes_into_it
    - raf_workplan::test_crate::records::a_written_record_reloads_with_no_finding_as_the_same_value
    - raf_workplan::test_crate::records::create_never_replaces_a_file
    - raf_workplan::test_crate::records::replace_puts_new_text_in_place_of_an_existing_record_only
    - raf_workplan::test_crate::records::a_record_that_would_not_load_clean_is_not_written
    - raf_workplan::test_crate::records::a_hundred_thousand_claims_load_with_one_finding_each
    - raf_workplan::test_crate::digest::each_kind_hashes_its_documented_encoding
    - raf_workplan::test_crate::digest::a_digest_is_the_same_on_every_run
    - raf_workplan::test_crate::digest::crlf_and_lf_give_the_same_digest
    - raf_workplan::test_crate::digest::a_regenerated_block_leaves_the_section_digest_and_an_edit_moves_it
    - raf_workplan::test_crate::digest::every_definition_of_a_path_counts_once_in_file_order
    - raf_workplan::digest::tests::sha256_gives_the_standard_test_vectors
    - raf_workplan::test_crate::bindings::a_binding_to_an_item_under_the_module_that_declares_its_file_is_not_found
    - raf_workplan::test_crate::bindings::a_capability_that_binds_nothing_has_no_binding_findings
    - raf_workplan::test_crate::bindings::a_failure_with_no_name_to_find_says_a_macro_or_derive_may_be_behind_it
    - raf_workplan::test_crate::bindings::a_library_that_runs_no_documentation_tests_has_no_doctest_examples
    - raf_workplan::test_crate::bindings::a_macro_in_the_file_of_a_module_is_read_for_the_path_of_that_module
    - raf_workplan::test_crate::bindings::a_name_held_only_inside_a_nested_group_of_the_arguments_is_still_named
    - raf_workplan::test_crate::bindings::every_finding_of_a_wrong_role_names_the_fix_for_its_list
    - raf_workplan::test_crate::bindings::every_unresolved_binding_of_every_list_is_reported
    - raf_workplan::test_crate::bindings::one_failing_site_fails_a_binding_that_resolves_to_several
    - raf_workplan::test_crate::bindings::only_a_path_before_a_bang_and_a_group_is_an_invocation
    - raf_workplan::test_crate::bindings::the_graph_keeps_each_resolution_for_the_rules_and_the_views_that_read_it
    - raf_workplan::test_crate::bindings::the_macro_hint_is_given_only_for_a_macro_at_the_item_level_of_the_parent_module
    - raf_workplan::test_crate::bindings::the_macro_hint_is_only_for_a_parent_that_is_a_module
    - raf_workplan::test_crate::bindings::the_role_is_judged_on_the_definition_a_re_export_leads_to
    - raf_workplan::test_crate::digest::a_digest_is_written_and_read_as_sixty_four_lowercase_hex_digits
    - raf_workplan::test_crate::digest::a_path_that_names_no_single_symbol_has_no_digest
    - raf_workplan::test_crate::digest::a_symbol_and_a_record_with_equal_bytes_differ
    - raf_workplan::test_crate::digest::a_symbol_digest_reads_the_definitions_of_the_path
    - raf_workplan::test_crate::documents::a_directory_that_cannot_be_listed_is_reported_by_name
    - raf_workplan::test_crate::documents::a_finding_renders_with_its_place_and_its_fix
    - raf_workplan::test_crate::documents::a_generated_index_may_carry_a_licence_header
    - raf_workplan::test_crate::documents::a_nested_typed_document_is_nested_and_not_read_as_a_document
    - raf_workplan::test_crate::documents::a_tree_that_cannot_be_built_is_a_finding_for_each_directory
    - raf_workplan::test_crate::documents::accepts_a_generated_capability_index
    - raf_workplan::test_crate::documents::accepts_a_generated_index_with_an_spdx_header
    - raf_workplan::test_crate::documents::accepts_comments_after_scalars
    - raf_workplan::test_crate::documents::accepts_optional_tags_and_timestamp
    - raf_workplan::test_crate::documents::accepts_quoted_and_block_forms
    - raf_workplan::test_crate::documents::an_empty_repository_has_no_findings
    - raf_workplan::test_crate::documents::an_index_is_navigation_and_not_a_stray_file
    - raf_workplan::test_crate::documents::an_oversize_file_is_reported_by_name_and_not_skipped
    - raf_workplan::test_crate::documents::comments_hide_text_but_not_the_text_beside_them
    - raf_workplan::test_crate::documents::files_that_are_not_markdown_are_not_documents
    - raf_workplan::test_crate::documents::findings_come_in_directory_then_name_order_and_repeat
    - raf_workplan::test_crate::documents::fixture_builder_is_self_consistent
    - raf_workplan::test_crate::documents::placeholder_scan_spares_code_and_longer_words
    - raf_workplan::test_crate::documents::placeholder_scan_spares_longer_words_and_dotted_prose
    - raf_workplan::test_crate::documents::rejects_id_filename_mismatch
    - raf_workplan::test_crate::documents::rejects_missing_section
    - raf_workplan::test_crate::documents::rejects_placeholder_spellings
    - raf_workplan::test_crate::documents::rejects_placeholder_tokens
    - raf_workplan::test_crate::documents::rejects_proposed_marker_in_change
    - raf_workplan::test_crate::documents::rejects_resource_key
    - raf_workplan::test_crate::documents::rejects_sections_left_empty_by_comments
    - raf_workplan::test_crate::documents::rejects_typed_document_outside_its_directory
    - raf_workplan::test_crate::documents::rejects_unclosed_frontmatter
    - raf_workplan::test_crate::documents::rejects_unknown_key
    - raf_workplan::test_crate::documents::rejects_unparseable_frontmatter
    - raf_workplan::test_crate::gate::a_directory_that_does_not_exist_is_an_error
    - raf_workplan::test_crate::gate::a_git_dir_that_names_no_repository_is_an_error_and_not_a_walk
    - raf_workplan::test_crate::gate::a_link_to_a_file_the_tracked_set_leaves_out_is_link_broken
    - raf_workplan::test_crate::gate::a_link_to_a_symlink_the_tracked_set_names_is_link_symlink
    - raf_workplan::test_crate::gate::a_source_workspace_that_does_not_load_is_an_error
    - raf_workplan::test_crate::gate::a_work_tree_whose_index_cannot_be_listed_is_an_error_and_not_a_walk
    - raf_workplan::test_crate::gate::each_call_of_workplan_reads_the_documents_as_they_are_on_disk_then
    - raf_workplan::test_crate::gate::open_gives_the_tree_the_workspace_the_tracked_paths_and_the_documents
    - raf_workplan::test_crate::gate::open_is_the_load_of_try_check_and_refuses_what_it_refuses
    - raf_workplan::test_crate::gate::the_gate_under_a_git_dir_that_names_nothing
    - raf_workplan::test_crate::gate::the_programs_of_a_gate_are_started_in_its_directory
    - raf_workplan::test_crate::gate::the_tracked_paths_a_caller_gives_replace_the_listing_and_start_no_process
    - raf_workplan::test_crate::graph::a_code_span_that_continues_the_bound_path_with_a_segment_does_not_substantiate
    - raf_workplan::test_crate::graph::a_link_that_leads_nowhere_in_the_model_is_no_edge
    - raf_workplan::test_crate::graph::bindings_record_what_the_source_index_made_of_each_path
    - raf_workplan::test_crate::graph::hostile_an_anchor_only_another_document_has_is_no_edge_and_a_finding
    - raf_workplan::test_crate::graph::hostile_duplicate_ids_across_files_load_one_capability_and_report_the_other_file
    - raf_workplan::test_crate::graph::hostile_duplicated_heading_anchors_are_numbered_and_the_next_one_is_missing
    - raf_workplan::test_crate::graph::hostile_links_in_fences_and_code_spans_are_neither_edges_nor_findings
    - raf_workplan::test_crate::graph::hostile_self_dependency_is_one_cycle_and_one_edge
    - raf_workplan::test_crate::graph::hostile_unterminated_backticks_name_nothing_and_break_nothing
    - raf_workplan::test_crate::graph::tests_for_a_binding_named_only_in_prose_substantiates_verification
    - raf_workplan::test_crate::graph::tests_for_a_path_that_does_not_resolve_is_still_a_binding_of_its_sections
    - raf_workplan::test_crate::graph::tests_for_ignores_a_name_inside_a_proposed_block_or_a_fence
    - raf_workplan::test_crate::graph::the_fixture_loads_with_only_its_two_unresolved_bindings
    - raf_workplan::test_crate::graph::the_queries_are_deterministic
    - raf_workplan::test_crate::kinds::every_finding_names_a_fix_and_a_message
    - raf_workplan::test_crate::kinds::kinds_are_unique_kebab_case_names
    - raf_workplan::test_crate::links::a_directory_the_walk_cannot_list_is_reported_and_not_skipped
    - raf_workplan::test_crate::links::a_document_is_judged_against_the_supplied_tracked_set
    - raf_workplan::test_crate::links::a_fragment_link_is_judged_against_its_own_document
    - raf_workplan::test_crate::links::a_link_to_a_directory_resolves_and_its_anchor_is_not_read
    - raf_workplan::test_crate::links::a_symlink_the_caller_names_is_tracked_and_still_a_symlink
    - raf_workplan::test_crate::links::accepts_resolving_reference_and_attribute_links
    - raf_workplan::test_crate::links::an_anchor_in_a_non_markdown_file_is_not_judged
    - raf_workplan::test_crate::links::autolinks_are_external_and_not_judged
    - raf_workplan::test_crate::links::does_not_read_attributes_that_only_end_in_href_or_src
    - raf_workplan::test_crate::links::links_inside_documents_are_judged_wherever_the_document_is_valid_or_not
    - raf_workplan::test_crate::links::reads_an_attribute_value_that_starts_on_the_next_line
    - raf_workplan::test_crate::links::reads_attributes_with_spaces_around_the_equals_sign
    - raf_workplan::test_crate::links::rejects_a_link_whose_target_cannot_be_read_for_its_anchor
    - raf_workplan::test_crate::links::rejects_absolute_and_empty_link_targets_beside_relative_ones
    - raf_workplan::test_crate::links::rejects_an_empty_link_target
    - raf_workplan::test_crate::links::rejects_broken_attributes_in_tags_split_across_lines
    - raf_workplan::test_crate::links::rejects_broken_href_and_src_attributes
    - raf_workplan::test_crate::links::rejects_broken_link_outside_the_document_directories
    - raf_workplan::test_crate::links::rejects_broken_reference_definition
    - raf_workplan::test_crate::links::rejects_broken_unquoted_attribute_values
    - raf_workplan::test_crate::links::tag_scan_spares_prose_autolinks_and_closed_tags
    - raf_workplan::test_crate::links::tag_state_does_not_survive_a_blank_line_or_a_fence
    - raf_workplan::test_crate::links::the_directories_of_a_named_symlink_are_tracked_directories
    - raf_workplan::test_crate::links::the_symlinks_a_tree_lists_are_tracked_symlinks
    - raf_workplan::test_crate::links::the_tree_is_walked_only_when_a_relative_link_needs_it
    - raf_workplan::test_crate::plain::a_claim_marker_outside_a_capability_is_reported_and_a_misspelt_marker_is_refused
    - raf_workplan::test_crate::plain::a_tested_by_marker_outside_a_capability_is_reported_and_a_malformed_one_is_refused
    - raf_workplan::test_crate::plain::accepts_one_line_links_and_links_in_fences_beside_the_split_rule
    - raf_workplan::test_crate::plain::an_index_in_a_document_directory_other_than_the_capability_index_is_read_as_markdown
    - raf_workplan::test_crate::plain::does_not_read_links_or_placeholders_inside_html_comments
    - raf_workplan::test_crate::plain::reads_a_proposed_block_in_plain_markdown_as_outside_a_capability
    - raf_workplan::test_crate::plain::reads_placeholders_in_plain_markdown_outside_code
    - raf_workplan::test_crate::plain::rejects_a_link_split_across_lines_in_a_profile_document
    - raf_workplan::test_crate::plain::rejects_a_link_split_across_lines_in_any_tracked_markdown_file
    - raf_workplan::test_crate::plain::rejects_generated_marker_errors
    - raf_workplan::test_crate::plain::rejects_unclosed_frontmatter_in_plain_markdown
    - raf_workplan::test_crate::plain::reports_marker_errors_of_plain_markdown_once
    - raf_workplan::test_crate::records::a_broken_record_is_a_finding_and_not_in_the_model
    - raf_workplan::test_crate::records::a_claim_inside_a_proposed_block_is_not_a_claim_until_it_is_resolved
    - raf_workplan::test_crate::records::a_claim_on_evidence_that_covers_its_capability_is_clean_and_an_edge
    - raf_workplan::test_crate::records::a_record_at_the_size_bound_with_the_most_measures_loads
    - raf_workplan::test_crate::records::a_record_in_the_directory_of_another_type_is_wrong_directory
    - raf_workplan::test_crate::records::a_record_under_another_file_name_and_a_nested_record_are_refused
    - raf_workplan::test_crate::records::a_reference_that_does_not_resolve_makes_no_edge
    - raf_workplan::test_crate::records::a_symbolic_link_on_the_way_to_the_record_is_refused
    - raf_workplan::test_crate::records::the_record_directory_is_created_below_the_root_only
    - raf_workplan::test_crate::references::a_matching_kebab_slug_is_accepted
    - raf_workplan::test_crate::references::a_reference_to_a_broken_document_is_not_a_missing_one
    - raf_workplan::test_crate::references::a_reference_to_a_file_without_frontmatter_is_missing
    - raf_workplan::test_crate::references::a_slug_that_is_not_kebab_case_is_refused
    - raf_workplan::test_crate::references::accepts_a_moved_outcome_that_names_an_existing_change
    - raf_workplan::test_crate::references::accepts_moved_outcome
    - raf_workplan::test_crate::references::every_reference_case_is_reported_exactly
    - raf_workplan::test_crate::references::two_separate_cycles_are_two_findings_and_a_cycle_with_a_tail_is_one
    - raf_workplan::test_crate::refs::a_banned_line_names_why
    - raf_workplan::test_crate::refs::a_reference_is_reported_once_for_a_stray_page_and_a_document_that_does_not_read
    - raf_workplan::test_crate::refs::issue_token_scan_spares_hex_colours
    - raf_workplan::test_crate::refs::rejects_acceptance_label
    - raf_workplan::test_crate::refs::rejects_an_issue_url_a_pull_label_and_an_issue_label_in_documents
    - raf_workplan::test_crate::refs::rejects_issue_number_token
    - raf_workplan::test_crate::refs::rejects_process_wording
    - raf_workplan::test_crate::refs::the_matcher_pins_its_edge_cases
    - raf_workplan::test_crate::roadmap::a_fully_bound_implementing_capability_has_no_gap
    - raf_workplan::test_crate::roadmap::a_resolving_binding_of_the_wrong_role_is_not_a_gap
    - raf_workplan::test_crate::roadmap::an_empty_repository_has_an_empty_roadmap
    - raf_workplan::test_crate::roadmap::an_entry_carries_what_the_catalogue_row_needs
    - raf_workplan::test_crate::roadmap::only_an_implementing_capability_has_gaps
    - raf_workplan::test_crate::roadmap::open_change_slugs_are_the_open_changes_without_their_prefix
    - raf_workplan::test_crate::roadmap::open_changes_are_the_changes_that_list_the_capability
    - raf_workplan::test_crate::roadmap::planned_is_designed_then_considered_and_not_a_state
    - raf_workplan::test_crate::roadmap::slug_order_is_not_file_name_order
    - raf_workplan::test_crate::roadmap::the_open_changes_are_sorted_by_slug_whatever_the_order_of_the_files
    - raf_workplan::test_crate::roadmap::the_roadmap_does_not_depend_on_insertion_order_and_repeats
    - raf_workplan::test_crate::states::a_block_that_names_a_broken_change_is_not_a_missing_change
    - raf_workplan::test_crate::states::a_broken_record_does_not_make_the_capabilities_it_lists_unlisted
    - raf_workplan::test_crate::states::a_record_that_lists_two_capabilities_must_affect_the_section_of_each
    - raf_workplan::test_crate::states::rejects_open_capability_without_change_record
    - raf_workplan::test_crate::states::the_blocks_of_a_capability_that_does_not_validate_still_name_a_record
    - raf_workplan::bindings::tests::the_invocations_of_a_module_are_listed_once_however_many_bindings_miss
    - raf_workplan::digest::tests::fields_of_different_kinds_with_equal_bytes_differ
    - raf_workplan::digest::tests::sha256_is_the_same_over_block_boundaries_and_pieces
    - raf_workplan::digest::tests::symbol_definitions_are_deduplicated_by_source_key_and_ordered
    - raf_workplan::digest::tests::the_encoding_is_unambiguous_across_names_texts_and_fields
    - raf_workplan::finding::tests::each_location_answers_to_exactly_one_accessor
    - raf_workplan::graph::tests::a_section_of_many_blocks_is_read_in_work_linear_in_its_blocks
    - raf_workplan::graph::tests::ten_thousand_named_paths_are_found_in_one_pass
    - raf_workplan::graph::tests::the_claims_of_many_capabilities_are_judged_against_sets_built_once
    - raf_workplan::graph::tests::the_paths_of_the_code_spans_are_the_paths_names_in_code_span_accepts
    - raf_workplan::work::tests::a_counter_increments_by_what_is_added
    - raf_workplan::work::tests::a_limited_kind_panics_at_its_ceiling
    - raf_workplan::work::tests::a_limited_measure_counts_the_other_kinds_without_a_ceiling
    - raf_workplan::work::tests::a_measure_that_panicked_leaves_neither_counts_nor_ceiling_behind
    - raf_workplan::work::tests::every_kind_has_its_own_counter
    - raf_workplan::work::tests::passing_the_ceiling_panics_at_once
    - raf_workplan::work::tests::reaching_the_ceiling_is_allowed
    - raf_workplan::work::tests::the_panic_comes_before_the_work_that_follows_it
    - raf_workplan::work::tests::the_result_of_the_closure_is_returned
    - raf_workplan::work::tests::the_tallies_are_reset_between_measures
    - raf_workplan::test_crate::digest::a_module_stands_for_its_files_and_the_modules_below_it_and_a_sibling_does_not_move_it
    - raf_workplan::test_crate::strict::the_record_and_the_floor_give_the_stronger_strictness
    - raf_workplan::test_crate::strict::the_strict_findings_are_made_by_check_after_every_other_finding_and_never_by_the_load
    - raf_workplan::test_crate::strict::a_strict_gate_judges_strict_and_the_workplans_it_opens_keep_its_floor
  examples:
    - raf_workplan::Gate
    - raf_workplan::LinkContext
    - raf_workplan::plain::check
depends: [cap:okf-profile, cap:source-index]
---

# Workplan document model

## Purpose

Plan changes to the documented system. Implement those changes. Reconcile the
documentation with the result. Let Git retain the previous states. The workplan model
is built on that sentence. A capability has one permanent document that states its
current contract, and a change is a temporary record of how that document is about to
move. A reader never has to reconstruct chronology to learn what applies now.

## Contract

**Capability documents.** A capability lives at `docs/capabilities/<slug>.md`, with
the identity `cap:<slug>`, the keys and the seven sections defined in
[the RAF profile of OKF](../architecture/okf-profile.md) (its
[section table](../architecture/okf-profile.md#body-rules) says what each section holds) and read by
[the OKF reader](./okf-profile.md). The document is permanent: it keeps its identity
and its path from the first time the capability is considered until the capability is
removed.

**The four states.** `state` is one of the following, and each fixes what the
document and the change records may say.

| State | What the capability document holds | Change record |
| --- | --- | --- |
| `considered` | Purpose, open questions and the candidate contract, the candidate inside PROPOSED blocks | An open record lists it |
| `designed` | The designed contract, the selected approach, rationale and planned examples, stated plainly | An open record lists it |
| `implementing` | The same contract with real code and test bindings and the remaining gaps | An open record lists it |
| `delivered` | The supported contract, usage, examples, limitations, current rationale and verification links | No open record lists it |

A change to a delivered capability puts the capability back under an open record,
which moves it to `implementing`: the current account stays as written, and the
new text of the change sits beside it in PROPOSED blocks. Everything that moves a capability
between states is a change.

**PROPOSED blocks.** Text that a change proposes is written into the capability
document, marked as a block quote whose first line names the change:

```markdown
> [!PROPOSED change:incremental-parsing]
> The parser accepts input in arbitrary chunks and reports incomplete input when
> the caller finishes the stream.
```

The block continues while lines keep the quote marker, and ends at the first line
without it, at the next marker, or at a fenced code block. The branch diff therefore
shows the code and the documentation it produces together. Outside PROPOSED
blocks a document states the current contract only. A PROPOSED block is valid only in
a Capability document, only when `work/changes/<slug>.md` exists, only when that
record lists the capability, and only when the record's `affects` names the section
that holds the block. A `delivered` or `designed` capability has none, and a
`considered` capability has at least one.

**Change documents.** A change lives at `work/changes/<slug>.md` with the identity
`change:<slug>`. Its keys are `type`, `id`, `title`, `description`, `changes`,
`affects`, `contract_effect` and `items`:

- `changes` lists the capabilities the change alters.
- `affects` lists `cap:<slug>#<section>` entries naming the sections that must be
  reconciled.
- `contract_effect` is `changes` when the documented contract moves and `none` when
  the change is internal.
- `items` lists the claims the change makes. Each has a `claim` and, before the
  change completes, an optional `outcome` of `implemented`, `dropped` or
  `moved:change:<slug>`. A `moved` outcome names an existing change that now carries
  the claim.

The body holds a non-empty `## Summary`. Completing a change deletes the file, and Git
holds its history.

**Slug identities.** A slug is lowercase kebab case and equals the file name without
`.md`. Every reference between documents uses the id form, so a rename is one edit to
the file name, the id and the references, which `check` verifies.

**Loading.** `Workplan::load` reads `docs/capabilities`, `docs/architecture` and
`work/changes` through a source tree, validates each document with the OKF reader
and keeps the typed documents and their bodies. A document that does not validate is
a finding and is not in the model; nothing is skipped and nothing panics. Links are
judged against the tracked paths of a `LinkContext` (`Workplan::load_with`);
`Workplan::load` takes every file of the tree as tracked. `Workplan::check` returns
every finding. A `Finding` has a stable kebab-case kind
from `Finding::KINDS`, the file, a location (a key, a section or a line) and a fix;
each rule of the profile and each reader error has its own kind.

**Records.** `Workplan::load` also reads the records of the profile: the evidence
records of `work/evidence`, the assessments of `work/assessments` and the coverage
record `work/coverage/default.md`, each validated with the OKF reader and kept as a
typed record (`Workplan::evidence`, `Workplan::assessments`, `Workplan::coverage`).
A record directory holds its records directly, as a document directory does, and
`work/coverage` holds only `default.md`: any other file there is
`singleton-other-file` and is not read. A directory that does not exist holds
nothing, so a repository without records loads and reports no finding of the record
rules; that it has no coverage record is the one finding
[two-way coverage](./workplan-two-way-coverage.md) gives it.

**Strictness.** `Workplan::strictness` returns how hard `check` judges the
repository: the stronger of the coverage record's `strictness` and the floor that
`Workplan::with_strictness` sets. A record without the key, a record that does not
load and a workplan with no floor count as `Strictness::Default`, and a floor below
the one already set changes nothing, so a caller can raise the strictness and
nothing lowers it. `Gate::strict()` and `Gate::strictness(floor)` set the floor of
a `Gate` in the same way, `Opened::strictness_floor` returns it, and every
`Workplan` an `Opened` gives has that floor, the one that `change complete` checks
once the record is removed included. Under `Strictness::Strict`, `Workplan::check`
appends the findings of the rules of [strict mode](./workplan-strict.md) after those
of the rules below; under `Strictness::Default` it judges no rule of strict mode.
The load judges none of them, so `Workplan::findings` is the same under either
strictness.

> [!PROPOSED change:workplan-strict]
> `Workplan::load` also reads the strict debt record `work/debt/default.md` of
> [strict mode](./workplan-strict.md) (`Workplan::debt`), and `work/debt` holds only
> `default.md`, as `work/coverage` does: any other file there is
> `singleton-other-file`. A repository without the directory has no debt record,
> which is no finding. Under `Strictness::Strict`, `Workplan::check` applies the debt
> record after the findings of the rules of strict mode.

**The graph.** `raf-workplan` builds a graph (`Workplan::graph`) from the documents
and the source.

- Nodes: capabilities, sections, changes, architecture pages, proposals, the
  code symbols, tests and examples that bindings name, the records, the crates the
  coverage record scopes and the symbols the inputs of records name.
- Edges: bindings (capability to symbol, test or example), `depends`, `changes`,
  `affects` and Markdown links. A link runs from the section that holds it to the
  section its anchor names, or to the document when it has no anchor.
- Record edges: `for` from an evidence record to each capability it covers; a claim
  from the section of a capability that holds a claim marker to each evidence
  record the marker names; the subject of an assessment, a section; `scope` from
  the coverage record to each crate; and an input edge from a record to the
  section, the definitions of the symbol or the evidence record each of its input
  rows names. An input whose name resolves to nothing the repository holds makes no
  edge and is no finding of the load: an input that moved is a matter of the
  record's currency. A claim marker inside a PROPOSED block is not a claim until
  the block is resolved.
- Tested-by edges: from the section of a capability that holds a tested-by marker
  to each test or example binding of the same capability whose path the marker
  names.

The graph must have no `depends` cycle. It answers questions in both directions:
which code realises this capability (`Graph::code_of`), which capabilities a symbol
is bound to (`Graph::capabilities_of`), which tests address a section
(`Graph::tests_for`), which sections a change must update (`Graph::sections_of`),
and which sections link to a section (`Graph::linking_sections`). For the records it
lists the `for` edges (`Graph::covers`) and the evidence that covers a capability
(`Graph::evidence_of`), the claims (`Graph::claims`) and those on one evidence
record (`Graph::claims_on`), the tested-by edges (`Graph::tested_by`), the subjects of the assessments (`Graph::assesses`)
and the assessments of a section (`Graph::assessments_of`), the `scope` edges
(`Graph::scope`), and the input edges (`Graph::inputs`) and those of one record
(`Graph::inputs_of`).

**Binding resolution.** Each entry in `bindings.code` is a Rust path to a shipping
item, each entry in `bindings.tests` a path to a function marked `#[test]`, and each
entry in `bindings.examples` an example target (`<package>::example_crate::<name>`)
or a documented item of a library whose documentation tests run. All resolve through
the [source index](./source-index.md), including re-exports, and the role is judged
on the definition the path leads to. A binding that does not resolve, that names more
than one definition, or whose item does not fit its list is an error naming the
capability, the list and the binding. A symbol that a macro generates is not in the
index, so it cannot be bound. A capability with nothing built binds nothing, so
`designed` capabilities carry empty lists. An `implementing` capability may have
empty or missing bindings; `status` and the catalogue report each one as a gap;
[reconcile](./workplan-reconcile.md) defines the obligation it becomes. A
`delivered` capability binds at least one code path and one test.

**Status.** `Workplan::status` returns the `Roadmap`: every capability that loaded,
grouped by state in the order `delivered`, `implementing`, `designed`, `considered`
and sorted by slug inside a group. Each entry carries its state, the open changes
that list it and, for an `implementing` capability, its gaps: the binding lists that
are empty and the bindings that do not resolve.

**Checks.** `check` reports every violation of these rules, each naming its file:

- the schemas and closed key sets of every document;
- flat document directories, each document in the directory of its type, and no
  Markdown file without frontmatter beside the documents;
- ids that equal their file names, and references that exist;
- an acyclic `depends` graph;
- links that resolve inside the repository against the tracked paths, including
  heading anchors, and no link to a tracked symlink;
- no issue or pull request reference and no development-process wording, by the
  one pattern set of `raf_workplan::refs` that every consumer's `check` applies;
- a document whose body mixes CRLF and LF line endings, which is an error;
- bindings that resolve through the source index in their roles;
- generated blocks that match what the sync operation (`Workplan::sync`) writes
  (see [generated documents](./workplan-generated-docs.md));
- PROPOSED blocks that name an existing change which lists their capability and
  affects their section;
- an open change record for every capability that is not `delivered`;
- a `delivered` capability with no PROPOSED block, no open change record and no
  empty required section;
- a `designed` capability with no PROPOSED block;
- a `considered` capability with its candidate contract in PROPOSED blocks;
- the references of the records and the claims, each with its file and, for a
  claim, the line of its marker: a claim on evidence that does not exist
  (`claim-evidence-missing`) or whose `for` does not list the capability
  (`claim-evidence-not-for`); an evidence record whose `for` names no capability
  document (`evidence-for-missing`); an assessment whose subject is a section of no
  capability document (`assessment-subject-missing`); an assessment whose id is not
  `assessment:<capability slug>-<section>` of its subject, the one id the
  assessment of that section has (`assessment-id-mismatch`); and a `scope` entry
  that is no package of the source workspace (`scope-crate-missing`). A name whose
  document or record exists and is broken is explained by the findings of that
  file;
- under either strictness, a tested-by marker entry that is no path of the
  capability's `tests` or `examples` bindings (`claim-test-unbound`), at the line of
  the marker; each entry that is one is a `TestedByEdge` of the graph
  (`Graph::tested_by`), from the section that holds the marker to the binding;
- under strict, the rules of [strict mode](./workplan-strict.md), each a `Finding`
  of a kind in `Finding::KINDS` with its file, location and fix;
- under either strictness, the rules of the line keys of the coverage record: a line
  exception that names nothing of the source index (`coverage-exception-unmatched`)
  and a `ceilings` row whose `crate` is no package of the source workspace
  (`scope-crate-missing`), each at its key.

> [!PROPOSED change:workplan-strict]
> - under either strictness, a debt record while the repository is judged under
>   `default` (`debt-without-strict`);
> - under strict, the debt record's own kinds, `debt-resolved` and
>   `debt-entry-invalid`.

Markdown outside the document directories, such as a README, is judged by the same
link, marker and reference rules through `raf_workplan::plain::check`.

**Digests.** `raf_workplan::digest::Digest` is the SHA-256 (FIPS 180-4, computed
in the crate) of a canonical encoding of a record input, one or more fields
`<kind> NUL <name> NUL <length> NUL <text>`: the kind (`section`, `symbol` or
`record`) keeps the domains apart and the decimal length says where each field
ends.
`Digest::section` hashes the authored text of a section, with the content of its
generated blocks left out; `Digest::symbol` hashes the source span of each
definition a path resolves to, once per source item and in file order, and for a
module below a crate root also the whole text of each file of the module and of the
modules below it, so a module binding stands for the module's content as it does in
coverage and an edit inside the module moves the digest; `Digest::record` hashes the
text of a record. Every text has CRLF read as LF, so a digest is the same on every
run and platform. A digest is written and read as 64 lowercase hexadecimal digits.

**Writing records.** `raf_workplan::record::create` writes a new `Record` file
exclusively, never replacing a file, below the root, creating the directory of
its type when it is missing; `record::replace` puts new text in place of an
existing record file through a temporary file and a rename. Each judges the way to
the record before anything is written, as every write of a record does: `work` and
the directory of the record type must each be a directory (`create` makes one that
is missing), and a symbolic link there, whether it leads inside the root or out of
it, a file or anything else is refused, naming that directory. A record is therefore written in the record
directory of the root and never through a link onto another file, such as a
document. Both write the text the OKF writer renders for the record, opened with
the two SPDX comment lines of `RecordOptions` when both are given, and refuse,
before anything is written, one of the two lines given without the other and a
record whose text would not read back as a record with no finding.

## Usage

A project runs the checks from a test with the dev-dependency API:

```rust
// tests/workplan.rs
#[test]
fn workplan() {
    raf_workplan::Gate::workspace(env!("CARGO_MANIFEST_DIR")).assert_clean();
}
```

`Gate::workspace` reads `docs/capabilities`, `docs/architecture` and `work/changes`
under the directory, and the records of `work/evidence`, `work/assessments` and
`work/coverage`, loads the workspace sources from the roots `["."]`
(`Gate::source_roots` changes them), judges links against the paths `git ls-files`
lists (every file under the directory when `git` answers that the directory is not
inside a work tree, and the caller's list with `Gate::tracked_paths`), and runs the
checks; `assert_clean` panics with every finding, each with its
location and fix, when there is one. A directory without a capability document is an
error, not a clean verdict, a missing `work/changes` is the empty set of changes, and
a missing record directory holds no record.
A command line is specified separately ([command line](./workplan-cli.md)) and is
not part of this capability.

## Examples

A change to a delivered capability, shown as the capability document, in state
`implementing`, reads while the change is open. The note before the quote is plain
text; the quote is the proposal.

```markdown
## Contract

The parser accepts a complete input and returns its tree.

> [!PROPOSED change:incremental-parsing]
> The parser also accepts input in arbitrary chunks through `push` and `finish`.
```

and the matching change record frontmatter:

```yaml
type: Change
id: change:incremental-parsing
title: "Add incremental parsing"
description: "Chunked input for the streaming parser."
changes: [cap:streaming-parser]
affects:
  - cap:streaming-parser#contract
  - cap:streaming-parser#usage
  - cap:streaming-parser#verification
contract_effect: changes
items:
  - claim: "Input may arrive in arbitrary chunks."
```

## Constraints

- Types, keys and values are exactly those of the profile page; this model adds
  rules between documents and the source, and the line-ending rule for the documents
  it rewrites, and restates the profile's single-document binding minimum for
  `delivered`.
- A change names at least one capability, and every `affects` slug is in `changes`.
- Outside PROPOSED blocks, text states the current contract. A statement about
  behaviour that is not built belongs in a `designed` or `considered` capability or
  inside a PROPOSED block, never in a delivered contract.
- Directories are flat: a document sits directly in its directory.
- A state does not advance on its own. The state of a capability is edited in the
  same change that earns it, and the check rules above reject a state its content
  does not support.
- Outside a git work tree, such as a copy of a commit's files, the gate takes every
  file under the directory as a tracked path, so a link to an untracked file passes
  there. `Gate::tracked_paths` supplies the list instead.

## Explanation

The pattern this model removes is the reader who has to apply amendments in their
head: the parser behaves as one decision says, except for a second decision, with an
error-handling adjustment from a third. The current documentation instead states the
complete contract that applies, and the historical sequence stays in Git, where
reading it is a deliberate investigation.

Each capability gets a permanent home because the work items that create or change it
are execution concerns, and the capability has a continuing identity. The roadmap
follows from the states: a considered or designed capability is upcoming work, an
implementing capability with unresolved bindings shows its gap, and a delivered one
is in the catalogue. A completed requirements document does not remain beside a
second, partly updated description of what was built.

Design begins in the shape of the eventual documentation. If the first design were
written as a proposal for approval, turning it into documentation would be a second
writing project, and that project is easy to neglect. With seven fixed sections from
the first draft, uncertainty is marked and resolved in place. Promotion to
`delivered` requires reconciliation, not just a status edit: the implementation may
have changed the API, a guarantee may have proved impractical, and an attractive
option may have been dropped. Those outcomes are written into the current text before
the state moves. Design material maps to outcomes in the same way: a stated need
becomes a documented guarantee with linked tests, an open question becomes an
explicit contract with a short explanation of the choice, a sketched API becomes a
real reference with a compiling example, and a possibility that was not pursued
becomes either a stated limitation or a claim with the outcome `dropped` or
`moved:change:<slug>`. A claim is never left open as an outcome. Normal links between current topics
remain useful: the reader follows the structure of the system and not its
chronology.

The graph keeps intention, implementation and documentation aligned in both
directions. It follows the discipline of the GMEOW
[Grounding Contract](https://github.com/Blackcat-Informatics/gmeow-ontology/blob/a631f73e6e39b4493d295bcd83b4deec80637f54/docs/GROUNDING.md):
a design promise needs an accounted-for realisation, and built functionality needs
documentation that follows it. Documentation maintenance becomes work the system can
identify: the graph shows which sections a broken binding or a failing example
touches, and the obligations those sections receive are defined by
[reconcile](./workplan-reconcile.md).

## Verification

- End-to-end fixtures for the Capability and Change documents: each rule of the OKF
  profile and each reader error surfaces through `check` with its file and location,
  and every kind in `Finding::KINDS` is produced by a named fixture.
- A state fixture per state: each accepted with matching content, and each rejected
  with a mismatch, such as a `delivered` capability holding a PROPOSED block or an
  open record, a `designed` capability holding a block, or a `considered` capability
  holding none.
- Marker fixtures: a block naming a missing change, a block whose change does not
  list the capability, a block whose section the change does not affect, a block in
  an Architecture document, and a marker inside a code fence that is not read as a
  block.
- Graph fixtures: every kind of node and edge, section-to-section links, each query,
  an acyclic graph that passes, cycles of one, two and three nodes that fail, and
  `depends`, `changes` and `moved` references to a missing document that fail.
- Link fixtures over every link form: a missing target, a missing anchor, a target
  outside the repository and a tracked symlink, judged against the tracked set and
  not the tree.
- Reference fixtures: issue and pull request numbers and process wording in every
  document type and in the frontmatter, beside hex colours and prose numbers that
  pass.
- Binding fixtures over a `MemoryTree` source: a symbol, a re-exported symbol, a test,
  an example target and a documented library item resolve; a missing path, an
  ambiguous path, a macro-generated symbol, a missing example target and an item of
  the wrong role fail with the capability, the list and the binding named.
- Roadmap fixtures: the group and slug order, the gaps of an `implementing`
  capability, and this repository with every capability once and each capability
  it has built either `delivered`, or `implementing` with the open change record
  named after it, with no gap in either case. The rule refuses a built capability
  that is `implementing` with no record of its name or with only a record of
  another name, one with a gap, one in another state and one that is absent, and
  accepts it `delivered` with no record or reopened under its own record.
- The dev-dependency API: `Gate::workspace(...).assert_clean()` passes on a clean
  fixture repository on disk, panics with every finding on a broken one, and is an
  error for a directory without capability documents.
- The documentation example of `Gate` writes a repository of one package and one
  delivered capability to a scratch directory and runs the gate over it with an
  empty tracked set: the bindings resolve, and the one finding is `index-missing`.
- Record fixtures in their directories: each record type loads into the model; a
  record in the directory of another type, under another file name or below its
  directory is refused; a file beside the coverage record is
  `singleton-other-file`; a repository without record directories reports nothing
  new.
- Reference fixtures for the records and the claims, each beside a neighbour that
  passes: a capability in `for` with no document, a subject with no capability, a
  `scope` crate that is no package, an assessment named otherwise than its
  subject, a claim on unknown evidence with its line, and a claim on evidence for
  another capability; a claim inside a PROPOSED block is
  not read. An input of each kind that resolves is an edge, a re-exported symbol
  leads to its definition, and an input of each kind that does not resolve is no
  edge and no finding.
- Digest fixtures: the SHA-256 test vectors (the empty message, `abc`, the 448-bit
  message and a million `a`), each encoding pinned against an independent SHA-256,
  two runs equal, CRLF and LF equal, a regenerated block that leaves a section
  digest, and each definition of a symbol counted once.
- Writer fixtures: a written record of each type reloads with no finding as the
  same value; `create` never replaces a file; `replace` needs a record file that
  exists, renames a new file over it, and in a directory that refuses a new file
  fails with the target unchanged and no temporary file left; a record that would
  not load clean, a bad slug and half an SPDX header are refused with nothing
  written.
- Link fixtures for the writer: a directory link inside the root at `work`, at
  `work/evidence`, at `work/assessments` and at `work/coverage`, with the record's
  own file behind it, is refused by `create` and by `replace`, naming the link, with
  the tree byte-identical, and the same writes through the real directory go
  through; with `work/coverage` a link to `docs` and `docs/default.md` there,
  `replace` of the coverage record is refused and the document keeps its text.
- Cost fixtures: a record at the size bound with the most measures loads, and a
  hundred thousand claims load with one finding for each citation of a record that
  does not exist. The work is counted, not timed: a section of many fenced blocks
  is read once, each character and each blanked range stepped over once, and the
  count at three sizes grows by no more than its second size did; the claims of
  many capabilities are judged against sets built once, each record entered
  once, and each slug of each claim is asked of the sets twice, a number the
  lookups of sets that count their own lookups pin exactly. A count sees the steps that are counted: a loop added
  elsewhere is not seen by it. The tallies are cfg(test)-only counters whose own
  tests pin that a count adds what it is given, that each kind has its own
  counter, that the tallies reset between measures, and that a ceiling panics at
  the step that passes it and is cleared when a measure ends, even by a panic.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `raf_workplan::Gate` | doctest | [Verification](#verification) |
| `raf_workplan::LinkContext` | doctest | [Verification](#verification) |
| `raf_workplan::bindings::tests::the_invocations_of_a_module_are_listed_once_however_many_bindings_miss` | test | [Verification](#verification) |
| `raf_workplan::digest::tests::fields_of_different_kinds_with_equal_bytes_differ` | test | [Verification](#verification) |
| `raf_workplan::digest::tests::sha256_gives_the_standard_test_vectors` | test | [Verification](#verification) |
| `raf_workplan::digest::tests::sha256_is_the_same_over_block_boundaries_and_pieces` | test | [Verification](#verification) |
| `raf_workplan::digest::tests::symbol_definitions_are_deduplicated_by_source_key_and_ordered` | test | [Verification](#verification) |
| `raf_workplan::digest::tests::the_encoding_is_unambiguous_across_names_texts_and_fields` | test | [Verification](#verification) |
| `raf_workplan::finding::tests::each_location_answers_to_exactly_one_accessor` | test | [Verification](#verification) |
| `raf_workplan::graph::tests::a_section_of_many_blocks_is_read_in_work_linear_in_its_blocks` | test | [Verification](#verification) |
| `raf_workplan::graph::tests::ten_thousand_named_paths_are_found_in_one_pass` | test | [Verification](#verification) |
| `raf_workplan::graph::tests::the_claims_of_many_capabilities_are_judged_against_sets_built_once` | test | [Verification](#verification) |
| `raf_workplan::graph::tests::the_paths_of_the_code_spans_are_the_paths_names_in_code_span_accepts` | test | [Verification](#verification) |
| `raf_workplan::plain::check` | doctest | [Contract](#contract) |
| `raf_workplan::test_crate::bindings::a_binding_through_a_re_export_resolves_to_its_definition` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_binding_to_a_macro_generated_function_is_not_found` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_binding_to_a_missing_path_is_reported_with_its_capability` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_binding_to_an_item_under_the_module_that_declares_its_file_is_not_found` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_capability_that_binds_nothing_has_no_binding_findings` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_code_binding_must_name_a_shipping_item` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_failure_with_no_name_to_find_says_a_macro_or_derive_may_be_behind_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_library_that_runs_no_documentation_tests_has_no_doctest_examples` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_macro_in_the_file_of_a_module_is_read_for_the_path_of_that_module` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_missing_example_target_is_reported_with_its_capability_list_and_binding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_name_held_only_inside_a_nested_group_of_the_arguments_is_still_named` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_path_two_globs_both_bring_in_is_ambiguous_and_names_both_definitions` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::a_tests_binding_must_name_a_function_marked_test` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::an_examples_binding_is_an_example_target_or_a_documented_library_item` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::bindings_that_name_real_symbols_resolve` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::every_finding_of_a_wrong_role_names_the_fix_for_its_list` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::every_unresolved_binding_of_every_list_is_reported` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::one_failing_site_fails_a_binding_that_resolves_to_several` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::only_a_path_before_a_bang_and_a_group_is_an_invocation` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::the_graph_keeps_each_resolution_for_the_rules_and_the_views_that_read_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::the_macro_hint_is_given_only_for_a_macro_at_the_item_level_of_the_parent_module` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::the_macro_hint_is_only_for_a_parent_that_is_a_module` | test | [Verification](#verification) |
| `raf_workplan::test_crate::bindings::the_role_is_judged_on_the_definition_a_re_export_leads_to` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::a_digest_is_the_same_on_every_run` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::a_digest_is_written_and_read_as_sixty_four_lowercase_hex_digits` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::a_module_stands_for_its_files_and_the_modules_below_it_and_a_sibling_does_not_move_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::a_path_that_names_no_single_symbol_has_no_digest` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::a_regenerated_block_leaves_the_section_digest_and_an_edit_moves_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::a_symbol_and_a_record_with_equal_bytes_differ` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::a_symbol_digest_reads_the_definitions_of_the_path` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::crlf_and_lf_give_the_same_digest` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::each_kind_hashes_its_documented_encoding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::digest::every_definition_of_a_path_counts_once_in_file_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::a_broken_document_is_a_finding_and_not_in_the_model` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::a_directory_that_cannot_be_listed_is_reported_by_name` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::a_finding_renders_with_its_place_and_its_fix` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::a_generated_index_may_carry_a_licence_header` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::a_nested_typed_document_is_nested_and_not_read_as_a_document` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::a_tree_that_cannot_be_built_is_a_finding_for_each_directory` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::accepts_a_generated_capability_index` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::accepts_a_generated_index_with_an_spdx_header` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::accepts_comments_after_scalars` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::accepts_optional_tags_and_timestamp` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::accepts_quoted_and_block_forms` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::accepts_valid_fixture` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::an_empty_repository_has_no_findings` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::an_index_is_navigation_and_not_a_stray_file` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::an_oversize_file_is_reported_by_name_and_not_skipped` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::comments_hide_text_but_not_the_text_beside_them` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::files_that_are_not_markdown_are_not_documents` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::findings_come_in_directory_then_name_order_and_repeat` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::fixture_builder_is_self_consistent` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::placeholder_scan_spares_code_and_longer_words` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::placeholder_scan_spares_longer_words_and_dotted_prose` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_capability_without_frontmatter` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_id_filename_mismatch` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_missing_section` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_nested_documents` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_placeholder_spellings` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_placeholder_tokens` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_proposed_marker_in_architecture` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_proposed_marker_in_change` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_resource_key` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_sections_left_empty_by_comments` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_type_in_the_wrong_directory` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_typed_document_outside_its_directory` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_unclosed_frontmatter` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_unknown_key` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::rejects_unparseable_frontmatter` | test | [Verification](#verification) |
| `raf_workplan::test_crate::documents::the_model_holds_typed_values_and_bodies` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_broken_repository_panics_listing_every_finding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_capabilities_directory_without_a_capability_document_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_clean_repository_passes` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_directory_that_does_not_exist_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_git_dir_that_names_no_repository_is_an_error_and_not_a_walk` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_link_to_a_file_the_tracked_set_leaves_out_is_link_broken` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_link_to_a_symlink_the_tracked_set_names_is_link_symlink` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_missing_capabilities_directory_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_missing_changes_directory_is_the_empty_set_of_changes` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_source_workspace_that_does_not_load_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::a_work_tree_whose_index_cannot_be_listed_is_an_error_and_not_a_walk` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::each_call_of_workplan_reads_the_documents_as_they_are_on_disk_then` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::inside_a_git_checkout_the_tracked_paths_come_from_the_index` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::open_gives_the_tree_the_workspace_the_tracked_paths_and_the_documents` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::open_is_the_load_of_try_check_and_refuses_what_it_refuses` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::the_gate_under_a_git_dir_that_names_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::the_programs_of_a_gate_are_started_in_its_directory` | test | [Verification](#verification) |
| `raf_workplan::test_crate::gate::the_tracked_paths_a_caller_gives_replace_the_listing_and_start_no_process` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::a_body_that_mixes_line_endings_is_refused_by_sync_and_reported_by_check` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::a_code_span_that_continues_the_bound_path_with_a_segment_does_not_substantiate` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::a_link_that_leads_nowhere_in_the_model_is_no_edge` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::bindings_record_what_the_source_index_made_of_each_path` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::capabilities_of_names_every_capability_bound_to_a_symbol` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::code_of_lists_the_items_that_realise_a_capability_in_path_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::hostile_an_anchor_only_another_document_has_is_no_edge_and_a_finding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::hostile_duplicate_ids_across_files_load_one_capability_and_report_the_other_file` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::hostile_duplicated_heading_anchors_are_numbered_and_the_next_one_is_missing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::hostile_links_in_fences_and_code_spans_are_neither_edges_nor_findings` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::hostile_self_dependency_is_one_cycle_and_one_edge` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::hostile_unterminated_backticks_name_nothing_and_break_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::linking_sections_are_section_granular` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::links_run_from_the_section_that_holds_them_to_the_section_they_name` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::sections_of_a_change_are_its_affected_sections_sorted` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::tests_for_a_binding_named_in_two_sections_substantiates_both` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::tests_for_a_binding_named_only_in_prose_substantiates_verification` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::tests_for_a_path_that_does_not_resolve_is_still_a_binding_of_its_sections` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::tests_for_ignores_a_name_inside_a_proposed_block_or_a_fence` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::the_fixture_loads_with_only_its_two_unresolved_bindings` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::the_graph_has_every_kind_of_edge` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::the_graph_has_every_kind_of_node` | test | [Verification](#verification) |
| `raf_workplan::test_crate::graph::the_queries_are_deterministic` | test | [Verification](#verification) |
| `raf_workplan::test_crate::kinds::every_finding_names_a_fix_and_a_message` | test | [Verification](#verification) |
| `raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind` | test | [Verification](#verification) |
| `raf_workplan::test_crate::kinds::every_rule_and_every_reader_error_has_its_own_kind` | test | [Verification](#verification) |
| `raf_workplan::test_crate::kinds::kinds_are_unique_kebab_case_names` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::a_directory_the_walk_cannot_list_is_reported_and_not_skipped` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::a_document_is_judged_against_the_supplied_tracked_set` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::a_fragment_link_is_judged_against_its_own_document` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::a_link_to_a_directory_resolves_and_its_anchor_is_not_read` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::a_symlink_the_caller_names_is_tracked_and_still_a_symlink` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::accepts_resolving_reference_and_attribute_links` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::an_anchor_in_a_non_markdown_file_is_not_judged` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::anchors_follow_github_rules` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::autolinks_are_external_and_not_judged` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::does_not_read_attributes_that_only_end_in_href_or_src` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::links_inside_documents_are_judged_wherever_the_document_is_valid_or_not` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::reads_an_attribute_value_that_starts_on_the_next_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::reads_attributes_with_spaces_around_the_equals_sign` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_a_link_whose_target_cannot_be_read_for_its_anchor` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_absolute_and_empty_link_targets_beside_relative_ones` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_an_empty_link_target` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_broken_anchor` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_broken_attributes_in_tags_split_across_lines` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_broken_href_and_src_attributes` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_broken_link_outside_the_document_directories` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_broken_reference_definition` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_broken_relative_link` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_broken_unquoted_attribute_values` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_link_leaving_the_repository` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::rejects_link_to_a_tracked_symlink` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::tag_scan_spares_prose_autolinks_and_closed_tags` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::tag_state_does_not_survive_a_blank_line_or_a_fence` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::the_directories_of_a_named_symlink_are_tracked_directories` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::the_oracle_is_the_tracked_set_and_not_the_tree` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::the_symlinks_a_tree_lists_are_tracked_symlinks` | test | [Verification](#verification) |
| `raf_workplan::test_crate::links::the_tree_is_walked_only_when_a_relative_link_needs_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::parse_errors::every_reader_error_has_a_fixture_reported_with_file_and_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::a_broken_link_and_a_split_link_are_reported_together_for_a_plain_file` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::a_claim_marker_outside_a_capability_is_reported_and_a_misspelt_marker_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::a_tested_by_marker_outside_a_capability_is_reported_and_a_malformed_one_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::accepts_one_line_links_and_links_in_fences_beside_the_split_rule` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::an_index_in_a_document_directory_other_than_the_capability_index_is_read_as_markdown` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::does_not_read_links_or_placeholders_inside_html_comments` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::reads_a_proposed_block_in_plain_markdown_as_outside_a_capability` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::reads_markdown_outside_the_document_directories_with_or_without_frontmatter` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::reads_placeholders_in_plain_markdown_outside_code` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::rejects_a_link_split_across_lines_in_a_profile_document` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::rejects_a_link_split_across_lines_in_any_tracked_markdown_file` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::rejects_generated_marker_errors` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::rejects_unclosed_frontmatter_in_plain_markdown` | test | [Verification](#verification) |
| `raf_workplan::test_crate::plain::reports_marker_errors_of_plain_markdown_once` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_broken_record_is_a_finding_and_not_in_the_model` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_claim_inside_a_proposed_block_is_not_a_claim_until_it_is_resolved` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_claim_on_evidence_for_another_capability_is_a_finding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_claim_on_evidence_that_covers_its_capability_is_clean_and_an_edge` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_claim_on_unknown_evidence_names_the_file_and_the_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_directory_link_inside_the_root_on_the_way_to_a_record_is_refused_and_a_real_directory_is_not` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_hundred_thousand_claims_load_with_one_finding_each` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_record_at_the_size_bound_with_the_most_measures_loads` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_record_in_the_directory_of_another_type_is_wrong_directory` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_record_that_names_what_does_not_exist_is_a_finding_and_its_neighbour_is_clean` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_record_that_would_not_load_clean_is_not_written` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_record_under_another_file_name_and_a_nested_record_are_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_reference_that_does_not_resolve_makes_no_edge` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_replacement_through_a_link_to_the_documents_never_overwrites_a_document` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_repository_without_record_directories_has_no_records_and_no_new_findings` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_symbolic_link_on_the_way_to_the_record_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::a_written_record_reloads_with_no_finding_as_the_same_value` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::claims_are_section_granular_and_one_edge_per_named_record` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::create_never_replaces_a_file` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::each_input_of_a_record_that_resolves_is_an_edge_and_one_that_does_not_is_no_finding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::each_record_type_loads_from_its_directory_into_the_model` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::replace_puts_new_text_in_place_of_an_existing_record_only` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::replace_renames_a_new_file_over_the_target_and_never_writes_into_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::the_record_directory_is_created_below_the_root_only` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::the_records_are_nodes_with_their_edges` | test | [Verification](#verification) |
| `raf_workplan::test_crate::records::work_coverage_holds_only_the_coverage_record` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::a_matching_kebab_slug_is_accepted` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::a_reference_to_a_broken_document_is_not_a_missing_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::a_reference_to_a_file_without_frontmatter_is_missing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::a_slug_id_that_differs_from_the_file_name_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::a_slug_that_is_not_kebab_case_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::accepts_a_depends_on_an_existing_capability_and_an_acyclic_graph` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::accepts_a_moved_outcome_that_names_an_existing_change` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::accepts_moved_outcome` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::every_reference_case_is_reported_exactly` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::rejects_changes_missing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::rejects_depends_cycles_once_for_each_group_naming_its_members` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::rejects_depends_missing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::rejects_moved_missing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::references::two_separate_cycles_are_two_findings_and_a_cycle_with_a_tail_is_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::refs::a_banned_line_names_why` | test | [Verification](#verification) |
| `raf_workplan::test_crate::refs::a_reference_is_reported_once_for_a_stray_page_and_a_document_that_does_not_read` | test | [Verification](#verification) |
| `raf_workplan::test_crate::refs::every_document_type_and_the_frontmatter_are_scanned` | test | [Verification](#verification) |
| `raf_workplan::test_crate::refs::issue_token_scan_spares_hex_colours` | test | [Verification](#verification) |
| `raf_workplan::test_crate::refs::rejects_acceptance_label` | test | [Verification](#verification) |
| `raf_workplan::test_crate::refs::rejects_an_issue_url_a_pull_label_and_an_issue_label_in_documents` | test | [Verification](#verification) |
| `raf_workplan::test_crate::refs::rejects_issue_number_token` | test | [Verification](#verification) |
| `raf_workplan::test_crate::refs::rejects_process_wording` | test | [Verification](#verification) |
| `raf_workplan::test_crate::refs::the_matcher_bans_references_and_process_wording` | test | [Verification](#verification) |
| `raf_workplan::test_crate::refs::the_matcher_pins_its_edge_cases` | test | [Verification](#verification) |
| `raf_workplan::test_crate::refs::the_matcher_spares_colours_prose_numbers_and_near_misses` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::a_built_capability_is_delivered_or_reopened_under_an_open_change_and_has_no_gap` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::a_fully_bound_implementing_capability_has_no_gap` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::a_resolving_binding_of_the_wrong_role_is_not_a_gap` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::an_empty_repository_has_an_empty_roadmap` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::an_entry_carries_what_the_catalogue_row_needs` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::an_implementing_capability_lists_its_gaps_exactly` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::only_an_implementing_capability_has_gaps` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::open_change_slugs_are_the_open_changes_without_their_prefix` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::open_changes_are_the_changes_that_list_the_capability` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::planned_is_designed_then_considered_and_not_a_state` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::slug_order_is_not_file_name_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::the_groups_are_in_catalogue_order_and_sorted_by_slug` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::the_open_changes_are_sorted_by_slug_whatever_the_order_of_the_files` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::the_roadmap_does_not_depend_on_insertion_order_and_repeats` | test | [Verification](#verification) |
| `raf_workplan::test_crate::roadmap::this_repository_lists_every_capability_once_in_its_state_group` | test | [Verification](#verification) |
| `raf_workplan::test_crate::rules::every_rule_has_a_fixture_reported_with_file_and_location` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::a_block_that_names_a_broken_change_is_not_a_missing_change` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::a_broken_record_does_not_make_the_capabilities_it_lists_unlisted` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::a_marker_in_a_code_fence_is_not_read_as_a_block` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::a_record_that_lists_two_capabilities_must_affect_the_section_of_each` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::accepts_each_state_with_matching_content` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_considered_without_proposed_block` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_delivered_with_open_change_record` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_delivered_with_proposed_block` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_designed_with_proposed_block` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_each_non_delivered_state_without_an_open_record` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_open_capability_without_change_record` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_orphan_proposed_marker` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_proposed_change_that_does_not_list_the_capability` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::rejects_proposed_section_missing_from_affects` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::surfaces_the_single_document_state_rules_from_the_profile` | test | [Verification](#verification) |
| `raf_workplan::test_crate::states::the_blocks_of_a_capability_that_does_not_validate_still_name_a_record` | test | [Verification](#verification) |
| `raf_workplan::test_crate::strict::a_strict_gate_judges_strict_and_the_workplans_it_opens_keep_its_floor` | test | [Verification](#verification) |
| `raf_workplan::test_crate::strict::the_record_and_the_floor_give_the_stronger_strictness` | test | [Verification](#verification) |
| `raf_workplan::test_crate::strict::the_strict_findings_are_made_by_check_after_every_other_finding_and_never_by_the_load` | test | [Verification](#verification) |
| `raf_workplan::work::tests::a_counter_increments_by_what_is_added` | test | [Verification](#verification) |
| `raf_workplan::work::tests::a_limited_kind_panics_at_its_ceiling` | test | [Verification](#verification) |
| `raf_workplan::work::tests::a_limited_measure_counts_the_other_kinds_without_a_ceiling` | test | [Verification](#verification) |
| `raf_workplan::work::tests::a_measure_that_panicked_leaves_neither_counts_nor_ceiling_behind` | test | [Verification](#verification) |
| `raf_workplan::work::tests::every_kind_has_its_own_counter` | test | [Verification](#verification) |
| `raf_workplan::work::tests::passing_the_ceiling_panics_at_once` | test | [Verification](#verification) |
| `raf_workplan::work::tests::reaching_the_ceiling_is_allowed` | test | [Verification](#verification) |
| `raf_workplan::work::tests::the_panic_comes_before_the_work_that_follows_it` | test | [Verification](#verification) |
| `raf_workplan::work::tests::the_result_of_the_closure_is_returned` | test | [Verification](#verification) |
| `raf_workplan::work::tests::the_tallies_are_reset_between_measures` | test | [Verification](#verification) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [okf-profile](./okf-profile.md) |
| Depends on | [source-index](./source-index.md) |
| Depended on by | [workplan-current-rationale](./workplan-current-rationale.md) |
| Depended on by | [workplan-entity-history](./workplan-entity-history.md) |
| Depended on by | [workplan-evidence-claims](./workplan-evidence-claims.md) |
| Depended on by | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depended on by | [workplan-prose-assessment](./workplan-prose-assessment.md) |
| Depended on by | [workplan-reconcile](./workplan-reconcile.md) |
| Depended on by | [workplan-two-way-coverage](./workplan-two-way-coverage.md) |
| Changed by | [workplan-strict](../../work/changes/workplan-strict.md) |
<!-- raf:generated:end -->
