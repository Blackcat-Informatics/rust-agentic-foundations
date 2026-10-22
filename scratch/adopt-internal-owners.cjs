const fs=require('fs');
const owners={
'workplan-cli':['cargo_workplan::bin_crate::cargo_workplan::command::LABEL','cargo_workplan::test_crate::sync::MATRIX'],
'okf-profile':['raf_okf::work','raf_okf::test_crate::bounds::Counting','raf_okf::test_crate::bounds::ALLOCATOR','raf_okf::test_crate::bounds::CEILING','raf_okf::test_crate::index::SPDX_ID','raf_okf::test_crate::index::SPDX_COPY','raf_okf::test_crate::tested_by::MARKER'],
'source-index':['raf_source::item','raf_source::settle','raf_source::work','raf_source::error::tests::HOSTILE_PATH','raf_source::test_crate::corpus_replay::support::shapes::PAD','raf_source::test_crate::corpus_replay::support::shapes::REFS','raf_source::test_crate::exported_items::OUTSIDE','raf_source::test_crate::workspace_seeds::workspace_input::MANIFEST','raf_source_fuzz::bin_crate::load_workspace','raf_source_fuzz::bin_crate::workspace_input'],
'census-normal-form':['raf_source_fuzz::bin_crate::census_normalize','raf_source_fuzz::bin_crate::census_report_read'],
'workplan-generated-docs':['raf_workplan::generated::END_MARKER'],
'workplan-strict':['raf_workplan::work','raf_workplan::test_crate::bindings::MAKE','raf_workplan::test_crate::claim_coverage::MARKER','raf_workplan::test_crate::obligations::EXCLUSIONS'],
'workplan-prose-assessment':['raf_workplan::test_crate::assessments::support::window::ACCURATE'],
'workplan-complete-gate':['raf_workplan::test_crate::complete::EXCLUSIONS','raf_workplan::test_crate::complete::MUL','raf_workplan::test_crate::complete::FRAME','raf_workplan::test_crate::complete::FRAME_LIB'],
'workplan-document-model':['raf_workplan::test_crate::documents::SPDX_ID','raf_workplan::test_crate::documents::SPDX_COPY'],
'workplan-evidence-claims':['raf_workplan::test_crate::evidence::REVIEW'],
};
for(const [owner,paths]of Object.entries(owners)){
 const path='docs/capabilities/'+owner+'.md';let s=fs.readFileSync(path,'utf8');const a=s.indexOf('  code:\n'),end=s.indexOf('  tests:\n',a);if(a<0||end<0)throw Error(path);
 const existing=s.slice(a,end);const added=paths.filter(name=>!existing.split('\n').some(line=>line.trim()==='- '+name));
 s=s.slice(0,end)+added.map(name=>'    - '+name+'\n').join('')+s.slice(end);fs.writeFileSync(path,s);
}
