const fs=require('fs');
const source='docs/capabilities/source-index.md';
let text=fs.readFileSync(source,'utf8'); const owners=new Set(); const T='raf_source::test_crate::';
function names(file,prefixes){const body=fs.readFileSync('crates/raf-source/tests/'+file+'.rs','utf8');return prefixes.map(prefix=>{const found=[...body.matchAll(/fn (\w+)\(/g)].map(m=>m[1]).filter(n=>n.startsWith(prefix));if(found.length!==1)throw Error(prefix+': '+found);return T+file+'::'+found[0];});}
function marker(paths){paths.forEach(p=>owners.add(p));return paths.length?'<!-- raf:tested-by '+paths.join(',')+' -->\n':'';}
function add(anchor,title,paths=[],replace=false){if(!text.includes(anchor))throw Error('missing '+anchor);text=text.replace(anchor,'\n\n### '+title+'\n'+marker(paths)+'\n'+(replace?'':anchor));}
function item(anchor,paths){const start=text.indexOf('- '+anchor);if(start<0)throw Error(anchor);let end=text.indexOf('\n',start);text=text.slice(0,end)+'\n  '+marker(paths).trim()+text.slice(end);}
text=text.replace('### Disk lookup failure boundary','### Linux descriptor failures\n'+marker(['raf_source::tree::disk::tests::a_missing_descriptor_link_is_an_io_failure_and_never_absence']));
add('`entry`\nreturns `OutsideRoot`','Entry lookup boundaries',['raf_source::tree::disk::tests::strategy::every_strategy_reports_entry_kinds_and_refuses_an_outside_directory']);
add('Nothing else in the\ncrate opens a file.','Disk opening ownership');
add('**Path containment.** ','Path containment',names('workspace_containment',['a_symlinked_module_file_to_outside','a_manifest_target_path_that_leaves']),true);
add('`DiskTree` refuses any file that is not a regular file','Regular-file boundaries',['raf_source::tree::disk::tests::strategy::every_strategy_refuses_a_fifo_and_a_directory_as_a_file_without_blocking','raf_source::tree::disk::tests::strategy::every_strategy_lists_a_directory_and_refuses_a_symlink_a_file_and_an_outside_link']);
add('A read, a listing and an entry lookup each open a handle','Verified-handle operations',['raf_source::tree::disk::tests::swap::a_directory_swapped_for_an_outside_symlink_after_the_check_is_refused','raf_source::tree::disk::tests::swap::a_directory_swapped_for_an_outside_symlink_before_a_listing_is_refused','raf_source::tree::disk::tests::swap::an_entry_whose_directory_is_swapped_for_an_outside_symlink_is_refused','raf_source::tree::disk::tests::swap::the_same_read_without_a_swap_is_accepted','raf_source::tree::disk::tests::swap::the_same_listing_without_a_swap_returns_the_entries','raf_source::tree::disk::tests::swap::the_same_entry_without_a_swap_is_found']);
add('**Resource bounds.** ','File admission and parser refusals',[],true);
add('The lexer skips comments','Prescan lexical exclusions',names('prescan_budgets',['tokens_inside_strings','a_comment_or_string_that_looks','a_lifetime_is_not']));
add('A file that exceeds a budget is','Typed prescan and syntax errors',names('prescan_budgets',['the_refusal_names_the_line','a_syntax_error_reports_its_own']));
add('The budgets are one table in the source','Prescan counter definitions');
add('**Work and memory bounds.** ','Directory and aggregate budget ownership',[],true);
add('Four byte budgets and a work budget bound one load.','Configured budget defaults',names('load_budgets',['the_default_budgets_are_the_stated_limits']));
add('- `max_total_bytes`','Byte-budget charge ownership');
add('Parsing then runs on a dedicated thread','Parser stack and universal admission guarantee');
add('**Mod-tree following.** ','Module directories and path attributes',names('module_tree',['a_flat_file_and_a_directory','a_non_mod_file_keeps','a_module_with_both_name','inline_modules_add','a_path_attribute_in_a_mod_rs','a_path_attribute_in_a_non_mod','a_path_attribute_inside_an_inline','a_path_attribute_on_an_inline_module_replaces','a_file_named_by_a_path_attribute','two_unconditional_path']),true);
add('A path in\n`#[cfg_attr','Conditional module paths',names('module_tree',['every_cfg_attr_path_alternative','the_default_file_is_also','a_cfg_attr_test_path']));
add('Files are told apart by their identity','Module identity and cycles',names('module_tree',['a_module_cycle_is_an_error','a_file_reached_twice_without']));
add('The facts of a\nfile stay inside the crate','Owned file facts');
add('**`cfg(test)` tagging.** ','Test-role matrix',names('module_tree',['the_cfg_test_matrix','a_cfg_test_module_tags']),true);
add('A `cfg`, `cfg_attr`, `path` or `test` attribute','Malformed source attributes',names('module_tree',['a_malformed_cfg_path','an_unparseable_cfg','a_non_literal_path_attribute']));
add('**Use-alias index.** ','Written import forms',names('use_resolution',['a_rename_a_nested','crate_self_and_super','super_above_the_crate','extern_crate_renames','a_use_group_with_leading','a_leading_colon_member','a_crate_root_inside_a_group']),true);
add('From edition 2018 on, a path whose first segment','Edition and dependency import heads',names('use_resolution',['a_use_path_of_edition_2015','a_crate_name_in_an_import','a_dev_dependency_is_named_only']));
add('An import never resolves through its own binding','Import self-shadow and settlement');
add('A glob brings in only the names the','Glob visibility and settlement');
add('**Resolution from every target.** ','Target-local resolution',names('target_resolution',['a_helper_resolves_from_an_integration','a_helper_resolves_from_a_nested','an_example_a_bench']),true);
add('The crate path that starts the module path is found','Target-origin lookup cost',['raf_source::resolve::tests::a_module_path_of_fifty_thousand_segments_is_read_once_to_find_its_crate']);
add('A path read there starts as code of that target starts it','Target extern preludes',names('target_resolution',['a_helper_resolves_from_an_integration','an_example_a_bench']));
add('A file that several targets include','Shared-file target origins',names('target_resolution',['a_support_module_that_two']));
add('A module path that no target holds','Invalid and ambiguous target origins',names('target_resolution',['a_module_path_that_no_target','a_module_of_the_library_at']));
add('**API surface.** ','Public API construction and extension guarantees',names('memory_tree',['entry_reports_file_dir_and_absence']).concat(names('symbol_index',['files_items_in_and_item_at_answer'])).concat(names('exported_items',['doc_hidden_items_are_left_out'])).concat(names('reachability',['the_answer_does_not_depend_on_the_order'])),true);
item('Parsing is syntactic.',names('symbol_index',['bodiless_functions_and_types']).concat(names('use_resolution',['missing_names_dependency'])));
item('Resolution covers items the source defines.',names('use_resolution',['missing_names_dependency']));
item('Every opened path must lead inside',names('workspace_containment',['a_symlinked_module_file_to_outside','a_manifest_target_path_that_leaves','a_fifo_named_as_a_module_file','a_target_root_that_is_a_fifo']));
item('A file larger than `max_file_bytes`',names('read_cap',['memory_tree_enforces_the_cap','disk_tree_enforces_the_cap']).concat(names('prescan_budgets',['neighbouring_valid_shapes','delimiter_nesting_is_refused','chains_that_overflowed','the_recursion_limit_is_half'])).concat(names('module_tree',['module_nesting_beyond','a_chain_of_files_that_each_declare'])).concat(names('walk_stack',['a_directory_at_the_depth_limit'])).concat(names('load_budgets',['the_default_budgets','the_bytes_read_at_the_total','index_records_at_the_item','the_fact_budget_counts','glob_tables_at_the_glob'])).concat(names('overflow_harness',['every_committed_overflow_shape','a_deliberate_overflow_aborts','the_judge_rejects'])));
item('In edition 2015 a `use` declaration inside a function body',names('reachability',['an_edition_2015_body_import']));
item('An impl block inside `const _: ()',names('reachability',['impls_inside_const_and_function_blocks']));
item('An item that syn returns as tokens',names('symbol_index',['bodiless_functions_and_types','a_bodiless_const_or_another']).concat(names('use_resolution',['a_use_group_with_leading'])));
item('Output order is sorted by path',names('symbol_index',['two_loads_are_equal_and_items']).concat(names('exported_items',['the_surface_is_the_same'])));
item('The text of an error shows input-derived text',['raf_source::error::tests::every_user_derived_field_of_every_display_is_escaped','raf_source::resolve::tests::every_user_derived_field_of_every_resolve_error_is_escaped','raf_source::path::display_tests::control_characters_in_a_refused_path_are_escaped']);
item('Delimiters:',names('prescan_budgets',['delimiter_nesting_is_refused']));
item('Nested `<`:',names('prescan_budgets',['five_hundred_nested_generics','neighbouring_valid_shapes']));
item('Prefix operators:',names('prescan_budgets',['four_thousand_bangs','twenty_thousand_reference','a_lifetime_between_reference']));
item('Closure bars:',names('prescan_budgets',['ten_thousand_closure_bars','closure_heads_with_parameters']));
item('`else if`:',names('prescan_budgets',['twenty_thousand_else_if','a_block_as_the_condition','separate_if_chains']));
item('Recursion:',names('prescan_budgets',['every_operator_counts','every_counted_keyword','a_label_a_use_path','a_level_returns_to_zero','a_construct_that_continues','a_macro_call_counts','chains_that_overflowed','the_recursion_refusal']));
// Split Self restrictions so a direct-edge proof cannot imply non-reachability.
text=text.replace('A `<Self as Tr>::name` or `<S>::name` path\n  names nothing past its `>`, and an impl block inside a function body is read as\n  tokens of the function, its `Self` as that of the block that holds the function.',
  '\n  '+marker(names('reachability',['self_paths_and_methods_called_on_self','self_in_a_trait_and_in_its_impl_blocks','a_variant_through_self'])).trim()+'\n- Qualified `<Self as Tr>::name` and `<S>::name` suffixes add no direct member\n  reference edge; types named inside the qualification can still reach members\n  through containment.\n  '+marker(names('reachability',['qualified_self_suffixes_are_not_member_edges'])).trim()+'\n- An impl block inside a function body is read as tokens of the function, its\n  `Self` as that of the block that holds the function.');
item('An impl block inside a function body is read as tokens',names('reachability',['nested_impl_self_is_read_in_the_enclosing']));
const nativeOwners=['raf_source::meter::tests::native_storage_preserves_the_item_limit_and_cumulative_charges','raf_source::meter::tests::native_storage_capacity_failure_has_its_original_error_source','raf_source::reach::format::identity::tests::native_propagation_visits_cycles_and_duplicate_edges_once_per_active_source','raf_source::reach::format::identity::tests::native_propagation_spill_refuses_without_refunding_the_cumulative_grant'];
text=text.replace('- Format capture inference covers', '- Format identity construction admits owned buffers and worklist spills through\n  the original cumulative item-bytes account; releasing scratch does not refund\n  cumulative charges, and storage failures preserve their original cause.\n  '+marker(nativeOwners).trim()+'\n- Format capture inference covers');
const formatOwner=names('format_reach',['a_cfg_alternative_sibling_alias']);
item('Format capture inference covers',formatOwner);
const boundary=text.indexOf('\n  examples:'); if(boundary<0)throw Error('frontmatter examples');
const missing=[...owners].filter(p=>!text.slice(0,boundary).includes('    - '+p+'\n')).sort();
if(missing.length)text=text.slice(0,boundary)+'\n'+missing.map(p=>'    - '+p).join('\n')+text.slice(boundary);
text=text.replace(/[ \t]+\n/g,'\n').replace(/\n{3,}/g,'\n\n');
fs.writeFileSync('.stage/raf-workplan-strict-mode-force-coverage/tasks/source-claim-candidate.md',text);
console.log('Candidate: '+owners.size+' owners, '+missing.length+' missing bindings.');
