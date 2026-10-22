// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Feeds arbitrary text through the pre-scan and the parse of `raf-source`.
//!
//! A refusal or a syntax error is a normal result. A panic or a stack
//! overflow is a finding.

#![no_main]

use libfuzzer_sys::fuzz_target;
use raf_source::{RelPath, parse_source};

fuzz_target!(|data: &[u8]| {
    // Source text is UTF-8; bytes that are not are not a candidate file.
    if let Ok(text) = std::str::from_utf8(data) {
        let file = RelPath::new("fuzz/input.rs").expect("a fixed relative path");
        let outcome = parse_source(&file, text, |tree| tree.items.len());
        drop(outcome);
    }
});
