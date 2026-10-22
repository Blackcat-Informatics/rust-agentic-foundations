const fs=require('fs'),cp=require('child_process');
const files=cp.execFileSync('rg',['--files','crates','docs','tests','work'],{encoding:'utf8'}).trim().split('\n');
const mappings=new Map([
['json_is_one_new_change_object_with_its_fields','rdf_preserves_created_change_fields_and_ordered_guidance'],
['json_is_one_impact_object_with_its_lists','rdf_preserves_impact_bindings_dependencies_links_and_list_order'],
['json_is_one_obligation_object_for_each_obligation','rdf_preserves_each_obligation_and_optional_location_in_library_order'],
['json_holds_the_entries_of_the_table_in_the_same_order','rdf_preserves_full_history_commits_sources_paths_and_removed_bindings'],
['json_is_one_binding_object_for_each_binding','rdf_preserves_real_binding_outcomes_proof_counts_and_quiet_filtering'],
['the_table_and_the_json_carry_the_same_entries_escaped','the_table_and_rdf_preserve_the_same_entries_and_escape_their_carriers'],
['the_table_and_the_json_carry_the_same_entries_in_the_same_order','the_table_preserves_typed_entries_sources_and_removed_bindings_in_order'],
['a_format_that_is_not_text_or_json_is_a_usage_error','an_unknown_or_json_format_is_a_usage_error'],
['the_report_renders_as_three_tables_and_as_json_with_the_same_rows','the_report_preserves_exact_line_counts_ranges_and_three_tables'],
['an_excepted_unit_is_the_excepted_table_in_text_and_a_lines_excepted_object_in_json','an_excepted_unit_preserves_ranges_reason_and_the_excepted_table'],
['a_walk_that_fails_is_the_error_table_in_text_and_a_lines_error_object_in_json','a_walk_that_fails_retains_a_single_typed_error_and_the_error_table'],
]);
for(const p of files.filter(p=>p.endsWith('.rs')&&p.includes('cargo-workplan'))){const s=fs.readFileSync(p,'utf8');for(const m of s.matchAll(/\bfn (\w*json\w*)\(/g)){
 const name=m[1];if(name==='an_unknown_or_json_format_is_a_usage_error'||name.includes('json_is_rejected'))continue;
 mappings.set(name,name.replaceAll('json','rdf'));
}}
const removed=[
'a_json_string_escapes_quotes_backslashes_and_every_control_character',
'quiet_prints_only_findings_and_keeps_the_exit',
'a_json_object_begins_with_its_kind_and_keeps_the_order_of_its_fields',
'a_record_with_no_text_has_no_line_in_text_and_an_object_in_json',
'an_empty_report_is_nothing_in_text_and_an_empty_array_in_json',
'a_finding_makes_the_exit_one_and_an_outcome_does_not',
'a_rendered_block_is_printed_as_given_and_quiet_hides_only_an_outcome',
'text_with_quotes_and_control_characters_is_escaped',
];
for(const p of files.filter(p=>p.endsWith('.rs')||p.endsWith('.md'))){let s=fs.readFileSync(p,'utf8');for(const [old,name]of [...mappings].sort((a,b)=>b[0].length-a[0].length))s=s.replaceAll(old,name);
 if(p.startsWith('docs/'))s=s.split('\n').filter(line=>!line.includes('cargo_workplan::test_crate::json::')&&!removed.some(name=>line.includes('::'+name))).join('\n');
 fs.writeFileSync(p,s);
}
let p='docs/capabilities/workplan-cli.md',s=fs.readFileSync(p,'utf8');const additions=[
'cargo_workplan::bin_crate::cargo_workplan::render::tests::diagnostic_write_failure_is_propagated_and_quiet_does_not_write_notes',
'cargo_workplan::bin_crate::cargo_workplan::render::tests::quiet_filters_results_without_changing_exit_and_machine_output_stays_a_graph',
'cargo_workplan::test_crate::rdf::check_preserves_every_finding_location_in_both_equivalent_formats',
'cargo_workplan::test_crate::rdf::quiet_keeps_findings_and_an_empty_result_is_still_a_canonical_graph',
'cargo_workplan::test_crate::rdf::status_preserves_ordered_capabilities_binding_gaps_and_open_changes',
'cargo_workplan::test_crate::rdf::preview_preserves_reconciled_text_and_sync_reports_only_actual_rewrites',
'cargo_workplan::test_crate::rdf::failures_have_no_machine_output_and_json_is_rejected_before_root_loading',
];
s=s.replace('  tests:\n','  tests:\n'+additions.map(name=>'    - '+name+'\n').join(''));fs.writeFileSync(p,s);
p='docs/capabilities/rdf-report-profile.md';s=fs.readFileSync(p,'utf8').replace('    - raf_workplan::report\n','    - raf_workplan::report\n    - cargo_workplan::bin_crate::cargo_workplan::result\n    - cargo_workplan::bin_crate::cargo_workplan::result_graph\n');s=s.replace('  tests:\n','  tests:\n'+[
'rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_first_party_manifests_have_no_json_result_dependency',
'rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_dependency_refusal_covers_aliases_targets_workspace_and_every_dependency_class',
'rust_agentic_foundations::test_crate::repo_gates::dependencies::rdf_dependency_scan_refuses_malformed_manifests_instead_of_skipping_them',
].map(name=>'    - '+name+'\n').join(''));fs.writeFileSync(p,s);
