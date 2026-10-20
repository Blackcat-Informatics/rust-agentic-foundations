---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-cli
title: "cargo workplan command line"
description: "The cargo workplan subcommands, options and exit codes."
state: implementing
bindings:
  code:
    - cargo_workplan::bin_crate::cargo_workplan::main
    - cargo_workplan::bin_crate::cargo_workplan::cli::parse
    - cargo_workplan::bin_crate::cargo_workplan::cli::Args
    - cargo_workplan::bin_crate::cargo_workplan::command::run
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
  tests:
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::each_command_parses_in_both_shapes
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_takes_a_slug_repeated_capabilities_and_affects_and_the_spdx_pair
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_needs_a_capability_and_both_or_neither_spdx_option
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_default_and_parse_in_both_shapes
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_sync_text_options_parse
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_format_that_is_not_text_or_json_is_a_usage_error
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_missing_command_an_unknown_command_and_a_missing_slug_are_usage_errors
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_that_starts_with_a_dash_is_a_usage_error_however_it_is_given
    - cargo_workplan::bin_crate::cargo_workplan::exit::tests::the_three_exits_have_the_documented_codes
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::quiet_prints_only_findings_and_keeps_the_exit
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
    - cargo_workplan::test_crate::verify::json_is_one_binding_object_for_each_binding
    - cargo_workplan::test_crate::json::check_prints_one_object_for_each_finding_of_the_gate_with_the_fields_of_its_location
    - cargo_workplan::test_crate::json::the_kinds_that_are_not_rules_are_not_rule_names
    - cargo_workplan::test_crate::json::every_object_is_a_single_line_inside_one_array
    - cargo_workplan::test_crate::json::a_run_that_could_not_run_prints_nothing_on_standard_output_in_json_too
    - cargo_workplan::test_crate::process::discovery_runs_exactly_cargo_metadata_from_the_cargo_variable
    - cargo_workplan::test_crate::process::discovery_runs_the_cargo_on_path_when_the_variable_is_unset
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
    - cargo_workplan::test_crate::complete::every_failure_is_listed_and_each_condition_is_named_in_json
    - cargo_workplan::test_crate::complete::json_for_a_completion_is_a_completed_object_and_a_rewrote_object_for_each_file
    - cargo_workplan::test_crate::complete::quiet_hides_a_completion_and_keeps_a_refusal
    - cargo_workplan::test_crate::complete::what_stops_it_looking_is_exit_two_and_the_record_stays
    - cargo_workplan::test_crate::impact::a_change_that_reaches_dependants_and_linking_sections_lists_them_sorted
    - cargo_workplan::test_crate::impact::a_record_that_does_not_load_is_exit_two_with_its_findings
    - cargo_workplan::test_crate::impact::a_slug_with_no_record_is_exit_two_and_names_the_open_changes
    - cargo_workplan::test_crate::impact::impact_finds_the_root_from_below_and_by_root
    - cargo_workplan::test_crate::impact::json_is_one_impact_object_with_its_lists
    - cargo_workplan::test_crate::impact::quiet_prints_nothing_and_the_exit_stays_zero
    - cargo_workplan::test_crate::json::a_clean_run_prints_one_empty_array
    - cargo_workplan::test_crate::json::preview_prints_a_preview_object_for_each_capability_with_the_reconciled_text
    - cargo_workplan::test_crate::json::quiet_keeps_the_findings_in_json_and_prints_an_empty_array_without_any
    - cargo_workplan::test_crate::json::status_prints_a_state_object_for_each_state_and_the_open_changes
    - cargo_workplan::test_crate::json::sync_prints_a_rewrote_object_for_each_file_and_an_empty_array_for_none
    - cargo_workplan::test_crate::json::the_json_and_the_text_forms_print_the_same_findings_in_the_same_order
    - cargo_workplan::test_crate::new::a_capability_given_twice_is_listed_once
    - cargo_workplan::test_crate::new::a_capability_that_is_missing_or_a_bad_option_is_a_usage_error_before_anything_is_read
    - cargo_workplan::test_crate::new::a_second_run_for_the_same_slug_is_exit_two_and_leaves_the_record_as_it_is
    - cargo_workplan::test_crate::new::affects_is_repeatable_and_the_record_holds_the_entries_given_in_order
    - cargo_workplan::test_crate::new::every_refusal_of_the_library_is_exit_two_with_the_tree_untouched
    - cargo_workplan::test_crate::new::json_is_one_new_change_object_with_its_fields
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
    - cargo_workplan::test_crate::process::a_cargo_that_cannot_be_started_is_exit_two
    - cargo_workplan::test_crate::process::a_cargo_that_fails_is_exit_two_with_what_it_said
    - cargo_workplan::test_crate::process::a_slug_that_is_not_a_slug_is_refused_before_any_process_starts
    - cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_before_any_process_starts
    - cargo_workplan::test_crate::process::a_usage_error_of_the_change_commands_starts_no_process
    - cargo_workplan::test_crate::process::a_usage_error_starts_no_process
    - cargo_workplan::test_crate::process::output_from_cargo_that_is_not_the_metadata_is_exit_two
    - cargo_workplan::test_crate::process::root_starts_no_cargo_at_all
    - cargo_workplan::test_crate::reconcile::a_bound_example_that_fails_is_an_obligation_and_nothing_is_written_to_the_tree
    - cargo_workplan::test_crate::reconcile::everything_that_stops_it_looking_is_exit_two
    - cargo_workplan::test_crate::reconcile::json_is_one_obligation_object_for_each_obligation
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
    - raf_workplan::test_crate::root::a_cargo_that_does_not_exist_is_an_error
    - raf_workplan::test_crate::root::a_directory_outside_any_workspace_is_an_error_that_says_what_cargo_said
    - raf_workplan::test_crate::root::configured_programs::discovery_does_not_run_the_programs_of_the_cargo_configuration
    - raf_workplan::test_crate::root::stubs::exactly_one_process_runs_with_the_documented_argv_directory_and_locale
    - raf_workplan::test_crate::root::stubs::hostile_output_is_an_error_and_never_a_guess
    - raf_workplan::test_crate::root::stubs::output_is_bounded_at_64_mebibytes
    - raf_workplan::test_crate::root::stubs::the_top_level_member_wins_over_a_nested_one_with_the_same_name
    - raf_workplan::test_crate::root::the_root_is_found_from_the_root_a_member_and_a_subdirectory
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_command_that_takes_a_slug_refuses_one_that_starts_with_a_dash_in_both_forms
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_must_be_a_slug
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::help_and_version_exit_zero_in_both_shapes
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_may_follow_the_command
    - cargo_workplan::bin_crate::cargo_workplan::exit::tests::a_library_error_becomes_its_display_text
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::a_finding_makes_the_exit_one_and_an_outcome_does_not
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::a_json_object_begins_with_its_kind_and_keeps_the_order_of_its_fields
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::a_record_with_no_text_has_no_line_in_text_and_an_object_in_json
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::an_empty_report_is_nothing_in_text_and_an_empty_array_in_json
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::text_with_quotes_and_control_characters_is_escaped
    - raf_workplan::root::json::tests::a_large_text_is_scanned_to_its_end
    - raf_workplan::root::json::tests::a_member_that_is_only_nested_is_not_the_member
    - raf_workplan::root::json::tests::a_member_twice_is_refused_even_when_the_values_agree
    - raf_workplan::root::json::tests::a_top_level_value_that_is_not_an_object_is_refused
    - raf_workplan::root::json::tests::a_value_that_is_not_a_string_is_refused
    - raf_workplan::root::json::tests::an_unpaired_surrogate_is_text_nobody_can_use
    - raf_workplan::root::json::tests::escapes_are_decoded
    - raf_workplan::root::json::tests::every_prefix_of_a_document_is_refused_and_no_mutation_panics
    - raf_workplan::root::json::tests::nesting_is_bounded_and_never_overflows_the_stack
    - raf_workplan::root::json::tests::numbers_of_the_grammar_pass
    - raf_workplan::root::json::tests::random_strings_of_json_punctuation_never_panic
    - raf_workplan::root::json::tests::text_that_is_not_json_is_refused
    - raf_workplan::root::json::tests::the_member_is_found_wherever_it_sits_at_the_top_level
    - raf_workplan::root::json::tests::the_output_of_cargo_metadata_gives_its_root
    - raf_workplan::root::tests::a_failing_cargo_is_an_error_with_what_it_said
    - raf_workplan::root::tests::a_root_that_is_not_an_absolute_path_is_refused
    - raf_workplan::root::tests::cargo_that_cannot_run_or_writes_too_much_is_an_error
    - raf_workplan::root::tests::output_that_does_not_give_a_root_is_an_error
    - raf_workplan::root::tests::the_root_is_the_workspace_root_member_and_the_command_is_exactly_one
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
    - cargo_workplan::bin_crate::cargo_workplan::render::tests::a_rendered_block_is_printed_as_given_and_quiet_hides_only_an_outcome
    - cargo_workplan::test_crate::assess::a_section_that_cannot_be_assessed_is_exit_two_and_writes_nothing
    - cargo_workplan::test_crate::assess::a_verdict_writes_exactly_one_record_and_a_rejection_still_exits_zero
    - cargo_workplan::test_crate::assess::the_context_is_the_same_whoever_asks_and_writes_nothing
    - cargo_workplan::test_crate::assess::the_list_shows_each_assessment_current_until_an_input_moves
    - cargo_workplan::test_crate::coverage::an_exclusion_takes_a_gap_out_and_a_report_without_gaps_exits_zero
    - cargo_workplan::test_crate::coverage::crate_selects_a_package_by_either_name_and_refuses_one_that_is_not_there
    - cargo_workplan::test_crate::coverage::gaps_in_either_direction_are_printed_and_exit_one
    - cargo_workplan::test_crate::coverage::this_repository_names_the_gaps_outside_its_scope_and_none_in_it
    - cargo_workplan::test_crate::claims::every_claim_names_a_test_prints_the_report_and_exits_zero
    - cargo_workplan::test_crate::claims::a_claim_that_names_no_test_is_a_finding_and_exits_one_in_text_and_json_alike
    - cargo_workplan::test_crate::claims::a_capability_that_is_not_loaded_exits_two_and_says_why
    - cargo_workplan::test_crate::lines::a_repository_whose_every_counted_line_is_covered_prints_the_report_and_exits_zero
    - cargo_workplan::test_crate::lines::an_orphan_line_is_a_finding_and_exits_one_in_text_and_json_alike_and_reverse_prints_the_same
    - cargo_workplan::test_crate::lines::a_package_that_is_not_one_exits_two_and_says_why
    - cargo_workplan::test_crate::evidence::a_bound_test_gives_the_verdict_and_a_path_that_names_none_starts_nothing
    - cargo_workplan::test_crate::evidence::a_claim_on_a_recorded_result_reconciles_clean_until_its_bound_code_moves
    - cargo_workplan::test_crate::evidence::a_command_that_passes_writes_exactly_one_record_and_exits_zero
    - cargo_workplan::test_crate::evidence::a_failing_command_or_a_value_over_its_limit_records_fail_and_exits_one
    - cargo_workplan::test_crate::evidence::a_request_that_cannot_be_recorded_runs_nothing
    - cargo_workplan::test_crate::evidence::cargo_passes_the_command_after_the_separator_to_the_binary_unchanged
    - cargo_workplan::test_crate::evidence::json_is_one_evidence_recorded_object_with_its_fields
    - cargo_workplan::test_crate::evidence::output_the_tool_cannot_read_is_exit_two_and_writes_no_record
    - cargo_workplan::test_crate::evidence::the_views_print_json_and_refuse_what_does_not_exist
    - cargo_workplan::test_crate::evidence::claims_exit_two_naming_a_capability_that_does_not_load_and_answer_beside_other_records
    - cargo_workplan::test_crate::evidence::a_claimed_statement_reaches_the_terminal_escaped_and_json_escapes_it_as_json
    - cargo_workplan::test_crate::assess::the_text_of_a_section_reaches_the_terminal_escaped_and_json_escapes_it_as_json
    - raf_workplan::escape::tests::each_control_and_bidirectional_formatting_character_is_escaped
    - raf_workplan::escape::tests::lines_keep_their_line_feeds_and_tabs_and_lose_every_other_control
    - cargo_workplan::test_crate::evidence::the_words_after_the_separator_are_the_command_options_included
    - cargo_workplan::test_crate::history::a_capability_lists_its_document_and_symbol_commits_newest_first_and_a_removed_binding
    - cargo_workplan::test_crate::history::a_shallow_clone_is_exit_two_and_its_full_clone_answers
    - cargo_workplan::test_crate::history::json_holds_the_entries_of_the_table_in_the_same_order
    - cargo_workplan::test_crate::history::quiet_prints_nothing_because_a_history_is_not_a_finding
    - cargo_workplan::test_crate::history::since_limits_the_commits_and_a_change_or_an_unknown_id_is_answered_or_refused
    - cargo_workplan::test_crate::process::an_id_a_day_or_a_value_of_the_record_commands_is_refused_before_any_process_starts
    - cargo_workplan::test_crate::process::history_asks_the_git_version_before_its_log_and_refuses_a_git_older_than_2_30
    - cargo_workplan::bin_crate::cargo_workplan::evidence::tests::a_program_named_the_separator_and_a_binding_beside_one_are_refused
    - cargo_workplan::test_crate::coverage::a_surface_that_cannot_be_listed_is_a_finding_of_the_code_report_only
    - cargo_workplan::test_crate::evidence::a_replacement_holds_the_spdx_lines_of_its_own_run_only
    - cargo_workplan::test_crate::history::a_bound_file_that_is_not_committed_is_exit_two_naming_it_until_it_is
    - cargo_workplan::bin_crate::cargo_workplan::cli::tests::strict_is_a_global_option_off_by_default_and_taken_before_or_after_the_command
    - cargo_workplan::test_crate::check::strict_check_exits_zero_on_the_strict_clean_repository
    - cargo_workplan::test_crate::check::strict_check_exits_one_on_one_thin_section_and_check_without_the_flag_exits_zero
    - cargo_workplan::test_crate::check::strict_check_exits_two_at_a_root_without_capability_documents
    - cargo_workplan::test_crate::check::check_without_the_flag_judges_strict_when_the_record_says_strict
    - cargo_workplan::test_crate::check::a_strict_finding_in_json_is_an_object_of_its_kind_with_the_fields_of_a_finding
    - cargo_workplan::test_crate::complete::strict_refuses_a_completion_that_the_default_gate_accepts
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

The binary is `cargo-workplan`, a thin wrapper over `raf-workplan`, parsed with clap
derive. It runs from any directory inside the workspace and finds the root from
`cargo metadata`. Cargo runs it for `cargo workplan <command>`, passing the argument
vector `cargo-workplan workplan <command>`, and it runs directly as
`cargo-workplan <command>`; both forms take the same arguments and give the same
result.

**Root discovery.** When `--root` is absent, root discovery in `raf-workplan` spawns
exactly this command, directly with no shell, as an argv vector. The program is the
one named by `$CARGO` when that is set (cargo sets it for a subcommand), and `cargo`
from `PATH` otherwise:

```text
cargo metadata --format-version 1 --no-deps --frozen
```

`--no-deps` skips dependency resolution, which is not needed to find the root, and
`--frozen` forbids network access and any change to `Cargo.lock`. No value from a
document is placed on the command line. Even so, cargo reads the workspace's
`.cargo/config.toml` and the rustup proxy reads its `rust-toolchain.toml`, so the
workspace is trusted build input, as it is for `cargo build` (see
[the security policy](../../SECURITY.md)). The binary reads no JSON beyond the
`workspace_root` field and bounds the output at 64 MiB. Root discovery runs before
the processes of the command itself.

| Command | Effect |
| --- | --- |
| `check` | Run every rule of the [document model](./workplan-document-model.md); print each violation |
| `status` | Print the roadmap: the capabilities of each state, the gaps of each implementing one, and the open changes |
| `docs sync` | Rewrite the generated blocks ([generated documents](./workplan-generated-docs.md)) |
| `docs preview <slug>` | Print the capabilities of the change as they read once reconciled |
| `change new <slug> --capability cap:<name>` | Create a change record |
| `change impact <slug>` | Print what the change touches |
| `change reconcile <slug>` | Print the obligation list ([reconcile](./workplan-reconcile.md)) |
| `change complete <slug>` | Run the [completion gate](./workplan-complete-gate.md) and delete the record |
| `verify [<slug>]` | Run the bound tests and examples |
| `history <id> [--since <YYYY-MM-DD>]` | Print the commits that shaped a document or record ([entity history](./workplan-entity-history.md)) |
| `evidence record --id <slug> --for cap:<slug>... --by human\|agent [--limit <name>=<number>]... [--revision <commit>] [--update] [--copyright <text> --licence <id>]` and `-- <program> <arg>...` or `--from-binding <path>` | Run the command or the bound test and write its evidence record ([evidence claims](./workplan-evidence-claims.md)) |
| `evidence claims <evidence-id>` | Print the statements that rest on an evidence record |
| `evidence for cap:<slug>` | Print the evidence records that support a capability |
| `assess cap:<slug>#<section> --context` | Print what an assessor reads ([prose assessment](./workplan-prose-assessment.md)) |
| `assess cap:<slug>#<section> --verdict accepted\|rejected --by human\|agent [--copyright <text> --licence <id>]` | Record the assessment of the section |
| `assess [cap:<slug>#<section>] --list` | Print each assessment, or the one of the section, and whether it is current |
| `coverage code\|tests [--crate <package>]` | Print the public items or the tests that no capability binds ([two-way coverage](./workplan-two-way-coverage.md)) |
| `coverage claims [--capability cap:<slug>]` | Print every claim of every delivered capability, or of the one named, with its key, line and tests ([strict mode](./workplan-strict.md)) |
| `coverage lines [--crate <package>]`, also `coverage --reverse [--crate <package>]` | Print the reverse coverage of every package, or of the one named: its counted, covered, excepted and orphan lines and ceilings, the modules that hold an orphan line and the orphan lines of each unit ([strict mode](./workplan-strict.md)) |

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

**Values of the record commands.** Each value is checked when the arguments are
parsed. A history id is `cap:`,
`change:`, `arch:`, `evidence:` or `assessment:` and a slug; an evidence id is
`evidence:<slug>` or the slug; a section is `cap:<slug>#<section>` with one of the
seven sections; `--since` is a day from 1970-01-01 to 2099-12-31; `--limit` is
`name=number` with no exponent; `--revision` is 40 or 64 lowercase hexadecimal
digits; `--crate` is a package name, given as cargo names it (`raf-okf`) or as its
crate is named (`raf_okf`). A value that is not one, or that starts with `-`, is a
usage error, exit 2, before any process starts.

**The command of `evidence record`.** Every word after the first `--` of the
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
since a record states who made it. A replacement, by `--update` or by a verdict on a section that has an
assessment, holds only what its own run gives, its SPDX lines included: without
`--copyright` and `--licence` the new record has none.

**Exit codes.** 0 means success: no violations, an empty obligation list, a completed
change, or every binding passing. 1 means findings: violations, obligations, a refused
completion or a failing binding. 2 means the tool could not run: a usage error, an
unreadable file, a missing `git`, a slug that does not exist. A Git older than 2.30 is
exit 2 for `change reconcile` and `change complete`, a change record that does not
load (an `outcome` that is not one of the three values, for instance) is exit 2, and
`status` exits 0 whatever gaps it lists, since a gap is not a violation.

**Exit codes of the record commands.** `history`, `evidence claims`,
`evidence for` and `assess` exit 0 when they have an answer: a recorded `rejected`
verdict, and a stale or rejected assessment that `--list` shows, are results and
not findings, and `change reconcile` lists what they owe. `evidence record` exits 0
when it writes a `pass` record and 1 when it writes a `fail` record, a result the
repository owes; it exits 2 when it writes none: a printed value that is not a
number, a name printed twice, a limit the command never printed, output past its
bound, a command that cannot start, or a request refused before the run.
`coverage code` and `coverage tests` exit 1 when the report names a gap, or in the
code direction a surface that cannot be listed, and 0 otherwise. `coverage claims`
exits 1 when a claim of a delivered capability has no test and 0 when every claim
has one, under either strictness. `coverage lines` exits 1 when it prints an orphan
line or could not count the lines and 0 otherwise, and `coverage --reverse` is the
same command, which prints the same bytes. Each exits 2 when
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

**Options.** `--root <dir>` sets the workspace root, `--format text|json` selects the
output, `-q` suppresses everything but findings, and `--strict` judges with
`Gate::workspace(root).strict()`: it raises the strictness of `check`,
`change reconcile` and `change complete` to [strict mode](./workplan-strict.md), and a
coverage record that says `strictness: strict` gives the same judgement without it.
`status`, `docs`, `history`, `evidence`, `assess`, `verify` and the coverage reports
print the same with and without `--strict`. Findings are printed to stdout in
a stable order and diagnostics to stderr. Each option is accepted before or after the
command. Under `-q` a command whose output holds no finding (`status`, `docs sync`,
`docs preview`, `change new`, `change impact`, a completion, the passing bindings of
`verify`) prints nothing on stdout and leaves out its notes on stderr, while the
obligations, a refusal and the failing bindings are findings and stay; a failure of
the tool is always written to stderr, and the exit code does not change.

**JSON.** `--format json` prints one JSON array for the run on stdout: one object for
each record the text form prints, in the same order, and `[]` when there is none.
Every object begins with `kind`, the discriminator, followed by the fields of its kind,
always present, with `null` for a field that does not apply. A finding of `check` has
the kind of its rule, from `Finding::KINDS`, and the fields `path`, `section`, `line`,
`key`, `message` and `fix`, a finding of strict mode as any other. The other kinds
are `rewrote`, `state`, `open-changes`, `preview`, `new-change`, `impact`,
`obligation`, `refused`, `refusal` (one for each failure of a refused completion,
with its condition), `completed` and `binding`, and none of them equals the kind of
a rule. Their fields are documented in the crate documentation of `cargo-workplan`.
A run that could not run prints nothing on stdout, so stdout is either one complete
array or empty.

**Output of the record commands.** `history` and `coverage` print the table and
the JSON array that `raf-workplan` renders, as one report: under `-q` a history
prints nothing (`[]` in JSON), and a coverage report is printed whole when it names
a gap and not at all when it names none, as a claims report is printed whole when a
claim names no test and not at all when every claim names one. Under `-q` a `pass` record, a recorded
verdict, a context, a list and the two evidence views print nothing, and a `fail`
record stays. The JSON kinds are `history-entry`, `history-removed`,
`evidence-recorded`, `evidence-claim`, `evidence-for`, `assess-context`,
`assessment-recorded`, `assessment`, `coverage-gap`, `coverage-excluded`,
`coverage-error`, `coverage-claim`, `lines-crate`, `lines-module`, `lines-orphan`,
`lines-excepted` and `lines-error`; none equals the kind of a rule, and their
members, in a fixed order, are documented in the crate documentation of
`cargo-workplan`. A lines report is printed whole under `-q` when it holds an orphan
line and not at all when it holds none.

> [!PROPOSED change:workplan-strict]
> **Strict mode.** The last command of [strict mode](./workplan-strict.md) joins the
> table: `debt --write [--copyright <text> --licence <id>]` writes the strict debt
> record `work/debt/default.md` through `Gate::write_debt`. It exits 0 when it
> writes, 1 when it refuses with the findings printed (growth, or a finding of the
> default rules), and 2 when the coverage record does not say `strictness: strict`
> or the tool cannot run. The JSON kind `debt-written` joins the record kinds, and it
> equals the kind of no rule. `debt --write` writes one file and the reports write
> nothing.

## Usage

```sh
cargo workplan check
cargo workplan status
cargo workplan change new incremental-parsing --capability cap:streaming-parser
cargo workplan docs preview incremental-parsing
cargo workplan change complete incremental-parsing
cargo-workplan --format json change reconcile incremental-parsing
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
- `cargo workplan --format json coverage code --crate raf-workplan` prints the
  public items of one package that no capability binds.
- `cargo workplan --strict check` judges the repository under strict mode before
  its coverage record says `strictness: strict`, and `cargo workplan check` judges
  it so once the record says it.
- `cargo workplan coverage claims --capability cap:streaming-parser` prints each
  claim of the capability and the tests its markers name.
- `cargo workplan coverage lines --crate raf-okf`, or
  `cargo workplan coverage --reverse --crate raf-okf`, prints the orphan lines of
  one package by crate, module and item.

> [!PROPOSED change:workplan-strict]
> The last command of strict mode is used the same way:
> `cargo workplan debt --write --copyright '2026 Example Inc.' --licence CC-BY-4.0`
> writes `work/debt/default.md` with its two SPDX lines.

## Examples

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

`--format json` prints one array for the run, with one object per finding, for
scripts and agents:

```text
$ cargo workplan --format json check
[
{"kind":"link-broken","path":"docs/capabilities/alpha-store.md","section":null,"line":12,"key":null,"message":"broken link `./gone.md`","fix":"point the link at a tracked path, or remove it"}
]
```

`evidence record` reports the record it wrote, and its exit code says whether the
result holds:

| Command after `evidence record --id peak --for cap:alpha --by agent --limit peak=64 --` | Standard output | Exit |
| --- | --- | --- |
| `/bin/sh -c 'echo peak=41'` | `recorded: work/evidence/peak.md (pass)` | 0 |
| `/bin/sh -c 'echo peak=80'`, with `--update` | `recorded: work/evidence/peak.md (fail): over the limit: peak` | 1 |
| `/bin/sh -c 'echo peak=1e-3'` | nothing: the refusal is on standard error and no record is written | 2 |

## Constraints

- The binary holds no logic of its own. Each command calls a function in
  `raf-workplan`, so the dev-dependency `Gate` and the CLI agree.
- `check`, `status`, `docs preview`, `change impact` and `change reconcile` write
  nothing in the repository; a bound example that `change reconcile` builds leaves
  only the build output of cargo. `docs sync` writes only generated blocks.
  `change new` writes one file. `change complete` deletes one file, and before it
  rewrites only the generated blocks that the deletion stales.
- `history`, `evidence claims`, `evidence for`, `assess --context`, `assess --list`
  and `coverage` write nothing. `evidence record` writes one record,
  `work/evidence/<id>.md`, and only after the run; `assess --verdict` writes one
  record, `work/assessments/<capability slug>-<section>.md`; neither edits a
  document.
- `verify` needs no Git beyond the listing of tracked paths and writes nothing in the
  repository; `status` exits 0 when it lists gaps.
- Every process the binary starts, directly or through `raf-workplan`, is spawned
  with no shell and an argv vector. Each spec that spawns lists its exact
  invocation, and every `cargo` command the tool composes carries `--frozen`. A
  value taken from a document reaches a process only after resolution and a stated
  pattern, and then in one of four forms: after `--`; inside a single
  `<rev>:<path>` or `-L<start>,<end>:<path>` argument; as a validated identifier
  inside one `-S<identifier>` argument of the history; or as a validated test
  filter (see [the security policy](../../SECURITY.md)).
- The binary starts no process of its own: each process is started through
  `raf-workplan`. The caller's command of `evidence record` is the one process whose
  argv comes from the command line, and `raf-workplan` runs it with no shell,
  exactly as given; no text of a document or record reaches it.
- A slug is lowercase letters and digits in parts joined by single `-`, so it starts
  with a letter or a digit; any other slug, one that starts with `-` included, is a
  usage error.
- A command that takes the slug of a change record (`docs preview`, `change impact`,
  `change reconcile`, `change complete` and `verify <slug>`) fails with exit 2 when
  `work/changes/<slug>.md` does not exist; `change new` fails with exit 2 when it
  exists.
- Document text reaches the terminal escaped. The text output of `evidence claims`
  (each statement) and of `assess --context` (the section and the bound code)
  writes each control character other than the line feed and the tab, a carriage
  return, an escape and a bell included, and each bidirectional formatting
  character (U+061C, U+200E, U+200F, U+202A to U+202E, U+2066 to U+2069) as its
  `\u{..}` escape (`raf_workplan::escape::shown_lines`), as the history and
  coverage tables escape their cells, so a document cannot clear the screen, set the
  title of a terminal or reorder a line. The JSON form carries the text itself:
  JSON escapes the control characters U+0000 to U+001F, and every other
  character, a bidirectional formatting character included, is written as it is,
  for the reader of the JSON to show safely.

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

- An argument-parsing test per command and option, including `--capability`
  repetition and `--format`.
- A fixture repository per command asserting stdout, files written and the exit code,
  with exit 0, 1 and 2 each exercised: a clean check, a finding, and a missing slug.
- A root-discovery fixture with a stub named by `$CARGO`, and one on `PATH` with
  `$CARGO` unset, records the argv and asserts
  it equals `cargo metadata --format-version 1 --no-deps --frozen`. A second fixture
  has a `.cargo/config.toml` with `build.rustc` set to a script, and asserts that
  root discovery does not run the script.
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
- JSON fixtures: one array per run with a `kind` first in each object, the kinds that
  are not rules never equal to a rule name, and stdout empty for a run that could not
  run.

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
  a bare `--`) are included, without `--root`, so that root discovery would show. A
  recording `git` stub on `PATH` that answers the version shows that `history` asks
  `git --version` before its log, refuses 2.29 and goes on at 2.30.
- Fixtures through the built binary for each command. `history` on a real Git
  history: the document and symbol commits newest first, a removed binding with the
  commit that removed it and its parent, `--since`, the same entries in the table and
  in JSON, a shallow clone refused, and a bound file that is not committed refused
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
  counted line is covered, 1 with an orphan helper in text and JSON, 2 for a package
  that is not one, and `coverage --reverse` printing the same bytes in each case.
- JSON pins of the members of each kind of the record commands in order, and every
  kind unequal to a rule name; `-q` hides each outcome and keeps a `fail` record and a
  coverage gap; the read-only commands leave the tree byte-identical.
- An escape sequence that clears the screen, an OSC title ended by a bell and a
  right-to-left override in a claimed statement and in a section: the text output
  of `evidence claims` and of `assess --context` holds none of them raw and shows
  each as its `\u{..}` escape, keeping the line feeds and the tabs, and the
  context equals the library's text escaped; the JSON form holds no raw escape,
  which it writes as JSON escapes a control character, and reads back as the text;
  plain text is printed unchanged.
- `evidence claims` while a capability document does not load: exit 2 in text and
  JSON, naming the document on standard error, in text with nothing on standard
  output, and nothing written, for an id with claims and for one without; a change
  record or an evidence record that does not load leaves the answer, exit 0, while
  `check` reports it; the clean neighbour answers.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_command_that_takes_a_slug_refuses_one_that_starts_with_a_dash_in_both_forms` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_format_that_is_not_text_or_json_is_a_usage_error` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_missing_command_an_unknown_command_and_a_missing_slug_are_usage_errors` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_must_be_a_slug` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::a_slug_that_starts_with_a_dash_is_a_usage_error_however_it_is_given` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::assess_takes_a_section_and_exactly_one_mode` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_needs_a_capability_and_both_or_neither_spdx_option` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::change_new_takes_a_slug_repeated_capabilities_and_affects_and_the_spdx_pair` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_claims_takes_an_optional_capability` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_lines_takes_an_optional_package_and_reverse_is_the_same_command` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::coverage_takes_a_direction_and_an_optional_package` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::each_command_parses_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::evidence_claims_and_for_take_one_id_each` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::evidence_record_reads_every_option_and_the_command_after_the_separator` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::evidence_record_refuses_a_separator_as_a_value_and_a_record_it_cannot_write` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::help_and_version_exit_zero_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::history_refuses_an_id_or_a_day_before_anything_runs` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::history_takes_an_id_of_each_kind_and_a_day` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::options_after_the_separator_belong_to_the_command` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::strict_is_a_global_option_off_by_default_and_taken_before_or_after_the_command` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_default_and_parse_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_options_may_follow_the_command` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::cli::tests::the_sync_text_options_parse` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::evidence::tests::a_program_named_the_separator_and_a_binding_beside_one_are_refused` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::evidence::tests::the_command_runs_only_when_both_readings_of_the_line_agree` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::exit::tests::a_library_error_becomes_its_display_text` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::exit::tests::the_three_exits_have_the_documented_codes` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::a_finding_makes_the_exit_one_and_an_outcome_does_not` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::a_json_object_begins_with_its_kind_and_keeps_the_order_of_its_fields` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::a_record_with_no_text_has_no_line_in_text_and_an_object_in_json` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::a_rendered_block_is_printed_as_given_and_quiet_hides_only_an_outcome` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::an_empty_report_is_nothing_in_text_and_an_empty_array_in_json` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::quiet_prints_only_findings_and_keeps_the_exit` | test | [Verification](#verification) |
| `cargo_workplan::bin_crate::cargo_workplan::render::tests::text_with_quotes_and_control_characters_is_escaped` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::assess::a_section_that_cannot_be_assessed_is_exit_two_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::assess::a_verdict_writes_exactly_one_record_and_a_rejection_still_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::assess::the_context_is_the_same_whoever_asks_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::assess::the_list_shows_each_assessment_current_until_an_input_moves` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::assess::the_text_of_a_section_reaches_the_terminal_escaped_and_json_escapes_it_as_json` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_clean_repository_exits_zero_and_prints_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_directory_that_is_not_a_repository_exits_two_and_says_why_on_standard_error` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_finding_is_gone_once_it_is_fixed` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_manifest_that_does_not_parse_exits_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_repository_with_findings_exits_one_and_prints_each_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_root_that_does_not_exist_exits_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_source_file_that_cannot_be_read_exits_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_stale_generated_block_is_a_finding_and_a_sync_clears_it` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::a_strict_finding_in_json_is_an_object_of_its_kind_with_the_fields_of_a_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::check_without_the_flag_judges_strict_when_the_record_says_strict` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::help_and_version_exit_zero_on_standard_output` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::quiet_keeps_the_findings_and_the_exit_code` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::quiet_on_a_clean_repository_prints_nothing_and_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::strict_check_exits_one_on_one_thin_section_and_check_without_the_flag_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::strict_check_exits_two_at_a_root_without_capability_documents` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::strict_check_exits_zero_on_the_strict_clean_repository` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_exit_agrees_with_assert_clean_and_every_finding_is_in_its_message` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::the_findings_are_those_of_the_library_gate_in_the_same_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::check::usage_errors_exit_two_with_a_message_on_standard_error` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::claims::a_capability_that_is_not_loaded_exits_two_and_says_why` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::claims::a_claim_that_names_no_test_is_a_finding_and_exits_one_in_text_and_json_alike` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::claims::every_claim_names_a_test_prints_the_report_and_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::a_capability_left_implementing_is_refused_and_nothing_is_written` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::a_stale_generated_block_is_refused_until_the_sync_is_run` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::both_shapes_complete_the_same_way` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::every_failure_is_listed_and_each_condition_is_named_in_json` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::from_new_to_a_completed_change_through_the_binary` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::json_for_a_completion_is_a_completed_object_and_a_rewrote_object_for_each_file` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::quiet_hides_a_completion_and_keeps_a_refusal` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::strict_refuses_a_completion_that_the_default_gate_accepts` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::complete::what_stops_it_looking_is_exit_two_and_the_record_stays` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::coverage::a_surface_that_cannot_be_listed_is_a_finding_of_the_code_report_only` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::coverage::an_exclusion_takes_a_gap_out_and_a_report_without_gaps_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::coverage::crate_selects_a_package_by_either_name_and_refuses_one_that_is_not_there` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::coverage::gaps_in_either_direction_are_printed_and_exit_one` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::coverage::this_repository_names_the_gaps_outside_its_scope_and_none_in_it` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::a_bound_test_gives_the_verdict_and_a_path_that_names_none_starts_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::a_claim_on_a_recorded_result_reconciles_clean_until_its_bound_code_moves` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::a_claimed_statement_reaches_the_terminal_escaped_and_json_escapes_it_as_json` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::a_command_that_passes_writes_exactly_one_record_and_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::a_failing_command_or_a_value_over_its_limit_records_fail_and_exits_one` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::a_replacement_holds_the_spdx_lines_of_its_own_run_only` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::a_request_that_cannot_be_recorded_runs_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::cargo_passes_the_command_after_the_separator_to_the_binary_unchanged` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::claims_exit_two_naming_a_capability_that_does_not_load_and_answer_beside_other_records` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::json_is_one_evidence_recorded_object_with_its_fields` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::output_the_tool_cannot_read_is_exit_two_and_writes_no_record` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::the_views_print_json_and_refuse_what_does_not_exist` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::evidence::the_words_after_the_separator_are_the_command_options_included` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::a_bound_file_that_is_not_committed_is_exit_two_naming_it_until_it_is` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::a_capability_lists_its_document_and_symbol_commits_newest_first_and_a_removed_binding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::a_shallow_clone_is_exit_two_and_its_full_clone_answers` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::json_holds_the_entries_of_the_table_in_the_same_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::quiet_prints_nothing_because_a_history_is_not_a_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::since_limits_the_commits_and_a_change_or_an_unknown_id_is_answered_or_refused` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::a_change_that_reaches_dependants_and_linking_sections_lists_them_sorted` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::a_record_that_does_not_load_is_exit_two_with_its_findings` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::a_slug_with_no_record_is_exit_two_and_names_the_open_changes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::impact_finds_the_root_from_below_and_by_root` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::impact_prints_what_the_change_reaches_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::json_is_one_impact_object_with_its_lists` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::impact::quiet_prints_nothing_and_the_exit_stays_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::a_clean_run_prints_one_empty_array` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::a_run_that_could_not_run_prints_nothing_on_standard_output_in_json_too` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::check_prints_one_object_for_each_finding_of_the_gate_with_the_fields_of_its_location` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::every_object_is_a_single_line_inside_one_array` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::preview_prints_a_preview_object_for_each_capability_with_the_reconciled_text` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::quiet_keeps_the_findings_in_json_and_prints_an_empty_array_without_any` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::status_prints_a_state_object_for_each_state_and_the_open_changes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::sync_prints_a_rewrote_object_for_each_file_and_an_empty_array_for_none` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::the_json_and_the_text_forms_print_the_same_findings_in_the_same_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::json::the_kinds_that_are_not_rules_are_not_rule_names` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::a_package_that_is_not_one_exits_two_and_says_why` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::a_repository_whose_every_counted_line_is_covered_prints_the_report_and_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::lines::an_orphan_line_is_a_finding_and_exits_one_in_text_and_json_alike_and_reverse_prints_the_same` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::a_capability_given_twice_is_listed_once` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::a_capability_that_is_missing_or_a_bad_option_is_a_usage_error_before_anything_is_read` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::a_second_run_for_the_same_slug_is_exit_two_and_leaves_the_record_as_it_is` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::affects_is_repeatable_and_the_record_holds_the_entries_given_in_order` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::every_refusal_of_the_library_is_exit_two_with_the_tree_untouched` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::json_is_one_new_change_object_with_its_fields` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::new_prints_the_library_text_writes_the_record_and_nothing_else_in_both_shapes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::new_works_from_a_subdirectory_and_by_root` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::quiet_writes_the_record_and_prints_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::spdx_text_that_is_not_one_line_is_exit_two_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_note_about_the_generated_blocks_is_there_when_the_record_makes_them_stale` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_printed_template_in_a_capability_of_each_state_gives_a_clean_check_and_one_unresolved_proposal` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_spdx_lines_are_the_two_comment_lines_of_the_records_of_this_repository` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::new::the_template_left_in_a_state_that_refuses_it_is_the_finding_the_guidance_names` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::a_capability_the_change_lists_that_has_no_document_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::a_change_record_that_did_not_load_is_exit_two_and_says_so` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::a_slug_that_is_not_a_slug_is_a_usage_error` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::an_unknown_slug_is_exit_two_and_names_the_open_changes` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::quiet_prints_nothing_because_a_preview_is_not_a_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::the_output_is_the_text_the_library_previews_under_a_header_for_each_document` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::the_preview_is_the_same_by_root_from_another_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::preview::the_proposed_text_is_printed_and_the_markers_are_not` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_cargo_that_cannot_be_started_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_cargo_that_fails_is_exit_two_with_what_it_said` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_missing_git_is_exit_two_and_names_it` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_slug_that_is_not_a_slug_is_refused_before_any_process_starts` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_before_any_process_starts` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_slug_that_starts_with_a_dash_is_refused_by_every_command_before_any_process_starts` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_usage_error_of_the_change_commands_starts_no_process` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::a_usage_error_starts_no_process` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::an_id_a_day_or_a_value_of_the_record_commands_is_refused_before_any_process_starts` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::cargo_dispatches_workplan_to_the_binary_on_path_and_passes_the_exit_code_back` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::discovery_does_not_run_the_build_rustc_program_of_the_workspace` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::discovery_runs_exactly_cargo_metadata_from_the_cargo_variable` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::discovery_runs_the_cargo_on_path_when_the_variable_is_unset` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::history_asks_the_git_version_before_its_log_and_refuses_a_git_older_than_2_30` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::output_from_cargo_that_is_not_the_metadata_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::process::root_starts_no_cargo_at_all` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::a_bound_example_that_fails_is_an_obligation_and_nothing_is_written_to_the_tree` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::everything_that_stops_it_looking_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::json_is_one_obligation_object_for_each_obligation` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::nothing_is_written_and_no_cargo_is_started_for_a_fixture_without_bound_examples` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::quiet_keeps_the_obligations_and_hides_nothing_that_is_a_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::a_directory_outside_any_workspace_is_exit_two_and_names_the_command` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::a_relative_root_is_taken_from_the_current_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::every_command_finds_the_root_from_a_subdirectory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::root_overrides_the_discovery_and_the_current_directory_is_not_read` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::sync_from_a_subdirectory_writes_at_the_root_and_nowhere_else` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::root::the_root_is_found_from_the_root_and_from_any_directory_below_it` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::a_directory_that_is_not_a_repository_is_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::a_state_with_no_capability_is_a_row_with_a_count_of_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::an_implementing_capability_with_every_binding_resolved_has_no_gap_beyond_its_empty_lists` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::quiet_leaves_standard_output_empty_because_the_roadmap_has_no_finding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::the_roadmap_groups_one_capability_in_each_state_with_the_gaps_and_the_change` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::status::the_roadmap_is_the_same_by_root_from_another_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_document_that_does_not_load_is_exit_two_and_nothing_is_written` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_missing_index_with_the_text_is_created_with_that_header` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_missing_index_without_the_text_is_exit_two_naming_the_field_and_writes_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_refused_file_is_exit_two_and_nothing_is_written` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_second_sync_writes_nothing_and_prints_nothing` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::a_value_that_cannot_be_written_into_the_header_is_refused_when_the_index_is_created` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::each_field_that_is_not_given_is_named` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::only_the_generated_blocks_are_rewritten` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::quiet_hides_the_files_rewritten_and_keeps_the_exit_code` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::the_index_is_created_by_root_from_another_directory` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::sync::the_text_is_not_read_when_the_index_exists` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::a_failing_example_and_a_binding_that_proves_nothing_are_failures` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::a_hostile_binding_is_a_failure_and_never_reaches_cargo` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::a_slug_with_no_record_and_a_document_that_does_not_load_are_exit_two` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::every_binding_passing_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::json_is_one_binding_object_for_each_binding` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::no_slug_runs_every_capability_and_a_slug_only_the_ones_the_change_lists` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::nothing_to_run_is_exit_zero_with_a_note_that_quiet_hides` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::verify::quiet_prints_only_the_failing_bindings_and_keeps_the_exit` | test | [Verification](#verification) |
| `raf_workplan::Workplan::impact` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::new_change` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::preview` | doctest | [Verification](#verification) |
| `raf_workplan::Workplan::sync` | doctest | [Verification](#verification) |
| `raf_workplan::escape::shown_lines` | doctest | [Constraints](#constraints) |
| `raf_workplan::escape::tests::each_control_and_bidirectional_formatting_character_is_escaped` | test | [Verification](#verification) |
| `raf_workplan::escape::tests::lines_keep_their_line_feeds_and_tabs_and_lose_every_other_control` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::a_large_text_is_scanned_to_its_end` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::a_member_that_is_only_nested_is_not_the_member` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::a_member_twice_is_refused_even_when_the_values_agree` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::a_top_level_value_that_is_not_an_object_is_refused` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::a_value_that_is_not_a_string_is_refused` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::an_unpaired_surrogate_is_text_nobody_can_use` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::escapes_are_decoded` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::every_prefix_of_a_document_is_refused_and_no_mutation_panics` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::nesting_is_bounded_and_never_overflows_the_stack` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::numbers_of_the_grammar_pass` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::random_strings_of_json_punctuation_never_panic` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::text_that_is_not_json_is_refused` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::the_member_is_found_wherever_it_sits_at_the_top_level` | test | [Verification](#verification) |
| `raf_workplan::root::json::tests::the_output_of_cargo_metadata_gives_its_root` | test | [Verification](#verification) |
| `raf_workplan::root::tests::a_failing_cargo_is_an_error_with_what_it_said` | test | [Verification](#verification) |
| `raf_workplan::root::tests::a_root_that_is_not_an_absolute_path_is_refused` | test | [Verification](#verification) |
| `raf_workplan::root::tests::cargo_that_cannot_run_or_writes_too_much_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::root::tests::output_that_does_not_give_a_root_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::root::tests::the_root_is_the_workspace_root_member_and_the_command_is_exactly_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::a_cargo_that_does_not_exist_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::a_directory_outside_any_workspace_is_an_error_that_says_what_cargo_said` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::configured_programs::discovery_does_not_run_the_programs_of_the_cargo_configuration` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::stubs::exactly_one_process_runs_with_the_documented_argv_directory_and_locale` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::stubs::hostile_output_is_an_error_and_never_a_guess` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::stubs::output_is_bounded_at_64_mebibytes` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::stubs::the_top_level_member_wins_over_a_nested_one_with_the_same_name` | test | [Verification](#verification) |
| `raf_workplan::test_crate::root::the_root_is_found_from_the_root_a_member_and_a_subdirectory` | test | [Verification](#verification) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-complete-gate](./workplan-complete-gate.md) |
| Depends on | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depends on | [workplan-reconcile](./workplan-reconcile.md) |
| Changed by | [workplan-strict](../../work/changes/workplan-strict.md) |
<!-- raf:generated:end -->
