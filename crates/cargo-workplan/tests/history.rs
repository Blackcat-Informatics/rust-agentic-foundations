// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `history` through the built binary and the real `git`: the document and
//! bound-symbol commits of a capability, newest first, in a table and in JSON
//! with the same entries; a binding that no longer resolves; `--since`; `-q`;
//! the exit 2 of an id no document has and of a shallow clone; and that the
//! tree is left as it was.

mod support;

use serde_json::Value;
use support::{Repo, Scratch, array, capability, change, git, git_with, member_names, strings};

const ALPHA: &str = "docs/capabilities/alpha.md";

/// Commit everything with an author and a committer date of `day` at noon
/// UTC (`--since` reads the committer date).
fn commit_on(repo: &Repo, message: &str, day: &str) {
    repo.git(&["add", "-A"]);
    let date = format!("{day}T12:00:00+00:00");
    git_with(
        repo.root(),
        &["commit", "-q", "--date", &date, "-m", message],
        &[("GIT_COMMITTER_DATE", &date)],
    );
}

/// A repository of four dated commits:
///
/// * `base` (2030-01-01) adds `alpha`, bound to `demo::real` and
///   `demo::gone`, and the change `grow`;
/// * `edit alpha` (2030-01-02) edits the document;
/// * `change real` (2030-01-03) edits the body of `demo::real`;
/// * `remove gone` (2030-01-04) removes `demo::gone`;
/// * `unrelated` (2030-01-05) edits the test file, which no source names.
fn dated(tag: &str) -> Repo {
    let repo = Repo::bare(tag);
    repo.put(
        "src/lib.rs",
        "/// A documented function.\npub fn real() {}\n\n/// Soon gone.\npub fn gone() {}\n",
    );
    repo.put(
        ALPHA,
        &capability(
            "alpha",
            "implementing",
            [
                "[demo::real, demo::gone]",
                "[demo::test_crate::it::case]",
                "[]",
            ],
            "[]",
            &[],
        ),
    );
    repo.put(
        "work/changes/grow.md",
        &change("grow", "[cap:alpha]", &["cap:alpha#contract"]),
    );
    repo.sync();
    repo.git(&["init", "-q", "-b", "main"]);
    commit_on(&repo, "base", "2030-01-01");
    repo.edit(ALPHA, "Text for Purpose of alpha.", "Alpha has a purpose.");
    commit_on(&repo, "edit alpha", "2030-01-02");
    repo.edit(
        "src/lib.rs",
        "pub fn real() {}",
        "pub fn real() {\n    let _ = 1;\n}",
    );
    commit_on(&repo, "change real", "2030-01-03");
    repo.edit("src/lib.rs", "\n/// Soon gone.\npub fn gone() {}\n", "");
    commit_on(&repo, "remove gone", "2030-01-04");
    repo.edit(
        "tests/it.rs",
        "fn case() {}",
        "fn case() {\n    let _ = 2;\n}",
    );
    commit_on(&repo, "unrelated", "2030-01-05");
    repo
}

fn commit_of(repo: &Repo, subject: &str) -> String {
    repo.git(&[
        "log",
        "--format=%H",
        "--fixed-strings",
        "--grep",
        subject,
        "-n",
        "1",
    ])
}

#[test]
fn a_capability_lists_its_document_and_symbol_commits_newest_first_and_a_removed_binding() {
    let repo = dated("history-table");
    let before = repo.worktree();
    let run = repo.both(&["history", "cap:alpha"]).expect(0);
    assert_eq!(run.stderr, "");
    let short = |subject: &str| commit_of(&repo, subject)[..7].to_string();
    let removed = commit_of(&repo, "remove gone");
    let parent = repo.git(&["rev-parse", &format!("{removed}^")]);
    assert_eq!(
        run.stdout,
        format!(
            "| Commit | Date | Subject | Source |\n| --- | --- | --- | --- |\n\
             | {} | 2030-01-03 | change real | `demo::real` |\n\
             | {} | 2030-01-02 | edit alpha | document |\n\
             | {} | 2030-01-01 | base | document, `demo::real` |\n\
             \n| Binding | Removed in | Date | Subject | Last resolved |\n| --- | --- | --- | --- | --- |\n\
             | `demo::gone` | {} | 2030-01-04 | remove gone | {} |\n",
            short("change real"),
            short("edit alpha"),
            short("base"),
            &removed[..7],
            &parent[..7],
        )
    );
    // The history reads Git and writes nothing.
    assert_eq!(repo.worktree(), before);
}

#[test]
fn json_holds_the_entries_of_the_table_in_the_same_order() {
    let repo = dated("history-json");
    let text = repo.both(&["history", "cap:alpha"]).expect(0);
    let run = repo
        .both(&["--format", "json", "history", "cap:alpha"])
        .expect(0);
    let objects = array(&run);
    let entries: Vec<&Value> = objects
        .iter()
        .filter(|o| o["kind"] == "history-entry")
        .collect();
    let rows: Vec<&str> = text
        .stdout
        .lines()
        .skip(2)
        .take_while(|line| !line.is_empty())
        .collect();
    assert_eq!(entries.len(), rows.len());
    for (entry, row) in entries.iter().zip(&rows) {
        let commit = entry["commit"].as_str().expect("a commit");
        let subject = entry["subject"].as_str().expect("a subject");
        assert!(row.starts_with(&format!("| {} | ", &commit[..7])), "{row}");
        assert!(row.contains(&format!("| {subject} |")), "{row}");
    }
    assert_eq!(
        entries[0]["sources"],
        serde_json::json!([{"source": "symbol", "binding": "demo::real"}])
    );
    assert_eq!(entries[0]["paths"], serde_json::json!(["src/lib.rs"]));
    assert_eq!(entries[1]["paths"], serde_json::json!([ALPHA]));
    let removed = objects
        .iter()
        .find(|o| o["kind"] == "history-removed")
        .expect("the removed binding");
    assert_eq!(removed["binding"], "demo::gone");
    assert_eq!(removed["lookup"], "found");
    assert_eq!(removed["commit"], commit_of(&repo, "remove gone").as_str());
    // The members of each kind, in order.
    let names = member_names(&run);
    assert_eq!(
        names[0],
        strings(&["kind", "commit", "date", "subject", "sources", "paths"])
    );
    assert_eq!(
        names.last().expect("the removed object"),
        &strings(&[
            "kind",
            "binding",
            "identifier",
            "dirs",
            "lookup",
            "commit",
            "date",
            "subject",
            "last_resolved"
        ])
    );
}

#[test]
fn since_limits_the_commits_and_a_change_or_an_unknown_id_is_answered_or_refused() {
    let repo = dated("history-since");
    let run = repo
        .both(&["history", "cap:alpha", "--since", "2030-01-03"])
        .expect(0);
    let rows: Vec<&str> = run
        .stdout
        .lines()
        .skip(2)
        .take_while(|l| !l.is_empty())
        .collect();
    assert_eq!(rows.len(), 1, "{run:#?}");
    assert!(rows[0].contains("| change real |"), "{run:#?}");
    // A change record has its own document history only.
    let change = repo.both(&["history", "change:grow"]).expect(0);
    assert_eq!(change.lines().len(), 3, "{change:#?}");
    assert!(change.stdout.contains("| base | document |"), "{change:#?}");
    // An id no document has is exit 2, naming the ids of its kind.
    let unknown = repo.both(&["history", "cap:nowhere"]).expect(2);
    assert_eq!(unknown.stdout, "");
    assert!(unknown.stderr.contains("cap:alpha"), "{unknown:#?}");
    let json = repo
        .both(&["--format", "json", "history", "cap:nowhere"])
        .expect(2);
    assert_eq!(json.stdout, "");
}

#[test]
fn a_bound_file_that_is_not_committed_is_exit_two_naming_it_until_it_is() {
    let repo = dated("history-uncommitted");
    repo.put(
        "src/later.rs",
        "/// Written after the last commit.\npub fn later() {}\n",
    );
    repo.edit(
        "src/lib.rs",
        "pub fn real() {",
        "pub mod later;\n\n/// A documented function.\npub fn real() {",
    );
    repo.edit(
        ALPHA,
        "[demo::real, demo::gone]",
        "[demo::real, demo::gone, demo::later::later]",
    );
    let run = repo.both(&["history", "cap:alpha"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr
            .contains("the bound symbol `demo::later::later` in `src/later.rs`")
            && run.stderr.contains("commit `src/later.rs`"),
        "{run:#?}"
    );
    // The neighbour: once the file is committed, it has a history.
    commit_on(&repo, "add later", "2030-01-06");
    let run = repo.both(&["history", "cap:alpha"]).expect(0);
    assert!(
        run.stdout
            .contains("| add later | document, `demo::later::later`, `demo::real` |"),
        "{run:#?}"
    );
}

#[test]
fn quiet_prints_nothing_because_a_history_is_not_a_finding() {
    let repo = dated("history-quiet");
    let run = repo.both(&["-q", "history", "cap:alpha"]).expect(0);
    assert_eq!(run.stdout, "");
    let json = repo
        .both(&["-q", "--format", "json", "history", "cap:alpha"])
        .expect(0);
    assert_eq!(json.stdout, "[]\n");
}

#[test]
fn a_shallow_clone_is_exit_two_and_its_full_clone_answers() {
    let repo = dated("history-shallow");
    let copies = Scratch::new("history-shallow-copies");
    let source = format!("file://{}", repo.root().display());
    git(
        copies.path(),
        &["clone", "-q", "--depth", "1", &source, "shallow"],
    );
    git(copies.path(), &["clone", "-q", &source, "full"]);
    let shallow = copies.path().join("shallow").to_string_lossy().into_owned();
    let run = support::run_both(
        &support::elsewhere(),
        &["--root", &shallow, "history", "cap:alpha"],
    )
    .expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("shallow clone"), "{run:#?}");
    // The neighbour: the full clone answers.
    let full = copies.path().join("full").to_string_lossy().into_owned();
    support::run_both(
        &support::elsewhere(),
        &["--root", &full, "history", "cap:alpha"],
    )
    .expect(0);
}
