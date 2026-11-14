// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Public load and actual report equivalence over the existing source fixtures.

#[path = "../../examples/support/workspace_report.rs"]
mod workspace_report;

use raf_source::{LoadError, LoadOptions, MemoryTree, ParallelOptions, Workspace};
use std::num::NonZeroUsize;

pub(crate) fn compare(tree: &MemoryTree, serial: &Result<Workspace, LoadError>) {
    let report = serial.as_ref().ok().map(workspace_report::report);
    for workers in [1, 2, 2] {
        let parallel = ParallelOptions::new(
            NonZeroUsize::new(workers).expect("nonzero"),
            16 * 1024 * 1024,
        )
        .expect("fixed scheduler fits");
        let parallel = Workspace::par_load(tree, &["."], &LoadOptions::default(), &parallel);
        match (serial, parallel) {
            (Ok(serial), Ok(parallel)) => {
                assert_eq!(*serial, parallel, "worker count {workers}");
                assert_eq!(serial.usage(), parallel.usage(), "worker count {workers}");
                assert_eq!(
                    report.as_ref().expect("serial report").as_bytes(),
                    workspace_report::report(&parallel).as_bytes()
                );
            }
            (Err(serial), Err(parallel)) => {
                assert_eq!(
                    serial.to_string(),
                    parallel.to_string(),
                    "worker count {workers}"
                );
            }
            (serial, parallel) => panic!(
                "serial and {workers}-worker outcomes differ: {serial:?} versus {parallel:?}"
            ),
        }
    }
}
