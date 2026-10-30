---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-generated-docs
title: "Generated documentation blocks"
description: "Generated blocks for the verification matrix, relationship table and capability catalogue, and the deterministic sync."
state: delivered
bindings:
  code:
    - raf_workplan::generated
    - raf_workplan::diagram
    - raf_workplan::assessment::Inputs
    - raf_workplan::assessment::symbol_input_now
    - raf_workplan::assessment::currency_from_rows
    - raf_workplan::Workplan::currency_inputs
    - raf_workplan::Workplan::currency_checked
    - raf_workplan::digest::SymbolDigestError::is_query_refusal
    - raf_workplan::Opened::preview_generated
    - raf_workplan::Workplan::sync
    - raf_workplan::Workplan::write
    - raf_workplan::Workplan::write_observed
    - raf_workplan::sync::stage_observed
    - raf_workplan::SyncOptions
    - raf_workplan::SyncReport
    - raf_workplan::SyncError
    - raf_workplan::Workplan::preview
    - raf_workplan::Preview
    - raf_workplan::PreviewError
    - raf_workplan::ChangeLookupError
    - raf_workplan::generated::END_MARKER
  tests:
    - cargo_workplan::test_crate::rdf::standalone_publication_preserves_host_files_and_refuses_changed_owned_bytes
    - cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading
    - raf_workplan::diagram::semantic::identity::tests::persisted_identity_profiles_pin_field_boundaries_and_escaping
    - raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal
    - raf_workplan::test_crate::sync::write_refuses_a_target_behind_a_directory_link_inside_the_root
    - raf_workplan::test_crate::generated::the_synced_base_has_no_finding_at_all
    - raf_workplan::test_crate::generated::a_verification_matrix_lists_each_test_and_example_by_path_with_its_sections
    - raf_workplan::test_crate::generated::rows_are_sorted_by_path_whatever_the_order_of_the_lists
    - raf_workplan::test_crate::generated::a_path_named_in_a_fence_or_a_proposal_is_not_a_section_that_it_substantiates
    - raf_workplan::test_crate::generated::a_tested_by_marker_substantiates_its_section_and_an_unmarked_binding_keeps_its_row
    - raf_workplan::graph::tests::a_comment_outside_a_code_span_is_removed_and_one_inside_a_span_is_kept
    - raf_workplan::test_crate::generated::a_relationship_table_lists_each_relation_sorted_by_slug_with_links
    - raf_workplan::test_crate::generated::a_capability_with_no_relation_has_a_table_of_the_header_only
    - raf_workplan::test_crate::generated::the_catalogue_is_a_table_per_state_in_order_with_gaps_for_implementing_only
    - raf_workplan::test_crate::generated::a_gap_names_an_unresolved_binding_and_an_empty_list
    - raf_workplan::test_crate::generated::groups_with_no_capability_are_omitted_and_each_group_is_sorted_by_slug
    - raf_workplan::test_crate::generated::cell_text_is_escaped_the_same_way_every_time
    - raf_workplan::test_crate::generated::a_matrix_and_a_relationship_table_sit_in_a_section_of_a_capability_document
    - raf_workplan::test_crate::generated::a_catalogue_block_is_valid_only_in_the_capability_index
    - raf_workplan::test_crate::generated::a_matrix_or_a_relationship_table_in_the_index_is_misplaced
    - raf_workplan::test_crate::generated::blocks_in_a_change_record_or_an_architecture_page_are_misplaced_and_never_rewritten
    - raf_workplan::test_crate::generated::a_generated_block_does_not_make_a_section_non_empty
    - raf_workplan::test_crate::generated::a_marker_fixture_for_each_error_nested_unclosed_orphan_end_and_unknown_kind
    - raf_workplan::test_crate::generated::an_edited_removed_or_added_matrix_row_is_stale_and_sync_repairs_it
    - raf_workplan::test_crate::generated::an_edited_removed_or_added_relationship_row_is_stale_and_sync_repairs_it
    - raf_workplan::test_crate::generated::an_edited_removed_or_added_catalogue_row_is_stale_and_sync_repairs_it
    - raf_workplan::test_crate::generated::the_stale_finding_points_at_the_opening_marker_of_the_block
    - raf_workplan::test_crate::generated::text_outside_a_block_is_never_changed_and_a_file_without_blocks_is_untouched
    - raf_workplan::test_crate::generated::a_crlf_document_stays_all_crlf_and_an_lf_document_all_lf
    - raf_workplan::test_crate::generated::a_crlf_frontmatter_over_an_lf_body_gets_lf_blocks_and_stays_clean
    - raf_workplan::test_crate::generated::a_body_that_mixes_line_endings_is_refused_by_sync_and_reported_by_check
    - raf_workplan::test_crate::generated::a_missing_index_is_a_finding_and_sync_creates_it_with_the_spdx_lines
    - raf_workplan::test_crate::generated::the_spdx_text_of_a_new_index_is_required_and_must_fit_one_comment_line
    - raf_workplan::test_crate::generated::an_index_with_text_outside_its_blocks_is_rejected_whatever_else_it_holds
    - raf_workplan::test_crate::generated::the_catalogue_cannot_be_forged_from_a_description_that_looks_like_a_marker
    - raf_workplan::test_crate::generated::the_kinds_the_crate_owns_are_the_three_documented_ones
    - raf_workplan::test_crate::sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal
    - raf_workplan::test_crate::sync::the_report_is_a_pure_function_of_the_documents_and_is_sorted_by_path
    - raf_workplan::test_crate::sync::a_document_that_does_not_load_stops_the_sync_since_the_tables_would_miss_it
    - raf_workplan::test_crate::sync::generated_checks_are_clean_exactly_when_sync_has_nothing_to_write
    - raf_workplan::sync::tests::a_late_rename_failure_preserves_published_files_and_discards_the_tail
    - raf_workplan::test_crate::sync::write_puts_every_edit_on_disk_and_a_second_run_writes_nothing
    - raf_workplan::test_crate::sync::write_refuses_a_target_that_is_not_an_existing_regular_file_or_the_new_index
    - raf_workplan::test_crate::sync::write_changes_no_target_when_a_later_file_cannot_be_written
    - raf_workplan::test_crate::sync::the_docs_sync_example_writes_the_expected_files_once_and_nothing_on_a_second_run
    - raf_workplan::test_crate::sync::the_roadmap_rows_equal_the_catalogue_rows
    - raf_workplan::test_crate::sync::the_catalogue_of_this_repository_equals_its_roadmap
    - raf_workplan::test_crate::preview::a_change_with_two_blocks_in_a_capability_reads_reconciled_and_other_blocks_stay
    - raf_workplan::test_crate::preview::a_crlf_document_keeps_its_line_endings
    - raf_workplan::test_crate::preview::a_tab_after_the_quote_marker_is_kept_as_written
    - raf_workplan::test_crate::preview::a_fenced_code_block_in_a_block_of_the_change_is_refused_with_its_line
    - raf_workplan::test_crate::preview::a_continuation_that_is_not_indented_as_the_marker_is_refused_with_its_line
    - raf_workplan::test_crate::preview::an_unknown_slug_names_it_and_lists_the_open_changes
    - raf_workplan::test_crate::preview::a_disk_fixture_is_byte_identical_afterwards
    - raf_workplan::test_crate::generated::a_backtick_in_a_binding_path_does_not_break_the_code_span
    - raf_workplan::test_crate::generated::a_block_in_a_section_other_than_verification_converges_in_one_sync
    - raf_workplan::test_crate::generated::a_relation_listed_twice_is_one_row
    - raf_workplan::test_crate::generated::a_repository_without_capabilities_needs_no_index
    - raf_workplan::test_crate::generated::an_example_that_does_not_resolve_is_listed_as_an_example_target
    - raf_workplan::test_crate::generated::an_existing_index_keeps_its_own_header_and_a_stale_catalogue_is_rewritten_in_place
    - raf_workplan::test_crate::generated::an_index_that_is_only_a_licence_header_and_blank_lines_is_not_a_generated_index
    - raf_workplan::test_crate::generated::an_lf_frontmatter_over_a_crlf_body_gets_crlf_blocks
    - raf_workplan::test_crate::generated::the_frontmatter_of_a_rewritten_document_is_kept_byte_for_byte
    - raf_workplan::test_crate::generated::the_sync_of_a_crlf_document_is_idempotent_and_stale_in_crlf_is_repaired_in_crlf
    - raf_workplan::test_crate::preview::a_block_at_the_end_of_a_file_without_a_final_break_is_reconciled
    - raf_workplan::test_crate::preview::a_block_in_a_list_item_keeps_its_indentation
    - raf_workplan::test_crate::preview::a_block_with_no_content_lines_disappears_with_its_marker
    - raf_workplan::test_crate::preview::a_change_is_found_by_its_slug_with_or_without_the_prefix
    - raf_workplan::test_crate::preview::a_change_record_that_did_not_load_says_why
    - raf_workplan::test_crate::preview::a_fenced_code_block_in_a_block_of_another_change_is_no_obstacle
    - raf_workplan::test_crate::preview::a_listed_capability_that_did_not_load_is_named_never_skipped
    - raf_workplan::test_crate::preview::a_listed_capability_without_a_document_is_an_error
    - raf_workplan::test_crate::preview::a_nested_quote_loses_one_level_only
    - raf_workplan::test_crate::preview::adjacent_blocks_of_two_changes_are_told_apart
    - raf_workplan::test_crate::preview::an_unknown_slug_is_told_apart_from_a_record_that_did_not_load
    - raf_workplan::test_crate::preview::another_change_reconciles_only_its_own_blocks
    - raf_workplan::test_crate::preview::preview_is_deterministic
    - raf_workplan::test_crate::preview::the_lookup_and_the_preview_say_the_same_thing_about_a_slug
    - raf_workplan::test_crate::preview::the_slug_is_taken_with_or_without_the_prefix
    - raf_workplan::test_crate::preview::the_text_outside_blocks_is_unchanged_byte_for_byte
    - raf_workplan::test_crate::sync::check_and_sync_agree_after_each_repair
    - raf_workplan::test_crate::sync::nothing_is_in_the_report_for_a_repository_that_is_in_sync
    - raf_workplan::test_crate::sync::the_docs_sync_example_asks_for_the_spdx_flags_only_when_the_index_must_be_created
    - raf_workplan::test_crate::sync::the_docs_sync_example_in_a_git_checkout_and_with_bad_arguments
    - raf_workplan::test_crate::sync::the_docs_sync_example_opens_the_repository_as_the_gate_does
    - raf_workplan::test_crate::sync::write_creates_the_index_only_where_the_directory_exists
    - raf_workplan::test_crate::sync::write_keeps_the_permissions_of_the_file_it_replaces
    - raf_workplan::test_crate::sync::write_never_touches_a_file_that_is_already_there_under_a_temporary_name
    - raf_workplan::generated::tests::a_code_span_fence_is_longer_than_any_backtick_run_inside
    - raf_workplan::generated::tests::a_state_heading_has_a_capital
    - raf_workplan::generated::tests::cell_text_is_escaped_by_the_documented_rule
    - raf_workplan::generated::tests::the_line_ending_is_the_first_break_and_a_change_is_mixed
  examples:
    - raf_workplan::example_crate::docs_sync
depends: [cap:workplan-document-model]
---

# Generated documentation blocks

## Purpose

Some documentation is mechanical: tables that restate bindings, links between
capabilities, and the catalogue of what exists and in what state. A tool produces
those deterministically so that they are always correct, and people spend their time
on the prose that needs judgement.

## Contract

Named diagrams use one native RDF 1.2 selection and one native visualization
export for SVG, captions, readable accessibility text and structured JSON. The
same cached bundles serve the library, command, check, preview and sync adapters.
Automatic families cover Cargo dependency declarations and separately identified
checked lock facts, capability dependencies, resolved implementation/test/example
ownership, evidence and assessment coverage, and declared change impact. A
selected configuration is labeled; unevaluated feature and target conditions are
not resolved runtime interactions. Current and stale evidence remain distinct.
Document identities reuse the checked document constructor; source input nodes
retain their binding digest, authority and target/definition provenance. Package
selection nodes are export scoped, not a complete repository identity.

Persisted selection, locked-package and relationship identities hash versioned
native JSON records with fixed field order, explicit nulls and typed enum labels.
Relationship fields retain their ordered values, including repeated field names;
native JSON escaping preserves field boundaries. Source and declared configuration
metadata use the same explicit typed projections.
<!-- raf:tested-by raf_workplan::diagram::semantic::identity::tests::persisted_identity_profiles_pin_field_boundaries_and_escaping -->

Focus and closed relationship filters retain only the selected entities and
relationships. Supported package or capability grouping retains original member
identities and provenance, with linked scoped detail bundles. Unsupported
relationships or grouping mappings and exceeded native limits refuse without
truncation. Evidence-claim views retain their exact native unasserted claim
selection and refuse additional grouping or filtering.

A declared view has a matching `diagram-<name>` block in its capability or
architecture document. The block links its SVG, text, RDF and JSON assets and
linked details. Assets use bounded paths derived from checked owner identity and
view name. A closed native RDF ownership manifest admits writes and removals;
unowned collisions refuse and unrelated neighboring files remain unchanged.
Declared views and their generated details share an owner namespace; duplicate
generated asset paths refuse before synchronization.
The writer uses the ordinary checked staging/rename path, an ownership intent
before publication and the final manifest after successful retirement. A partial
publication remains recoverable through the intent; it is not a transaction.
<!-- raf:tested-by raf_workplan::test_crate::sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal,raf_workplan::test_crate::sync::the_report_is_a_pure_function_of_the_documents_and_is_sorted_by_path,raf_workplan::test_crate::sync::nothing_is_in_the_report_for_a_repository_that_is_in_sync -->


One cached named bundle supplies Markdown SVG and readable alternatives,
standard mdBook transformation and a self-contained explorer. All presentations
retain native identities, geometry and provenance. Owned versioned output
preserves unrelated files and cumulative documentation.
<!-- raf:tested-by cargo_workplan::test_crate::rdf::standalone_publication_preserves_host_files_and_refuses_changed_owned_bytes,cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading -->

Completed execution views use this same checked publication writer for their
generated edits. Its private observed result retains the permissions read from
the actual staging handle. Existing replacements preserve permissions and break
hardlink aliases by staging and rename; a missing generated catalogue uses
observed native creation permissions. The public `write` result remains unchanged.
<!-- raf:tested-by raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal -->

### Generated block syntax
<!-- raf:tested-by raf_workplan::test_crate::generated::a_marker_fixture_for_each_error_nested_unclosed_orphan_end_and_unknown_kind -->

A generated block is delimited as follows. The kind is lowercase
kebab case. Each block is opened once, closed once and never nested.

```markdown
<!-- raf:generated:verification-matrix -->
...
<!-- raf:generated:end -->
```

### Generated kinds
<!-- raf:tested-by raf_workplan::test_crate::generated::a_marker_fixture_for_each_error_nested_unclosed_orphan_end_and_unknown_kind,raf_workplan::test_crate::generated::the_synced_base_has_no_finding_at_all -->

`raf-workplan` owns three fixed kinds and the declared `diagram-<name>` family:

### Verification matrix rows
<!-- raf:tested-by raf_workplan::test_crate::generated::a_verification_matrix_lists_each_test_and_example_by_path_with_its_sections,raf_workplan::test_crate::generated::rows_are_sorted_by_path_whatever_the_order_of_the_lists,raf_workplan::test_crate::generated::a_tested_by_marker_substantiates_its_section_and_an_unmarked_binding_keeps_its_row -->

`verification-matrix`: in a section of a capability document, a table with the
columns Item, Kind and Section and one row per binding in `bindings.tests` (kind
`test`) and `bindings.examples` (kind `example target` or `doctest`), sorted by
path. A binding substantiates each section other than Verification whose text,
outside fenced code, PROPOSED blocks and HTML comments, holds the bound path whole
in a code span, and each section other than Verification that holds a
`raf:tested-by` marker naming its path, outside fenced code and PROPOSED blocks, as
[complete policy](./workplan-strict.md) defines the marker, so a test that a Contract
claim names shows `[Contract](#contract)` in the Section cell; with no such
section it substantiates Verification. The Section cell links each of those
sections in document order. A document with no marker has the matrix that the
code-span rule alone gives.

### Relationship table rows
<!-- raf:tested-by raf_workplan::test_crate::generated::a_relationship_table_lists_each_relation_sorted_by_slug_with_links,raf_workplan::test_crate::generated::a_capability_with_no_relation_has_a_table_of_the_header_only -->

`relationships`: in a section of a capability document, a table with the columns
Relation and Capability and the rows Depends on, Depended on by and Changed by
(the open changes that list it), in that order, each capability or change a link
and sorted by slug. A relation with nothing is omitted, so a capability with no
relation has the header only.

### Catalogue states and gaps
<!-- raf:tested-by raf_workplan::test_crate::generated::the_catalogue_is_a_table_per_state_in_order_with_gaps_for_implementing_only,raf_workplan::test_crate::generated::a_gap_names_an_unresolved_binding_and_an_empty_list,raf_workplan::test_crate::generated::groups_with_no_capability_are_omitted_and_each_group_is_sorted_by_slug -->

`catalogue`: in `docs/capabilities/index.md`, the catalogue and roadmap. It is one
`## <State>` table per state, in the order `delivered`, `implementing`, `designed`,
`considered`, with empty groups omitted and each group sorted by slug. A row links
to the document and carries its description, and an `implementing` row shows the
gaps: binding lists that are empty and bindings that do not resolve. The rows are
those of `Workplan::status`.

### Placement and escaped cells
<!-- raf:tested-by raf_workplan::test_crate::generated::a_matrix_and_a_relationship_table_sit_in_a_section_of_a_capability_document,raf_workplan::test_crate::generated::a_catalogue_block_is_valid_only_in_the_capability_index,raf_workplan::test_crate::generated::a_matrix_or_a_relationship_table_in_the_index_is_misplaced,raf_workplan::test_crate::generated::blocks_in_a_change_record_or_an_architecture_page_are_misplaced_and_never_rewritten,raf_workplan::test_crate::generated::cell_text_is_escaped_the_same_way_every_time -->

A block of a kind in the wrong place, such as a catalogue in a capability document or
any block in a change record, is an error and is never
rewritten. Text from a document written into a table cell is escaped by one rule:
`\` becomes `\\`, `|` becomes `\|`, `[` and `]` become `\[` and `\]`, `<` becomes
`&lt;`, and each line break becomes one space, so a description can never open a
comment or forge a marker.

### Index creation and refusal
<!-- raf:tested-by raf_workplan::test_crate::generated::a_missing_index_is_a_finding_and_sync_creates_it_with_the_spdx_lines,raf_workplan::test_crate::generated::the_spdx_text_of_a_new_index_is_required_and_must_fit_one_comment_line,raf_workplan::test_crate::generated::an_index_with_text_outside_its_blocks_is_rejected_whatever_else_it_holds -->

`docs/capabilities/index.md` is the only index. It has no frontmatter
and carries only its SPDX comment and generated blocks written by the sync operation
(`Workplan::sync`); a hand-written index is not allowed. A missing index is an error
when there is a capability, and sync creates it with the two SPDX comment lines of
`SyncOptions`, a blank line and the `catalogue` block. Sync refuses to create it when
the SPDX text is unset, empty, longer than one line or holds `-->`.

### Pure sync report and stable output
<!-- raf:tested-by raf_workplan::test_crate::sync::the_report_is_a_pure_function_of_the_documents_and_is_sorted_by_path,raf_workplan::test_crate::sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal,raf_workplan::test_crate::generated::text_outside_a_block_is_never_changed_and_a_file_without_blocks_is_untouched,raf_workplan::test_crate::generated::the_frontmatter_of_a_rewritten_document_is_kept_byte_for_byte,raf_workplan::test_crate::generated::a_crlf_document_stays_all_crlf_and_an_lf_document_all_lf -->

The sync operation (`Workplan::sync`) computes every generated block in the
repository from the current documents and source index, and returns the whole new
text of every stale or missing file as a `SyncReport`; it writes nothing.
`Workplan::write` puts a report on disk under a root. The output is a pure function
of its inputs: the same repository always yields the same bytes, in the document's
line ending, with no timestamps and stable ordering. A block takes the line ending of
the body it sits in, read from the body's first line break, and a new index is
written with LF. A file with no generated block is not touched, and text outside a
block is never changed. A block of an unknown kind is an error. A document whose
body mixes line endings is an error, and sync refuses it.

### Unloaded documents stop synchronization
<!-- raf:tested-by raf_workplan::test_crate::sync::a_document_that_does_not_load_stops_the_sync_since_the_tables_would_miss_it -->

Sync also stops while a capability, change or architecture document does not load, since the
tables that list it would be wrong.

### Generated staleness and repair
<!-- raf:tested-by raf_workplan::test_crate::generated::the_stale_finding_points_at_the_opening_marker_of_the_block,raf_workplan::test_crate::sync::generated_checks_are_clean_exactly_when_sync_has_nothing_to_write,raf_workplan::test_crate::sync::check_and_sync_agree_after_each_repair -->

A block whose content differs from what the sync operation would
write is stale. `check` fails on a stale block and names the file, the kind and the
line of its opening marker. A sync clears that staleness finding. Both read the same
renderers, so generated-block checks are clean exactly when sync has nothing to
write. Other document or binding findings can remain after synchronization.

### Preview selection and preserved text
<!-- raf:tested-by raf_workplan::test_crate::preview::a_change_with_two_blocks_in_a_capability_reads_reconciled_and_other_blocks_stay,raf_workplan::test_crate::preview::another_change_reconciles_only_its_own_blocks,raf_workplan::test_crate::preview::the_slug_is_taken_with_or_without_the_prefix,raf_workplan::test_crate::preview::preview_is_deterministic,raf_workplan::test_crate::preview::a_crlf_document_keeps_its_line_endings,raf_workplan::test_crate::preview::a_block_in_a_list_item_keeps_its_indentation,raf_workplan::test_crate::preview::a_tab_after_the_quote_marker_is_kept_as_written,raf_workplan::test_crate::preview::a_nested_quote_loses_one_level_only,raf_workplan::test_crate::preview::an_unknown_slug_names_it_and_lists_the_open_changes,raf_workplan::test_crate::preview::a_change_record_that_did_not_load_says_why,raf_workplan::test_crate::preview::a_listed_capability_that_did_not_load_is_named_never_skipped -->

`Workplan::preview` takes a change slug, with or without `change:`, and
returns each capability document the change lists, sorted by id, as it reads once
that change is reconciled. Each PROPOSED block of the change is replaced by its
content: the marker line goes, and so does the quote marker of each following line,
`>` and one space after it, after the block's own indentation, which stays. A tab
after `>` is kept as written. The blocks of other changes and all text outside blocks
are kept byte for byte, and every line keeps its own line ending. An unknown slug is
an error that lists the open changes, and a record or a listed capability that did
not load is an error with its findings.

### Authored text preservation
<!-- raf:tested-by raf_workplan::test_crate::generated::text_outside_a_block_is_never_changed_and_a_file_without_blocks_is_untouched -->

Generation covers verification matrices, relationship tables, the capability
catalogue and declared diagram blocks.
Explanation and instructional prose are written by a person or an agent and are never
generated. The generator does not write into the seven sections outside a block.

## Usage

Call `named_diagram` on the loaded Workplan for `cap:<slug>#<name>` or
`arch:<slug>#<name>`, or call `diagram` with an explicit typed selection.
`sync` returns generated text plus opaque asset admission; `removals` enumerates
retired owned paths. Publish that report with `write`. Supply the existing SPDX
options when the ownership manifest is first created. `preview_generated` on
Opened computes reconciled diagram blocks and asset edits/removals without writes.
<!-- raf:tested-by raf_workplan::test_crate::sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal,raf_workplan::test_crate::sync::nothing_is_in_the_report_for_a_repository_that_is_in_sync -->


Configure mdBook to invoke `cargo workplan docs mdbook`; standalone references
use `{{#workplan-diagram cap:api#workspace}}`. Publish the explorer and static
artifacts through `cargo workplan docs build standalone --view cap:api#workspace
--output-dir <existing-directory>`.
<!-- raf:tested-by cargo_workplan::test_crate::rdf::standalone_publication_preserves_host_files_and_refuses_changed_owned_bytes,cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading -->

Callers publish a `SyncReport` through the checked writer shown below. Completion consumes
the same writer internally and validates its resulting copy before running bindings.

Run `raf_workplan::example_crate::docs_sync` to inspect or rewrite the generated documentation blocks through the production sync API.

A repository syncs from a program or from its build:

```rust
use std::path::Path;

use raf_source::{DiskTree, Workspace};
use raf_workplan::{SyncOptions, Workplan};

let root = Path::new(".");
let tree = DiskTree::new(root)?;
let source = Workspace::load(&tree, &["."])?;
let plan = Workplan::load(&tree, &source);
let report = plan.sync(&SyncOptions::new("2026 Example Inc.", "CC-BY-4.0"))?;
Workplan::write(&report, root)?;
for preview in plan.preview("incremental-parsing")? {
    println!("{}\n{}", preview.path, preview.text);
}
```

The `docs_sync` example of `raf-workplan` wraps the same calls. It prints each file it
rewrote; with `--check` it writes nothing, prints each stale file and exits 1 when
there is one. `--copyright` and `--licence` give the SPDX text of a new index and are
read only when the index must be created.

```sh
make docs-sync                                              # rewrite every stale block
cargo run -p raf-workplan --example docs_sync -- --check   # exit 1 when a block is stale
```

A capability document that wants a verification matrix includes an empty block of
that kind in its Verification section. The first sync fills it. A command line is
specified separately ([command line](./workplan-cli.md)) and is not part of this
capability.

## Examples

A generic multi-crate workspace distinguishes normal, development and build
dependencies, requested features and target conditions. Capability and ownership
views expose checked document links and binding inputs; coverage preserves
record currency. A grouped dependency view links detail views. Repeated rendering
is byte identical; a second synchronization is empty. Removing the declaration
and its block retires only the manifest-owned assets, preserving a neighboring
authored SVG. A forged report or unowned destination collision refuses.
<!-- raf:tested-by raf_workplan::test_crate::sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal,raf_workplan::test_crate::sync::the_report_is_a_pure_function_of_the_documents_and_is_sorted_by_path,raf_workplan::test_crate::sync::nothing_is_in_the_report_for_a_repository_that_is_in_sync -->


A multi-crate workspace uses the same declarations for a nested book chapter
and the explorer under a non-root URL prefix.
Native Linux browser interaction covers keyboard navigation, long labels,
relationship filtering and explicit overview/detail links.
<!-- raf:tested-by cargo_workplan::test_crate::rdf::standalone_publication_preserves_host_files_and_refuses_changed_owned_bytes,cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading -->

If a capability document has a hardlink alias, rewriting its generated table
replaces only the named target. A completed execution view retains the alias's
original bytes and the target's rewritten bytes, matching ordinary publication.

The relationship table below links the current dependencies and dependants to their capability documents. Sync computes those rows from the loaded graph and replaces only the generated block:

```markdown
<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-document-model](./workplan-document-model.md) |
| Depended on by | [workplan-reconcile](./workplan-reconcile.md) |
<!-- raf:generated:end -->
```

## Constraints

The mdBook adapter admits at most 4096 chapters, 64 chapter levels and 256
diagram references. A chapter admits 1 MiB of source; aggregate source admits
16 MiB and transformed chapter content admits 64 MiB. Markdown nesting is
limited to 64 containers. Unsupported renderers, malformed references and
unavailable named views refuse before returning transformed content.
<!-- raf:tested-by cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading -->

At most 64 views may be declared on a document and 256 across the loaded model.
Native limits admit up to 10000 statements and up to 30000 terms.
Generated diagram-asset publication is limited to 4096 assets, 4 MiB per file and 32 MiB of
aggregate text. Missing or malformed inputs refuse; no partial bundle is returned.
<!-- raf:tested-by raf_workplan::test_crate::sync::nothing_is_in_the_report_for_a_repository_that_is_in_sync,raf_workplan::test_crate::sync::the_report_is_a_pure_function_of_the_documents_and_is_sorted_by_path,raf_workplan::test_crate::sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal -->

Every asset path crosses checked real directories and ends in an admitted regular
file or missing generated destination. Existing assets must have manifest
ownership. Directory creation can leave empty generated directories after an I/O
failure; retirement removes files, not unrelated neighbors or directories.
Filesystem rechecks are observational and retain the writer's existing race
boundary; no atomic snapshot or multi-file transaction is claimed.
<!-- raf:tested-by raf_workplan::test_crate::sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal,raf_workplan::test_crate::sync::nothing_is_in_the_report_for_a_repository_that_is_in_sync -->


- Capability generated blocks sit inside a section. They do not replace a required section and do
  not count toward its being non-empty.
  <!-- raf:tested-by raf_workplan::test_crate::generated::a_matrix_and_a_relationship_table_sit_in_a_section_of_a_capability_document,raf_workplan::test_crate::generated::a_generated_block_does_not_make_a_section_non_empty -->
- Editing inside a block by hand produces a stale block, so the edit is overwritten by
  the next sync and reported by `check` first.
  <!-- raf:tested-by raf_workplan::test_crate::generated::an_edited_removed_or_added_matrix_row_is_stale_and_sync_repairs_it,raf_workplan::test_crate::generated::an_edited_removed_or_added_relationship_row_is_stale_and_sync_repairs_it,raf_workplan::test_crate::generated::an_edited_removed_or_added_catalogue_row_is_stale_and_sync_repairs_it -->
- Sync is idempotent: once the report of `Workplan::sync` is written, a second
  `Workplan::sync` reports nothing, so running it twice changes nothing the second
  time.
  <!-- raf:tested-by raf_workplan::test_crate::sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal,raf_workplan::test_crate::sync::write_puts_every_edit_on_disk_and_a_second_run_writes_nothing -->
- Except for computed manifest-owned diagram assets, `Workplan::write` writes only to a regular file that already exists or to the new
  index in an existing `docs/capabilities`, below directories that are each a
  directory and not a symbolic link, and refuses a symbolic link, a directory, a
  path outside the root, a path below a directory link, whether the link leads
  inside the root or out of it, and a path listed twice. Every target is judged before
  the first byte is written.
  <!-- raf:tested-by raf_workplan::test_crate::sync::write_refuses_a_target_that_is_not_an_existing_regular_file_or_the_new_index,raf_workplan::test_crate::sync::write_refuses_a_target_behind_a_directory_link_inside_the_root -->
- Every text goes to a temporary file first and is renamed into place once all are
  written. A staging failure leaves every target unchanged and removes the temporary
  files already staged.
  <!-- raf:tested-by raf_workplan::test_crate::sync::write_changes_no_target_when_a_later_file_cannot_be_written -->
- A rename that fails after earlier ones succeeded leaves those files with their
  new text, preserves the remaining targets and removes the unpublished temporary
  files. Once the failing target can be replaced, the next sync computes the
  remaining edits.
  <!-- raf:tested-by raf_workplan::sync::tests::a_late_rename_failure_preserves_published_files_and_discards_the_tail -->
- `preview` returns text and writes nothing.
  <!-- raf:tested-by raf_workplan::test_crate::preview::a_disk_fixture_is_byte_identical_afterwards -->
- `preview` refuses a PROPOSED block whose quoted text holds a fenced code block,
  because the block reader ends a block at the fence; the error names the line. It
  also refuses a block line that is not indented as its marker line is.
  <!-- raf:tested-by raf_workplan::test_crate::preview::a_fenced_code_block_in_a_block_of_the_change_is_refused_with_its_line,raf_workplan::test_crate::preview::a_continuation_that_is_not_indented_as_the_marker_is_refused_with_its_line,raf_workplan::test_crate::preview::a_block_in_a_list_item_keeps_its_indentation -->

## Explanation

Generation is limited to what a program can get right. A table that restates the
bindings is correct by construction, and it cannot drift from the bindings it
restates. The reasons behind a design, and the instruction for using it, cannot be
derived from structure, and a generator that fabricated them would make documents
that look complete and say nothing.

Failing `check` on a stale block is what makes the tables trustworthy: a reader who
sees a verification matrix knows it was produced from the bindings at this revision.
Determinism makes that cheap to maintain, because a sync on an unchanged repository
produces no diff.

The catalogue is the roadmap. A planned capability appears as upcoming work, an
implementing one appears with its gaps, and a delivered one appears in the catalogue,
so the question "what is built and what is not" has one answer that is derived rather
than written.

## Verification

The native completion fixture checks that a generated replacement breaks its
original hardlink relationship while retaining the ignored alias's original bytes.

- Exact-output fixtures for each kind: the matrix rows, kinds, sort order and section
  links; the relationship rows, their order and the header-only table; the catalogue
  groups, slug order and gaps; and the cell escaping. A relation listed twice is one
  row, an example that does not resolve is listed as an example target, a backtick in
  a binding path keeps its code span, whose fence is longer than any backtick run
  inside it, and a state heading starts with a capital.
- A fixture repository synced twice: the second sync reports no change and the bytes
  are equal, and `check` is clean exactly when sync has nothing to write. `check` and
  sync agree after each repair, a repository in sync has nothing in the report, a
  block in a section other than Verification converges in one sync, and the
  frontmatter of a rewritten document is kept byte for byte.
- A staleness fixture per kind: an edited row, a removed row and an added binding each
  make `check` fail naming the kind, and a sync repairs it.
- Placement fixtures: a matrix and a relationship table in a section of a capability
  document, a catalogue only in the index, and blocks in a change record or an
  architecture page refused and never rewritten.
- A marker fixture for each error: nested blocks, an unclosed block, an end marker
  without a start and an unknown kind.
- A fixture where text outside a block carries a distinctive string, asserting sync
  leaves it unchanged, and a file without blocks that is not in the report.
- Line-ending fixtures: a CRLF document, an LF document, a CRLF frontmatter over an
  LF body, and a body with mixed endings that sync refuses and `check` reports. The
  line ending of a body is its first break, so a CRLF frontmatter over an LF body
  gets LF blocks and an LF frontmatter over a CRLF body gets CRLF blocks, and a CRLF
  document syncs idempotently, a stale block in it repaired in CRLF.
- Index fixtures: a missing index created with its SPDX lines, SPDX text that is
  required and fits one comment line, and a hand-written `index.md` that is rejected.
  An existing index keeps its own header while a stale catalogue is rewritten in
  place, an index that is only a licence header and blank lines is not a generated
  index, and a repository without capabilities needs no index.
- Write fixtures on disk: every edit written and nothing on a second run, a refused
  target, and no target changed when a later file cannot be written; the `docs_sync`
  example writes the expected files once and nothing on a second run. A new index is
  created only where its directory exists, a replaced file keeps its permissions, and
  a file already there under the temporary name is never touched. The `docs_sync`
  example opens the repository as the gate does, asks for the SPDX flags only when it
  must create the index, syncs a git checkout, and refuses an unknown option, a flag
  without its value and an extra argument with exit 2 and nothing on standard
  output.
- A directory link inside the root at `docs` or at `docs/capabilities`, made after
  the report was computed, is refused when the report is written, with nothing
  written, though the files behind it are regular files; the same report through
  the real directories is written.
- The four tests that run the `docs_sync` example hold one lock of their test
  binary from their first process to their last, as the other process tests of the
  crate hold theirs, so no run of the example starts it while another `cargo`
  writes it.
- Preview fixtures: a change with two blocks in one capability and a block of another
  change that stays, line endings kept, a tab after `>` kept, a fenced code block and
  a misindented line refused with their lines, an unknown slug, and a disk fixture
  that is byte-identical afterwards. A block at the end of a file without a final
  break is resolved, a block in a list item keeps its indentation, a block with no
  content lines disappears with its marker, a nested quote loses one level only,
  adjacent blocks of two changes are told apart, and a fenced code block in a block
  of another change is no obstacle; the text outside the blocks is unchanged byte for
  byte, and two previews are equal. A change is found by its slug with or without the
  `change:` prefix, the lookup and the preview say the same thing about a slug, an
  unknown slug is told apart from a record that did not load, which says why, and a
  listed capability that did not load or has no document is an error that names it.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `cargo_workplan::test_crate::rdf::standalone_publication_preserves_host_files_and_refuses_changed_owned_bytes` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples) |
| `raf_workplan::diagram::semantic::identity::tests::persisted_identity_profiles_pin_field_boundaries_and_escaping` | test | [Contract](#contract) |
| `raf_workplan::example_crate::docs_sync` | example target | [Usage](#usage) |
| `raf_workplan::generated::tests::a_code_span_fence_is_longer_than_any_backtick_run_inside` | test | [Verification](#verification) |
| `raf_workplan::generated::tests::a_state_heading_has_a_capital` | test | [Verification](#verification) |
| `raf_workplan::generated::tests::cell_text_is_escaped_by_the_documented_rule` | test | [Verification](#verification) |
| `raf_workplan::generated::tests::the_line_ending_is_the_first_break_and_a_change_is_mixed` | test | [Verification](#verification) |
| `raf_workplan::graph::tests::a_comment_outside_a_code_span_is_removed_and_one_inside_a_span_is_kept` | test | [Verification](#verification) |
| `raf_workplan::sync::tests::a_late_rename_failure_preserves_published_files_and_discards_the_tail` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::complete::a_delivered_capability_with_nothing_owed_completes_and_the_only_writes_are_the_rewrite_and_the_removal` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::a_backtick_in_a_binding_path_does_not_break_the_code_span` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::a_block_in_a_section_other_than_verification_converges_in_one_sync` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::a_body_that_mixes_line_endings_is_refused_by_sync_and_reported_by_check` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::a_capability_with_no_relation_has_a_table_of_the_header_only` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::a_catalogue_block_is_valid_only_in_the_capability_index` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::a_crlf_document_stays_all_crlf_and_an_lf_document_all_lf` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::a_crlf_frontmatter_over_an_lf_body_gets_lf_blocks_and_stays_clean` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::a_gap_names_an_unresolved_binding_and_an_empty_list` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::a_generated_block_does_not_make_a_section_non_empty` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::generated::a_marker_fixture_for_each_error_nested_unclosed_orphan_end_and_unknown_kind` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::a_matrix_and_a_relationship_table_sit_in_a_section_of_a_capability_document` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::generated::a_matrix_or_a_relationship_table_in_the_index_is_misplaced` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::a_missing_index_is_a_finding_and_sync_creates_it_with_the_spdx_lines` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::a_path_named_in_a_fence_or_a_proposal_is_not_a_section_that_it_substantiates` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::a_relation_listed_twice_is_one_row` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::a_relationship_table_lists_each_relation_sorted_by_slug_with_links` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::a_repository_without_capabilities_needs_no_index` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::a_tested_by_marker_substantiates_its_section_and_an_unmarked_binding_keeps_its_row` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::a_verification_matrix_lists_each_test_and_example_by_path_with_its_sections` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::an_edited_removed_or_added_catalogue_row_is_stale_and_sync_repairs_it` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::generated::an_edited_removed_or_added_matrix_row_is_stale_and_sync_repairs_it` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::generated::an_edited_removed_or_added_relationship_row_is_stale_and_sync_repairs_it` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::generated::an_example_that_does_not_resolve_is_listed_as_an_example_target` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::an_existing_index_keeps_its_own_header_and_a_stale_catalogue_is_rewritten_in_place` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::an_index_that_is_only_a_licence_header_and_blank_lines_is_not_a_generated_index` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::an_index_with_text_outside_its_blocks_is_rejected_whatever_else_it_holds` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::an_lf_frontmatter_over_a_crlf_body_gets_crlf_blocks` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::blocks_in_a_change_record_or_an_architecture_page_are_misplaced_and_never_rewritten` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::cell_text_is_escaped_the_same_way_every_time` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::groups_with_no_capability_are_omitted_and_each_group_is_sorted_by_slug` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::rows_are_sorted_by_path_whatever_the_order_of_the_lists` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::text_outside_a_block_is_never_changed_and_a_file_without_blocks_is_untouched` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::the_catalogue_cannot_be_forged_from_a_description_that_looks_like_a_marker` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::the_catalogue_is_a_table_per_state_in_order_with_gaps_for_implementing_only` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::the_frontmatter_of_a_rewritten_document_is_kept_byte_for_byte` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::the_kinds_the_crate_owns_are_the_three_documented_ones` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::the_spdx_text_of_a_new_index_is_required_and_must_fit_one_comment_line` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::the_stale_finding_points_at_the_opening_marker_of_the_block` | test | [Contract](#contract) |
| `raf_workplan::test_crate::generated::the_sync_of_a_crlf_document_is_idempotent_and_stale_in_crlf_is_repaired_in_crlf` | test | [Verification](#verification) |
| `raf_workplan::test_crate::generated::the_synced_base_has_no_finding_at_all` | test | [Contract](#contract) |
| `raf_workplan::test_crate::preview::a_block_at_the_end_of_a_file_without_a_final_break_is_reconciled` | test | [Verification](#verification) |
| `raf_workplan::test_crate::preview::a_block_in_a_list_item_keeps_its_indentation` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_workplan::test_crate::preview::a_block_with_no_content_lines_disappears_with_its_marker` | test | [Verification](#verification) |
| `raf_workplan::test_crate::preview::a_change_is_found_by_its_slug_with_or_without_the_prefix` | test | [Verification](#verification) |
| `raf_workplan::test_crate::preview::a_change_record_that_did_not_load_says_why` | test | [Contract](#contract) |
| `raf_workplan::test_crate::preview::a_change_with_two_blocks_in_a_capability_reads_reconciled_and_other_blocks_stay` | test | [Contract](#contract) |
| `raf_workplan::test_crate::preview::a_continuation_that_is_not_indented_as_the_marker_is_refused_with_its_line` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::preview::a_crlf_document_keeps_its_line_endings` | test | [Contract](#contract) |
| `raf_workplan::test_crate::preview::a_disk_fixture_is_byte_identical_afterwards` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::preview::a_fenced_code_block_in_a_block_of_another_change_is_no_obstacle` | test | [Verification](#verification) |
| `raf_workplan::test_crate::preview::a_fenced_code_block_in_a_block_of_the_change_is_refused_with_its_line` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::preview::a_listed_capability_that_did_not_load_is_named_never_skipped` | test | [Contract](#contract) |
| `raf_workplan::test_crate::preview::a_listed_capability_without_a_document_is_an_error` | test | [Verification](#verification) |
| `raf_workplan::test_crate::preview::a_nested_quote_loses_one_level_only` | test | [Contract](#contract) |
| `raf_workplan::test_crate::preview::a_tab_after_the_quote_marker_is_kept_as_written` | test | [Contract](#contract) |
| `raf_workplan::test_crate::preview::adjacent_blocks_of_two_changes_are_told_apart` | test | [Verification](#verification) |
| `raf_workplan::test_crate::preview::an_unknown_slug_is_told_apart_from_a_record_that_did_not_load` | test | [Verification](#verification) |
| `raf_workplan::test_crate::preview::an_unknown_slug_names_it_and_lists_the_open_changes` | test | [Contract](#contract) |
| `raf_workplan::test_crate::preview::another_change_reconciles_only_its_own_blocks` | test | [Contract](#contract) |
| `raf_workplan::test_crate::preview::preview_is_deterministic` | test | [Contract](#contract) |
| `raf_workplan::test_crate::preview::the_lookup_and_the_preview_say_the_same_thing_about_a_slug` | test | [Verification](#verification) |
| `raf_workplan::test_crate::preview::the_slug_is_taken_with_or_without_the_prefix` | test | [Contract](#contract) |
| `raf_workplan::test_crate::preview::the_text_outside_blocks_is_unchanged_byte_for_byte` | test | [Verification](#verification) |
| `raf_workplan::test_crate::sync::a_document_that_does_not_load_stops_the_sync_since_the_tables_would_miss_it` | test | [Contract](#contract) |
| `raf_workplan::test_crate::sync::check_and_sync_agree_after_each_repair` | test | [Contract](#contract) |
| `raf_workplan::test_crate::sync::generated_checks_are_clean_exactly_when_sync_has_nothing_to_write` | test | [Contract](#contract) |
| `raf_workplan::test_crate::sync::nothing_is_in_the_report_for_a_repository_that_is_in_sync` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_workplan::test_crate::sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_workplan::test_crate::sync::the_catalogue_of_this_repository_equals_its_roadmap` | test | [Verification](#verification) |
| `raf_workplan::test_crate::sync::the_docs_sync_example_asks_for_the_spdx_flags_only_when_the_index_must_be_created` | test | [Verification](#verification) |
| `raf_workplan::test_crate::sync::the_docs_sync_example_in_a_git_checkout_and_with_bad_arguments` | test | [Verification](#verification) |
| `raf_workplan::test_crate::sync::the_docs_sync_example_opens_the_repository_as_the_gate_does` | test | [Verification](#verification) |
| `raf_workplan::test_crate::sync::the_docs_sync_example_writes_the_expected_files_once_and_nothing_on_a_second_run` | test | [Verification](#verification) |
| `raf_workplan::test_crate::sync::the_report_is_a_pure_function_of_the_documents_and_is_sorted_by_path` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_workplan::test_crate::sync::the_roadmap_rows_equal_the_catalogue_rows` | test | [Verification](#verification) |
| `raf_workplan::test_crate::sync::write_changes_no_target_when_a_later_file_cannot_be_written` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::sync::write_creates_the_index_only_where_the_directory_exists` | test | [Verification](#verification) |
| `raf_workplan::test_crate::sync::write_keeps_the_permissions_of_the_file_it_replaces` | test | [Verification](#verification) |
| `raf_workplan::test_crate::sync::write_never_touches_a_file_that_is_already_there_under_a_temporary_name` | test | [Verification](#verification) |
| `raf_workplan::test_crate::sync::write_puts_every_edit_on_disk_and_a_second_run_writes_nothing` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::sync::write_refuses_a_target_behind_a_directory_link_inside_the_root` | test | [Constraints](#constraints) |
| `raf_workplan::test_crate::sync::write_refuses_a_target_that_is_not_an_existing_regular_file_or_the_new_index` | test | [Constraints](#constraints) |
<!-- raf:generated:end -->

<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [workplan-document-model](./workplan-document-model.md) |
| Depended on by | [workplan-cli](./workplan-cli.md) |
| Depended on by | [workplan-complete-gate](./workplan-complete-gate.md) |
| Depended on by | [workplan-prose-assessment](./workplan-prose-assessment.md) |
| Depended on by | [workplan-reconcile](./workplan-reconcile.md) |
| Depended on by | [workplan-rustdoc](./workplan-rustdoc.md) |
| Depended on by | [workplan-strict](./workplan-strict.md) |
<!-- raf:generated:end -->
