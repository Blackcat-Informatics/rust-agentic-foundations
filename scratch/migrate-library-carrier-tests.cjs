const fs=require('fs');
function replaceFn(s,name,body){const a=s.indexOf('fn '+name+'(');if(a<0)throw Error(name);let i=s.indexOf('{',a),depth=0,quoted=false,escaped=false;for(;i<s.length;i++){const c=s[i];if(quoted){if(escaped)escaped=false;else if(c==='\\')escaped=true;else if(c==='"')quoted=false;continue;}if(c==='"'){quoted=true;continue;}if(c==='{')depth++;if(c==='}'&&--depth===0)break;}return s.slice(0,a)+body+s.slice(i+1);}
let p='crates/raf-workplan/tests/coverage.rs',s=fs.readFileSync(p,'utf8');s=s.replace('assert_eq!(render::json(&first, kind), render::json(&second, kind));',`for format in [raf_rdf::Format::Turtle, raf_rdf::Format::NTriples] {
            assert_eq!(raf_workplan::report::coverage(&first, kind, format).unwrap(),raf_workplan::report::coverage(&second, kind, format).unwrap());
        }`);
let a=s.indexOf('    assert_eq!(\n        render::json(&report, GapKind::PublicItem),'),b=s.indexOf('    // A surface error',a);
s=s.slice(0,a)+`    for kind in [GapKind::PublicItem,GapKind::Test] {
        let graph=raf_workplan::report::coverage_graph(&report,kind).unwrap();
        for format in [raf_rdf::Format::Turtle,raf_rdf::Format::NTriples] {
            let text=graph.to_text(format).unwrap();
            assert_eq!(raf_rdf::Graph::from_text(&text,format).unwrap(),graph);
        }
        if kind==GapKind::Test {
            assert!(graph.triples().any(|t| t.predicate.as_str().ends_with("#reason") && matches!(&t.object,raf_rdf::Object::Atom(raf_rdf::Atom::Literal { lexical,.. }) if lexical=="Internals | <!-- not a comment --> [x]")));
        }
    }
`+s.slice(b);
a=s.indexOf('    let json = render::json(&other,');b=s.indexOf('\n}\n',a);
s=s.slice(0,a)+`    assert_eq!(other.errors().len(),1);
    assert_eq!(other.errors()[0].package(),"other");
    assert_eq!(other.errors()[0].root(),"crates/other/src/lib.rs");
    let graph=raf_workplan::report::coverage_graph(&other,GapKind::PublicItem).unwrap();
    assert!(graph.triples().any(|t|t.predicate.as_str().ends_with("#failure") && t.object==raf_rdf::Object::Atom(raf_rdf::Atom::string("unresolved"))));
    let empty=raf_workplan::report::coverage_graph(&CoverageReport::default(),GapKind::Test).unwrap();
    assert!(!empty.triples().any(|t|t.predicate.as_str().ends_with("#result")));
`+s.slice(b);s=s.replace('fn the_table_and_the_json_carry_the_same_entries_escaped()','fn the_table_and_rdf_preserve_the_same_entries_and_escape_their_carriers()');fs.writeFileSync(p,s);
p='crates/raf-workplan/tests/history.rs';s=fs.readFileSync(p,'utf8');s=s.replace('    assert!(render::json(&history).contains("\\\"subject\\\":\\\"odd\\\\u001f0123456789abcdef"));\n','').replace('    assert!(render::json(&history).contains("\\\"last_resolved\\\":null"));\n','');
s=replaceFn(s,'the_table_and_the_json_carry_the_same_entries_in_the_same_order',`fn the_table_preserves_typed_entries_sources_and_removed_bindings_in_order() {
 let _lock=serial(); let story=story();
 let history=story.repo.history("cap:alpha",&HistoryOptions::new()).expect("a history");
 let table=render::table(&history); let mut lines=table.lines();
 assert_eq!(lines.next(),Some("| Commit | Date | Subject | Source |")); assert_eq!(lines.next(),Some("| --- | --- | --- | --- |"));
 let rows:Vec<&str>=lines.by_ref().take_while(|line|!line.is_empty()).collect(); assert_eq!(rows.len(),history.entries().len());
 for (row,entry) in rows.iter().zip(history.entries()) {
   let commit=entry.commit().as_str(); assert!(row.starts_with(&format!("| {} | {} | ",&commit[..7],entry.date().day())));
   assert!(row.contains(entry.subject()));
   for source in entry.sources() { match source {
     Source::Document=>assert!(row.ends_with(" |")&&row.contains("| document")),
     Source::Symbol(path)=>assert!(row.contains(&format!("\x60{path}\x60"))),
     _=>panic!("unknown source"),
   } }
 }
 let removed:Vec<&str>=lines.collect();
 assert_eq!(removed[0],"| Binding | Removed in | Date | Subject | Last resolved |");
 assert!(removed[2].starts_with(&format!("| \x60demo::alpha::gone\x60 | {} | ",&story.removed[..7])));
 assert!(removed[2].ends_with(&format!(" | {} |",&story.first[..7])));
 assert_eq!(history.removed().len(),1);
 assert_eq!(history.removed()[0].binding(),"demo::alpha::gone");
}`);fs.writeFileSync(p,s);
p='crates/raf-workplan/tests/lines.rs';s=fs.readFileSync(p,'utf8').replace(/const (HELPER|EXCEPTED)_JSON: &str = "[\s\S]*?\n";\n\n/g,'').replace('    assert_eq!(render::lines_json(&lf), render::lines_json(&again));','    assert_eq!(lf, again);').replace('    assert_eq!(render::lines_json(&report), HELPER_JSON);',`    let row=&report.crates()[0];
    assert_eq!((row.counted(),row.covered(),row.excepted(),row.orphan(),row.shipping_orphan(),row.test_orphan(),row.debt()),(6,1,0,5,5,0,0));
    assert!(row.enforced()); assert_eq!(row.ceiling_text(),"shipping 10 lines");
    assert_eq!((report.modules()[0].counted(),report.modules()[0].orphan()),(6,5));
    assert_eq!(report.orphans()[0].item(),"demo::helper");
    assert_eq!(report.orphans()[0].ranges(),&[(3,7)]);`).replace('    assert_eq!(render::lines_json(&report), EXCEPTED_JSON);',`    assert_eq!(report.excepted().len(),1);
    assert_eq!(report.excepted()[0].exception(),"demo::helper");
    assert_eq!(report.excepted()[0].reason(),"Kept for a caller a | away.");
    assert_eq!(report.excepted()[0].lines().ranges(),&[(3,7)]);`).replace('    assert!(!render::lines_json(&plain).contains("lines-excepted"));','    assert!(plain.excepted().is_empty());');fs.writeFileSync(p,s);
