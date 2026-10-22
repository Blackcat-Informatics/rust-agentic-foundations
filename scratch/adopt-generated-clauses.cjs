const fs = require('fs');
const path = 'docs/capabilities/workplan-generated-docs.md';
let text = fs.readFileSync(path, 'utf8');
const marker = names => '<!-- raf:tested-by ' + names.map(name => 'raf_workplan::test_crate::' + name).join(',') + ' -->';
function replace(before, after) {
  if (!text.includes(before)) throw Error('Missing anchor: ' + before);
  text = text.replace(before, after);
}
function lead(label, title, tests) {
  replace('**' + label + '.** ', '### ' + title + (tests.length ? '\n' + marker(tests) : '') + '\n\n');
}
lead('Block syntax', 'Generated block syntax', ['generated::a_marker_fixture_for_each_error_nested_unclosed_orphan_end_and_unknown_kind']);
lead('Kinds', 'Generated kinds', []);
const kinds = [
  ['verification-matrix', 'Verification matrix rows', ['generated::a_verification_matrix_lists_each_test_and_example_by_path_with_its_sections','generated::rows_are_sorted_by_path_whatever_the_order_of_the_lists','generated::a_tested_by_marker_substantiates_its_section_and_an_unmarked_binding_keeps_its_row']],
  ['relationships', 'Relationship table rows', ['generated::a_relationship_table_lists_each_relation_sorted_by_slug_with_links','generated::a_capability_with_no_relation_has_a_table_of_the_header_only']],
  ['catalogue', 'Catalogue states and gaps', ['generated::the_catalogue_is_a_table_per_state_in_order_with_gaps_for_implementing_only','generated::a_gap_names_an_unresolved_binding_and_an_empty_list','generated::groups_with_no_capability_are_omitted_and_each_group_is_sorted_by_slug']],
];
for (const [kind,title,tests] of kinds) {
  const start=text.indexOf('- `'+kind+'`:');
  if(start<0)throw Error(kind);
  const ends=['\n- `','\n\n'].map(x=>text.indexOf(x,start+1)).filter(x=>x>=0);
  const end=Math.min(...ends);
  const clause=text.slice(start,end).replace(/^- /,'').replace(/\n  /g,'\n');
  text=text.slice(0,start)+'### '+title+'\n'+marker(tests)+'\n\n'+clause+'\n\n'+text.slice(end).trimStart();
}
replace('A block of a kind in the wrong place,', '### Placement and escaped cells\n'+marker(['generated::a_matrix_and_a_relationship_table_sit_in_a_section_of_a_capability_document','generated::a_catalogue_block_is_valid_only_in_the_capability_index','generated::a_matrix_or_a_relationship_table_in_the_index_is_misplaced','generated::blocks_in_a_change_record_or_an_architecture_page_are_misplaced_and_never_rewritten','generated::cell_text_is_escaped_the_same_way_every_time'])+'\n\nA block of a kind in the wrong place,');
lead('The index', 'Index creation and refusal', ['generated::a_missing_index_is_a_finding_and_sync_creates_it_with_the_spdx_lines','generated::the_spdx_text_of_a_new_index_is_required_and_must_fit_one_comment_line','generated::an_index_with_text_outside_its_blocks_is_rejected_whatever_else_it_holds']);
lead('Sync', 'Pure sync report and stable output', ['sync::the_report_is_a_pure_function_of_the_documents_and_is_sorted_by_path','sync::syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal','generated::text_outside_a_block_is_never_changed_and_a_file_without_blocks_is_untouched','generated::the_frontmatter_of_a_rewritten_document_is_kept_byte_for_byte','generated::a_crlf_document_stays_all_crlf_and_an_lf_document_all_lf']);
replace('Sync also stops while a\ncapability or change document does not load, since the tables that list it would be\nwrong.', '\n\n### Unloaded documents stop synchronization\n'+marker(['sync::a_document_that_does_not_load_stops_the_sync_since_the_tables_would_miss_it'])+'\n\nSync also stops while a capability or change document does not load, since the\ntables that list it would be wrong.');
lead('Staleness', 'Generated staleness and repair', ['generated::the_stale_finding_points_at_the_opening_marker_of_the_block','sync::generated_checks_are_clean_exactly_when_sync_has_nothing_to_write','sync::check_and_sync_agree_after_each_repair']);
lead('Preview', 'Preview selection and preserved text', ['preview::a_change_with_two_blocks_in_a_capability_reads_reconciled_and_other_blocks_stay','preview::another_change_reconciles_only_its_own_blocks','preview::the_slug_is_taken_with_or_without_the_prefix','preview::preview_is_deterministic','preview::a_crlf_document_keeps_its_line_endings','preview::a_block_in_a_list_item_keeps_its_indentation','preview::a_tab_after_the_quote_marker_is_kept_as_written','preview::a_nested_quote_loses_one_level_only','preview::an_unknown_slug_names_it_and_lists_the_open_changes','preview::a_change_record_that_did_not_load_says_why','preview::a_listed_capability_that_did_not_load_is_named_never_skipped']);
lead('Mechanical versus explanatory', 'Authored text preservation', ['generated::text_outside_a_block_is_never_changed_and_a_file_without_blocks_is_untouched']);
text=text.replaceAll('[strict mode](./workplan-strict.md)','[complete policy](./workplan-strict.md)');
fs.writeFileSync(path,text);
