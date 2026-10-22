// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
use raf_source::{MemoryTree, Workspace};
use raf_workplan::{Workplan, report, coverage::GapKind};
use raf_rdf::{Format, Graph};
fn write(name: &str, graph: Graph) {
    for (suffix, format) in [("ttl", Format::Turtle), ("nt", Format::NTriples)] {
        std::fs::write(format!("crates/raf-workplan/tests/fixtures/reports/{name}.{suffix}"), graph.to_text(format).unwrap()).unwrap();
    }
}
fn main() {
    let source = Workspace::load(&MemoryTree::from([("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\n"), ("src/lib.rs", "")]), &["."]).unwrap();
    let docs = MemoryTree::from([("docs/architecture/broken.md", "not a document\n")]);
    write("findings", report::findings_graph(&Workplan::load(&docs, &source).check()).unwrap());
    let source = Workspace::load(&MemoryTree::from([
        ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\n"),
        ("src/lib.rs", "pub fn open() {}\npub fn excluded() {}\n#[cfg(test)] mod tests { #[test] fn case() {} }\n"),
    ]), &["."]).unwrap();
    let docs = MemoryTree::from([("work/coverage/default.md", "---\ntype: Coverage\nid: coverage:default\nscope: [demo]\nexclusions:\n  - item: demo::excluded\n    reason: A deliberate example.\n---\n")]);
    let coverage = Workplan::load(&docs, &source).coverage_report();
    write("coverage-code", report::coverage_graph(&coverage, GapKind::PublicItem).unwrap());
    write("coverage-tests", report::coverage_graph(&coverage, GapKind::Test).unwrap());
    let source = Workspace::load(&MemoryTree::from([
        ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\n"),
        ("src/lib.rs", "pub mod left { pub fn dup() {} } pub mod right { pub fn dup() {} } pub use left::*; pub use right::*;"),
    ]), &["."]).unwrap();
    let coverage = Workplan::load(&MemoryTree::default(), &source).coverage_report();
    write("coverage-error", report::coverage_graph(&coverage, GapKind::PublicItem).unwrap());
}
