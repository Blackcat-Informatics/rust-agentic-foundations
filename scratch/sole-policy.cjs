const fs = require('fs');
function edit(path, change) { const s=fs.readFileSync(path,'utf8'); const n=change(s); if(s===n) throw Error('No change: '+path); fs.writeFileSync(path,n); }
function span(s,start,end,replacement='') { const a=s.indexOf(start), b=s.indexOf(end,a); if(a<0||b<0)throw Error('Missing span '+start);return s.slice(0,a)+replacement+s.slice(b); }
edit('crates/raf-okf/src/profile/typed.rs',s=>{
 s=span(s,'/// How hard a repository is judged:','/// The keys every document type shares.');
 s=span(s,'    /// How hard the repository is judged:','    /// The package names of the crates');
 s=span(s,'    /// The record with the strictness `strictness`.','    /// The record with the ceilings `ceilings`.');
 return s.replace('    /// of text. [Strict mode] asks for it on a `dropped` or `moved` item.\n    ///\n    /// [Strict mode]: Strictness::Strict','    /// of text. Completion requires it on a `dropped` or `moved` item.').replace('    /// the strictness [`Strictness::Default`] and no ceiling or exception.','    /// and no ceiling or exception.').replace('            strictness: Strictness::Default,\n','');
});
for (const p of ['crates/raf-okf/src/lib.rs','crates/raf-okf/src/profile/mod.rs','crates/raf-okf/src/profile/write.rs']) edit(p,s=>s.replaceAll(', Strictness',''));
edit('crates/raf-okf/src/profile/mod.rs',s=>{
 s=s.replace('    strictness: Slot<Strictness>,\n','').replace('            ValueKind::Strictness => {\n                found.strictness = Slot::from_check(check_strictness(check, entry));\n            }\n','').replace('            strictness,\n','').replace('                strictness: strictness.settle().ok()?.unwrap_or_default(),\n','');
 return span(s,'fn check_strictness(', 'fn check_items(');
});
edit('crates/raf-okf/src/profile/schema.rs',s=>s.replace('    /// `default` or `strict`.\n    Strictness,\n','').replace(/\s*optional\("strictness", ValueKind::Strictness\),/,'').replace(/\s*optional\("strictness", Strictness\),/,''));
edit('crates/raf-okf/src/profile/write.rs',s=>s.replace('    /// `scope` or `exclusions` is written `[]`. The strictness\n    /// [`Strictness::Strict`] is written `strictness: strict`, and\n    /// [`Strictness::Default`] is left out, as a record without the key reads.','    /// `scope` or `exclusions` is written `[]`.').replace('        if self.strictness != Strictness::Default {\n            entries.push(text("strictness", self.strictness.name())?);\n        }\n',''));
edit('crates/raf-okf/src/profile/violation.rs',s=>s.replace(/    \/\/\/ A `strictness` other than `default` and `strict`\.\n    BadStrictness \{\n        \/\/\/ The value found\.\n        found: String,\n    \},\n/,'').replace(/            Self::BadStrictness \{ found \} => \{\n                write!\(f, "bad strictness `\{found\}`; it is `default` or `strict`"\)\n            \}\n/,'').replace('    /// [`Rule::BadStrictness`].\n    BadStrictness,\n','').replace('            Self::BadStrictness => "bad-strictness",\n','').replace('            RuleTag::BadStrictness,\n','').replace('            Self::BadStrictness { .. } => RuleTag::BadStrictness,\n','').replace('            RuleTag::BadStrictness => Rule::BadStrictness {\n                found: String::new(),\n            },\n','').replace('            Rule::BadStrictness { .. } => 59,\n',''));
edit('crates/raf-workplan/src/lib.rs',s=>s.replace('pub use raf_okf::Strictness;\n',''));
edit('crates/raf-workplan/src/model.rs',s=>{
 s=s.replace(', Strictness','').replace('    /// The least strictness the caller judges with; see\n    /// [`Workplan::strictness`].\n    floor: Strictness,\n','').replace('            floor: Strictness::Default,\n','');
 s=span(s,'    /// How hard [`Workplan::check`] judges','    /// The roadmap:');
 s=span(s,'    /// Under [`Strictness::Strict`]','    #[must_use]\n    pub fn check', '    /// Coverage, prose, claims and debt are judged on every check.\n');
 return s.replace('        if self.strictness() == Strictness::Strict {\n            findings.extend(strict::check(self));\n        } else {\n            findings.extend(debt::unjudged(self));\n        }','        findings.extend(strict::check(self));').replace('    /// The findings the load made. They are the same under either\n    /// strictness: the rules of strict mode are judged by\n    /// [`Workplan::check`], never by the load.','    /// The findings of loading and relating documents. Complete policy\n    /// findings are judged by [`Workplan::check`], never by the load.').replace('    /// every rule that does not depend on the strictness.','    /// document loading and generated-block consistency.');
});
edit('crates/raf-workplan/src/gate.rs',s=>{
 s=s.replace(', Strictness','').replaceAll('    floor: Strictness,\n','').replace('            floor: Strictness::Default,\n','').replace('            floor: self.floor,\n','');
 s=span(s,'/// [`Gate::strict`] and [`Gate::strictness`]','#[derive(Debug, Clone)]','/// Every opened workplan judges the complete coverage, prose, claims and\n/// debt policy. There is no caller-selected policy floor.\n');
 s=span(s,'    /// Judge the repository under strict mode','    /// Open the debt record');
 s=span(s,'    /// The strictness floor of the [`Gate`]','    /// The documents of the repository');
 return s.replaceAll('.with_strictness(self.floor)','').replace('    /// source workspace, judged with the strictness floor of the [`Gate`].','    /// source workspace, using the complete repository policy.').replace('    /// file, so a link to it is broken. Nothing is written. The workplan is\n    /// judged with the strictness floor of the [`Gate`], as\n    /// [`Opened::workplan`] is.','    /// file, so a link to it is broken. Nothing is written. The same complete\n    /// policy as [`Opened::workplan`] judges the resulting documents.').replace('/// with at least the strictness floor of the [`Gate`]\n/// ([`Opened::strictness_floor`]).','/// with the complete repository policy.');
});
edit('crates/raf-workplan/src/debt.rs',s=>{
 s=s.replace(', Strictness',''); s=span(s,'/// The findings of the debt record under the default strictness:','/// Judge the findings of strict mode');
 return s.replace('    /// The repository is not judged strict, so it has no debt to record.\n    NotStrict,\n','').replace('            Self::NotStrict => f.write_str(\n                "the repository is not judged strict, so it has no debt to record; set `strictness: strict` in the coverage record, or pass --strict",\n            ),\n','').replace('Self::NotStrict | Self::Findings(_)','Self::Findings(_)').replace('        if plan.strictness() != Strictness::Strict {\n            return Err(DebtError::NotStrict);\n        }\n','');
});
edit('crates/raf-workplan/src/finding.rs',s=>s.replace('    DEBT_WITHOUT_STRICT = "debt-without-strict";\n','').replace('    OKF_BAD_STRICTNESS = "okf-bad-strictness";\n','').replace('        RuleTag::BadStrictness => (\n            kind::OKF_BAD_STRICTNESS,\n            "set `strictness` to default or strict, or remove the key, which reads as default",\n        ),\n',''));
edit('crates/raf-workplan/src/obligation.rs',s=>{
 s=s.replace(', Strictness','');
 s=span(s,'/// Whether `effect` leaves an obligation','#[must_use]\npub fn suppressed', '/// Whether a no-contract-change declaration suppresses an untouched Contract.\n/// Usage, Examples and every other obligation remain enforced.\n');
 s=s.replace('    strictness: Strictness,\n','');
 return span(s,'    let unaffected = match strictness {','    effect ==', '    let unaffected = section == CapabilitySection::Contract;\n');
});
edit('crates/raf-workplan/src/obligation/sources.rs',s=>s.replace(', Strictness','').replace('    let strictness = plan.strictness();\n','').replace('effect, strictness','effect').replace('    if plan.strictness() == Strictness::Strict {\n        for (index, item) in value.items.iter().enumerate() {','    for (index, item) in value.items.iter().enumerate() {').replace('        }\n    }\n    let known = Counted::new(', '    }\n    let known = Counted::new('));
edit('crates/raf-workplan/src/coverage/lines.rs',s=>{
 s=s.replace(', Strictness','').replace('    enforced: bool,\n','').replace('        enforced: false,\n','').replace('    let enforced = plan.strictness() == Strictness::Strict;\n','').replace('        row.enforced = enforced;\n','');
 return span(s,'    /// Whether its ceilings are judged:','    /// The counted lines of its files.');
});
edit('crates/raf-workplan/src/coverage/render.rs',s=>s.replace('        if row.enforced() { "yes" } else { "no" },\n','').replace('"| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",','"| {} | {} | {} | {} | {} | {} | {} | {} | {} |",').replaceAll(' Crate | Enforced |',' Crate |').replaceAll('| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |','| --- | --- | --- | --- | --- | --- | --- | --- | --- |'));
edit('crates/cargo-workplan/src/result_graph.rs',s=>s.replace('        boolean(g, &n, "enforced", row.enforced())?;\n',''));
edit('crates/cargo-workplan/src/cli.rs',s=>span(s,'    /// Judge under strict mode,','    /// The command to run.'));
edit('crates/cargo-workplan/src/command.rs',s=>{
 s=s.replaceAll(', args.strict','').replace('pub(crate) fn gate(root: &Path, strict: bool) -> Gate {\n    let gate = Gate::workspace(root);\n    if strict { gate.strict() } else { gate }\n}','pub(crate) fn gate(root: &Path) -> Gate {\n    Gate::workspace(root)\n}').replace('fn check(root: &Path, strict: bool)','fn check(root: &Path)').replace('gate(root, strict)','gate(root)'); return s;
});
edit('crates/cargo-workplan/src/change.rs',s=>s.replaceAll(', strict: bool','').replaceAll('gate(root, strict)','gate(root)').replaceAll('open(root, strict)','open(root)'));
edit('crates/cargo-workplan/src/debt.rs',s=>s.replace(', strict: bool','').replace('gate(root, strict)','gate(root)'));
edit('crates/cargo-workplan/src/coverage.rs',s=>{
 s=s.replace('use raf_workplan::{Gate, Strictness};','use raf_workplan::Gate;').replace(', strict: bool','');
 return span(s,'    let floor = if strict {','    let whole = plan.lines_report();','    let plan = opened.workplan();\n');
});
