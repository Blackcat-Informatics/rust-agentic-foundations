// Completion-audit probes. Scratch only; never committed.
#![allow(missing_docs, clippy::all, clippy::pedantic)]

use raf_census::{Census, CensusOptions, Normalizer, Slice};
use raf_source::MemoryTree;

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

fn load_files(files: &[(&str, &str)]) -> Census {
    let mut all = vec![("Cargo.toml", MANIFEST)];
    all.extend_from_slice(files);
    let tree: MemoryTree = all.into_iter().collect();
    Census::load(&tree, &["."], &CensusOptions::default()).expect("loads")
}

fn packages(sources: &[(&str, &str)]) -> Census {
    let members: Vec<String> = sources.iter().map(|(n, _)| format!("\"{n}\"")).collect();
    let mut files = vec![(
        "Cargo.toml".to_owned(),
        format!("[workspace]\nmembers = [{}]\nresolver = \"2\"\n", members.join(", ")),
    )];
    for (name, source) in sources {
        files.push((
            format!("{name}/Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        ));
        files.push((format!("{name}/src/lib.rs"), (*source).to_owned()));
    }
    let tree: MemoryTree = files.iter().map(|(p, t)| (p.as_str(), t.as_str())).collect();
    Census::load(&tree, &["."], &CensusOptions::default()).expect("loads")
}

fn show(census: &Census, label: &str) {
    println!("== {label}");
    for u in census.units() {
        let p = u.print.as_ref();
        println!(
            "  unit {} kind={:?} pkg={} file={} line={} shipping={} tokens={:?} body={:?} shim={:?} shim_key={:?} trait_impl={} via_self={} consts={:?} tables={:?} strings={:?}",
            u.symbol,
            u.kind,
            u.package,
            u.file,
            u.line,
            u.shipping,
            p.map(|p| p.tokens),
            p.and_then(|p| p.body.as_ref()).map(|h| h.fingerprint.to_string()),
            p.and_then(|p| p.shim.as_ref()).map(|h| h.fingerprint.to_string()),
            u.shim_key.as_ref().map(ToString::to_string),
            u.trait_impl,
            u.via_self,
            u.constants,
            u.tables,
            u.strings.iter().map(|s| (s.value.clone(), s.line)).collect::<Vec<_>>(),
        );
    }
    for g in census.groups() {
        println!(
            "  group {:?} {} tokens={} members={:?}",
            g.kind,
            g.fingerprint,
            g.tokens,
            g.member_symbols().collect::<Vec<_>>()
        );
    }
    for t in census.hex_tables() {
        println!("  hex {} {:?}", t.member.symbol, t.table);
    }
    for c in census.shared_constants() {
        println!(
            "  shared {:#x} packages={} members={:?}",
            c.value,
            c.packages,
            c.members.iter().map(|m| m.symbol.clone()).collect::<Vec<_>>()
        );
    }
}

fn explain(label: &str, a: &str, pa: &[&str], b: &str, pb: &[&str]) {
    let n = Normalizer::new(&CensusOptions::default());
    let sa = Slice::new("a.rs", 1, a).params(pa.iter().copied());
    let sb = Slice::new("b.rs", 1, b).params(pb.iter().copied());
    let d = n.explain(&sa, &sb).expect("lexes");
    let ta = n.tower(&sa).expect("a");
    let tb = n.tower(&sb).expect("b");
    let pra = n.print(&sa).expect("a");
    let prb = n.print(&sb).expect("b");
    println!("== {label}");
    println!("  a value: {}", ta.form(raf_census::Level::Value).texts().join(" "));
    println!("  b value: {}", tb.form(raf_census::Level::Value).texts().join(" "));
    println!(
        "  tokens a={} b={} equal_from={:?} differs_through={:?} body_a={:?} body_b={:?}",
        pra.tokens,
        prb.tokens,
        d.equal_from,
        d.differs_through,
        pra.body.as_ref().map(|h| h.fingerprint.to_string()),
        prb.body.as_ref().map(|h| h.fingerprint.to_string())
    );
}

#[test]
fn p01_item_used_as_a_value_is_kept() {
    // Two bodies that pass different function items by name.
    let src = "\
pub fn double(x: u8) -> u8 { x * 2 }
pub fn triple(x: u8) -> u8 { x * 3 }
pub fn is_small(x: &u8) -> bool { *x < 10 }
pub fn is_big(x: &u8) -> bool { *x > 200 }
pub fn first(xs: &[u8]) -> Vec<u8> { xs.iter().copied().map(double).filter(is_small).collect::<Vec<u8>>() }
pub fn second(xs: &[u8]) -> Vec<u8> { xs.iter().copied().map(triple).filter(is_big).collect::<Vec<u8>>() }
pub fn third(xs: &[u8]) -> Vec<u8> { xs.iter().copied().map(|v| double(v)).filter(|v| is_small(v)).collect::<Vec<u8>>() }
pub fn fourth(xs: &[u8]) -> Vec<u8> { xs.iter().copied().map(|v| triple(v)).filter(|v| is_big(v)).collect::<Vec<u8>>() }
";
    let census = load_files(&[("src/lib.rs", src)]);
    show(&census, "p01 item used as a value");
    explain(
        "p01 explain first vs second",
        "xs.iter().copied().map(double).filter(is_small).collect::<Vec<u8>>()",
        &["xs"],
        "xs.iter().copied().map(triple).filter(is_big).collect::<Vec<u8>>()",
        &["xs"],
    );
}

#[test]
fn p02_lowercase_static_and_const_item_used() {
    let src = "\
pub static limit_a: u32 = 7;
pub static limit_b: u32 = 9;
pub fn ua(v: u32) -> u32 { let mut t = v; t = t + limit_a; t = t * limit_a; t = t - limit_a; t }
pub fn ub(v: u32) -> u32 { let mut t = v; t = t + limit_b; t = t * limit_b; t = t - limit_b; t }
";
    let census = load_files(&[("src/lib.rs", src)]);
    show(&census, "p02 lowercase statics");
}

#[test]
fn p03_block_after_capitalised_name() {
    // A block after a capitalised constant is read as a struct literal.
    explain(
        "p03 if x == MAX { local } with renamed local",
        "let total = x + 1; let other = total * 2; if x == MAX { total } else { other }",
        &["x"],
        "let sum = x + 1; let next = sum * 2; if x == MAX { sum } else { next }",
        &["x"],
    );
}

#[test]
fn p04_collapse_and_non_collapse_through_load() {
    let src = "\
pub fn a(items: &Vec<String>, key: &str) -> usize { let mut count: usize = 0; for item in items.iter() { if item.as_str() == key { count += 1; } } count }
pub fn b(list: &Vec<String>, wanted: &str) -> usize { let mut n = 0; for entry in list.iter() { if *entry.clone() == *wanted.to_owned() { n += 1; } } n }
pub fn c(list: &Vec<String>, wanted: &str) -> usize { let mut n = 0; for entry in list.iter() { if entry.starts_with(wanted) { n += 1; } } n }
pub fn d(v: u32) -> u32 { let mut acc = v; acc = acc ^ 0xff; acc = acc.rotate_left(3); acc = acc ^ 0x10; acc }
pub fn e(w: u32) -> u32 { let mut z = w; z = z ^ 255; z = z.rotate_left(3); z = z ^ 16; z }
pub fn f(w: u32) -> u32 { let mut z = w; z = z ^ 254; z = z.rotate_left(3); z = z ^ 16; z }
";
    let census = load_files(&[("src/lib.rs", src)]);
    show(&census, "p04 collapse");
}

#[test]
fn p05_shim_rules_through_load() {
    let src = "\
pub fn target(a: u32, b: u32) -> u32 { a + b }
pub fn s1(x: u32) -> u32 { target(x, 7) }
pub fn s2(y: u32) -> u32 { target(y, 7) }
pub fn s3(y: u32) -> u32 { target(y, 8) }
pub struct S;
impl S {
    fn helper(a: u32, b: u32) -> u32 { a + b }
    pub fn s4(z: u32) -> u32 { Self::helper(z, 7) }
    pub fn s5(w: u32) -> u32 { Self::helper(w, 7) }
    pub fn s6(w: u32) -> u32 { target(w, 7) }
}
pub trait T { fn go(&self, q: u32) -> u32; }
impl T for S { fn go(&self, q: u32) -> u32 { target(q, 7) } }
pub trait U { fn run(q: u32) -> u32; }
impl U for S { fn run(q: u32) -> u32 { target(q, 7) } }
pub fn s7(x: u32) -> u32 { let t = 7; target(x, t) }
pub fn s8(x: u32) -> u32 { target(x, 7) + target(x, 7) + target(x, 7) + target(x, 7) }
";
    let census = load_files(&[("src/lib.rs", src)]);
    show(&census, "p05 shim");
}

#[test]
fn p06_units_and_shipping() {
    let lib = "\
/// docs
#[inline]
pub fn shipped(a: u8) -> u8 { a }
pub const K: u32 = 0x1_0000;
pub static ST: [char; 16] = ['0','1','2','3','4','5','6','7','8','9','A','B','C','D','E','F'];
pub struct W;
impl W { pub fn m(&self) -> u8 { 1 } }
#[cfg(test)]
mod tests {
    pub fn only_test(b: u8) -> u8 { b }
}
";
    let bin = "fn main() { let _ = demo::shipped(1); }\npub fn in_bin() -> &'static [u8] { b\"0123456789abcdef\" }\n";
    let test = "pub fn in_test() -> [u8; 16] { [b'0',b'1',b'2',b'3',b'4',b'5',b'6',b'7',b'8',b'9',b'a',b'b',b'c',b'd',b'e',b'f'] }\n";
    let example = "fn main() {}\npub fn in_example() -> u32 { 65_536 }\n";
    let census = load_files(&[
        ("src/lib.rs", lib),
        ("src/main.rs", bin),
        ("tests/t.rs", test),
        ("examples/ex.rs", example),
    ]);
    show(&census, "p06 units");
    for f in census.files() {
        println!("  file {} pkg={} shipping={}", f.path, f.package, f.shipping);
    }
}

#[test]
fn p07_shared_constants_by_value() {
    let census = packages(&[
        ("pa", "pub fn fa() -> u64 { 0x10000 }\npub const SMALL: u32 = 0xffff;\n"),
        ("pb", "pub const FB: u64 = 65_536;\npub const SMALL2: u32 = 65535;\n"),
        ("pc", "pub fn fc() -> u64 { 0o200000u64 }\n"),
    ]);
    show(&census, "p07 shared constants");
}

#[test]
fn p08_macro_arguments() {
    explain(
        "p08 macro args renamed locals",
        "let total = a + b; println!(\"{} {}\", total, a); assert_eq!(total, a + b); vec![total; 3]",
        &["a", "b"],
        "let sum = x + y; println!(\"{} {}\", sum, x); assert_eq!(sum, x + y); vec![sum; 3]",
        &["x", "y"],
    );
    explain(
        "p08b macro args different callee",
        "let total = a + b; println!(\"{} {}\", total, a); assert_eq!(total, foo(a)); vec![total; 3]",
        &["a", "b"],
        "let sum = x + y; println!(\"{} {}\", sum, x); assert_eq!(sum, bar(x)); vec![sum; 3]",
        &["x", "y"],
    );
}

#[test]
fn p09_determinism_fresh_loads() {
    let src = "\
pub fn alpha(a: u32) -> u32 { a + a + a + a + a + a + a + a + a + a + a }
pub fn beta(b: u32) -> u32 { b + b + b + b + b + b + b + b + b + b + b }
pub fn gamma(c: u32) -> u32 { c * c * c * c * c * c * c * c * c * c * c }
pub fn delta(d: u32) -> u32 { d * d * d * d * d * d * d * d * d * d * d }
pub fn eps(d: u32) -> u32 { d * d * d * d * d * d * d * d * d * d * d }
";
    let one = load_files(&[("src/lib.rs", src)]);
    let two = load_files(&[("src/lib.rs", src)]);
    let a = format!("{}{}", one.baseline().to_json().unwrap(), one.index(&[]).to_json().unwrap());
    let b = format!("{}{}", two.baseline().to_json().unwrap(), two.index(&[]).to_json().unwrap());
    println!("== p09 identical={}", a == b);
    show(&one, "p09 ordering");
}

#[test]
fn p10_value_form_each_erasure() {
    let base = "let mut acc = 0; for v in xs.iter() { acc = acc + v.len(); } let s = name; s.len() + acc";
    let cases = [
        ("borrow", "let mut acc = 0; for v in &xs.iter() { acc = acc + v.len(); } let s = &name; s.len() + acc"),
        ("mut borrow", "let mut acc = 0; for v in xs.iter() { acc = acc + v.len(); } let s = &mut name; s.len() + acc"),
        ("deref", "let mut acc = 0; for v in xs.iter() { acc = acc + (*v).len(); } let s = *name; s.len() + acc"),
        ("ref", "let mut acc = 0; for ref v in xs.iter() { acc = acc + v.len(); } let s = name; s.len() + acc"),
        ("clone", "let mut acc = 0; for v in xs.iter() { acc = acc + v.clone().len(); } let s = name.clone(); s.len() + acc"),
        ("as_str", "let mut acc = 0; for v in xs.iter() { acc = acc + v.as_str().len(); } let s = name.as_str(); s.len() + acc"),
        ("to_owned", "let mut acc = 0; for v in xs.iter() { acc = acc + v.len(); } let s = name.to_owned(); s.len() + acc"),
        ("full range", "let mut acc = 0; for v in xs[..].iter() { acc = acc + v.len(); } let s = name; s.len() + acc"),
        ("let annot", "let mut acc: usize = 0; for v in xs.iter() { acc = acc + v.len(); } let s: &str = name; s.len() + acc"),
        ("turbofish", "let mut acc = 0; for v in xs.iter() { acc = acc + v.len(); } let s = name; s.len() + acc + std::mem::size_of::<u8>()"),
        ("binary and kept", "let mut acc = 0; for v in xs.iter() { acc = acc & v.len(); } let s = name; s.len() + acc"),
    ];
    for (label, text) in cases {
        explain(&format!("p10 {label}"), base, &["xs", "name"], text, &["xs", "name"]);
    }
}

#[test]
fn p11_literal_tier() {
    explain(
        "p11 short bodies differ by string",
        "match k { 1 => \"one\", 2 => \"two\", _ => \"many\" }",
        &["k"],
        "match k { 1 => \"uno\", 2 => \"dos\", _ => \"muchos\" }",
        &["k"],
    );
    explain(
        "p11 long bodies differ by string",
        "match k { 1 => \"one\", 2 => \"two\", 3 => \"three\", 4 => \"four\", 5 => \"five\", _ => \"many\" }",
        &["k"],
        "match k { 1 => \"uno\", 2 => \"dos\", 3 => \"tres\", 4 => \"cuatro\", 5 => \"cinco\", _ => \"muchos\" }",
        &["k"],
    );
}

#[test]
fn p12_lifetimes_and_labels() {
    explain(
        "p12 lifetimes/labels renamed",
        "let r: &'a str = s; 'outer: for c in r.chars() { if c == 'x' { break 'outer; } } r.len() + s.len()",
        &["s"],
        "let q: &'b str = t; 'scan: for d in q.chars() { if d == 'x' { break 'scan; } } q.len() + t.len()",
        &["t"],
    );
}

#[test]
fn p13_parameter_order() {
    let src = "\
pub fn count_a(xs: &[u32], k: u32) -> usize { let mut n = 0; for x in xs.iter() { if *x == k { n += 1; } } n + xs.len() }
pub fn count_b(k: u32, xs: &[u32]) -> usize { let mut n = 0; for x in xs.iter() { if *x == k { n += 1; } } n + xs.len() }
pub fn count_c(ys: &[u32], j: u32) -> usize { let mut m = 0; for y in ys.iter() { if *y == j { m += 1; } } m + ys.len() }
";
    let census = load_files(&[("src/lib.rs", src)]);
    show(&census, "p13 parameter order");
    explain(
        "p13 explain count_a vs count_b",
        "let mut n = 0; for x in xs.iter() { if *x == k { n += 1; } } n + xs.len()",
        &["xs", "k"],
        "let mut n = 0; for x in xs.iter() { if *x == k { n += 1; } } n + xs.len()",
        &["k", "xs"],
    );
}

#[test]
fn p14_encoding_by_hand_and_version_gate() {
    let n = Normalizer::new(&CensusOptions::default());
    let s = Slice::new("a.rs", 1, "a + a + a + a + a + a + a + a + a + a + a").params(["a"]);
    let tower = n.tower(&s).unwrap();
    let mut bytes = Vec::new();
    for t in tower.form(raf_census::Level::Value).texts() {
        bytes.extend_from_slice(&(t.len() as u32).to_le_bytes());
        bytes.extend_from_slice(t.as_bytes());
    }
    let digest = blake3::hash(&bytes);
    let first8 = u64::from_be_bytes(digest.as_bytes()[..8].try_into().unwrap());
    let by_hand = format!("body:{first8:016x}");
    let print = n.print(&s).unwrap();
    println!("== p14 by_hand={by_hand} census={:?}", print.body.as_ref().map(|h| h.fingerprint.to_string()));
    for text in ["body:6bb022e241a79a28", "shim:203616cb54a3e53d", "table:hex-lower", "table:hex-upper", "rule:std-default-hasher", "blob:6bb022e241a79a28", "body:6bb022e241a79a2", "body:6BB022E241A79A28", "table:hex-mixed", "rule:"] {
        let parsed: Result<raf_census::Fingerprint, _> = text.parse();
        println!("  parse {text} -> {:?}", parsed.map(|f| f.to_string()));
    }
    for v in [0u32, 1, 2] {
        println!("  check_normal_form({v}) -> {:?}", raf_census::check_normal_form(v).map_err(|e| e.to_string()));
    }
}

#[test]
fn p15_block_after_capitalised_name_through_load() {
    let src = "\
pub const MAX: u32 = 100;
pub fn clamp_a(value: u32, step: u32) -> u32 { let next = value + step; let fallback = value * 2; if next == MAX { next } else { fallback } }
pub fn clamp_b(input: u32, delta: u32) -> u32 { let later = input + delta; let other = input * 2; if later == MAX { later } else { other } }
pub fn clamp_c(input: u32, delta: u32) -> u32 { let later = input + delta; let other = input * 2; if later == LIMIT { later } else { other } }
pub const LIMIT: u32 = 100;
pub fn clamp_d(value: u32, step: u32) -> u32 { let next = value + step; let fallback = value * 2; if next == limit() { next } else { fallback } }
pub fn clamp_e(input: u32, delta: u32) -> u32 { let later = input + delta; let other = input * 2; if later == limit() { later } else { other } }
pub fn limit() -> u32 { 100 }
";
    let census = load_files(&[("src/lib.rs", src)]);
    show(&census, "p15 block after capitalised name");
}
