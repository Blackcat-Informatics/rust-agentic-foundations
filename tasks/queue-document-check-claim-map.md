<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Document model and generated documents: bounded claim map

Read-only preparation against the current issue checkout. The requested second
document was corrected by the owner to `workplan-generated-docs.md`; no
`workplan-check.md` exists. No builds, tests, product edits, lifecycle operations
or completion claims were made. The assertions described below were inspected
in source; this is not evidence that they passed in the current checkout.

## Placement and path notation

Contract needs meaningful `###` clauses, rather than its present bold paragraph
labels. The headings suggested below are insertion anchors: preserve the
corresponding prose under each heading, split compound claims where indicated,
and place only its supporting marker inside that clause. Paths written here as
`module::test` expand exactly to
`raf_workplan::test_crate::module::test`. Their source is
`crates/raf-workplan/tests/module.rs`, at the named function. For example:

```markdown
### Invalid documents remain findings

A document that does not validate is a finding and is not in the model.
<!-- raf:tested-by raf_workplan::test_crate::documents::a_broken_document_is_a_finding_and_not_in_the_model -->
```

**Correction to the earlier census map:** a Constraints marker must immediately
follow the actual guarantee paragraph or list item. A marker below a new
Constraints heading does not cover the following limitation. Current
`src/strict/claims.rs:714-731` uses the marker line for a Contract clause but
`marker.statement().lines.start` for a Constraints unit. Keep each Constraints
list item and put its marker after that item, before the next item. Contract
heading markers can legitimately cover their containing clause.

## Document model: Contract

Current anchors are the bold labels at `docs/capabilities/workplan-document-model.md:349-574`.
The following groups are independent clauses, not one blanket suite.

| Proposed heading / existing anchor | Inspected supporting paths and actual assertions | Limits or necessary split |
| --- | --- | --- |
| Capability identity and shape / Capability documents | `documents::the_model_holds_typed_values_and_bodies` checks paths, ids, typed states, bindings, seven body headings, change fields and architecture body; `references::a_slug_id_that_differs_from_the_file_name_is_refused`, `references::a_slug_that_is_not_kebab_case_is_refused`, `references::a_matching_kebab_slug_is_accepted` check specific refused kinds/keys and valid neighbor. | Permanent identity across a capability's lifetime is repository policy, not a property established by loading one snapshot. Keep that promise visibly owned; do not mark it proved by these fixtures. |
| State and open-record consistency / The four states | `states::accepts_each_state_with_matching_content` checks accepted considered/designed/delivered and implementing with and without proposals; `states::rejects_each_non_delivered_state_without_an_open_record` checks no-open-record and delivered neighbor; `states::rejects_delivered_with_open_change_record` checks kind, state key and named record. | These check structural state consistency, not whether supported prose is semantically true or whether an editor earned the state change. Split the transition/history policy paragraph from the structural rule. |
| Proposal ownership and placement / PROPOSED blocks | `states::rejects_orphan_proposed_marker`, `states::rejects_proposed_section_missing_from_affects`, `states::rejects_proposed_change_that_does_not_list_the_capability` check kind/file/marker location and named relationship; `states::rejects_considered_without_proposed_block`, `states::rejects_delivered_with_proposed_block`, `states::rejects_designed_with_proposed_block` check state refusals. `documents::rejects_proposed_marker_in_architecture` and `documents::rejects_proposed_marker_in_change` check profile refusal/line. | These are not a proof of every quote termination rule. Preview tests below exercise adjacent changes, nested quotes and fences; the OKF block parser owns the underlying syntax. Keep parser-boundary proof separate rather than claiming this state fixture proves it. |
| Change record structure / Change documents | `documents::the_model_holds_typed_values_and_bodies` checks changes, affects and contract effect. `rules::every_rule_has_a_fixture_reported_with_file_and_location` iterates `Rule::ALL`, requires the corresponding named rule fixture and calls its exact-result assertion, beside a clean valid fixture. `references::accepts_a_moved_outcome_that_names_an_existing_change` is an existing bound candidate whose body was not inspected here. | The rule registry is relevant to schema enforcement only; identify the specific `support/cases` rule for a new guarantee before binding it. File deletion on completion belongs to actual completion tests, not these load tests. Git retention is external policy. |
| Slug references / Slug identities | The three `references` slug tests above; `references::rejects_depends_cycles_once_for_each_group_naming_its_members` checks self/two/three-node cycles, one finding, file, exact member occurrence count; `references::two_separate_cycles_are_two_findings_and_a_cycle_with_a_tail_is_one` checks SCC finding count. | Renaming all references is a user's operation. The test proves mismatches/refusals, not automatic rename. |
| Typed loading and invalid exclusion / Loading | `documents::accepts_valid_fixture` checks loaded collection/package counts and clean load/check fixture; `documents::the_model_holds_typed_values_and_bodies`; `documents::a_broken_document_is_a_finding_and_not_in_the_model` checks two remaining capabilities and one bad-state finding. `documents::an_oversize_file_is_reported_by_name_and_not_skipped` checks unreadable-file and name. | “Nothing panics” is universal wording not proved by finite fixtures; these demonstrate representative invalid inputs. Separate loading through Tree from any stronger panic-safety claim. |
| Finding vocabulary and diagnostics / Loading, final sentences | `kinds::kinds_are_unique_kebab_case_names` checks uniqueness/syntax; `kinds::every_rule_and_every_reader_error_has_its_own_kind` checks exact sets/count; `kinds::every_kind_is_produced_by_a_named_fixture_and_every_produced_kind_is_a_kind` invokes exact fixture checks and compares produced vs declared kinds; `kinds::every_finding_names_a_fix_and_a_message` checks nonempty messages/fixes. `documents::a_finding_renders_with_its_place_and_its_fix` checks rendered path/key/kind/fix. | Registry exhaustiveness is current vocabulary coverage, not proof every arbitrary invalid input is diagnosed. |
| Typed records and absent directories / Records | `records::a_repository_without_record_directories_has_no_records_and_no_new_findings` actually expects exactly coverage-record-missing after removing coverage, empty record collections/edges, and clean empty non-Markdown directories. `records::each_record_type_loads_from_its_directory_into_the_model` inspects typed collections. | Extra-file and debt singleton assertions remain separate bound candidates (`records::work_coverage_holds_only_the_coverage_record`, `debt::a_file_other_than_the_one_record_is_a_finding`); do not mark these inspected on the strength of their names. |
| Complete policy and ordering / Complete policy | `strict::content_and_binding_findings_follow_load_findings` asserts load excludes content/coverage findings, checked starts with load, then exact added kinds after excluding generated findings. `strict::the_removed_policy_key_is_rejected_even_for_its_former_values` refuses strict/default/typo key values at the coverage key, accepts no key. `strict::the_default_gate_judges_the_complete_policy_in_its_opened_workplan` compares direct/opened check and exact thin-section finding after removing generated findings. | These prove the observed load/check/Gate paths. Reconcile and completion paths require their own source-driven execution assertions; not inferred from Gate. |
| Graph structure / The graph, nodes and ordinary edges | `graph::the_graph_has_every_kind_of_node` checks NodeKind inventory, all seven sections per capability, proposal fields, actual symbol/test/example nodes and uniqueness. `graph::the_graph_has_every_kind_of_edge` checks ordinary edge counts, deduplicated affects, exact relationship values and tested-by section/path/line. | Pair with specific link and record clauses below; no assertion that a source resolver's untested cases work. |
| Graph binding and reverse queries / The graph, queries | `graph::bindings_record_what_the_source_index_made_of_each_path` checks actual reexport definition and unresolved empty sites. `graph::code_of_lists_the_items_that_realise_a_capability_in_path_order` checks ordered code, excludes tests/unresolved; `graph::capabilities_of_names_every_capability_bound_to_a_symbol` checks reverse code/test/example lists. `graph::sections_of_a_change_are_its_affected_sections_sorted` checks exact sections and unknown empty result. | `graph::linking_sections_are_section_granular` and `graph::the_queries_are_deterministic` are bound candidates; the full bodies were not inspected in this pass. |
| Record edges and unresolved input absence / The graph, Record edges | `records::the_records_are_nodes_with_their_edges` checks typed record/crate nodes, exact covers/assessment/scope values, reverse assessment query and empty neighbor. `records::each_input_of_a_record_that_resolves_is_an_edge_and_one_that_does_not_is_no_finding` checks clean verdict, exact section/symbol/reexport/record input targets, edge totals and symbol-node deduplication. `records::a_reference_that_does_not_resolve_makes_no_edge` checks missing covers/scope and no invented crate node while the evidence node remains. | Claim-edge tests `records::claims_are_section_granular_and_one_edge_per_named_record` and proposal-inactive tests are bound candidates here; inspect bodies before selecting them for this clause. |
| Binding roles and resolution / Binding resolution | `bindings::a_binding_through_a_re_export_resolves_to_its_definition` checks actual definition id/path; `bindings::the_role_is_judged_on_the_definition_a_re_export_leads_to` checks exact wrong-role message. `bindings::a_tests_binding_must_name_a_function_marked_test`, `bindings::a_code_binding_must_name_a_shipping_item`, `bindings::an_examples_binding_is_an_example_target_or_a_documented_library_item` exercise real role refusals and allowed example/doctest neighbors. `bindings::a_binding_to_a_missing_path_is_reported_with_its_capability`, `bindings::a_path_two_globs_both_bring_in_is_ambiguous_and_names_both_definitions`, `bindings::a_binding_to_a_macro_generated_function_is_not_found` check exact kind and contextual message. | Keep gap reporting separate. These use their actual fixture source; do not import unrelated source-index resolver conclusions. |
| Roadmap grouping and gaps / Status | `roadmap::an_implementing_capability_lists_its_gaps_exactly` asserts exact ordered Gap values and display text; `roadmap::only_an_implementing_capability_has_gaps` checks all other states have none; `roadmap::a_resolving_binding_of_the_wrong_role_is_not_a_gap` distinguishes a check refusal from no gap. `roadmap::open_changes_are_the_changes_that_list_the_capability` checks exact lists; `roadmap::the_roadmap_does_not_depend_on_insertion_order_and_repeats` compares reversed insertion and repeated load; `roadmap::slug_order_is_not_file_name_order` distinguishes loader/file and status/slug order. | Preserve wrong-role distinction; “gaps” must not expand silently to all failed bindings. |
| Profile and directory checks / Checks, first bullets | `rules::every_rule_has_a_fixture_reported_with_file_and_location` is specifically a profile-rule inventory assertion; `documents::rejects_nested_documents`, `documents::rejects_type_in_the_wrong_directory`, `documents::rejects_capability_without_frontmatter` assert separate file/location/kind failures. | Do not use the inventory test as evidence for unrelated link, graph, content or I/O claims. Split this large Checks list by enforcement domain. |
| Links and plain Markdown / Checks, links and final paragraph | `links::rejects_broken_relative_link`, `links::rejects_broken_anchor`, `links::rejects_link_leaving_the_repository`, `links::rejects_link_to_a_tracked_symlink` check real kinds/messages and clean regular-file neighbor; `links::anchors_follow_github_rules` checks accepted duplicate/Unicode anchors and case/missing refusal. `plain::rejects_a_link_split_across_lines_in_any_tracked_markdown_file`, `plain::accepts_one_line_links_and_links_in_fences_beside_the_split_rule`, `plain::does_not_read_links_or_placeholders_inside_html_comments` assert shared forms, suppression and visible neighbor. | Shared refs/process-wording tests and relationship-refusal tests remain specific candidates until their actual bodies are inspected. Do not attach plain link tests to refs or all marker behavior. |
| Canonical digests / Digests | `digest::each_kind_hashes_its_documented_encoding` pins three literal independent SHA-256 values; `digest::a_symbol_and_a_record_with_equal_bytes_differ` checks domain separation; `digest::crlf_and_lf_give_the_same_digest` checks all domains plus lone/doubled CR neighbors. `digest::a_regenerated_block_leaves_the_section_digest_and_an_edit_moves_it` checks generated content excluded, markers retained, other section irrelevant. `digest::every_definition_of_a_path_counts_once_in_file_order` checks multiple definitions, changed second definition and duplicate cfg-file deduplication. `digest::a_module_stands_for_its_files_and_the_modules_below_it_and_a_sibling_does_not_move_it` checks own/child edits move digest, sibling doesn't. `digest::a_digest_is_written_and_read_as_sixty_four_lowercase_hex_digits` checks byte length, roundtrip and malformed variants. | Split encoding/section/symbol/module/representation guarantees for focused markers. Platform determinism is supported by normalization assertions, not a native run on every platform. |
| Exclusive record creation / Writing records | `records::create_never_replaces_a_file` checks Exists and unchanged full snapshots for preexisting and repeated create. `records::a_written_record_reloads_with_no_finding_as_the_same_value` checks actual paths, both SPDX lines, typed roundtrip and no-SPDX neighbor. | These are separate from replacement safety. |
| Record replacement and preflight refusal / Writing records | `records::replace_puts_new_text_in_place_of_an_existing_record_only` checks missing refusal/snapshot, changed verdict roundtrip, no temporary leftovers and directory refusal. `records::a_record_that_would_not_load_clean_is_not_written` checks invalid digest/reference/slug/SPDX halves/multiline, unchanged snapshot and valid write neighbor. | Symlink/rename tests already bound in frontmatter are candidates requiring their own body read for the full filesystem clause; no race-free universal guarantee inferred. |

### Document-model drift that binding cannot repair

The Checks list still says “under strict”, describes `default`, and claims
`debt-without-strict` from the removed switch. This conflicts with its Complete
policy clause and the inspected removed-key/default-check tests. The kind may
remain in vocabulary for a different condition, so removal of the string alone
is not the repair: describe the current actual debt refusal/ratchet condition
and bind its specific assertion. No acceptance waiver or lightweight policy is
proposed. Also inspect reconcile/completion callers before asserting their
complete-policy execution coverage here.

## Document model: Constraints

Retain all six current top-level limitations and place markers immediately after
the relevant item. Do not introduce heading markers for Constraints.

1. Exact profile plus cross-document/source/line-ending/delivered minimum is
   several guarantees. Split into profile vocabulary (specific Rule fixture),
   binding role (the inspected bindings tests above), and line endings
   (`generated::a_body_that_mixes_line_endings_is_refused_by_sync_and_reported_by_check`
   is a bound candidate, not inspected here). Do not attach every rule test to all.
2. Nonempty changes and affects membership: identify their exact `Rule` fixtures
   in `support/cases` before selecting `rules::every_rule_has_a_fixture_reported_with_file_and_location`.
   The state tests about proposal affects do not establish this broader schema rule.
3. Current/built prose outside proposals: semantic authoring policy remains
   unproved by structural state/refusal tests. Preserve the policy and owner;
   do not claim tests detect arbitrary false prose. A rationale explanation can
   explain that check enforces structure while assessment owns semantic review.
4. Flat directories: `documents::rejects_nested_documents` asserts both capability
   and change nested paths at Document location. A typed nested document candidate
   adds exclusion from reading; inspect its body before selecting it.
5. No automatic advancement / same-change earning: static load assertions show
   consistency, not historical transition earning. Keep separate from structural
   state checks and identify actual lifecycle/history execution if claiming that.
6. Outside Git / caller override: `gate::open_gives_the_tree_the_workspace_the_tracked_paths_and_the_documents`
   checks no tracked override outside Git and clean loaded check;
   `gate::the_tracked_paths_a_caller_gives_replace_the_listing_and_start_no_process`
   installs a nonexistent git, succeeds with explicit paths, and fails without
   them with Tracked error and actionable message. The explicit untracked-link
   acceptance case should be confirmed in the relevant gate fixture before
   claiming this entire consequence proved by these two tests.

## Generated documents: Contract

Use these headings instead of the bold labels at
`docs/capabilities/workplan-generated-docs.md:125-205`. All names below are actual
bound tests. The named assertions in this table were inspected unless explicitly
marked candidate.

| Proposed heading / anchor | Supporting paths and exact assertion scope |
| --- | --- |
| Generated block syntax / Block syntax | `generated::a_marker_fixture_for_each_error_nested_unclosed_orphan_end_and_unknown_kind` checks each distinct reader/kind refusal and sync Unloaded vs Refused outcome. It includes malformed uppercase/underscore kind and the valid synced base neighbor. |
| Verification matrix rows / Kinds first item | `generated::a_verification_matrix_lists_each_test_and_example_by_path_with_its_sections` compares full matrix including target/doctest/test kind, multiple sections, Verification fallback and empty header. `generated::rows_are_sorted_by_path_whatever_the_order_of_the_lists` compares exact reordered matrix. `generated::a_tested_by_marker_substantiates_its_section_and_an_unmarked_binding_keeps_its_row` checks exact marker-driven matrix and clean check. Fence/proposal test only adds a fenced path in its actual body; do not assume its name proves every proposal/comment case. |
| Relationship table rows / Kinds second item | `generated::a_relationship_table_lists_each_relation_sorted_by_slug_with_links` compares complete tables across three capabilities, ordered relation classes, slugs and relative targets. Header-only test body was partially read; retain as candidate until its final assertion is checked. |
| Catalogue grouping and gaps / Kinds third item | `generated::the_catalogue_is_a_table_per_state_in_order_with_gaps_for_implementing_only` compares entire index including all four states/descriptions/gap cells. `generated::a_gap_names_an_unresolved_binding_and_an_empty_list` checks literal gap cell. `generated::groups_with_no_capability_are_omitted_and_each_group_is_sorted_by_slug` checks exact rows and only Delivered heading. Actual equality with Roadmap is separately owned by `sync::the_roadmap_rows_equal_the_catalogue_rows` (candidate body not inspected here). |
| Placement and table escaping / after Kinds | `generated::a_matrix_and_a_relationship_table_sit_in_a_section_of_a_capability_document`, `generated::a_catalogue_block_is_valid_only_in_the_capability_index`, `generated::a_matrix_or_a_relationship_table_in_the_index_is_misplaced`, `generated::blocks_in_a_change_record_or_an_architecture_page_are_misplaced_and_never_rewritten` check misplaced kinds/files and refused sync. `generated::cell_text_is_escaped_the_same_way_every_time` checks actual escaped pipe/brackets/angle/backslash/newlines in a catalogue cell; source unit `raf_workplan::generated::tests::cell_text_is_escaped_by_the_documented_rule` is a candidate for individual-character completeness. |
| Index creation and refusal / The index | `generated::a_missing_index_is_a_finding_and_sync_creates_it_with_the_spdx_lines` checks exactly missing-index finding, one edit, header/content bytes and repaired check. `generated::the_spdx_text_of_a_new_index_is_required_and_must_fit_one_comment_line` checks unset/empty/multiline/comment-ending cases and existing-index options neighbor. `generated::an_index_with_text_outside_its_blocks_is_rejected_whatever_else_it_holds` checks hand-written finding/file/fix and sync Refused. |
| Pure sync report / Sync first sentences | `sync::the_report_is_a_pure_function_of_the_documents_and_is_sorted_by_path` compares repeated reports and exact ordered edit paths. `sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal` checks nonempty first, empty second/third and unchanged applied bytes. `sync::write_puts_every_edit_on_disk_and_a_second_run_writes_nothing` checks all four actual file bytes, untouched file, no temporary leftovers and empty second report. Pure no-I/O assertion requires source inspection (`src/sync.rs` separates report computation from write); a returned report alone does not prove no disk mutation. |
| Preserved authored bytes / Sync preservation | `generated::text_outside_a_block_is_never_changed_and_a_file_without_blocks_is_untouched` checks outside-block bytes including Unicode/tabs/spacing, exact touched set, and untouched no-block/change/architecture files. `generated::the_frontmatter_of_a_rewritten_document_is_kept_byte_for_byte` checks unusual quoting/comments preserved. `generated::a_crlf_document_stays_all_crlf_and_an_lf_document_all_lf` checks exact normalized expectations and clean check; differing frontmatter/body and mixed-ending cases remain specific candidates until read. |
| Unloaded documents stop sync / Sync final sentence | `sync::a_document_that_does_not_load_stops_the_sync_since_the_tables_would_miss_it` checks exact unloaded capability id and nonempty check. It does not test unloaded change documents. Add/inspect a real change-document failure before marking the whole “capability or change” guarantee. |
| Generated staleness / Staleness | `generated::the_stale_finding_points_at_the_opening_marker_of_the_block` checks opening line and nonempty fix. `sync::check_is_clean_exactly_when_sync_has_nothing_to_write` compares **generated_clean**, not all check findings, across more than 30 fixtures and skips Unloaded. `sync::check_and_sync_agree_after_each_repair` also checks generated_clean only. Existing three edited/removed/added row tests are candidates for precise repair byte evidence. |
| Preview selection and byte preservation / Preview | `preview::a_change_with_two_blocks_in_a_capability_reads_reconciled_and_other_blocks_stay` checks sorted ids/paths, full expected transformed texts and unchanged no-block capability. `preview::another_change_reconciles_only_its_own_blocks`, `preview::the_slug_is_taken_with_or_without_the_prefix`, `preview::preview_is_deterministic`, `preview::a_crlf_document_keeps_its_line_endings`, `preview::a_block_in_a_list_item_keeps_its_indentation`, `preview::a_tab_after_the_quote_marker_is_kept_as_written`, `preview::a_nested_quote_loses_one_level_only` compare literal text or equal results. |
| Preview lookup failures / Preview last sentence | `preview::an_unknown_slug_names_it_and_lists_the_open_changes` checks raw slug/known list, display and empty/blank neighbors; `preview::a_change_record_that_did_not_load_says_why`, `preview::a_listed_capability_that_did_not_load_is_named_never_skipped` check typed error fields, own-file findings and display. Missing capability test is candidate: its full body was not read. |
| Authored explanatory prose / Mechanical versus explanatory | The inspected outside-block preservation test proves the implementation does not alter prose outside blocks. Who authors explanatory prose is workflow policy, not a runtime assertion. Retain the instruction as policy/rationale without implying a test proves authorship. |

### Generated-document overstatement and real missing proof

“A sync makes it pass” and “a clean check is a sync with nothing to write” must
explicitly refer to generated-content findings. The inspected test intentionally
filters to generated-content kinds. Sync can repair generated blocks while
unrelated model, content or binding findings persist. Do not attach this test as
proof of a universally clean full check, or weaken those unrelated checks.

The change-document-unloaded sync refusal and late-rename partial-success
guarantee below need actual targeted proof. They are not established by nearby
capability-unloaded/preflight/write-failure fixtures. This map identifies missing
proof, not permission to drop the current guarantees.

## Generated documents: Constraints

Keep markers directly after each actual list item; split the long write item into
separate items if separate guarantees need independently reviewable evidence.

1. Section placement and generated content not satisfying nonempty:
   `generated::a_matrix_and_a_relationship_table_sit_in_a_section_of_a_capability_document`
   and `generated::a_generated_block_does_not_make_a_section_non_empty`.
   The latter actually tests block-only Verification refusal and prose neighbor.
2. Manual edits/stale/repair: specific matrix/relationship/catalogue mutation tests
   are bound candidates; the opening-line assertion above is inspected but does
   not itself prove every repair. Keep a distinct marker for each chosen family.
3. Idempotence: `sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal`
   and actual disk neighbor `sync::write_puts_every_edit_on_disk_and_a_second_run_writes_nothing`.
4. Write-target preflight: `sync::write_refuses_a_target_that_is_not_an_existing_regular_file_or_the_new_index`
   checks valid edit preceding bad missing/directory/symlink/duplicate targets,
   unchanged first file and untouched link destination. Unix-only
   `sync::write_refuses_a_target_behind_a_directory_link_inside_the_root` checks
   links at both ancestor levels, unchanged whole snapshot and real-directory
   success neighbor. State the platform scope of these assertions.
5. Staging before rename / write failure: Unix-only
   `sync::write_changes_no_target_when_a_later_file_cannot_be_written` uses a
   read-only directory, checks unchanged three targets and no temporary files;
   it **returns without exercising failure** if the user can still create there.
   A log passing this test is not unconditional evidence of that failure path.
   It also does not exercise a rename failure after an earlier successful rename.
   Preserve that documented partial-success behavior and request an actual
   controlled late-rename neighbor before claiming execution proof.
6. Preview writes nothing: `preview::a_disk_fixture_is_byte_identical_afterwards`
   is a bound candidate; the actual snapshot assertions were not inspected here.
   Pure textual comparisons alone are insufficient proof of no disk writes.
7. Preview malformed quote refusal:
   `preview::a_continuation_that_is_not_indented_as_the_marker_is_refused_with_its_line`
   checks Prefix file/line/display; list-indentation and tab cases above are valid
   neighbors. Fenced-block refusal is a bound candidate requiring its actual body
   read; no claim that the inspected continuation test proves it.

## Remaining qualification boundaries

This is a focused adoption map, not an exhaustive new proof of every current
bound test. Candidate rows deliberately remain unproved until their assertion
bodies are read or a genuine missing test is supplied. No blanket module/suite
marker should replace that work. Keep all guarantees and limitations visible,
keep capability states implementing, and perform the normal complete policy and
required gates only after the owner settles the actual edits.
