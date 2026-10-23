// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Compare edition-specific code and import reading contexts.

use raf_source::{MemoryTree, Namespace, Workspace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source =
        "pub fn selected(){} pub mod child { pub fn selected(){} pub fn reader(){selected();} }";
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='demo'\nversion='0.1.0'\nedition='2015'\n",
        ),
        ("src/lib.rs", source),
    ]);
    let workspace = Workspace::load(&tree, &["."])?;
    let reader = workspace
        .resolve_in("demo::child::reader", Namespace::Value)?
        .sites[0];
    let byte = source.find("selected();").expect("the code observation");
    let code = workspace.code_resolver()?;
    assert_eq!(code.reading_module(reader, byte)?, "demo::child");
    assert_eq!(
        code.resolve_from_item_at(reader, byte, "selected", Namespace::Value)?
            .path,
        "demo::child::selected"
    );
    assert_eq!(
        workspace
            .resolve_from_in("demo::child", "selected", Namespace::Value)?
            .path,
        "demo::selected"
    );
    println!("code and import heads retain their distinct reading contexts");
    Ok(())
}
