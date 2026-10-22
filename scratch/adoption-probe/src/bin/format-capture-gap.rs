// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A public-source-API reproduction of implicit format dependencies.

use raf_source::{MemoryTree, Namespace, Node, Workspace};

const SOURCE: &str = r#"
const VALUE: u32 = 7;
pub fn captured() -> String { format!("{VALUE}") }
pub fn direct() -> String { format!("{}", VALUE) }
pub fn escaped() -> String { format!("{{VALUE}}") }
pub fn ordinary() -> &'static str { "{VALUE}" }
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = Workspace::load(&MemoryTree::from([
        ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n"),
        ("src/lib.rs", SOURCE),
    ]), &["."])?;
    let value = Node::Item(source.resolve_in("demo::VALUE", Namespace::Value)?.sites[0]);
    for name in ["captured", "direct", "escaped", "ordinary"] {
        let root = Node::Item(source.resolve_in(&format!("demo::{name}"), Namespace::Value)?.sites[0]);
        let reached = source.reachable(&[root])?;
        println!("{name}: VALUE reached = {}", reached.contains(value));
        assert_eq!(reached.contains(value), name == "direct");
    }
    Ok(())
}
