//! probe
use raf_census::*;
use raf_source::MemoryTree;
#[test]
fn probe() {
    let src = "pub trait Make { fn make(x: u32) -> u32; }\npub struct S;\nimpl Make for S { fn make(x: u32) -> u32 { helper(x, 1) } }\npub fn p(a: u32) -> u32 { helper(a, 1) }\npub fn q(b: u32) -> u32 { helper(b, 1) }\nimpl S { pub fn via(x: u32) -> u32 { Self::inner(x, 1) } }\npub fn r(x: u32) -> u32 { Self::inner(x, 1) }\n";
    let tree: MemoryTree = [("Cargo.toml","[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),("src/lib.rs",src)].into_iter().collect();
    let c = Census::load(&tree,&["."],&CensusOptions::default()).unwrap();
    for g in c.groups() { println!("GROUP {:?} {:?}", g.fingerprint, g.member_symbols().collect::<Vec<_>>()); }
    let fp = c.groups()[0].fingerprint.clone();
    let job = Job::new("j","demo::p").forbidding(JobForbidden{fingerprints: vec![fp], constants: vec![]});
    for m in &c.match_jobs(&[job])[0].matches { println!("MATCH {} {:?}", m.symbol, m.class); }
    for u in c.units() { println!("UNIT {} key={:?}", u.symbol, u.shim_key.is_some()); }
}
