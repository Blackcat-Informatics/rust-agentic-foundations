// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Retained output-prefix authority through the immutable publisher.

use super::{Root, publish_files, relative};
use std::fs;

#[test]
fn a_replaced_output_binding_refuses_before_owned_payloads() {
    let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let directory = scratch.path().join("output");
    fs::create_dir(&directory).unwrap();
    let root = Root::open(scratch.path()).unwrap();
    let prefix = relative("output").unwrap();
    let admitted = root.directory(&prefix).unwrap();
    fs::rename(&directory, scratch.path().join("original")).unwrap();
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("host.html"), "unrelated").unwrap();
    assert!(admitted.verify().is_err());
    assert!(publish_files(vec![], &root, &prefix).is_err());
    assert!(!directory.join("raf-workplan").exists());
    assert_eq!(
        fs::read_dir(scratch.path().join("original"))
            .unwrap()
            .count(),
        0
    );
    assert_eq!(
        fs::read_to_string(directory.join("host.html")).unwrap(),
        "unrelated"
    );
    let fresh = Root::open(scratch.path()).unwrap();
    let published = publish_files(vec![], &fresh, &prefix).unwrap();
    assert!(published.join("manifest.ttl").is_file());
}
