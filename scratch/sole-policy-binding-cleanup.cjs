const fs=require('fs');
function walk(p){return fs.readdirSync(p,{withFileTypes:true}).flatMap(x=>x.isDirectory()?walk(p+'/'+x.name):[p+'/'+x.name]);}
for(const p of walk('docs').filter(p=>p.endsWith('.md'))){let s=fs.readFileSync(p,'utf8'),n=s;
 n=n.replace(/^[ \t]*- (?:raf_okf::(?:Strictness|CoverageRecord::with_strictness)|raf_workplan::(?:Gate::strict(?:ness)?|Opened::strictness_floor|Workplan::(?:strictness|with_strictness)))\r?\n/gm,'');
 n=n.replace(/^[ \t]*- .*::only_the_two_kinds_of_strict_mode_are_strict\r?\n/gm,'');
 n=n.replaceAll('raf_rdf::example_crate::report::main','raf_rdf::example_crate::report');
 const split=n.indexOf('\n---\n',4);
 if(split>0){const header=n.slice(0,split).split('\n'),seen=new Set();n=header.filter(line=>{if(!/^\s*- (?:raf_|cargo_|rust_agentic)/.test(line))return true;if(seen.has(line))return false;seen.add(line);return true;}).join('\n')+n.slice(split);}
 if(n!==s)fs.writeFileSync(p,n);
}
