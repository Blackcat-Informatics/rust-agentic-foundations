// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Counter admission and integer cost bounds of the real instruction comparator.

#[path = "../examples/glob_instruction_gate.rs"]
#[allow(
    dead_code,
    reason = "this target tests comparator helpers; paired measurements exercise command execution"
)]
mod driver;

use driver::{bounds, counter};

#[test]
fn counter_requires_one_positive_fully_counted_event() {
    let complete = "instructions:u:42 9 9\n";
    assert_eq!(
        counter("# header\n\n42,,instructions:u,9,100.00,,\n", complete),
        Ok(42)
    );
    assert_eq!(
        counter(
            "42,,instructions:u,20000,100.00,,",
            "instructions:u:42 20000 20000",
        ),
        Ok(42)
    );
    for (text, metadata) in [
        ("", complete),
        ("0,,instructions:u,9,100.00,,", "instructions:u:0 9 9"),
        ("<not counted>,,instructions:u,0,0.00,,", complete),
        ("42,,instructions:u,9,99.99,,", complete),
        ("42,,cycles:u,9,100.00,,", complete),
        (
            "42,,instructions:u,9,100.00,,\n42,,instructions:u,9,100.00,,",
            complete,
        ),
        ("42,,instructions:u,9,100.00,,", ""),
        ("42,,instructions:u,9,100.00,,", "instructions:u:43 9 9"),
        (
            "42,,instructions:u,20000,100.00,,",
            "instructions:u:42 20001 20000",
        ),
    ] {
        assert!(counter(text, metadata).is_err(), "{text}\n{metadata}");
    }
}

#[test]
fn bounds_enforce_original_ratios_and_growth_without_rounding() {
    let baseline = [100, 200, 400, 800];
    assert_eq!(
        bounds(&baseline, &[150, 300, 600, 1200], 215),
        Vec::<String>::new()
    );
    assert_eq!(
        bounds(&baseline, &[150, 300, 600, 1201], 215),
        ["candidate/baseline at 800 exceeds 1.5"]
    );
    assert_eq!(
        bounds(&[1000, 2000, 4000, 8000], &[100, 200, 400, 861], 215),
        ["growth from 400 to 800 exceeds 215/100"]
    );
    assert_eq!(
        bounds(&baseline, &[100, 216, 432, 864], 215),
        Vec::<String>::new()
    );
}
