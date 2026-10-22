const fs=require('fs');
const usage={
'census-normal-form':'Run `raf_census::example_crate::baseline` to collect a workspace and emit its canonical baseline report.',
'okf-profile':'The runnable API example `raf_okf::EvidenceRecord::to_document` constructs the document model before canonical Markdown serialization.',
'rdf-report-profile':'Run `raf_rdf::example_crate::report` to serialize the same typed ground graph in either supported canonical format.',
'source-index':'The runnable API example `raf_source::Workspace::exported_items` loads a memory-backed workspace and inspects its supported public surface.',
'workplan-cli':'The runnable API example `raf_workplan::Workplan::new_change` shows the typed change creation that the command wrapper renders and writes.',
'workplan-complete-gate':'The runnable API examples in `raf_workplan::bound` exercise resolution of the tests and examples that the completion gate executes.',
'workplan-current-rationale':'The runnable API examples in `raf_workplan::rationale` demonstrate the heading and list rules used to check current architectural reasoning.',
'workplan-document-model':'The runnable API example `raf_workplan::Gate` opens a repository through the same source and document boundary used by command callers.',
'workplan-entity-history':'The runnable API example `raf_workplan::Workplan::history` queries the supported Git-backed history view from a loaded workplan.',
'workplan-evidence-claims':'The runnable API example `raf_workplan::evidence::parse_limit` validates the execution limit used when recording command evidence.',
'workplan-generated-docs':'Run `raf_workplan::example_crate::docs_sync` to inspect or rewrite the generated documentation blocks through the production sync API.',
'workplan-prose-assessment':'The runnable API example `raf_workplan::assessment::record_slug` derives the stable record name used for a prose assessment.',
'workplan-reconcile':'The runnable API example `raf_workplan::Workplan::reconcile` obtains obligations from a loaded workplan and its merge-base comparison.',
'workplan-two-way-coverage':'The runnable API example `raf_workplan::Workplan::coverage_report` obtains the supported item and test directions from a loaded workplan.',
};
for(const [name,text]of Object.entries(usage)){
 const path='docs/capabilities/'+name+'.md';let s=fs.readFileSync(path,'utf8');
 if(!s.includes('## Usage\n\n'))throw Error(name);
 s=s.replace('## Usage\n\n','## Usage\n\n'+text+'\n\n');
 fs.writeFileSync(path,s);
}
const p='docs/capabilities/workplan-generated-docs.md';let s=fs.readFileSync(p,'utf8');s=s.replace('A relationship table after a sync:', 'The relationship table below links the current dependencies and dependants to their capability documents. Sync computes those rows from the loaded graph and replaces only the generated block:');fs.writeFileSync(p,s);
