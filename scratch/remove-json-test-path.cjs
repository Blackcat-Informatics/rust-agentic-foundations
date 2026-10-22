const fs=require('fs');let p='crates/cargo-workplan/tests/check.rs',s=fs.readFileSync(p,'utf8').replace('use serde_json::{Value, json};\n','').replace('use support::{Repo, array, member_names, strict, strings};','use support::{Repo, rdf::Rdf, strict};').replaceAll('"json"','"turtle"');
let a=s.indexOf('    assert_eq!(\n        member_names(&run),'),b=s.indexOf('    let text = repo.both',a);
s=s.slice(0,a)+`    let rdf=Rdf::of(&run); let rows=rdf.rows("Finding"); assert_eq!(rows.len(),1);
    let actual=Gate::workspace(repo.root()).strict().check(); let finding=&actual[0];
    assert_eq!(rows[0].required("kind"),"strict-section-thin");
    assert_eq!(rows[0].required("file"),"docs/capabilities/demo.md");
    assert_eq!(rows[0].required("section"),"Examples");
    assert!(rows[0].text("line").is_none()); assert!(rows[0].text("key").is_none());
    assert_eq!(rows[0].required("message"),finding.message);
    assert_eq!(rows[0].required("fix"),finding.fix);
`+s.slice(b);s=s.replace('text.lines().len(), objects.len()','text.lines().len(), rows.len()');fs.writeFileSync(p,s);
p='crates/cargo-workplan/tests/support/mod.rs';s=fs.readFileSync(p,'utf8').replace('use serde_json::Value;\n','');a=s.indexOf('/// The one array standard output holds.');b=s.indexOf('/// The files that differ between two snapshots:',a);if(a<0||b<0)throw Error('helpers');s=s.slice(0,a)+s.slice(b);fs.writeFileSync(p,s);
p='crates/cargo-workplan/Cargo.toml';s=fs.readFileSync(p,'utf8').replace('serde_json = "1.0.154"\n','');fs.writeFileSync(p,s);
// This case checks the command's exit class rather than its obsolete carrier.
p='crates/cargo-workplan/tests/process.rs';s=fs.readFileSync(p,'utf8').replace('vec!["--format", "json", "change", "reconcile", "grow"]','vec!["--format", "turtle", "change", "reconcile", "grow"]');fs.writeFileSync(p,s);
