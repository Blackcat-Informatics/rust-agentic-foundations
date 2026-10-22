// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `docs sync` through the built binary: the capability index is created only
//! when the SPDX text is given, the generated blocks are rewritten and nothing
//! else is, a second run writes nothing, and a refusal writes nothing.

mod support;

use support::{COPYRIGHT, LICENCE, Repo};

const INDEX: &str = "docs/capabilities/index.md";
const MATRIX: &str = "<!-- raf:generated:verification-matrix -->\n<!-- raf:generated:end -->\n";

/// Leave only the catalogue missing, with the capability's blocks current.
fn without_index(tag: &str) -> Repo {
    let repo = Repo::synced(tag, false);
    std::fs::remove_file(repo.root().join(INDEX)).expect("remove the generated index");
    repo
}

/// Make the real, already populated verification matrix stale.
fn empty_matrix(repo: &Repo) {
    let path = "docs/capabilities/alpha.md";
    let text = repo.read(path);
    let (before, tail) = text
        .split_once("<!-- raf:generated:verification-matrix -->\n")
        .expect("the fixture has a verification matrix");
    let (_, after) = tail
        .split_once("<!-- raf:generated:end -->\n")
        .expect("the matrix has an end marker");
    repo.put(path, &format!("{before}{MATRIX}{after}"));
}

/// The paths whose bytes differ between two snapshots.
fn changed(
    before: &std::collections::BTreeMap<String, Vec<u8>>,
    after: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Vec<String> {
    let mut paths: Vec<String> = after
        .iter()
        .filter(|(path, bytes)| before.get(*path) != Some(*bytes))
        .map(|(path, _)| path.clone())
        .collect();
    paths.extend(
        before
            .keys()
            .filter(|path| !after.contains_key(*path))
            .cloned(),
    );
    paths.sort();
    paths
}

#[test]
fn a_missing_index_without_the_text_is_exit_two_naming_the_field_and_writes_nothing() {
    let repo = Repo::roadmap("sync-missing", false);
    let before = repo.snapshot();
    let run = repo.both(&["docs", "sync"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("--copyright"), "{run:#?}");
    assert!(run.stderr.contains(INDEX), "{run:#?}");
    assert_eq!(repo.snapshot(), before, "a refused sync wrote");
}

#[test]
fn each_field_that_is_not_given_is_named() {
    let repo = Repo::roadmap("sync-one-field", false);
    let only_copyright = repo
        .both(&["docs", "sync", "--copyright", COPYRIGHT])
        .expect(2);
    assert!(
        only_copyright.stderr.contains("--licence"),
        "{only_copyright:#?}"
    );
    assert!(
        !only_copyright.stderr.contains("--copyright is not"),
        "{only_copyright:#?}"
    );
    let only_licence = repo.both(&["docs", "sync", "--licence", LICENCE]).expect(2);
    assert!(
        only_licence.stderr.contains("--copyright"),
        "{only_licence:#?}"
    );
    assert!(!repo.root().join(INDEX).exists());
}

#[test]
fn a_missing_index_with_the_text_is_created_with_that_header() {
    let repo = without_index("sync-create");
    let run = repo
        .both_writing(&[
            "docs",
            "sync",
            "--copyright",
            COPYRIGHT,
            "--licence",
            LICENCE,
        ])
        .expect(0);
    assert_eq!(run.lines(), [format!("rewrote {INDEX}")], "{run:#?}");
    assert_eq!(run.stderr, "");
    let index = repo.read(INDEX);
    let mut lines = index.lines();
    assert_eq!(
        lines.next(),
        Some(
            format!(
                "<!-- {}{COPYRIGHT} -->",
                concat!("SPDX-", "FileCopyrightText: ")
            )
            .as_str()
        )
    );
    assert_eq!(
        lines.next(),
        Some(
            format!(
                "<!-- {}{LICENCE} -->",
                concat!("SPDX-", "License-Identifier: ")
            )
            .as_str()
        )
    );
    // The repository is now in sync, by the binary's own check.
    repo.both(&["check"]).expect(0);
}

#[test]
fn a_second_sync_writes_nothing_and_prints_nothing() {
    let repo = Repo::roadmap("sync-idempotent", false);
    let args = [
        "docs",
        "sync",
        "--copyright",
        COPYRIGHT,
        "--licence",
        LICENCE,
    ];
    repo.run(&args).expect(0);
    let after_first = repo.snapshot();
    let second = repo.both(&args).expect(0);
    assert_eq!((second.stdout.as_str(), second.stderr.as_str()), ("", ""));
    assert_eq!(repo.snapshot(), after_first);
}

#[test]
fn the_text_is_not_read_when_the_index_exists() {
    let repo = Repo::synced("sync-existing-index", false);
    let before = repo.snapshot();
    // Neither value could be written into a header, and neither is read.
    let run = repo
        .both(&["docs", "sync", "--copyright", "", "--licence", "a\nb"])
        .expect(0);
    assert_eq!((run.stdout.as_str(), run.stderr.as_str()), ("", ""));
    assert_eq!(repo.snapshot(), before);
}

#[test]
fn a_value_that_cannot_be_written_into_the_header_is_refused_when_the_index_is_created() {
    let repo = Repo::roadmap("sync-invalid", false);
    let before = repo.snapshot();
    for (copyright, licence, field) in [
        ("ends --> the comment", LICENCE, "copyright"),
        (COPYRIGHT, "two\nlines", "licence"),
        ("", LICENCE, "copyright"),
    ] {
        let run = repo
            .both(&[
                "docs",
                "sync",
                "--copyright",
                copyright,
                "--licence",
                licence,
            ])
            .expect(2);
        assert!(run.stderr.contains(field), "{run:#?}");
        assert_eq!(repo.snapshot(), before);
    }
    // The neighbour: a value that fits is written.
    repo.both_writing(&[
        "docs",
        "sync",
        "--copyright",
        COPYRIGHT,
        "--licence",
        LICENCE,
    ])
    .expect(0);
}

#[test]
fn only_the_generated_blocks_are_rewritten() {
    let repo = Repo::synced("sync-blocks", false);
    empty_matrix(&repo);
    let before = repo.snapshot();
    // The block is empty, so the repository is stale until the sync.
    let stale = repo.both(&["check"]).expect(1);
    assert!(stale.stdout.contains("[generated-stale]"), "{stale:#?}");
    let run = repo.both_writing(&["docs", "sync"]).expect(0);
    assert_eq!(
        run.lines(),
        ["rewrote docs/capabilities/alpha.md"],
        "{run:#?}"
    );
    let after = repo.snapshot();
    assert_eq!(changed(&before, &after), ["docs/capabilities/alpha.md"]);
    let alpha = repo.read("docs/capabilities/alpha.md");
    assert!(
        alpha.contains("demo::test_crate::it::case"),
        "the matrix lists the test: {alpha}"
    );
    // Every byte outside the matrix is as it was.
    let before_text =
        String::from_utf8(before["docs/capabilities/alpha.md"].clone()).expect("UTF-8");
    let (head, tail) = before_text.split_once(MATRIX).expect("the empty matrix");
    assert!(alpha.starts_with(head));
    assert!(alpha.ends_with(tail));
    repo.both(&["check"]).expect(0);
}

#[test]
fn quiet_hides_the_files_rewritten_and_keeps_the_exit_code() {
    let repo = Repo::roadmap("sync-quiet", false);
    let run = repo
        .both_writing(&[
            "-q",
            "docs",
            "sync",
            "--copyright",
            COPYRIGHT,
            "--licence",
            LICENCE,
        ])
        .expect(0);
    assert_eq!((run.stdout.as_str(), run.stderr.as_str()), ("", ""));
    assert!(repo.root().join(INDEX).is_file(), "-q still writes");
}

#[test]
fn a_refused_file_is_exit_two_and_nothing_is_written() {
    let repo = Repo::synced("sync-refused", false);
    // Text outside the generated blocks is not a generated index.
    let index = repo.read(INDEX);
    repo.put(INDEX, &format!("{index}\nA hand-written paragraph.\n"));
    empty_matrix(&repo);
    let before = repo.snapshot();
    let run = repo.both(&["docs", "sync"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("index-hand-written"), "{run:#?}");
    assert_eq!(repo.snapshot(), before, "a refused sync wrote");
    // The neighbour: the index put right, the same sync writes.
    repo.put(INDEX, &index);
    let run = repo.both_writing(&["docs", "sync"]).expect(0);
    assert_eq!(run.lines(), ["rewrote docs/capabilities/alpha.md"]);
}

#[test]
fn a_document_that_does_not_load_is_exit_two_and_nothing_is_written() {
    let repo = Repo::synced("sync-unloaded", false);
    repo.edit(
        "docs/capabilities/gamma.md",
        "state: designed",
        "state: sleeping",
    );
    let before = repo.snapshot();
    let run = repo.both(&["docs", "sync"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("cap:gamma"), "{run:#?}");
    assert_eq!(repo.snapshot(), before);
}

#[test]
fn the_index_is_created_by_root_from_another_directory() {
    let repo = without_index("sync-rooted");
    let root = repo.root().to_string_lossy().into_owned();
    let args = [
        "--root",
        root.as_str(),
        "docs",
        "sync",
        "--copyright",
        COPYRIGHT,
        "--licence",
        LICENCE,
    ];
    let run = support::run_in(&support::elsewhere(), support::Shape::Direct, &args).expect(0);
    assert_eq!(run.lines(), [format!("rewrote {INDEX}")]);
    assert!(repo.root().join(INDEX).is_file());
}
