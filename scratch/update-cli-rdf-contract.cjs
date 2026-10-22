const fs=require('fs'),p='docs/capabilities/workplan-cli.md';let s=fs.readFileSync(p,'utf8').replaceAll('--format text|json','--format text|turtle|ntriples');let a=s.indexOf('**JSON.**'),b=s.indexOf('**Strict mode.**',a);if(a<0||b<0)throw Error('machine contract');
s=s.slice(0,a)+`**Machine reports.** Both RDF formats use the closed canonical
[report profile](./rdf-report-profile.md), with its versioned application
vocabulary and one snapshot identity. Every command projects typed domain values;
there is no independent pre-rendered machine block. Named rows and explicit
ordinals preserve nested sequence, and absent optional fields have no triple.
Findings retain their location variant, rule kind, message and fix. Histories
retain full commit ids, dates, sources, paths and removal results. Evidence and
assessment views retain actual input digests and unresolved-input reasons;
measurements remain exact decimal strings. Verification includes only invocation
rows returned by the executor, with their arguments and process outcomes. A
planning refusal does not invent a run. The shared profile documents every class,
property and object type.

JSON formats are usage errors, exit 2, before root loading or process execution.
Successful machine output is one complete canonical graph, including an empty
Report envelope when no visible result remains. A command that cannot run has
empty standard output and a diagnostic on standard error. An output or diagnostic
write failure is also exit 2; a closed diagnostic sink cannot turn a failed run
into success.

**Quiet reports.** A history, an evidence view, a recorded assessment and its
context or list are outcomes. A coverage report is a finding as a whole when it
has a gap or unreadable code surface; a claims report when a claim names no test;
a lines report when it has orphan lines or a counting error. Quiet output keeps
those complete reports and removes outcome reports. A failed evidence record
stays and a passing record is hidden. Text prints no line for an omitted outcome;
RDF still prints its empty command envelope. Quiet never changes the exit class.

`+s.slice(b);
s=s.replace('The JSON kind `debt-written` joins the record\nkinds, with the members `path`, `change`, `findings` and `lines`, and it equals the\nkind of no rule.','The typed DebtWritten result retains its path, change classification, finding\ncount and orphan-line item count.');
s=s.replaceAll('--format json','--format turtle');
a=s.indexOf('`--format turtle` prints one array');b=s.indexOf('`evidence record` reports',a);const example=fs.readFileSync('crates/cargo-workplan/tests/fixtures/reports/check.ttl','utf8');s=s.slice(0,a)+'A capability that binds a missing symbol retains its frontmatter location and\na concrete repair in the canonical report:\n\n```turtle\n'+example+'```\n\n'+s.slice(b);
s=s.replace('The JSON form carries the text itself:\n  JSON escapes the control characters U+0000 to U+001F, and every other\n  character, a bidirectional formatting character included, is written as it is,\n  for the reader of the JSON to show safely.','The RDF forms retain the original text through canonical RDF lexical escaping.\n  Decoded data still requires the consumer to choose safe terminal display;\n  the human text option performs that display escaping.');
s=s.replace('- JSON fixtures: one array per run with a `kind` first in each object, the kinds that\n  are not rules never equal to a rule name, and stdout empty for a run that could not\n  run.','- Real command RDF fixtures: exact native byte goldens, complete emitted-term\n  equality with the documented registries, typed scalar and ordered child\n  assertions, both-format graph equivalence, empty envelopes and usage refusals.');
s=s.replace('in JSON, a shallow clone refused','in typed RDF, a shallow clone refused').replace('JSON pins of the members of each kind of the record commands in order, and every\n  kind unequal to a rule name;','RDF projections of every record command preserve typed fields and nested\n  order;').replace('the JSON form holds no raw escape,\n  which it writes as JSON escapes a control character, and reads back as the text;','the RDF forms hold no raw escape control and decode to the original text;').replace('JSON, naming the document','RDF, naming the document');
fs.writeFileSync(p,s);
