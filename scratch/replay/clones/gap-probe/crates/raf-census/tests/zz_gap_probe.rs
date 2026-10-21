// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
//! Throwaway gap-analysis probes; never committed.
#![allow(missing_docs, clippy::all, clippy::pedantic)]

use raf_census::{Baseline, Census, CensusOptions, Level, Normalizer, Slice};
use raf_source::MemoryTree;

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

#[test]
fn probe_noise_across_the_literal_tier() {
    let n = Normalizer::new(&CensusOptions::default());
    let clean = Slice::new("a.rs", 1, "let n = v.len(); if n > 3 { return 7; } n + 1").params(["v"]);
    let noisy = Slice::new("b.rs", 1, "let n = v.clone().as_slice().to_owned().len(); if n > 3 { return 7; } n + 1").params(["v"]);
    let tc = n.tower(&clean).unwrap();
    let tn = n.tower(&noisy).unwrap();
    println!("clean structural={} value={}", tc.structural_tokens(), tc.form(Level::Value).len());
    println!("noisy structural={} value={}", tn.structural_tokens(), tn.form(Level::Value).len());
    println!("clean value: {:?}", tc.form(Level::Value).texts().join(" "));
    println!("noisy value: {:?}", tn.form(Level::Value).texts().join(" "));
    let d = n.explain(&clean, &noisy).unwrap();
    println!("explain: equal_from={:?} differs_through={:?}", d.equal_from, d.differs_through);
    // Control: the same noise below the tier collapses.
    let clean2 = Slice::new("a.rs", 1, "let n = v.len(); n + 1").params(["v"]);
    let noisy2 = Slice::new("b.rs", 1, "let n = v.clone().len(); n + 1").params(["v"]);
    let d2 = n.explain(&clean2, &noisy2).unwrap();
    println!("control explain: equal_from={:?}", d2.equal_from);
}

#[test]
fn probe_noise_across_min_tokens() {
    let n = Normalizer::new(&CensusOptions::default());
    let clean = Slice::new("a.rs", 1, "if v.len() > 3 { 1 } else { 0 }").params(["v"]);
    let noisy = Slice::new("b.rs", 1, "if v.clone().as_slice().len() > 3 { 1 } else { 0 }").params(["v"]);
    let pc = n.print(&clean).unwrap();
    let pn = n.print(&noisy).unwrap();
    println!("clean tokens={} body={:?}", pc.tokens, pc.body.as_ref().map(|h| h.fingerprint.to_string()));
    println!("noisy tokens={} body={:?}", pn.tokens, pn.body.as_ref().map(|h| h.fingerprint.to_string()));
}

#[test]
fn probe_report_of_another_normal_form_reads_back() {
    let tree: MemoryTree = [("Cargo.toml", MANIFEST), ("src/lib.rs", "pub fn f(a: u8) -> u8 { a }\n")].into_iter().collect();
    let census = Census::load(&tree, &["."], &CensusOptions::default()).unwrap();
    let text = census.baseline().to_json().unwrap().replace("\"normal_form\": 1", "\"normal_form\": 7");
    let read = Baseline::from_json(&text);
    println!("baseline with normal_form 7 reads: {:?}", read.map(|b| b.normal_form));
}

fn nested(k: usize, per: usize) -> String {
    let mut t = "u8".to_string();
    for _ in 0..k {
        t = format!("({}{}{},)", "A<".repeat(per), t, ">".repeat(per));
    }
    t
}

fn deep_tree(k: usize) -> MemoryTree {
    let src = format!("pub fn f(x: {}) -> u8 {{ 0 }}\n", nested(k, 60));
    let manifest: &'static str = MANIFEST;
    let src: &'static str = Box::leak(src.into_boxed_str());
    [("Cargo.toml", manifest), ("src/lib.rs", src)].into_iter().collect()
}

#[test]
fn probe_deep_signature_raf_source_alone_2mib() {
    let k: usize = std::env::var("PROBE_K").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let r = std::thread::Builder::new().stack_size(2 << 20).spawn(move || {
        let tree = deep_tree(k);
        raf_source::Workspace::load_with(&tree, &["."], &raf_source::LoadOptions::default()).map(|_| ()).map_err(|e| e.to_string())
    }).unwrap().join().unwrap();
    println!("raf-source load k={k} on a 2 MiB thread: {r:?}");
}

#[test]
fn probe_deep_signature_census_2mib() {
    let k: usize = std::env::var("PROBE_K").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let r = std::thread::Builder::new().stack_size(2 << 20).spawn(move || {
        let tree = deep_tree(k);
        Census::load(&tree, &["."], &CensusOptions::default()).map(|c| c.units().len()).map_err(|e| e.to_string())
    }).unwrap().join().unwrap();
    println!("census load k={k} on a 2 MiB thread: {r:?}");
}

#[test]
fn probe_deep_signature_census_8mib() {
    let k: usize = std::env::var("PROBE_K").ok().and_then(|v| v.parse().ok()).unwrap_or(24);
    let r = std::thread::Builder::new().stack_size(8 << 20).spawn(move || {
        let tree = deep_tree(k);
        Census::load(&tree, &["."], &CensusOptions::default()).map(|c| c.units().len()).map_err(|e| e.to_string())
    }).unwrap().join().unwrap();
    println!("census load k={k} on an 8 MiB thread: {r:?}");
}

fn run(stack: usize, src: String) -> Result<usize, String> {
    std::thread::Builder::new().stack_size(stack).spawn(move || {
        let src: &'static str = Box::leak(src.into_boxed_str());
        let tree: MemoryTree = [("Cargo.toml", MANIFEST), ("src/lib.rs", src)].into_iter().collect();
        Census::load(&tree, &["."], &CensusOptions::default()).map(|c| c.units().len()).map_err(|e| e.to_string())
    }).unwrap().join().unwrap()
}

#[test]
fn probe_deep_type_in_body_2mib() {
    let per: usize = std::env::var("PROBE_PER").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    let ty = format!("{}u8{}", "A<".repeat(per), ">".repeat(per));
    println!("body-deep per={per}: {:?}", run(2 << 20, format!("pub fn f() -> u8 {{ let x: {ty} = 0; x }}\n")));
}

#[test]
fn probe_deep_type_in_signature_2mib() {
    let per: usize = std::env::var("PROBE_PER").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    let ty = format!("{}u8{}", "A<".repeat(per), ">".repeat(per));
    println!("sig-deep per={per}: {:?}", run(2 << 20, format!("pub fn f(x: {ty}) -> u8 {{ 0 }}\n")));
}

#[test]
fn probe_deep_const_initializer_2mib() {
    let per: usize = std::env::var("PROBE_PER").ok().and_then(|v| v.parse().ok()).unwrap_or(120);
    let e = format!("{}1{}", "(".repeat(per), ")".repeat(per));
    println!("const-deep per={per}: {:?}", run(2 << 20, format!("pub const C: u8 = {e};\n")));
}
