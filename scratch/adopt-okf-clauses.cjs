const fs = require('fs');
const path = 'docs/capabilities/okf-profile.md';
let text = fs.readFileSync(path, 'utf8');
const marker = names => '<!-- raf:tested-by ' + names.map(name => 'raf_okf::test_crate::' + name).join(',') + ' -->';
function replace(before, after) {
  if (!text.includes(before)) throw Error('Missing exact anchor: ' + before);
  text = text.replace(before, after);
}
function heading(before, title, tests) {
  replace(before, '### ' + title + (tests.length ? '\n' + marker(tests) : '') + '\n\n');
}
heading('**Parse.** ', 'Parse framing', ['refusals::a_document_without_a_fence_on_line_one_is_refused','refusals::an_unclosed_fence_is_refused_at_the_opening_fence','refusals::a_frontmatter_that_is_not_a_block_mapping_is_refused']);
heading('SPDX lines are YAML comments', 'Comment attachment', ['render::comments_attach_to_the_next_key_and_trailing_comments_stay_on_their_line']);
text = text.replace('### Comment attachment\n'+marker(['render::comments_attach_to_the_next_key_and_trailing_comments_stay_on_their_line'])+'\n\n', '### Comment attachment\n'+marker(['render::comments_attach_to_the_next_key_and_trailing_comments_stay_on_their_line'])+'\n\nSPDX lines are YAML comments');
heading('**Resource bounds.** ', 'Bounded parsing', []);
heading('**Document types.** ', 'Document types', ['schema::each_type_lists_its_keys_in_the_order_of_the_architecture_page','schema::only_coverage_and_debt_are_singletons_and_only_the_four_records_are_records']);
heading('**Closed coverage policy.** ', 'Closed coverage policy', ['records::a_coverage_record_rejects_the_removed_policy_key','record_writer::a_coverage_record_writes_only_its_closed_policy_independent_keys']);
heading('**Ceilings and exceptions.** ', 'Ceiling values and classes', ['records::a_ceiling_names_a_crate_one_whole_number_of_lines_or_a_percent_and_an_optional_class','records::a_ceiling_of_another_shape_is_refused_with_its_rule']);
replace('A row of `exceptions` is an', '\n\n### Exception targets and reasons\n'+marker(['records::an_exception_names_one_item_prefix_file_or_directory_and_its_reason','records::an_exception_of_another_shape_or_without_a_reason_is_refused_with_its_rule'])+'\n\nA row of `exceptions` is an');
replace('`CoverageRecord::ceilings` and\n', '\n\n### Coverage extension writer\n'+marker(['record_writer::ceilings_and_exceptions_are_written_after_the_exclusions_and_read_back_equal'])+'\n\n`CoverageRecord::ceilings` and\n');
heading('**The debt record.** ', 'Typed debt rows', ['records::a_valid_debt_record_validates_to_its_typed_value_and_an_empty_one_opts_out_visibly','records::a_row_of_findings_holds_exactly_a_kind_and_a_subject_each_pair_once','records::a_row_of_lines_holds_exactly_an_item_and_a_whole_number_of_lines_each_item_once','records::both_lists_of_a_debt_record_are_required','record_writer::a_debt_record_is_written_sorted_and_read_back_equal']);
heading('**Validate.** ', 'Positioned validation results', ['profile::a_document_with_three_faults_returns_three_violations']);
heading('**Body model.** ', 'Body model', []);
heading('**Write.** ', 'Deterministic document rendering', ['render::keys_out_of_order_render_in_table_order_and_then_stay_put','render::keys_outside_the_table_follow_in_input_order','render::a_document_of_an_unknown_type_keeps_input_order','render::tags_render_sorted_and_stay_sorted','render::a_parsed_document_equals_its_rendering_read_back','render::equality_compares_keys_as_a_map_with_comments_and_body_bytes']);
heading('**Writing records.** ', 'Typed record construction and writing', ['record_writer::a_built_evidence_record_renders_to_a_record_that_reads_back_equal','record_writer::a_built_assessment_and_coverage_record_read_back_equal','record_writer::the_keys_are_written_in_the_order_of_the_key_table','record_writer::a_value_the_profile_refuses_is_written_and_refused_when_it_is_read_back','record_writer::a_value_the_frontmatter_cannot_hold_is_refused_by_the_writer','record_writer::verdict_and_subject_names_are_the_values_of_their_keys']);
heading('**Index handling.** ', 'Generated-only index', ['index::a_generated_only_index_is_accepted_with_either_header_form','index::prose_outside_the_blocks_is_refused_with_its_line','index::only_the_capability_index_path_is_the_index','index::document_parse_of_index_text_fails_at_line_one','index::indented_and_mid_line_markers_are_refused','index::a_header_with_unknown_tags_or_in_the_wrong_place_is_refused','index::an_empty_header_tag_is_refused','index::an_index_with_no_block_or_an_unclosed_block_is_refused','splice::the_catalogue_of_an_index_is_replaced_through_the_index']);
text=text.replaceAll('[strict mode](./workplan-strict.md)','[complete policy](./workplan-strict.md)').replaceAll('[Strict mode](./workplan-strict.md)','[Complete policy](./workplan-strict.md)');
fs.writeFileSync(path,text);
