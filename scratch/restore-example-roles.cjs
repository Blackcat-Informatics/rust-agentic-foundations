const fs=require('fs'),cp=require('child_process');
for(const name of ['okf-profile','source-index','workplan-complete-gate','workplan-current-rationale','workplan-document-model','workplan-entity-history','workplan-reconcile','workplan-two-way-coverage']){
 const path='docs/capabilities/'+name+'.md';
 const original=cp.execFileSync('git',['show','HEAD:'+path],{encoding:'utf8'});
 const examples=original.match(/  examples:\n(?:    - [^\n]+\n)*/)?.[0];
 if(!examples)throw Error(path);
 let s=fs.readFileSync(path,'utf8');s=s.replace(/  examples:\n(?:    - [^\n]+\n)*/,examples);fs.writeFileSync(path,s);
}
