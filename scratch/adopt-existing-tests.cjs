const fs=require('fs');
const data=fs.readFileSync('.stage/raf-workplan-strict-mode-force-coverage/raw/adoption-candidate/tests.md','utf8');
const owners={'raf-census':'census-normal-form','raf-okf':'okf-profile','raf-source':'source-index'};
const rows=new Map();
for(const match of data.matchAll(/^\| test \| ([^|]+) \| `([^`]+)`/gm)){
 const owner=owners[match[1].trim()];if(!owner)throw Error('review required for '+match[1]);
 if(!rows.has(owner))rows.set(owner,new Set());rows.get(owner).add(match[2]);
}
for(const [owner,tests]of rows){const path='docs/capabilities/'+owner+'.md';let s=fs.readFileSync(path,'utf8');const a=s.indexOf('  tests:\n'),end=s.indexOf('  examples:',a);if(a<0||end<0)throw Error(path);const existing=s.slice(a,end);const added=[...tests].sort().filter(name=>!existing.split('\n').some(line=>line.trim()==='- '+name));s=s.slice(0,end)+added.map(name=>'    - '+name+'\n').join('')+s.slice(end);fs.writeFileSync(path,s);process.stdout.write(owner+': '+added.length+' actual tests\n');}
