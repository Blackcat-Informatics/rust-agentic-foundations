// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The body model over every tracked Markdown file of the repository.

use std::path::{Path, PathBuf};
use std::process::Command;

use raf_okf::Body;

fn gate_root() -> PathBuf {
    std::env::var_os("RAF_GATE_ROOT").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        PathBuf::from,
    )
}

fn tracked_markdown() -> Vec<String> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(["ls-files", "-z", "*.md"])
        .output()
        .expect("git ls-files must run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
        .stdout
        .split(|b| *b == 0)
        .filter(|name| !name.is_empty())
        .map(|name| String::from_utf8(name.to_vec()).expect("tracked paths are UTF-8"))
        .collect()
}

#[test]
fn no_tracked_markdown_file_holds_a_split_link_or_a_malformed_block() {
    let paths = tracked_markdown();
    assert!(
        paths.len() > 30,
        "found only {} Markdown files",
        paths.len()
    );
    let root = gate_root();
    for path in &paths {
        let text =
            std::fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"));
        let body = Body::parse(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
        let split: Vec<usize> = body
            .split_links()
            .iter()
            .map(raf_okf::SplitLink::line)
            .collect();
        assert!(split.is_empty(), "{path}: split links at lines {split:?}");
    }
}
