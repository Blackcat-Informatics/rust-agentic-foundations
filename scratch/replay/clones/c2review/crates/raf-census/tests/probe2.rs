//! probe
use raf_census::{CensusOptions, Normalizer, Slice};
#[test]
fn normalizer_deep() {
    for d in [100, 1000, 100000] {
        let src = format!("{}1{}", "(".repeat(d), ")".repeat(d));
        let r = std::thread::Builder::new().stack_size(2<<20).spawn(move || {
            let n = Normalizer::new(&CensusOptions::default());
            match n.print(&Slice::new("a.rs", 1, &src)) { Ok(_) => "ok".to_string(), Err(e) => format!("err {}", e.to_string().chars().take(80).collect::<String>()) }
        }).unwrap().join();
        eprintln!("PROBE2 {d}: {:?}", r);
    }
}
