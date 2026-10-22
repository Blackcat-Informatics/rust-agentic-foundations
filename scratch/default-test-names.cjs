const fs=require('fs');
const names={
strict_check_exits_zero_on_the_strict_clean_repository:'the_default_check_accepts_the_complete_fixture',
strict_check_exits_one_on_one_thin_section_and_check_without_the_flag_exits_zero:'the_default_check_refuses_a_thin_section_without_a_policy_flag',
strict_check_exits_two_at_a_root_without_capability_documents:'the_default_check_without_capability_documents_is_a_run_failure',
check_without_the_flag_judges_strict_when_the_record_says_strict:'a_removed_policy_key_is_refused_beside_the_default_content_refusal',
a_strict_finding_in_rdf_is_an_object_of_its_kind_with_the_fields_of_a_finding:'policy_findings_keep_typed_locations_and_messages_in_rdf',
strict_refuses_a_completion_that_the_default_gate_accepts:'the_default_completion_refuses_an_underexplained_section_and_accepts_its_neighbor',
under_strict_an_untouched_usage_of_a_none_change_exits_one_and_its_edit_zero:'an_untouched_usage_of_a_none_change_refuses_until_edited',
under_strict_a_bound_source_file_that_does_not_parse_at_the_merge_base_is_exit_two:'unreadable_bound_source_at_the_merge_base_is_refused_beside_a_parsed_neighbor',
under_strict_a_bound_package_whose_surface_cannot_be_listed_is_exit_two:'an_unreadable_bound_surface_is_refused_beside_a_resolved_neighbor',
a_marker_that_names_a_path_the_capability_does_not_bind_is_a_finding_under_either_strictness:'an_unbound_claim_marker_is_refused_beside_covered_neighbors',
an_invalid_row_is_a_finding_under_either_strictness:'an_invalid_debt_row_is_refused_without_a_policy_selector',
};
function walk(p){return fs.readdirSync(p,{withFileTypes:true}).flatMap(e=>e.isDirectory()?walk(p+'/'+e.name):[p+'/'+e.name]);}
const replace=s=>{for(const[a,b]of Object.entries(names))s=s.replaceAll(a,b);return s;};
for(const p of ['crates','docs','tests','work'].flatMap(walk).filter(p=>/\.(rs|md)$/.test(p))){const s=fs.readFileSync(p,'utf8');const n=p.endsWith('.md')?s.split(/(<!-- raf:generated:[\s\S]*?<!-- raf:generated:end -->)/).map((part,i)=>i%2?part:replace(part)).join(''):replace(s);if(n!==s)fs.writeFileSync(p,n);}
