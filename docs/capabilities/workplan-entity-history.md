---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-entity-history
title: Native entity history
description: Read-only native Git document, source-span and removed-binding history.
state: delivered
bindings:
  code:
    - raf_workplan::identifiers
    - raf_workplan::history
    - raf_workplan::Workplan::history
    - raf_workplan::git
    - raf_workplan::git::document_log
    - raf_workplan::git::symbol_log
    - raf_workplan::git::last_change
    - raf_workplan::git::is_shallow
    - raf_workplan::git::AuthorDate
    - raf_workplan::git::AuthorDate::parse
    - raf_workplan::git::LogEntry
    - raf_workplan::git::LastChange
  tests:
    - raf_workplan::git::log::tests::author_dates_keep_strict_zone_and_day_semantics
    - raf_workplan::git::log::tests::selected_lines_follow_shifts_and_own_edits_without_unrelated_history
    - raf_workplan::test_crate::history::ambiguous_bindings_refuse_and_a_single_symbol_has_native_history
    - raf_workplan::test_crate::history::since_counts_from_the_start_of_the_day_in_utc
    - raf_workplan::test_crate::history::unknown_and_unloaded_ids_refuse_without_changing_the_repository
    - raf_workplan::test_crate::identifiers::a_generic_list_holds_exactly_the_characters_of_the_pattern
    - raf_workplan::test_crate::identifiers::a_refusal_names_the_pattern_and_shows_the_value_safely
    - raf_workplan::test_crate::identifiers::a_tree_path_refuses_every_hostile_value_that_is_not_printable_path_text
    - raf_workplan::test_crate::identifiers::each_newtype_is_built_only_by_its_pattern
    - raf_workplan::test_crate::identifiers::every_pattern_accepts_what_it_states
    - raf_workplan::test_crate::identifiers::every_pattern_refuses_hostile_values
    - raf_workplan::test_crate::identifiers::every_pattern_refuses_what_it_does_not_state
    - raf_workplan::test_crate::identifiers::the_patterns_are_the_ones_the_documents_state
    - raf_workplan::test_crate::history::a_capability_history_holds_its_document_through_a_move_and_its_bound_code
    - raf_workplan::test_crate::history::removed_binding_lookup_tracks_substring_counts_instead_of_semantic_removal
    - raf_workplan::test_crate::history::symbol_history_follows_a_file_rename_but_stops_at_a_move_between_files
    - raf_workplan::test_crate::history::a_nested_workspace_keeps_git_path_bases_beside_diff_relative
    - raf_workplan::test_crate::history::missing_partial_clone_objects_refuse_without_writes_and_a_complete_clone_succeeds
    - raf_workplan::test_crate::history::a_module_binding_lists_the_commits_of_its_files_and_not_of_a_sibling
    - raf_workplan::test_crate::history::a_span_beyond_the_committed_file_names_the_binding_until_committed
    - raf_workplan::test_crate::history::oversized_native_commits_refuse_and_the_exact_object_bound_is_read
    - cargo_workplan::test_crate::history::stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone
    - raf_workplan::test_crate::history::a_merge_whose_resolution_changed_the_document_is_in_its_history_and_a_clean_one_is_not
    - raf_workplan::test_crate::history::a_module_file_that_is_not_committed_names_the_binding_and_the_file_until_it_is
    - raf_workplan::test_crate::history::a_root_commit_that_changed_the_identifier_has_no_parent
    - raf_workplan::test_crate::history::a_shallow_clone_is_refused_with_the_missing_history_named_and_a_full_clone_is_not
    - raf_workplan::test_crate::history::a_subject_with_unit_separators_and_a_hash_is_reported_whole
    - raf_workplan::test_crate::history::each_kind_of_id_has_the_history_of_its_file
    - raf_workplan::test_crate::history::since_limits_the_logs_and_not_the_removal_lookup
    - raf_workplan::test_crate::history::since_selects_by_the_author_date_and_not_by_the_committer_date
    - raf_workplan::test_crate::history::the_table_preserves_typed_entries_sources_and_removed_bindings_in_order
    - raf_workplan::history::render::tests::a_table_cell_cannot_break_the_table_or_reach_the_terminal
    - cargo_workplan::test_crate::history::a_shallow_clone_is_exit_two_and_its_full_clone_answers
    - cargo_workplan::test_crate::history::rdf_preserves_full_history_commits_sources_paths_and_removed_bindings
  examples:
    - raf_workplan::Workplan::history
depends: [cap:workplan-document-model, cap:source-index]
---

## Purpose

Read the commits that shaped an authored document and its bound source without executing Git. Native object access keeps repository history useful to analysis while leaving working files, index and repository administration unchanged.

## Contract

**Document history.** Native tracing follows file renames and records root commits and merge resolutions that differ from every parent. Document and source paths preserve the selected workspace prefix and the repository's relative-path presentation. A subject remains complete even when it contains separators or control characters; terminal rendering escapes them.
<!-- raf:tested-by raf_workplan::test_crate::history::a_capability_history_holds_its_document_through_a_move_and_its_bound_code,raf_workplan::test_crate::history::a_nested_workspace_keeps_git_path_bases_beside_diff_relative -->

**Source history.** Resolved source spans are traced through native line diffs, including detected file renames. A module also contributes the histories of its source files, while siblings remain separate. A file absent at HEAD or a span beginning beyond its committed contents refuses with contextual binding and filename information. Removed-binding lookup compares substring occurrence counts, retaining its explicitly heuristic removal and first-parent identity.
<!-- raf:tested-by raf_workplan::test_crate::history::removed_binding_lookup_tracks_substring_counts_instead_of_semantic_removal,raf_workplan::test_crate::history::symbol_history_follows_a_file_rename_but_stops_at_a_move_between_files,raf_workplan::test_crate::history::a_module_binding_lists_the_commits_of_its_files_and_not_of_a_sibling,raf_workplan::test_crate::history::a_span_beyond_the_committed_file_names_the_binding_until_committed -->

**Local data authority.** History reads use a native read-only repository adapter. Missing partial-clone objects, shallow history, malformed data and exhausted native bounds refuse locally. RAF never fetches objects, invokes hooks, filters or helpers, or changes Git state. Imported history metadata grants no command execution authority.
<!-- raf:tested-by raf_workplan::test_crate::history::missing_partial_clone_objects_refuse_without_writes_and_a_complete_clone_succeeds,raf_workplan::test_crate::history::oversized_native_commits_refuse_and_the_exact_object_bound_is_read -->

**Typed data identifiers.** Public identifier types validate their closed textual
patterns before native lookups or documentation selections. Pattern descriptions,
Git references, commit IDs, relative paths and calendar dates remain data. A
validated name grants no process authority. Invalid values retain bounded,
escaped diagnostics beside accepted neighboring values.
<!-- raf:tested-by raf_workplan::test_crate::identifiers::every_pattern_accepts_what_it_states,raf_workplan::test_crate::identifiers::every_pattern_refuses_hostile_values,raf_workplan::test_crate::identifiers::a_refusal_names_the_pattern_and_shows_the_value_safely -->

## Usage

Use `cargo workplan history cap:NAME` for a capability or another supported typed document ID. An optional `--since YYYY-MM-DD` selects entries by author date at or after midnight UTC. Library callers use `raf_workplan::Workplan::history` with an explicit native `git::Repository`; the operation writes nothing.

## Examples

The `raf_workplan::Workplan::history` API accepts an explicitly opened native repository. Renaming a file preserves earlier document entries. A missing local object refuses beside a complete clone, and neither operation writes working files or Git administration. A moved source symbol retains span history without substituting a whole-file log.

## Constraints

- This is repository history, not proof that a test or compiler ran. Native object and complete-result bounds refuse oversized data without partial results. Commit messages require UTF-8; unsupported declared encodings refuse rather than being transcoded.
<!-- raf:tested-by raf_workplan::test_crate::history::oversized_native_commits_refuse_and_the_exact_object_bound_is_read,raf_workplan::test_crate::history::missing_partial_clone_objects_refuse_without_writes_and_a_complete_clone_succeeds -->

## Explanation

Document, span and substring-count queries preserve different meanings. Native line projection and rename matching retain the useful history behavior previously supplied through Git commands, while the analysis boundary removes executable discovery, helper protocols and implicit fetching entirely.

## Verification

The named native fixtures cover rename and merge history, nested workspaces, source-span boundaries, substring-count removal and complete local-object refusal. They compare repository bytes around missing-object operations and pair bounded-object refusals with valid neighboring data.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::test_crate::history::a_shallow_clone_is_exit_two_and_its_full_clone_answers` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::rdf_preserves_full_history_commits_sources_paths_and_removed_bindings` | test | [Verification](#verification) |
| `cargo_workplan::test_crate::history::stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone` | test | [Verification](#verification) |
| `raf_workplan::Workplan::history` | doctest | [Usage](#usage), [Examples](#examples) |
| `raf_workplan::git::log::tests::author_dates_keep_strict_zone_and_day_semantics` | test | [Verification](#verification) |
| `raf_workplan::git::log::tests::selected_lines_follow_shifts_and_own_edits_without_unrelated_history` | test | [Verification](#verification) |
| `raf_workplan::history::render::tests::a_table_cell_cannot_break_the_table_or_reach_the_terminal` | test | [Verification](#verification) |
| `raf_workplan::test_crate::history::a_capability_history_holds_its_document_through_a_move_and_its_bound_code` | test | [Contract](#contract) |
| `raf_workplan::test_crate::history::a_merge_whose_resolution_changed_the_document_is_in_its_history_and_a_clean_one_is_not` | test | [Verification](#verification) |
| `raf_workplan::test_crate::history::a_module_binding_lists_the_commits_of_its_files_and_not_of_a_sibling` | test | [Contract](#contract) |
| `raf_workplan::test_crate::history::a_module_file_that_is_not_committed_names_the_binding_and_the_file_until_it_is` | test | [Verification](#verification) |
| `raf_workplan::test_crate::history::a_nested_workspace_keeps_git_path_bases_beside_diff_relative` | test | [Contract](#contract) |
| `raf_workplan::test_crate::history::a_root_commit_that_changed_the_identifier_has_no_parent` | test | [Verification](#verification) |
| `raf_workplan::test_crate::history::a_shallow_clone_is_refused_with_the_missing_history_named_and_a_full_clone_is_not` | test | [Verification](#verification) |
| `raf_workplan::test_crate::history::a_span_beyond_the_committed_file_names_the_binding_until_committed` | test | [Contract](#contract) |
| `raf_workplan::test_crate::history::a_subject_with_unit_separators_and_a_hash_is_reported_whole` | test | [Verification](#verification) |
| `raf_workplan::test_crate::history::ambiguous_bindings_refuse_and_a_single_symbol_has_native_history` | test | [Verification](#verification) |
| `raf_workplan::test_crate::history::each_kind_of_id_has_the_history_of_its_file` | test | [Verification](#verification) |
| `raf_workplan::test_crate::history::missing_partial_clone_objects_refuse_without_writes_and_a_complete_clone_succeeds` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::history::oversized_native_commits_refuse_and_the_exact_object_bound_is_read` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::history::removed_binding_lookup_tracks_substring_counts_instead_of_semantic_removal` | test | [Contract](#contract) |
| `raf_workplan::test_crate::history::since_counts_from_the_start_of_the_day_in_utc` | test | [Verification](#verification) |
| `raf_workplan::test_crate::history::since_limits_the_logs_and_not_the_removal_lookup` | test | [Verification](#verification) |
| `raf_workplan::test_crate::history::since_selects_by_the_author_date_and_not_by_the_committer_date` | test | [Verification](#verification) |
| `raf_workplan::test_crate::history::symbol_history_follows_a_file_rename_but_stops_at_a_move_between_files` | test | [Contract](#contract) |
| `raf_workplan::test_crate::history::the_table_preserves_typed_entries_sources_and_removed_bindings_in_order` | test | [Verification](#verification) |
| `raf_workplan::test_crate::history::unknown_and_unloaded_ids_refuse_without_changing_the_repository` | test | [Verification](#verification) |
| `raf_workplan::test_crate::identifiers::a_generic_list_holds_exactly_the_characters_of_the_pattern` | test | [Verification](#verification) |
| `raf_workplan::test_crate::identifiers::a_refusal_names_the_pattern_and_shows_the_value_safely` | test | [Contract](#contract) |
| `raf_workplan::test_crate::identifiers::a_tree_path_refuses_every_hostile_value_that_is_not_printable_path_text` | test | [Verification](#verification) |
| `raf_workplan::test_crate::identifiers::each_newtype_is_built_only_by_its_pattern` | test | [Verification](#verification) |
| `raf_workplan::test_crate::identifiers::every_pattern_accepts_what_it_states` | test | [Contract](#contract) |
| `raf_workplan::test_crate::identifiers::every_pattern_refuses_hostile_values` | test | [Contract](#contract) |
| `raf_workplan::test_crate::identifiers::every_pattern_refuses_what_it_does_not_state` | test | [Verification](#verification) |
| `raf_workplan::test_crate::identifiers::the_patterns_are_the_ones_the_documents_state` | test | [Verification](#verification) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [source-index](./source-index.md) |
| Depends on | [workplan-document-model](./workplan-document-model.md) |
| Depended on by | [workplan-current-rationale](./workplan-current-rationale.md) |
<!-- raf:generated:end -->
