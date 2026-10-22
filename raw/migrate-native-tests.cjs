const fs = require('fs');
function edit(p,f){fs.writeFileSync(p,f(fs.readFileSync(p,'utf8')));}
const profile='crates/raf-rdf/tests/profile.rs';
edit(profile,s=>s.replace('Atom, Error, Format, Graph, Iri, Object, Statement, Triple','Error, Format, Graph, TermBox, TermValue, Triple')
.replace(/fn absolute[\s\S]*?fn example\(\)/,`fn absolute(value: &str) -> TermValue { TermValue::iri(value) }
fn local(value: &str) -> TermValue { TermValue::iri(value) }
fn triple(subject: &str, object: TermValue) -> Triple {
 Triple { subject: local(subject), predicate: absolute("urn:predicate"), object }
}
fn statement(subject: TermValue, predicate: TermValue, object: TermValue) -> TermValue {
 TermValue::Triple { s: TermBox::new(subject), p: TermBox::new(predicate), o: TermBox::new(object) }
}
fn example()`)
.replace(/Atom::string/g,'TermValue::simple_literal').replace(/Atom::integer/g,'raf_rdf::integer')
.replace('Atom::boolean(true)','TermValue::typed_literal("true", vocab::BOOLEAN)')
.replace(/Atom::Iri\((local|absolute)\(([^)]*)\)\)/g,'$1($2)')
.replace(/Object::Atom\((local\("#absent"\))\)/g,'$1')
.replace(/Atom::Literal \{\s*lexical: ([^\n]+)\.into\(\),\s*datatype: absolute\(([^\n]+)\),\s*\}/g,'TermValue::typed_literal($1, $2)')
.replace(/Atom::Literal \{\s*lexical: "value"\.into\(\),\s*datatype,\s*\}/g,'TermValue::typed_literal("value", datatype_iri(&datatype))')
.replace(/Object::Triple\(Statement \{\s*subject: ([^\n]+),\s*predicate: ([^\n]+),\s*object: ([^\n]+),?\s*\}\)/g,(_,a,b,c)=>`statement(${a}, ${b}, ${c.replace(/,$/,'')})`)
.replace('.cloned()','.map(|t| Triple { subject: absolute(t.subject), predicate: absolute(t.predicate), object: t.object.clone() })')
.replace('assert!(Iri::absolute(value).is_err(), "{value:?}");','assert!(Graph::new().insert(triple("", absolute(value))).is_err(), "{value:?}");')
.replace('assert_eq!(Iri::absolute(value).unwrap().as_str(), value);','assert!(Graph::new().insert(triple("", absolute(value))).is_ok());')
.replace('Object::Atom(Atom::Iri(local("#absent")))','local("#absent")')
.replace('Atom::Iri(absolute(&external))','absolute(&external)')
.replace(/Object::Atom\(Atom::Literal \{ lexical, datatype \}\)/g,'TermValue::Literal { lexical_form: lexical, datatype, .. }')
.replace(/Object::Atom\(Atom::Literal \{ lexical, \.\. \}\)/g,'TermValue::Literal { lexical_form: lexical, .. }')
.replace('Object::Triple(Statement { object: Atom::Literal { lexical, datatype }, .. }) if lexical == "123" && datatype.as_str().ends_with("#integer")','TermValue::Triple { o, .. } if matches!(o.as_ref(), TermValue::Literal { lexical_form, datatype, .. } if lexical_form == "123" && datatype.ends_with("#integer"))')
.replace('fn example()','fn datatype_iri(value: &TermValue) -> &str { match value { TermValue::Iri(s) => s, _ => unreachable!() } }\nfn example()'));
edit('crates/cargo-workplan/tests/support/rdf.rs',s=>s.replace('Atom, Format, Graph, Iri, Object','Format, Graph, TermValue').replace(/&'a Iri/g,"&'a str").replace(/triple\.predicate\.as_str\(\)/g,'triple.predicate').replace(/t\.(subject|predicate)\.as_str\(\)/g,'t.$1').replace('id: &triple.subject','id: triple.subject').replace(/&t\.subject == self.id/g,'t.subject == self.id').replace(/Object::Atom\(Atom::Iri\(iri\)\)/g,'TermValue::Iri(iri)').replace(/Object::Atom\(Atom::Literal \{ lexical, datatype \}\)/g,'TermValue::Literal { lexical_form: lexical, datatype, .. }').replace('(lexical.as_str(), datatype)','(lexical.as_str(), datatype.as_str())').replace(/datatype.as_str\(\)/g,'datatype').replace('self.id.as_str()','self.id').replace('(lexical.as_str(), datatype)','(lexical.as_str(), datatype.as_str())'));
edit('crates/raf-workplan/tests/rdf_reports.rs',s=>s.replace('Atom, Format, Graph, Object','Format, Graph, TermValue').replace(/t\.predicate\.as_str\(\)/g,'t.predicate').replace(/Object::Atom\(Atom::Literal \{ lexical, \.\. \}\)/g,'TermValue::Literal { lexical_form: lexical, .. }').replace('Object::Triple(statement) if matches!(&statement.object, Atom::Literal { lexical, .. } if lexical == "demo::generated::Made")','TermValue::Triple { o, .. } if matches!(o.as_ref(), TermValue::Literal { lexical_form, .. } if lexical_form == "demo::generated::Made")'));
edit('crates/cargo-workplan/src/result_graph/tests.rs',s=>s.replace('t.object == Atom::Iri(iri("LinesError").unwrap()).into()','t.object == &iri("LinesError")'));
for (const p of ['crates/raf-census/tests/forms.rs','crates/raf-census/examples/selection.rs']) edit(p,s=>s.replace(/blake3::hash\(([^\n]*?)\)\.to_hex\(\)/g,'purrdf_hash::hex::encode(purrdf_hash::blake3::hash($1).as_bytes())').replace('blake3::hash(&bytes)','purrdf_hash::blake3::hash(&bytes)').replace(/expected.to_hex\(\).as_str\(\)/g,'purrdf_hash::hex::encode(expected.as_bytes())'));
