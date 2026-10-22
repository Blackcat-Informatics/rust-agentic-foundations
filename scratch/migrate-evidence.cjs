const fs=require('fs'),root='crates/cargo-workplan/';
function replaceFunction(s,name,body){const a=s.indexOf('fn '+name+'(');if(a<0)throw Error(name);let b=s.indexOf('\n///',a);if(b<0)b=s.length;return s.slice(0,a)+body+'\n'+s.slice(b);}
let p=root+'src/evidence.rs',s=fs.readFileSync(p,'utf8').replace('use serde_json::{Value, json};','use crate::result::Result as CommandResult;').replace('CapabilitySection, DocType, EvidenceVerdict, Measure','CapabilitySection, DocType, EvidenceVerdict').replace('ClaimStatement, EvidenceRequest, Failed, Recorded, Support','ClaimStatement, EvidenceRequest, Recorded, Support');
s=replaceFunction(s,'measure','');s=replaceFunction(s,'failed','');s=replaceFunction(s,'recorded_record',`fn recorded_record(recorded:&Recorded)->Record {
 let result=CommandResult::EvidenceRecorded(recorded.clone());
 if recorded.failed.is_empty() { Record::outcome(result,recorded.to_string()) } else { Record::finding(result,recorded.to_string()) }
}`);
s=s.replace(/"evidence-claim",\n        vec!\[[\s\S]*?\n        \],/,'CommandResult::EvidenceClaim { evidence:evidence.into(), claim:claim.clone() },');
s=s.replace(/"evidence-for",\n        vec!\[[\s\S]*?\n        \],/,'CommandResult::EvidenceSupport { capability:capability.into(), support:entry.clone() },');fs.writeFileSync(p,s);
p=root+'src/history.rs';s=fs.readFileSync(p,'utf8').replace('use crate::render::{Class, Report};','use crate::render::{Class, Report};\nuse crate::result::Result as CommandResult;').replace('Report::rendered(','Report::block(').replace('render::json(&history),','CommandResult::History(history),');fs.writeFileSync(p,s);
p=root+'src/coverage.rs';s=fs.readFileSync(p,'utf8').replace('use crate::render::{Class, Report};','use crate::render::{Class, Report};\nuse crate::result::Result as CommandResult;').replaceAll('Report::rendered(','Report::block(').replace('render::json(&report, kind),','CommandResult::Coverage { report, direction:kind },').replace('render::claims_json(&report),','CommandResult::Claims(report),').replace('render::lines_json(&report),','CommandResult::Lines(report),');fs.writeFileSync(p,s);
p=root+'src/debt.rs';s=fs.readFileSync(p,'utf8').replace('use serde_json::Value;','use crate::result::Result as CommandResult;').replace(/"debt-written",\n                vec!\[[\s\S]*?\n                \],/,'CommandResult::DebtWritten(written),');fs.writeFileSync(p,s);
p=root+'src/cli.rs';s=fs.readFileSync(p,'utf8').replaceAll('Format::Json','Format::Turtle').replaceAll('"json"','"turtle"');fs.writeFileSync(p,s);
