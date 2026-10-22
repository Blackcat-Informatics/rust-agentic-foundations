const fs=require('fs');
function edit(p,f){let s=fs.readFileSync(p,'utf8'),n=f(s);if(n===s)throw Error(p);fs.writeFileSync(p,n);}
function span(s,a,b,r=''){let x=s.indexOf(a),y=s.indexOf(b,x);if(x<0||y<0)throw Error(a);return s.slice(0,x)+r+s.slice(y);}
edit('crates/raf-workplan/src/strict.rs',s=>span(s,'//! Strict mode:','//! The findings that a repository',`//! The complete content and binding policy of every workplan check.
//!
//! The complete check judges these rules after the document-model checks.
//! Loading documents retains its own findings without judging these rules;
//! every complete check applies them, without a policy switch.
//!
`).replace('//! [`Workplan::check`]','//! [`Workplan::check`]').replace('//! under either strictness.','//! on every complete check.').replace('//! strictness, `claim-test-unbound`.','//! check, `claim-test-unbound`.').replace('//! either strictness.','//! every complete check.').replace('/// strict mode. The number is part of the contract and is not configurable: a\n/// repository that asks for less does not opt into strict mode.','/// the complete policy. The number is part of the contract and is not configurable.'));
edit('crates/raf-workplan/src/finding.rs',s=>s.replace('    ///   capability, judged only when [`crate::Workplan::strictness`] is\n    ///   [`crate::Strictness::Strict`]; see [`crate::strict`].','    ///   capability, judged on every complete check; see [`crate::strict`].').replace('`debt-resolved`, `debt-without-strict` and `debt-entry-invalid`','`debt-resolved` and `debt-entry-invalid`').replace('    ///   `cargo workplan debt --write` lowers the record; a debt record in a\n    ///   repository judged under [`crate::Strictness::Default`] is\n    ///   `debt-without-strict`; a recorded finding whose kind is no debt or','    ///   `cargo workplan debt --write` lowers the record; a recorded finding whose kind is no debt or'));
edit('crates/raf-workplan/src/coverage/lines.rs',s=>s.replace('//! strictness, and returns a [`LinesReport`].','//! complete policy, and returns a [`LinesReport`].').replace('    /// ceilings of the coverage record (none without a record), under either\n    /// strictness; see [the module documentation](crate::coverage::lines).\n    /// Each package says whether its ceilings are enforced, which they are\n    /// under strict mode. Nothing is written and no process is started.','    /// ceilings of the coverage record (none without a record); see\n    /// [the module documentation](crate::coverage::lines). Every complete\n    /// check enforces those ceilings. Nothing is written and no process is started.'));
edit('crates/raf-workplan/src/debt.rs',s=>s.replace('//! * [`DebtError::NotStrict`] when the repository is not judged strict;\n','').replaceAll('strictness: strict\\n',''));
edit('crates/raf-workplan/src/obligation.rs',s=>{
 s=s.replace('under strict mode only, ','').replace('; `contract-effect-understated` under strict mode only','');
 s=s.replace('//! the `contract`, `usage` and `examples` sections and nothing else, and under\n//! strict mode for the `contract` section alone ([`suppressed`]).','//! the `contract` section alone ([`suppressed`]); Usage and Examples remain owed.');
 return span(s,'    /// Whether only [strict mode]','    /// The place of the kind');
});
edit('crates/raf-workplan/src/obligation/tests.rs',s=>{
 const a=s.indexOf('    let strict: Vec<ObligationKind>');
 if(a<0)throw Error('strict classification test');
 const end=s.indexOf('\n}',a);return s.slice(0,a)+s.slice(end);
});
edit('crates/raf-workplan/tests/claim_coverage.rs',s=>s.replace('kinds(&judged(&code)) == ["claim-test-unbound"]','kinds(&judged(&code)) == ["claim-test-unbound", "strict-example-missing", "strict-usage-unbound"]'));
