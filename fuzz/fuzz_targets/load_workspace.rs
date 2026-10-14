// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Builds a small workspace in memory from the input, loads it through
//! `Workspace::load` and asks the index what a tool asks. The input format is
//! in `workspace_input.rs`, which the seed replay test shares.
//!
//! A tree that refuses its keys, a load error or a resolution error is a
//! normal result. A panic, a stack overflow or unbounded memory is a finding.
//! Before the first input a seed with items must load with its items: a
//! build that lost the positions of parsed syntax fails there and does not
//! run on, rejecting every input as a load error.

#![no_main]

mod workspace_input;

use std::sync::Once;

use libfuzzer_sys::fuzz_target;
use raf_source::Workspace;

/// Runs the self-check once per process.
static SELF_CHECK: Once = Once::new();

fuzz_target!(|data: &[u8]| {
    SELF_CHECK.call_once(workspace_input::self_check);
    let Some(tree) = workspace_input::tree(data) else {
        return;
    };
    if let Ok(workspace) = Workspace::load(&tree, &["."]) {
        std::hint::black_box(workspace_input::exercise(&workspace));
    }
});
