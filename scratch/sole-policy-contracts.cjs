const fs=require('fs');
function edit(p,f){const s=fs.readFileSync(p,'utf8'),n=f(s);if(n===s)throw Error(p);fs.writeFileSync(p,n);}
function span(s,a,b,r=''){const x=s.indexOf(a),y=s.indexOf(b,x+a.length);if(x<0||y<0)throw Error(a);return s.slice(0,x)+r+s.slice(y);}
edit('docs/capabilities/okf-profile.md',s=>{
 s=span(s,'**Strictness.**','**Ceilings and exceptions.**','**Closed coverage policy.** The coverage profile has no policy selector. The\nremoved `strictness` key is an unknown key, including its former values, and the\ntyped coverage record and writer expose only the current closed key set.\n\n');
 return s.replace('  `strictness`, when it is there, is `default` or `strict`, its `scope` holds','  `scope` holds').replace('`EvidenceSubject`, `ExclusionItem`, `Strictness`, `CeilingClass`','`EvidenceSubject`, `ExclusionItem`, `CeilingClass`');
});
edit('docs/architecture/okf-profile.md',s=>{
 s=span(s,'- **`strictness`** (optional)','- **`scope`**');
 return s.replace('optional `strictness`, ','');
});
edit('docs/capabilities/workplan-document-model.md',s=>{
 s=span(s,'**Strictness.**','**The graph.**','**Complete policy.** Every `Workplan::check` judges the complete\n[content and binding policy](./workplan-strict.md) after the document-model and\ngenerated-block findings, then applies the debt ratchet. Loading documents\nretains only loading and relationship findings in `Workplan::findings`; it does\nnot select a lighter check. Gate callers, reconciliation and the completion check\nafter record removal use the same complete policy. There is no policy enum,\ncaller floor or coverage key that disables those checks.\n\n');
 return s.replaceAll('under either strictness, ','on every check, ');
});
edit('docs/capabilities/workplan-two-way-coverage.md',s=>{
 s=span(s,'**Strict mode.**','**The account.**','**Binding-reached packages.** The complete [workplan policy](./workplan-strict.md)\nalso enforces both directions for every package reached by a resolved code, test\nor example binding of a loaded capability. The same exclusions and refusal kinds\napply. A gap of a package reached only by bindings names the capability and path\nthat requires its coverage. An unreadable public surface is reported at that\ncapability\'s bindings. The code and tests reports list their whole workspace\naccount independently of these enforcement reasons. The coverage record also\nholds line ceilings and exceptions; it contains no policy selector.\n\n');
 return s.replace('`scope: []` enforces no crate and says so in the\nrepository.','`scope: []` adds no package to the explicit scope; packages reached by bindings\nremain enforced by the complete policy.');
});
edit('docs/capabilities/workplan-reconcile.md',s=>span(s,'**Strict mode.**','- `outcome-without-reason`','**Complete policy.** Reconciliation always judges the complete\n[workplan policy](./workplan-strict.md). A no-contract-change declaration only\nsuppresses an untouched Contract; affected Usage and Examples still need edits.\nThe following obligations apply without a caller floor, coverage selector or flag.\n\n'));
edit('docs/capabilities/workplan-complete-gate.md',s=>{
 s=span(s,'**Strict mode.**','refuses on `contract-effect-understated`','**Complete policy.** The check after record removal judges the complete\n[workplan policy](./workplan-strict.md), at both the initial judgement and the\nsecond judgement immediately before writes. A policy finding refuses completion\nwithout writing. There is no caller-selected policy floor.\n\n**Completion conditions.** The six conditions retain their order. Condition 5\nruns every bound test and example, and condition 1 checks supported-behavior\nwording and token-level contract effects. Condition 1\n');
 return s.replace('usage and examples sections, and under strict mode for the contract alone (see\n**Strict conditions**).','section alone. Affected Usage and Examples remain owed.');
});
edit('docs/capabilities/workplan-cli.md',s=>{
 s=span(s,'output, `-q` suppresses everything but findings,','a stable order and diagnostics to stderr.','output, and `-q` suppresses everything but findings. Every check, reconciliation\nand completion uses the complete [workplan policy](./workplan-strict.md). The\nremoved `--strict` option is a usage error before or after a command. Findings\nare printed to stdout in\n');
 s=s.replace('and 2 when the repository is not judged strict (the coverage record\ndoes not say `strictness: strict` and `--strict` is not given), `--write` is\nmissing, or the tool cannot run.','and 2 when `--write` is missing or the tool cannot run.');
 s=s.replace('- `cargo workplan --strict check` judges the repository under strict mode before\n  its coverage record says `strictness: strict`, and `cargo workplan check` judges\n  it so once the record says it.','- `cargo workplan check` judges the complete content, binding and line policy\n  without a policy flag or coverage selector.');
 return s;
});
edit('docs/capabilities/workplan-strict.md',s=>{
 s=span(s,'The default rules of the workplan','## Contract','The complete workplan policy connects documented behavior to runnable examples,\nclaim-level tests, explained source items and counted lines. Every check enforces\nthat policy, so a delivered capability cannot omit proof or leave source debt\nunreported. Completion also checks outcome reasons and token-level contract effects.\n\n');
 s=span(s,'**The setting.**','**Examples and the matrix.**','**One policy.** Every `Workplan::check`, `Gate` caller and command judges the\ncomplete policy. The closed coverage profile rejects `strictness`; the library\nexposes no policy enum or floor, and the command parser rejects `--strict`. Loading\ndocuments does not judge policy units, while every complete check does. The\ncompletion check after record removal applies the same policy at both judgements.\nContent and binding findings follow document-model and generated-block findings,\nthen the debt ratchet applies.\n\n');
 s=s.replaceAll('Under strict, ','').replaceAll('Under either strictness, ','On every check, ').replaceAll('under either strictness','on every check');
 s=s.replace('and in JSON as objects of the kind `coverage-claim` with the fields `capability`,\n`section`, `claim`, `line` and `tests` (`coverage::render::claims_json`).','and in the typed Claims RDF results of the [shared report profile](./rdf-report-profile.md).');
 s=s.replace('The strict rules compute this coverage themselves; the default coverage check\nis the same on every check.','The complete check combines explicit-scope enforcement with binding-reached\npackages; neither reason provides a policy opt-out.');
 s=s.replace('reason. Under `default` neither obligation of this capability is raised.','reason. Both obligations apply to every complete reconciliation and completion.');
 s=s.replace('The report prints three tables. The crate table has the columns Crate, Enforced\n(`yes` under strict mode), Counted, Covered, Excepted, Orphan, Shipping orphan, Test','The report prints three tables. The crate table has the columns Crate,\nCounted, Covered, Excepted, Orphan, Shipping orphan, Test');
 s=span(s,'counted. Rows are sorted by package','The report holds no absolute path','counted. Rows are sorted by package, module path, file and first line. The typed\nLines RDF results retain package totals, ordered range rows, exception reasons\nand counting errors through the [shared report profile](./rdf-report-profile.md).\n');
 return s.replace('; under `--strict` the ceilings are shown as enforced.','.');
});
edit('docs/capabilities/rdf-report-profile.md',s=>s.replace(', `enforced` boolean',''));
