const fs=require('fs');let p='crates/raf-workplan/src/report.rs',s=fs.readFileSync(p,'utf8'),a=s.indexOf('        if !Finding::KINDS.contains'),b=s.indexOf('\n    }\n    Ok(graph)',a);let body=s.slice(a,b);body=body.replace('        let id = member(&mut graph, ordinal, "Finding")?;\n','').replaceAll('&mut graph','graph').replaceAll('&id','id').replace(/^        /gm,'    ');let insert=s.indexOf('/// Render typed findings');
const helper=`/// Append a typed finding at an owned subject shared by command projections.
/// The caller supplies membership and ordinal links; the finding class and all
/// location fields are written here. Discard the graph when an error occurs.
///
/// # Errors
/// Refuses unknown finding kinds, zero lines, unsupported locations and invalid
/// or duplicate graph terms.
pub fn finding_into(graph: &mut Graph, id: &str, finding: &Finding) -> Result<(), Error> {
    class(graph, id, "Finding")?;
${body}
    Ok(())
}

`;
s=s.slice(0,a)+`        let id = format!("#result/{ordinal}");
        add(&mut graph, "", "result", Atom::Iri(local(&id)?))?;
        add(&mut graph, &id, "ordinal", Atom::integer(ordinal)?)?;
        finding_into(&mut graph, &id, finding)?;`+s.slice(b);insert=s.indexOf('/// Render typed findings');s=s.slice(0,insert)+helper+s.slice(insert);fs.writeFileSync(p,s);
p='crates/cargo-workplan/src/result_graph.rs';s=fs.readFileSync(p,'utf8').replace('CommandResult::Finding(finding) => finding_fields(g, id, finding)?,','CommandResult::Finding(finding) => raf_workplan::report::finding_into(g, id, finding)?,');a=s.indexOf('fn finding_fields(');b=s.indexOf('\nfn state_fields(',a);if(a<0||b<0)throw Error('finding helper');s=s.slice(0,a)+s.slice(b+1);fs.writeFileSync(p,s);
