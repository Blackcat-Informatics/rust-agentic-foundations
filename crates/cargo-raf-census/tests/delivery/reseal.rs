// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `reseal --in-place`, the one path that replaces a ledger a person edited by hand.
//! Each test runs the real command in a scratch workspace under the target directory.

#![expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]

use super::{ledger, run, source_fixture};
use purrdf_testkit::TempDir;
use std::{fs, path::Path};

const APPROVED: &str = "The approved constant";
const EDITED: &str = "The edited constant";

/// A scratch workspace holding the form-6 ledger the delivery fixture writes.
fn workspace() -> TempDir {
    let scratch = TempDir::new_in(Path::new(env!("CARGO_TARGET_TMPDIR"))).unwrap();
    source_fixture(scratch.path());
    fs::write(scratch.path().join(raf_census::LEDGER_FILE), ledger(1)).unwrap();
    scratch
}

/// The ledger text with exactly one occurrence of `from` replaced by `to`, so a test
/// never passes on an edit that did not happen.
fn edit(text: &str, from: &str, to: &str) -> String {
    assert_eq!(
        text.matches(from).count(),
        1,
        "fixture must contain {from} once"
    );
    text.replace(from, to)
}

#[test]
fn in_place_replaces_an_edited_ledger_with_one_the_ledger_reader_accepts() {
    let scratch = workspace();
    let root = scratch.path();
    let ledger_path = root.join(raf_census::LEDGER_FILE);
    let original = fs::read_to_string(&ledger_path).unwrap();
    fs::write(&ledger_path, edit(&original, APPROVED, EDITED)).unwrap();

    let sealed = run(root, &["reseal", "--in-place"]);
    assert!(sealed.status.success(), "{sealed:?}");
    let stored = fs::read_to_string(&ledger_path).unwrap();
    assert!(stored.contains(EDITED), "the edited row is in the ledger");
    assert!(!stored.contains(APPROVED), "the replaced row is gone");
    assert!(
        !root.join("helpers-ledger.ttl.reseal-tmp").exists(),
        "a successful replace leaves no sibling file"
    );
    let read_back = run(root, &["dump-ledger"]);
    assert!(read_back.status.success(), "{read_back:?}");
}

#[test]
fn in_place_refuses_an_emptied_required_field_and_leaves_the_ledger_byte_identical() {
    let scratch = workspace();
    let root = scratch.path();
    let ledger_path = root.join(raf_census::LEDGER_FILE);
    let original = fs::read(&ledger_path).unwrap();
    let text = String::from_utf8(original).unwrap();
    fs::write(&ledger_path, edit(&text, "\"number\"", "\"\"")).unwrap();
    let edited = fs::read(&ledger_path).unwrap();

    let refused = run(root, &["reseal", "--in-place"]);
    assert_eq!(refused.status.code(), Some(1), "{refused:?}");
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains("jobId") && stderr.contains("on row #job"),
        "the refusal names the field and its row: {stderr}"
    );
    assert_eq!(
        fs::read(&ledger_path).unwrap(),
        edited,
        "a refused edit leaves the ledger as the person left it"
    );
    assert!(!root.join("helpers-ledger.ttl.reseal-tmp").exists());
}

#[test]
fn in_place_refuses_a_leftover_temporary_file_and_names_its_remedy() {
    let scratch = workspace();
    let root = scratch.path();
    let ledger_path = root.join(raf_census::LEDGER_FILE);
    let original = fs::read_to_string(&ledger_path).unwrap();
    let edited = edit(&original, APPROVED, EDITED);
    fs::write(&ledger_path, &edited).unwrap();
    let leftover = root.join("helpers-ledger.ttl.reseal-tmp");
    fs::write(&leftover, "a stale partial write").unwrap();

    let refused = run(root, &["reseal", "--in-place"]);
    assert_eq!(refused.status.code(), Some(1), "{refused:?}");
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains("remove this file"),
        "the error says the leftover file may be removed: {stderr}"
    );
    assert_eq!(fs::read_to_string(&ledger_path).unwrap(), edited);
    assert_eq!(
        fs::read_to_string(&leftover).unwrap(),
        "a stale partial write",
        "the leftover file is not removed by the command"
    );
}

#[cfg(unix)]
#[test]
fn in_place_keeps_the_ledger_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let scratch = workspace();
    let root = scratch.path();
    let ledger_path = root.join(raf_census::LEDGER_FILE);
    let original = fs::read_to_string(&ledger_path).unwrap();
    fs::write(&ledger_path, edit(&original, APPROVED, EDITED)).unwrap();
    fs::set_permissions(&ledger_path, fs::Permissions::from_mode(0o600)).unwrap();

    let sealed = run(root, &["reseal", "--in-place"]);
    assert!(sealed.status.success(), "{sealed:?}");
    let mode = fs::metadata(&ledger_path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "a private ledger stays private");
}

#[cfg(unix)]
#[test]
fn in_place_refuses_a_symbolic_link_and_leaves_its_target_unchanged() {
    let scratch = workspace();
    let root = scratch.path();
    let ledger_path = root.join(raf_census::LEDGER_FILE);
    let original = fs::read_to_string(&ledger_path).unwrap();
    let target = root.join("elsewhere.ttl");
    fs::write(&target, edit(&original, APPROVED, EDITED)).unwrap();
    let target_bytes = fs::read(&target).unwrap();
    fs::remove_file(&ledger_path).unwrap();
    std::os::unix::fs::symlink(&target, &ledger_path).unwrap();

    let refused = run(root, &["reseal", "--in-place"]);
    assert!(!refused.status.success(), "{refused:?}");
    assert!(
        fs::symlink_metadata(&ledger_path)
            .unwrap()
            .file_type()
            .is_symlink(),
        "the link is kept rather than replaced by a file"
    );
    assert_eq!(fs::read(&target).unwrap(), target_bytes);
}

#[test]
fn out_refuses_an_n_triples_destination_and_writes_a_turtle_one() {
    // INVARIANT: reseal writes Turtle only, so an N-Triples destination is refused before
    // any file is written; a Turtle destination beside it succeeds.
    let scratch = workspace();
    let root = scratch.path();
    let refused = run(root, &["reseal", "--out", "sealed.nt"]);
    assert_eq!(refused.status.code(), Some(1), "{refused:?}");
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains(".ttl"),
        "the refusal names the extension: {stderr}"
    );
    assert!(
        !root.join("sealed.nt").exists(),
        "a refused destination is not written"
    );
    let accepted = run(root, &["reseal", "--out", "sealed.ttl"]);
    assert!(accepted.status.success(), "{accepted:?}");
    assert!(root.join("sealed.ttl").exists());
}
