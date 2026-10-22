const fs=require('fs');
let p='crates/raf-rdf/src/term.rs',s=fs.readFileSync(p,'utf8');s=s.replace('use std::cmp::Ordering;\n','');
const a=s.indexOf('// The report profile orders'); const b=s.indexOf('/// A canonical signed',a); if(a<0||b<0)throw Error('term anchors');s=s.slice(0,a)+s.slice(b);fs.writeFileSync(p,s);
p='crates/raf-source/src/settle.rs';s=fs.readFileSync(p,'utf8');
s=s.replace('    let index = purrdf_core::graph::scc_component_index(successors);', '    let mut resident = purrdf_lex::allocation::Resident;\n    let mut memory = purrdf_lex::allocation::Memory::new(&mut resident);\n    let index = purrdf_core::graph::scc_component_index_with_memory(successors, &mut memory)\n        .expect("resident component index over a valid dense graph");');
s=s.replace('    let mut groups = vec![Vec::new(); count];\n    for (node, group) in index.into_iter().enumerate() { groups[group].push(node); }', '    let mut groups = Vec::new();\n    memory.reserve(&mut groups, count).expect("resident component groups");\n    groups.resize_with(count, Vec::new);\n    for (node, &group) in index.iter().enumerate() {\n        memory.reserve_for_push(&mut groups[group]).expect("resident component members");\n        groups[group].push(node);\n    }\n    memory.release_vec(index).expect("component index storage is released");');
fs.writeFileSync(p,s);

