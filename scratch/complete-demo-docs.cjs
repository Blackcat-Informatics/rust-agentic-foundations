const fs=require('fs');
const b=String.fromCharCode(96);
function edit(p,f){const s=fs.readFileSync(p,'utf8'),n=f(s);if(s===n)throw Error(p);fs.writeFileSync(p,n);}
edit('crates/raf-workplan/tests/support/mod.rs',s=>{
 s=s.replace('/// The coverage record that enforces nothing: `scope: []` and no exclusion.','/// An empty explicit scope with no exclusions; binding-reached packages remain judged.');
 s=s.replace('                format!("## {name}\\n\\n{extra}Text for {name}.\\n")',`                let lead = if *name == "Contract" { "**Result.** " } else { "" };
                let marker = if state == "delivered" {
                    "<!-- raf:tested-by demo::test_crate::it::case -->\\n"
                } else { "" };
                let policy = match *name {
                    "Contract" => marker.to_string(),
                    "Constraints" => format!("{marker}\\n- The small fixture describes its own source declarations and examples, without claiming behavior for another package or unlisted external input.\\n{marker}"),
                    "Usage" => "Run ${b}demo::example_crate::show${b} as the bound example.\\n".to_string(),
                    "Verification" => super::strict::MATRIX.to_string(),
                    _ => String::new(),
                };
                format!("## {name}\\n\\n{extra}{lead}Text for {name}. This fixture describes the declared source surface and its bindings so each observed result can be checked against the same small workspace.\\n{policy}")`);
 s=s.replace('        format!(\n            "---\\n# SPDX-FileCopyrightText: 2026 Example\\n# SPDX-License-Identifier: CC-BY-4.0\\n\\\n             type: Capability', '        let examples = if state == "delivered" { "[demo::example_crate::show]" } else { "[]" };\n        format!(\n            "---\\n# SPDX-FileCopyrightText: 2026 Example\\n# SPDX-License-Identifier: CC-BY-4.0\\n\\\n             type: Capability');
 s=s.replace('tests: {tests}\\n  examples: []\\n\\\n             depends: {depends}', 'tests: {tests}\\n  examples: {examples}\\n\\\n             depends: {depends}');
 s=s.replace('.replacen("  examples: []", &format!("  examples: {examples}"), 1)', '.replacen("  examples: [demo::example_crate::show]", &format!("  examples: {examples}"), 1)');
 return s;
});
edit('crates/raf-workplan/tests/support/synced.rs',s=>s.replace('    put(super::COVERAGE, super::open_coverage());\n    repo','    put(super::COVERAGE, super::open_coverage());\n    super::with_auxiliary(repo)'));
edit('crates/raf-workplan/tests/support/changes.rs',s=>s.replace('synced::synced(&super::with_open_coverage(repo))','synced::synced(&super::with_auxiliary(super::with_open_coverage(repo)))'));
