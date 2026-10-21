//! scratch
use raf_census::*;
fn t(text: &str, params: &[&str]) -> Vec<String> {
    Normalizer::new(&CensusOptions::default()).tower(&Slice::new("a.rs", 1, text).params(params.iter().copied())).unwrap().form(Level::Value).texts()
}
#[test]
fn probe() {
    println!("DOC {:?}", t("/// doc\nlet x = f(a); x // c\n", &["a"]));
    println!("PLAIN {:?}", t("// doc\nlet x = f(a); x /* c */\n", &["a"]));
    println!("REFMUT {:?}", t("let ref mut z = a; z", &["a"]));
    println!("CSTR {:?}", t("c\"x\"", &[]));
}
