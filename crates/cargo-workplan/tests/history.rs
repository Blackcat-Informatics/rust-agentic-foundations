// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `history` through the built binary and the real `git`: the document and
//! bound-symbol commits of a capability, newest first, in a table and in RDF 1.2
//! with the same entries; a binding that no longer resolves; `--since`; `-q`;
//! the exit 2 of an id no document has and of a shallow clone; and that the
//! tree is left as it was.

mod support;
use support::rdf::Rdf;

use support::history::{ALPHA, commit_on, dated};
use support::{Repo, Scratch, git};

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
fn rdf_preserves_full_history_commits_sources_paths_and_removed_bindings() {
    let repo = dated("history-rdf");
    let text = repo.both(&["history", "cap:alpha"]).expect(0);
    for format in ["turtle", "ntriples"] {
        let run = repo
            .both(&["--format", format, "history", "cap:alpha"])
            .expect(0);
        let rdf = Rdf::of(&run);
        let entries = rdf.rows("HistoryEntry");
        let rows: Vec<&str> = text
            .stdout
            .lines()
            .skip(2)
            .take_while(|line| !line.is_empty())
            .collect();
        assert_eq!(entries.len(), rows.len());
        for (ordinal, (entry, row)) in entries.iter().zip(&rows).enumerate() {
            assert_eq!(entry.number("ordinal"), ordinal);
            let commit = entry.required("commit");
            assert_eq!(commit.len(), 40);
            assert_eq!(
                entry.required("date"),
                repo.git(&["show", "-s", "--format=%aI", commit])
            );
            for property in ["source", "path"] {
                for (ordinal, child) in entry.children(property).iter().enumerate() {
                    assert_eq!(child.number("ordinal"), ordinal);
                }
            }
            assert!(row.starts_with(&format!("| {} | ", &commit[..7])));
            assert!(row.contains(&format!("| {} |", entry.required("subject"))));
        }
        let source = entries[0].children("source");
        assert_eq!(source.len(), 1);
        assert_eq!(source[0].required("kind"), "symbol");
        assert_eq!(source[0].required("binding"), "demo::real");
        assert_eq!(
            entries[0].children("path")[0].required("path"),
            "src/lib.rs"
        );
        assert_eq!(entries[1].children("path")[0].required("path"), ALPHA);
        let removed = rdf.rows("RemovedBinding");
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].required("binding"), "demo::gone");
        assert_eq!(removed[0].required("removal"), "found");
        assert_eq!(
            removed[0].required("commit"),
            commit_of(&repo, "remove gone").as_str()
        );
        assert_eq!(removed[0].required("identifier"), "gone");
        assert!(!removed[0].children("directory").is_empty());
        let removed_commit = commit_of(&repo, "remove gone");
        assert_eq!(
            removed[0].required("lastResolved"),
            repo.git(&["rev-parse", &format!("{removed_commit}^")])
        );
    }
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
        .both(&["--format", "turtle", "history", "cap:nowhere"])
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
        run.stderr.contains(
            "`git log -L` could not read the lines of the bound symbol `demo::later::later` in `src/later.rs` from `HEAD`"
        ) && run.stderr.contains("one way to read them is to commit `src/later.rs`"),
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
        .both(&["-q", "--format", "turtle", "history", "cap:alpha"])
        .expect(0);
    assert_eq!(Rdf::of(&json).result_count(), 0);
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
