const fs=require('fs');
function edit(p,f){let s=fs.readFileSync(p,'utf8');let n=f(s);if(n===s)throw Error(p);fs.writeFileSync(p,n);}
function span(s,a,b,r=''){let x=s.indexOf(a),y=s.indexOf(b,x);if(x<0||y<0)throw Error(a);return s.slice(0,x)+r+s.slice(y);}
function walk(p){return fs.readdirSync(p,{withFileTypes:true}).flatMap(x=>x.isDirectory()?walk(p+'/'+x.name):[p+'/'+x.name]);}
for(const p of walk('crates').filter(p=>p.endsWith('.rs')&&p.includes('/tests/'))){let s=fs.readFileSync(p,'utf8');let n=s.replaceAll('Strictness::Default::Strict','Strictness::ALL').replace(/(?<!Strictness)::(?:Default|Strict)\b/g,'').replace(/\((?:Strictness::(?:Strict|Default))\)/g,'()');if(n!==s)fs.writeFileSync(p,n);}
edit('crates/raf-okf/tests/records.rs',s=>span(s,'#[test]\nfn the_strictness_of_a_coverage_record','#[test]\nfn scope_holds',`#[test]
fn a_coverage_record_rejects_the_removed_policy_key() {
    clean(&coverage(), COVERAGE_PATH);
    for value in ["default", "strict", "strct", "Strict", "true", "[]"] {
        let broken = mutate(&coverage(), "\\nscope:", &format!("\\nstrictness: {value}\\nscope:"));
        assert_only(&broken, COVERAGE_PATH, &key("strictness"), &Rule::UnknownKey {
            key: "strictness".to_string(),
        });
    }
}

`));
edit('crates/raf-okf/tests/record_writer.rs',s=>{
 s=span(s,'#[test]\nfn a_strict_coverage_record','#[test]\nfn empty_lists',`#[test]
fn a_coverage_record_writes_only_its_closed_policy_independent_keys() {
    let record = CoverageRecord::new(Vec::new(), Vec::new());
    let text = record.to_document().expect("the record writes").render();
    assert_eq!(text, "---\\ntype: Coverage\\nid: coverage:default\\nscope: []\\nexclusions: []\\n---\\n");
    assert_eq!(read(&text, "work/coverage/default.md"), RafDocument::Coverage(record));
}

`);return s;
});
edit('crates/raf-workplan/tests/support/mod.rs',s=>{
 s=span(s,'    check_in_with(repo, workspace)','/// Load `repo` through a memory tree and return every finding.',`    let tree: MemoryTree = repo.iter().collect();
    without_sync_findings(Workplan::load(&tree, workspace).check())
}

`);
 return s;
});
// There is one check helper; no test-only alternate policy remains.
for(const p of walk('crates/raf-workplan/tests').filter(p=>p.endsWith('.rs'))){let s=fs.readFileSync(p,'utf8');let n=s.replaceAll('check_in_with','check_in');if(n!==s)fs.writeFileSync(p,n);}
edit('crates/raf-workplan/tests/support/cases.rs',s=>{
 s=s.replace('    /// The strictness floor the case is judged with.\n    pub(crate) strictness: Strictness,\n','').replace('            strictness: Strictness::Default,\n','');
 s=span(s,'    /// A case judged with the strictness floor','    /// A case judged against the source workspace');
 s=s.replace(', self.strictness','');
 s=span(s,'        assert_eq!(\n            self.strictness,','        if self.unfiltered');
 s=span(s,'        RuleTag::BadStrictness => coverage(','        RuleTag::BadCeiling');
 return span(s,'        // A record in a repository that is not judged strict','        // The thin Examples section');
});
edit('crates/raf-workplan/src/obligation/tests.rs',s=>{
 s=s.replace(', Strictness','');
 return span(s,'#[test]\nfn contract_effect_none_suppresses','// --- order and rendering',`#[test]
fn contract_effect_none_suppresses_only_an_untouched_contract() {
    let sections = CapabilitySection::ALL.into_iter().map(Part::Section)
        .chain([Part::Document, Part::Item(1)]);
    for section in sections {
        for kind in ObligationKind::ALL {
            let expected = kind == ObligationKind::SectionUntouched
                && section == Part::Section(CapabilitySection::Contract);
            assert_eq!(suppressed(kind, section, ContractEffect::None), expected, "{kind} in {section}");
            assert!(!suppressed(kind, section, ContractEffect::Changes), "{kind} in {section}");
        }
    }
}

`);
});
edit('crates/raf-workplan/src/strict/tests.rs',s=>{
 s=s.replace('Strictness, ','');
 return s.slice(0,s.indexOf('#[test]\nfn under_default_no_unit'))+`#[test]
fn the_default_check_judges_each_policy_unit_once_without_policy_work_during_load() {
    let source = Workspace::load(&tree(&fixture::source_files()), &["."]).expect("the source");
    let docs = documents("");
    let (plan, load) = measure_kind(Work::StrictUnits, 0, || Workplan::load(&docs, &source));
    assert_eq!(load.get(Work::StrictUnits), 0);
    let (found, tally) = measure_kind(Work::StrictUnits, 2, || plan.check());
    assert_eq!(tally.get(Work::StrictUnits), 2);
    assert!(found.iter().any(|finding| finding.kind == "strict-section-thin"), "{found:#?}");
}
`;
});
