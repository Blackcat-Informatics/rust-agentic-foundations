const fs=require('fs');
function edit(p,f){const s=fs.readFileSync(p,'utf8'),n=f(s);if(n===s)throw Error(p);fs.writeFileSync(p,n);}
function span(s,a,b,r=''){const x=s.indexOf(a),y=s.indexOf(b,x+a.length);if(x<0||y<0)throw Error(a);return s.slice(0,x)+r+s.slice(y);}
edit('docs/capabilities/workplan-strict.md',s=>{
 s=s.replace('default `all`. Under strict every package of the report is judged:','default `all`. Every package of the report is judged:').replace('`okf-bad-measure`. Under `default` no ceiling is judged, and the report shows them.','`okf-bad-measure`. Every complete check judges the ceilings.');
 s=s.replace('- a debt record while the repository is judged under `default` is\n  `debt-without-strict`, whose fix is to set `strictness: strict` or remove the\n  record.','');
 s=s.replace('mode. A repository that is not judged strict is `DebtError::NotStrict`. The write','documented debt. The write').replace('and 2 when the repository is not judged strict, `--write` is missing, or the tool\ncannot run.','and 2 when `--write` is missing or the tool cannot run.');
 s=span(s,'**The kinds.**','## Usage','**The kinds.** The closed `Finding::KINDS` registry includes example, matrix,\nclaim, section, usage and limitation refusals, coverage exceptions and line\nceilings, and debt resolution or malformed entries. Removed policy selectors are\nordinary unknown-key or unknown-option errors. The typed Finding RDF projection\npreserves kind, exact source location, message and fix through the\n[shared report profile](./rdf-report-profile.md). Outcome reasons and understated\ncontract effects remain obligations of the completion lifecycle.\n\n');
 s=span(s,'## Usage','## Constraints',`## Usage

Call the runnable example on raf_workplan::Workplan::check to see the same
complete policy used by repository gates. Its valid record produces no finding,
while a record containing the removed policy selector is refused with its key
location. Repository commands require no policy flag:

~~~sh
cargo workplan check
cargo workplan change reconcile incremental-parsing
cargo workplan coverage claims --capability cap:streaming-parser
cargo workplan coverage lines --crate raf-okf
cargo workplan --format turtle coverage lines --crate raf-okf
~~~

## Examples

A delivered capability binds a runnable example and a test. Its Contract clauses
and Constraints units name those bound paths in tested-by markers. Removing the
last example is refused; restoring its binding and naming it in Usage satisfies
that requirement without changing the gate invocation.

A package with one uncalled private helper reports that helper's physical line as
orphan debt. Binding or reaching the helper covers it. A declared exception keeps
its exact target and reason visible, while an exception matching nothing is
refused. A debt record can hold present findings, but recording them is distinct
from a repository with zero uncovered items, claims and lines.

`);
 s=s.replace('- A caller cannot lower the strictness of a repository whose record says `strict`.','- Callers cannot disable the complete policy through a flag, typed floor or coverage key.');
 const exp='## Explanation';let x=s.indexOf(exp),v=s.indexOf('## Verification',x);
 s=s.slice(0,x)+`## Explanation

The complete policy makes a delivered contract independently checkable: runnable
examples show usage, bound tests substantiate individual claims, and source
coverage exposes unexplained behavior. One policy keeps direct library calls,
repository gates and command invocations consistent. The loading findings remain
separate because callers also use the document model for inspection and repair.
The debt ratchet records existing obligations without converting them into proof
of zero-debt adoption.

`+s.slice(v);
 const start=s.indexOf('## Verification'),gen=s.indexOf('<!-- raf:generated:',start);
 if(gen<0)throw Error('verification blocks');
 s=s.slice(0,start)+`## Verification

The bound fixtures exercise every content refusal with a neighboring accepted
document, counted work during loading and complete checks, binding-reached
packages, each claim boundary, outcome reasons and token-level contract effects.
Line fixtures cover all target kinds, overlapping physical ranges, private and
test code, exceptions, explicit ceilings, default zero ceilings and actionable
counting failures. Debt fixtures cover all ten recordable finding kinds, growth,
paid rows, invalid rows, unchanged bytes and typed refusal errors. Command
fixtures compare library results, human output and typed RDF results, including
quiet behavior and refusal exit classes. The removed selector is refused by both
the closed record profile and command parser beside valid ordinary invocations.

`+s.slice(gen);
 s=s.replace(/^[ \t]*- raf_workplan::coverage::render::(?:claims_json|lines_json)\n/gm,'');
 return s;
});
