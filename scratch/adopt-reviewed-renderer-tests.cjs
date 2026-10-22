const fs=require('fs');
const domain=['cargo_workplan::test_crate::evidence::recorded_binding_failures_keep_failed_and_unresolved_outcomes_distinct','cargo_workplan::test_crate::assess::moved_inputs_preserve_raw_unresolved_reasons_only_for_unresolved_changes'];
const cases={
'rdf-report-profile':domain,
'workplan-cli':[...domain,'cargo_workplan::bin_crate::cargo_workplan::render::tests::diagnostic_flush_failure_propagates_and_quiet_never_touches_that_sink','cargo_workplan::test_crate::process::help_and_version_output_failures_exit_two_beside_successful_output'],
};
for(const[owner,names]of Object.entries(cases)){const p='docs/capabilities/'+owner+'.md';let s=fs.readFileSync(p,'utf8');const a=s.indexOf('  tests:\n'),end=s.indexOf('  examples:\n',a);const existing=s.slice(a,end);const added=names.filter(n=>!existing.split('\n').some(l=>l.trim()==='- '+n));s=s.slice(0,end)+added.map(n=>'    - '+n+'\n').join('')+s.slice(end);fs.writeFileSync(p,s);}
const p='docs/capabilities/workplan-prose-assessment.md';let s=fs.readFileSync(p,'utf8');s=s.replace('evidence record a claim marker of the section names. \n','evidence record a claim marker of the section names.\n');fs.writeFileSync(p,s);
