// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Filesystem outcomes at the publication boundary after all texts are staged.

use super::publish;
use crate::publication_io::{Root, Staged};
use raf_source::RelPath;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

struct Scratch(purrdf_testkit::TempDir);

impl Scratch {
    fn new() -> Self {
        let binary = std::env::current_exe().expect("the test binary");
        let target = binary.parent().expect("the test target directory");
        Self(purrdf_testkit::TempDir::new_in(target).expect("create publication fixture"))
    }
}

fn staged(root: &Path) -> Vec<Staged> {
    fs::create_dir(root).expect("create target directory");
    let authority = Root::open(root).expect("open native publication root");
    ["one.md", "two.md", "three.md"]
        .into_iter()
        .map(|name| {
            let target = root.join(name);
            fs::write(&target, format!("old {name}\n")).expect("write original");
            if name == "one.md" {
                fs::hard_link(&target, root.join("one-alias.md")).expect("create original alias");
            }
            authority
                .capture(&RelPath::new(name).unwrap())
                .unwrap()
                .stage(&format!("new {name}\n"))
                .expect("stage text")
        })
        .collect()
}

#[test]
fn a_late_rename_failure_preserves_published_files_and_discards_the_tail() {
    let scratch = Scratch::new();
    let empty = BTreeMap::new();
    let guarded = BTreeMap::from([(RelPath::new("one.md").unwrap(), "old one.md\n".to_owned())]);
    for (case, originals, expected) in [
        ("refused", &empty, "new one.md\n"),
        ("guarded", &guarded, "old one.md\n"),
    ] {
        let root = scratch.0.path().join(case);
        let files = staged(&root);
        // An ordinary sync keeps published files. Completion restores its
        // promoted capability when a later document publication refuses.
        let second = root.join(files[1].target_path().as_str());
        fs::remove_file(&second).expect("remove second target");
        fs::create_dir(&second).expect("replace it with a directory");
        let sentinel = second.join("sentinel");
        fs::write(&sentinel, "untouched\n").expect("write sentinel");

        publish(&files, originals).expect_err("the second rename must fail");
        assert_eq!(
            fs::read_to_string(root.join(files[0].target_path().as_str())).unwrap(),
            expected
        );
        assert_eq!(
            fs::read_to_string(root.join("one-alias.md")).unwrap(),
            "old one.md\n"
        );
        assert_eq!(fs::read_to_string(&sentinel).unwrap(), "untouched\n");
        assert_eq!(
            fs::read_to_string(root.join(files[2].target_path().as_str())).unwrap(),
            "old three.md\n"
        );
        for file in &files {
            let temporary = file.temporary_path();
            assert!(
                !temporary.exists(),
                "unpublished temporary file {temporary:?}"
            );
        }
    }

    let root = scratch.0.path().join("accepted");
    let valid = staged(&root);
    publish(&valid, &empty).expect("regular targets accept every staged replacement");
    assert_eq!(
        fs::read_to_string(root.join("one-alias.md")).unwrap(),
        "old one.md\n"
    );
    for (name, file) in ["one.md", "two.md", "three.md"].into_iter().zip(valid) {
        assert_eq!(
            fs::read_to_string(root.join(file.target_path().as_str())).unwrap(),
            format!("new {name}\n")
        );
        assert!(!file.temporary_path().exists());
    }

    let root = scratch.0.path().join("foreign-temporary");
    let files = staged(&root);
    let temporary = files[0].temporary_path();
    fs::remove_file(temporary).unwrap();
    fs::write(temporary, "foreign\n").unwrap();
    let error =
        publish(&files, &empty).expect_err("a replaced temporary is never published or removed");
    assert!(
        error.to_string().contains("metadata cleanup also failed"),
        "{error}"
    );
    assert_eq!(fs::read_to_string(temporary).unwrap(), "foreign\n");
    for file in &files {
        assert_eq!(
            fs::read_to_string(root.join(file.target_path().as_str())).unwrap(),
            format!("old {}\n", file.target_path())
        );
    }
    assert!(
        files[1..]
            .iter()
            .all(|file| !file.temporary_path().exists())
    );
}
