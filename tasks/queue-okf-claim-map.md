<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# OKF profile claim preparation

Read-only mapping against the current issue 16 checkout. No builds, tests,
measurements or product edits were performed. All execution acceptance remains
with the owner. Contract currently spans lines 393–667; Constraints 723–738.
Quoted anchors are authoritative if line numbers move.

Introduce meaningful named clauses before the anchors below, retaining the
existing semantics. A heading plus tested-by marker maps one clause, for example:

```markdown
### Closed coverage policy
<!-- raf:tested-by raf_okf::test_crate::records::a_coverage_record_rejects_the_removed_policy_key,raf_okf::test_crate::record_writer::a_coverage_record_writes_only_its_closed_policy_independent_keys -->
```

`T::` in this artifact abbreviates the exact marker prefix
`raf_okf::test_crate::`; expand it when inserting. The paths proposed here are
existing bound tests. An inspected assertion is distinguished from a named
candidate needing final assertion reread.

## Contract map: inspected assertions

| Proposed clause / insertion anchor | Specific marker paths | Source evidence and limits |
|---|---|---|
| Parse framing; keep Parse opening, split before `SPDX lines` | `T::refusals::a_document_without_a_fence_on_line_one_is_refused`, `T::refusals::a_frontmatter_that_is_not_a_block_mapping_is_refused` | Exact functions verified at `refusals.rs:46,80`; bodies are final-reread candidates, not independently reviewed proof in this map. Keep framing and YAML-subset rejection distinct from comment preservation. |
| Comment attachment; before `SPDX lines are YAML comments` | `T::render::comments_attach_to_the_next_key_and_trailing_comments_stay_on_their_line` | `render.rs:347` checks exact leading comments/blank lines, trailing comment spacing, tag-item comment attachment after sort, end comments staying last and stable second rendering. |
| Whole-document size; split Resource bounds before `the text is at most 1 MiB` | `T::refusals::a_text_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused` | `refusals.rs:460` accepts exactly MAX_TEXT_BYTES, checks body length; one extra byte yields exact TooLarge length/max, line 4, offset and absent column; multibyte case confirms bytes rather than character count. |
| YAML depth and alias refusal; split before `YAML nesting` | `T::refusals::nesting_over_thirty_two_levels_is_refused_before_descent`, `T::refusals::a_billion_laughs_document_is_refused_at_the_first_anchor` | `refusals.rs:498` accepts 32 maps, refuses 33 with typed depth/line/column and checks flow-list adds a level; line 555 checks first anchor name `a` and exact line/column. Anchor/alias/merge/tag variants have a separate bound test for final reread. |
| Markdown depth; before `Markdown block nesting` | `T::body_structure::block_nesting_of_sixty_four_levels_parses_and_sixty_five_is_refused` | `body_structure.rs:268` checks quoted, compact quoted, inline-list, indented-list and mixed quote/list 64/65 boundaries with typed errors and locations. |
| Link scanner bound; before `a link destination or title is at most 4096 bytes` | `T::links::a_destination_or_title_of_exactly_4096_bytes_is_read_and_one_more_is_refused` | `links.rs:175` checks 4096/4097 bare and angle destinations and quoted/single-quoted/parenthesized titles; bounds exclude delimiters and failures have exact ErrorKind/line. No clock or timing claim. |
| Closed coverage policy; keep exact named clause | `T::records::a_coverage_record_rejects_the_removed_policy_key`, `T::record_writer::a_coverage_record_writes_only_its_closed_policy_independent_keys` | `records.rs:711` validates unchanged neighbor, injects strictness values default/strict/strct/Strict/true/[], demands exactly UnknownKey at strictness. `record_writer.rs:176` pins exact writer bytes containing only type/id/scope/exclusions and reads equal typed CoverageRecord. This proves both refusal and actual writer production path. |
| Ceiling values/classes; split Ceilings and exceptions at `A row of exceptions` | `T::records::a_ceiling_names_a_crate_one_whole_number_of_lines_or_a_percent_and_an_optional_class`, `T::records::a_ceiling_of_another_shape_is_refused_with_its_rule` | `records.rs:845` checks absent lists, exact typed Lines/Percent/Class values and 0/100 boundaries. Line 888 checks bad class/case, neither/both limits, fraction/negative lines, out-of-range percent, malformed Number, unknown/missing row keys, invalid crate, duplicate crate+All (implicit versus explicit), and wrong list shape with exact rules/location. |
| Exception targets/reasons; before `A row of exceptions` | `T::records::an_exception_names_one_item_prefix_file_or_directory_and_its_reason`, `T::records::an_exception_of_another_shape_or_without_a_reason_is_refused_with_its_rule` | `records.rs:979` checks exact four typed targets and written round-trip syntax. Line 1019 checks crate-root/wildcard misuse, malformed item, absolute/drive/backslash/parent/dot/empty path, both/neither target, empty/missing/placeholder reason and duplicate target, matching exact rules at exceptions. |
| Coverage extension writer order; before `CoverageRecord::ceilings` | `T::record_writer::ceilings_and_exceptions_are_written_after_the_exclusions_and_read_back_equal` | `record_writer.rs:134` asserts complete output bytes including list order, absent all class, explicit test class, every target spelling and equal read-back; bare record omits both empty optional keys. |
| Typed debt rows; retain The debt record, split row rules before `It is validated into` | `T::records::a_valid_debt_record_validates_to_its_typed_value_and_an_empty_one_opts_out_visibly`, `T::records::a_row_of_findings_holds_exactly_a_kind_and_a_subject_each_pair_once`, `T::records::a_row_of_lines_holds_exactly_an_item_and_a_whole_number_of_lines_each_item_once`, `T::records::both_lists_of_a_debt_record_are_required` | `records.rs:1237` checks typed nonempty/empty record; line 1260 exact unknown/missing/duplicate finding pair, distinct-kind same-subject neighbor and wrong shape; line 1320 unknown/missing lines fields, negative/fraction/text/exponent/plus/empty orphans, duplicate item and valid zero; line 1377 requires each list. Singleton path and sorted writer need their own candidates below. |
| Validation gathers violations; retain Validate opening, split key rules afterwards | `T::profile::a_document_with_three_faults_returns_three_violations` | `profile.rs:1448` independently introduces bad state, bad capability id and empty Usage, requires exactly all three corresponding rule/location values. Does not assert every possible multi-fault combination. |
| Required keys and special resource key; before `the key set comes from one table per type` | `T::profile::every_required_key_of_every_type_is_reported_when_missing`, `T::profile::the_key_resource_has_its_own_rule_on_every_type` | `profile.rs:293` iterates every DocType and required key, accepts original, requires exact missing-key location/rule (explicitly handles changes causing a dependent second violation). Line 514 injects resource for every type, requires one ResourceKey finding and checks the actionable message names each allowed key, with clean neighbor. |
| Type selects schema; before `a missing type` | `T::profile::a_missing_type_key_is_the_only_violation`, `T::profile::an_unknown_type_is_one_violation_at_the_type_key` | `profile.rs:494` exact missing-type-only finding and scalar-shape error; line 532 unknown Concept and case-sensitive architecture produce one type finding, with Architecture neighbor valid. |
| Exact decimal numbers; split evidence rules before `values and limits` | `T::records::numbers_compare_exactly_by_sign_whole_part_and_fraction`, `T::records::a_value_over_its_limit_is_judged_on_its_text_and_not_its_nearest_float` | `records.rs:680` checks signed zero equality, zero padding/fractions, negative ordering, tiny decimal distinctions and huge exponents-as-decimal text; reverses every order and checks over/exact/under neighbors. The end-to-end evidence limit test is a candidate for final full reread. |
| Whole-text body coordinates; keep Body model opening, split before `The model holds` | `T::body_structure::spans_are_in_whole_text_coordinates_when_frontmatter_precedes_the_body`, `T::body_structure::markdown_without_frontmatter_is_the_whole_body`, `T::body_structure::sections_carry_heading_slug_and_content_up_to_the_next_second_level_heading`, `T::body_structure::a_section_that_ends_the_body_without_a_final_newline_reaches_the_end` | `body_structure.rs:20,42,53,68` assert exact section text slices, heading slugs, line ranges, whole-text frontmatter-offset equality between Document and Body parse, no-frontmatter span/text, ignored fenced link and unclosed leading rule treated as body. |
| Tested-by mapping; split before `tested-by markers` | `T::tested_by::a_tested_by_marker_names_its_tests_and_maps_the_paragraph_above_it`, `T::tested_by::a_tested_by_marker_maps_a_list_item_a_heading_or_the_statement_of_its_claim_marker` | `tested_by.rs:41` checks exact ordered paths, line, marker bytes, statement bytes/lines, section and absence from prose/claims. Line 68 checks list item, heading, same claim statement, indentation and generic-argument comma not splitting path. |
| Tested-by syntax and attachment refusal; split malformed-marker paragraph before `a malformed tested-by marker` | `T::tested_by::a_malformed_tested_by_marker_is_refused_with_its_line`, `T::tested_by::a_tested_by_marker_with_no_statement_directly_above_is_refused` | `tested_by.rs:113` checks space-after-comma/repeat/empty/attribute/spacing/case/invalid item path/quote/midline forms, multiline marker typed error at opening line and valid neighbor. Line 153 checks empty/blank/comment/second-marker/before-claim/fence/proposal/generated attachment errors and valid direct/after-claim neighbors. |
| Marker context exclusions; split before `outside fenced code, code spans and PROPOSED blocks` | `T::tested_by::tested_by_markers_in_fences_code_spans_and_proposed_blocks_are_not_read` | `tested_by.rs:185` checks valid/malformed markers ignored in those three contexts and resolution of proposal text makes the marker active. Claim/generated exclusions have separate family-specific tests; do not infer all from tested-by alone. |
| Atomic body splice; before `Body::replace_spans` | `T::splice::two_generated_blocks_change_only_their_byte_ranges`, `T::splice::overlapping_spans_are_refused_and_the_body_is_unchanged`, `T::splice::a_malformed_result_is_refused_and_the_body_is_unchanged`, `T::splice::spans_outside_the_body_or_inside_a_character_are_refused` | `splice.rs:15` exact whole result and unchanged outside prefixes/suffixes; line 79 checks typed overlap, no mutation and same-point adjacent insertion positive; line 105 checks Reparse/OrphanGeneratedEnd and unchanged model; line 124 exact frontmatter/outside/reversed/UTF8-byte-boundary refusal and accepted whole-character neighbor. |
| Deterministic writer/key order; retain Write opening | `T::render::keys_out_of_order_render_in_table_order_and_then_stay_put`, `T::render::a_parsed_document_equals_its_rendering_read_back`, `T::render::tags_render_sorted_and_stay_sorted` | `render.rs:248` pins reordered output/comment movement, equal parse and stable render; line 449 compares output/input and read-back for capability/change/CRLF variants; line 420 pins sorted tags in flow/block style and another list preserving author order. |
| Equality semantics; split before `Two documents are equal` | `T::render::equality_compares_keys_as_a_map_with_comments_and_body_bytes` | `render.rs:462` proves key-order-independent map equality; changed value, trailing/leading/end comment, body byte and scalar style differ; fence newline layout does not affect equality. |
| Generated-only index; retain Index handling, split path rule from text grammar | `T::index::a_generated_only_index_is_accepted_with_either_header_form`, `T::index::prose_outside_the_blocks_is_refused_with_its_line` | `index.rs:22` exact single/multiline/both SPDX headers, no-header/two-block and CRLF acceptance with byte-identical render; line 43 rejects prose before/after/inside header at exact line, with typed IndexText/IndexHeader. Remaining index grammar candidates below. |
| No filesystem/process I/O; move from Constraints into a matching named clause if needed | `T::no_io_in_source::the_crate_sources_name_neither_the_file_system_nor_the_process_module` | `no_io_in_source.rs:112` enumerates current src Rust files, requires at least ten and no scanner-reported forbidden module line. This proves source policy for scanned spellings; it is not dynamic tracing of arbitrary dependency I/O. Scanner positive/negative self-tests are separately bound. |

## Additional meaningful clauses: exact candidates, assertions not fully reread

These paths are precise next reads, not evidence that this reviewer checked
their entire bodies. Keep their independent guarantees separate when adding
markers. A table row can expand into several named clauses rather than one
oversized marker-bearing paragraph.

| Insertion anchor / clause | Exact candidate marker paths |
|---|---|
| Parse YAML subset and number-as-text; before `The frontmatter accepts` | `T::refusals::numbers_booleans_and_null_are_read_as_strings`, `T::refusals::anchors_aliases_tags_and_merge_keys_are_refused_naming_the_token`, `T::refusals::a_duplicate_key_is_refused_at_every_level`, `T::refusals::block_scalars_are_refused_naming_the_header`, plus the actual flow/list/escape/layout boundary tests for each stated subset rule. |
| Document types/key table | `T::schema::each_type_lists_its_keys_in_the_order_of_the_architecture_page`, `T::schema::only_coverage_and_debt_are_singletons_and_only_the_four_records_are_records`. |
| Debt path, sorted construction and rendering | `T::records::debt_is_one_record_at_its_one_path`, `T::record_writer::a_debt_record_is_written_sorted_and_read_back_equal`, `T::record_writer::a_debt_record_with_no_debt_writes_two_empty_lists`, `T::record_writer::a_debt_row_the_profile_refuses_is_written_and_refused_when_it_is_read_back`. |
| Unknown/nested foreign keys | `T::profile::an_unknown_key_is_reported_at_that_key`, `T::profile::an_unknown_or_foreign_key_is_reported_on_every_type`, `T::profile::a_nested_unknown_key_is_reported_in_bindings_and_in_a_change_item`. |
| Document directory/id/title/state/bindings | `T::profile::a_document_must_sit_directly_in_the_directory_of_its_type`, `T::profile::an_id_needs_the_prefix_of_its_type_a_kebab_slug_and_the_file_name`, `T::profile::a_title_and_a_description_are_non_empty_strings`, `T::profile::a_state_is_one_of_the_four`, `T::profile::bindings_have_exactly_the_keys_code_tests_and_examples_each_a_list`, `T::profile::bindings_by_state_designed_and_considered_bind_nothing_delivered_binds_code_and_tests`. |
| Changes, affects, effects, item reason/outcomes | `T::profile::changes_is_a_non_empty_list_of_cap_ids`, `T::profile::affects_entries_have_the_form_a_section_of_the_seven_and_a_capability_in_changes`, `T::profile::contract_effect_is_none_or_changes`, `T::profile::items_need_a_claim_known_keys_and_a_known_outcome`, `T::profile::an_item_reason_is_a_non_empty_line_beside_any_outcome_and_an_empty_one_is_refused`. Do not make this profile require a dropped/moved reason merely because workplan requires it. |
| Tags/timestamps and body requirements | `T::profile::tags_are_non_empty_unique_strings_and_a_timestamp_is_rfc_3339`, `T::profile::a_capability_holds_the_seven_sections_in_order`, `T::profile::a_section_is_empty_when_only_comments_and_generated_blocks_remain`, `T::profile::a_change_needs_a_non_empty_summary`, `T::profile::an_architecture_body_is_not_empty`, `T::records::a_record_has_no_title_description_or_body`. |
| Proposals, claim/tested-by ownership and placeholders | `T::profile::a_proposed_block_is_valid_only_inside_a_section_of_a_capability`, `T::tested_by::a_tested_by_marker_belongs_in_a_capability_document`, actual claim-ownership test and `T::profile::placeholder_tokens_in_prose_are_reported_at_their_line`, `T::profile::placeholder_tokens_in_frontmatter_values_are_reported_at_the_key`, `T::placeholder_lines::reports_each_token_in_prose_with_its_line`. |
| Typed evidence and assessment | `T::records::a_valid_evidence_record_validates_to_its_typed_value`, `T::records::a_valid_assessment_validates_to_its_typed_value`, `T::records::a_verdict_is_one_of_the_verdicts_of_its_type`, `T::records::by_is_human_or_agent_and_never_a_name`, `T::records::a_digest_is_sixty_four_lowercase_hex_digits`, `T::records::a_revision_is_a_full_commit_id`. Subject/input matching and command/value/limit requirements need their corresponding bound records tests, not just happy-path record tests. |
| Coverage identity/scope/exclusions | `T::records::coverage_is_one_record_at_its_one_path`, `T::records::a_valid_coverage_record_validates_and_an_empty_scope_opts_out_visibly`, `T::records::scope_holds_crate_package_names`, `T::records::an_exclusion_names_an_item_or_a_module_prefix_below_a_crate_root`, `T::records::an_exclusion_has_a_reason_with_no_placeholder`, `T::records::a_list_that_holds_each_entry_once_refuses_a_repeat`. |
| Heading and section model | `T::body_structure::headings_of_every_level_carry_github_anchors_with_repeats_numbered`, `T::body_structure::only_atx_headings_are_headings_and_setext_headings_are_not`, `T::body_structure::headings_inside_fences_and_comments_are_not_headings`. |
| Link forms/targets/context/splits | `T::links::every_link_form_is_read_with_its_target_anchor_and_line`, `T::links::targets_resolve_against_the_document_path`, `T::links::targets_that_leave_the_root_or_are_empty_or_absolute_are_refused`, `T::links::a_link_split_across_lines_is_recorded_and_not_read`, plus the actual quote/HTML/code/comment negative cases. |
| Fences/prose/generated/proposed blocks | `T::body_structure::a_fence_closes_only_on_the_same_character_with_at_least_the_same_length`, `T::body_structure::fences_are_recognised_indented_up_to_three_spaces_and_inside_quotes_and_lists`, `T::body_structure::a_fenced_block_exposes_its_content_and_whether_it_is_closed`, `T::body_structure::prose_lines_skip_fences_and_comments_and_remove_code_spans`, `T::markers::markers_inside_a_fence_produce_no_blocks`, `T::markers::every_misspelling_of_the_proposed_marker_is_refused_with_its_line`, `T::markers::each_generated_marker_error_names_its_line`. |
| Claims/multiline raf comments | Bound `T::claims::*` specific statement/evidence/context/malformed/unattached/multiline tests. Insert separate `Claim mapping`, `Marker grammar`, `Multiline comment handling` headings; no wildcard in actual markers. |
| Record construction/writer | `T::record_writer::a_built_evidence_record_renders_to_a_record_that_reads_back_equal`, `T::record_writer::a_built_assessment_and_coverage_record_read_back_equal`, `T::record_writer::the_keys_are_written_in_the_order_of_the_key_table`, `T::record_writer::a_value_the_profile_refuses_is_written_and_refused_when_it_is_read_back`, `T::record_writer::a_value_the_frontmatter_cannot_hold_is_refused_by_the_writer`, `T::record_writer::verdict_and_subject_names_are_the_values_of_their_keys`. |
| Index special grammar/path/splice | `T::index::only_the_capability_index_path_is_the_index`, `T::index::document_parse_of_index_text_fails_at_line_one`, `T::index::indented_and_mid_line_markers_are_refused`, `T::index::a_header_with_unknown_tags_or_in_the_wrong_place_is_refused`, `T::index::an_empty_header_tag_is_refused`, `T::index::an_index_with_no_block_or_an_unclosed_block_is_refused`, `T::splice::the_catalogue_of_an_index_is_replaced_through_the_index`. |

## Constraints and proof limits

1. **Text only/no I/O**: use the inspected source policy scanner and retain its
   source-level scope; no blanket dynamic dependency behavior is inferred.
2. **Resource bounds**: reuse the individually mapped whole-text/YAML/Markdown/
   link boundary tests. `Body::parse` also has its own exact 1 MiB/one-extra-byte
   test at `body_structure.rs:421`, and Index its own size test. Keep these
   standalone callers explicit if a clause claims every parsing entry point.
3. **ATX/fence-only model and indented-code limitation**: use
   `T::body_structure::only_atx_headings_are_headings_and_setext_headings_are_not`
   and `T::body_structure::indented_code_is_ordinary_text_so_links_and_markers_in_it_count`.
   These are exact named candidates; reread their assertion bodies before final
   coverage assignment.
4. **Closed keys/no pass-through**: bind every-type unknown/foreign/nested-key
   negatives, and the inspected removed strictness-key negative for policy.
   Writing an invalid document with retained unknown keys does not mean profile
   validation accepts them: reader/writer and validation are distinct APIs.
5. **Valid/invalid only, no warnings**: all-violations test gives a concrete hard
   failure and validates its neighbor. The absence of a warning API is a source
   type property; one runtime negative alone does not prove that API fact.
6. **Comments preserved/body not reflowed**: inspected comment, complete byte
   render and splice result tests demonstrate it. Keep non-body frontmatter
   canonicalization distinct: tags/key ordering may change intentionally.
7. **No position on tense/content**: `T::profile::future_tense_prose_is_not_a_violation`
   is the precise candidate. Avoid the broader literal wording `no position on
   content` as if placeholders, nonempty titles and seven body sections were
   not enforced. The existing neighboring sentence clarifies reconcile owns
   prose semantics; retain that distinction.

The resource Contract says `never a panic, abort or unbounded allocation`.
Boundary fixtures and source checks support the specified limit/refusal paths;
they do not by themselves prove that universal quantifier for every byte input.
`alloc_bound.rs:39` is a serial counting-allocator test over exactly two refused
families: nine-level billion laughs and a roughly 900 KiB many-key document
ending in an alias. It asserts growth <= 8 × input bytes + 16 KiB fixed
allowance, checks exact Anchor/Alias errors, and avoids concurrent tests adding
to the peak. Do not describe that as a universal successful-document 8× memory
guarantee, and do not omit its fixed allowance. Deep-flow/list and quote-marker
tests exercise 100,000 markers on a 2 MiB thread but are still finite shapes.

If documenting those observed ratios or validation-at-cap counts, place their
precise measurements in Explanation with inputs, allocator allowance and limits.
The current Contract itself has no historic measurement catalogue requiring a
mass move. Keep mandatory input/depth/link bounds and hard refusal guarantees
in Contract. Missing proof is not permission to weaken them or attach a nearby
test unrelated to the asserted behavior.

Other distinctions needing source/compile evidence are exhaustive enum
compatibility promises, no-new-rule equivalence with the architecture page and
cross-document nonvalidation. Schema table tests prove actual table agreement;
they do not automatically prove every semantic rule in a large prose page has
an independent negative fixture. The body/record candidate rows above mark
that remaining assertion work honestly rather than manufacturing complete
coverage from a handful of convenient tests.
