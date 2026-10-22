const fs=require('fs');const p='docs/capabilities/workplan-current-rationale.md';let s=fs.readFileSync(p,'utf8');
const t='raf_workplan::test_crate::rationale::';
function mark(anchor,tests){if(!s.includes(anchor))throw Error(anchor);s=s.replace(anchor,anchor+'\n<!-- raf:tested-by '+tests.map(x=>t+x).join(',')+' -->');}
// The markers attach to the exact opening paragraph of each existing clause.
mark('depends on the decision links to it.',['a_decision_page_linked_from_a_capability_with_its_conditions_passes','an_unlinked_page_is_reported_by_name_and_a_link_to_itself_does_not_count']);
mark('architecture page that records no decision needs no such list.',['a_decision_page_without_conditions_to_revisit_is_reported_by_name']);
mark('`### Decision`.',['an_explanation_decision_without_conditions_is_reported_and_with_them_passes','a_decision_page_without_conditions_to_revisit_is_reported_by_name','an_empty_conditions_list_is_reported_and_a_list_item_with_text_passes']);
mark('PROPOSED block.',['headings_match_exactly_at_their_level','headings_and_list_items_in_fenced_code_or_proposed_blocks_do_not_count','an_empty_conditions_list_is_reported_and_a_list_item_with_text_passes']);
mark('[history command](./workplan-entity-history.md) on the page or the capability.',['an_empty_conditions_list_is_reported_and_a_list_item_with_text_passes']);
mark('**Checks.** `cargo workplan check` runs these rules over the loaded documents:',['a_link_from_a_capability_a_change_or_another_page_links_the_page','no_page_is_reported_unlinked_while_a_document_that_may_link_it_does_not_load','a_capability_link_to_a_missing_page_or_heading_fails_the_link_check','the_findings_are_the_same_on_every_run_in_file_and_line_order']);
const start=s.indexOf('**Current rationale and historical deliberation.**');const end=s.indexOf('**Architecture pages.**',start);if(start<0||end<0)throw Error('authorship table');const authorship=s.slice(start,end);s=s.slice(0,start)+s.slice(end);
const rationaleConstraints='- A decision an architecture page records is resolved. The page does not narrate\n  how the decision was reached.\n- Rationale keeps only constraints that still govern. A constraint that was removed\n  leaves the page.\n- Rationale sits next to the contract it explains. It is not collected in a separate\n  history document.\n';
if(!s.includes(rationaleConstraints))throw Error('author constraints');s=s.replace(rationaleConstraints,'');
mark('  been met; a person judges it with the history command.',['an_empty_conditions_list_is_reported_and_a_list_item_with_text_passes']);
mark('  wording or at another level is not a decision or a conditions heading.',['headings_match_exactly_at_their_level']);
s=s.replace('## Explanation\n\n','## Explanation\n\n'+authorship+'The authoring policy keeps the distinction explicit:\n\n'+rationaleConstraints+'\n');
fs.writeFileSync(p,s);
