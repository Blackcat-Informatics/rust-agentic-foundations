//! probe
use raf_census::{CensusOptions, Level, Normalizer, Slice, Tower};
fn show(label: &str, params: &[&str], body: &str) {
    let t = Tower::new(&Slice::new("p.rs", 1, body).params(params.iter().copied())).unwrap();
    for l in Level::ALL { println!("{label} {l}: {}", t.form(l).texts().join(" ")); }
}
fn cmp(label: &str, pa: &[&str], a: &str, pb: &[&str], b: &str) {
    let n = Normalizer::new(&CensusOptions::default().min_tokens(1).unwrap());
    let d = n.explain(&Slice::new("p.rs",1,a).params(pa.iter().copied()), &Slice::new("p.rs",1,b).params(pb.iter().copied())).unwrap();
    println!("CMP {label}: equal_from={:?} differs_through={:?}", d.equal_from, d.differs_through);
}
#[test]
fn probe() {
    cmp("unused trailing param", &["a","b"], "let x = a; x", &["a"], "let x = a; x");
    cmp("param only in fmt", &["a"], "let x = 1; println!(\"{a} {x}\")", &["a"], "let x = 1; println!(\"{x} {x}\")");
    show("fmtA", &["a"], "let x = 1; println!(\"{a} {x}\")");
    show("fmtB", &["a"], "let x = 1; println!(\"{x} {x}\")");
    cmp("fmt named arg", &["a"], "let x = 1; println!(\"{x}\", x = a)", &["a"], "let y = 1; println!(\"{y}\", y = a)");
    cmp("pos second", &["a","b"], "let x = b; x", &["a"], "let x = a; x");
    cmp("shadow param", &["a","b"], "let b = a; b", &["a"], "let x = a; x");
    cmp("local type", &[], "struct A; let x = A; x", &[], "struct B; let y = B; y");
    show("dotmethod", &["len"], "v.len() + len");
    show("field param", &["x"], "S { x }.x + x");
    cmp("hex u8 vs dec", &[], "0xffu8", &[], "255u8");
    cmp("float", &[], "1e3 + 1.0", &[], "1000.0 + 1.0");
    show("int with e", &[], "0x1e3 + 1_000i64 + 0b1");
    show("closure", &["a"], "|q| q + a");
    show("nested fmt param only", &["a"], "{ let z = 2; format!(\"{a}{z}\") }");
}
