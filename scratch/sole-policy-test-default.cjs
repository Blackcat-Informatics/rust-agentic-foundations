const fs=require('fs');
function edit(p,f){let s=fs.readFileSync(p,'utf8');let n=f(s);if(n===s)throw Error(p);fs.writeFileSync(p,n);}
function span(s,a,b,r=''){let x=s.indexOf(a),y=s.indexOf(b,x);if(x<0||y<0)throw Error(a);return s.slice(0,x)+r+s.slice(y);}
function unwrap(s,head){for(;;){let a=s.indexOf(head);if(a<0)return s;let open=s.indexOf('{',a),depth=1,i=open+1;for(;depth;i++){if(s[i]==='{')depth++;else if(s[i]==='}')depth--;}s=s.slice(0,a)+s.slice(open+1,i-1)+s.slice(i);}}
for(const p of ['crates/raf-workplan/tests/strict.rs','crates/raf-workplan/tests/claim_coverage.rs','crates/raf-workplan/tests/lines.rs'])edit(p,s=>{
 s=unwrap(s,'for strictness in Strictness::ALL {');s=unwrap(s,'for strictness in [Strictness::ALL] {');
 return s.replaceAll('"{strictness:?}"','"the complete policy"');
});
edit('crates/raf-workplan/tests/strict.rs',s=>{
 s=s.replace('    assert_eq!(judged(capability.clone()), []);\n','').replace('    assert_eq!(\n        in_two_packages(reached.clone(), "[]"),\n        []\n    );\n','');
 s=span(s,'    let (default_load, default) = check_of','/// The strict-clean repository on a disk',`    let (load, checked) = check_of(capability, "");
    assert!(!load.iter().any(|finding| finding.kind.starts_with("strict-")
        || finding.kind == "coverage-item-unbound"), "{load:#?}");
    assert!(checked.starts_with(&load), "{checked:#?}");
    assert!(kinds(&load).contains(&"link-broken"), "{load:#?}");
    let added: Vec<&str> = kinds(&checked[load.len()..]).into_iter()
        .filter(|kind| *kind != "generated-stale" && *kind != "index-missing").collect();
    assert_eq!(added, ["strict-section-thin", "coverage-item-unbound", "coverage-test-unbound"], "{checked:#?}");
}

#[test]
fn the_removed_policy_key_is_rejected_even_for_its_former_values() {
    let source = workspace_of(&fixture::source_files());
    for value in ["strict", "default", "strct"] {
        let found = check_in(&repo(fixture::capability(), fixture::coverage(&format!("strictness: {value}\\n"))), &source);
        assert_eq!(kinds(&found), ["okf-unknown-key"], "{found:#?}");
        assert_eq!((found[0].file.as_str(), &found[0].location),
            (fixture::COVERAGE, &Location::Key("strictness".to_string())));
    }
    assert_eq!(check_in(&repo(fixture::capability(), fixture::coverage("")), &source), []);
}

`);
 s=span(s,'/// The kinds `gate` reports','#[test]\nfn while_a_capability',`#[test]
fn the_default_gate_judges_the_complete_policy_in_its_opened_workplan() {
    let scratch = on_disk(&fixture::thin("Examples"), "");
    let gate = Gate::workspace(scratch.path()).tracked_paths(Vec::new());
    let opened = gate.open().expect("the fixture opens");
    let retained = |found: Vec<Finding>| -> Vec<Finding> { found.into_iter()
        .filter(|finding| finding.kind != "index-missing" && finding.kind != "generated-stale").collect() };
    let direct = retained(gate.check());
    assert_eq!(kinds(&direct), ["strict-section-thin"]);
    assert_eq!(direct, retained(opened.workplan().check()));
}

`);
 return s.replaceAll('"okf-bad-strictness"','"okf-unknown-key"');
});
edit('crates/raf-workplan/tests/debt.rs',s=>{
 s=span(s,'#[test]\nfn a_repository_that_is_not_judged_strict','#[test]\nfn a_debt_record_in_a_repository',`#[test]
fn the_default_gate_records_present_debt_without_a_policy_switch() {
    let repo = repository("debt-default", &thin(&["Examples"]), strict::LIBRARY, &coverage("", ""));
    let written = gate(repo.path()).write_debt().expect("adopts");
    assert_eq!(written.change(), DebtChange::Created);
    assert_eq!(written.findings(), 1);
    assert_eq!(found(repo.path()), []);
}

`);
 s=span(s,'#[test]\nfn a_debt_record_in_a_repository','#[test]',`#[test]
fn an_empty_debt_record_is_judged_without_a_policy_switch() {
    let repo = repository("debt-empty", &strict::capability(), strict::LIBRARY, &coverage("", ""));
    put(repo.path(), DEBT, &debt_record(" []", " []"));
    assert_eq!(found(repo.path()), []);
}

`);
 return s.replace('    assert!(std::error::Error::source(&DebtError::NotStrict).is_none());\n','').replace('["debt-without-strict", "debt-entry-invalid"]','["debt-entry-invalid"]');
});
edit('crates/raf-workplan/tests/complete.rs',s=>{
 s=span(s,'    /// The lifecycle of the real programs over a gate with the strictness','}\n\n/// The refusal of a completion.');
 s=s.replaceAll('.lifecycle_with()', '.lifecycle()').replaceAll('.complete_with()', '.complete()').replace('    assert_eq!(strict.opened().strictness_floor());\n','');
 s=s.replace('    let completion = fixture\n        .over(passing(git_answers(), "passes"))\n        .complete("shape")\n        .expect("complete answers");\n    assert!(completion.is_completed(), "{completion}");\n}\n\n/// The record is removed through real directories only.', '}\n\n/// The record is removed through real directories only.');
 return s;
});
edit('crates/raf-workplan/src/coverage/lines/tests.rs',s=>{
 s=s.replace('Strictness, ','');
 s=span(s,'    let (found, tally) = measure_kind(Work::LineUnits, 0, || plan.check());','    // The function and its documentation test, and the test.', '    let (found, tally) = measure_kind(Work::LineUnits, usize::MAX, || plan.check());\n');
 return s;
});
edit('crates/raf-workplan/tests/lines.rs',s=>{
 s=s.replace('    assert!(row.enforced());\n','').replace('    assert!(!report.crates()[0].enforced());\n','');
 s=s.replace('    assert_eq!(\n        line_findings(&ws, &["demo::run"], ""),\n        []\n    );\n','');
 return s.replaceAll('| Crate | Enforced |','| Crate |').replaceAll('| demo | yes |','| demo |').replaceAll('| demo | no |','| demo |').replaceAll('| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |','| --- | --- | --- | --- | --- | --- | --- | --- | --- |');
});
