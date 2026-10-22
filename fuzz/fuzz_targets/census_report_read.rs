// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Exercises the closed RDF 1.2 report readers and their canonical spelling.
#![no_main]

use libfuzzer_sys::fuzz_target;
use raf_census::{Baseline, Error, Index, ReportError};
use std::sync::Once;

static SELF_CHECK: Once = Once::new();
fn exercise(data: &[u8]) {
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
        let baseline = Baseline::from_turtle(baseline).expect("baseline shape");
        let index = Index::from_turtle(index).expect("index shape");
        let baseline_nt = baseline.to_ntriples().expect("baseline N-Triples");
        let index_nt = index.to_ntriples().expect("index N-Triples");
        assert_eq!(
            Baseline::from_ntriples(&baseline_nt).expect("N-Triples shape"),
            baseline
        );
        assert_eq!(
            Index::from_ntriples(&index_nt).expect("N-Triples shape"),
            index
        );
        assert!(matches!(
            Baseline::from_turtle(include_str!("../corpus/census-report-read/unknown.ttl")),
            Err(Error::Report(ReportError::UnknownPredicate(_)))
        ));
        assert!(matches!(
            Index::from_turtle(include_str!("../corpus/census-report-read/copies.ttl")),
            Err(Error::Report(ReportError::Inconsistent(_)))
        ));
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
    if let Ok(report) = Baseline::from_ntriples(text) {
        assert_eq!(
            report.to_ntriples().expect("accepted baseline renders"),
            text
        );
    }
    if let Ok(report) = Index::from_ntriples(text) {
        assert_eq!(report.to_ntriples().expect("accepted index renders"), text);
    }
}

fuzz_target!(|data: &[u8]| exercise(data));
