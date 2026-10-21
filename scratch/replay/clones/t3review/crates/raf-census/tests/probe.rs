//! probe
use raf_census::{Census, CensusOptions};
use raf_source::MemoryTree;
const M: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
fn load(src: &str) -> Result<Census, raf_census::Error> {
    let t: MemoryTree = vec![("Cargo.toml", M), ("src/lib.rs", src)].into_iter().collect();
    Census::load(&t, &["."], &CensusOptions::default())
}
#[test]
fn probe() {
    let src = "pub struct S;\nimpl S {\n    pub fn turbo(x: u32) -> u32 {\n        helper::<u8>(x, 1)\n    }\n    pub fn plain(x: u32) -> u32 {\n        helper(x, 1)\n    }\n    pub fn nested(x: u32) -> u32 {\n        helper(x, Self::f())\n    }\n    pub fn strs() -> usize {\n        let a = b\"\\xff\\xfe\";\n        let b = c\"hi\";\n        let c = r#\"a\nb\"#;\n        let d = \"z\";\n        let e = 340282366920938463463374607431768211456u128;\n        let f = 1.5e3;\n        let g = 'a';\n        0\n    }\n}\nconst _: u8 = 1;\nconst _: u8 = 2;\npub fn outer() { const IN: u32 = 0xdead; fn inner() { let _ = 0xbeef; } }\n";
    let c = load(src).unwrap();
    for u in c.units() {
        println!("{} L{} kind={:?} shim={:?} key={:?} via={} consts={:?} strs={:?} tabs={:?} tokens={:?}", u.symbol, u.line, u.kind, u.print.as_ref().map(|p| p.shim.is_some()), u.shim_key.is_some(), u.via_self, u.constants, u.strings, u.tables, u.print.as_ref().map(|p| p.tokens));
    }
    let c2 = load(src).unwrap();
    let a: Vec<_> = c.units().iter().map(|u| (&u.symbol, u.line, &u.file)).collect();
    let b: Vec<_> = c2.units().iter().map(|u| (&u.symbol, u.line, &u.file)).collect();
    assert_eq!(a, b);
}
#[test]
fn nested_self() {
    let c = load("pub struct S;\nimpl S {\n pub fn a(x: u32) -> u32 { helper(x, Self::f(1)) }\n pub fn b(x: u32) -> u32 { Self::f(helper(x, 1)) }\n pub fn c(x: u32) -> u32 { Self::V(x, 1) }\n pub fn d(x: u32) -> u32 { Vec::<u8>::new(x, 1) }\n}\n").unwrap();
    for u in c.units() { println!("{} via={} key={} shim={:?}", u.symbol, u.via_self, u.shim_key.is_some(), u.print.as_ref().map(|p| p.shim.is_some())); }
}
#[test]
fn many() {
    let src = std::fs::read_to_string("/tmp/many.rs").unwrap();
    let c = load(&src).unwrap();
    assert_eq!(c.units().len(), 180);
    for i in 0..60u128 {
        let u = c.unit(&format!("demo::C{i}")).unwrap();
        assert_eq!(u.constants.iter().copied().collect::<Vec<_>>(), { let mut v = vec![i, i + 7, i + 70000]; v.sort(); v.dedup(); v }, "C{i}");
        let s = c.unit(&format!("demo::S{i}")).unwrap();
        assert_eq!(s.strings[0].value, format!("s{i}"));
        assert_eq!(s.strings[0].line as u128, 6*i+5, "S{i} line");
    }
}
#[test]
fn repo() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let d = raf_source::DiskTree::new(&root).unwrap();
    let a = Census::load(&d, &["crates"], &CensusOptions::default()).unwrap();
    let b = Census::load(&d, &["crates"], &CensusOptions::default()).unwrap();
    let k = |c: &Census| c.units().iter().map(|u| (u.symbol.clone(), u.file.clone(), u.line, u.shim_key.clone(), u.print.as_ref().and_then(|p| p.body.as_ref().map(|h| h.fingerprint.clone())))).collect::<Vec<_>>();
    assert_eq!(k(&a), k(&b));
    let n = a.units().len();
    println!("units {n} keyed {} via {} trait_impl {} files {}", a.units().iter().filter(|u| u.shim_key.is_some()).count(), a.units().iter().filter(|u| u.via_self).count(), a.units().iter().filter(|u| u.trait_impl).count(), a.files().len());
    let nostring = a.units().iter().filter(|u| u.strings.iter().any(|s| s.line < u.line)).count();
    println!("string-before-unit-line {nostring}");
}
