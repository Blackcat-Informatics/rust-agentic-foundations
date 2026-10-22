const fs=require('fs');
const text=fs.readFileSync('.stage/raf-workplan-strict-mode-force-coverage/raw/queue-adoption-code-account.log','utf8');
const owners={'raf-census':'census-normal-form','raf-okf':'okf-profile','raf-source':'source-index','raf-rdf':'rdf-report-profile'};
const additions=new Map();
for(const match of text.matchAll(/^\| public item \| ([^|]+) \| `([^`]+)`/gm)){
 const owner=owners[match[1].trim()];if(!owner)throw Error('unknown owner '+match[1]);
 if(!additions.has(owner))additions.set(owner,new Set());
 additions.get(owner).add(match[2].split('::').slice(0,2).join('::'));
}
for(const [owner,items]of additions){const path='docs/capabilities/'+owner+'.md';let s=fs.readFileSync(path,'utf8');const a=s.indexOf('  code:\n'),e=s.indexOf('  tests:',a);if(a<0||e<0)throw Error(path);const code=s.slice(a,e);const fresh=[...items].sort().filter(name=>!code.split('\n').some(line=>line.trim()==='- '+name));s=s.slice(0,e)+fresh.map(name=>'    - '+name+'\n').join('')+s.slice(e);fs.writeFileSync(path,s);process.stdout.write(owner+': '+fresh.join(', ')+'\n');}
