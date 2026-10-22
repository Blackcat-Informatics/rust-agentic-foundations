const fs = require('fs');
const path = 'docs/capabilities/workplan-entity-history.md';
let text = fs.readFileSync(path, 'utf8');
const used = new Set();
function marker(names) {
  const paths = names.map(name => name.startsWith('raf_') || name.startsWith('cargo_')
    ? name : 'raf_workplan::test_crate::history::' + name);
  for (const path of paths) used.add(path);
  return '<!-- raf:tested-by ' + paths.join(',') + ' -->';
}
function heading(label, title, tests) {
  const anchor = '**' + label + '.** ';
  if (!text.includes(anchor)) throw Error(anchor);
  text = text.replace(anchor, '### ' + title + '\n' + marker(tests) + '\n\n');
}
function before(anchor, title, tests) {
  if (!text.includes(anchor)) throw Error(anchor);
  text = text.replace(anchor, '\n\n### ' + title + '\n' + marker(tests) + '\n\n' + anchor);
}
heading('The command', 'History entries and order', [
  'each_kind_of_id_has_the_history_of_its_file',
  'a_capability_history_holds_its_document_through_a_move_and_its_bound_code',
  'every_question_is_asked_in_order_as_exactly_its_argv',
  'since_reaches_no_process_and_keeps_each_entry_by_its_author_date',
]);
heading('Document history', 'Document moves and merge resolutions', [
  'a_capability_history_holds_its_document_through_a_move_and_its_bound_code',
  'a_merge_whose_resolution_changed_the_document_is_in_its_history_and_a_clean_one_is_not',
  'every_question_is_asked_in_order_as_exactly_its_argv',
]);
heading('Binding history', 'Symbol span history', [
  'a_capability_history_holds_its_document_through_a_move_and_its_bound_code',
  'a_binding_that_names_no_single_symbol_is_refused_before_any_process',
  'a_symbol_log_that_git_refuses_names_the_binding_and_the_file_to_commit',
]);
before('A binding that resolves to a module below its crate root', 'Whole module history', [
  'a_module_binding_lists_the_commits_of_its_files_and_not_of_a_sibling',
  'a_module_binding_asks_its_span_and_then_the_log_of_each_of_its_files_in_order',
]);
before('A file of the module that no commit holds', 'Uncommitted module failure', [
  'a_module_file_that_is_not_committed_names_the_binding_and_the_file_until_it_is',
]);
before('A binding that no longer resolves is reported', 'Removed binding lookup', [
  'a_capability_history_holds_its_document_through_a_move_and_its_bound_code',
  'every_question_is_asked_in_order_as_exactly_its_argv',
  'raf_workplan::git::log::tests::the_pickaxe_answers_one_record_with_its_parent_or_none',
]);
heading('Invocation', 'Validated Git invocation', [
  'every_question_is_asked_in_order_as_exactly_its_argv',
  'an_id_or_a_date_that_cannot_be_asked_about_starts_no_process',
  'stubs::a_path_named_like_an_option_reaches_git_after_the_separator_or_the_colon',
  'an_old_git_or_a_shallow_answer_stops_before_any_history_is_read',
  'cargo_workplan::test_crate::history::a_shallow_clone_is_exit_two_and_its_full_clone_answers',
]);
before('Each record of the first two forms opens', 'Strict log records', [
  'raf_workplan::git::log::tests::a_subject_keeps_every_byte_but_a_newline',
  'raf_workplan::git::log::tests::output_that_is_not_a_log_of_the_form_is_an_error',
  'raf_workplan::git::log::tests::a_symbol_log_lists_no_path_and_refuses_any_other_line',
]);
before('The command reads at most 16 MiB', 'Whole log admission', [
  'a_log_past_sixteen_mebibytes_is_an_error_and_the_bound_itself_is_read',
]);
heading('Output', 'Table output and terminal safety', [
  'the_table_preserves_typed_entries_sources_and_removed_bindings_in_order',
  'raf_workplan::history::render::tests::a_table_cell_cannot_break_the_table_or_reach_the_terminal',
]);
before('`--format turtle` and `--format ntriples` emit typed RDF', 'Typed history output', [
  'cargo_workplan::test_crate::history::rdf_preserves_full_history_commits_sources_paths_and_removed_bindings',
]);
for (const path of used) {
  if (!text.slice(0, text.indexOf('\n---\n')).includes('    - ' + path + '\n')) {
    text = text.replace('  examples:\n', '    - ' + path + '\n  examples:\n');
  }
}
fs.writeFileSync(path, text.replace(/[ \t]+$/gm, ''));
