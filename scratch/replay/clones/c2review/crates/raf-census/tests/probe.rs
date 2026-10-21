//! probe
use raf_census::{Census, CensusOptions};
use raf_source::MemoryTree;
const M: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
fn deep(levels: usize, g: usize) -> String {
    let mut ty = String::from("u8");
    for _ in 0..levels { ty = format!("({}{ty}{},)", "A<".repeat(g), ">".repeat(g)); }
    ty
}
fn run(name: &str, src: String) {
    let n = name.to_owned();
    let r = std::thread::Builder::new().stack_size(2<<20).spawn(move || {
        let t: MemoryTree = vec![("Cargo.toml", M), ("src/lib.rs", src.as_str())].into_iter().collect();
        match Census::load(&t, &["."], &CensusOptions::default()) {
            Ok(c) => format!("ok units={}", c.units().len()),
            Err(e) => format!("err {}", e.to_string().chars().take(90).collect::<String>()),
        }
    }).unwrap().join();
    eprintln!("PROBE {n}: {:?}", r);
}
#[test]
fn probes() {
    for (l,g) in [(1,60),(8,60),(24,60),(1,63),(1,64),(1,200)] {
        run(&format!("sig {l}x{g}"), format!("pub fn go(x: {}) -> u8 {{ 0 }}\n", deep(l,g)));
        run(&format!("ret {l}x{g}"), format!("pub fn go() -> {} {{ loop {{}} }}\n", deep(l,g)));
        run(&format!("where {l}x{g}"), format!("pub fn go<T>(x: T) where T: Into<{}> {{ }}\n", deep(l,g)));
        run(&format!("const {l}x{g}"), format!("pub const N: {} = (0,);\n", deep(l,g)));
    }
    for d in [50, 120, 127, 200, 1000, 5000] {
        run(&format!("parens init {d}"), format!("pub const N: u8 = {}1{};\n", "(".repeat(d), ")".repeat(d)));
        run(&format!("blocks init {d}"), format!("pub static N: u8 = {}1{};\n", "{".repeat(d), "}".repeat(d)));
        run(&format!("arr init {d}"), format!("pub const N: u8 = {}1{};\n", "[".repeat(d), "]".repeat(d)));
        run(&format!("body {d}"), format!("pub fn f() -> u8 {{ {}1{} }}\n", "{".repeat(d), "}".repeat(d)));
        run(&format!("impl method {d}"), format!("pub struct S; impl S {{ pub const C: u8 = {}1{}; pub fn m(&self, (a,b): (u8,u8)) -> u8 {{ a }} }}\n", "(".repeat(d), ")".repeat(d)));
    }
}
