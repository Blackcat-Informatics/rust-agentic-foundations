// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Exercises the public normalizer and a complete census over arbitrary source.
//!
//! Syntax and budget errors are expected outcomes; a panic, overflow or missing
//! source positions is a failure. The self-check proves valid input reaches a
//! unit before mutations run. Span storage is cleared between fuzz iterations.

#![no_main]

use std::sync::Once;

use libfuzzer_sys::fuzz_target;
use raf_census::{Census, CensusOptions, Normalizer, Slice};
use raf_source::MemoryTree;

static SELF_CHECK: Once = Once::new();

fn load(source: &str) -> Result<Census, raf_census::Error> {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='fuzz'\nversion='0.1.0'\nedition='2024'\n",
        ),
        ("src/lib.rs", source),
    ]);
    Census::load(&tree, &["."], &CensusOptions::default())
}

fn exercise(data: &[u8]) {
    SELF_CHECK.call_once(|| {
        let census = load("pub fn check(value: u8) -> u8 { value + 1 }")
            .expect("valid source must reach the census");
        assert_eq!(census.units().len(), 1);
        let unit = &census.units()[0];
        assert_eq!(unit.symbol, "fuzz::check");
        assert_eq!(unit.file, "src/lib.rs");
        assert_eq!(unit.line, 1);
        assert_eq!(unit.params, ["value"]);
        assert!(unit.print.is_some());
    });
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let options = CensusOptions::default();
    drop(Normalizer::new(&options).print(&Slice::new("src/lib.rs", 1, text)));
    // No token or span survives the print call.
    proc_macro2::extra::invalidate_current_thread_spans();
    drop(load(text));
}

fuzz_target!(|data: &[u8]| exercise(data));
