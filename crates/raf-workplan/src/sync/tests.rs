// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Filesystem outcomes at the publication boundary after all texts are staged.

use super::publish;
use crate::publication_io::{Rollback, Root, Staged};
use raf_source::RelPath;
use std::fs;
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

struct Scratch(purrdf_testkit::TempDir);

impl Scratch {
    fn new() -> Self {
        let binary = std::env::current_exe().expect("the test binary");
        let target = binary.parent().expect("the test target directory");
        Self(purrdf_testkit::TempDir::new_in(target).expect("create publication fixture"))
    }
}

fn staged(root: &Path, guarded: bool) -> (Vec<Staged>, Rollback) {
    fs::create_dir(root).expect("create target directory");
    let authority = Root::open(root).expect("open native publication root");
    let targets: Vec<_> = ["one.md", "two.md", "three.md"]
        .into_iter()
        .map(|name| {
            let target = root.join(name);
            fs::write(&target, format!("old {name}\n")).expect("write original");
            if name == "one.md" {
                fs::hard_link(&target, root.join("one-alias.md")).expect("create original alias");
            }
            authority.capture(&RelPath::new(name).unwrap()).unwrap()
        })
        .collect();
    let mut grant = authority
        .reserve_staging(targets.len() + usize::from(guarded), targets.iter())
        .unwrap();
    let mut rollback = Rollback::default();
    let files = targets
        .into_iter()
        .zip(["one.md", "two.md", "three.md"])
        .map(|(target, name)| {
            rollback
                .stage(
                    target,
                    &format!("new {name}\n"),
                    (guarded && name == "one.md").then_some("old one.md\n"),
                    &mut grant,
                )
                .expect("stage text")
        })
        .collect();
    (files, rollback)
}

fn staged_original(root: &Path) -> std::path::PathBuf {
    fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with(".sync.tmp")
                && fs::read_to_string(path).unwrap() == "old one.md\n"
        })
        .expect("the original is staged before publication")
}

#[test]
fn a_late_rename_failure_preserves_published_files_and_discards_the_tail() {
    let scratch = Scratch::new();
    for (case, guarded, expected) in [
        ("refused", false, "new one.md\n"),
        ("guarded", true, "old one.md\n"),
    ] {
        let root = scratch.0.path().join(case);
        let (files, mut rollback) = staged(&root, guarded);
        let backup = guarded.then(|| staged_original(&root));
        let backup_inode = backup
            .as_ref()
            .map(|path| fs::metadata(path).unwrap().ino());
        // An ordinary sync keeps published files. Completion restores its
        // promoted capability when a later document publication refuses.
        let second = root.join(files[1].target_path().as_str());
        fs::remove_file(&second).expect("remove second target");
        fs::create_dir(&second).expect("replace it with a directory");
        let sentinel = second.join("sentinel");
        fs::write(&sentinel, "untouched\n").expect("write sentinel");

        publish(&files, &mut rollback).expect_err("the second rename must fail");
        rollback.restore().expect("finish the guarded restoration");
        rollback.discard().expect("clean the staged originals");
        if let Some(inode) = backup_inode {
            assert_eq!(fs::metadata(root.join("one.md")).unwrap().ino(), inode);
        }
        assert!(backup.is_none_or(|path| !path.exists()));
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
    let (valid, mut rollback) = staged(&root, true);
    publish(&valid, &mut rollback).expect("regular targets accept every staged replacement");
    rollback.discard().expect("discard the staged original");
    assert!(fs::read_dir(&root).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".sync.tmp")
    }));
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
    let (files, mut rollback) = staged(&root, false);
    let temporary = files[0].temporary_path();
    fs::remove_file(temporary).unwrap();
    fs::write(temporary, "foreign\n").unwrap();
    let error = publish(&files, &mut rollback)
        .expect_err("a replaced temporary is never published or removed");
    rollback
        .restore()
        .expect("no guarded original was published");
    rollback.discard().expect("no guarded original was staged");
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
