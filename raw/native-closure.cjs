const fs=require('fs');
let cargo=fs.readFileSync('Cargo.toml','utf8');cargo=cargo.replace(/(purrdf-[a-z]+ = \{ )git/g,'$1version = "3.0.1", git');fs.writeFileSync('Cargo.toml',cargo);
for(const p of ['deny.toml','fuzz/deny.toml']){
 let s=fs.readFileSync(p,'utf8');s=s.replace('wildcards = "deny"','wildcards = "deny"\n# The native RDF parser includes wasm-bindgen, whose procedural macro uses syn 2.\n# RAF uses syn 3 for its source index; these major versions cannot unify.\nskip = [{ crate = "syn@=2.0.119", reason = "Pinned native RDF wasm-bindgen macro closure; RAF source parsing uses syn 3." }]');fs.writeFileSync(p,s);
}
const a=fs.readFileSync('.stage/raf-workplan-strict-mode-force-coverage/raw/purrdf-native-oracle/profile.ttl.nt','utf8');
const b=fs.readFileSync('.stage/raf-workplan-strict-mode-force-coverage/raw/purrdf-native-oracle/profile.nt.nt','utf8');
if(a!==b)throw new Error('Registry parser format graphs differ');
fs.writeFileSync('crates/raf-rdf/tests/fixtures/profile.oracle.nt',a);
