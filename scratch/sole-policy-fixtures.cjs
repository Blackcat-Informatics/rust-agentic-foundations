const fs=require('fs');
function edit(p,f){const s=fs.readFileSync(p,'utf8'),n=f(s);if(s===n)throw Error(p);fs.writeFileSync(p,n);}
edit('crates/raf-workplan/src/debt/tests.rs',s=>s.replaceAll('strictness: strict\\n',''));
edit('crates/raf-workplan/src/coverage/lines/tests.rs',s=>s.replaceAll('| Crate | Enforced |','| Crate |').replaceAll('| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |','| --- | --- | --- | --- | --- | --- | --- | --- | --- |'));
edit('crates/raf-okf/tests/schema.rs',s=>s.replace('            "strictness",\n','').replace('matches!(spec.key, "strictness" | "ceilings" | "exceptions")','matches!(spec.key, "ceilings" | "exceptions")'));
edit('crates/raf-workplan/tests/claim_coverage.rs',s=>s.replace('    assert_eq!(judged(&text), []);\n    let found = judged(&text);','    let found = judged(&text);').replace('    assert_eq!(judged(&long), []);\n    // The neighbours:', '    // The neighbours:'));
for(const p of ['crates/cargo-workplan/tests/check.rs','crates/cargo-workplan/tests/complete.rs','crates/cargo-workplan/tests/reconcile.rs','crates/cargo-workplan/tests/claims.rs','crates/cargo-workplan/tests/debt.rs','crates/cargo-workplan/tests/lines.rs'])edit(p,s=>s.replaceAll('"--strict", ','').replaceAll(', "--strict"',''));
edit('crates/cargo-workplan/tests/lines.rs',s=>s.replace(/^\s*assert!\(!?(?:rows\[0\]|package)\.boolean\("enforced"\)\);\n/gm,'').replaceAll('| Crate | Enforced |','| Crate |').replaceAll('| demo | yes |','| demo |').replaceAll('| demo | no |','| demo |').replaceAll('| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |','| --- | --- | --- | --- | --- | --- | --- | --- | --- |'));
edit('crates/raf-workplan/tests/complete.rs',s=>s.replace('assert_eq!(merge_base_reads(&default), Vec::<String>::new());','assert_eq!(merge_base_reads(&default), files);'));
edit('crates/raf-workplan/src/report.rs',s=>s.replace('    "enforced",\n',''));
edit('docs/capabilities/rdf-report-profile.md',s=>s.replace('`enforced` (boolean), ',''));
