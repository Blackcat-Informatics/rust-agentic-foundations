const fs=require('fs'),p='crates/cargo-workplan/tests/complete.rs';let s=fs.readFileSync(p,'utf8').replace('use serde_json::{Value, json};\n','').replace('use support::{BLOCKS, Repo, array, capability, change, changed, member_names, strings};','use support::{BLOCKS, Repo, capability, change, changed, rdf::Rdf};').replaceAll('"json"','"turtle"');
let a=s.indexOf('    let objects = array(&json);'),b=s.indexOf('    assert_eq!(\n        summary,',a);
s=s.slice(0,a)+`    let rdf = Rdf::of(&json);
    let refused = rdf.rows("Refused");
    assert_eq!(refused.len(), 1);
    assert_eq!(refused[0].number("failures"), 3);
    assert_eq!(refused[0].required("message"), "refused: 1 obligation, 1 item without outcome, 1 finding once the record is removed");
    let refusals = rdf.rows("Refusal");
    let summary: Vec<(String, Option<u64>, String, bool)> = refusals.iter().map(|row| (
        row.required("condition").into(), row.text("number").map(|n|n.parse().unwrap()),
        row.required("source").into(), row.boolean("repeated"),
    )).collect();
`+s.slice(b);
s=s.replace('.filter(|object| object["repeated"] == false)\n        .map(|object| object["message"].as_str().expect("message"))','.filter(|row| !row.boolean("repeated"))\n        .map(|row| row.required("message"))');
a=s.indexOf('    let sources: Vec<(Value, Value)>');b=s.indexOf('    assert!(repo.root().join(RECORD).is_file());',a);
s=s.slice(0,a)+`    let rdf = Rdf::of(&json);
    let sources = rdf.rows("Refusal").iter().map(|r|(r.number("number"), r.required("source").to_owned())).collect::<Vec<_>>();
    assert_eq!(sources, [(1,"example-failing".to_owned()),(5,"bound-example".to_owned())]);
`+s.slice(b);
a=s.indexOf('    let refusal = &array(&json)[1];');b=s.indexOf('    // The neighbour:',a);
s=s.slice(0,a)+`    let rdf = Rdf::of(&json);
    let refusal = rdf.rows("Refusal")[0];
    assert_eq!(refusal.required("condition"), "bound-runs");
    assert_eq!(refusal.number("number"), 5);
    assert_eq!(refusal.required("source"), "bound-test");
`+s.slice(b);
a=s.indexOf('    assert_eq!(\n        array(&run),');b=s.indexOf('\n}\n',a);
s=s.slice(0,a)+`    let rdf = Rdf::of(&run);
    assert_eq!(rdf.result_count(), 2);
    assert_eq!(rdf.rows("Completed")[0].required("path"), RECORD);
    assert_eq!(rdf.rows("Completed")[0].number("ordinal"), 0);
    assert_eq!(rdf.rows("Rewritten")[0].required("path"), CAP);
    assert_eq!(rdf.rows("Rewritten")[0].number("ordinal"), 1);
`+s.slice(b);fs.writeFileSync(p,s);
