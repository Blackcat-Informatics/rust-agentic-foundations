# T3 ported map: docs_contract tests and the raf-workplan fixtures that replace them

All raf-workplan paths are under `crates/raf-workplan/tests/`. A kept name means the test has the same name there.
"Task 7" means the docs_contract test is removed in Task 7 once the thin caller is in place.

## Links (links.rs, same names)
rejects_broken_relative_link, rejects_broken_anchor, rejects_link_to_a_tracked_symlink, rejects_link_leaving_the_repository,
anchors_follow_github_rules, rejects_broken_reference_definition, rejects_broken_href_and_src_attributes,
rejects_broken_link_outside_the_document_directories, accepts_resolving_reference_and_attribute_links,
rejects_a_link_whose_target_cannot_be_read_for_its_anchor, rejects_an_empty_link_target,
rejects_absolute_and_empty_link_targets_beside_relative_ones, rejects_broken_unquoted_attribute_values,
reads_attributes_with_spaces_around_the_equals_sign, does_not_read_attributes_that_only_end_in_href_or_src,
rejects_broken_attributes_in_tags_split_across_lines, tag_scan_spares_prose_autolinks_and_closed_tags,
tag_state_does_not_survive_a_blank_line_or_a_fence, reads_an_attribute_value_that_starts_on_the_next_line
-> replaced (links.rs, same name).

New beside them (no docs_contract original): autolinks_are_external_and_not_judged, a_link_to_a_directory_resolves_and_its_anchor_is_not_read,
an_anchor_in_a_non_markdown_file_is_not_judged, the_oracle_is_the_tracked_set_and_not_the_tree,
a_symlink_the_caller_names_is_tracked_and_still_a_symlink, a_document_is_judged_against_the_supplied_tracked_set,
a_directory_the_walk_cannot_list_is_reported_and_not_skipped, the_tree_is_walked_only_when_a_relative_link_needs_it,
links_inside_documents_are_judged_wherever_the_document_is_valid_or_not, a_fragment_link_is_judged_against_its_own_document.

KEPT in docs_contract (G4): rejects_a_tracked_symlink_that_nothing_links_to (the tracked-symlink rule; its disk fixture arrives in Task 7).

## Plain Markdown outside the document folders (plain.rs, same names)
rejects_a_link_split_across_lines_in_any_tracked_markdown_file, rejects_a_link_split_across_lines_in_a_profile_document,
accepts_one_line_links_and_links_in_fences_beside_the_split_rule, does_not_read_links_or_placeholders_inside_html_comments,
reads_placeholders_in_plain_markdown_outside_code, reports_marker_errors_of_plain_markdown_once,
reads_a_proposed_block_in_plain_markdown_as_outside_a_capability,
reads_markdown_outside_the_document_directories_with_or_without_frontmatter, rejects_unclosed_frontmatter_in_plain_markdown,
rejects_generated_marker_errors -> replaced (plain.rs, same name; the document variant of the marker errors runs through `check`).

## PROPOSED blocks and states (states.rs, same names)
rejects_orphan_proposed_marker (kind proposed-missing-change), rejects_considered_without_proposed_block
(state-considered-no-proposal), rejects_open_capability_without_change_record (no-open-record),
rejects_delivered_with_proposed_block (state-delivered-proposal), rejects_delivered_with_open_change_record
(state-delivered-open-record), rejects_designed_with_proposed_block (state-designed-proposal),
rejects_proposed_section_missing_from_affects (proposed-section-not-affected),
rejects_proposed_change_that_does_not_list_the_capability (proposed-change-not-listing) -> replaced.
Named fixtures for every kind: support/cases.rs `state_cases`, `state_rule_cases`, `link_cases`.

## Surfaced from raf-okf, not re-implemented (covered by the per-rule fixtures in rules.rs / parse_errors.rs and states.rs)
rejects_proposed_block_outside_a_section (okf-proposed-outside-section), rejects_malformed_proposed_marker,
rejects_lowercase_proposed_marker, rejects_proposed_marker_without_space, rejects_proposed_marker_with_extra_spacing_or_nesting
(okf-parse-malformed-proposed), rejects_proposed_marker_in_architecture / _in_change (documents.rs, Task 1),
rejects_bindings_before_delivery (okf-binding-not-empty), rejects_delivered_without_code_or_test_binding (okf-binding-required),
rejects_sections_left_empty_by_comments / comments_hide_text_but_not_the_text_beside_them (okf-section-empty; states.rs
surfaces_the_single_document_state_rules_from_the_profile pins the four through `check`).

## Issue references (refs.rs, same names)
rejects_issue_number_token, issue_token_scan_spares_hex_colours, rejects_process_wording, rejects_acceptance_label
-> replaced. The matcher fixtures in tests/repo_gates.rs stay and now run against `raf_workplan::refs`.

## Task 2 carry-over (graph.rs)
a_code_span_that_continues_the_bound_path_with_a_segment_does_not_substantiate, and the hostile_* tests:
duplicate ids across files, self-dependency, an anchor only another document has, duplicated heading anchors,
links in fences and spans, unterminated backticks.

## Losses against the old docs_contract walk, decided in the review pass
- Profile validation of a Markdown file that has frontmatter but sits outside the document directories (for example a
  `docs/other.md` that says `type: Capability`): `plain::check` reads such a file as plain Markdown and does not validate it.
  Not restored: the plan makes the document folders the only place typed documents live, and
  `reads_markdown_outside_the_document_directories_with_or_without_frontmatter` pins the plain reading. LIST FOR THE TASK 7
  DISPOSITION TABLE as a deliberate change.
- PROPOSED blocks of a capability that fails validation: RESTORED. The blocks are collected at load and each one that names a
  change with no record is `proposed-missing-change` (test `the_blocks_of_a_capability_that_does_not_validate_still_name_a_record`).
- `plain::check` never reports `unreadable-directory`; only `Workplan::load` does, for the walk of an `of_tree` context. In
  `plain::check` an unlistable directory makes links into it `link-broken`, which fails closed. LIST FOR THE TASK 7 DISPOSITION
  TABLE; a caller that supplies its tracked list has no walk at all.
- A non-capability `index.md` in `docs/architecture` or `work/changes`: RESTORED. It is read with the plain rules
  (plain.rs `an_index_in_a_document_directory_other_than_the_capability_index_is_read_as_markdown`).
- A broken change record no longer produces `no-open-record` for any capability (a record that failed to load may list it);
  `a_broken_record_does_not_make_the_capabilities_it_lists_unlisted`.
