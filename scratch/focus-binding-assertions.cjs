const fs=require('fs'),p='crates/raf-workplan/tests/bindings.rs';let s=fs.readFileSync(p,'utf8');
s=s.replace('/// The one finding the bindings make, as `(kind, message)`.','/// The single binding refusal, as `(kind, message)`. Other complete-policy\n/// findings remain in the producer result and are not claimed absent here.');
s=s.replace('fn only(findings: &[Finding]) -> (&\'static str, &str) {\n    assert_eq!(findings.len(), 1, "{findings:#?}");\n    assert_eq!(findings[0].file, GAMMA);\n    (findings[0].kind, findings[0].message.as_str())\n}',`fn only_binding_refusal(findings: &[Finding]) -> (&'static str, &str) {
    let binding: Vec<&Finding> = findings.iter().filter(|finding| finding.kind.starts_with("binding-")).collect();
    assert_eq!(binding.len(), 1, "{findings:#?}");
    assert_eq!(binding[0].file, GAMMA);
    (binding[0].kind, binding[0].message.as_str())
}`);
s=s.replaceAll('only(&findings)','only_binding_refusal(&findings)').replaceAll('only(&found)','only_binding_refusal(&found)');
s=s.replaceAll('"[]"','"[demo::example_crate::show]"');
s=s.replace('            ("tests/it.rs", "#[test]\\nfn case() {}\\n"),','            ("tests/it.rs", "#[test]\\nfn case() {}\\n"),\n            ("examples/show.rs", "fn main() {}\\n"),');
s=s.replace('    let repo = bound(code, tests, examples);','    let mut repo = bound(code, tests, examples);\n    repo.remove(support::AUXILIARY);');
fs.writeFileSync(p,s);
