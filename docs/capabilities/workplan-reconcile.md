---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-reconcile
title: Static change reconciliation
description: Typed document obligations and native merge-base comparisons without consumer execution.
state: delivered
bindings:
  code:
    - raf_workplan::Workplan::new_change
    - raf_workplan::Workplan::impact
    - raf_workplan::Workplan::reconcile
    - raf_workplan::lifecycle::Lifecycle::reconcile
    - raf_workplan::obligation
    - raf_workplan::lifecycle::touched
    - raf_workplan::lifecycle::untouched
    - raf_workplan::lifecycle::changed
    - raf_workplan::lifecycle::NewChangeRequest
    - raf_workplan::lifecycle::NewChange
    - raf_workplan::lifecycle::NewChangeError
    - raf_workplan::lifecycle::Impact
    - raf_workplan::obligation::Reconciled
    - raf_workplan::obligation::Obligation
    - raf_workplan::obligation::ObligationKind
    - raf_workplan::obligation::Needed
    - raf_workplan::obligation::suppressed
    - raf_workplan::obligation::tense::PHRASES
    - raf_workplan::lifecycle::Lifecycle::open
    - raf_workplan::lifecycle::section_digest
    - raf_workplan::git::show_prefix
    - raf_workplan::git::default_merge_base
    - raf_workplan::git::blob_at
    - raf_workplan::git::MergeBase
    - raf_workplan::git::ResolvedCommit
    - raf_workplan::git::resolve_commit
    - raf_workplan::git::resolve_commit_id
    - raf_workplan::obligation::ClaimProblem
    - raf_workplan::obligation::OverLimit
    - raf_workplan::obligation::InputMoved
    - raf_workplan::obligation::EvidenceChange
    - raf_workplan::lifecycle
    - raf_workplan::git
  tests:
    - raf_workplan::obligation::tests::only_the_three_history_dependent_kinds_need_more_than_local_documents
    - raf_workplan::obligation::tests::the_evidence_strings_are_the_evidence_column_of_the_contract
    - raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind
    - raf_workplan::test_crate::lifecycle::new_writes_one_file_with_the_keys_the_id_and_a_summary_and_it_loads_clean
    - raf_workplan::test_crate::untouched::an_unchanged_affected_section_is_owed_with_the_line_the_contract_shows
    - raf_workplan::test_crate::complete::prospective_completion_refuses_an_implementing_capability_and_an_undone_item
    - raf_workplan::test_crate::lifecycle::a_bad_slug_is_refused_before_anything_is_written_and_a_good_one_next_to_it_is_not
    - raf_workplan::test_crate::lifecycle::an_existing_file_is_refused_and_left_byte_identical
    - raf_workplan::test_crate::lifecycle::a_capability_that_does_not_exist_is_refused
    - raf_workplan::test_crate::lifecycle::a_capability_whose_document_did_not_load_is_named_and_never_skipped
    - raf_workplan::test_crate::lifecycle::an_affects_entry_must_name_a_listed_capability_and_one_of_the_seven_sections
    - raf_workplan::test_crate::lifecycle::several_capabilities_are_listed_in_the_order_given_without_repeats
    - raf_workplan::test_crate::lifecycle::the_default_affects_is_the_contract_of_each_listed_capability
    - raf_workplan::test_crate::lifecycle::explicit_affects_are_written_as_given_and_impact_reports_them_sorted
    - raf_workplan::test_crate::lifecycle::the_guidance_follows_the_state_of_each_listed_capability
    - raf_workplan::test_crate::lifecycle::the_printed_text_holds_the_template_and_the_guidance
    - raf_workplan::test_crate::lifecycle::the_template_in_a_capability_of_each_state_gives_one_block_and_a_clean_check
    - raf_workplan::test_crate::lifecycle::both_spdx_fields_open_the_record_with_the_two_comment_lines_of_the_records_of_the_repository
    - raf_workplan::test_crate::lifecycle::one_spdx_field_without_the_other_is_an_error_that_names_the_missing_one
    - raf_workplan::test_crate::lifecycle::impact_gives_the_capabilities_sections_bindings_dependants_and_linking_sections
    - raf_workplan::test_crate::lifecycle::impact_prints_each_part_in_a_fixed_order
    - raf_workplan::test_crate::obligations::each_fault_is_owed_with_its_kind_location_and_evidence_and_fixing_it_removes_the_line
    - raf_workplan::test_crate::obligations::every_document_local_kind_is_raised_and_history_kinds_are_not
    - raf_workplan::test_crate::obligations::the_binding_obligations_are_the_gaps_of_the_roadmap
    - raf_workplan::test_crate::obligations::contract_effect_none_keeps_every_obligation_the_documents_can_show
    - raf_workplan::test_crate::obligations::equal_inputs_print_equal_output
    - raf_workplan::test_crate::obligations::wording_that_only_looks_like_a_phrase_owes_nothing
    - raf_workplan::test_crate::obligations::text_of_a_proposed_block_is_not_read_for_wording
    - raf_workplan::test_crate::obligations::reconcile_writes_nothing
    - raf_workplan::test_crate::obligations::each_assessment_fault_is_owed_at_the_section_naming_the_input_and_a_current_acceptance_clears_it
    - raf_workplan::test_crate::obligations::contract_effect_none_keeps_the_assessments_and_another_section_is_not_judged
    - raf_workplan::test_crate::assessments::with_no_bound_symbol_named_every_bound_symbol_is_an_input
    - raf_workplan::test_crate::assessments::an_evidence_record_a_claim_of_the_section_names_is_an_input
    - cargo_workplan::test_crate::reconcile::an_unlistable_public_surface_is_exit_two_and_the_resolved_neighbor_answers
    - cargo_workplan::test_crate::reconcile::unparseable_merge_base_source_is_an_error_beside_a_readable_base
    - raf_workplan::test_crate::obligations::a_dropped_or_moved_item_needs_a_reason_and_an_explained_outcome_is_accepted
    - raf_workplan::test_crate::untouched::a_section_edited_on_the_branch_is_not_reported_and_its_neighbour_is
    - raf_workplan::test_crate::untouched::an_edit_that_is_not_committed_counts_because_the_working_tree_is_compared
    - raf_workplan::test_crate::untouched::only_a_generated_block_changed_is_still_reported
    - raf_workplan::test_crate::untouched::a_file_absent_at_the_merge_base_counts_as_touched
    - raf_workplan::test_crate::untouched::on_the_default_branch_itself_every_affected_section_is_owed_and_it_is_not_an_error
    - raf_workplan::test_crate::untouched::a_linked_work_tree_compares_its_own_head
    - raf_workplan::test_crate::untouched::a_workspace_below_the_top_of_the_repository_is_compared_at_its_prefix
    - raf_workplan::test_crate::untouched::a_local_branch_named_origin_main_does_not_stand_in_for_the_remote_one
    - raf_workplan::test_crate::untouched::a_default_branch_named_like_an_option_is_resolved_and_never_reaches_git_as_one
    - raf_workplan::test_crate::untouched::no_default_branch_is_an_error_that_names_the_fix_and_the_fix_clears_it
    - raf_workplan::test_crate::untouched::a_file_over_one_mebibyte_at_the_merge_base_is_an_error_and_exactly_one_is_read
    - raf_workplan::test_crate::untouched::a_file_at_the_merge_base_that_does_not_parse_is_an_error_and_not_a_pass
    - raf_workplan::test_crate::untouched::contract_effect_none_keeps_usage_and_examples_and_requires_a_default_branch
    - raf_workplan::test_crate::untouched::the_digest_leaves_out_generated_blocks_on_both_sides_and_nothing_else
    - raf_workplan::test_crate::untouched::reconcile_writes_nothing_and_prints_the_same_list_twice
    - raf_workplan::test_crate::scenarios::several_redesigns_leave_one_readable_present
    - raf_workplan::test_crate::scenarios::planning_produces_the_documentation
    - cargo_workplan::test_crate::reconcile::a_list_with_obligations_is_printed_one_line_each_and_exits_one
    - cargo_workplan::test_crate::reconcile::an_empty_list_prints_nothing_and_exits_zero
    - cargo_workplan::test_crate::reconcile::fixing_an_obligation_removes_its_line_and_only_its_line
    - cargo_workplan::test_crate::new::the_printed_steps_end_in_a_clean_check_and_skipping_the_stage_step_does_not
    - raf_workplan::test_crate::git::a_blob_is_returned_byte_for_byte
    - raf_workplan::test_crate::git::a_blob_over_one_mebibyte_is_an_error_and_exactly_one_mebibyte_is_read
    - raf_workplan::test_crate::git::a_default_branch_named_like_an_option_is_a_ref_and_never_an_option
    - raf_workplan::test_crate::git::a_directory_is_an_error_and_not_an_absent_file
    - raf_workplan::test_crate::git::a_directory_that_is_no_work_tree_has_no_listing_and_a_broken_index_is_an_error
    - raf_workplan::test_crate::git::a_file_is_read_as_it_was_at_the_merge_base
    - raf_workplan::test_crate::git::a_linked_work_tree_asks_about_its_own_head_and_shares_the_remote_refs
    - raf_workplan::test_crate::git::a_linked_work_tree_is_its_own_top
    - raf_workplan::test_crate::git::a_local_branch_named_origin_main_does_not_stand_in_for_the_remote_one
    - raf_workplan::test_crate::git::a_repository_without_origin_head_has_no_default_branch_and_the_fix_is_named
    - raf_workplan::test_crate::git::histories_with_no_common_ancestor_have_no_merge_base
    - raf_workplan::test_crate::git::the_default_branch_of_a_clone_is_the_full_remote_tracking_name
    - raf_workplan::test_crate::git::the_merge_base_is_where_the_branch_left_the_default_branch
    - raf_workplan::test_crate::git::the_prefix_is_where_the_directory_is_below_the_top_of_its_work_tree
    - raf_workplan::test_crate::git::the_tracked_paths_are_the_index_and_not_the_files_on_disk
    - raf_workplan::test_crate::git::the_tracked_paths_of_a_subdirectory_are_relative_to_it
    - raf_workplan::test_crate::lifecycle::a_file_or_a_link_where_work_changes_belongs_is_refused
    - raf_workplan::test_crate::lifecycle::a_link_at_the_path_is_an_existing_file_and_is_not_followed
    - raf_workplan::test_crate::lifecycle::a_slug_that_reads_as_a_tracker_reference_leaves_no_file
    - raf_workplan::test_crate::lifecycle::an_spdx_field_that_is_not_one_line_of_text_is_refused_and_nothing_is_written
    - raf_workplan::test_crate::lifecycle::impact_of_a_change_that_just_was_created_reads_the_record_from_the_disk
    - raf_workplan::test_crate::lifecycle::impact_of_a_change_that_reaches_nothing_says_none
    - raf_workplan::test_crate::lifecycle::impact_of_the_repositorys_own_change_lists_what_the_record_says
    - raf_workplan::test_crate::lifecycle::impact_tells_an_unknown_slug_from_a_record_that_did_not_load
    - raf_workplan::test_crate::lifecycle::neither_spdx_field_writes_no_comment_line
    - raf_workplan::test_crate::lifecycle::the_bad_slug_message_says_what_a_slug_is
    - raf_workplan::test_crate::lifecycle::the_guidance_for_a_delivered_capability_names_both_findings_it_avoids
    - raf_workplan::test_crate::lifecycle::the_record_beside_a_delivered_capability_is_a_finding_until_the_state_is_set
    - raf_workplan::test_crate::lifecycle::the_sync_note_is_there_when_the_target_file_was_already_stale
    - raf_workplan::test_crate::lifecycle::the_template_in_the_wrong_state_is_the_finding_the_guidance_names
    - raf_workplan::test_crate::lifecycle::work_changes_is_created_when_the_repository_has_none
    - raf_workplan::test_crate::obligations::a_block_of_another_change_in_a_capability_that_does_not_load_is_not_owed
    - raf_workplan::test_crate::obligations::a_capability_over_the_size_bound_is_not_loaded_and_refused
    - raf_workplan::test_crate::obligations::a_delivered_capability_owes_code_and_tests_but_not_examples
    - raf_workplan::test_crate::obligations::a_delivered_capability_that_breaks_another_rule_too_is_not_read_as_unbound
    - raf_workplan::test_crate::obligations::a_hundred_thousand_lines_are_a_hundred_thousand_obligations_in_order
    - raf_workplan::test_crate::obligations::a_listed_capability_that_is_not_loaded_is_an_error_and_not_a_skipped_one
    - raf_workplan::test_crate::obligations::a_very_long_line_is_one_obligation
    - raf_workplan::test_crate::obligations::an_unknown_change_is_not_a_change_that_did_not_load
    - raf_workplan::test_crate::obligations::an_unterminated_fence_hides_the_rest_of_the_section
    - raf_workplan::test_crate::obligations::an_unterminated_fence_in_an_earlier_section_hides_the_headings_after_it
    - raf_workplan::test_crate::obligations::capabilities_that_bind_nothing_owe_no_binding
    - raf_workplan::test_crate::obligations::fixing_one_fault_removes_exactly_its_lines
    - raf_workplan::test_crate::obligations::needed_is_a_typed_value_whose_text_starts_with_the_contract_string
    - raf_workplan::test_crate::obligations::only_the_capabilities_the_change_lists_are_judged
    - raf_workplan::test_crate::obligations::text_that_is_read_as_wording_at_a_hostile_size_owes_nothing
    - raf_workplan::test_crate::obligations::the_baseline_owes_nothing_and_check_agrees
    - raf_workplan::test_crate::obligations::the_change_is_found_with_or_without_its_prefix
    - raf_workplan::test_crate::obligations::the_proposals_are_those_of_the_graph_for_this_change_only
    - raf_workplan::test_crate::obligations::two_blocks_in_one_section_print_in_document_order_and_every_line_is_in_order
    - raf_workplan::test_crate::obligations::wording_is_read_in_every_section_of_an_implementing_capability
    - raf_workplan::test_crate::untouched::a_checkout_with_converted_line_endings_is_untouched_in_every_section
    - raf_workplan::test_crate::untouched::a_default_branch_with_no_common_ancestor_is_an_error
    - raf_workplan::test_crate::untouched::a_prefix_with_a_space_is_an_error_and_a_neighbour_without_one_is_not
    - raf_workplan::test_crate::untouched::a_repository_with_no_remote_at_all_is_an_error_and_not_a_pass
    - raf_workplan::test_crate::untouched::a_root_outside_any_work_tree_is_an_error_at_the_open
    - raf_workplan::test_crate::untouched::a_section_missing_from_the_merge_base_file_counts_as_touched
    - raf_workplan::test_crate::untouched::an_affected_section_the_working_tree_file_lacks_is_an_error
    - raf_workplan::test_crate::untouched::contract_effect_none_still_reports_the_other_sections
    - raf_workplan::test_crate::untouched::line_endings_do_not_touch_a_section
    - raf_workplan::test_crate::untouched::text_in_a_fence_and_in_a_proposed_block_is_authored
    - raf_workplan::test_crate::untouched::the_default_branch_moving_on_after_the_fork_does_not_touch_the_section
    - raf_workplan::lifecycle::changed::tests::a_value_over_a_limit_by_less_than_a_float_tells_apart_is_a_change
    - raf_workplan::lifecycle::changed::tests::an_unchanged_or_better_record_owes_nothing
    - raf_workplan::lifecycle::changed::tests::each_change_is_named
    - raf_workplan::obligation::tests::a_part_is_ordered_document_then_sections_in_document_order_then_items
    - raf_workplan::obligation::tests::a_phrase_continues_over_one_line_break_and_not_over_anything_else
    - raf_workplan::obligation::tests::all_lists_every_kind_once_in_order_with_unique_kebab_names
    - raf_workplan::obligation::tests::an_empty_list_prints_nothing
    - raf_workplan::obligation::tests::an_unterminated_fence_hides_the_rest_and_does_not_panic
    - raf_workplan::obligation::tests::contract_effect_none_suppresses_only_an_untouched_contract
    - raf_workplan::obligation::tests::every_phrase_has_a_sentence_that_finds_it_and_every_hit_is_a_phrase
    - raf_workplan::obligation::tests::generated_blocks_are_not_read
    - raf_workplan::obligation::tests::hostile_text_is_read_in_linear_time
    - raf_workplan::obligation::tests::moves_to_unknown_records_cost_the_same_with_few_records_as_with_many
    - raf_workplan::obligation::tests::near_misses_are_not_hits
    - raf_workplan::obligation::tests::only_prose_is_read_and_the_hit_names_its_section_and_line
    - raf_workplan::obligation::tests::only_the_marker_name_is_exempt_in_capitals
    - raf_workplan::obligation::tests::several_hits_of_a_phrase_on_one_line_are_one_hit_and_two_lines_are_two
    - raf_workplan::obligation::tests::text_before_the_first_section_has_no_section
    - raf_workplan::obligation::tests::the_evidence_names_the_thing_it_is_about_in_parentheses
    - raf_workplan::obligation::tests::the_kinds_match_the_contract
    - raf_workplan::obligation::tests::the_list_is_sorted_by_subject_part_kind_line_then_evidence_and_is_without_repeats
    - raf_workplan::obligation::tests::the_trace_evidence_names_the_record_and_what_is_wrong_with_it
    - raf_workplan::obligation::trace::tests::a_value_over_its_limit_is_found_by_its_text_and_not_its_nearest_float
  examples:
    - raf_workplan::Workplan::new_change
    - raf_workplan::Workplan::impact
    - raf_workplan::Workplan::reconcile
    - raf_workplan::lifecycle::section_digest
depends: [cap:workplan-document-model, cap:workplan-generated-docs, cap:source-index]
---

## Purpose

Turn a change's documented effect into a concrete list of owed document and source-accounting work. Static source declarations and native history make each obligation inspectable without running the consumer behavior it describes.

## Contract

**Static contract.** `change new` exclusively creates one owned change record for explicit known capabilities and affected sections. Slugs, sections and paired SPDX fields are validated before publication. `impact` reports listed capabilities, affected sections, bindings, dependants and linking sections without writing.
<!-- raf:tested-by raf_workplan::test_crate::lifecycle::new_writes_one_file_with_the_keys_the_id_and_a_summary_and_it_loads_clean -->

Reconciliation reports unresolved proposals, missing item outcomes or reasons, missing bindings, stale generated output, unsupported tense, unsupported claims, stale or changed evidence metadata, missing or stale assessments, and missing or stale required reviews. Each obligation retains its typed subject, location and needed evidence. Ordering is deterministic; a refused source or document read remains an error.
<!-- raf:tested-by raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind -->

Historical obligations use native read-only Git. Affected authored sections are compared with the selected merge base after generated blocks and proposal text are removed from the comparison. A section new on the branch is touched. `contract_effect: none` suppresses an untouched Contract obligation, but affected Usage and Examples still require accounting. Changes in bound public source tokens or documentation require a stated contract effect; whitespace and plain comments alone do not. Public surface comparison preserves package, target and impl-member identity.
<!-- raf:tested-by raf_workplan::test_crate::untouched::an_unchanged_affected_section_is_owed_with_the_line_the_contract_shows,raf_workplan::test_crate::complete::prospective_completion_refuses_an_implementing_capability_and_an_undone_item -->

## Usage

The documented data path is illustrated by `raf_workplan::Workplan::new_change` and `raf_workplan::Workplan::impact` and `raf_workplan::Workplan::reconcile`. These examples are source references for a caller to inspect; RAF does not initiate their execution.

The closed obligation vocabulary and the evidence needed to resolve each kind are:

| Kind | Raised when | Evidence that resolves it |
| --- | --- | --- |
| `unresolved-proposal` | A PROPOSED block names the change | The block is replaced by plain contract text, or removed because the claim was dropped |
| `item-without-outcome` | An item has no `outcome`, or its `moved` outcome names no change record or the record itself | An outcome of `implemented`, `dropped` or `moved:change:<slug>` |
| `outcome-without-reason` | Under the complete policy, an item whose outcome is `dropped` or `moved:change:<slug>` has no `reason` | A reason for the outcome |
| `binding-missing` | A capability the change lists binds nothing where it should, or a binding does not resolve or names an item its list does not take | A binding that resolves through the source index |
| `section-untouched` | A section in `affects` has the same text as at the merge base | A diff of that section against the merge base, or an edit that records why none is needed |
| `contract-effect-understated` | Under the complete policy, the record says `contract_effect: none` and a bound public item differs from the merge base | `contract_effect: changes`, the capability listed and the sections it owes reconciled |
| `generated-stale` | A generated block of a listed capability or of the capability index differs from what `docs sync` writes, or the index does not exist | A `docs sync` run |
| `tense-mismatch` | The prose of a listed delivered or implementing capability uses proposal or forward-looking wording, or a listed delivered capability has a PROPOSED block | Text rewritten as the supported behaviour |
| `claim-unsupported` | A claim of a listed capability names an evidence record that does not load, whose verdict is `fail`, or that holds a value over its limit | A passing record within its limits for each evidence id the claim names, or the claim reworded or removed |
| `evidence-stale` | An input of an evidence record whose `for` lists a listed capability, or that a claim of one names, has another digest now than the one the record keeps, or no longer resolves | The authored evidence metadata reviewed and updated against its current inputs |
| `evidence-changed` | An evidence record a claim of a listed capability names passed at the merge base and now fails, holds a value over a limit (its own or the one at the merge base), has lost a limit it had there, or no longer loads | The claim reviewed against the new result: the evidence passes within the limits of the merge base again, or the claim no longer names it |
| `assessment-missing` | An authored section in `affects` has no assessment | An assessment of the section recorded with `assess` |
| `assessment-stale` | An input of the assessment of an authored section in `affects` has another digest now than the one the record keeps, no longer resolves, is no longer an input, or is an input now and was not then | The section assessed again with `assess` |
| `assessment-rejected` | The assessment of an authored section in `affects` is current and its verdict is `rejected` | The section revised and assessed again with `assess`, with the verdict `accepted` |
| `review-missing` | A listed capability requires a review kind but has no completed review of its bound-code scope | A completed current review of the required kind recorded for the capability |
| `review-stale` | A completed required review of a listed capability has changed source inputs | A completed current review of the required kind recorded for the capability |

The typed evidence descriptions agree with this closed obligation table.
<!-- raf:tested-by raf_workplan::obligation::tests::the_evidence_strings_are_the_evidence_column_of_the_contract -->

Create the record with `cargo workplan change new SLUG --capability cap:NAME`. Inspect it with `change impact SLUG` and `change reconcile SLUG`. Reconciliation reads declarations and history; it never runs the named test or example bodies.

## Examples

An unchanged affected Usage section produces `section-untouched` with its capability and section. An unresolved source binding produces `binding-missing`. A consumer example containing a panic contributes no runtime failure obligation because its body is never executed.

## Constraints

- All reconciliation is static. There is no `example-failing` runtime obligation. Native history requires complete locally available objects; missing or malformed data refuse without fetching, helpers or Git-state mutation. Blob reads retain their explicit size limits.
<!-- raf:tested-by raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind -->

## Explanation

Local document rules and historical comparisons are separate data operations. The lifecycle adds native merge-base context to the document model's local obligations. No process runner, executable selection, namespace or stored test receipt is needed.

## Verification

Named obligation fixtures cover every current kind. Native merge-base fixtures cover unchanged and edited sections and refusal paths. Prospective completion enforces the resulting obligations before owned publication.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::test_crate::new::the_printed_steps_end_in_a_clean_check_and_skipping_the_stage_step_does_not` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::a_list_with_obligations_is_printed_one_line_each_and_exits_one` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::an_empty_list_prints_nothing_and_exits_zero` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::an_unlistable_public_surface_is_exit_two_and_the_resolved_neighbor_answers` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::fixing_an_obligation_removes_its_line_and_only_its_line` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::reconcile::unparseable_merge_base_source_is_an_error_beside_a_readable_base` | test | [Verification](#verification) |
| `raf_workplan::Workplan::impact` | doctest | [Usage](#usage) |
| `raf_workplan::Workplan::new_change` | doctest | [Usage](#usage) |
| `raf_workplan::Workplan::reconcile` | doctest | [Usage](#usage) |
| `raf_workplan::lifecycle::changed::tests::a_value_over_a_limit_by_less_than_a_float_tells_apart_is_a_change` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::changed::tests::an_unchanged_or_better_record_owes_nothing` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::changed::tests::each_change_is_named` | test | [Verification](#verification) |
| `raf_workplan::lifecycle::section_digest` | doctest | [Verification](#verification) |
| `raf_workplan::obligation::tests::a_part_is_ordered_document_then_sections_in_document_order_then_items` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::a_phrase_continues_over_one_line_break_and_not_over_anything_else` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::all_lists_every_kind_once_in_order_with_unique_kebab_names` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::an_empty_list_prints_nothing` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::an_unterminated_fence_hides_the_rest_and_does_not_panic` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::contract_effect_none_suppresses_only_an_untouched_contract` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::every_phrase_has_a_sentence_that_finds_it_and_every_hit_is_a_phrase` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::generated_blocks_are_not_read` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::hostile_text_is_read_in_linear_time` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::moves_to_unknown_records_cost_the_same_with_few_records_as_with_many` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::near_misses_are_not_hits` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::only_prose_is_read_and_the_hit_names_its_section_and_line` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::only_the_marker_name_is_exempt_in_capitals` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::only_the_three_history_dependent_kinds_need_more_than_local_documents` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::several_hits_of_a_phrase_on_one_line_are_one_hit_and_two_lines_are_two` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::text_before_the_first_section_has_no_section` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::the_evidence_names_the_thing_it_is_about_in_parentheses` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::the_evidence_strings_are_the_evidence_column_of_the_contract` | test | [Usage](#usage) |
| `raf_workplan::obligation::tests::the_kinds_match_the_contract` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::the_list_is_sorted_by_subject_part_kind_line_then_evidence_and_is_without_repeats` | test | [Verification](#verification) |
| `raf_workplan::obligation::tests::the_trace_evidence_names_the_record_and_what_is_wrong_with_it` | test | [Verification](#verification) |
| `raf_workplan::obligation::trace::tests::a_value_over_its_limit_is_found_by_its_text_and_not_its_nearest_float` | test | [Verification](#verification) |
| `raf_workplan::test_crate::assessments::an_evidence_record_a_claim_of_the_section_names_is_an_input` | test | [Verification](#verification) |
| `raf_workplan::test_crate::assessments::with_no_bound_symbol_named_every_bound_symbol_is_an_input` | test | [Verification](#verification) |
| `raf_workplan::test_crate::complete::prospective_completion_refuses_an_implementing_capability_and_an_undone_item` | test | [Contract](#contract) |
| `raf_workplan::test_crate::git::a_blob_is_returned_byte_for_byte` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_blob_over_one_mebibyte_is_an_error_and_exactly_one_mebibyte_is_read` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_default_branch_named_like_an_option_is_a_ref_and_never_an_option` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_directory_is_an_error_and_not_an_absent_file` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_directory_that_is_no_work_tree_has_no_listing_and_a_broken_index_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_file_is_read_as_it_was_at_the_merge_base` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_linked_work_tree_asks_about_its_own_head_and_shares_the_remote_refs` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_linked_work_tree_is_its_own_top` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_local_branch_named_origin_main_does_not_stand_in_for_the_remote_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::a_repository_without_origin_head_has_no_default_branch_and_the_fix_is_named` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::histories_with_no_common_ancestor_have_no_merge_base` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::the_default_branch_of_a_clone_is_the_full_remote_tracking_name` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::the_merge_base_is_where_the_branch_left_the_default_branch` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::the_prefix_is_where_the_directory_is_below_the_top_of_its_work_tree` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::the_tracked_paths_are_the_index_and_not_the_files_on_disk` | test | [Verification](#verification) |
| `raf_workplan::test_crate::git::the_tracked_paths_of_a_subdirectory_are_relative_to_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::lifecycle::a_bad_slug_is_refused_before_anything_is_written_and_a_good_one_next_to_it_is_not` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::a_capability_that_does_not_exist_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::a_capability_whose_document_did_not_load_is_named_and_never_skipped` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::a_file_or_a_link_where_work_changes_belongs_is_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::a_link_at_the_path_is_an_existing_file_and_is_not_followed` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::a_slug_that_reads_as_a_tracker_reference_leaves_no_file` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::an_affects_entry_must_name_a_listed_capability_and_one_of_the_seven_sections` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::an_existing_file_is_refused_and_left_byte_identical` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::an_spdx_field_that_is_not_one_line_of_text_is_refused_and_nothing_is_written` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::both_spdx_fields_open_the_record_with_the_two_comment_lines_of_the_records_of_the_repository` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::explicit_affects_are_written_as_given_and_impact_reports_them_sorted` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::impact_gives_the_capabilities_sections_bindings_dependants_and_linking_sections` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::impact_of_a_change_that_just_was_created_reads_the_record_from_the_disk` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::impact_of_a_change_that_reaches_nothing_says_none` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::impact_of_the_repositorys_own_change_lists_what_the_record_says` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::impact_prints_each_part_in_a_fixed_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::impact_tells_an_unknown_slug_from_a_record_that_did_not_load` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::neither_spdx_field_writes_no_comment_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::new_writes_one_file_with_the_keys_the_id_and_a_summary_and_it_loads_clean` | test | [Contract](#contract) |
| `raf_workplan::test_crate::lifecycle::one_spdx_field_without_the_other_is_an_error_that_names_the_missing_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::several_capabilities_are_listed_in_the_order_given_without_repeats` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_bad_slug_message_says_what_a_slug_is` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_default_affects_is_the_contract_of_each_listed_capability` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_guidance_follows_the_state_of_each_listed_capability` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_guidance_for_a_delivered_capability_names_both_findings_it_avoids` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_printed_text_holds_the_template_and_the_guidance` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_record_beside_a_delivered_capability_is_a_finding_until_the_state_is_set` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_sync_note_is_there_when_the_target_file_was_already_stale` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_template_in_a_capability_of_each_state_gives_one_block_and_a_clean_check` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::the_template_in_the_wrong_state_is_the_finding_the_guidance_names` | test | [Verification](#verification) |
| `raf_workplan::test_crate::lifecycle::work_changes_is_created_when_the_repository_has_none` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_block_of_another_change_in_a_capability_that_does_not_load_is_not_owed` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_capability_over_the_size_bound_is_not_loaded_and_refused` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_delivered_capability_owes_code_and_tests_but_not_examples` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_delivered_capability_that_breaks_another_rule_too_is_not_read_as_unbound` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_dropped_or_moved_item_needs_a_reason_and_an_explained_outcome_is_accepted` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_hundred_thousand_lines_are_a_hundred_thousand_obligations_in_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_listed_capability_that_is_not_loaded_is_an_error_and_not_a_skipped_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::a_very_long_line_is_one_obligation` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::an_unknown_change_is_not_a_change_that_did_not_load` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::an_unterminated_fence_hides_the_rest_of_the_section` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::an_unterminated_fence_in_an_earlier_section_hides_the_headings_after_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::capabilities_that_bind_nothing_owe_no_binding` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::contract_effect_none_keeps_every_obligation_the_documents_can_show` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::contract_effect_none_keeps_the_assessments_and_another_section_is_not_judged` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::each_assessment_fault_is_owed_at_the_section_naming_the_input_and_a_current_acceptance_clears_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::each_fault_is_owed_with_its_kind_location_and_evidence_and_fixing_it_removes_the_line` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::equal_inputs_print_equal_output` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::every_document_local_kind_is_raised_and_history_kinds_are_not` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::fixing_one_fault_removes_exactly_its_lines` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::needed_is_a_typed_value_whose_text_starts_with_the_contract_string` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::only_the_capabilities_the_change_lists_are_judged` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::reconcile_writes_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::text_of_a_proposed_block_is_not_read_for_wording` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::text_that_is_read_as_wording_at_a_hostile_size_owes_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::the_baseline_owes_nothing_and_check_agrees` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::the_binding_obligations_are_the_gaps_of_the_roadmap` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::the_change_is_found_with_or_without_its_prefix` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::the_proposals_are_those_of_the_graph_for_this_change_only` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::two_blocks_in_one_section_print_in_document_order_and_every_line_is_in_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::wording_is_read_in_every_section_of_an_implementing_capability` | test | [Verification](#verification) |
| `raf_workplan::test_crate::obligations::wording_that_only_looks_like_a_phrase_owes_nothing` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::planning_produces_the_documentation` | test | [Verification](#verification) |
| `raf_workplan::test_crate::scenarios::several_redesigns_leave_one_readable_present` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_checkout_with_converted_line_endings_is_untouched_in_every_section` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_default_branch_named_like_an_option_is_resolved_and_never_reaches_git_as_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_default_branch_with_no_common_ancestor_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_file_absent_at_the_merge_base_counts_as_touched` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_file_at_the_merge_base_that_does_not_parse_is_an_error_and_not_a_pass` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_file_over_one_mebibyte_at_the_merge_base_is_an_error_and_exactly_one_is_read` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_linked_work_tree_compares_its_own_head` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_local_branch_named_origin_main_does_not_stand_in_for_the_remote_one` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_prefix_with_a_space_is_an_error_and_a_neighbour_without_one_is_not` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_repository_with_no_remote_at_all_is_an_error_and_not_a_pass` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_root_outside_any_work_tree_is_an_error_at_the_open` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_section_edited_on_the_branch_is_not_reported_and_its_neighbour_is` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_section_missing_from_the_merge_base_file_counts_as_touched` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::a_workspace_below_the_top_of_the_repository_is_compared_at_its_prefix` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::an_affected_section_the_working_tree_file_lacks_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::an_edit_that_is_not_committed_counts_because_the_working_tree_is_compared` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::an_unchanged_affected_section_is_owed_with_the_line_the_contract_shows` | test | [Contract](#contract) |
| `raf_workplan::test_crate::untouched::contract_effect_none_keeps_usage_and_examples_and_requires_a_default_branch` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::contract_effect_none_still_reports_the_other_sections` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::line_endings_do_not_touch_a_section` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::no_default_branch_is_an_error_that_names_the_fix_and_the_fix_clears_it` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::on_the_default_branch_itself_every_affected_section_is_owed_and_it_is_not_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::only_a_generated_block_changed_is_still_reported` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::reconcile_writes_nothing_and_prints_the_same_list_twice` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::text_in_a_fence_and_in_a_proposed_block_is_authored` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::the_default_branch_moving_on_after_the_fork_does_not_touch_the_section` | test | [Verification](#verification) |
| `raf_workplan::test_crate::untouched::the_digest_leaves_out_generated_blocks_on_both_sides_and_nothing_else` | test | [Verification](#verification) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [source-index](./source-index.md) |
| Depends on | [workplan-document-model](./workplan-document-model.md) |
| Depends on | [workplan-generated-docs](./workplan-generated-docs.md) |
| Depended on by | [workplan-complete-gate](./workplan-complete-gate.md) |
| Depended on by | [workplan-strict](./workplan-strict.md) |
<!-- raf:generated:end -->
