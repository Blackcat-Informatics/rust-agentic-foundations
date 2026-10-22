const fs=require('fs');function edit(p,f){fs.writeFileSync(p,f(fs.readFileSync(p,'utf8')))}
edit('crates/raf-okf/Cargo.toml',s=>s.replace('[dependencies]','[dependencies]\npurrdf-xsd.workspace = true'));
edit('crates/raf-okf/src/profile/typed.rs',s=>s.replace('    value: f64,','    value: purrdf_xsd::exact::Decimal,').replace('let value = text.parse::<f64>().ok()?;','let value = purrdf_xsd::numeric::parse_decimal(text).ok()?;').replace('        self.value\n','        self.value.to_f64()\n').replace(/        let \(left, right\) = \(Decimal::of[\s\S]*?\n    }\n\n    \/\/\/ Whether/,'        self.value.cmp(&other.value)\n    }\n\n    /// Whether').replace(/\/\/\/ The parts of the text of a \[`Number`\][\s\S]*?\nimpl PartialEq for Number/,'impl PartialEq for Number'));
edit('crates/raf-rdf/examples/report.rs',s=>s.replace('object: atom.into()','object: atom'));
edit('crates/raf-rdf/tests/profile.rs',s=>s.replace('nodes.insert("".into(), properties)','nodes.insert(String::new(), properties)').replace('TermValue::typed_literal("not a number", &format!("{}integer", vocab::XSD))\n                .into()','TermValue::typed_literal("not a number", vocab::INTEGER)'));
edit('crates/raf-workplan/src/digest.rs',s=>s.replace('implementation as PurRDF','implementation as `PurRDF`').replaceAll('hasher.update(&[0])','hasher.update([0])'));
edit('crates/raf-workplan/src/coverage/report_projection_tests.rs',s=>s.replace('TermValue::typed_literal("value", format!("{}datatype", vocab::REPORT)).into()','TermValue::typed_literal("value", format!("{}datatype", vocab::REPORT))').replace('named("object").into()','named("object")').replace('.map(|p| TermValue::simple_literal(p))','.map(TermValue::simple_literal)'));
edit('crates/raf-workplan/src/report.rs',s=>s.replace('Gap, GapKind, SurfaceFailure','Gap, GapKind, SurfaceError, SurfaceFailure').replace('                TermValue::typed_literal((scope).to_string(), vocab::BOOLEAN),','                TermValue::typed_literal(scope.to_string(), vocab::BOOLEAN),').replace(/            add\(\n                &mut graph,\n                &id,\n                "inScope",\n                TermValue::typed_literal\(scope.to_string\(\), vocab::BOOLEAN\),\n            \)\?;[\s\S]*?\n            }\n        }\n    }\n    Ok\(graph\)/,`            surface_error(&mut graph, &id, error, scope)?;
        }
    }
    Ok(graph)`)
.replace('/// Render typed item/test coverage in either canonical RDF 1.2 format.',`fn surface_error(graph: &mut Graph, id: &str, error: &SurfaceError, scope: bool) -> Result<(), Error> {
 add(graph, id, "inScope", TermValue::typed_literal(scope.to_string(), vocab::BOOLEAN))?;
 for (key,text) in [("package",error.package()),("target",error.target()),("root",error.root()),("failure",error.failure().name()),("message",error.message())] {
  add(graph,id,key,TermValue::simple_literal(text))?;
 }
 if let SurfaceFailure::CarryBudget {path} = error.failure() { add(graph,id,"failurePath",TermValue::simple_literal(path))?; }
 Ok(())
}

/// Render typed item/test coverage in either canonical RDF 1.2 format.`));
