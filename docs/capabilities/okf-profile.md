---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:okf-profile
title: "OKF profile reader and writer"
description: "The raf-okf crate: parse OKF Markdown, validate the RAF profile with closed key sets, and write documents back deterministically."
state: delivered
bindings:
  code:
    - raf_okf::profile::build
    - raf_okf::profile::schema
    - raf_okf::profile::check_values
    - raf_okf::profile::diagrams
    - raf_okf::DiagramDeclaration
    - raf_okf::DiagramKind
    - raf_okf::DiagramGrouping
    - raf_okf::Document
    - raf_okf::dataset
    - raf_okf::identity
    - raf_okf::profile::id_slug
    - raf_okf::DocType::document_iri
    - raf_okf::Document::to_dataset
    - raf_okf::Document::from_dataset
    - raf_okf::Body
    - raf_okf::Profile
    - raf_okf::Index
    - raf_okf::is_capability_index
    - raf_okf::Span
    - raf_okf::RafDocument
    - raf_okf::EvidenceRecord
    - raf_okf::AssessmentRecord
    - raf_okf::CoverageRecord
    - raf_okf::Claim
    - raf_okf::TestedBy
    - raf_okf::EvidenceRecord::new
    - raf_okf::EvidenceRecord::to_document
    - raf_okf::AssessmentRecord::new
    - raf_okf::AssessmentRecord::to_document
    - raf_okf::CoverageRecord::new
    - raf_okf::CoverageRecord::to_document
    - raf_okf::CoverageRecord::with_ceilings
    - raf_okf::CoverageRecord::with_exceptions
    - raf_okf::Ceiling
    - raf_okf::CeilingClass
    - raf_okf::CeilingLimit
    - raf_okf::Exception
    - raf_okf::ExceptionTarget
    - raf_okf::Number::compare
    - raf_okf::Number::exceeds
    - raf_okf::DebtRecord
    - raf_okf::DebtFinding
    - raf_okf::DebtLine
    - raf_okf::DebtRecord::new
    - raf_okf::DebtRecord::to_document
    - raf_okf::DebtFinding::new
    - raf_okf::DebtLine::new
    - raf_okf::Actor
    - raf_okf::Affects
    - raf_okf::Architecture
    - raf_okf::AssessmentVerdict
    - raf_okf::Bindings
    - raf_okf::Capability
    - raf_okf::CapabilitySection
    - raf_okf::Change
    - raf_okf::ChangeItem
    - raf_okf::Comments
    - raf_okf::ConstructError
    - raf_okf::ContractEffect
    - raf_okf::DocType
    - raf_okf::ESCAPES
    - raf_okf::Entry
    - raf_okf::ErrorKind
    - raf_okf::ErrorTag
    - raf_okf::EvidenceSubject
    - raf_okf::EvidenceVerdict
    - raf_okf::Exclusion
    - raf_okf::ExclusionItem
    - raf_okf::FencedBlock
    - raf_okf::Frontmatter
    - raf_okf::GeneratedBlock
    - raf_okf::Header
    - raf_okf::Heading
    - raf_okf::Input
    - raf_okf::InputKind
    - raf_okf::ItemValue
    - raf_okf::KeySpec
    - raf_okf::Link
    - raf_okf::LinkError
    - raf_okf::LinkForm
    - raf_okf::LinkTarget
    - raf_okf::List
    - raf_okf::ListForm
    - raf_okf::ListItem
    - raf_okf::Location
    - raf_okf::MAX_BLOCK_NESTING
    - raf_okf::MAX_TEXT_BYTES
    - raf_okf::MAX_YAML_DEPTH
    - raf_okf::Map
    - raf_okf::Measure
    - raf_okf::Number
    - raf_okf::Outcome
    - raf_okf::ParseError
    - raf_okf::ProposedBlock
    - raf_okf::ProseLine
    - raf_okf::RecordHeader
    - raf_okf::Rule
    - raf_okf::RuleTag
    - raf_okf::Scalar
    - raf_okf::ScalarStyle
    - raf_okf::Section
    - raf_okf::SectionRef
    - raf_okf::SpliceError
    - raf_okf::SplitLink
    - raf_okf::State
    - raf_okf::TimestampError
    - raf_okf::Value
    - raf_okf::ValueKind
    - raf_okf::Violation
    - raf_okf::is_kebab
    - raf_okf::plain_ok
    - raf_okf::validate_timestamp
    - raf_okf::work
    - raf_okf::Number::exact
  tests:
    - raf_workplan::test_crate::sync::nothing_is_in_the_report_for_a_repository_that_is_in_sync
    - raf_okf::test_crate::schema::timestamp_diagnostics_preserve_lexical_calendar_and_offset_order
    - raf_okf::test_crate::profile::bindings_by_state_designed_and_considered_bind_nothing_delivered_binds_code_and_tests
    - raf_okf::test_crate::refusals::a_document_without_a_fence_on_line_one_is_refused
    - raf_okf::test_crate::refusals::a_frontmatter_that_is_not_a_block_mapping_is_refused
    - raf_okf::test_crate::refusals::a_billion_laughs_document_is_refused_at_the_first_anchor
    - raf_okf::test_crate::refusals::nesting_over_thirty_two_levels_is_refused_before_descent
    - raf_okf::test_crate::refusals::a_text_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused
    - raf_okf::test_crate::refusals::a_leading_byte_order_mark_is_refused_at_line_one_column_one
    - raf_okf::test_crate::refusals::a_nul_in_the_body_is_kept_and_renders_back
    - raf_okf::test_crate::refusals::a_frontmatter_fence_line_with_a_trailing_space_is_not_a_fence
    - raf_okf::test_crate::validate_cost::a_change_with_maximal_affects_and_changes_validates_at_the_cap
    - raf_okf::test_crate::validate_cost::a_capability_with_many_sections_and_blocks_is_refused_for_a_duplicated_section_at_the_cap
    - raf_okf::test_crate::profile::every_required_key_of_every_type_is_reported_when_missing
    - raf_okf::test_crate::profile::an_unknown_key_is_reported_at_that_key
    - raf_okf::test_crate::profile::an_unknown_or_foreign_key_is_reported_on_every_type
    - raf_okf::test_crate::profile::a_nested_unknown_key_is_reported_in_bindings_and_in_a_change_item
    - raf_okf::test_crate::profile::the_key_resource_has_its_own_rule_on_every_type
    - raf_okf::test_crate::profile::an_unknown_type_is_one_violation_at_the_type_key
    - raf_okf::test_crate::profile::a_state_is_one_of_the_four
    - raf_okf::test_crate::profile::bindings_have_exactly_the_keys_code_tests_and_examples_each_a_list
    - raf_okf::test_crate::profile::affects_entries_have_the_form_a_section_of_the_seven_and_a_capability_in_changes
    - raf_okf::test_crate::profile::items_need_a_claim_known_keys_and_a_known_outcome
    - raf_okf::test_crate::profile::an_item_reason_is_a_non_empty_line_beside_any_outcome_and_an_empty_one_is_refused
    - raf_okf::test_crate::profile::a_document_with_three_faults_returns_three_violations
    - raf_okf::test_crate::profile::a_split_link_is_reported_at_its_line_and_a_one_line_link_is_not
    - raf_okf::test_crate::profile::a_proposed_block_is_valid_only_inside_a_section_of_a_capability
    - raf_okf::test_crate::schema::each_type_lists_its_keys_in_the_order_of_the_architecture_page
    - raf_okf::test_crate::render::every_tracked_document_renders_to_its_own_bytes
    - raf_okf::test_crate::render::each_fixture_document_renders_to_its_own_bytes_and_validates
    - raf_okf::test_crate::render::a_parsed_document_equals_its_rendering_read_back
    - raf_okf::test_crate::render::keys_out_of_order_render_in_table_order_and_then_stay_put
    - raf_okf::test_crate::render::comments_attach_to_the_next_key_and_trailing_comments_stay_on_their_line
    - raf_okf::test_crate::render::tags_render_sorted_and_stay_sorted
    - raf_okf::test_crate::splice::two_generated_blocks_change_only_their_byte_ranges
    - raf_okf::test_crate::splice::overlapping_spans_are_refused_and_the_body_is_unchanged
    - raf_okf::test_crate::splice::a_malformed_result_is_refused_and_the_body_is_unchanged
    - raf_okf::test_crate::index::a_generated_only_index_is_accepted_with_either_header_form
    - raf_okf::test_crate::index::prose_outside_the_blocks_is_refused_with_its_line
    - raf_okf::test_crate::index::document_parse_of_index_text_fails_at_line_one
    - raf_okf::test_crate::index::only_the_capability_index_path_is_the_index
    - raf_okf::test_crate::markers::markers_inside_a_fence_produce_no_blocks
    - raf_okf::test_crate::markers::every_misspelling_of_the_proposed_marker_is_refused_with_its_line
    - raf_okf::test_crate::markers::each_generated_marker_error_names_its_line
    - raf_okf::test_crate::links::every_link_form_is_read_with_its_target_anchor_and_line
    - raf_okf::test_crate::links::a_link_split_across_lines_is_recorded_and_not_read
    - raf_okf::test_crate::links::a_destination_or_title_of_exactly_4096_bytes_is_read_and_one_more_is_refused
    - raf_okf::test_crate::links::targets_that_leave_the_root_or_are_empty_or_absolute_are_refused
    - raf_okf::test_crate::body_structure::block_nesting_of_sixty_four_levels_parses_and_sixty_five_is_refused
    - raf_okf::test_crate::body_structure::indented_code_is_ordinary_text_so_links_and_markers_in_it_count
    - raf_okf::test_crate::body_structure::a_fenced_block_exposes_its_content_and_whether_it_is_closed
    - raf_okf::test_crate::body_structure::a_fenced_block_prints_and_compares_without_the_body_around_it
    - raf_okf::test_crate::body_structure::only_atx_headings_are_headings_and_setext_headings_are_not
    - raf_okf::test_crate::body_structure::headings_of_every_level_carry_github_anchors_with_repeats_numbered
    - raf_okf::test_crate::alloc_bound::refused_documents_stay_within_eight_times_their_size
    - raf_okf::test_crate::placeholder_lines::reports_each_token_in_prose_with_its_line
    - raf_okf::test_crate::no_io_in_source::the_crate_sources_name_neither_the_file_system_nor_the_process_module
    - raf_okf::test_crate::usage_docs::the_examples_block_validates_between_fences_and_resource_is_rejected
    - raf_okf::test_crate::repo_markdown::no_tracked_markdown_file_holds_a_split_link_or_a_malformed_block
    - raf_okf::test_crate::schema::singleton_and_record_roles_follow_the_document_types
    - raf_okf::test_crate::records::a_valid_evidence_record_validates_to_its_typed_value
    - raf_okf::test_crate::records::a_value_over_its_limit_is_judged_on_its_text_and_not_its_nearest_float
    - raf_okf::test_crate::records::numbers_compare_exactly_by_sign_whole_part_and_fraction
    - raf_okf::test_crate::records::a_valid_assessment_validates_to_its_typed_value
    - raf_okf::test_crate::records::a_valid_coverage_record_validates_and_an_empty_scope_opts_out_visibly
    - raf_okf::test_crate::records::a_coverage_record_rejects_the_removed_policy_key
    - raf_okf::test_crate::records::a_ceiling_names_a_crate_one_whole_number_of_lines_or_a_percent_and_an_optional_class
    - raf_okf::test_crate::records::an_exception_names_one_item_prefix_file_or_directory_and_its_reason
    - raf_okf::test_crate::records::a_ceiling_of_another_shape_is_refused_with_its_rule
    - raf_okf::test_crate::records::an_exception_of_another_shape_or_without_a_reason_is_refused_with_its_rule
    - raf_okf::test_crate::records::a_verdict_is_one_of_the_verdicts_of_its_type
    - raf_okf::test_crate::records::by_is_human_or_agent_and_never_a_name
    - raf_okf::test_crate::records::a_digest_is_sixty_four_lowercase_hex_digits
    - raf_okf::test_crate::records::a_revision_is_a_full_commit_id
    - raf_okf::test_crate::records::an_exclusion_names_an_item_or_a_module_prefix_below_a_crate_root
    - raf_okf::test_crate::records::an_exclusion_has_a_reason_with_no_placeholder
    - raf_okf::test_crate::records::a_list_that_holds_each_entry_once_refuses_a_repeat
    - raf_okf::test_crate::records::coverage_is_one_record_at_its_one_path
    - raf_okf::test_crate::records::a_record_sits_in_its_directory_and_its_id_matches_its_file
    - raf_okf::test_crate::records::a_record_has_no_title_description_or_body
    - raf_okf::test_crate::claims::a_claim_marker_names_its_evidence_and_supports_the_paragraph_above_it
    - raf_okf::test_crate::claims::a_claim_may_rest_on_several_records
    - raf_okf::test_crate::claims::a_claim_after_a_list_item_supports_that_item_only
    - raf_okf::test_crate::claims::markers_in_fences_code_spans_and_proposed_blocks_are_not_read
    - raf_okf::test_crate::claims::every_other_raf_comment_is_refused_with_its_line
    - raf_okf::test_crate::claims::a_malformed_claim_marker_is_refused_with_its_line
    - raf_okf::test_crate::claims::a_claim_marker_with_no_statement_directly_above_is_refused
    - raf_okf::test_crate::claims::claims_in_a_document_carry_whole_text_lines
    - raf_okf::test_crate::render::record_keys_out_of_order_render_in_table_order_and_validate
    - raf_okf::test_crate::record_writer::a_built_evidence_record_renders_to_a_record_that_reads_back_equal
    - raf_okf::test_crate::record_writer::a_built_assessment_and_coverage_record_read_back_equal
    - raf_okf::test_crate::record_writer::a_coverage_record_writes_only_its_closed_policy_independent_keys
    - raf_okf::test_crate::record_writer::ceilings_and_exceptions_are_written_after_the_exclusions_and_read_back_equal
    - raf_okf::test_crate::record_writer::every_record_fixture_reads_back_equal_after_writing
    - raf_okf::test_crate::record_writer::the_keys_are_written_in_the_order_of_the_key_table
    - raf_okf::test_crate::record_writer::a_value_the_profile_refuses_is_written_and_refused_when_it_is_read_back
    - raf_okf::test_crate::claims::a_raf_generated_comment_in_another_case_or_spacing_is_text
    - raf_okf::test_crate::claims::a_raf_comment_split_over_lines_is_a_marker_error_and_other_split_comments_are_text
    - raf_okf::test_crate::tested_by::a_tested_by_marker_names_its_tests_and_maps_the_paragraph_above_it
    - raf_okf::test_crate::tested_by::a_tested_by_marker_maps_a_list_item_a_heading_or_the_statement_of_its_claim_marker
    - raf_okf::test_crate::tested_by::a_malformed_tested_by_marker_is_refused_with_its_line
    - raf_okf::test_crate::tested_by::a_tested_by_marker_with_no_statement_directly_above_is_refused
    - raf_okf::test_crate::tested_by::tested_by_markers_in_fences_code_spans_and_proposed_blocks_are_not_read
    - raf_okf::test_crate::tested_by::a_tested_by_marker_belongs_in_a_capability_document
    - cargo_workplan::test_crate::check::a_raf_comment_split_over_lines_fails_check_and_other_split_comments_pass
    - raf_okf::test_crate::validate_cost::an_evidence_record_with_maximal_values_and_limits_validates_at_the_cap
    - raf_okf::profile::cost_tests::a_change_at_the_cap_judges_each_affects_entry_once_against_one_set
    - raf_okf::profile::cost_tests::a_capability_at_the_cap_looks_at_each_block_once
    - raf_okf::profile::cost_tests::the_blocks_looked_at_grow_with_sections_and_blocks_together
    - raf_okf::profile::cost_tests::an_evidence_record_at_the_cap_indexes_each_value_once_and_looks_up_each_limit_once
    - raf_okf::profile::cost_tests::the_lines_tested_for_a_fence_grow_with_the_length_of_the_page
    - raf_okf::profile::cost_tests::an_unterminated_fence_is_tested_once_for_each_line_it_swallows
    - raf_okf::test_crate::records::a_valid_debt_record_validates_to_its_typed_value_and_an_empty_one_opts_out_visibly
    - raf_okf::test_crate::records::a_row_of_findings_holds_exactly_a_kind_and_a_subject_each_pair_once
    - raf_okf::test_crate::records::a_row_of_lines_holds_exactly_an_item_and_a_whole_number_of_lines_each_item_once
    - raf_okf::test_crate::records::both_lists_of_a_debt_record_are_required
    - raf_okf::test_crate::records::debt_is_one_record_at_its_one_path
    - raf_okf::test_crate::record_writer::a_debt_record_is_written_sorted_and_read_back_equal
    - raf_okf::test_crate::record_writer::a_debt_record_with_no_debt_writes_two_empty_lists
    - raf_okf::test_crate::record_writer::a_debt_row_the_profile_refuses_is_written_and_refused_when_it_is_read_back
    - raf_okf::error::tag_tests::all_lists_every_error_kind_once_in_variant_order
    - raf_okf::error::tag_tests::names_are_unique_lowercase_kebab_case
    - raf_okf::profile::violation::tag_tests::all_lists_every_rule_once_in_variant_order
    - raf_okf::profile::violation::tag_tests::names_are_unique_lowercase_kebab_case
    - raf_okf::test_crate::body_robustness::random_token_soup_never_panics_and_parsed_spans_lie_in_the_text
    - raf_okf::test_crate::body_structure::a_body_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused
    - raf_okf::test_crate::body_structure::a_document_built_with_a_malformed_body_reports_the_line_in_the_rendered_text
    - raf_okf::test_crate::body_structure::a_fence_closes_only_on_the_same_character_with_at_least_the_same_length
    - raf_okf::test_crate::body_structure::a_hundred_thousand_quote_markers_are_refused_on_a_small_stack
    - raf_okf::test_crate::body_structure::a_section_that_ends_the_body_without_a_final_newline_reaches_the_end
    - raf_okf::test_crate::body_structure::a_thematic_break_is_not_a_run_of_list_markers
    - raf_okf::test_crate::body_structure::a_three_space_heading_is_a_section_and_a_four_space_heading_is_not
    - raf_okf::test_crate::body_structure::code_spans_of_any_backtick_length_hide_their_content
    - raf_okf::test_crate::body_structure::fences_are_recognised_indented_up_to_three_spaces_and_inside_quotes_and_lists
    - raf_okf::test_crate::body_structure::headings_inside_fences_and_comments_are_not_headings
    - raf_okf::test_crate::body_structure::markdown_without_frontmatter_is_the_whole_body
    - raf_okf::test_crate::body_structure::prose_lines_skip_fences_and_comments_and_remove_code_spans
    - raf_okf::test_crate::body_structure::section_text_is_the_content_of_the_section
    - raf_okf::test_crate::body_structure::sections_carry_heading_slug_and_content_up_to_the_next_second_level_heading
    - raf_okf::test_crate::body_structure::spans_are_in_whole_text_coordinates_when_frontmatter_precedes_the_body
    - raf_okf::test_crate::bounds::a_long_heading_over_many_lines_stays_linear
    - raf_okf::test_crate::claims::a_capability_states_claims_in_its_sections
    - raf_okf::test_crate::claims::a_raf_generated_comment_keeps_its_own_rules
    - raf_okf::test_crate::claims::a_statement_stops_at_a_heading_a_blank_line_or_a_fence
    - raf_okf::test_crate::index::a_header_with_unknown_tags_or_in_the_wrong_place_is_refused
    - raf_okf::test_crate::index::an_empty_header_tag_is_refused
    - raf_okf::test_crate::index::an_index_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused
    - raf_okf::test_crate::index::an_index_with_no_block_or_an_unclosed_block_is_refused
    - raf_okf::test_crate::index::indented_and_mid_line_markers_are_refused
    - raf_okf::test_crate::links::a_link_split_across_lines_inside_a_block_quote_is_recorded
    - raf_okf::test_crate::links::a_space_between_the_text_and_the_parenthesis_is_not_a_link
    - raf_okf::test_crate::links::a_split_link_does_not_hide_a_link_after_it
    - raf_okf::test_crate::links::an_attribute_value_that_starts_on_the_next_line_is_read
    - raf_okf::test_crate::links::an_escaped_closing_parenthesis_belongs_to_the_destination
    - raf_okf::test_crate::links::an_html_tag_split_across_lines_yields_its_attributes_with_their_lines
    - raf_okf::test_crate::links::html_attributes_are_read_with_spacing_case_and_unquoted_values
    - raf_okf::test_crate::links::links_inside_code_spans_of_any_length_and_fences_are_not_links
    - raf_okf::test_crate::links::links_inside_html_comments_are_not_links
    - raf_okf::test_crate::links::nested_brackets_images_in_links_and_stray_brackets_are_read
    - raf_okf::test_crate::links::only_ascii_space_and_controls_end_a_destination_not_unicode_space
    - raf_okf::test_crate::links::reference_definitions_take_angle_targets_and_skip_footnotes
    - raf_okf::test_crate::links::split_links_are_not_recorded_in_fences_code_spans_or_for_one_line_links
    - raf_okf::test_crate::links::tag_state_does_not_survive_a_blank_line_or_a_fence
    - raf_okf::test_crate::links::targets_resolve_against_the_document_path
    - raf_okf::test_crate::links::the_tag_scan_spares_prose_autolinks_and_closed_tags
    - raf_okf::test_crate::markers::a_proposed_block_reports_slug_spans_content_and_section
    - raf_okf::test_crate::markers::document_parse_builds_its_body_through_the_body_model
    - raf_okf::test_crate::markers::generated_blocks_report_kind_outer_and_content_spans
    - raf_okf::test_crate::markers::marker_errors_in_a_document_carry_whole_text_lines
    - raf_okf::test_crate::markers::two_markers_in_a_row_make_two_blocks
    - raf_okf::test_crate::no_io_in_source::the_scanner_finds_every_spelling_of_a_forbidden_module
    - raf_okf::test_crate::no_io_in_source::the_scanner_leaves_neighbouring_paths_alone
    - raf_okf::test_crate::placeholder_lines::does_not_read_code_or_comments
    - raf_okf::test_crate::profile::a_capability_holds_the_seven_sections_in_order
    - raf_okf::test_crate::profile::a_change_needs_a_non_empty_summary
    - raf_okf::test_crate::profile::a_delivered_capability_with_code_and_tests_is_typed_with_its_bindings
    - raf_okf::test_crate::profile::a_document_must_sit_directly_in_the_directory_of_its_type
    - raf_okf::test_crate::profile::a_missing_type_key_is_the_only_violation
    - raf_okf::test_crate::profile::a_placeholder_token_in_a_code_span_of_a_frontmatter_value_is_spared
    - raf_okf::test_crate::profile::a_section_is_empty_when_only_comments_and_generated_blocks_remain
    - raf_okf::test_crate::profile::a_title_and_a_description_are_non_empty_strings
    - raf_okf::test_crate::profile::a_violation_displays_the_file_the_place_and_the_rule
    - raf_okf::test_crate::profile::an_architecture_body_is_not_empty
    - raf_okf::test_crate::profile::an_id_needs_the_prefix_of_its_type_a_kebab_slug_and_the_file_name
    - raf_okf::test_crate::profile::an_invalid_changes_entry_does_not_cascade_into_the_affects_entries
    - raf_okf::test_crate::profile::changes_is_a_non_empty_list_of_cap_ids
    - raf_okf::test_crate::profile::contract_effect_is_none_or_changes
    - raf_okf::test_crate::profile::depends_is_a_list_of_cap_ids_and_does_not_need_the_capability_to_exist
    - raf_okf::test_crate::profile::future_tense_prose_is_not_a_violation
    - raf_okf::test_crate::profile::placeholder_scan_spares_code_longer_words_and_dotted_prose
    - raf_okf::test_crate::profile::placeholder_tokens_in_frontmatter_values_are_reported_at_the_key
    - raf_okf::test_crate::profile::placeholder_tokens_in_prose_are_reported_at_their_line
    - raf_okf::test_crate::profile::tags_are_non_empty_unique_strings_and_a_timestamp_is_rfc_3339
    - raf_okf::test_crate::profile::the_three_outcomes_are_typed
    - raf_okf::test_crate::profile::valid_documents_of_each_type_validate_to_typed_values
    - raf_okf::test_crate::record_writer::a_value_the_frontmatter_cannot_hold_is_refused_by_the_writer
    - raf_okf::test_crate::record_writer::empty_lists_and_absent_keys_are_written_as_the_profile_reads_them
    - raf_okf::test_crate::record_writer::verdict_and_subject_names_are_the_values_of_their_keys
    - raf_okf::test_crate::records::a_passing_record_holds_no_value_above_its_limit
    - raf_okf::test_crate::records::a_record_type_is_named_in_the_unknown_type_message
    - raf_okf::test_crate::records::an_assessment_holds_the_section_row_of_its_subject
    - raf_okf::test_crate::records::an_input_kind_is_one_the_type_takes_and_its_name_fits_the_kind
    - raf_okf::test_crate::records::symbol_inputs_accept_crate_roots_and_refuse_malformed_paths
    - raf_okf::test_crate::records::evidence_from_a_bound_test_has_no_command_and_may_have_no_measures
    - raf_okf::test_crate::records::for_lists_at_least_one_capability_id
    - raf_okf::test_crate::records::input_rows_have_exactly_kind_name_and_digest
    - raf_okf::test_crate::records::scope_holds_crate_package_names
    - raf_okf::test_crate::records::the_command_key_is_present_exactly_when_the_subject_is_command
    - raf_okf::test_crate::records::the_subject_takes_the_form_of_its_type
    - raf_okf::test_crate::records::values_and_limits_are_numbers_and_each_limit_names_a_value
    - raf_okf::test_crate::refusals::a_colon_space_inside_a_flow_list_is_refused
    - raf_okf::test_crate::refusals::a_duplicate_key_is_refused_at_every_level
    - raf_okf::test_crate::refusals::a_flow_mapping_value_is_refused_naming_the_key
    - raf_okf::test_crate::refusals::a_hundred_thousand_levels_return_an_error_on_a_small_stack
    - raf_okf::test_crate::refusals::an_unclosed_fence_is_refused_at_the_opening_fence
    - raf_okf::test_crate::refusals::an_unterminated_quote_and_text_after_a_quote_are_refused
    - raf_okf::test_crate::refusals::anchors_aliases_tags_and_merge_keys_are_refused_naming_the_token
    - raf_okf::test_crate::refusals::block_indicators_inside_an_items_entry_are_refused
    - raf_okf::test_crate::refusals::block_scalars_are_refused_naming_the_header
    - raf_okf::test_crate::refusals::flow_list_shape_errors_are_refused
    - raf_okf::test_crate::refusals::items_keep_their_maps_and_scalars
    - raf_okf::test_crate::refusals::layout_errors_are_refused_with_their_position
    - raf_okf::test_crate::refusals::line_endings_must_agree_within_the_frontmatter
    - raf_okf::test_crate::refusals::numbers_booleans_and_null_are_read_as_strings
    - raf_okf::test_crate::refusals::plain_scalars_with_a_colon_space_or_an_indicator_are_refused
    - raf_okf::test_crate::refusals::stray_and_over_indented_lines_after_a_flow_value_are_refused
    - raf_okf::test_crate::refusals::text_that_plain_ok_accepts_is_read_back_plain_in_every_position
    - raf_okf::test_crate::refusals::text_that_the_reader_refuses_is_never_plain_ok
    - raf_okf::test_crate::refusals::trailing_comments_are_not_part_of_the_value
    - raf_okf::test_crate::refusals::unknown_escapes_are_refused_and_every_known_escape_is_read
    - raf_okf::test_crate::refusals::unterminated_quotes_are_refused_in_flow_list_item_and_nested_item_positions
    - raf_okf::test_crate::render::a_built_document_with_sorted_tags_equals_what_it_reads_back_as
    - raf_okf::test_crate::render::a_change_document_built_from_constructors_renders_and_reads_back_equal
    - raf_okf::test_crate::render::a_document_of_an_unknown_type_keeps_input_order
    - raf_okf::test_crate::render::a_parsed_empty_value_renders_as_an_empty_value
    - raf_okf::test_crate::render::constructed_nesting_and_size_are_bounded_like_the_reader
    - raf_okf::test_crate::render::constructors_choose_a_style_the_reader_reads_back
    - raf_okf::test_crate::render::constructors_refuse_what_the_reader_would_not_read_back
    - raf_okf::test_crate::render::crlf_documents_and_a_missing_final_newline_render_byte_identically
    - raf_okf::test_crate::render::equality_compares_keys_as_a_map_with_comments_and_body_bytes
    - raf_okf::test_crate::render::every_escape_round_trips_and_other_control_characters_are_refused
    - raf_okf::test_crate::render::flow_lists_are_written_in_one_spacing
    - raf_okf::test_crate::render::keys_outside_the_table_follow_in_input_order
    - raf_okf::test_crate::render::the_body_span_locates_the_body_in_the_text
    - raf_okf::test_crate::render::the_body_span_uses_the_coordinates_of_the_text_the_document_came_from
    - raf_okf::test_crate::render::unsorted_tags_render_sorted_and_the_document_equals_its_rendering_read_back
    - raf_okf::test_crate::schema::a_type_is_found_by_its_exact_name_only
    - raf_okf::test_crate::schema::malformed_and_out_of_range_timestamps_are_refused
    - raf_okf::test_crate::schema::each_type_states_its_required_and_optional_keys
    - raf_okf::test_crate::schema::valid_timestamps_are_accepted
    - raf_okf::test_crate::splice::a_document_delegates_and_renders_the_edit_with_its_frontmatter
    - raf_okf::test_crate::splice::a_document_edit_over_one_mebibyte_is_refused_and_the_document_is_unchanged
    - raf_okf::test_crate::splice::a_document_edit_with_a_malformed_result_is_refused_and_the_document_is_unchanged
    - raf_okf::test_crate::splice::a_document_whose_closing_fence_has_no_line_ending_gains_one_for_a_body
    - raf_okf::test_crate::splice::a_second_call_uses_spans_read_from_the_rebuilt_model
    - raf_okf::test_crate::splice::an_empty_edit_list_leaves_the_body_equal
    - raf_okf::test_crate::splice::edits_given_out_of_order_apply_in_one_pass
    - raf_okf::test_crate::splice::spans_outside_the_body_or_inside_a_character_are_refused
    - raf_okf::test_crate::splice::the_catalogue_of_an_index_is_replaced_through_the_index
    - raf_okf::work::tests::a_counter_increments_by_what_is_added
    - raf_okf::work::tests::a_limited_kind_panics_at_its_ceiling
    - raf_okf::work::tests::a_limited_measure_counts_the_other_kinds_without_a_ceiling
    - raf_okf::work::tests::a_measure_that_panicked_leaves_neither_counts_nor_ceiling_behind
    - raf_okf::work::tests::every_kind_has_its_own_counter
    - raf_okf::work::tests::passing_the_ceiling_panics_at_once
    - raf_okf::work::tests::reaching_the_ceiling_is_allowed
    - raf_okf::work::tests::the_panic_comes_before_the_work_that_follows_it
    - raf_okf::work::tests::the_result_of_the_closure_is_returned
    - raf_okf::work::tests::the_tallies_are_reset_between_measures
  examples:
    - raf_okf::Number::exact
    - raf_okf::EvidenceRecord::to_document
    - raf_okf::Claim
    - raf_okf::TestedBy
depends: []
---

# OKF profile reader and writer

## Purpose

`raf-okf` is the one reader and writer of RAF documents. Everything that touches a
capability, change or architecture page, the workplan gate and the repository
tooling alike, goes through it, so the profile is interpreted in a single place and
a document means the same thing to every tool.

## Contract

The profile has eight types. `Review` is an empty-body record with a required RFC 3339 timestamp, full reviewed revision, source scope, kind, verdict, reviewer category, derived inputs and findings. Its optional findings reference names a readable repository file or an opaque HTTP(S) URL. Finding rows retain severity, source location and open, fixed, accepted or false-positive disposition. Capability headers optionally declare distinct closed `review_required` kinds. The architecture profile states the complete closed key and value sets.
<!-- raf:tested-by raf_okf::test_crate::render::each_fixture_document_renders_to_its_own_bytes_and_validates -->

Capability and Architecture frontmatter may declare `diagrams`. Each closed row
names one automatic family and scope, with optional focus, grouping, relationship
selection and positive native limits. The typed `DiagramDeclaration` preserves
these fields; unknown keys, duplicate names and invalid values refuse. No other
document type accepts this key. The authoring dataset codec retains its exact
frontmatter representation through the same existing profile.
<!-- raf:tested-by raf_workplan::test_crate::sync::nothing_is_in_the_report_for_a_repository_that_is_in_sync -->


### Profile ownership
<!-- raf:tested-by raf_okf::test_crate::schema::each_type_lists_its_keys_in_the_order_of_the_architecture_page,raf_okf::test_crate::profile::every_required_key_of_every_type_is_reported_when_missing,raf_okf::test_crate::profile::an_unknown_or_foreign_key_is_reported_on_every_type -->

The facts of the profile, which are the document types, the closed key sets, the
value rules, the body rules and the syntax of the PROPOSED blocks, the generated
blocks, the claim markers and the tested-by markers, are stated
once, in [the RAF profile of OKF](../architecture/okf-profile.md). `raf-okf`
implements that page and adds no rule of its own. This document states only what the
crate does with it.

### Parse framing
<!-- raf:tested-by raf_okf::test_crate::refusals::a_document_without_a_fence_on_line_one_is_refused,raf_okf::test_crate::refusals::an_unclosed_fence_is_refused_at_the_opening_fence,raf_okf::test_crate::refusals::a_frontmatter_that_is_not_a_block_mapping_is_refused -->

`Document::parse` takes the text of one file and returns the frontmatter
and a body model. It fails with a positioned error when the file has no frontmatter
fence on line 1, when the YAML does not parse, or when the YAML is not a mapping.

### Comment attachment
<!-- raf:tested-by raf_okf::test_crate::render::comments_attach_to_the_next_key_and_trailing_comments_stay_on_their_line -->

SPDX lines are YAML comments inside the frontmatter and are preserved. Comments are
kept with the key they belong to: a full-line comment or blank line attaches to the
key that follows it, a trailing ` # comment` stays on its line, and comments after
the last key stay last.

### Frontmatter subset
<!-- raf:tested-by raf_okf::test_crate::refusals::numbers_booleans_and_null_are_read_as_strings,raf_okf::test_crate::refusals::anchors_aliases_tags_and_merge_keys_are_refused_naming_the_token,raf_okf::test_crate::refusals::a_duplicate_key_is_refused_at_every_level,raf_okf::test_crate::refusals::block_scalars_are_refused_naming_the_header -->

The frontmatter accepts the subset of YAML stated in
[the RAF profile of OKF](../architecture/okf-profile.md#frontmatter-subset) and
refuses the rest. Numbers are read as strings.

### Bounded parsing
<!-- raf:tested-by raf_okf::test_crate::refusals::a_hundred_thousand_levels_return_an_error_on_a_small_stack,raf_okf::test_crate::body_structure::a_hundred_thousand_quote_markers_are_refused_on_a_small_stack,raf_okf::test_crate::alloc_bound::refused_documents_stay_within_eight_times_their_size,raf_okf::test_crate::body_robustness::random_token_soup_never_panics_and_parsed_spans_lie_in_the_text -->

Documents are read from sources the reader may not control, so
`Document::parse` bounds its input and fails with a positioned error and never a
panic, abort or unbounded allocation.

### Whole-document byte limit
<!-- raf:tested-by raf_okf::test_crate::refusals::a_text_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused -->

The text is at most 1 MiB.

### YAML depth limit
<!-- raf:tested-by raf_okf::test_crate::refusals::nesting_over_thirty_two_levels_is_refused_before_descent -->

YAML nesting is at most 32 levels, and the check runs before any recursive
descent can overflow the stack.

### Alias expansion refusal
<!-- raf:tested-by raf_okf::test_crate::refusals::a_billion_laughs_document_is_refused_at_the_first_anchor,raf_okf::test_crate::refusals::anchors_aliases_tags_and_merge_keys_are_refused_naming_the_token -->

YAML anchors, aliases and merge keys are refused, because the closed key schema
has no use for them and an alias expansion can grow without bound.

### Markdown depth limit
<!-- raf:tested-by raf_okf::test_crate::body_structure::block_nesting_of_sixty_four_levels_parses_and_sixty_five_is_refused -->

Markdown block nesting (quotes and lists) is at most 64 levels.

### Link part byte limit
<!-- raf:tested-by raf_okf::test_crate::links::a_destination_or_title_of_exactly_4096_bytes_is_read_and_one_more_is_refused -->

A link destination or title is at most 4096 bytes, not counting the delimiters of a
title or of an angle-bracket destination. This bounds the link scanner, which
would otherwise rescan a long paragraph for every `](`. A longer part is a
positioned error, never a link that is silently not read.

### Document types
<!-- raf:tested-by raf_okf::test_crate::schema::each_type_states_its_required_and_optional_keys,raf_okf::test_crate::schema::singleton_and_record_roles_follow_the_document_types -->

The profile has eight types: the documents `Capability`, `Change`
and `Architecture`, which people write, and the records `Evidence`, `Assessment`,
`Coverage`, `Debt` and `Review`, which state a result in their frontmatter. `DocType::ALL`
lists them.

### Closed coverage policy
<!-- raf:tested-by raf_okf::test_crate::records::a_coverage_record_rejects_the_removed_policy_key,raf_okf::test_crate::record_writer::a_coverage_record_writes_only_its_closed_policy_independent_keys -->

The coverage profile has no policy selector. The
removed `strictness` key is an unknown key, including its former values, and the
typed coverage record and writer expose only the current closed key set.

### Ceiling values and classes
<!-- raf:tested-by raf_okf::test_crate::records::a_ceiling_names_a_crate_one_whole_number_of_lines_or_a_percent_and_an_optional_class,raf_okf::test_crate::records::a_ceiling_of_another_shape_is_refused_with_its_rule -->

The coverage record takes, after `exclusions`, the
optional keys `ceilings` and `exceptions`, which the reverse coverage of
[complete policy](./workplan-strict.md) reads; [the RAF profile of OKF](../architecture/okf-profile.md)
states their value rules. A row of `ceilings` is a `Ceiling`: its `crate`, a crate
package name; its `CeilingLimit`, exactly one of `lines`, a whole number
(`CeilingLimit::Lines`), and `percent`, a number from 0 to 100
(`CeilingLimit::Percent`), each held as a `Number` and compared on its decimal
text; and its `CeilingClass`, the optional `of`, `shipping`, `test` or `all`
(`CeilingClass::ALL`, `CeilingClass::name`, `CeilingClass::from_name`), and `All`
without the key. A crate has at most one row for each class
(`okf-duplicate-entry`); a row with neither or both of `lines` and `percent`, an
`of` of another value, a `lines` that is not whole or a `percent` outside 0 to 100
is `Rule::BadCeiling` at `ceilings`, reported as `okf-bad-ceiling`, and a value
that is not a number is `okf-bad-measure`.

### Exception targets and reasons
<!-- raf:tested-by raf_okf::test_crate::records::an_exception_names_one_item_prefix_file_or_directory_and_its_reason,raf_okf::test_crate::records::an_exception_of_another_shape_or_without_a_reason_is_refused_with_its_rule -->

A row of `exceptions` is an
`Exception`: an `ExceptionTarget` and a non-empty `reason`. The target is exactly
one of `item`, an item path (`ExceptionTarget::Item`) or a `<module>::*` prefix
(`ExceptionTarget::Prefix`) below a crate root, and `file`, a repository-relative
path (`ExceptionTarget::File`) or a directory with a trailing `/`
(`ExceptionTarget::Directory`), as `ExceptionTarget::written` writes it again. A row
with neither or both of `item` and `file`, an `item` that is a crate root or
neither an item path nor a prefix, and a `file` that is absolute, holds a `..`, `.`
or empty segment or a backslash is `Rule::BadException` at `exceptions`, reported as
`okf-bad-exception`; an empty reason is `okf-empty-reason`, a reason with a
placeholder token is refused as an exclusion's is, and a target written twice is
`okf-duplicate-entry`. Whether an exception matches anything is judged by the
workplan, which needs the source index.

### Coverage extension writer
<!-- raf:tested-by raf_okf::test_crate::record_writer::ceilings_and_exceptions_are_written_after_the_exclusions_and_read_back_equal -->

`CoverageRecord::ceilings` and
`CoverageRecord::exceptions` hold them, empty without the keys;
`CoverageRecord::with_ceilings` and `CoverageRecord::with_exceptions` set them,
and `CoverageRecord::to_document` writes each list after the exclusions in record
order, a ceiling's `of` only when it is not `all`, and leaves out a key that holds
none, as a record without it reads.

### Typed debt rows
<!-- raf:tested-by raf_okf::test_crate::records::a_valid_debt_record_validates_to_its_typed_value_and_an_empty_one_opts_out_visibly,raf_okf::test_crate::records::a_row_of_findings_holds_exactly_a_kind_and_a_subject_each_pair_once,raf_okf::test_crate::records::a_row_of_lines_holds_exactly_an_item_and_a_whole_number_of_lines_each_item_once,raf_okf::test_crate::records::both_lists_of_a_debt_record_are_required,raf_okf::test_crate::record_writer::a_debt_record_is_written_sorted_and_read_back_equal -->

The record `Debt` is the strict debt of
[complete policy](./workplan-strict.md), in `work/debt` with the prefix `debt:` and the one
id `debt:default`, as the coverage record has `coverage:default`.
[The RAF profile of OKF](../architecture/okf-profile.md) states its keys, and `raf-okf`
validates them: the debt record has the keys `type`, `id`, the optional `tags` and
`timestamp`, `findings`, rows of exactly `kind` and `subject`, and `lines`, rows of
exactly `item` and `orphans`, each pair or item once, and an empty body; an `orphans`
that is not a whole number is `okf-bad-measure`. It is validated into
`RafDocument::Debt` holding a `DebtRecord` of `DebtFinding` and `DebtLine` rows, which
`DebtRecord::new` builds, sorted, and `DebtRecord::to_document` writes as the other
records are written, with the rows in that order. Which kinds a row may hold and what
form a subject takes are rules of the complete policy and not of the profile.

### Symbol input names
<!-- raf:tested-by raf_okf::test_crate::records::symbol_inputs_accept_crate_roots_and_refuse_malformed_paths -->

A `symbol` input names a crate-root identifier or a path of identifiers joined by
`::`. Empty segments, a lone underscore, leading digits and hyphens are refused.

### Positioned validation results
<!-- raf:tested-by raf_okf::test_crate::profile::a_document_with_three_faults_returns_three_violations -->

`Profile::validate` checks a parsed document and its path against the
closed key set of its `type` and returns a typed result: the typed document
(`RafDocument`, with `State`, `Bindings`, `Affects` entries, `ChangeItem` entries and
their `Outcome`), a record as `RafDocument::Evidence`, `RafDocument::Assessment`,
`RafDocument::Coverage` or `RafDocument::Debt`, holding an `EvidenceRecord`, an
`AssessmentRecord`, a `CoverageRecord` or a `DebtRecord` with a `RecordHeader`, since a record has no `title` and no
`description`, or the list of every violation, not only the first. Each violation
names the file, the rule and, for a frontmatter rule, the key; a body rule names the
section or the line instead. The rules are:

- the key set comes from one table per type, in the order of the architecture page. A
  key outside the table, a missing required key, and the base key `resource` (a rule of
  its own, which names the key set of the type) are violations. The optional keys
  `tags` and `timestamp` may be absent;
- a missing `type`, or one outside `DocType::ALL`, is the only violation reported for
  that document, because no key set applies;
- a document of a known type sits directly in the directory of its type;
- the `id` carries the prefix of its type and a kebab-case slug equal to the file name
  without `.md`; `title` and `description` are non-empty strings;
- `state` is an enumeration, and `bindings` is a map with exactly `code`, `tests` and
  `examples`, each a list. A `designed` or `considered` capability binds nothing, and a
  `delivered` one binds at least one `code` and one `tests` entry;
- `depends` and `changes` hold capability ids (`changes` at least one), `affects`
  entries are parsed into a capability id and one of the seven sections, and name a
  capability listed in `changes`; `contract_effect` is `none` or `changes`; `items`
  entries are parsed into a non-empty claim, an optional outcome
  (`implemented`, `dropped` or `moved:change:<slug>`) and an optional reason, a
  non-empty line of text in `ChangeItem::reason` that is accepted beside any outcome
  or none. A reason that is empty, holds only spaces or is not text is
  `Rule::ItemReasonEmpty`, which the workplan reports as `okf-item-reason-empty`,
  and the fix of an unknown item key names `claim`, `outcome` and `reason`.
  [Complete policy](./workplan-strict.md) asks for a reason on a `dropped` or `moved`
  item. `Outcome` and `LinkTarget`
  are exhaustive enums, because the profile closes both sets: a consumer matches
  every variant with no fallback arm, and a new variant is a breaking change;
- `tags` is a list of non-empty strings with no duplicate, and `timestamp` is an RFC
  3339 date and time in the closed authored subset below;
- a Capability body holds the seven sections in order, none empty; a Change body holds
  a non-empty `Summary`; an Architecture body is not empty; a record body is empty. A
  section is empty when nothing is left after its HTML comments and generated blocks
  are removed;
- a PROPOSED block appears only in a Capability and only inside a section;
- a claim marker appears only in a Capability: one in any other document is a
  violation, and so is a tested-by marker, `Rule::TestedByOutsideCapability`, which
  the workplan reports as `okf-tested-by-outside-capability`;
- no prose line, title, description, item claim or item reason holds a placeholder
  token, and no link is split across lines. `Body::placeholder_lines` returns the
  placeholder tokens of any body, with their lines, by the same rule, so a caller
  that reads Markdown without frontmatter applies the rule `Profile::validate`
  applies; fenced code, code spans and HTML comments are not read.

The records follow the key table and value rules of the architecture page:

- evidence and an assessment share the trace keys `subject`, `inputs`, `verdict`
  and `by`. The subject is parsed into an `EvidenceSubject` (`command` or
  `binding:<path>`) or a `SectionRef`, each input row into an `Input` with its
  `InputKind` and a digest of 64 lowercase hexadecimal digits, the verdict into
  an `EvidenceVerdict` or an `AssessmentVerdict`, and `by` into an `Actor`, which
  is `human` or `agent` and never the name of a person;
- evidence names the capabilities it covers in `for`, takes `symbol` inputs
  only, and states `command` exactly when its subject is `command`; `revision`
  is a full commit id; `values` and `limits` hold `Number`s, each limit of
  evidence that passes names a value, and evidence that passes holds no value
  above its limit, compared exactly on retained native decimal values
  (`Number::compare`). `Number::exact` borrows that value without reparsing;
  textual equality retains the authored spelling, while numeric comparison
  ignores padding and the sign of zero. `Number::value` is a display conversion
  to the nearest float and retains the authored sign of zero;
- an assessment holds the `section` input of its subject;
- the coverage record is `coverage:default` at `work/coverage/default.md`. Its
  `scope` holds
  distinct crate package names, and each exclusion an `ExclusionItem` below a crate
  root with a non-empty reason; a crate root is refused. Its `ceilings` and
  `exceptions`, when they are there, are the rows above;
- `DocType`, `InputKind`, `Actor`, `EvidenceVerdict`, `AssessmentVerdict`,
  `EvidenceSubject`, `ExclusionItem`, `CeilingClass`, `CeilingLimit`
  and `ExceptionTarget` are exhaustive enums, because the profile closes each set.

Checks that need other documents, such as whether an id exists or whether a graph is
acyclic, belong to [the document model](./workplan-document-model.md); `raf-okf`
validates one document at a time. Prose that talks about future work is not a
violation: the crate takes no position on tense.

### Timestamp lexical subset
<!-- raf:tested-by raf_okf::test_crate::schema::valid_timestamps_are_accepted,raf_okf::test_crate::schema::malformed_and_out_of_range_timestamps_are_refused -->

`validate_timestamp` delegates RFC parsing and Gregorian validation to PurRDF.
The authored profile requires a four-digit year, uppercase `T` and `Z`, or a
numeric offset, and seconds from 00 through 59. Fractions retain one or more
digits; an absent zone, lowercase separators and leap seconds are refused.
Malformed syntax and out-of-range fields remain distinct typed errors, with
the out-of-range month, day, clock field or offset field named.

### Native exact numeric values
<!-- raf:tested-by raf_okf::test_crate::records::numbers_compare_exactly_by_sign_whole_part_and_fraction,raf_okf::Number::exact -->

`Number` retains its validated native PurRDF Decimal once. `Number::exact`
borrows that value for exact arithmetic without reparsing the authored text or
allocating a second value. `Number::compare` uses native value order, so padding
and the sign of zero do not alter the value. Number equality and `Number::text`
retain the authored spelling; `Number::value` is a nearest floating-point display
value that preserves an authored negative zero. The profile grammar remains
narrower than the native decimal value space.

### Body model
<!-- raf:tested-by raf_okf::test_crate::body_structure::spans_are_in_whole_text_coordinates_when_frontmatter_precedes_the_body,raf_okf::test_crate::body_structure::markdown_without_frontmatter_is_the_whole_body,raf_okf::test_crate::body_structure::sections_carry_heading_slug_and_content_up_to_the_next_second_level_heading,raf_okf::test_crate::body_structure::a_section_that_ends_the_body_without_a_final_newline_reaches_the_end -->

The body is parsed into a structure that tools query instead of
re-scanning text. `Document::parse` builds the body of a document through
`Body::parse`, which also reads Markdown that has no frontmatter, such as a README: a
leading `---` fence closed by a later `---` line is skipped, and otherwise the whole
text is the body. Every span the model returns carries both line numbers and byte
offsets, in the coordinates of the whole text. Line ranges are 1-based and half-open,
as are byte ranges, so a span on line `n` alone has the lines `n..n+1`. The model
holds the following independently addressable structures.

### Heading anchors
<!-- raf:tested-by raf_okf::test_crate::body_structure::headings_of_every_level_carry_github_anchors_with_repeats_numbered,raf_okf::test_crate::body_structure::headings_inside_fences_and_comments_are_not_headings -->

Headings of levels 1 to 6 outside fenced code, each with its GitHub anchor (repeats
numbered `-1`, `-2`), and a check whether an anchor exists.

### Section spans
<!-- raf:tested-by raf_okf::test_crate::body_structure::sections_carry_heading_slug_and_content_up_to_the_next_second_level_heading,raf_okf::test_crate::body_structure::a_section_that_ends_the_body_without_a_final_newline_reaches_the_end -->

Second-level sections, each with its heading, its slug and a span that starts on the
line after the heading and ends before the next second-level heading.

### Link forms and containment
<!-- raf:tested-by raf_okf::test_crate::links::every_link_form_is_read_with_its_target_anchor_and_line,raf_okf::test_crate::links::links_inside_code_spans_of_any_length_and_fences_are_not_links,raf_okf::test_crate::links::links_inside_html_comments_are_not_links,raf_okf::test_crate::links::targets_resolve_against_the_document_path,raf_okf::test_crate::links::targets_that_leave_the_root_or_are_empty_or_absolute_are_refused,raf_okf::test_crate::links::a_link_split_across_lines_inside_a_block_quote_is_recorded -->

Links, each with its target, its anchor, its line and its form: inline link, image,
reference definition, `href` attribute or `src` attribute. Links inside fenced
code, code spans of any backtick length and HTML comments are not links. A target
resolves against the path of the linking document to an external target, a
fragment, or a path inside the repository, and a target that climbs above the root
or is an absolute path is an error, never clamped. A link inside a block quote reads
the same on every quoted line.

### Split links
<!-- raf:tested-by raf_okf::test_crate::links::a_link_split_across_lines_is_recorded_and_not_read,raf_okf::test_crate::links::split_links_are_not_recorded_in_fences_code_spans_or_for_one_line_links -->

Split links: an inline link or image whose text or parentheses span a line break is
not read as a link and is recorded with its line, so that a split link is a
violation.

### Fenced code blocks
<!-- raf:tested-by raf_okf::test_crate::body_structure::a_fence_closes_only_on_the_same_character_with_at_least_the_same_length,raf_okf::test_crate::body_structure::fences_are_recognised_indented_up_to_three_spaces_and_inside_quotes_and_lists,raf_okf::test_crate::body_structure::a_fenced_block_exposes_its_content_and_whether_it_is_closed -->

Fenced code blocks, matched by character and length and recognised inside block
quotes and lists, so markers inside a fence are never read as markers. Each block
has its span, its info string, its content (`FencedBlock::content`, the text between
the fence lines as written) and whether a closing fence ends it
(`FencedBlock::is_closed`); a block that is never closed runs to the end of the body.

### Generated block spans
<!-- raf:tested-by raf_okf::test_crate::markers::generated_blocks_report_kind_outer_and_content_spans -->

Generated blocks, each with its kind, its outer span (markers included) and its
content span (markers excluded).

### Proposal block spans
<!-- raf:tested-by raf_okf::test_crate::markers::a_proposed_block_reports_slug_spans_content_and_section,raf_okf::test_crate::markers::two_markers_in_a_row_make_two_blocks -->

PROPOSED blocks, each with the change slug it names, its span, its marker line, its
unquoted content and the section that holds it.

### Prose view
<!-- raf:tested-by raf_okf::test_crate::body_structure::prose_lines_skip_fences_and_comments_and_remove_code_spans,raf_okf::test_crate::body_structure::code_spans_of_any_backtick_length_hide_their_content -->

Prose lines: the lines outside fenced code and HTML comments with code spans
removed, each with its section and whether it lies in a PROPOSED or generated block.

### Evidence claim markers
<!-- raf:tested-by raf_okf::test_crate::claims::a_claim_marker_names_its_evidence_and_supports_the_paragraph_above_it,raf_okf::test_crate::claims::a_claim_after_a_list_item_supports_that_item_only,raf_okf::test_crate::claims::markers_in_fences_code_spans_and_proposed_blocks_are_not_read -->

Claims: `Body::claims` returns each claim marker outside fenced code, code spans
and PROPOSED blocks as a `Claim`: the evidence slugs it names, its line, its marker
span, the span of the paragraph or list item it supports, and its section.

### Test ownership markers
<!-- raf:tested-by raf_okf::test_crate::tested_by::a_tested_by_marker_names_its_tests_and_maps_the_paragraph_above_it,raf_okf::test_crate::tested_by::a_tested_by_marker_maps_a_list_item_a_heading_or_the_statement_of_its_claim_marker,raf_okf::test_crate::tested_by::tested_by_markers_in_fences_code_spans_and_proposed_blocks_are_not_read -->

Tested-by markers: `Body::tested_by` returns each marker
`<!-- raf:tested-by <path>[,<path>...] -->` outside fenced code, code spans and
PROPOSED blocks as a `TestedBy`: the paths it names, in marker order, its line, its
marker span, the span of the statement it maps and its section. It is a third
marker family beside the generated and the claim markers. Its statement is the
statement of the claim marker directly above it, the heading line directly above
it, or the run of lines directly above it read as for a claim marker.

### Marker refusals
<!-- raf:tested-by raf_okf::test_crate::markers::every_misspelling_of_the_proposed_marker_is_refused_with_its_line,raf_okf::test_crate::markers::each_generated_marker_error_names_its_line,raf_okf::test_crate::claims::a_malformed_claim_marker_is_refused_with_its_line,raf_okf::test_crate::claims::a_claim_marker_with_no_statement_directly_above_is_refused,raf_okf::test_crate::tested_by::a_malformed_tested_by_marker_is_refused_with_its_line,raf_okf::test_crate::tested_by::a_tested_by_marker_with_no_statement_directly_above_is_refused,raf_okf::test_crate::claims::every_other_raf_comment_is_refused_with_its_line,raf_okf::test_crate::claims::a_raf_comment_split_over_lines_is_a_marker_error_and_other_split_comments_are_text -->

A malformed PROPOSED marker in any spelling, an unclosed generated block, a nested
generated block and an end marker without a start are parse errors with line numbers.
So are block quotes and list items nested over 64 levels and, outside fenced code,
code spans and PROPOSED blocks, a malformed claim marker, a claim marker with no
statement directly above it, a malformed tested-by marker
(`ErrorKind::MalformedTestedBy`: a space after a comma, a repeated or an empty entry,
an entry that is not an `<item-path>`, or an attribute such as `tests=`), a tested-by
marker with no statement directly above it, after a blank line or another comment, or
directly before a claim marker, whose place is between the statement and the tested-by
marker (`ErrorKind::UnattachedTestedBy`), and a `raf:` comment that is neither a
`raf:generated:` comment, a claim marker nor a tested-by marker, so a misspelt marker
never reads as a plain comment. The workplan reports the two tested-by errors as
`okf-parse-malformed-tested-by` and `okf-parse-unattached-tested-by`, and the malformed
and unattached forms of the claim marker stay as they are.
A marker is one line: an HTML comment that `<!--` opens at the end of a line, outside
its code spans, and whose first non-blank text on the lines below is `raf:` in any
letter case and not `raf:generated:`, is the same error at the line of `<!--`,
`okf-parse-malformed-claim` for `raf:claim`, `okf-parse-malformed-tested-by` for
`raf:tested-by` and `okf-parse-unknown-marker` for another word, so a marker split
over lines never reads as a plain comment
either. A multi-line `raf:generated:` comment, an SPDX header written over several
lines and every other multi-line comment are plain comments.

### Atomic body edits
<!-- raf:tested-by raf_okf::test_crate::splice::two_generated_blocks_change_only_their_byte_ranges,raf_okf::test_crate::splice::edits_given_out_of_order_apply_in_one_pass,raf_okf::test_crate::splice::overlapping_spans_are_refused_and_the_body_is_unchanged,raf_okf::test_crate::splice::a_malformed_result_is_refused_and_the_body_is_unchanged,raf_okf::test_crate::splice::spans_outside_the_body_or_inside_a_character_are_refused,raf_okf::test_crate::splice::a_document_delegates_and_renders_the_edit_with_its_frontmatter,raf_okf::test_crate::splice::the_catalogue_of_an_index_is_replaced_through_the_index -->

`Body::replace_spans`
applies non-overlapping byte-range edits in one pass and reads the model again; it
is all or nothing, and it fails with a typed error for a range outside the body, a
range inside a character, two ranges that overlap, or a result that no longer
parses. `Document` and `Index` delegate to it.

### Deterministic document rendering
<!-- raf:tested-by raf_okf::test_crate::render::keys_out_of_order_render_in_table_order_and_then_stay_put,raf_okf::test_crate::render::keys_outside_the_table_follow_in_input_order,raf_okf::test_crate::render::a_document_of_an_unknown_type_keeps_input_order,raf_okf::test_crate::render::tags_render_sorted_and_stay_sorted,raf_okf::test_crate::render::a_parsed_document_equals_its_rendering_read_back,raf_okf::test_crate::render::equality_compares_keys_as_a_map_with_comments_and_body_bytes -->

`Document::render` writes a document back. The writer is deterministic
and round-trips: parsing the output yields an equal document, and rendering a
document that was produced by `render` returns identical bytes. Key order follows
the order of the profile table: the keys of the document's type come first in table
order, then any other keys in input order, and a document of an unknown type keeps
input order. The top-level `tags` list is put in sorted order when a document is
parsed or built, so it is written sorted and a document equals the reading of its own
rendering; every other list keeps author order. Comments travel with the key they are attached to, and the body is written
unchanged apart from the replaced spans that a caller supplies, such as a generated
block. Two documents are equal when their keys, compared as a map, have equal values
and equal attached comments, and their bodies hold the same bytes.

### Native authoring dataset
<!-- raf:tested-by raf_okf::test_crate::render::each_fixture_document_renders_to_its_own_bytes_and_validates,raf_okf::test_crate::render::every_tracked_document_renders_to_its_own_bytes -->

`Document::to_dataset(path)` validates one of the eight RAF types and returns a
native PurRDF dataset. `Document::from_dataset(dataset, path)` recovers its model
through the existing constructors, Markdown writer, parser and profile validator.
The model must remain equal after rendering and parsing. The body bytes, scalar
text and quoting style, list form, item order and multiplicity, comments and the
first mapping entry used by a list item survive. Mapping key order and source
coordinates remain outside equality.

The asserted document IRI is `DOCUMENT_IRI_BASE`, followed by the lowercase type,
`/` and the validated id encoded by the native IRI path encoder. Content edits
preserve that identity. Descendants use the native BLAKE3 hash of the document IRI
and bounded numeric suffixes. The closed vocabulary under
`https://blackcatinformatics.ca/raf/okf/1#` distinguishes Document, Map, Entry,
Scalar, List, Item and Comment nodes. Owning edges and contiguous ordinals retain
each position; a mapping item names its first Entry separately. Authoring values
are native string literals, with canonical integer ordinals. Native RDF 1.2
Turtle and N-Triples readers and canonical writers carry the dataset directly.

The decoder refuses unknown predicates and roles, wrong cardinalities, dangling
references, cycles, shared children, gaps, unrelated nodes, identity disagreement,
named graphs, blank nodes and statement layers. Both directions admit at most
131072 structural nodes, 1048576 asserted triples and 1 MiB of aggregate authoring
text, checked before cloning values. Native failures retain their error source.
The generated navigation `Index` is a separate model and is refused explicitly.
The snapshot report profile remains unchanged. Repository semantic relations,
whole-repository claim relations, native authoring-shape validation and review
records remain distinct from this library authoring carrier.

The checked `document_iri` constructor on `DocType` exposes this same identity
without constructing a document. It shares prefix, kebab-slug and singleton
checks with profile validation, refuses ids over 1 MiB before encoding, and
requires `default` for Coverage and Debt. The profile additionally checks the
filename. Navigation Index is not one of the eight document types. The authoring
codec and semantic claim query use this one constructor.
Its shared prefix, slug and singleton rule helper is `id_slug`.
<!-- raf:tested-by raf_okf::test_crate::profile::an_id_needs_the_prefix_of_its_type_a_kebab_slug_and_the_file_name,raf_okf::test_crate::render::each_fixture_document_renders_to_its_own_bytes_and_validates -->

### Typed record construction and writing
<!-- raf:tested-by raf_okf::test_crate::record_writer::a_built_evidence_record_renders_to_a_record_that_reads_back_equal,raf_okf::test_crate::record_writer::a_built_assessment_and_coverage_record_read_back_equal,raf_okf::test_crate::record_writer::the_keys_are_written_in_the_order_of_the_key_table,raf_okf::test_crate::record_writer::a_value_the_profile_refuses_is_written_and_refused_when_it_is_read_back,raf_okf::test_crate::record_writer::a_value_the_frontmatter_cannot_hold_is_refused_by_the_writer,raf_okf::test_crate::record_writer::verdict_and_subject_names_are_the_values_of_their_keys -->

`EvidenceRecord::new`, `AssessmentRecord::new` and
`CoverageRecord::new` build a record with its header (`RecordHeader::new`), from
`Input::new`, `SectionRef::new`, `Measure::new` and `Exclusion::new`; the
constructors judge nothing. `EvidenceRecord::to_document`,
`AssessmentRecord::to_document` and `CoverageRecord::to_document` write a record as
a document with its keys in key-table order, its lists and measures in record
order, numbers unquoted as written, an empty list as `[]`, absent optional keys left
out, and an empty body. `Profile::validate` reads the rendering back as an equal
record when the values satisfy the profile, and refuses it with the violations
otherwise; a value the frontmatter cannot hold is a `ConstructError`.
`EvidenceVerdict::name`, `AssessmentVerdict::name` and `EvidenceSubject::text` give
the values of the keys.

### Generated-only index
<!-- raf:tested-by raf_okf::test_crate::index::a_generated_only_index_is_accepted_with_either_header_form,raf_okf::test_crate::index::prose_outside_the_blocks_is_refused_with_its_line,raf_okf::test_crate::index::only_the_capability_index_path_is_the_index,raf_okf::test_crate::index::document_parse_of_index_text_fails_at_line_one,raf_okf::test_crate::index::indented_and_mid_line_markers_are_refused,raf_okf::test_crate::index::a_header_with_unknown_tags_or_in_the_wrong_place_is_refused,raf_okf::test_crate::index::an_empty_header_tag_is_refused,raf_okf::test_crate::index::an_index_with_no_block_or_an_unclosed_block_is_refused,raf_okf::test_crate::splice::the_catalogue_of_an_index_is_replaced_through_the_index -->

`docs/capabilities/index.md` is the one navigation file in the
capability directory. `is_capability_index` is true for that path and false for any
other, including `docs/index.md`. `Index::parse` accepts a text only when apart from an
SPDX header comment (one line, or several between `<!--` and `-->` lines, each tag with
a value) every non-blank line lies inside a generated block and each marker starts its
own line with no indentation. Prose, an indented or mid-line marker, an empty header
tag, a header after a block, an unclosed block and a text with no block are errors with
a line. The text of an index never parses as a Capability document: `Document::parse`
fails on it at line 1. `Index::render` returns the text, and `Index::replace_spans`
replaces a block and refuses an edit that leaves anything but an index.

Scalar escape values, numeric quoting, calendar validation, exact decimal
comparison and canonical hexadecimal decoding use PurRDF primitives. The
document boundary retains the closed profile: six permitted escapes, the
authored decimal grammar, uppercase timestamp separators and no leap seconds.
Timestamp diagnostics first check date, clock and fraction syntax, then
calendar and clock ranges, then the complete offset syntax and its ranges.
Thus an incomplete date is malformed even when its month is invalid, while
an invalid month in a complete date and clock precedes a missing timezone.

## Usage

Validate a Review through `Document::validate` and write it through the shared native record writer. Record source inputs through `raf_workplan::review::record_review` so that they describe the actual reviewed tree. An omitted or empty required-kind list adds no review requirement.
<!-- raf:tested-by raf_okf::test_crate::render::each_fixture_document_renders_to_its_own_bytes_and_validates -->

Declare a row with `name: structure`, `kind: ownership` and `scope: workspace`
in the optional `diagrams` list. Workplan resolves the declaration and its matching
`diagram-structure` block; the profile supplies the closed typed declaration.
<!-- raf:tested-by raf_workplan::test_crate::sync::nothing_is_in_the_report_for_a_repository_that_is_in_sync -->


Use `DocType::Evidence.document_iri("evidence:memory-bound")` to name the same
evidence document from a semantic graph without building an authoring document.

The runnable API example `raf_okf::EvidenceRecord::to_document` constructs the document model before canonical Markdown serialization.

```rust
use raf_okf::{Document, Profile, RafDocument};

const TEXT: &str = "\
---
type: Architecture
id: arch:example
title: \"Example\"
description: \"One line.\"
---

# Example

What this page says.
";

let document = Document::parse(TEXT).expect("the text is a document");
assert_eq!(document.render(), TEXT);
let dataset = document.to_dataset("docs/architecture/example.md").expect("native RDF");
let recovered = Document::from_dataset(&dataset, "docs/architecture/example.md")
    .expect("closed authoring shape");
assert_eq!(recovered, document);
match Profile.validate(&document, "docs/architecture/example.md") {
    Ok(RafDocument::Architecture(page)) => assert_eq!(page.header.title, "Example"),
    Ok(other) => panic!("not an architecture page: {other:?}"),
    Err(violations) => panic!("{violations:?}"),
}
```

The text comes from the caller, so the block needs no file access. A caller that rewrites
a span of the body reads the span from the body model, for example
`document.body().generated_blocks()[0].content()`, then calls
`document.replace_spans(&[(span.clone(), new_text)])` and `document.render()`.

## Examples

The eighth authoring fixture contains a Review with all four finding dispositions and both source-location forms. The existing native round-trip case preserves its complete authoring model.
<!-- raf:tested-by raf_okf::test_crate::render::each_fixture_document_renders_to_its_own_bytes_and_validates -->

A valid architecture ownership declaration loads. The same row with an unknown
key, a zero term limit or a repeated scope key refuses beside that valid input.
<!-- raf:tested-by raf_workplan::test_crate::sync::nothing_is_in_the_report_for_a_repository_that_is_in_sync -->


Coverage and Debt identities use `coverage:default` and `debt:default`;
`coverage:other` is refused by the shared singleton rule.

The runnable `Document` usage above projects a validated architecture page into a
native dataset and recovers an equal authoring model. The eight document fixtures
also pass through native Turtle and N-Triples, including comments and quoted values.
<!-- raf:tested-by raf_okf::test_crate::render::each_fixture_document_renders_to_its_own_bytes_and_validates -->

A frontmatter block the profile accepts, for a capability with nothing built:

```yaml
type: Capability
id: cap:example
title: "Example"
description: "One line."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: []
```

Between `---` fences, with the seven sections as its body, at the path
`docs/capabilities/example.md`, the block validates. The same block with
`resource: https://example.org/x` added is rejected with a violation at the key
`resource`, whose message names the `resource` key and the closed key set of
`Capability`.

## Constraints

Checked identity construction validates the complete prefixed id, not a file or
document body. Profile validation still enforces the filename and all content.
<!-- raf:tested-by raf_okf::test_crate::profile::an_id_needs_the_prefix_of_its_type_a_kebab_slug_and_the_file_name -->

The native authoring dataset is one default graph with a single stable document
root. It admits at most 131072 structural nodes, 1048576 triples and 1 MiB of
aggregate authoring text. Resource preflight precedes authoring value cloning.
The decoder checks the closed shape and ownership during bounded reconstruction,
then refuses any representation that fails the existing RAF parser or profile
before returning a document. Navigation `Index` is not a `Document` carrier.
<!-- raf:tested-by raf_okf::test_crate::render::each_fixture_document_renders_to_its_own_bytes_and_validates -->

- The crate handles text and caller-provided native datasets. It touches no file
  and runs no process.
  <!-- raf:tested-by raf_okf::test_crate::render::a_change_document_built_from_constructors_renders_and_reads_back_equal -->
- Input is bounded to 1 MiB, 32 YAML levels, 64 Markdown block levels and 4096 bytes
  per link destination or title, and YAML anchors and aliases are refused. A bound
  violation is a parse error.
  <!-- raf:tested-by raf_okf::test_crate::refusals::a_text_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused,raf_okf::test_crate::refusals::nesting_over_thirty_two_levels_is_refused_before_descent,raf_okf::test_crate::body_structure::block_nesting_of_sixty_four_levels_parses_and_sixty_five_is_refused,raf_okf::test_crate::links::a_destination_or_title_of_exactly_4096_bytes_is_read_and_one_more_is_refused -->
- Only ATX headings (`#` marks) and fenced code blocks are modelled. A setext heading
  is not a heading, and indented code is read as ordinary text, so a marker or a link
  in it counts; write code that holds a marker or a link in a fence.
  <!-- raf:tested-by raf_okf::test_crate::body_structure::only_atx_headings_are_headings_and_setext_headings_are_not,raf_okf::test_crate::body_structure::indented_code_is_ordinary_text_so_links_and_markers_in_it_count -->
- Keys are exactly those of the profile. There is no pass-through for extra keys.
  <!-- raf:tested-by raf_okf::test_crate::profile::an_unknown_or_foreign_key_is_reported_on_every_type,raf_okf::test_crate::profile::a_nested_unknown_key_is_reported_in_bindings_and_in_a_change_item -->
- A document is valid or invalid. There is no warning level.
  <!-- raf:tested-by raf_okf::test_crate::profile::a_document_with_three_faults_returns_three_violations,raf_okf::test_crate::profile::valid_documents_of_each_type_validate_to_typed_values -->
- The writer preserves comments in the frontmatter but does not reflow the body.
  <!-- raf:tested-by raf_okf::test_crate::render::comments_attach_to_the_next_key_and_trailing_comments_stay_on_their_line,raf_okf::test_crate::render::a_parsed_document_equals_its_rendering_read_back,raf_okf::test_crate::splice::two_generated_blocks_change_only_their_byte_ranges -->
- The crate takes no position on tense or content. Checks on prose belong to the
  reconcile operation.
  <!-- raf:tested-by raf_okf::test_crate::profile::future_tense_prose_is_not_a_violation -->

## Explanation

The profile is deliberately narrower than its parent format. A base that requires one
key and leaves the rest open suits a wide ecosystem; a toolchain that must answer
"is this document complete" needs closed key sets whose per-type tables distinguish
required and optional keys. The common keys `tags` and `timestamp` are optional;
each record type states its other optional fields. A missing required key means a
missing fact rather than an optional one, and a misspelled key is an error rather
than a silent no-op.

The frontmatter adapter preserves scalar spelling and quote style, attached
comments, collection style and key/item positions. The public native YAML value
API resolves core scalars and does not retain that successful-node metadata, so
it cannot replace this lossless closed reader and writer. Shared quoted ECHAR
values come from the native lexical library; `0` to NUL is the profile's explicit
extension, and other native ECHAR members remain refused.

The body model exists so that no other crate scans Markdown. A line that begins with
the PROPOSED marker inside a code fence is documentation of the syntax, not a
proposal, and only a parser that tracks fences can tell the difference. Putting that
rule in one crate keeps the gate, the generator and the reconcile operation agreeing.

The writer is deterministic because generated blocks are rewritten in files that
people also edit. If rendering could reorder keys or alter whitespace, every sync
would produce noise in the diff, and a clean check could not be told from a stale one.

## Verification

The existing identity refusal case covers all eight types, wrong prefixes and
slugs, wrong singleton ids and the byte bound. Authoring round trips verify that
the shared constructor names the actual asserted root for every type.

The existing render cases exercise every tracked authoring document through the
native dataset inverse. Seven independent fixtures cover both canonical native
text carriers, reordered RDF interning, fresh construction, repeated values,
stable identity across body edits and malformed carriers beside valid neighbors.
<!-- raf:tested-by raf_okf::test_crate::render::each_fixture_document_renders_to_its_own_bytes_and_validates,raf_okf::test_crate::render::every_tracked_document_renders_to_its_own_bytes -->

- Native scalar fixtures preserve accepted escape values, exact refusal
  locations and numeric quoting; native-only escapes remain refused. Exact
  decimal fixtures retain spelling equality, negative-zero display and ordering
  beyond floating-point precision. Timestamp fixtures preserve diagnostic
  precedence on multiply-invalid inputs beside valid calendar/offset neighbors.
- The allocation-bound fixtures use native current-thread measurement windows
  around synchronous parsing. Their original byte allowances remain enforced,
  and the body fixture's counting allocator remains wrapped in a 1 GiB process
  cap. Deterministic body samples use the shared MMIX stream.
- A parse fixture per failure: no frontmatter, malformed YAML, a non-mapping
  frontmatter.
- A validation fixture per type for each missing key, each extra key, the key
  `resource`, and an unknown `type`; each asserts the named key and rule.
- A value fixture per rule: `state` outside its set, `bindings` missing a key,
  an `affects` entry with an unknown section, an `items` outcome outside
  `implemented`, `dropped` and `moved:change:<slug>`, and an item `reason` that is
  empty, blank or not text beside reasons accepted with each outcome and with none.
- Bound fixtures: a document of 1 MiB plus one byte, frontmatter nested 33 levels
  deep, a quote nested 65 levels deep, and a link destination or title of 4097 bytes
  each fail with a positioned error, and documents at the limit (a destination or
  title of 4096 bytes included) parse.
- An alias fixture: frontmatter with an anchor and an alias (a "billion laughs"
  document) fails with a positioned error naming the anchor. Native calling-thread
  allocation windows measure both this refusal and a large alias refusal against
  eight times the input plus 16 KiB. These synchronous parses spawn no workers;
  the heading fixture retains its separate hard 1 GiB process ceiling.
  `raf_okf::test_crate::alloc_bound::ALLOCATOR` registers the native probe for
  refused documents; `raf_okf::test_crate::bounds::ALLOCATOR` registers it through
  the hard-limit wrapper, whose `raf_okf::test_crate::bounds::CEILING` bounds the
  whole heading-test process. Registration is by `#[global_allocator]`, rather
  than an explicit call from a test body.
- A body fixture where a PROPOSED marker and a generated marker appear only inside a
  fence and produce no blocks, and fixtures for each marker error.
- Refusal fixtures for a leading byte order mark (no fence at line 1, column 1) and
  for a frontmatter fence line with a trailing space; a NUL in the body is kept and
  renders back.
- Cost fixtures, counted and not timed. A change at the 1 MiB cap with the most
  `changes` and `affects` entries it holds validates, and the count of `affects`
  entries judged equals the number of entries while the set of `changes` is built
  once, not once per entry. A capability at the cap with the most sections and
  generated blocks is refused for the duplicated section, and each generated block
  is looked at once. With sections and generated blocks growing together at one,
  two and three units, the blocks looked at are nonzero and the third unit adds no
  more than the second. The lines tested for a fence grow in step with the length of
  a page, and an unterminated fence is tested once for each line it swallows. A
  count sees the steps that are counted: a loop added elsewhere is not seen by it.
- A round-trip property over the fixture documents: `render(parse(x))` equals `x`
  for canonical input, and `parse(render(d))` equals `d`.
- An index fixture: a generated-block index is accepted and a hand-written one is
  rejected.
- A record fixture per rule, each beside an accepted neighbour: a verdict outside
  the set of each type, a `by` that names a person, a malformed digest, a short
  commit id, an empty exclusion reason, a crate-root exclusion, a repeated scope
  entry, a record in another directory or under another file name, and a
  coverage record under another slug.
- Ceiling and exception fixtures, each beside an accepted neighbour: ceilings of
  each class, a percent with a fraction and the bounds 0 and 100 accepted; an `of`
  of another value, both or neither of `lines` and `percent`, a fractional or a
  negative `lines`, a `percent` over 100 or negative, a value that is not a
  number, an unknown key, no `crate`, a crate that is not a package name, a class
  twice for a crate and a list that is not of maps refused; an exception of each
  target accepted and written again as it was read; a crate root, a prefix of one,
  a path that is neither, an absolute, a drive, a backslash, a `..`, an empty and a
  `.` segment, both and neither of `item` and `file`, an empty or a missing reason,
  a placeholder in the reason and a target twice refused; a record with both keys
  written and read back equal, and one with neither written without them.
- Claim fixtures: a claim with one record and with several, after a paragraph
  and after a list item; markers inside a fence, a code span and a PROPOSED block
  are not read; an unknown `raf:` comment is refused; a `raf:generated:` comment
  keeps its own rules; each malformed and each unattached marker is refused with
  its line.
- Tested-by fixtures: a marker after a paragraph, a list item, a heading and a
  claim marker, with its paths and statement, and a generic argument list whose
  comma does not split an entry; malformed with a space after a comma, a repeated
  entry, no entry, a `tests=` form and an entry that is not an `<item-path>`, each
  beside the marker written exactly; unattached after a blank line, after another
  comment and directly before a claim marker, beside the marker after the statement
  and after its claim marker; not read in a fence, a code span or a PROPOSED block;
  a violation outside a Capability and none inside one; the fixtures of the claim
  marker unchanged.
- The record fixtures render to their own bytes and validate, and a record with
  its keys out of order renders them in table order.
- A record of each type built with the constructors, and each record fixture,
  written by `to_document` reads back as an equal record with its keys in table
  order; empty lists and absent optional keys are written as the profile reads
  them; a value the profile refuses is refused on reading back, and a value the
  frontmatter cannot hold by the writer.
- A `raf:generated:` comment in another letter case or spacing is text, beside
  an unknown `raf:` comment that is refused.
- Split-marker fixtures of the body model: `<!--`, `raf:claim evidence=x` and `-->`
  on three lines, the same with a blank line and `  RAF:Claim` below `<!--`, and a
  comment opened after a statement on its line are each a malformed claim marker at
  the line of `<!--`, and `<!--`, `raf:foo`, `-->` is an unknown marker; beside them
  a split `raf:generated:` comment in two spellings, an SPDX header over several
  lines, an ordinary multi-line comment, an empty one and one opened in a code span
  are plain, and nothing is read in fenced code or a PROPOSED block. Through
  `check`, the three-line claim marker and the one with a blank line in a
  capability are `okf-parse-malformed-claim` and the `raf:foo` comment
  `okf-parse-unknown-marker`, each exit 1 with nothing written, and the split
  `raf:generated:` comment, the SPDX header and the ordinary comment exit 0.
- Cost fixture, counted and not timed: an evidence record at the 1 MiB cap with the
  most `values` and `limits` validates, each value is indexed once and each limit
  looked up once, and a table built for each limit would pass the count at the
  second limit.
- The documentation examples of `EvidenceRecord::to_document` (a record built,
  written, parsed and validated back to an equal record), of `Claim` (a claim
  marker read with its evidence, line, section and statement) and of `TestedBy` (a
  tested-by marker after a heading read with its paths, line, section and
  statement) run as the bound examples.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::test_crate::check::a_raf_comment_split_over_lines_fails_check_and_other_split_comments_pass` | test | [Verification](#verification) |
| `raf_okf::Claim` | doctest | [Verification](#verification) |
| `raf_okf::EvidenceRecord::to_document` | doctest | [Usage](#usage) |
| `raf_okf::Number::exact` | doctest | [Contract](#contract) |
| `raf_okf::TestedBy` | doctest | [Verification](#verification) |
| `raf_okf::error::tag_tests::all_lists_every_error_kind_once_in_variant_order` | test | [Verification](#verification) |
| `raf_okf::error::tag_tests::names_are_unique_lowercase_kebab_case` | test | [Verification](#verification) |
| `raf_okf::profile::cost_tests::a_capability_at_the_cap_looks_at_each_block_once` | test | [Verification](#verification) |
| `raf_okf::profile::cost_tests::a_change_at_the_cap_judges_each_affects_entry_once_against_one_set` | test | [Verification](#verification) |
| `raf_okf::profile::cost_tests::an_evidence_record_at_the_cap_indexes_each_value_once_and_looks_up_each_limit_once` | test | [Verification](#verification) |
| `raf_okf::profile::cost_tests::an_unterminated_fence_is_tested_once_for_each_line_it_swallows` | test | [Verification](#verification) |
| `raf_okf::profile::cost_tests::the_blocks_looked_at_grow_with_sections_and_blocks_together` | test | [Verification](#verification) |
| `raf_okf::profile::cost_tests::the_lines_tested_for_a_fence_grow_with_the_length_of_the_page` | test | [Verification](#verification) |
| `raf_okf::profile::violation::tag_tests::all_lists_every_rule_once_in_variant_order` | test | [Verification](#verification) |
| `raf_okf::profile::violation::tag_tests::names_are_unique_lowercase_kebab_case` | test | [Verification](#verification) |
| `raf_okf::test_crate::alloc_bound::refused_documents_stay_within_eight_times_their_size` | test | [Contract](#contract) |
| `raf_okf::test_crate::body_robustness::random_token_soup_never_panics_and_parsed_spans_lie_in_the_text` | test | [Contract](#contract) |
| `raf_okf::test_crate::body_structure::a_body_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused` | test | [Verification](#verification) |
| `raf_okf::test_crate::body_structure::a_document_built_with_a_malformed_body_reports_the_line_in_the_rendered_text` | test | [Verification](#verification) |
| `raf_okf::test_crate::body_structure::a_fence_closes_only_on_the_same_character_with_at_least_the_same_length` | test | [Contract](#contract) |
| `raf_okf::test_crate::body_structure::a_fenced_block_exposes_its_content_and_whether_it_is_closed` | test | [Contract](#contract) |
| `raf_okf::test_crate::body_structure::a_fenced_block_prints_and_compares_without_the_body_around_it` | test | [Verification](#verification) |
| `raf_okf::test_crate::body_structure::a_hundred_thousand_quote_markers_are_refused_on_a_small_stack` | test | [Contract](#contract) |
| `raf_okf::test_crate::body_structure::a_section_that_ends_the_body_without_a_final_newline_reaches_the_end` | test | [Contract](#contract) |
| `raf_okf::test_crate::body_structure::a_thematic_break_is_not_a_run_of_list_markers` | test | [Verification](#verification) |
| `raf_okf::test_crate::body_structure::a_three_space_heading_is_a_section_and_a_four_space_heading_is_not` | test | [Verification](#verification) |
| `raf_okf::test_crate::body_structure::block_nesting_of_sixty_four_levels_parses_and_sixty_five_is_refused` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_okf::test_crate::body_structure::code_spans_of_any_backtick_length_hide_their_content` | test | [Contract](#contract) |
| `raf_okf::test_crate::body_structure::fences_are_recognised_indented_up_to_three_spaces_and_inside_quotes_and_lists` | test | [Contract](#contract) |
| `raf_okf::test_crate::body_structure::headings_inside_fences_and_comments_are_not_headings` | test | [Contract](#contract) |
| `raf_okf::test_crate::body_structure::headings_of_every_level_carry_github_anchors_with_repeats_numbered` | test | [Contract](#contract) |
| `raf_okf::test_crate::body_structure::indented_code_is_ordinary_text_so_links_and_markers_in_it_count` | test | [Constraints](#constraints) |
| `raf_okf::test_crate::body_structure::markdown_without_frontmatter_is_the_whole_body` | test | [Contract](#contract) |
| `raf_okf::test_crate::body_structure::only_atx_headings_are_headings_and_setext_headings_are_not` | test | [Constraints](#constraints) |
| `raf_okf::test_crate::body_structure::prose_lines_skip_fences_and_comments_and_remove_code_spans` | test | [Contract](#contract) |
| `raf_okf::test_crate::body_structure::section_text_is_the_content_of_the_section` | test | [Verification](#verification) |
| `raf_okf::test_crate::body_structure::sections_carry_heading_slug_and_content_up_to_the_next_second_level_heading` | test | [Contract](#contract) |
| `raf_okf::test_crate::body_structure::spans_are_in_whole_text_coordinates_when_frontmatter_precedes_the_body` | test | [Contract](#contract) |
| `raf_okf::test_crate::bounds::a_long_heading_over_many_lines_stays_linear` | test | [Verification](#verification) |
| `raf_okf::test_crate::claims::a_capability_states_claims_in_its_sections` | test | [Verification](#verification) |
| `raf_okf::test_crate::claims::a_claim_after_a_list_item_supports_that_item_only` | test | [Contract](#contract) |
| `raf_okf::test_crate::claims::a_claim_marker_names_its_evidence_and_supports_the_paragraph_above_it` | test | [Contract](#contract) |
| `raf_okf::test_crate::claims::a_claim_marker_with_no_statement_directly_above_is_refused` | test | [Contract](#contract) |
| `raf_okf::test_crate::claims::a_claim_may_rest_on_several_records` | test | [Verification](#verification) |
| `raf_okf::test_crate::claims::a_malformed_claim_marker_is_refused_with_its_line` | test | [Contract](#contract) |
| `raf_okf::test_crate::claims::a_raf_comment_split_over_lines_is_a_marker_error_and_other_split_comments_are_text` | test | [Contract](#contract) |
| `raf_okf::test_crate::claims::a_raf_generated_comment_in_another_case_or_spacing_is_text` | test | [Verification](#verification) |
| `raf_okf::test_crate::claims::a_raf_generated_comment_keeps_its_own_rules` | test | [Verification](#verification) |
| `raf_okf::test_crate::claims::a_statement_stops_at_a_heading_a_blank_line_or_a_fence` | test | [Verification](#verification) |
| `raf_okf::test_crate::claims::claims_in_a_document_carry_whole_text_lines` | test | [Verification](#verification) |
| `raf_okf::test_crate::claims::every_other_raf_comment_is_refused_with_its_line` | test | [Contract](#contract) |
| `raf_okf::test_crate::claims::markers_in_fences_code_spans_and_proposed_blocks_are_not_read` | test | [Contract](#contract) |
| `raf_okf::test_crate::index::a_generated_only_index_is_accepted_with_either_header_form` | test | [Contract](#contract) |
| `raf_okf::test_crate::index::a_header_with_unknown_tags_or_in_the_wrong_place_is_refused` | test | [Contract](#contract) |
| `raf_okf::test_crate::index::an_empty_header_tag_is_refused` | test | [Contract](#contract) |
| `raf_okf::test_crate::index::an_index_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused` | test | [Verification](#verification) |
| `raf_okf::test_crate::index::an_index_with_no_block_or_an_unclosed_block_is_refused` | test | [Contract](#contract) |
| `raf_okf::test_crate::index::document_parse_of_index_text_fails_at_line_one` | test | [Contract](#contract) |
| `raf_okf::test_crate::index::indented_and_mid_line_markers_are_refused` | test | [Contract](#contract) |
| `raf_okf::test_crate::index::only_the_capability_index_path_is_the_index` | test | [Contract](#contract) |
| `raf_okf::test_crate::index::prose_outside_the_blocks_is_refused_with_its_line` | test | [Contract](#contract) |
| `raf_okf::test_crate::links::a_destination_or_title_of_exactly_4096_bytes_is_read_and_one_more_is_refused` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_okf::test_crate::links::a_link_split_across_lines_inside_a_block_quote_is_recorded` | test | [Contract](#contract) |
| `raf_okf::test_crate::links::a_link_split_across_lines_is_recorded_and_not_read` | test | [Contract](#contract) |
| `raf_okf::test_crate::links::a_space_between_the_text_and_the_parenthesis_is_not_a_link` | test | [Verification](#verification) |
| `raf_okf::test_crate::links::a_split_link_does_not_hide_a_link_after_it` | test | [Verification](#verification) |
| `raf_okf::test_crate::links::an_attribute_value_that_starts_on_the_next_line_is_read` | test | [Verification](#verification) |
| `raf_okf::test_crate::links::an_escaped_closing_parenthesis_belongs_to_the_destination` | test | [Verification](#verification) |
| `raf_okf::test_crate::links::an_html_tag_split_across_lines_yields_its_attributes_with_their_lines` | test | [Verification](#verification) |
| `raf_okf::test_crate::links::every_link_form_is_read_with_its_target_anchor_and_line` | test | [Contract](#contract) |
| `raf_okf::test_crate::links::html_attributes_are_read_with_spacing_case_and_unquoted_values` | test | [Verification](#verification) |
| `raf_okf::test_crate::links::links_inside_code_spans_of_any_length_and_fences_are_not_links` | test | [Contract](#contract) |
| `raf_okf::test_crate::links::links_inside_html_comments_are_not_links` | test | [Contract](#contract) |
| `raf_okf::test_crate::links::nested_brackets_images_in_links_and_stray_brackets_are_read` | test | [Verification](#verification) |
| `raf_okf::test_crate::links::only_ascii_space_and_controls_end_a_destination_not_unicode_space` | test | [Verification](#verification) |
| `raf_okf::test_crate::links::reference_definitions_take_angle_targets_and_skip_footnotes` | test | [Verification](#verification) |
| `raf_okf::test_crate::links::split_links_are_not_recorded_in_fences_code_spans_or_for_one_line_links` | test | [Contract](#contract) |
| `raf_okf::test_crate::links::tag_state_does_not_survive_a_blank_line_or_a_fence` | test | [Verification](#verification) |
| `raf_okf::test_crate::links::targets_resolve_against_the_document_path` | test | [Contract](#contract) |
| `raf_okf::test_crate::links::targets_that_leave_the_root_or_are_empty_or_absolute_are_refused` | test | [Contract](#contract) |
| `raf_okf::test_crate::links::the_tag_scan_spares_prose_autolinks_and_closed_tags` | test | [Verification](#verification) |
| `raf_okf::test_crate::markers::a_proposed_block_reports_slug_spans_content_and_section` | test | [Contract](#contract) |
| `raf_okf::test_crate::markers::document_parse_builds_its_body_through_the_body_model` | test | [Verification](#verification) |
| `raf_okf::test_crate::markers::each_generated_marker_error_names_its_line` | test | [Contract](#contract) |
| `raf_okf::test_crate::markers::every_misspelling_of_the_proposed_marker_is_refused_with_its_line` | test | [Contract](#contract) |
| `raf_okf::test_crate::markers::generated_blocks_report_kind_outer_and_content_spans` | test | [Contract](#contract) |
| `raf_okf::test_crate::markers::marker_errors_in_a_document_carry_whole_text_lines` | test | [Verification](#verification) |
| `raf_okf::test_crate::markers::markers_inside_a_fence_produce_no_blocks` | test | [Verification](#verification) |
| `raf_okf::test_crate::markers::two_markers_in_a_row_make_two_blocks` | test | [Contract](#contract) |
| `raf_okf::test_crate::no_io_in_source::the_crate_sources_name_neither_the_file_system_nor_the_process_module` | test | [Verification](#verification) |
| `raf_okf::test_crate::no_io_in_source::the_scanner_finds_every_spelling_of_a_forbidden_module` | test | [Verification](#verification) |
| `raf_okf::test_crate::no_io_in_source::the_scanner_leaves_neighbouring_paths_alone` | test | [Verification](#verification) |
| `raf_okf::test_crate::placeholder_lines::does_not_read_code_or_comments` | test | [Verification](#verification) |
| `raf_okf::test_crate::placeholder_lines::reports_each_token_in_prose_with_its_line` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::a_capability_holds_the_seven_sections_in_order` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::a_change_needs_a_non_empty_summary` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::a_delivered_capability_with_code_and_tests_is_typed_with_its_bindings` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::a_document_must_sit_directly_in_the_directory_of_its_type` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::a_document_with_three_faults_returns_three_violations` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_okf::test_crate::profile::a_missing_type_key_is_the_only_violation` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::a_nested_unknown_key_is_reported_in_bindings_and_in_a_change_item` | test | [Constraints](#constraints) |
| `raf_okf::test_crate::profile::a_placeholder_token_in_a_code_span_of_a_frontmatter_value_is_spared` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::a_proposed_block_is_valid_only_inside_a_section_of_a_capability` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::a_section_is_empty_when_only_comments_and_generated_blocks_remain` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::a_split_link_is_reported_at_its_line_and_a_one_line_link_is_not` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::a_state_is_one_of_the_four` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::a_title_and_a_description_are_non_empty_strings` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::a_violation_displays_the_file_the_place_and_the_rule` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::affects_entries_have_the_form_a_section_of_the_seven_and_a_capability_in_changes` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::an_architecture_body_is_not_empty` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::an_id_needs_the_prefix_of_its_type_a_kebab_slug_and_the_file_name` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_okf::test_crate::profile::an_invalid_changes_entry_does_not_cascade_into_the_affects_entries` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::an_item_reason_is_a_non_empty_line_beside_any_outcome_and_an_empty_one_is_refused` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::an_unknown_key_is_reported_at_that_key` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::an_unknown_or_foreign_key_is_reported_on_every_type` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_okf::test_crate::profile::an_unknown_type_is_one_violation_at_the_type_key` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::bindings_by_state_designed_and_considered_bind_nothing_delivered_binds_code_and_tests` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::bindings_have_exactly_the_keys_code_tests_and_examples_each_a_list` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::changes_is_a_non_empty_list_of_cap_ids` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::contract_effect_is_none_or_changes` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::depends_is_a_list_of_cap_ids_and_does_not_need_the_capability_to_exist` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::every_required_key_of_every_type_is_reported_when_missing` | test | [Contract](#contract) |
| `raf_okf::test_crate::profile::future_tense_prose_is_not_a_violation` | test | [Constraints](#constraints) |
| `raf_okf::test_crate::profile::items_need_a_claim_known_keys_and_a_known_outcome` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::placeholder_scan_spares_code_longer_words_and_dotted_prose` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::placeholder_tokens_in_frontmatter_values_are_reported_at_the_key` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::placeholder_tokens_in_prose_are_reported_at_their_line` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::tags_are_non_empty_unique_strings_and_a_timestamp_is_rfc_3339` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::the_key_resource_has_its_own_rule_on_every_type` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::the_three_outcomes_are_typed` | test | [Verification](#verification) |
| `raf_okf::test_crate::profile::valid_documents_of_each_type_validate_to_typed_values` | test | [Constraints](#constraints) |
| `raf_okf::test_crate::record_writer::a_built_assessment_and_coverage_record_read_back_equal` | test | [Contract](#contract) |
| `raf_okf::test_crate::record_writer::a_built_evidence_record_renders_to_a_record_that_reads_back_equal` | test | [Contract](#contract) |
| `raf_okf::test_crate::record_writer::a_coverage_record_writes_only_its_closed_policy_independent_keys` | test | [Contract](#contract) |
| `raf_okf::test_crate::record_writer::a_debt_record_is_written_sorted_and_read_back_equal` | test | [Contract](#contract) |
| `raf_okf::test_crate::record_writer::a_debt_record_with_no_debt_writes_two_empty_lists` | test | [Verification](#verification) |
| `raf_okf::test_crate::record_writer::a_debt_row_the_profile_refuses_is_written_and_refused_when_it_is_read_back` | test | [Verification](#verification) |
| `raf_okf::test_crate::record_writer::a_value_the_frontmatter_cannot_hold_is_refused_by_the_writer` | test | [Contract](#contract) |
| `raf_okf::test_crate::record_writer::a_value_the_profile_refuses_is_written_and_refused_when_it_is_read_back` | test | [Contract](#contract) |
| `raf_okf::test_crate::record_writer::ceilings_and_exceptions_are_written_after_the_exclusions_and_read_back_equal` | test | [Contract](#contract) |
| `raf_okf::test_crate::record_writer::empty_lists_and_absent_keys_are_written_as_the_profile_reads_them` | test | [Verification](#verification) |
| `raf_okf::test_crate::record_writer::every_record_fixture_reads_back_equal_after_writing` | test | [Verification](#verification) |
| `raf_okf::test_crate::record_writer::the_keys_are_written_in_the_order_of_the_key_table` | test | [Contract](#contract) |
| `raf_okf::test_crate::record_writer::verdict_and_subject_names_are_the_values_of_their_keys` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::a_ceiling_names_a_crate_one_whole_number_of_lines_or_a_percent_and_an_optional_class` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::a_ceiling_of_another_shape_is_refused_with_its_rule` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::a_coverage_record_rejects_the_removed_policy_key` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::a_digest_is_sixty_four_lowercase_hex_digits` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::a_list_that_holds_each_entry_once_refuses_a_repeat` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::a_passing_record_holds_no_value_above_its_limit` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::a_record_has_no_title_description_or_body` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::a_record_sits_in_its_directory_and_its_id_matches_its_file` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::a_record_type_is_named_in_the_unknown_type_message` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::a_revision_is_a_full_commit_id` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::a_row_of_findings_holds_exactly_a_kind_and_a_subject_each_pair_once` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::a_row_of_lines_holds_exactly_an_item_and_a_whole_number_of_lines_each_item_once` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::a_valid_assessment_validates_to_its_typed_value` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::a_valid_coverage_record_validates_and_an_empty_scope_opts_out_visibly` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::a_valid_debt_record_validates_to_its_typed_value_and_an_empty_one_opts_out_visibly` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::a_valid_evidence_record_validates_to_its_typed_value` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::a_value_over_its_limit_is_judged_on_its_text_and_not_its_nearest_float` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::a_verdict_is_one_of_the_verdicts_of_its_type` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::an_assessment_holds_the_section_row_of_its_subject` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::an_exception_names_one_item_prefix_file_or_directory_and_its_reason` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::an_exception_of_another_shape_or_without_a_reason_is_refused_with_its_rule` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::an_exclusion_has_a_reason_with_no_placeholder` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::an_exclusion_names_an_item_or_a_module_prefix_below_a_crate_root` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::an_input_kind_is_one_the_type_takes_and_its_name_fits_the_kind` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::both_lists_of_a_debt_record_are_required` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::by_is_human_or_agent_and_never_a_name` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::coverage_is_one_record_at_its_one_path` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::debt_is_one_record_at_its_one_path` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::evidence_from_a_bound_test_has_no_command_and_may_have_no_measures` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::for_lists_at_least_one_capability_id` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::input_rows_have_exactly_kind_name_and_digest` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::numbers_compare_exactly_by_sign_whole_part_and_fraction` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::scope_holds_crate_package_names` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::symbol_inputs_accept_crate_roots_and_refuse_malformed_paths` | test | [Contract](#contract) |
| `raf_okf::test_crate::records::the_command_key_is_present_exactly_when_the_subject_is_command` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::the_subject_takes_the_form_of_its_type` | test | [Verification](#verification) |
| `raf_okf::test_crate::records::values_and_limits_are_numbers_and_each_limit_names_a_value` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::a_billion_laughs_document_is_refused_at_the_first_anchor` | test | [Contract](#contract) |
| `raf_okf::test_crate::refusals::a_colon_space_inside_a_flow_list_is_refused` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::a_document_without_a_fence_on_line_one_is_refused` | test | [Contract](#contract) |
| `raf_okf::test_crate::refusals::a_duplicate_key_is_refused_at_every_level` | test | [Contract](#contract) |
| `raf_okf::test_crate::refusals::a_flow_mapping_value_is_refused_naming_the_key` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::a_frontmatter_fence_line_with_a_trailing_space_is_not_a_fence` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::a_frontmatter_that_is_not_a_block_mapping_is_refused` | test | [Contract](#contract) |
| `raf_okf::test_crate::refusals::a_hundred_thousand_levels_return_an_error_on_a_small_stack` | test | [Contract](#contract) |
| `raf_okf::test_crate::refusals::a_leading_byte_order_mark_is_refused_at_line_one_column_one` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::a_nul_in_the_body_is_kept_and_renders_back` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::a_text_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_okf::test_crate::refusals::an_unclosed_fence_is_refused_at_the_opening_fence` | test | [Contract](#contract) |
| `raf_okf::test_crate::refusals::an_unterminated_quote_and_text_after_a_quote_are_refused` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::anchors_aliases_tags_and_merge_keys_are_refused_naming_the_token` | test | [Contract](#contract) |
| `raf_okf::test_crate::refusals::block_indicators_inside_an_items_entry_are_refused` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::block_scalars_are_refused_naming_the_header` | test | [Contract](#contract) |
| `raf_okf::test_crate::refusals::flow_list_shape_errors_are_refused` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::items_keep_their_maps_and_scalars` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::layout_errors_are_refused_with_their_position` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::line_endings_must_agree_within_the_frontmatter` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::nesting_over_thirty_two_levels_is_refused_before_descent` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_okf::test_crate::refusals::numbers_booleans_and_null_are_read_as_strings` | test | [Contract](#contract) |
| `raf_okf::test_crate::refusals::plain_scalars_with_a_colon_space_or_an_indicator_are_refused` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::stray_and_over_indented_lines_after_a_flow_value_are_refused` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::text_that_plain_ok_accepts_is_read_back_plain_in_every_position` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::text_that_the_reader_refuses_is_never_plain_ok` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::trailing_comments_are_not_part_of_the_value` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::unknown_escapes_are_refused_and_every_known_escape_is_read` | test | [Verification](#verification) |
| `raf_okf::test_crate::refusals::unterminated_quotes_are_refused_in_flow_list_item_and_nested_item_positions` | test | [Verification](#verification) |
| `raf_okf::test_crate::render::a_built_document_with_sorted_tags_equals_what_it_reads_back_as` | test | [Verification](#verification) |
| `raf_okf::test_crate::render::a_change_document_built_from_constructors_renders_and_reads_back_equal` | test | [Constraints](#constraints) |
| `raf_okf::test_crate::render::a_document_of_an_unknown_type_keeps_input_order` | test | [Contract](#contract) |
| `raf_okf::test_crate::render::a_parsed_document_equals_its_rendering_read_back` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_okf::test_crate::render::a_parsed_empty_value_renders_as_an_empty_value` | test | [Verification](#verification) |
| `raf_okf::test_crate::render::comments_attach_to_the_next_key_and_trailing_comments_stay_on_their_line` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_okf::test_crate::render::constructed_nesting_and_size_are_bounded_like_the_reader` | test | [Verification](#verification) |
| `raf_okf::test_crate::render::constructors_choose_a_style_the_reader_reads_back` | test | [Verification](#verification) |
| `raf_okf::test_crate::render::constructors_refuse_what_the_reader_would_not_read_back` | test | [Verification](#verification) |
| `raf_okf::test_crate::render::crlf_documents_and_a_missing_final_newline_render_byte_identically` | test | [Verification](#verification) |
| `raf_okf::test_crate::render::each_fixture_document_renders_to_its_own_bytes_and_validates` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_okf::test_crate::render::equality_compares_keys_as_a_map_with_comments_and_body_bytes` | test | [Contract](#contract) |
| `raf_okf::test_crate::render::every_escape_round_trips_and_other_control_characters_are_refused` | test | [Verification](#verification) |
| `raf_okf::test_crate::render::every_tracked_document_renders_to_its_own_bytes` | test | [Contract](#contract) |
| `raf_okf::test_crate::render::flow_lists_are_written_in_one_spacing` | test | [Verification](#verification) |
| `raf_okf::test_crate::render::keys_out_of_order_render_in_table_order_and_then_stay_put` | test | [Contract](#contract) |
| `raf_okf::test_crate::render::keys_outside_the_table_follow_in_input_order` | test | [Contract](#contract) |
| `raf_okf::test_crate::render::record_keys_out_of_order_render_in_table_order_and_validate` | test | [Verification](#verification) |
| `raf_okf::test_crate::render::tags_render_sorted_and_stay_sorted` | test | [Contract](#contract) |
| `raf_okf::test_crate::render::the_body_span_locates_the_body_in_the_text` | test | [Verification](#verification) |
| `raf_okf::test_crate::render::the_body_span_uses_the_coordinates_of_the_text_the_document_came_from` | test | [Verification](#verification) |
| `raf_okf::test_crate::render::unsorted_tags_render_sorted_and_the_document_equals_its_rendering_read_back` | test | [Verification](#verification) |
| `raf_okf::test_crate::repo_markdown::no_tracked_markdown_file_holds_a_split_link_or_a_malformed_block` | test | [Verification](#verification) |
| `raf_okf::test_crate::schema::a_type_is_found_by_its_exact_name_only` | test | [Verification](#verification) |
| `raf_okf::test_crate::schema::each_type_lists_its_keys_in_the_order_of_the_architecture_page` | test | [Contract](#contract) |
| `raf_okf::test_crate::schema::each_type_states_its_required_and_optional_keys` | test | [Contract](#contract) |
| `raf_okf::test_crate::schema::malformed_and_out_of_range_timestamps_are_refused` | test | [Contract](#contract) |
| `raf_okf::test_crate::schema::singleton_and_record_roles_follow_the_document_types` | test | [Contract](#contract) |
| `raf_okf::test_crate::schema::timestamp_diagnostics_preserve_lexical_calendar_and_offset_order` | test | [Verification](#verification) |
| `raf_okf::test_crate::schema::valid_timestamps_are_accepted` | test | [Contract](#contract) |
| `raf_okf::test_crate::splice::a_document_delegates_and_renders_the_edit_with_its_frontmatter` | test | [Contract](#contract) |
| `raf_okf::test_crate::splice::a_document_edit_over_one_mebibyte_is_refused_and_the_document_is_unchanged` | test | [Verification](#verification) |
| `raf_okf::test_crate::splice::a_document_edit_with_a_malformed_result_is_refused_and_the_document_is_unchanged` | test | [Verification](#verification) |
| `raf_okf::test_crate::splice::a_document_whose_closing_fence_has_no_line_ending_gains_one_for_a_body` | test | [Verification](#verification) |
| `raf_okf::test_crate::splice::a_malformed_result_is_refused_and_the_body_is_unchanged` | test | [Contract](#contract) |
| `raf_okf::test_crate::splice::a_second_call_uses_spans_read_from_the_rebuilt_model` | test | [Verification](#verification) |
| `raf_okf::test_crate::splice::an_empty_edit_list_leaves_the_body_equal` | test | [Verification](#verification) |
| `raf_okf::test_crate::splice::edits_given_out_of_order_apply_in_one_pass` | test | [Contract](#contract) |
| `raf_okf::test_crate::splice::overlapping_spans_are_refused_and_the_body_is_unchanged` | test | [Contract](#contract) |
| `raf_okf::test_crate::splice::spans_outside_the_body_or_inside_a_character_are_refused` | test | [Contract](#contract) |
| `raf_okf::test_crate::splice::the_catalogue_of_an_index_is_replaced_through_the_index` | test | [Contract](#contract) |
| `raf_okf::test_crate::splice::two_generated_blocks_change_only_their_byte_ranges` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_okf::test_crate::tested_by::a_malformed_tested_by_marker_is_refused_with_its_line` | test | [Contract](#contract) |
| `raf_okf::test_crate::tested_by::a_tested_by_marker_belongs_in_a_capability_document` | test | [Verification](#verification) |
| `raf_okf::test_crate::tested_by::a_tested_by_marker_maps_a_list_item_a_heading_or_the_statement_of_its_claim_marker` | test | [Contract](#contract) |
| `raf_okf::test_crate::tested_by::a_tested_by_marker_names_its_tests_and_maps_the_paragraph_above_it` | test | [Contract](#contract) |
| `raf_okf::test_crate::tested_by::a_tested_by_marker_with_no_statement_directly_above_is_refused` | test | [Contract](#contract) |
| `raf_okf::test_crate::tested_by::tested_by_markers_in_fences_code_spans_and_proposed_blocks_are_not_read` | test | [Contract](#contract) |
| `raf_okf::test_crate::usage_docs::the_examples_block_validates_between_fences_and_resource_is_rejected` | test | [Verification](#verification) |
| `raf_okf::test_crate::validate_cost::a_capability_with_many_sections_and_blocks_is_refused_for_a_duplicated_section_at_the_cap` | test | [Verification](#verification) |
| `raf_okf::test_crate::validate_cost::a_change_with_maximal_affects_and_changes_validates_at_the_cap` | test | [Verification](#verification) |
| `raf_okf::test_crate::validate_cost::an_evidence_record_with_maximal_values_and_limits_validates_at_the_cap` | test | [Verification](#verification) |
| `raf_okf::work::tests::a_counter_increments_by_what_is_added` | test | [Verification](#verification) |
| `raf_okf::work::tests::a_limited_kind_panics_at_its_ceiling` | test | [Verification](#verification) |
| `raf_okf::work::tests::a_limited_measure_counts_the_other_kinds_without_a_ceiling` | test | [Verification](#verification) |
| `raf_okf::work::tests::a_measure_that_panicked_leaves_neither_counts_nor_ceiling_behind` | test | [Verification](#verification) |
| `raf_okf::work::tests::every_kind_has_its_own_counter` | test | [Verification](#verification) |
| `raf_okf::work::tests::passing_the_ceiling_panics_at_once` | test | [Verification](#verification) |
| `raf_okf::work::tests::reaching_the_ceiling_is_allowed` | test | [Verification](#verification) |
| `raf_okf::work::tests::the_panic_comes_before_the_work_that_follows_it` | test | [Verification](#verification) |
| `raf_okf::work::tests::the_result_of_the_closure_is_returned` | test | [Verification](#verification) |
| `raf_okf::work::tests::the_tallies_are_reset_between_measures` | test | [Verification](#verification) |
| `raf_workplan::test_crate::sync::nothing_is_in_the_report_for_a_repository_that_is_in_sync` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples) |
<!-- raf:generated:end -->
