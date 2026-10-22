const fs=require('fs');
function edit(p,f){let s=fs.readFileSync(p,'utf8'),n=f(s);if(n===s)throw Error(p);fs.writeFileSync(p,n);}
edit('crates/raf-workplan/tests/debt.rs',s=>{let a=s.indexOf('#[test]\nfn a_debt_record_in_a_repository_judged_default'),b=s.indexOf('/// The kinds a debt record',a);return s.slice(0,a)+s.slice(b);});
edit('crates/raf-workplan/tests/strict.rs',s=>s.replace('fn every_strict_case_reports_nothing_under_default() {\n    for case in strict_cases() {\n        let default = case;\n        assert_eq!(default.run(), [], "{}", default.name);\n    }\n}',`fn every_policy_refusal_case_is_enforced_by_the_default_check() {
    for case in strict_cases() {
        let found = case.run();
        assert!(found.iter().any(|finding| finding.kind == case.kind
            && finding.file == case.file && finding.location == case.at), "{}: {found:#?}", case.name);
    }
}`));
edit('crates/cargo-workplan/tests/check.rs',s=>{
 s=s.replace('    let run = repo.both(&["check"]).expect(0);\n    assert_eq!(run.stdout, "");\n}\n\n#[test]\nfn strict_check_exits_two', '    let repeated = repo.both(&["check"]).expect(1);\n    assert_eq!(repeated, run);\n}\n\n#[test]\nfn strict_check_exits_two');
 s=s.replace('    let run = default.both(&["check"]).expect(0);\n    assert_eq!(run.stdout, "");','    let run = default.both(&["check"]).expect(1);\n    assert!(run.stdout.contains("[okf-unknown-key]"), "{run:#?}");');
 return s;
});
edit('crates/cargo-workplan/tests/complete.rs',s=>s.replace('    let run = repo.run(&["change", "complete", "shape"]).expect(0);\n    assert!(run.stdout.starts_with("completed: "), "{run:#?}");\n}', '    let repeated = repo.run(&["change", "complete", "shape"]).expect(1);\n    assert_eq!(repeated, run);\n}'));
edit('crates/cargo-workplan/tests/reconcile.rs',s=>s.replace('    repo.both(&["change", "reconcile", "calm"]).expect(0);\n    let before = repo.worktree();','    repo.both(&["change", "reconcile", "calm"]).expect(1);\n    let before = repo.worktree();'));
