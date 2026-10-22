// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

use std::fmt::Write as _;
use raf_source::{LoadOptions, MemoryTree, Workspace};
use raf_workplan::Workplan;

fn main() {
    let mut lib = "mod z {}\npub use c0::*;\n".to_string();
    for i in 0..16 {
        writeln!(lib, "mod c{i} {{ pub use crate::c{}::*; }}", i + 1).unwrap();
    }
    lib.push_str("mod c16 { pub use crate::m::*; pub use crate::a::*; pub use crate::b::*; }\n");
    let (mut carried, mut bound) = (String::new(), String::new());
    for i in 0..8 {
        writeln!(carried, "pub use std::cell::Cell as C{i};").unwrap();
        writeln!(bound, "fn C{i}() {{}}").unwrap();
    }
    writeln!(lib, "mod m {{ {carried} }}\nmod a {{ {bound} pub use crate::z::*; }}\nmod b {{ {bound} pub use crate::z::*; }}").unwrap();
    let tree = MemoryTree::from([
        ("probe/Cargo.toml", "[package]\nname=\"probe\"\nversion=\"0.1.0\"\nedition=\"2021\"\n"),
        ("probe/src/lib.rs", lib.as_str()),
    ]);
    let spent = Workspace::load(&tree, &["probe"]).unwrap().usage().item_bytes;
    let docs = MemoryTree::default();
    for extra in (256..=65536).step_by(256) {
        let mut options = LoadOptions::default();
        options.max_item_bytes = spent + extra;
        let source = Workspace::load_with(&tree, &["probe"], &options).unwrap();
        let report = Workplan::load(&docs, &source).coverage_report();
        if report.errors().iter().any(|e| matches!(e.failure(), raf_workplan::coverage::SurfaceFailure::CarryBudget { .. })) {
            println!("spent={spent} extra={extra} error={:?}", report.errors());
            return;
        }
    }
    panic!("no carried-name budget boundary found");
}
