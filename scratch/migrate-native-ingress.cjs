const fs=require('fs');
for(const p of ['crates/raf-workplan/src/report.rs','crates/cargo-workplan/src/result_graph.rs']) {
 let s=fs.readFileSync(p,'utf8');
 s=s.replace(/use raf_rdf::\{[^\n]+\};/, 'use raf_rdf::{Error, Format, Graph, TermBox, TermValue, Triple, vocab};');
 s=s.replaceAll('Atom::Iri(named(name)?)','named(name)').replaceAll('Atom::Iri(local(&id)?)','local(&id)').replaceAll('Atom::Iri(local(&proof)?)','local(&proof)');
 s=s.replaceAll('Atom::Iri(iri(name)?)','iri(name)');
 s=s.replaceAll('Atom::Iri(local(&id))','local(&id)');
 s=s.replaceAll('Atom::string(', 'TermValue::simple_literal(').replaceAll('Atom::integer(', 'raf_rdf::integer(');
 s=s.replace(/Atom::boolean\(([^()]*)\)/g, 'TermValue::typed_literal(($1).to_string(), vocab::BOOLEAN)');
 s=s.replaceAll('Atom::Iri(', 'TermValue::Iri(');
 s=s.replaceAll('Iri::absolute(vocab::TYPE)?','TermValue::iri(vocab::TYPE)').replaceAll('Iri::absolute(vocab::REIFIES)?','TermValue::iri(vocab::REIFIES)');
 s=s.replaceAll('value: impl Into<Object>', 'value: TermValue').replaceAll('atom: Atom','atom: TermValue').replaceAll('object: value.into()', 'object: value').replaceAll('object: atom.into()', 'object: atom');
 s=s.replace(/fn (named|iri)\(([^)]*)\) -> (?:Result<Iri, Error>|Answer<Iri>) \{\n    Iri::absolute\(format!\("\{\}\{(?:value|name)\}", vocab::REPORT\)\)\n\}/g, (_,fn,args)=>'fn '+fn+'('+args+') -> TermValue {\n    TermValue::iri(format!("{}{'+(fn==='named'?'value':'name')+'}", vocab::REPORT))\n}');
 s=s.replace(/fn local\(([^)]*)\) -> (?:Result<Iri, Error>|Answer<Iri>) \{\n    Iri::local\(([^)]*)\)\n\}/g, 'fn local($1) -> TermValue {\n    TermValue::iri($2)\n}');
 s=s.replace(/\b(named|local|iri)\(([^()\n]*)\)\?/g,'$1($2)');
 s=s.replaceAll('object: named(name).into()', 'object: named(name)');
 fs.writeFileSync(p,s);
}

