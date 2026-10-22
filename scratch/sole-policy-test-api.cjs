const fs=require('fs');
function walk(p){return fs.readdirSync(p,{withFileTypes:true}).flatMap(x=>x.isDirectory()?walk(p+'/'+x.name):[p+'/'+x.name]);}
const files=walk('crates').concat(walk('tests')).filter(p=>p.endsWith('.rs'));
for(const p of files){let s=fs.readFileSync(p,'utf8'),n=s;
 if(!p.includes('/tests/')&&!p.startsWith('tests/'))continue;
 n=n.replace(/, Strictness\b/g,'').replace(/\bStrictness, /g,'');
 n=n.replace(/\.with_strictness\([^)]*\)/g,'').replace(/\.strictness\([^)]*\)/g,'').replace(/\.strict\((?:Strictness::(?:Default|Strict)|strictness)?\)/g,'');
 n=n.replace(/, Strictness::(?:Default|Strict)\b/g,'').replace(/^\s*Strictness::(?:Default|Strict),\n/gm,'');
 n=n.replace(/, strictness: Strictness\b/g,'').replace(/^\s*strictness: Strictness,\n/gm,'').replace(/, floor: Strictness\b/g,'');
 n=n.replace(/, strictness\)/g,')').replace(/^\s*strictness,\n/gm,'');
 n=n.replaceAll('strictness: strict\\n','');
 if(n!==s)fs.writeFileSync(p,n);
}
