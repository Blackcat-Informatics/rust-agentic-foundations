// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Exercises the closed RDF 1.2 report readers and their canonical spelling.
#![no_main]

use libfuzzer_sys::fuzz_target;
use raf_census::{Baseline, Index};
use std::sync::Once;

static SELF_CHECK: Once = Once::new();
fuzz_target!(|data: &[u8]| {
    SELF_CHECK.call_once(|| {
        let baseline = include_str!("../../crates/raf-census/tests/fixtures/reports/baseline.ttl");
        let index = include_str!("../../crates/raf-census/tests/fixtures/reports/index.ttl");
        assert_eq!(
            Baseline::from_turtle(baseline)
                .expect("valid baseline reaches shape checking")
                .to_turtle()
                .expect("renders"),
            baseline
        );
        assert_eq!(
            Index::from_turtle(index)
                .expect("valid index reaches shape checking")
                .to_turtle()
                .expect("renders"),
            index
        );
    });
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    if let Ok(report) = Baseline::from_turtle(text) {
        assert_eq!(report.to_turtle().expect("accepted baseline renders"), text);
    }
    if let Ok(report) = Index::from_turtle(text) {
        assert_eq!(report.to_turtle().expect("accepted index renders"), text);
    }
});
