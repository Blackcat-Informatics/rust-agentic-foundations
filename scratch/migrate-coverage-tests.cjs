const fs=require('fs');let p='crates/cargo-workplan/tests/claims.rs',s=fs.readFileSync(p,'utf8');
s=s.replace('use support::{Repo, array, member_names, strict, strings};','use support::{Repo, rdf::Rdf, strict};').replace(/\/\/\/ The members of a `coverage-claim` object, in order\.\nconst CLAIM:[^\n]*\n\n/,'').replace(/\/\/\/ The same rows as JSON\.\nconst CLEAN_JSON: &str = "[\s\S]*?\n";\n\n/,'').replaceAll('"json"','"turtle"');
s=s.replaceAll('assert_eq!(json.stdout, CLEAN_JSON);',`let rdf = Rdf::of(&json);
    let claims = rdf.rows("ClaimCoverage");
    assert_eq!(claims.len(), 3);
    assert!(claims.iter().all(|c| c.required("capability") == "cap:demo"));
    assert_eq!(claims.iter().map(|c| c.number("line")).collect::<Vec<_>>(), [39, 42, 24]);
    assert!(claims.iter().all(|c| c.children("test")[0].required("path") == "demo::test_crate::it::case"));`);
let a=s.indexOf('    assert_eq!(member_names(&json),');let b=s.indexOf('    // Sorted by capability',a);
s=s.slice(0,a)+`    let rdf = Rdf::of(&json);
    let from_rdf: Vec<Vec<String>> = rdf.rows("ClaimCoverage").iter().map(|claim| vec![
        claim.required("capability").into(), claim.required("section").into(),
        claim.required("claim").into(), claim.number("line").to_string(),
        claim.children("test").iter().map(|test| format!("\x60{}\x60",test.required("path"))).collect::<Vec<_>>().join(", "),
    ]).collect();
    assert_eq!(from_rdf, table);
`+s.slice(b);fs.writeFileSync(p,s);
p='crates/cargo-workplan/tests/lines.rs';s=fs.readFileSync(p,'utf8').replace('use support::{Repo, array, member_names, strict, strings};','use support::{Repo, rdf::Rdf, strict};').replace(/\/\/\/ The members of each kind of object, in order\.[\s\S]*?(?=\/\/\/ The report of the strict-clean)/,'').replace(/const (CLEAN|ORPHANED)_JSON: &str = "[\s\S]*?\n";\n\n/g,'').replaceAll('"json"','"turtle"');
s=s.replace('    assert_eq!(json.stdout, CLEAN_JSON);\n    assert_eq!(member_names(&json), vec![strings(&CRATE)]);',`    let rdf = Rdf::of(&json);
    let rows = rdf.rows("CrateLines");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].required("package"), "demo");
    assert_eq!(rows[0].number("counted"), 8);
    assert_eq!(rows[0].number("covered"), 8);
    assert_eq!(rows[0].number("orphan"), 0);
    assert!(!rows[0].boolean("enforced"));
    assert!(rdf.rows("UnitLines").is_empty());`);
let a2=s.indexOf('    assert_eq!(json.stdout, ORPHANED_JSON);'),b2=s.indexOf('    assert_eq!(\n        repo.both',a2);
s=s.slice(0,a2)+`    let rdf = Rdf::of(&json);
    let package = rdf.rows("CrateLines")[0];
    assert_eq!(package.number("orphan"), 1);
    assert!(package.boolean("enforced"));
    assert_eq!(rdf.rows("ModuleLines")[0].number("counted"), 7);
    let unit = rdf.rows("UnitLines")[0];
    assert_eq!(unit.required("item"), "demo::helper");
    assert_eq!(unit.required("unit"), "fn");
    let range = unit.children("range")[0];
    assert_eq!((range.number("first"), range.number("last")), (8, 8));
`+s.slice(b2);
s=s.replace(/        assert!\(\n            json.stdout\n                .contains\("\\"kind\\":\\"coverage-lines-over-ceiling\\""\),\n            "\{json:#\?\}"\n        \);/,`        assert!(Rdf::of(&json).rows("Finding").iter().any(|f| f.required("kind") == "coverage-lines-over-ceiling"));`);
const a3=s.indexOf('    assert_eq!(\n        member_names(&json),');if(a3<0)throw Error('excepted');s=s.slice(0,a3)+`    let rdf = Rdf::of(&json);
    assert_eq!(rdf.rows("CrateLines")[0].number("excepted"), 1);
    let row = rdf.rows("ExceptedLines")[0];
    assert_eq!(row.required("exception"), "demo::helper");
    assert_eq!(row.required("reason"), "Kept for a caller.");
    let range = row.children("range")[0];
    assert_eq!((range.number("first"), range.number("last")), (8, 8));
}\n`;fs.writeFileSync(p,s);
