// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Filesystem outcomes at the publication boundary after all texts are staged.

use super::{fs, publish, stage};
use std::path::{Path, PathBuf};

struct Scratch(purrdf_testkit::TempDir);

impl Scratch {
    fn new() -> Self {
        Self(
            purrdf_testkit::TempDir::new_in(std::env::temp_dir())
                .expect("create publication fixture"),
        )
    }
}

fn staged(root: &Path) -> Vec<(PathBuf, PathBuf)> {
    fs::create_dir(root).expect("create target directory");
    ["one.md", "two.md", "three.md"]
        .into_iter()
        .map(|name| {
            let target = root.join(name);
            fs::write(&target, format!("old {name}\n")).expect("write original");
            let temporary = stage(&target, &format!("new {name}\n")).expect("stage text");
            (temporary, target)
        })
        .collect()
}

#[test]
fn a_late_rename_failure_preserves_published_files_and_discards_the_tail() {
    let scratch = Scratch::new();
    let files = staged(&scratch.0.path().join("refused"));
    // Model a target replacement after preflight and staging. A nonempty
    // directory cannot be replaced by the staged regular file on any target.
    fs::remove_file(&files[1].1).expect("remove second target");
    fs::create_dir(&files[1].1).expect("replace it with a directory");
    let sentinel = files[1].1.join("sentinel");
    fs::write(&sentinel, "untouched\n").expect("write sentinel");

    publish(&files).expect_err("the second rename must fail");
    assert_eq!(fs::read_to_string(&files[0].1).unwrap(), "new one.md\n");
    assert_eq!(fs::read_to_string(&sentinel).unwrap(), "untouched\n");
    assert_eq!(fs::read_to_string(&files[2].1).unwrap(), "old three.md\n");
    for (temporary, _) in &files {
        assert!(
            !temporary.exists(),
            "unpublished temporary file {temporary:?}"
        );
    }

    let valid = staged(&scratch.0.path().join("accepted"));
    publish(&valid).expect("regular targets accept every staged replacement");
    for (name, (temporary, target)) in ["one.md", "two.md", "three.md"].into_iter().zip(valid) {
        assert_eq!(fs::read_to_string(target).unwrap(), format!("new {name}\n"));
        assert!(!temporary.exists());
    }
}
