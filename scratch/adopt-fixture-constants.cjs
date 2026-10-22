const fs=require('fs');const body=fs.readFileSync('.stage/raf-workplan-strict-mode-force-coverage/raw/adoption-candidate-second/lines.md','utf8');
const owners={graph:'workplan-document-model',history:'workplan-entity-history',lifecycle:'workplan-reconcile',obligations:'workplan-reconcile',untouched:'workplan-reconcile',verify:'workplan-complete-gate'};const rows=new Map();
for(const m of body.matchAll(/^\| raf-workplan \| `([^`]+)` \| const \|/gm)){
 const name=m[1];const mod=name.match(/^raf_workplan::test_crate::([^:]+)/)?.[1];const owner=owners[mod];if(!owner)continue;
 if(!rows.has(owner))rows.set(owner,[]);rows.get(owner).push(name);
}
for(const[owner,names]of rows){const p='docs/capabilities/'+owner+'.md';let s=fs.readFileSync(p,'utf8');const end=s.indexOf('  tests:\n');const before=s.slice(0,end);const added=names.filter(n=>!before.split('\n').some(l=>l.trim()==='- '+n));s=before+added.map(n=>'    - '+n+'\n').join('')+s.slice(end);fs.writeFileSync(p,s);}
