const fs=require('fs');
function edit(p,f){let s=fs.readFileSync(p,'utf8');let n=f(s);if(n===s)throw Error(p);fs.writeFileSync(p,n);}
function span(s,a,b,r=''){let x=s.indexOf(a),y=s.indexOf(b,x);if(x<0||y<0)throw Error(a);return s.slice(0,x)+r+s.slice(y);}
edit('crates/raf-okf/src/profile/schema.rs',s=>s.replace('const COVERAGE: [KeySpec; 9]','const COVERAGE: [KeySpec; 8]'));
edit('crates/raf-okf/src/profile/mod.rs',s=>s.replace('        strictness,\n','').replace('            strictness: strictness.settle().ok()?.unwrap_or_default(),\n',''));
edit('crates/raf-okf/src/profile/violation.rs',s=>s.replace('pub const ALL: [RuleTag; 64]','pub const ALL: [RuleTag; 63]').replace('        RuleTag::BadStrictness,\n',''));
edit('crates/raf-workplan/src/model.rs',s=>s.replace('use crate::debt;\n',''));
edit('crates/raf-workplan/src/lifecycle/session.rs',s=>{
 s=s.replace(', Strictness','').replace('        let strictness = plan.strictness();\n','').replace('            strictness,\n','').replace('if strictness == Strictness::Strict && effect == ContractEffect::None','if effect == ContractEffect::None');
 return span(s,'/// The strictness floor of that [`Gate`]','/// ```rust,no_run','/// The complete repository policy judges both reconciliation and the final\n/// completion check after the change record is removed.\n///\n');
});
edit('crates/raf-workplan/src/lifecycle/untouched.rs',s=>s.replace(', Strictness','').replace('    strictness: Strictness,\n','').replace('                strictness,\n','').replace('/// the effect of the change under its strictness ([`suppressed`]). Otherwise the merge base is found once','/// the effect of the change ([`suppressed`]). Otherwise the merge base is found once'));
