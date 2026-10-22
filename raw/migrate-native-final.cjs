const fs=require('fs');function edit(p,f){fs.writeFileSync(p,f(fs.readFileSync(p,'utf8')))}
edit('fuzz/deny.toml',s=>{let base=fs.readFileSync('deny.toml','utf8');const extra=s.slice(s.indexOf('# `libfuzzer-sys`'),s.indexOf('# purrdf-lex'));return base.replace('# purrdf-lex',extra+'# purrdf-lex')});
edit('crates/raf-workplan/src/digest.rs',s=>s.replace('hex(&hasher.finalize().into())','hex(&hasher.finalize())'));
edit('crates/raf-census/tests/reports.rs',s=>s.replace('Atom, Format, Graph, Iri, Object','Format, Graph, TermValue').replace(/fn property\(name: &str\) -> Iri \{\n    Iri::absolute\(format!\("\{\}\{name\}", vocab::REPORT\)\).unwrap\(\)\n\}/,'fn property(name: &str) -> TermValue { TermValue::iri(format!("{}{name}", vocab::REPORT)) }')
.replace('original.triples().cloned()','original.triples().map(|t| Triple { subject: TermValue::iri(t.subject), predicate: TermValue::iri(t.predicate), object: t.object.clone() })')
.replace(/Iri::local\(""\).unwrap\(\)/g,'TermValue::iri("")').replace(/Iri::absolute\("([^"\n]*)"\).unwrap\(\)/g,'TermValue::iri("$1")')
.replace(/Object::Atom\(Atom::Iri\(property\("(\w+)"\)\)\)/g,'property("$1")').replace(/Atom::Iri\(property\("(\w+)"\)\)/g,'property("$1")')
.replace(/Atom::integer/g,'raf_rdf::integer').replace(/Atom::string/g,'TermValue::simple_literal')
.replace('.find(|t| t.predicate == property("body"))','.find(|t| t.predicate == format!("{}body", vocab::REPORT))')
.replace('.unwrap()\n        .clone();','.map(|t| Triple { subject: TermValue::iri(t.subject), predicate: TermValue::iri(t.predicate), object: t.object.clone() }).unwrap();')
.replace('repeated.object = Atom::Iri(\n        Iri::absolute(format!(','repeated.object = TermValue::iri(format!(').replace('        ))\n        .unwrap(),\n    )\n    .into();','        ));')
.replace('(!t.subject.as_str().starts_with("#match/")).then_some(t)','(!matches!(&t.subject, TermValue::Iri(s) if s.starts_with("#match/"))).then_some(t)'));
edit('crates/raf-workplan/src/coverage/report_projection_tests.rs',s=>s.replace('use raf_rdf::{Atom, Object};','use raf_rdf::{TermBox, TermValue};').replace(/fn collect_terms\([\s\S]*?\n#\[test\]/,`fn collect_terms(graph: &raf_rdf::Graph, terms: &mut std::collections::BTreeSet<String>) {
 fn iri(value: &str, terms: &mut std::collections::BTreeSet<String>) { if let Some(name) = value.strip_prefix(raf_rdf::vocab::REPORT) { terms.insert(name.into()); } }
 for t in graph.triples() {
  iri(t.subject, terms); iri(t.predicate, terms);
  let _ = t.object.visit_terms(|v| { match v { TermValue::Iri(v) => iri(v,terms), TermValue::Literal {datatype,..} => iri(datatype,terms), _=>{} } std::ops::ControlFlow::<()>::Continue(()) });
 }
}
#[test]`).replace('Graph, Iri, Statement, Triple','Graph, Triple').replace('Iri::absolute(format!("{}{name}", vocab::REPORT)).unwrap()','TermValue::iri(format!("{}{name}", vocab::REPORT))').replace(/Iri::local\(""\).unwrap\(\)/g,'TermValue::iri("")')
.replace(/Atom::Literal \{\s*lexical: "value".into\(\),\s*datatype: named\("(\w+)"\),\s*\}/g,'TermValue::typed_literal("value", format!("{}$1", vocab::REPORT))')
.replace(/Atom::Iri\(named\("(\w+)"\)\)/g,'named("$1")')
.replace(/Object::Triple\(Statement \{\s*subject: named\("subject"\),\s*predicate: named\("predicate"\),\s*object: ([^\n]+),\s*\}\)/g,'TermValue::Triple { s: TermBox::new(named("subject")), p: TermBox::new(named("predicate")), o: TermBox::new($1) }')
.replace(/t.predicate.as_str\(\)/g,'t.predicate').replace('t.object == Object::Atom(Atom::string(name))','t.object == &TermValue::simple_literal(name)').replace('.map(|t| &t.object)','.map(|t| t.object)').replace('Object::Atom(Atom::string(p))','TermValue::simple_literal(p)'));
