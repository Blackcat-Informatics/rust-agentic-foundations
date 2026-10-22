// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Observe ordinary-path false positives at a format argument binding.

use raf_source::{MemoryTree, Namespace, Node, Workspace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = Workspace::load(&MemoryTree::from([
        ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n"),
        ("src/lib.rs", r#"
const VALUE: u32 = 7;
const OTHER: u32 = 11;
pub fn named() -> String { format!("{VALUE}", VALUE = OTHER) }
pub fn direct() -> String { format!("{}", VALUE) }
pub fn ordinary() -> &'static str { "{VALUE}" }
"#),
    ]), &["."])?;
    for name in ["named", "direct", "ordinary"] {
        let root = Node::Item(workspace.resolve_in(&format!("demo::{name}"), Namespace::Value)?.sites[0]);
        let reached = workspace.reachable(&[root])?;
        for constant in ["VALUE", "OTHER"] {
            let node = Node::Item(workspace.resolve_in(&format!("demo::{constant}"), Namespace::Value)?.sites[0]);
            println!("{name}: {constant} reached = {}", reached.contains(node));
        }
    }
    Ok(())
}
