// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Feeds arbitrary text through `Document::parse`, the body model and
//! `Profile::validate` of `raf-okf`.
//!
//! A parse error or a list of violations is a normal result. A panic, a stack
//! overflow or unbounded memory is a finding.

#![no_main]

use libfuzzer_sys::fuzz_target;
use raf_okf::{Document, Profile};

fuzz_target!(|data: &[u8]| {
    // Documents are UTF-8; bytes that are not are not a candidate file.
    if let Ok(text) = std::str::from_utf8(data)
        && let Ok(document) = Document::parse(text)
    {
        // Read the body model the way a tool does, then validate.
        let body = document.body();
        let sizes = [
            body.headings().len(),
            body.sections().len(),
            body.links().len(),
            body.prose_lines().len(),
            body.proposed_blocks().len(),
        ];
        std::hint::black_box(sizes);
        drop(Profile.validate(&document, "docs/capabilities/fuzz-input.md"));
    }
});
