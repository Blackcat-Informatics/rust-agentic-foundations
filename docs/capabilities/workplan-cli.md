---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-cli
title: "cargo workplan command line"
description: "The cargo workplan subcommands, options and exit codes."
state: delivered
bindings:
  code:
    - cargo_workplan::bin_crate::cargo_workplan::command::diagram
    - cargo_workplan::bin_crate::cargo_workplan::main
    - cargo_workplan::bin_crate::cargo_workplan::cli::parse
    - cargo_workplan::bin_crate::cargo_workplan::cli::Args
    - cargo_workplan::bin_crate::cargo_workplan::command::run
    - cargo_workplan::bin_crate::cargo_workplan::command::export
    - cargo_workplan::bin_crate::cargo_workplan::render
    - cargo_workplan::bin_crate::cargo_workplan::cli::Docs
    - cargo_workplan::bin_crate::cargo_workplan::cli::ExportArgs
    - cargo_workplan::bin_crate::cargo_workplan::exit::Exit
    - raf_workplan::root::discover
    - raf_workplan::root
    - cargo_workplan::bin_crate::cargo_workplan::history::history
    - cargo_workplan::bin_crate::cargo_workplan::evidence::source
    - cargo_workplan::bin_crate::cargo_workplan::evidence::record
    - cargo_workplan::bin_crate::cargo_workplan::evidence::claims
    - cargo_workplan::bin_crate::cargo_workplan::evidence::support
    - cargo_workplan::bin_crate::cargo_workplan::assess::assess
    - cargo_workplan::bin_crate::cargo_workplan::coverage::coverage
    - cargo_workplan::bin_crate::cargo_workplan::render::Report
    - raf_workplan::escape
    - cargo_workplan::bin_crate::cargo_workplan::command::LABEL
    - cargo_workplan::bin_crate::cargo_workplan::change::complete
    - cargo_workplan::bin_crate::cargo_workplan::change::completion_records
  tests:
    - cargo_workplan::test_crate::verify::host_nonapplicability_is_exit_zero_and_preserved_in_machine_output
    - cargo_workplan::test_crate::evidence::an_opposite_host_binding_is_exit_two_and_cannot_create_or_replace_evidence
    - cargo_workplan::test_crate::complete::a_completed_change_prints_what_was_removed_and_rewritten_and_leaves_a_clean_check
    - raf_workplan::exec::tests::no_shell::every_argument_is_one_element_and_no_shell_reads_it
    - raf_workplan::bound::tests::a_test_with_clean_facts_is_planned_and_its_neighbours_are_refused
    - raf_workplan::bound::tests::a_doc_test_binding_lists_the_package_and_runs_the_bare_item_path
    - raf_workplan::bound::tests::an_example_runs_under_its_cargo_name_and_the_exit_status_decides
    - cargo_workplan::test_crate::process::a_git_older_than_2_30_is_exit_two_before_any_other_process
    - cargo_workplan::test_crate::verify::verification_uses_only_tracked_path_git_calls_and_frozen_cargo_without_a_commit
    - cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms
    - raf_source::test_crate::root::member_and_virtual_workspace_directories_resolve_without_sources
    - raf_source::test_crate::root::standalone_excluded_and_nested_packages_have_their_own_roots
    - raf_source::test_crate::root::transitive_path_dependencies_are_members
    - raf_source::test_crate::root::explicit_workspace_requires_a_safe_path_and_agreed_membership
    - raf_source::test_crate::root::malformed_manifests_and_missing_directories_fail_hard
    - raf_source::test_crate::root::manifest_size_bound_is_enforced_beside_the_exact_limit
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::diagnostic_write_failure_is_propagated_and_quiet_does_not_write_notes
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::quiet_filters_results_without_changing_exit_and_machine_output_stays_a_graph
    - cargo_workplan::test_crate::rdf::check_preserves_every_finding_location_in_both_equivalent_formats
    - cargo_workplan::test_crate::rdf::quiet_keeps_findings_and_an_empty_result_is_still_a_canonical_graph
    - cargo_workplan::test_crate::rdf::status_preserves_ordered_capabilities_binding_gaps_and_open_changes
    - cargo_workplan::test_crate::rdf::preview_preserves_reconciled_text_and_sync_reports_only_actual_rewrites
    - cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading
    - raf_workplan::test_crate::root::tree_directory_aliases_fail_instead_of_selecting_lexical_ancestors
    - raf_workplan::test_crate::root::a_dangling_manifest_is_a_finding_instead_of_an_ancestor_fallback
    - raf_workplan::test_crate::root::memory_tree_membership_discovery_needs_no_disk_adapter
    - raf_workplan::test_crate::root::a_directory_outside_any_workspace_is_an_actionable_error
    - raf_workplan::test_crate::root::a_nonexistent_directory_is_refused
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::each_command_parses_in_both_shapes
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_takes_a_slug_repeated_capabilities_and_affects_and_the_spdx_pair
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_needs_a_capability_and_both_or_neither_spdx_option
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_default_and_parse_in_both_shapes
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_sync_text_options_parse
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::an_unknown_output_format_is_a_usage_error
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_missing_command_an_unknown_command_and_a_missing_slug_are_usage_errors
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_that_starts_with_a_dash_is_a_usage_error_however_it_is_given
    - cargo_workplan::bin_crate::cargo_workplan::exit::tests::the_three_exits_have_the_documented_codes
    - cargo_workplan::test_crate::check::a_clean_repository_exits_zero_and_prints_nothing
    - cargo_workplan::test_crate::check::a_repository_with_findings_exits_one_and_prints_each_finding
    - cargo_workplan::test_crate::check::the_findings_are_those_of_the_library_gate_in_the_same_order
    - cargo_workplan::test_crate::check::the_exit_agrees_with_assert_clean_and_every_finding_is_in_its_message
    - cargo_workplan::test_crate::check::usage_errors_exit_two_with_a_message_on_standard_error
    - cargo_workplan::test_crate::status::the_roadmap_groups_one_capability_in_each_state_with_the_gaps_and_the_change
    - cargo_workplan::test_crate::preview::the_proposed_text_is_printed_and_the_markers_are_not
    - cargo_workplan::test_crate::preview::an_unknown_slug_is_exit_two_and_names_the_open_changes
    - cargo_workplan::test_crate::sync::a_missing_index_with_the_text_is_created_with_that_header
    - cargo_workplan::test_crate::sync::only_the_generated_blocks_are_rewritten
    - cargo_workplan::test_crate::new::new_prints_the_library_text_writes_the_record_and_nothing_else_in_both_shapes
    - cargo_workplan::test_crate::new::the_printed_template_in_a_capability_of_each_state_gives_a_clean_check_and_one_unresolved_proposal
    - cargo_workplan::test_crate::impact::impact_prints_what_the_change_reaches_and_writes_nothing
    - cargo_workplan::test_crate::reconcile::nothing_is_written_and_no_cargo_is_started_for_a_fixture_without_bound_examples
    - cargo_workplan::test_crate::complete::from_new_to_a_completed_change_through_the_binary
    - cargo_workplan::test_crate::complete::both_shapes_complete_the_same_way
    - cargo_workplan::test_crate::verify::rdf_preserves_real_binding_outcomes_proof_counts_and_quiet_filtering
    - cargo_workplan::test_crate::process::discovery_starts_no_cargo_from_the_cargo_variable
    - cargo_workplan::test_crate::process::discovery_starts_no_cargo_on_path_when_the_variable_is_unset
    - cargo_workplan::test_crate::process::discovery_does_not_run_the_build_rustc_program_of_the_workspace
    - cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_by_every_command_before_any_process_starts
    - cargo_workplan::test_crate::process::a_missing_git_is_exit_two_and_names_it
    - cargo_workplan::test_crate::process::cargo_dispatches_workplan_to_the_binary_on_path_and_passes_the_exit_code_back
    - cargo_workplan::test_crate::root::the_root_is_found_from_the_root_and_from_any_directory_below_it
    - cargo_workplan::test_crate::root::every_command_finds_the_root_from_a_subdirectory
    - cargo_workplan::test_crate::check::a_directory_that_is_not_a_repository_exits_two_and_says_why_on_standard_error
    - cargo_workplan::test_crate::check::a_finding_is_gone_once_it_is_fixed
    - cargo_workplan::test_crate::check::a_manifest_that_does_not_parse_exits_two
    - cargo_workplan::test_crate::check::a_root_that_does_not_exist_exits_two
    - cargo_workplan::test_crate::check::a_source_file_that_cannot_be_read_exits_two
    - cargo_workplan::test_crate::check::a_stale_generated_block_is_a_finding_and_a_sync_clears_it
    - cargo_workplan::test_crate::check::help_and_version_exit_zero_on_standard_output
    - cargo_workplan::test_crate::check::quiet_keeps_the_findings_and_the_exit_code
    - cargo_workplan::test_crate::check::quiet_on_a_clean_repository_prints_nothing_and_exits_zero
    - cargo_workplan::test_crate::complete::a_capability_left_implementing_is_refused_and_nothing_is_written
    - cargo_workplan::test_crate::complete::a_stale_generated_block_is_refused_until_the_sync_is_run
    - cargo_workplan::test_crate::complete::every_failure_is_listed_and_each_condition_is_named_in_rdf
    - cargo_workplan::test_crate::complete::rdf_for_a_completion_is_a_completed_object_and_a_rewrote_object_for_each_file
    - cargo_workplan::test_crate::complete::quiet_hides_a_completion_and_keeps_a_refusal
    - cargo_workplan::test_crate::complete::what_stops_it_looking_is_exit_two_and_the_record_stays
    - cargo_workplan::test_crate::impact::a_change_that_reaches_dependants_and_linking_sections_lists_them_sorted
    - cargo_workplan::test_crate::impact::a_record_that_does_not_load_is_exit_two_with_its_findings
    - cargo_workplan::test_crate::impact::a_slug_with_no_record_is_exit_two_and_names_the_open_changes
    - cargo_workplan::test_crate::impact::impact_finds_the_root_from_below_and_by_root
    - cargo_workplan::test_crate::impact::rdf_preserves_impact_bindings_dependencies_links_and_list_order
    - cargo_workplan::test_crate::impact::quiet_prints_nothing_and_the_exit_stays_zero
    - cargo_workplan::test_crate::new::a_capability_given_twice_is_listed_once
    - cargo_workplan::test_crate::new::a_capability_that_is_missing_or_a_bad_option_is_a_usage_error_before_anything_is_read
    - cargo_workplan::test_crate::new::a_second_run_for_the_same_slug_is_exit_two_and_leaves_the_record_as_it_is
    - cargo_workplan::test_crate::new::affects_is_repeatable_and_the_record_holds_the_entries_given_in_order
    - cargo_workplan::test_crate::new::every_refusal_of_the_library_is_exit_two_with_the_tree_untouched
    - cargo_workplan::test_crate::new::rdf_preserves_created_change_fields_and_ordered_guidance
    - cargo_workplan::test_crate::new::new_works_from_a_subdirectory_and_by_root
    - cargo_workplan::test_crate::new::quiet_writes_the_record_and_prints_nothing
    - cargo_workplan::test_crate::new::spdx_text_that_is_not_one_line_is_exit_two_and_writes_nothing
    - cargo_workplan::test_crate::new::the_note_about_the_generated_blocks_is_there_when_the_record_makes_them_stale
    - cargo_workplan::test_crate::new::the_spdx_lines_are_the_two_comment_lines_of_the_records_of_this_repository
    - cargo_workplan::test_crate::new::the_template_left_in_a_state_that_refuses_it_is_the_finding_the_guidance_names
    - cargo_workplan::test_crate::preview::a_capability_the_change_lists_that_has_no_document_is_exit_two
    - cargo_workplan::test_crate::preview::a_change_record_that_did_not_load_is_exit_two_and_says_so
    - cargo_workplan::test_crate::preview::a_slug_that_is_not_a_slug_is_a_usage_error
    - cargo_workplan::test_crate::preview::quiet_prints_nothing_because_a_preview_is_not_a_finding
    - cargo_workplan::test_crate::preview::the_output_is_the_text_the_library_previews_under_a_header_for_each_document
    - cargo_workplan::test_crate::preview::the_preview_is_the_same_by_root_from_another_directory
    - cargo_workplan::test_crate::process::discovery_succeeds_when_cargo_cannot_be_started
    - cargo_workplan::test_crate::process::a_slug_that_is_not_a_slug_is_refused_before_any_process_starts
    - cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_before_any_process_starts
    - cargo_workplan::test_crate::process::a_usage_error_of_the_change_commands_starts_no_process
    - cargo_workplan::test_crate::process::a_usage_error_starts_no_process
    - cargo_workplan::test_crate::process::root_starts_no_cargo_at_all
    - cargo_workplan::test_crate::reconcile::a_bound_example_that_fails_is_an_obligation_and_nothing_is_written_to_the_tree
    - cargo_workplan::test_crate::reconcile::everything_that_stops_it_looking_is_exit_two
    - cargo_workplan::test_crate::reconcile::rdf_preserves_each_obligation_and_optional_location_in_library_order
    - cargo_workplan::test_crate::reconcile::quiet_keeps_the_obligations_and_hides_nothing_that_is_a_finding
    - cargo_workplan::test_crate::root::a_directory_outside_any_workspace_is_exit_two_and_names_the_command
    - cargo_workplan::test_crate::root::a_relative_root_is_taken_from_the_current_directory
    - cargo_workplan::test_crate::root::root_overrides_the_discovery_and_the_current_directory_is_not_read
    - cargo_workplan::test_crate::root::sync_from_a_subdirectory_writes_at_the_root_and_nowhere_else
    - cargo_workplan::test_crate::status::a_directory_that_is_not_a_repository_is_exit_two
    - cargo_workplan::test_crate::status::a_state_with_no_capability_is_a_row_with_a_count_of_zero
    - cargo_workplan::test_crate::status::an_implementing_capability_with_every_binding_resolved_has_no_gap_beyond_its_empty_lists
    - cargo_workplan::test_crate::status::quiet_leaves_standard_output_empty_because_the_roadmap_has_no_finding
    - cargo_workplan::test_crate::status::the_roadmap_is_the_same_by_root_from_another_directory
    - cargo_workplan::test_crate::sync::a_document_that_does_not_load_is_exit_two_and_nothing_is_written
    - cargo_workplan::test_crate::sync::a_missing_index_without_the_text_is_exit_two_naming_the_field_and_writes_nothing
    - cargo_workplan::test_crate::sync::a_refused_file_is_exit_two_and_nothing_is_written
    - cargo_workplan::test_crate::sync::a_second_sync_writes_nothing_and_prints_nothing
    - cargo_workplan::test_crate::sync::a_value_that_cannot_be_written_into_the_header_is_refused_when_the_index_is_created
    - cargo_workplan::test_crate::sync::each_field_that_is_not_given_is_named
    - cargo_workplan::test_crate::sync::quiet_hides_the_files_rewritten_and_keeps_the_exit_code
    - cargo_workplan::test_crate::sync::the_index_is_created_by_root_from_another_directory
    - cargo_workplan::test_crate::sync::the_text_is_not_read_when_the_index_exists
    - cargo_workplan::test_crate::verify::a_failing_example_and_a_binding_that_proves_nothing_are_failures
    - cargo_workplan::test_crate::verify::a_hostile_binding_is_a_failure_and_never_reaches_cargo
    - cargo_workplan::test_crate::verify::a_slug_with_no_record_and_a_document_that_does_not_load_are_exit_two
    - cargo_workplan::test_crate::verify::every_binding_passing_exits_zero
    - cargo_workplan::test_crate::verify::no_slug_runs_every_capability_and_a_slug_only_the_ones_the_change_lists
    - cargo_workplan::test_crate::verify::nothing_to_run_is_exit_zero_with_a_note_that_quiet_hides
    - cargo_workplan::test_crate::verify::quiet_prints_only_the_failing_bindings_and_keeps_the_exit
    - raf_workplan::test_crate::root::the_root_is_found_from_the_root_a_member_and_a_subdirectory
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_command_that_takes_a_slug_refuses_one_that_starts_with_a_dash_in_both_forms
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_must_be_a_slug
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::help_and_version_exit_zero_in_both_shapes
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_may_follow_the_command
    - cargo_workplan::bin_crate::cargo_workplan::exit::tests::a_library_error_becomes_its_display_text
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::assess_takes_a_section_and_exactly_one_mode
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_takes_a_direction_and_an_optional_package
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_claims_takes_an_optional_capability
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_lines_takes_an_optional_package_and_reverse_is_the_same_command
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::evidence_claims_and_for_take_one_id_each
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::evidence_record_reads_every_option_and_the_command_after_the_separator
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::evidence_record_refuses_a_separator_as_a_value_and_a_record_it_cannot_write
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::history_refuses_an_id_or_a_day_before_anything_runs
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::history_takes_an_id_of_each_kind_and_a_day
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::options_after_the_separator_belong_to_the_command
    - cargo_workplan::bin_crate::cargo_workplan::evidence::tests::the_command_runs_only_when_both_readings_of_the_line_agree
    - cargo_workplan::test_crate::assess::a_section_that_cannot_be_assessed_is_exit_two_and_writes_nothing
    - cargo_workplan::test_crate::assess::a_verdict_writes_exactly_one_record_and_a_rejection_still_exits_zero
    - cargo_workplan::test_crate::assess::the_context_is_the_same_whoever_asks_and_writes_nothing
    - cargo_workplan::test_crate::assess::the_list_shows_each_assessment_current_until_an_input_moves
    - cargo_workplan::test_crate::coverage::an_exclusion_takes_a_gap_out_and_a_report_without_gaps_exits_zero
    - cargo_workplan::test_crate::coverage::crate_selects_a_package_by_either_name_and_refuses_one_that_is_not_there
    - cargo_workplan::test_crate::coverage::gaps_in_either_direction_are_printed_and_exit_one
    - cargo_workplan::test_crate::coverage::this_repository_has_no_code_or_test_gaps_in_its_scoped_packages
    - cargo_workplan::test_crate::claims::every_claim_names_a_test_prints_the_report_and_exits_zero
    - cargo_workplan::test_crate::claims::a_claim_that_names_no_test_is_a_finding_and_exits_one_in_text_and_rdf_alike
    - cargo_workplan::test_crate::claims::a_capability_that_is_not_loaded_exits_two_and_says_why
    - cargo_workplan::test_crate::lines::a_repository_whose_every_counted_line_is_covered_prints_the_report_and_exits_zero
    - cargo_workplan::test_crate::lines::an_orphan_line_is_a_finding_and_exits_one_in_text_and_rdf_alike_and_reverse_prints_the_same
    - cargo_workplan::test_crate::lines::a_package_that_is_not_one_exits_two_and_says_why
    - cargo_workplan::test_crate::lines::a_ceiling_row_and_its_class_decide_the_exit_of_the_default_check_on_the_binary
    - cargo_workplan::test_crate::lines::an_exception_that_matches_nothing_fails_check_under_default_and_its_matched_neighbour_passes
    - cargo_workplan::test_crate::lines::an_exception_is_the_excepted_table_and_a_lines_excepted_object_and_the_report_exits_zero
    - cargo_workplan::test_crate::evidence::a_bound_test_gives_the_verdict_and_a_path_that_names_none_starts_nothing
    - cargo_workplan::test_crate::evidence::a_claim_on_a_recorded_result_reconciles_clean_until_its_bound_code_moves
    - cargo_workplan::test_crate::evidence::a_command_that_passes_writes_exactly_one_record_and_exits_zero
    - cargo_workplan::test_crate::evidence::a_failing_command_or_a_value_over_its_limit_records_fail_and_exits_one
    - cargo_workplan::test_crate::evidence::a_request_that_cannot_be_recorded_runs_nothing
    - cargo_workplan::test_crate::evidence::cargo_passes_the_command_after_the_separator_to_the_binary_unchanged
    - cargo_workplan::test_crate::evidence::rdf_is_one_evidence_recorded_object_with_its_fields
    - cargo_workplan::test_crate::evidence::output_the_tool_cannot_read_is_exit_two_and_writes_no_record
    - cargo_workplan::test_crate::evidence::the_views_print_rdf_and_refuse_what_does_not_exist
    - cargo_workplan::test_crate::evidence::claims_exit_two_naming_a_capability_that_does_not_load_and_answer_beside_other_records
    - cargo_workplan::test_crate::evidence::a_claimed_statement_reaches_the_terminal_escaped_and_rdf_escapes_it_as_rdf
    - cargo_workplan::test_crate::assess::the_text_of_a_section_reaches_the_terminal_escaped_and_rdf_escapes_it_as_rdf
    - raf_workplan::escape::tests::each_control_and_bidirectional_formatting_character_is_escaped
    - raf_workplan::escape::tests::lines_keep_their_line_feeds_and_tabs_and_lose_every_other_control
    - cargo_workplan::test_crate::evidence::the_words_after_the_separator_are_the_command_options_included
    - cargo_workplan::test_crate::history::a_capability_lists_its_document_and_symbol_commits_newest_first_and_a_removed_binding
    - cargo_workplan::test_crate::history::a_shallow_clone_is_exit_two_and_its_full_clone_answers
    - cargo_workplan::test_crate::history::rdf_preserves_full_history_commits_sources_paths_and_removed_bindings
    - cargo_workplan::test_crate::history::quiet_prints_nothing_because_a_history_is_not_a_finding
    - cargo_workplan::test_crate::history::since_limits_the_commits_and_a_change_or_an_unknown_id_is_answered_or_refused
    - cargo_workplan::test_crate::process::an_id_a_day_or_a_value_of_the_record_commands_is_refused_before_any_process_starts
    - cargo_workplan::test_crate::process::history_asks_the_git_version_before_its_log_and_refuses_a_git_older_than_2_30
    - cargo_workplan::bin_crate::cargo_workplan::evidence::tests::a_program_named_the_separator_and_a_binding_beside_one_are_refused
    - cargo_workplan::test_crate::coverage::a_surface_that_cannot_be_listed_is_a_finding_of_the_code_report_only
    - cargo_workplan::test_crate::evidence::a_replacement_holds_the_spdx_lines_of_its_own_run_only
    - cargo_workplan::test_crate::history::a_bound_file_that_is_not_committed_is_exit_two_naming_it_until_it_is
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_removed_policy_flag_is_rejected_before_and_after_the_command
    - cargo_workplan::test_crate::check::the_default_check_accepts_the_complete_fixture
    - cargo_workplan::test_crate::check::the_default_check_refuses_a_thin_section_without_a_policy_flag
    - cargo_workplan::test_crate::check::the_default_check_without_capability_documents_is_a_run_failure
    - cargo_workplan::test_crate::check::a_removed_policy_key_is_refused_beside_the_default_content_refusal
    - cargo_workplan::test_crate::check::policy_findings_keep_typed_locations_and_messages_in_rdf
    - cargo_workplan::test_crate::complete::the_default_completion_refuses_an_underexplained_section_and_accepts_its_neighbor
    - cargo_workplan::test_crate::debt::the_write_creates_the_record_exits_zero_and_the_repository_then_passes
    - cargo_workplan::test_crate::debt::a_second_write_is_unchanged_and_the_bytes_are_the_same
    - cargo_workplan::test_crate::debt::the_written_record_in_rdf_is_one_object_with_its_members_in_order
    - cargo_workplan::test_crate::debt::debt_that_grew_is_exit_one_with_its_findings_and_nothing_is_written
    - cargo_workplan::test_crate::debt::paid_debt_fails_the_check_with_debt_resolved_until_the_write_lowers_the_record
    - cargo_workplan::test_crate::debt::a_finding_of_the_default_rules_stops_the_write_with_exit_one
    - cargo_workplan::test_crate::debt::the_default_policy_can_record_a_current_structural_finding
    - cargo_workplan::test_crate::debt::orphan_lines_over_the_ceiling_are_recorded_per_item
    - cargo_workplan::test_crate::debt::the_spdx_lines_are_given_together_and_kept_when_the_record_is_lowered
    - cargo_workplan::test_crate::debt::a_command_misused_is_a_usage_error_before_any_work
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::debt_takes_write_and_the_spdx_pair_together_or_not_at_all
    - cargo_workplan::test_crate::evidence::recorded_binding_failures_keep_failed_and_unresolved_outcomes_distinct
    - cargo_workplan::test_crate::assess::moved_inputs_preserve_raw_unresolved_reasons_only_for_unresolved_changes
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::diagnostic_flush_failure_propagates_and_quiet_never_touches_that_sink
    - cargo_workplan::test_crate::process::help_and_version_output_failures_exit_two_beside_successful_output
    - raf_workplan::test_crate::verify::reconcile_runs_trusted_examples_that_can_write_workspace_files
    - raf_workplan::test_crate::history::a_partial_clone_history_can_fetch_git_objects_without_writing_working_files_or_index
  examples:
    - raf_workplan::Workplan::sync
    - raf_workplan::Workplan::preview
    - raf_workplan::Workplan::new_change
    - raf_workplan::Workplan::impact
    - raf_workplan::escape::shown_lines
depends: [cap:workplan-complete-gate, cap:workplan-reconcile, cap:workplan-generated-docs]
---

# cargo workplan command line

## Purpose

`cargo workplan` is the interactive surface of the workplan. The library checks
everything in a test; the binary is how a person or an agent opens a change, reads
what the documentation owes, and completes the work.

## Contract

The diagram command supports every automatic family through the same native
bundle API. Either select `--view cap:<slug>#<name>` or
`--view arch:<slug>#<name>`, or supply `--name`, `--kind` and `--scope` with optional
`--focus`, `--grouping` and `--relation` filters. The two forms conflict.
Named selections reuse the documentation bundle. Unknown selections and native
render failures return no artifact. Quiet mode remains refused before loading.
Documentation previews include generated edits for both capability and
architecture owners. Each machine entry identifies its owner with `subject`,
alongside the document path and complete preview text.
<!-- raf:tested-by cargo_workplan::test_crate::evidence::the_views_print_rdf_and_refuse_what_does_not_exist -->


`diagram --name <slug> --kind evidence-claims --scope <evidence-id>` computes one
checked library `DiagramBundle`. The default text output uses native semantic
accessibility descriptions with terminal controls escaped; `--format svg` emits
accessible SVG with its complete native export, and `--format json` emits the named-view metadata and that same
export. Turtle and N-Triples carry the bundle's exact semantic dataset. Invalid
names, kinds and syntactically malformed scope ids refuse before repository loading.
Unknown selections are resolved through the loaded semantic model. The command refuses
quiet mode; SVG and JSON formats refuse for every other command. Both invocation
shapes use the same library API, leave repository inputs unchanged and propagate
selection, rendering and output failures.
<!-- raf:tested-by cargo_workplan::test_crate::evidence::the_views_print_rdf_and_refuse_what_does_not_exist,cargo_workplan::test_crate::evidence::a_claimed_statement_reaches_the_terminal_escaped_and_rdf_escapes_it_as_rdf -->

`change complete` executes bound verification in a checked completed repository
view prepared by its runner. Machine completion and refusal results retain
ordered binding receipts for processes that actually ran, including explicit
materialized context and original tracked-index provenance. Native receipts name
the `bwrap` filesystem backend, logical source root, physical backing copy,
isolated Git top and original and owned Git-selector paths. Ordinary text summaries and quiet selection remain unchanged. Preparation, revalidation
or cleanup errors refuse publication with their native causes.
<!-- raf:tested-by cargo_workplan::test_crate::complete::both_shapes_complete_the_same_way -->

### Invocation forms
<!-- raf:tested-by cargo_workplan::test_crate::process::cargo_dispatches_workplan_to_the_binary_on_path_and_passes_the_exit_code_back,cargo_workplan::test_crate::check::the_findings_are_those_of_the_library_gate_in_the_same_order,cargo_workplan::test_crate::root::the_root_is_found_from_the_root_and_from_any_directory_below_it -->

The binary is `cargo-workplan`, a thin wrapper over `raf-workplan`, parsed with clap
derive. It runs from any directory inside the workspace and finds the root from
ancestor manifests. Cargo runs it for `cargo workplan <command>`, passing the argument
vector `cargo-workplan workplan <command>`, and it runs directly as
`cargo-workplan <command>`; both forms take the same arguments and give the same
result.

### Root discovery reads
<!-- raf:tested-by cargo_workplan::test_crate::process::discovery_starts_no_cargo_from_the_cargo_variable,cargo_workplan::test_crate::process::discovery_starts_no_cargo_on_path_when_the_variable_is_unset,cargo_workplan::test_crate::process::discovery_does_not_run_the_build_rustc_program_of_the_workspace,raf_source::test_crate::root::manifest_size_bound_is_enforced_beside_the_exact_limit -->

When `--root` is absent, `raf-workplan` opens a bounded
`DiskTree` from the filesystem root and calls `raf_source::discover_root` from
the canonical current directory. Ancestor discovery uses Tree operations without
invoking Cargo or the workspace-configured Rust compiler, reading Cargo
configuration, resolving registry dependencies or changing the lockfile. Opening
the discovered repository may separately use Git to list tracked paths. A manifest is bounded by the source
reader's default file-size limit.

### Workspace membership
<!-- raf:tested-by raf_source::test_crate::root::member_and_virtual_workspace_directories_resolve_without_sources,raf_source::test_crate::root::standalone_excluded_and_nested_packages_have_their_own_roots,raf_source::test_crate::root::transitive_path_dependencies_are_members,raf_source::test_crate::root::explicit_workspace_requires_a_safe_path_and_agreed_membership,raf_source::test_crate::root::malformed_manifests_and_missing_directories_fail_hard,cargo_workplan::test_crate::root::the_root_is_found_from_the_root_and_from_any_directory_below_it -->

The nearest explicit `[workspace]` is selected. A package must belong to its
member set, including transitive in-workspace path dependencies; an unlisted
package is a finding with instructions to list it, exclude it, or add its own
`[workspace]`. A standalone package and an excluded package resolve to their
own directories. A nested workspace resolves to the nested root. An explicit
`package.workspace` must name a workspace reachable inside the tree whose
membership includes the package; missing, conflicting or malformed declarations
fail hard. A directory with no ancestor package or workspace is an error.

### Explicit roots
<!-- raf:tested-by cargo_workplan::test_crate::root::root_overrides_the_discovery_and_the_current_directory_is_not_read,cargo_workplan::test_crate::root::a_relative_root_is_taken_from_the_current_directory -->

`--root` directly selects a directory and bypasses discovery.

### Command dispatch
<!-- raf:tested-by cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms -->

| Command | Effect |
| --- | --- |
| `check` | Run every rule of the [document model](./workplan-document-model.md); print each violation |
| `status` | Print the roadmap: the capabilities of each state, the gaps of each implementing one, and the open changes |
| `docs sync` | Rewrite the generated blocks ([generated documents](./workplan-generated-docs.md)) |
| `docs preview <slug>` | Print reconciled capabilities and generated capability or architecture edits |
| `docs export <path>` | Export one validated authoring document as its native RDF dataset; requires an explicit RDF format and refuses quiet mode |
| `change new <slug> --capability cap:<name>` | Create a change record |
| `change impact <slug>` | Print what the change touches |
| `change reconcile <slug>` | Print the obligation list ([reconcile](./workplan-reconcile.md)) |
| `change complete <slug>` | Run the [completion gate](./workplan-complete-gate.md) and delete the record |
| `verify [<slug>]` | Run the bound tests and examples |
| `history <id> [--since <YYYY-MM-DD>]` | Print the commits that shaped a document or record ([entity history](./workplan-entity-history.md)) |
| `evidence record --id <slug> --for cap:<slug>... --by human\|agent [--limit <name>=<number>]... [--revision <commit>] [--update] [--copyright <text> --licence <id>]` and `-- <program> <arg>...` or `--from-binding <path>` | Run the command or the bound test and write its evidence record ([evidence claims](./workplan-evidence-claims.md)) |
| `evidence claims <evidence-id>` | Print the statements that rest on an evidence record |
| `evidence for cap:<slug>` | Print the evidence records that support a capability |
| `diagram --name <slug> --kind evidence-claims --scope <evidence-id>` | Emit a checked named evidence-claim diagram bundle |
| `assess cap:<slug>#<section> --context` | Print what an assessor reads ([prose assessment](./workplan-prose-assessment.md)) |
| `assess cap:<slug>#<section> --verdict accepted\|rejected --by human\|agent [--copyright <text> --licence <id>]` | Record the assessment of the section |
| `assess [cap:<slug>#<section>] --list` | Print each assessment, or the one of the section, and whether it is current |
| `coverage code\|tests [--crate <package>]` | Print the public items or the tests that no capability binds ([two-way coverage](./workplan-two-way-coverage.md)) |
| `coverage claims [--capability cap:<slug>]` | Print every claim of every delivered capability, or of the one named, with its key, line and tests ([the complete policy](./workplan-strict.md)) |
| `coverage lines [--crate <package>]`, also `coverage --reverse [--crate <package>]` | Print the reverse coverage of every package, or of the one named: its counted, covered, excepted and orphan lines and ceilings, the modules that hold an orphan line and the orphan lines of each unit ([the complete policy](./workplan-strict.md)) |

`status` groups capabilities by state, lists for each `implementing` capability the
bindings that are missing or do not resolve, as gaps, and names the open changes. `docs preview` applies the
PROPOSED blocks of the named change to a copy of the text, removes the markers,
and prints the result; it never writes a file. `docs sync` takes
`--copyright <text>` and `--licence <id>`, the two SPDX lines of a new capability
index; they are read only when `docs/capabilities/index.md` must be created, and then
a missing one is an error that names it. `change new` takes `--capability` once or
more, `--affects cap:<name>#<section>` any number of times, and `--copyright` with
`--licence`, as [reconcile](./workplan-reconcile.md) states, and prints the PROPOSED
block to place and the state each listed capability needs. `change complete` prints
`completed: removed <path>` and a `rewrote <path>` line for each file whose generated
blocks the removal rewrote, or `refused:` with the count of each kind of failure and a
line for each failure that no other line repeats.

### Selected authoring document export
<!-- raf:tested-by cargo_workplan::test_crate::rdf::preview_preserves_reconciled_text_and_sync_reports_only_actual_rewrites,cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading,cargo_workplan::test_crate::process::a_usage_error_starts_no_process,cargo_workplan::test_crate::process::help_and_version_output_failures_exit_two_beside_successful_output -->

`docs export <path>` reads one repository-relative authoring document through
the contained `DiskTree` reader. It calls `Document::parse` and
`Document::to_dataset(path)` for any of the seven validated RAF types, retaining
the stable asserted document IRI, scalar styles, comments, ordered repeated
values and body bytes. It emits that native dataset directly through the native
canonical Turtle or N-Triples serializer. It has no snapshot Report envelope.
The [authoring codec](./okf-profile.md#native-authoring-dataset) owns its closed
shape and resource limits; the shared report profile remains unchanged.

An explicit `--format turtle` or `--format ntriples` is required. Text, the default
format and quiet mode are errors before root or input loading. Missing, escaping,
oversized, malformed or profile-invalid inputs fail with their cause; navigation
`Index` is refused explicitly. Input and serialization failures emit no dataset.
Serialization finishes before stdout is written. A physical output failure can
leave a prefix and exits 2. A successful export exits 0. The command writes no
repository file and starts no process. It exports one authoring model; semantic
relationships across the repository, navigation and stored reviews have their
own contracts.

### Record argument values
<!-- raf:tested-by cargo_workplan::bin_crate::cargo_workplan::cli::tests::history_takes_an_id_of_each_kind_and_a_day,cargo_workplan::bin_crate::cargo_workplan::cli::tests::history_refuses_an_id_or_a_day_before_anything_runs,cargo_workplan::bin_crate::cargo_workplan::cli::tests::evidence_record_reads_every_option_and_the_command_after_the_separator,cargo_workplan::bin_crate::cargo_workplan::cli::tests::evidence_record_refuses_a_separator_as_a_value_and_a_record_it_cannot_write,cargo_workplan::bin_crate::cargo_workplan::cli::tests::assess_takes_a_section_and_exactly_one_mode,cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_takes_a_direction_and_an_optional_package,cargo_workplan::test_crate::process::an_id_a_day_or_a_value_of_the_record_commands_is_refused_before_any_process_starts -->

Each value is checked when the arguments are
parsed. A history id is `cap:`,
`change:`, `arch:`, `evidence:` or `assessment:` and a slug; an evidence id is
`evidence:<slug>` or the slug; a section is `cap:<slug>#<section>` with one of the
seven sections; `--since` is a day from 1970-01-01 to 2099-12-31; `--limit` is
`name=number` with no exponent; `--revision` is 40 or 64 lowercase hexadecimal
digits; `--crate` is a package name, given as cargo names it (`raf-okf`) or as its
crate is named (`raf_okf`). A value that is not one, or that starts with `-`, is a
usage error, exit 2, before any process starts.

### Evidence command arguments
<!-- raf:tested-by cargo_workplan::test_crate::evidence::the_words_after_the_separator_are_the_command_options_included,cargo_workplan::test_crate::evidence::cargo_passes_the_command_after_the_separator_to_the_binary_unchanged,cargo_workplan::bin_crate::cargo_workplan::evidence::tests::the_command_runs_only_when_both_readings_of_the_line_agree,cargo_workplan::test_crate::process::an_id_a_day_or_a_value_of_the_record_commands_is_refused_before_any_process_starts,cargo_workplan::bin_crate::cargo_workplan::cli::tests::evidence_record_refuses_a_separator_as_a_value_and_a_record_it_cannot_write -->

Every word after the first `--` of the
command line is the command: a `-q`, a `--format` or a second `--` after it is a
word of the command and not an option of the tool. `raf-workplan` builds the
command from the command line of the process
(`TrustedArgv::from_command_line(std::env::args_os())`) and never from the text of
a document or record; the parser reads the same words as the trailing arguments,
and a run whose two readings differ is refused before anything starts. An option
whose value would be `--` is a usage error, so no option can move the start of the
command, and a process whose program name is `--` is refused with exit 2 before
anything starts. A command and `--from-binding` exclude each other, a bare `--`
beside `--from-binding` included, and one of them is required. `--by` is required,
since a record states who made it.

### Replacement metadata
<!-- raf:tested-by cargo_workplan::test_crate::evidence::a_replacement_holds_the_spdx_lines_of_its_own_run_only,cargo_workplan::test_crate::assess::a_verdict_writes_exactly_one_record_and_a_rejection_still_exits_zero -->

A replacement, by `--update` or by a verdict on a section that has an
assessment, holds only what its own run gives, its SPDX lines included: without
`--copyright` and `--licence` the new record has none.

### Exit classes
<!-- raf:tested-by cargo_workplan::test_crate::check::a_clean_repository_exits_zero_and_prints_nothing,cargo_workplan::test_crate::check::a_repository_with_findings_exits_one_and_prints_each_finding,cargo_workplan::test_crate::check::a_directory_that_is_not_a_repository_exits_two_and_says_why_on_standard_error,cargo_workplan::test_crate::complete::both_shapes_complete_the_same_way,cargo_workplan::test_crate::complete::a_capability_left_implementing_is_refused_and_nothing_is_written,cargo_workplan::test_crate::verify::every_binding_passing_exits_zero,cargo_workplan::test_crate::verify::a_failing_example_and_a_binding_that_proves_nothing_are_failures,cargo_workplan::test_crate::process::a_missing_git_is_exit_two_and_names_it,cargo_workplan::test_crate::process::a_git_older_than_2_30_is_exit_two_before_any_other_process,cargo_workplan::test_crate::reconcile::everything_that_stops_it_looking_is_exit_two,cargo_workplan::test_crate::status::the_roadmap_groups_one_capability_in_each_state_with_the_gaps_and_the_change -->

0 means success: no violations, an empty obligation list, a completed
change, or every binding passing or explicitly excluded by its compiler-observed
platform guard. 1 means findings: violations, obligations, a refused
completion or a failing binding. 2 means the tool could not run: a usage error, an
unreadable file, a missing `git`, a slug that does not exist. A Git older than 2.30 is
exit 2 for `change reconcile` and `change complete`, a change record that does not
load (an `outcome` that is not one of the three values, for instance) is exit 2, and
`status` exits 0 whatever gaps it lists, since a gap is not a violation.
<!-- raf:tested-by cargo_workplan::test_crate::verify::host_nonapplicability_is_exit_zero_and_preserved_in_machine_output -->

### Record view exits
<!-- raf:tested-by cargo_workplan::test_crate::history::a_capability_lists_its_document_and_symbol_commits_newest_first_and_a_removed_binding,cargo_workplan::test_crate::evidence::the_views_print_rdf_and_refuse_what_does_not_exist,cargo_workplan::test_crate::assess::a_verdict_writes_exactly_one_record_and_a_rejection_still_exits_zero,cargo_workplan::test_crate::assess::the_list_shows_each_assessment_current_until_an_input_moves -->

`history`, `evidence claims`,
`evidence for` and `assess` exit 0 when they have an answer: a recorded `rejected`
verdict, and a stale or rejected assessment that `--list` shows, are results and
not findings, and `change reconcile` lists what they owe.

### Evidence recording exits
<!-- raf:tested-by cargo_workplan::test_crate::evidence::a_command_that_passes_writes_exactly_one_record_and_exits_zero,cargo_workplan::test_crate::evidence::a_failing_command_or_a_value_over_its_limit_records_fail_and_exits_one,cargo_workplan::test_crate::evidence::output_the_tool_cannot_read_is_exit_two_and_writes_no_record,cargo_workplan::test_crate::evidence::a_request_that_cannot_be_recorded_runs_nothing -->

`evidence record` exits 0
when it writes a `pass` record and 1 when it writes a `fail` record, a result the
repository owes; it exits 2 when it writes none: a printed value that is not a
number, a name printed twice, a limit the command never printed, output past its
bound, a command that cannot start, or a request refused before the run.
### Coverage report exits
<!-- raf:tested-by cargo_workplan::test_crate::coverage::gaps_in_either_direction_are_printed_and_exit_one,cargo_workplan::test_crate::coverage::an_exclusion_takes_a_gap_out_and_a_report_without_gaps_exits_zero,cargo_workplan::test_crate::coverage::a_surface_that_cannot_be_listed_is_a_finding_of_the_code_report_only,cargo_workplan::test_crate::claims::every_claim_names_a_test_prints_the_report_and_exits_zero,cargo_workplan::test_crate::claims::a_claim_that_names_no_test_is_a_finding_and_exits_one_in_text_and_rdf_alike,cargo_workplan::test_crate::lines::a_repository_whose_every_counted_line_is_covered_prints_the_report_and_exits_zero,cargo_workplan::test_crate::lines::an_orphan_line_is_a_finding_and_exits_one_in_text_and_rdf_alike_and_reverse_prints_the_same -->

`coverage code` and `coverage tests` exit 1 when the report names a gap, or in the
code direction a surface that cannot be listed, and 0 otherwise. `coverage claims`
exits 1 when a claim of a delivered capability has no test and 0 when every claim
has one. `coverage lines` exits 1 when it prints an orphan
line or could not count the lines and 0 otherwise, and `coverage --reverse` is the
same command, which prints the same bytes.

### Record lookup failures
<!-- raf:tested-by cargo_workplan::test_crate::history::since_limits_the_commits_and_a_change_or_an_unknown_id_is_answered_or_refused,cargo_workplan::test_crate::history::a_shallow_clone_is_exit_two_and_its_full_clone_answers,cargo_workplan::test_crate::history::a_bound_file_that_is_not_committed_is_exit_two_naming_it_until_it_is,cargo_workplan::test_crate::process::history_asks_the_git_version_before_its_log_and_refuses_a_git_older_than_2_30,cargo_workplan::test_crate::coverage::crate_selects_a_package_by_either_name_and_refuses_one_that_is_not_there,cargo_workplan::test_crate::claims::a_capability_that_is_not_loaded_exits_two_and_says_why,cargo_workplan::test_crate::lines::a_package_that_is_not_one_exits_two_and_says_why,cargo_workplan::test_crate::evidence::claims_exit_two_naming_a_capability_that_does_not_load_and_answer_beside_other_records,cargo_workplan::test_crate::assess::a_section_that_cannot_be_assessed_is_exit_two_and_writes_nothing -->

Each exits 2 when
the tool cannot run: an id or a capability that no loaded document has, a shallow
clone, a Git older than 2.30 or a bound file whose lines are not committed for
`history`, a package that `--crate` does not name, a capability that `--capability`
of `coverage claims` names and no loaded document has. `evidence claims` also exits 2,
naming each capability document that exists and does not load, with nothing on
standard output, because a claim in such a document may name the record; a change
record or an evidence record that does not load holds no claim and leaves its
answer and exit 0 as they are. A section given to `assess --list` must be one that
`assess --context` answers for, so a named section that can no longer be assessed
(its capability gone or not loading, or a bound symbol it reads no longer
resolving, for instance) is exit 2, while a bare `--list` still shows an assessment
of it.

### Output and policy options
<!-- raf:tested-by cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_default_and_parse_in_both_shapes,cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_may_follow_the_command,cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_removed_policy_flag_is_rejected_before_and_after_the_command,cargo_workplan::test_crate::check::the_default_check_refuses_a_thin_section_without_a_policy_flag,cargo_workplan::test_crate::complete::the_default_completion_refuses_an_underexplained_section_and_accepts_its_neighbor,cargo_workplan::bin_crate::cargo_workplan::render::tests::quiet_filters_results_without_changing_exit_and_machine_output_stays_a_graph,cargo_workplan::test_crate::preview::quiet_prints_nothing_because_a_preview_is_not_a_finding,cargo_workplan::test_crate::verify::quiet_prints_only_the_failing_bindings_and_keeps_the_exit -->

`--root <dir>` sets the workspace root, `--format text|turtle|ntriples|svg|json` selects the
output, and `-q` suppresses everything but findings. `docs export` requires an RDF
format and refuses `-q`. The RDF view of `evidence claims` also refuses `-q`
before repository loading; its ordinary text view retains quiet suppression.
SVG and JSON are accepted only by `diagram`, which refuses quiet mode in every format.
Every check, reconciliation
and completion uses the complete [workplan policy](./workplan-strict.md). The
removed `--strict` option is a usage error before or after a command. Findings
are printed to stdout in
a stable order and diagnostics to stderr. Each option is accepted before or after the
command.

### Quiet command outcomes
<!-- raf:tested-by cargo_workplan::test_crate::status::quiet_leaves_standard_output_empty_because_the_roadmap_has_no_finding,cargo_workplan::test_crate::sync::quiet_hides_the_files_rewritten_and_keeps_the_exit_code,cargo_workplan::test_crate::new::quiet_writes_the_record_and_prints_nothing,cargo_workplan::test_crate::impact::quiet_prints_nothing_and_the_exit_stays_zero,cargo_workplan::test_crate::reconcile::quiet_keeps_the_obligations_and_hides_nothing_that_is_a_finding,cargo_workplan::test_crate::complete::quiet_hides_a_completion_and_keeps_a_refusal,cargo_workplan::test_crate::preview::quiet_prints_nothing_because_a_preview_is_not_a_finding,cargo_workplan::test_crate::verify::quiet_prints_only_the_failing_bindings_and_keeps_the_exit -->

Under `-q` a command whose output holds no finding (`status`, `docs sync`,
`docs preview`, `change new`, `change impact`, a completion, the passing bindings of
`verify`) prints nothing on stdout and leaves out its notes on stderr, while the
obligations, a refusal and the failing bindings are findings and stay; a failure of
the tool is always written to stderr, and the exit code does not change.

### Typed machine reports
<!-- raf:tested-by cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms,cargo_workplan::test_crate::rdf::check_preserves_every_finding_location_in_both_equivalent_formats,cargo_workplan::test_crate::rdf::status_preserves_ordered_capabilities_binding_gaps_and_open_changes,cargo_workplan::test_crate::rdf::preview_preserves_reconciled_text_and_sync_reports_only_actual_rewrites -->

Command reports in both RDF formats use the closed canonical
[report profile](./rdf-report-profile.md), with its versioned application
vocabulary and one snapshot identity. `docs export` carries its native authoring
dataset with stable asserted document identity instead. `evidence claims`
uses the complete native [semantic claim selection](./workplan-evidence-claims.md#semantic-claim-datasets)
for RDF formats: distinct marker occurrences reify unasserted capability,
`claimedStatement` and exact-text triple terms, with stable evidence references
and path, line and optional section provenance. This records authored references,
not proof of the prose's truth. It has no snapshot Report envelope. Every command projects typed domain values;
there is no independent pre-rendered machine block. Named rows and explicit
ordinals preserve nested sequence, and absent optional fields have no triple.
Findings retain their location variant, rule kind, message and fix. Histories
retain full commit ids, dates, sources, paths and removal results. Evidence and
assessment views retain actual input digests and unresolved-input reasons;
measurements remain exact decimal strings. Verification includes only invocation
rows returned by the executor, with their arguments and process outcomes. A
planning refusal does not invent a run. The shared profile documents every class,
property and object type.

### Machine refusal and output failures
<!-- raf:tested-by cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading,cargo_workplan::test_crate::rdf::quiet_keeps_findings_and_an_empty_result_is_still_a_canonical_graph,cargo_workplan::bin_crate::cargo_workplan::render::tests::diagnostic_write_failure_is_propagated_and_quiet_does_not_write_notes,cargo_workplan::bin_crate::cargo_workplan::render::tests::diagnostic_flush_failure_propagates_and_quiet_never_touches_that_sink,cargo_workplan::test_crate::process::help_and_version_output_failures_exit_two_beside_successful_output -->

JSON formats are usage errors, exit 2, before root loading or process execution.
Successful machine output is one complete canonical graph. Command reports include
an empty Report envelope when no visible result remains; document exports retain
their authoring graph and claim queries retain an explicit selection root even
when the selected known evidence has no claims. Native dataset serialization is
buffered before stdout. A physical output failure can leave partial bytes.
A command that cannot run has
empty standard output and a diagnostic on standard error. An output or diagnostic
write failure is also exit 2; a closed diagnostic sink cannot turn a failed run
into success.

### Quiet report selection
<!-- raf:tested-by cargo_workplan::bin_crate::cargo_workplan::render::tests::quiet_filters_results_without_changing_exit_and_machine_output_stays_a_graph,cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms,cargo_workplan::test_crate::rdf::quiet_keeps_findings_and_an_empty_result_is_still_a_canonical_graph -->

A history, an evidence view, a recorded assessment and its
context or list are outcomes. A coverage report is a finding as a whole when it
has a gap or unreadable code surface; a claims report when a claim names no test;
a lines report when it has orphan lines or a counting error. Quiet output keeps
those complete reports and removes outcome reports. A failed evidence record
stays and a passing record is hidden. Text prints no line for an omitted outcome;
RDF snapshot reports still print their empty command envelope. The semantic RDF
claim query refuses quiet mode before loading. Quiet never changes the exit class
of commands that admit it.

### Explicit debt writes
<!-- raf:tested-by cargo_workplan::test_crate::debt::the_write_creates_the_record_exits_zero_and_the_repository_then_passes,cargo_workplan::test_crate::debt::debt_that_grew_is_exit_one_with_its_findings_and_nothing_is_written,cargo_workplan::test_crate::debt::a_finding_of_the_default_rules_stops_the_write_with_exit_one,cargo_workplan::test_crate::debt::a_command_misused_is_a_usage_error_before_any_work,cargo_workplan::test_crate::debt::the_written_record_in_rdf_is_one_object_with_its_members_in_order -->

The command
`debt --write [--copyright <text> --licence <id>]` writes the debt
record `work/debt/default.md` through `Gate::write_debt`. It exits 0 when it
writes, 1 when it refuses with the findings printed (growth, or a finding of the
default rules), and 2 when `--write` is missing or the tool cannot run. The typed DebtWritten result retains its path, change classification, finding
count and orphan-line item count. `debt --write` writes one file and the reports write nothing.

## Usage

Use `cargo workplan --format svg diagram --name dependencies --kind
workspace-dependencies --scope workspace`, or render a declared view with
`cargo workplan --format json diagram --view cap:parser#structure`.
`docs preview` accepts the synchronization SPDX options for new diagram assets
and reports planned asset writes/removals on diagnostics without writing them.
<!-- raf:tested-by cargo_workplan::test_crate::evidence::the_views_print_rdf_and_refuse_what_does_not_exist -->


Run `cargo workplan --format svg diagram --name memory-claims --kind
evidence-claims --scope memory-bound` to write an accessible evidence-claim view
to standard output. Select `json` for its named native export bundle, `text` for
readable semantic descriptions, or `turtle` or `ntriples` for the same RDF
selection. The direct `cargo-workplan` command accepts the same arguments.
<!-- raf:tested-by cargo_workplan::test_crate::evidence::the_views_print_rdf_and_refuse_what_does_not_exist -->

Run completion from the original repository. The native runner creates and
releases its owned detached execution worktree; callers do not redirect the
command or alter the original tracked oracle.

Use `cargo workplan --format turtle evidence claims memory-bound` for the native
semantic claim selection. Omit the RDF format for its ordinary statement view.

`cargo workplan --format ntriples docs export docs/architecture/okf-profile.md`
exports one authoring document. Select `--format turtle` for the other native
carrier. The path is relative to the selected workspace root.
<!-- raf:tested-by cargo_workplan::test_crate::rdf::preview_preserves_reconciled_text_and_sync_reports_only_actual_rewrites -->

The runnable API example `raf_workplan::Workplan::new_change` shows the typed change creation that the command wrapper renders and writes.

```sh
cargo workplan check
cargo workplan status
cargo workplan change new incremental-parsing --capability cap:streaming-parser
cargo workplan docs preview incremental-parsing
cargo workplan change complete incremental-parsing
cargo-workplan --format turtle change reconcile incremental-parsing
```

`make docs-sync` in this repository runs `cargo-workplan docs sync` with the SPDX text
of a new index.

The record commands are used the same way:

- `cargo workplan history cap:workplan-reconcile --since 2026-01-01` prints the
  commits of the capability and of its bound code.
- `cargo workplan evidence record --id peak-rss --for cap:streaming-parser --by agent --limit peak=64 -- cargo bench parse`
  runs the benchmark and writes `work/evidence/peak-rss.md`.
- `cargo workplan assess cap:streaming-parser#usage --context` prints what an
  assessor reads, and `cargo workplan assess cap:streaming-parser#usage --verdict accepted --by human`
  records the verdict.
- `cargo workplan --format turtle coverage code --crate raf-workplan` prints the
  public items of one package that no capability binds.
- `cargo workplan check` judges the complete content, binding and line policy
  without a policy flag or coverage selector.
- `cargo workplan coverage claims --capability cap:streaming-parser` prints each
  claim of the capability and the tests its markers name.
- `cargo workplan coverage lines --crate raf-okf`, or
  `cargo workplan coverage --reverse --crate raf-okf`, prints the orphan lines of
  one package by crate, module and item.

The last command of the complete policy is used the same way:
`cargo workplan debt --write --copyright '2026 Example Inc.' --licence CC-BY-4.0`
writes `work/debt/default.md` with its two SPDX lines.

## Examples

Both argument shapes render the five automatic families through native JSON
exports, and render a declared ownership selection by reference. A conflicting
explicit selection, missing view or invalid scope refuses with empty stdout;
input snapshots remain identical. Coverage after a recorded input changes names
its stale input rather than showing current execution support.
<!-- raf:tested-by cargo_workplan::test_crate::evidence::the_views_print_rdf_and_refuse_what_does_not_exist,cargo_workplan::test_crate::evidence::a_claim_on_a_recorded_result_reconciles_clean_until_its_bound_code_moves -->


The same valid named scope yields identical artifact bytes through direct and
Cargo invocation. A malformed name or evidence scope, an unknown kind, a quiet
diagram request and a JSON request for `check` all exit with a usage error and
leave repository inputs unchanged. A neighbouring valid diagram request succeeds.
<!-- raf:tested-by cargo_workplan::test_crate::evidence::the_views_print_rdf_and_refuse_what_does_not_exist -->

An RDF completion result owns its actual `Binding` and `VerificationRun` rows.
The run's `logicalRoot` names the original repository and `physicalRoot` names
the released execution copy. Their difference is explicit provenance, not a
claim that the copied files retain original physical identities.

An evidence record with no claim markers yields a `ClaimsSelection` with its
stable evidence IRI and no occurrence links. Two markers supporting identical
text yield two distinct reifier occurrences, preserving their source lines.

The same export command accepts a capability, change, architecture page, evidence,
assessment, coverage or debt record. A native reader can parse its output and
`Document::from_dataset(dataset, path)` recovers an equal authoring model, including
comments and quoted values. The generated navigation Index is a separate model.
<!-- raf:tested-by cargo_workplan::test_crate::rdf::preview_preserves_reconciled_text_and_sync_reports_only_actual_rewrites -->

```text
$ cargo workplan status
delivered     2  alpha-store, beta-index
implementing  1  streaming-parser (gap: bindings.tests is empty; changes: incremental-parsing)
designed      0
considered    0
open changes  1  incremental-parsing (cap:streaming-parser)
$ echo $?
0
```

A capability that binds a missing symbol retains its frontmatter location and
a concrete repair in the canonical report:

```turtle
VERSION "1.2"
BASE <https://blackcatinformatics.ca/.well-known/genid/raf-report/909ce0bb63eae85a7c56dc7483631148fac2059fcfd06d804f4a8bdf59fb4e33>
<> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://blackcatinformatics.ca/raf/report/1#Report> .
<> <https://blackcatinformatics.ca/raf/report/1#command> "check" .
<> <https://blackcatinformatics.ca/raf/report/1#reportForm> "1"^^<http://www.w3.org/2001/XMLSchema#integer> .
<> <https://blackcatinformatics.ca/raf/report/1#result> <#result/0> .
<#result/0> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://blackcatinformatics.ca/raf/report/1#Finding> .
<#result/0> <https://blackcatinformatics.ca/raf/report/1#file> "docs/capabilities/beta.md" .
<#result/0> <https://blackcatinformatics.ca/raf/report/1#fix> "write the canonical path of an item the source index defines (check the spelling, the module declarations and the target the item is in), or remove the entry; a name that a macro, derive or attribute generates is not in the index and cannot be bound" .
<#result/0> <https://blackcatinformatics.ca/raf/report/1#key> "bindings" .
<#result/0> <https://blackcatinformatics.ca/raf/report/1#kind> "binding-unresolved" .
<#result/0> <https://blackcatinformatics.ca/raf/report/1#message> "cap:beta code binding `demo::missing` does not resolve: `demo::missing` names nothing in the workspace" .
<#result/0> <https://blackcatinformatics.ca/raf/report/1#ordinal> "0"^^<http://www.w3.org/2001/XMLSchema#integer> .
```

`evidence record` reports the record it wrote, and its exit code says whether the
result holds:

| Command after `evidence record --id peak --for cap:alpha --by agent --limit peak=64 --` | Standard output | Exit |
| --- | --- | --- |
| `/bin/sh -c 'echo peak=41'` | `recorded: work/evidence/peak.md (pass)` | 0 |
| `/bin/sh -c 'echo peak=80'`, with `--update` | `recorded: work/evidence/peak.md (fail): over the limit: peak` | 1 |
| `/bin/sh -c 'echo peak=1e-3'` | nothing: the refusal is on standard error and no record is written | 2 |

## Constraints

Document export uses the existing bounded, contained reader and authoring codec.
Its raw read admits at most `MAX_TEXT_BYTES + 1` bytes so the parser can report an
oversized document; the parser accepts at most 1 MiB. The codec also bounds
structural nodes, triples and aggregate authoring text. Export has no import mode
or output-file writer, and performs no whole-repository semantic projection.
<!-- raf:tested-by cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading -->

- Root discovery needs no Cargo executable. An unreadable or malformed ancestor
  manifest and inconsistent package membership are errors, with no fallback to a
  guessed root; an explicit `--root` bypasses this lookup.
  <!-- raf:tested-by cargo_workplan::test_crate::process::discovery_starts_no_cargo_from_the_cargo_variable,cargo_workplan::test_crate::process::discovery_starts_no_cargo_on_path_when_the_variable_is_unset,cargo_workplan::test_crate::process::discovery_does_not_run_the_build_rustc_program_of_the_workspace,raf_source::test_crate::root::malformed_manifests_and_missing_directories_fail_hard,raf_source::test_crate::root::explicit_workspace_requires_a_safe_path_and_agreed_membership,cargo_workplan::test_crate::root::root_overrides_the_discovery_and_the_current_directory_is_not_read -->
- The binary owns argument parsing and result rendering. Domain judgments call
  `raf-workplan`, so the dev-dependency `Gate` and the CLI agree on findings.
  <!-- raf:tested-by cargo_workplan::test_crate::check::the_findings_are_those_of_the_library_gate_in_the_same_order,cargo_workplan::test_crate::check::the_exit_agrees_with_assert_clean_and_every_finding_is_in_its_message -->
- Tool-owned publication by `check`, `status`, `docs preview`, `change impact`
  and `change reconcile` writes nothing in the repository. `docs sync` writes only generated blocks.
  `change new` writes one file. `change complete` deletes one file, and before it
  rewrites only the generated blocks that the deletion stales.
  <!-- raf:tested-by cargo_workplan::test_crate::check::a_clean_repository_exits_zero_and_prints_nothing,cargo_workplan::test_crate::check::a_repository_with_findings_exits_one_and_prints_each_finding,cargo_workplan::test_crate::status::the_roadmap_groups_one_capability_in_each_state_with_the_gaps_and_the_change,cargo_workplan::test_crate::preview::the_proposed_text_is_printed_and_the_markers_are_not,cargo_workplan::test_crate::impact::impact_prints_what_the_change_reaches_and_writes_nothing,cargo_workplan::test_crate::reconcile::nothing_is_written_and_no_cargo_is_started_for_a_fixture_without_bound_examples,cargo_workplan::test_crate::reconcile::a_bound_example_that_fails_is_an_obligation_and_nothing_is_written_to_the_tree,cargo_workplan::test_crate::sync::only_the_generated_blocks_are_rewritten,cargo_workplan::test_crate::new::new_prints_the_library_text_writes_the_record_and_nothing_else_in_both_shapes,cargo_workplan::test_crate::complete::a_completed_change_prints_what_was_removed_and_rewritten_and_leaves_a_clean_check -->
- `history`, `evidence claims`, `evidence for`, `assess --context`, `assess --list`
  and `coverage` publish no repository files themselves. `evidence record` writes one record,
  `work/evidence/<id>.md`, and only after the run; `assess --verdict` writes one
  record, `work/assessments/<capability slug>-<section>.md`; neither edits a
  document.
  <!-- raf:tested-by cargo_workplan::test_crate::history::a_capability_lists_its_document_and_symbol_commits_newest_first_and_a_removed_binding,cargo_workplan::test_crate::evidence::the_views_print_rdf_and_refuse_what_does_not_exist,cargo_workplan::test_crate::assess::the_context_is_the_same_whoever_asks_and_writes_nothing,cargo_workplan::test_crate::assess::the_list_shows_each_assessment_current_until_an_input_moves,cargo_workplan::test_crate::coverage::gaps_in_either_direction_are_printed_and_exit_one,cargo_workplan::test_crate::claims::every_claim_names_a_test_prints_the_report_and_exits_zero,cargo_workplan::test_crate::lines::a_repository_whose_every_counted_line_is_covered_prints_the_report_and_exits_zero,cargo_workplan::test_crate::evidence::a_command_that_passes_writes_exactly_one_record_and_exits_zero,cargo_workplan::test_crate::assess::a_verdict_writes_exactly_one_record_and_a_rejection_still_exits_zero -->
- `verify` needs only the Git worktree check and tracked-path listing, with no
  commit or history query, and publishes no repository files itself.
  <!-- raf:tested-by cargo_workplan::test_crate::verify::verification_uses_only_tracked_path_git_calls_and_frozen_cargo_without_a_commit -->
- Invoked tests, examples, build scripts and caller-selected evidence programs
  are trusted executable input, not sandboxed by the publication boundary. A
  bound example or build script can write workspace files. Git history may fetch
  missing objects into Git metadata in a partial clone. These effects are
  separate from the tool-owned document and record writes above.
  <!-- raf:tested-by cargo_workplan::test_crate::reconcile::a_bound_example_that_fails_is_an_obligation_and_nothing_is_written_to_the_tree,raf_workplan::test_crate::verify::reconcile_runs_trusted_examples_that_can_write_workspace_files,raf_workplan::test_crate::history::a_partial_clone_history_can_fetch_git_objects_without_writing_working_files_or_index -->
- `status` exits 0 when it lists gaps.
  <!-- raf:tested-by cargo_workplan::test_crate::status::the_roadmap_groups_one_capability_in_each_state_with_the_gaps_and_the_change -->
- Every process the binary starts, directly or through `raf-workplan`, is spawned
  with no shell and an argv vector. Each spec that spawns lists its exact
  invocation, and every `cargo` command the tool composes carries `--frozen`. A
  value taken from a document reaches a process only after resolution and a stated
  pattern, and then in one of four forms: after `--`; inside a single
  `<rev>:<path>` or `-L<start>,<end>:<path>` argument; as a validated identifier
  inside one `-S<identifier>` argument of the history; or as a validated test
  filter (see [the security policy](../../SECURITY.md)).
  <!-- raf:tested-by raf_workplan::exec::tests::no_shell::every_argument_is_one_element_and_no_shell_reads_it,raf_workplan::bound::tests::a_test_with_clean_facts_is_planned_and_its_neighbours_are_refused,raf_workplan::bound::tests::a_doc_test_binding_lists_the_package_and_runs_the_bare_item_path,raf_workplan::bound::tests::an_example_runs_under_its_cargo_name_and_the_exit_status_decides,cargo_workplan::test_crate::verify::verification_uses_only_tracked_path_git_calls_and_frozen_cargo_without_a_commit,cargo_workplan::test_crate::history::a_capability_lists_its_document_and_symbol_commits_newest_first_and_a_removed_binding -->
- The binary starts no process of its own: each process is started through
  `raf-workplan`. The caller's command of `evidence record` is the one process whose
  argv comes from the command line, and `raf-workplan` runs it with no shell,
  exactly as given; no text of a document or record reaches it.
  <!-- raf:tested-by raf_workplan::exec::tests::no_shell::every_argument_is_one_element_and_no_shell_reads_it,cargo_workplan::test_crate::evidence::the_words_after_the_separator_are_the_command_options_included,cargo_workplan::test_crate::process::an_id_a_day_or_a_value_of_the_record_commands_is_refused_before_any_process_starts,cargo_workplan::test_crate::verify::verification_uses_only_tracked_path_git_calls_and_frozen_cargo_without_a_commit -->
- A slug is lowercase letters and digits in parts joined by single `-`, so it starts
  with a letter or a digit; any other slug, one that starts with `-` included, is a
  usage error.
  <!-- raf:tested-by cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_must_be_a_slug,cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_by_every_command_before_any_process_starts -->
- A command that takes the slug of a change record (`docs preview`, `change impact`,
  `change reconcile`, `change complete` and `verify <slug>`) fails with exit 2 when
  `work/changes/<slug>.md` does not exist; `change new` fails with exit 2 when it
  exists.
  <!-- raf:tested-by cargo_workplan::test_crate::preview::an_unknown_slug_is_exit_two_and_names_the_open_changes,cargo_workplan::test_crate::impact::a_slug_with_no_record_is_exit_two_and_names_the_open_changes,cargo_workplan::test_crate::reconcile::everything_that_stops_it_looking_is_exit_two,cargo_workplan::test_crate::complete::what_stops_it_looking_is_exit_two_and_the_record_stays,cargo_workplan::test_crate::verify::a_slug_with_no_record_and_a_document_that_does_not_load_are_exit_two,cargo_workplan::test_crate::new::a_second_run_for_the_same_slug_is_exit_two_and_leaves_the_record_as_it_is -->
- Document text reaches the terminal escaped. The text output of `evidence claims`
  (each statement) and of `assess --context` (the section and the bound code)
  writes each control character other than the line feed and the tab, a carriage
  return, an escape and a bell included, and each bidirectional formatting
  character (U+061C, U+200E, U+200F, U+202A to U+202E, U+2066 to U+2069) as its
  `\u{..}` escape (`raf_workplan::escape::shown_lines`), as the history and
  coverage tables escape their cells, so a document cannot clear the screen, set the
  title of a terminal or reorder a line. The RDF forms retain the original text through canonical RDF lexical escaping.
  Decoded data still requires the consumer to choose safe terminal display;
  the human text option performs that display escaping.
  <!-- raf:tested-by cargo_workplan::test_crate::evidence::a_claimed_statement_reaches_the_terminal_escaped_and_rdf_escapes_it_as_rdf,cargo_workplan::test_crate::assess::the_text_of_a_section_reaches_the_terminal_escaped_and_rdf_escapes_it_as_rdf,raf_workplan::escape::tests::each_control_and_bidirectional_formatting_character_is_escaped,raf_workplan::escape::tests::lines_keep_their_line_feeds_and_tabs_and_lose_every_other_control,cargo_workplan::test_crate::history::rdf_preserves_full_history_commits_sources_paths_and_removed_bindings -->

## Explanation

Library and binary are separate so that the check runs where it cannot be skipped, in
`cargo test`, and the lifecycle commands are available where a person is working. The
exit codes distinguish "the repository has findings" from "the tool failed", which a
script and a hook need in order to react correctly.

The command set mirrors the lifecycle. `change new` opens, `impact` and `reconcile`
read, `docs preview` shows the outcome, `verify` runs the evidence available, and
`complete` closes. `status` is the roadmap, derived from the states of the documents
rather than kept by hand.

## Verification

Both command invocation shapes and both RDF carriers exercise native completion.
The machine-output case checks retained context, native index provenance,
cleanup and the exact original capability/record publication footprint.

Evidence query cases compare native Turtle and N-Triples with the direct library
dataset and preserve input snapshots on success and refusal. The output-sink
case checks semantic claims alongside authoring exports and ordinary help.

The existing RDF documentation-command case exports all seven authoring fixtures
through both invocation forms and both native carriers, then compares the native
inverse with the original model. It checks repeated values and deterministic
replay while preserving every input byte. Existing refusal and sink cases cover
input containment, parser and codec bounds, profile and Index errors, argument
refusal before loading and physical output failures beside successful neighbors.
<!-- raf:tested-by cargo_workplan::test_crate::rdf::preview_preserves_reconciled_text_and_sync_reports_only_actual_rewrites,cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading,cargo_workplan::test_crate::process::a_usage_error_starts_no_process,cargo_workplan::test_crate::process::help_and_version_output_failures_exit_two_beside_successful_output -->

- An argument-parsing test per command and option, including `--capability`
  repetition and `--format`.
- A fixture repository per command asserting stdout, files written and the exit code,
  with exit 0, 1 and 2 each exercised: a clean check, a finding, and a missing slug.
- Root and nested-member fixtures exercise both CLI argument shapes. Recording
  Cargo stubs on `PATH` and in `$CARGO` observe no discovery calls. Memory-tree
  fixtures cover standalone, excluded, nested, unlisted and explicit-workspace
  packages, alongside malformed declarations and missing directories.
- A slug argument that begins with `-`, such as `-rf` or `--since=x`, is a usage
  error with exit 2 before any process is spawned, and is never read as an option by
  a spawned process.
- A read-only check: the tree is byte-identical after `check`, `status`,
  `docs preview`, `change impact` and `change reconcile`.
- A `status` fixture with one capability in each state asserting the grouping and the
  gaps listed.
- A `docs preview` fixture asserting that PROPOSED markers are absent from the output
  and the text of the blocks is present.
- An equivalence test: the violations from `check` equal those of
  `Gate::workspace(...).assert_clean()` on the same fixtures.
- Both argument shapes, `cargo workplan <command>` through a real `cargo` dispatch
  and `cargo-workplan <command>`, give the same output and exit code; when a
  command is killed for writing too much output, the two runs can measure a
  different time, so the seconds of that message are left out of the comparison.
- Real command RDF fixtures: exact native byte goldens, complete emitted-term
  equality with the documented registries, typed scalar and ordered child
  assertions, both-format graph equivalence, empty envelopes and usage refusals.

- Argument-parsing tests of the record commands in both shapes: each id, day, limit,
  revision, section and package refused when it is not one or starts with `-`; every
  word after the first `--` taken as the command, options and a second `--` included;
  an option whose value is `--` refused; exactly one mode of `assess`; exactly one
  source of the verdict of `evidence record`; and the two readings of the command
  compared word by word.
- A fixture runs the binary with recording stubs of `git` and `cargo` first on `PATH`
  and `$CARGO` naming the `cargo` stub, and asserts that each such refusal starts no
  `git` and no `cargo` and runs no command, with and without `--root`, beside a
  neighbour that starts a process; the refusals of the binary itself (an empty
  program, a word that is not UTF-8, a program name of `--`, `--from-binding` beside
  a bare `--`) are included, without `--root`. A
  recording `git` stub on `PATH` that answers the version shows that `history` asks
  `git --version` before its log, refuses 2.29 and goes on at 2.30.
- Fixtures through the built binary for each command. `history` on a real Git
  history: the document and symbol commits newest first, a removed binding with the
  commit that removed it and its parent, `--since`, the same entries in the table and
  in typed RDF, a shallow clone refused, and a bound file that is not committed refused
  naming it until it is. `evidence record` of a `/bin/sh -c` command: pass with exit
  0; fail by its exit status and by a value over its limit with exit 1; each parse
  failure and a command that cannot start with exit 2 and no record; exactly one file
  written; a replacement with and without the SPDX pair; `--from-binding` with the
  real `cargo`; a real `cargo workplan` dispatch that passes the words after `--`
  unchanged; and a claim that reconciles clean until its bound code moves. `assess`:
  the context identical for `--by human` and `--by agent`, a verdict writing one
  file, the list current and then stale naming the section, and an unknown capability
  exit 2. `coverage`: gaps with exit 1, an exclusion with exit 0, a surface that
  cannot be listed with exit 1 for `coverage code` and 0 for `coverage tests`,
  `--crate` in both forms, and this repository, whose reports name the gaps of the
  crates outside the scope and none of `raf-workplan` and `cargo-workplan`.
  `coverage lines`: exit 0 with the literal report of a repository whose every
  counted line is covered, 1 with an orphan helper in text and RDF, 2 for a package
  that is not one, and `coverage --reverse` printing the same bytes in each case;
  with an exception, the excepted table and the `lines-excepted` members in order; and
  through `check`, a ceiling row and its `of:` class deciding exit 0 or 1 under
  the default policy, and an exception that matches nothing exit 1 beside a matched one.
- RDF projections of every record command preserve typed fields and nested
  order; `-q` hides each outcome and keeps a `fail` record and a
  coverage gap; the read-only commands leave the tree byte-identical.
- An escape sequence that clears the screen, an OSC title ended by a bell and a
  right-to-left override in a claimed statement and in a section: the text output
  of `evidence claims` and of `assess --context` holds none of them raw and shows
  each as its `\u{..}` escape, keeping the line feeds and the tabs, and the
  context equals the library's text escaped; the RDF forms hold no raw escape control and decode to the original text;
  plain text is printed unchanged.
- `evidence claims` while a capability document does not load: exit 2 in text and
  RDF, naming the document on standard error, in text with nothing on standard
  output, and nothing written, for an id with claims and for one without; a change
  record or an evidence record that does not load leaves the answer, exit 0, while
  `check` reports it; the clean neighbour answers.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_command_that_takes_a_slug_refuses_one_that_starts_with_a_dash_in_both_forms` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_missing_command_an_unknown_command_and_a_missing_slug_are_usage_errors` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_must_be_a_slug` | test | [Constraints](#constraints) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_that_starts_with_a_dash_is_a_usage_error_however_it_is_given` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::an_unknown_output_format_is_a_usage_error` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::assess_takes_a_section_and_exactly_one_mode` | test | [Contract](#contract) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_needs_a_capability_and_both_or_neither_spdx_option` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_takes_a_slug_repeated_capabilities_and_affects_and_the_spdx_pair` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_claims_takes_an_optional_capability` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_lines_takes_an_optional_package_and_reverse_is_the_same_command` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_takes_a_direction_and_an_optional_package` | test | [Contract](#contract) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::debt_takes_write_and_the_spdx_pair_together_or_not_at_all` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::each_command_parses_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::evidence_claims_and_for_take_one_id_each` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::evidence_record_reads_every_option_and_the_command_after_the_separator` | test | [Contract](#contract) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::evidence_record_refuses_a_separator_as_a_value_and_a_record_it_cannot_write` | test | [Contract](#contract) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::help_and_version_exit_zero_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::history_refuses_an_id_or_a_day_before_anything_runs` | test | [Contract](#contract) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::history_takes_an_id_of_each_kind_and_a_day` | test | [Contract](#contract) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::options_after_the_separator_belong_to_the_command` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_default_and_parse_in_both_shapes` | test | [Contract](#contract) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_may_follow_the_command` | test | [Contract](#contract) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_removed_policy_flag_is_rejected_before_and_after_the_command` | test | [Contract](#contract) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_sync_text_options_parse` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::evidence::tests::a_program_named_the_separator_and_a_binding_beside_one_are_refused` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::evidence::tests::the_command_runs_only_when_both_readings_of_the_line_agree` | test | [Contract](#contract) |
| `cargo_workplan::bin_crate::cargo_workplan::exit::tests::a_library_error_becomes_its_display_text` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::exit::tests::the_three_exits_have_the_documented_codes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::diagnostic_flush_failure_propagates_and_quiet_never_touches_that_sink` | test | [Contract](#contract) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::diagnostic_write_failure_is_propagated_and_quiet_does_not_write_notes` | test | [Contract](#contract) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::quiet_filters_results_without_changing_exit_and_machine_output_stays_a_graph` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::assess::a_section_that_cannot_be_assessed_is_exit_two_and_writes_nothing` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::assess::a_verdict_writes_exactly_one_record_and_a_rejection_still_exits_zero` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::assess::moved_inputs_preserve_raw_unresolved_reasons_only_for_unresolved_changes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::assess::the_context_is_the_same_whoever_asks_and_writes_nothing` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::assess::the_list_shows_each_assessment_current_until_an_input_moves` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::assess::the_text_of_a_section_reaches_the_terminal_escaped_and_rdf_escapes_it_as_rdf` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::check::a_clean_repository_exits_zero_and_prints_nothing` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::check::a_directory_that_is_not_a_repository_exits_two_and_says_why_on_standard_error` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::check::a_finding_is_gone_once_it_is_fixed` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_manifest_that_does_not_parse_exits_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_removed_policy_key_is_refused_beside_the_default_content_refusal` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_repository_with_findings_exits_one_and_prints_each_finding` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::check::a_root_that_does_not_exist_exits_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_source_file_that_cannot_be_read_exits_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_stale_generated_block_is_a_finding_and_a_sync_clears_it` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::help_and_version_exit_zero_on_standard_output` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::policy_findings_keep_typed_locations_and_messages_in_rdf` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::quiet_keeps_the_findings_and_the_exit_code` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::quiet_on_a_clean_repository_prints_nothing_and_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_default_check_accepts_the_complete_fixture` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_default_check_refuses_a_thin_section_without_a_policy_flag` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::check::the_default_check_without_capability_documents_is_a_run_failure` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_exit_agrees_with_assert_clean_and_every_finding_is_in_its_message` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::check::the_findings_are_those_of_the_library_gate_in_the_same_order` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::check::usage_errors_exit_two_with_a_message_on_standard_error` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::claims::a_capability_that_is_not_loaded_exits_two_and_says_why` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::claims::a_claim_that_names_no_test_is_a_finding_and_exits_one_in_text_and_rdf_alike` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::claims::every_claim_names_a_test_prints_the_report_and_exits_zero` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::complete::a_capability_left_implementing_is_refused_and_nothing_is_written` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::complete::a_completed_change_prints_what_was_removed_and_rewritten_and_leaves_a_clean_check` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::complete::a_stale_generated_block_is_refused_until_the_sync_is_run` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::both_shapes_complete_the_same_way` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::complete::every_failure_is_listed_and_each_condition_is_named_in_rdf` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::from_new_to_a_completed_change_through_the_binary` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::quiet_hides_a_completion_and_keeps_a_refusal` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::complete::rdf_for_a_completion_is_a_completed_object_and_a_rewrote_object_for_each_file` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::the_default_completion_refuses_an_underexplained_section_and_accepts_its_neighbor` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::complete::what_stops_it_looking_is_exit_two_and_the_record_stays` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::coverage::a_surface_that_cannot_be_listed_is_a_finding_of_the_code_report_only` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::coverage::an_exclusion_takes_a_gap_out_and_a_report_without_gaps_exits_zero` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::coverage::crate_selects_a_package_by_either_name_and_refuses_one_that_is_not_there` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::coverage::gaps_in_either_direction_are_printed_and_exit_one` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::coverage::this_repository_has_no_code_or_test_gaps_in_its_scoped_packages` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::a_command_misused_is_a_usage_error_before_any_work` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::debt::a_finding_of_the_default_rules_stops_the_write_with_exit_one` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::debt::a_second_write_is_unchanged_and_the_bytes_are_the_same` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::debt_that_grew_is_exit_one_with_its_findings_and_nothing_is_written` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::debt::orphan_lines_over_the_ceiling_are_recorded_per_item` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::paid_debt_fails_the_check_with_debt_resolved_until_the_write_lowers_the_record` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::the_default_policy_can_record_a_current_structural_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::the_spdx_lines_are_given_together_and_kept_when_the_record_is_lowered` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::debt::the_write_creates_the_record_exits_zero_and_the_repository_then_passes` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::debt::the_written_record_in_rdf_is_one_object_with_its_members_in_order` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::evidence::a_bound_test_gives_the_verdict_and_a_path_that_names_none_starts_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::a_claim_on_a_recorded_result_reconciles_clean_until_its_bound_code_moves` | test | [Examples](#examples) |
| `cargo_workplan::test_crate::evidence::a_claimed_statement_reaches_the_terminal_escaped_and_rdf_escapes_it_as_rdf` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::evidence::a_command_that_passes_writes_exactly_one_record_and_exits_zero` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::evidence::a_failing_command_or_a_value_over_its_limit_records_fail_and_exits_one` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::evidence::a_replacement_holds_the_spdx_lines_of_its_own_run_only` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::evidence::a_request_that_cannot_be_recorded_runs_nothing` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::evidence::an_opposite_host_binding_is_exit_two_and_cannot_create_or_replace_evidence` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::cargo_passes_the_command_after_the_separator_to_the_binary_unchanged` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::evidence::claims_exit_two_naming_a_capability_that_does_not_load_and_answer_beside_other_records` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::evidence::output_the_tool_cannot_read_is_exit_two_and_writes_no_record` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::evidence::rdf_is_one_evidence_recorded_object_with_its_fields` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::recorded_binding_failures_keep_failed_and_unresolved_outcomes_distinct` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::the_views_print_rdf_and_refuse_what_does_not_exist` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `cargo_workplan::test_crate::evidence::the_words_after_the_separator_are_the_command_options_included` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::history::a_bound_file_that_is_not_committed_is_exit_two_naming_it_until_it_is` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::history::a_capability_lists_its_document_and_symbol_commits_newest_first_and_a_removed_binding` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::history::a_shallow_clone_is_exit_two_and_its_full_clone_answers` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::history::quiet_prints_nothing_because_a_history_is_not_a_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::rdf_preserves_full_history_commits_sources_paths_and_removed_bindings` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::history::since_limits_the_commits_and_a_change_or_an_unknown_id_is_answered_or_refused` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::impact::a_change_that_reaches_dependants_and_linking_sections_lists_them_sorted` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::a_record_that_does_not_load_is_exit_two_with_its_findings` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::a_slug_with_no_record_is_exit_two_and_names_the_open_changes` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::impact::impact_finds_the_root_from_below_and_by_root` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::impact_prints_what_the_change_reaches_and_writes_nothing` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::impact::quiet_prints_nothing_and_the_exit_stays_zero` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::impact::rdf_preserves_impact_bindings_dependencies_links_and_list_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::a_ceiling_row_and_its_class_decide_the_exit_of_the_default_check_on_the_binary` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::a_package_that_is_not_one_exits_two_and_says_why` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::lines::a_repository_whose_every_counted_line_is_covered_prints_the_report_and_exits_zero` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::lines::an_exception_is_the_excepted_table_and_a_lines_excepted_object_and_the_report_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::an_exception_that_matches_nothing_fails_check_under_default_and_its_matched_neighbour_passes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::an_orphan_line_is_a_finding_and_exits_one_in_text_and_rdf_alike_and_reverse_prints_the_same` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::new::a_capability_given_twice_is_listed_once` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::a_capability_that_is_missing_or_a_bad_option_is_a_usage_error_before_anything_is_read` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::a_second_run_for_the_same_slug_is_exit_two_and_leaves_the_record_as_it_is` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::new::affects_is_repeatable_and_the_record_holds_the_entries_given_in_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::every_refusal_of_the_library_is_exit_two_with_the_tree_untouched` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::new_prints_the_library_text_writes_the_record_and_nothing_else_in_both_shapes` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::new::new_works_from_a_subdirectory_and_by_root` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::quiet_writes_the_record_and_prints_nothing` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::new::rdf_preserves_created_change_fields_and_ordered_guidance` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::spdx_text_that_is_not_one_line_is_exit_two_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_note_about_the_generated_blocks_is_there_when_the_record_makes_them_stale` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_printed_template_in_a_capability_of_each_state_gives_a_clean_check_and_one_unresolved_proposal` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_spdx_lines_are_the_two_comment_lines_of_the_records_of_this_repository` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_template_left_in_a_state_that_refuses_it_is_the_finding_the_guidance_names` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::a_capability_the_change_lists_that_has_no_document_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::a_change_record_that_did_not_load_is_exit_two_and_says_so` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::a_slug_that_is_not_a_slug_is_a_usage_error` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::an_unknown_slug_is_exit_two_and_names_the_open_changes` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::preview::quiet_prints_nothing_because_a_preview_is_not_a_finding` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::preview::the_output_is_the_text_the_library_previews_under_a_header_for_each_document` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::the_preview_is_the_same_by_root_from_another_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::the_proposed_text_is_printed_and_the_markers_are_not` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::process::a_git_older_than_2_30_is_exit_two_before_any_other_process` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::process::a_missing_git_is_exit_two_and_names_it` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::process::a_slug_that_is_not_a_slug_is_refused_before_any_process_starts` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_before_any_process_starts` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_by_every_command_before_any_process_starts` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::process::a_usage_error_of_the_change_commands_starts_no_process` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_usage_error_starts_no_process` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::process::an_id_a_day_or_a_value_of_the_record_commands_is_refused_before_any_process_starts` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::process::cargo_dispatches_workplan_to_the_binary_on_path_and_passes_the_exit_code_back` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::process::discovery_does_not_run_the_build_rustc_program_of_the_workspace` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::process::discovery_starts_no_cargo_from_the_cargo_variable` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::process::discovery_starts_no_cargo_on_path_when_the_variable_is_unset` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::process::discovery_succeeds_when_cargo_cannot_be_started` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::help_and_version_output_failures_exit_two_beside_successful_output` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::process::history_asks_the_git_version_before_its_log_and_refuses_a_git_older_than_2_30` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::process::root_starts_no_cargo_at_all` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::rdf::check_preserves_every_finding_location_in_both_equivalent_formats` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::rdf::preview_preserves_reconciled_text_and_sync_reports_only_actual_rewrites` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples) |
| `cargo_workplan::test_crate::rdf::quiet_keeps_findings_and_an_empty_result_is_still_a_canonical_graph` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::rdf::status_preserves_ordered_capabilities_binding_gaps_and_open_changes` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::reconcile::a_bound_example_that_fails_is_an_obligation_and_nothing_is_written_to_the_tree` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::reconcile::everything_that_stops_it_looking_is_exit_two` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::reconcile::nothing_is_written_and_no_cargo_is_started_for_a_fixture_without_bound_examples` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::reconcile::quiet_keeps_the_obligations_and_hides_nothing_that_is_a_finding` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::reconcile::rdf_preserves_each_obligation_and_optional_location_in_library_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::registry::command_extension_registry_matches_the_documented_terms` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::root::a_directory_outside_any_workspace_is_exit_two_and_names_the_command` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::a_relative_root_is_taken_from_the_current_directory` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::root::every_command_finds_the_root_from_a_subdirectory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::root_overrides_the_discovery_and_the_current_directory_is_not_read` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::root::sync_from_a_subdirectory_writes_at_the_root_and_nowhere_else` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::the_root_is_found_from_the_root_and_from_any_directory_below_it` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::status::a_directory_that_is_not_a_repository_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::a_state_with_no_capability_is_a_row_with_a_count_of_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::an_implementing_capability_with_every_binding_resolved_has_no_gap_beyond_its_empty_lists` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::quiet_leaves_standard_output_empty_because_the_roadmap_has_no_finding` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::status::the_roadmap_groups_one_capability_in_each_state_with_the_gaps_and_the_change` | test | [Contract](#contract), [Constraints](#constraints) |
| `cargo_workplan::test_crate::status::the_roadmap_is_the_same_by_root_from_another_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_document_that_does_not_load_is_exit_two_and_nothing_is_written` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_missing_index_with_the_text_is_created_with_that_header` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_missing_index_without_the_text_is_exit_two_naming_the_field_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_refused_file_is_exit_two_and_nothing_is_written` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_second_sync_writes_nothing_and_prints_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_value_that_cannot_be_written_into_the_header_is_refused_when_the_index_is_created` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::each_field_that_is_not_given_is_named` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::only_the_generated_blocks_are_rewritten` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::sync::quiet_hides_the_files_rewritten_and_keeps_the_exit_code` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::sync::the_index_is_created_by_root_from_another_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::the_text_is_not_read_when_the_index_exists` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::a_failing_example_and_a_binding_that_proves_nothing_are_failures` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::verify::a_hostile_binding_is_a_failure_and_never_reaches_cargo` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::a_slug_with_no_record_and_a_document_that_does_not_load_are_exit_two` | test | [Constraints](#constraints) |
| `cargo_workplan::test_crate::verify::every_binding_passing_exits_zero` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::verify::host_nonapplicability_is_exit_zero_and_preserved_in_machine_output` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::verify::no_slug_runs_every_capability_and_a_slug_only_the_ones_the_change_lists` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::nothing_to_run_is_exit_zero_with_a_note_that_quiet_hides` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::quiet_prints_only_the_failing_bindings_and_keeps_the_exit` | test | [Contract](#contract) |
| `cargo_workplan::test_crate::verify::rdf_preserves_real_binding_outcomes_proof_counts_and_quiet_filtering` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::verification_uses_only_tracked_path_git_calls_and_frozen_cargo_without_a_commit` | test | [Constraints](#constraints) |
| `raf_source::test_crate::root::explicit_workspace_requires_a_safe_path_and_agreed_membership` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::root::malformed_manifests_and_missing_directories_fail_hard` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_source::test_crate::root::manifest_size_bound_is_enforced_beside_the_exact_limit` | test | [Contract](#contract) |
| `raf_source::test_crate::root::member_and_virtual_workspace_directories_resolve_without_sources` | test | [Contract](#contract) |
| `raf_source::test_crate::root::standalone_excluded_and_nested_packages_have_their_own_roots` | test | [Contract](#contract) |
| `raf_source::test_crate::root::transitive_path_dependencies_are_members` | test | [Contract](#contract) |
| `raf_workplan::Workplan::impact` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::new_change` | doctest | [Usage](#usage) |
| `raf_workplan::Workplan::preview` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::sync` | doctest | [Verification](#verification) |
| `raf_workplan::bound::tests::a_doc_test_binding_lists_the_package_and_runs_the_bare_item_path` | test | [Constraints](#constraints) |
| `raf_workplan::bound::tests::a_test_with_clean_facts_is_planned_and_its_neighbours_are_refused` | test | [Constraints](#constraints) |
| `raf_workplan::bound::tests::an_example_runs_under_its_cargo_name_and_the_exit_status_decides` | test | [Constraints](#constraints) |
| `raf_workplan::escape::shown_lines` | doctest | [Constraints](#constraints) |
| `raf_workplan::escape::tests::each_control_and_bidirectional_formatting_character_is_escaped` | test | [Constraints](#constraints) |
| `raf_workplan::escape::tests::lines_keep_their_line_feeds_and_tabs_and_lose_every_other_control` | test | [Constraints](#constraints) |
| `raf_workplan::exec::tests::no_shell::every_argument_is_one_element_and_no_shell_reads_it` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::history::a_partial_clone_history_can_fetch_git_objects_without_writing_working_files_or_index` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::root::a_dangling_manifest_is_a_finding_instead_of_an_ancestor_fallback` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::a_directory_outside_any_workspace_is_an_actionable_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::a_nonexistent_directory_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::memory_tree_membership_discovery_needs_no_disk_adapter` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::the_root_is_found_from_the_root_a_member_and_a_subdirectory` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::tree_directory_aliases_fail_instead_of_selecting_lexical_ancestors` | test | [Verification](#verification) |
| `raf_workplan::test_crate::verify::reconcile_runs_trusted_examples_that_can_write_workspace_files` | test | [Constraints](#constraints) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-complete-gate](./workplan-complete-gate.md) |
| Depends on | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depends on | [workplan-reconcile](./workplan-reconcile.md) |
<!-- raf:generated:end -->
