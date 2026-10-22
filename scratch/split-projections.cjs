const fs=require('fs'),p='crates/cargo-workplan/src/result_graph.rs';let s=fs.readFileSync(p,'utf8');
const specs=[
['Finding','finding_fields','finding: &raf_workplan::Finding','finding'],
['State','state_fields','state: &str, capabilities: &[raf_workplan::RoadmapEntry]','state, capabilities'],
['OpenChanges','open_changes_fields','changes: &[raf_workplan::OpenChange]','changes'],
['NewChange','new_change_fields','change: &raf_workplan::lifecycle::NewChange','change'],
['Impact','impact_fields','impact: &raf_workplan::lifecycle::Impact','impact'],
['Refusal','refusal_fields','failure: &raf_workplan::lifecycle::Failure, repeated: bool','failure, *repeated'],
['Binding','binding_fields','evidence: &raf_workplan::bound::Evidence, message: &str','evidence, message'],
['EvidenceClaim','claim_fields','evidence: &str, claim: &evidence::ClaimStatement','evidence, claim'],
['EvidenceSupport','support_fields','capability: &str, support: &evidence::Support','capability, support'],
['AssessmentRecorded','assessed_fields','assessed: &assessment::Assessed','assessed'],
['Assessment','assessment_fields','state: &assessment::AssessmentState','state'],
['Claims','claims_fields','report: &raf_workplan::strict::ClaimsReport','report'],
];
for(const [variant,name,params,args] of specs){
 const start=s.indexOf('        CommandResult::'+variant+(variant==='State'||variant==='Refusal'||variant==='Binding'||variant==='EvidenceClaim'||variant==='EvidenceSupport'?' {':'('));
 if(start<0) throw Error(variant);const a=s.indexOf('=> {',start)+3;let i=a,depth=0,quoted=false,escaped=false;
 for(;i<s.length;i++){const c=s[i];if(quoted){if(escaped)escaped=false;else if(c==='\\')escaped=true;else if(c==='"')quoted=false;continue;}if(c==='"'){quoted=true;continue;}if(c==='{')depth++;if(c==='}'&&--depth===0)break;}
 let body=s.slice(a+1,i).replace(/^        /gm,'');
 if(variant==='Refusal')body=body.replaceAll('*repeated','repeated');
 if(variant==='Binding')body=body.replaceAll('message.as_str()','message');
 if(variant==='EvidenceSupport')body=body.replaceAll('capability.as_str()','capability');
 s=s.slice(0,a)+name+'(g, id, '+args+')?,'+s.slice(i+1);
 // Replace the arm's removed opening brace as part of its expression.
 s=s.replace('=> '+name,'=> '+name);
 s+='\nfn '+name+'(g: &mut Graph, id: &str, '+params+') -> Answer<()> {'+body+'\n    Ok(())\n}\n';
}
s=s.replaceAll('&measure.value.text()','measure.value.text()').replaceAll('&v.text()','v.text()').replaceAll('.map(|s| s.slug())','.map(raf_okf::CapabilitySection::slug)').replaceAll('.map(|v| v.name())','.map(raf_okf::EvidenceVerdict::name)').replaceAll('.map(|c| c.as_str())','.map(raf_workplan::exec::Commit::as_str)');fs.writeFileSync(p,s);
for(const [name,obsolete] of [['assess','/// The name of how an input moved, as the JSON form writes it.\n\n/// A moved input as a JSON object.\n\n'],['command','/// A capability as a JSON object.\n\n'],['evidence','/// A measure as a JSON object; the number is its decimal text, never a float.\n\n/// Why a verdict is `fail`, as a JSON object.\n\n']]){const path='crates/cargo-workplan/src/'+name+'.rs';fs.writeFileSync(path,fs.readFileSync(path,'utf8').replace(obsolete,''));}
const path='crates/cargo-workplan/src/change.rs';fs.writeFileSync(path,fs.readFileSync(path,'utf8').replace('path: removed.to_string()','path: removed.clone()').replace('path: path.to_string()','path: path.clone()'));
