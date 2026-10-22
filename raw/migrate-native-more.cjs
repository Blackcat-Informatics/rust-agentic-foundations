const fs=require('fs');function edit(p,f){fs.writeFileSync(p,f(fs.readFileSync(p,'utf8')))}
edit('crates/raf-rdf/examples/report.rs',s=>s.replace('Atom, Format, Graph, Iri, Object, Statement, Triple','Format, Graph, TermBox, TermValue, Triple').replace(/Iri::(?:local|absolute)\(([^\n]+?)\)\?/g,'TermValue::iri($1)').replace(/Atom::string/g,'TermValue::simple_literal').replace(/Atom::integer/g,'raf_rdf::integer').replace('Atom::boolean(true)','TermValue::typed_literal("true", vocab::BOOLEAN)').replace('Atom::Literal {\n            lexical: "\\tcustom".into(),\n            datatype: TermValue::iri("urn:example:datatype"),\n        }','TermValue::typed_literal("\\tcustom", "urn:example:datatype")').replace('Atom::Iri(proof.clone()).into()','proof.clone()').replace('Object::Triple(Statement {\n            subject: root,\n            predicate,\n            object: raf_rdf::integer(123)?,\n        })','TermValue::Triple { s: TermBox::new(root), p: TermBox::new(predicate), o: TermBox::new(raf_rdf::integer(123)?) }'));
edit('crates/cargo-workplan/tests/registry.rs',s=>s.replace('Atom, Format, Graph, Iri, Object','Format, Graph, TermValue').replace(/fn terms\([\s\S]*?\nfn observe/,`fn terms(graph: &Graph, found: &mut BTreeSet<String>) {
 fn iri(value: &str, found: &mut BTreeSet<String>) {
  if let Some(name) = value.strip_prefix(vocab::REPORT) { found.insert(name.into()); }
 }
 for triple in graph.triples() {
  iri(triple.subject, found); iri(triple.predicate, found);
  let _ = triple.object.visit_terms(|term| {
   match term { TermValue::Iri(v) => iri(v, found), TermValue::Literal { datatype, .. } => iri(datatype, found), _ => {} }
   std::ops::ControlFlow::<()>::Continue(())
  });
 }
}
fn observe`));
