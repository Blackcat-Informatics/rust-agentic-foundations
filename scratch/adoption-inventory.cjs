const fs=require('fs');
const record=fs.readFileSync('work/changes/repository-adoption.md','utf8');
const names=record.match(/^changes: \[(.*)\]$/m)[1].split(', ').map(x=>x.slice(4));
for(const name of names){const path='docs/capabilities/'+name+'.md';let s=fs.readFileSync(path,'utf8');s=s.replace(/^state: (delivered|implementing)$/m,'state: implementing');fs.writeFileSync(path,s);}
let s=fs.readFileSync('tests/docs_contract.rs','utf8');
let start=s.indexOf('const INVENTORY: Inventory'),end=s.indexOf('    changes:',start);
let caps=s.slice(start,end);for(const name of names)caps=caps.replace('("'+name+'", "delivered")','("'+name+'", "implementing")');
caps=caps.replace('        ("source-index",','        ("repository-contract", "implementing"),\n        ("source-index",');
s=s.slice(0,start)+caps+s.slice(end);
const at=s.indexOf('        (\n            "workplan-strict",',start);
if(at<0)throw Error('inventory marker');
const row='        (\n            "repository-adoption",\n            &[\n'+names.map(n=>'                "'+n+'",').join('\n')+'\n            ],\n        ),\n';
s=s.slice(0,at)+row+s.slice(at);
fs.writeFileSync('tests/docs_contract.rs',s);
