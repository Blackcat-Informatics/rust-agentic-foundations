// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The history of a document and its bound code: real-git fixture
//! repositories (a document moved, a function changed in one of several
//! commits, a symbol deleted, a file named like an option, a shallow clone),
//! the in-process fake for the exact argv of every question and for each
//! refusal before any process, and stub `git` programs for the version gate,
//! the output bound and the argv a real process receives.

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use raf_source::{MemoryTree, Workspace};
use raf_workplan::Workplan;
use raf_workplan::exec::{
    DEFAULT_OUTPUT_BOUND, Date, FakeRunner, Output, Program, Programs, SystemRunner,
};
use raf_workplan::git::GitError;
use raf_workplan::history::{History, HistoryError, HistoryOptions, Removal, Source, render};

use support::cases::{assessment_record, evidence_record};
use support::checkout::{git, git_command, serial};
use support::disk::{Scratch, on_disk, put, snapshot};
use support::fixtures::change_record;
use support::synced::document;

const VERSION: &str = "git version 2.45.1\n";
const FORMAT: &str = "--format=%x1e%H%x1f%aI%x1f%s";
const A: &str = "0123456789abcdef0123456789abcdef01234567";
const B: &str = "1111111111111111111111111111111111111111";
const C: &str = "0000000000000000000000000000000000000000";
const P: &str = "fedcba9876543210fedcba9876543210fedcba98";

// ----------------------------------------------------------------- fixture

const MANIFEST: &str =
    "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[lib]\npath = \"lib.rs\"\n";

const LIB: &str = "#[path = \"src/alpha.rs\"]\npub mod alpha;\n#[path = \"-rf.rs\"]\npub mod rf;\n#[path = \"--output=x.rs\"]\npub mod out;\n";

const ALPHA_RS: &str = "\
pub fn first() {
    let _ = 1;
}

pub fn second() {
    let _ = 1;
}

pub fn third() {
    let _ = 1;
}

pub fn gone() {}
";

const ARCH: &str = "---\ntype: Architecture\nid: arch:overview\ntitle: Overview\n\
                    description: How it fits.\n---\n\n# Overview\n\nBody text.\n";

const CODE: &str = "[demo::alpha::second, demo::alpha::gone, demo::rf::dashed, demo::out::output]";

fn capability(extra: &str) -> String {
    document(
        "alpha",
        "Alpha.",
        "implementing",
        [CODE, "[]", "[]"],
        "[]",
        &[("Purpose", extra)],
    )
}

/// A repository on a disk with a history of distinct, known author dates.
struct Repo {
    scratch: Scratch,
    root: PathBuf,
    day: usize,
}

impl Repo {
    fn new(tag: &str) -> Self {
        let scratch = Scratch::new(tag);
        let root = scratch.path().join("work");
        fs::create_dir_all(&root).expect("a work directory");
        git(&root, &["init", "-q", "-b", "main"]);
        // A user setting that hides the first commit of a history from a log
        // that does not ask for it: every fixture runs under it.
        git(&root, &["config", "log.showRoot", "false"]);
        Self {
            scratch,
            root,
            day: 0,
        }
    }

    fn put(&self, path: &str, text: &str) {
        put(&self.root, path, text);
    }

    fn git(&self, args: &[&str]) -> String {
        git(&self.root, args)
    }

    /// Add everything and commit with `message` on the next day, at noon in
    /// a zone that alternates, so the order of the instants is known and the
    /// zones are not.
    fn commit(&mut self, message: &str) -> String {
        self.day += 1;
        let zone = if self.day.is_multiple_of(2) {
            "+05:00"
        } else {
            "-03:00"
        };
        let date = format!("2026-01-{:02}T12:00:00{zone}", self.day);
        self.commit_at(message, &date)
    }

    /// Add everything and commit with `message`, authored and committed at
    /// `date`.
    fn commit_at(&self, message: &str, date: &str) -> String {
        self.git(&["add", "-A"]);
        let output = git_command(&self.root)
            .args(["commit", "-q", "-m", message])
            .env("GIT_AUTHOR_DATE", date)
            .env("GIT_COMMITTER_DATE", date)
            .output()
            .expect("git runs");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        self.git(&["rev-parse", "HEAD"])
    }

    fn runner(&self) -> SystemRunner {
        runner_in(&self.root)
    }

    fn history(&self, id: &str, options: &HistoryOptions) -> Result<History, HistoryError> {
        let runner = self.runner();
        on_disk(&self.root, |plan| plan.history(id, options, &runner))
    }
}

fn runner_in(dir: &Path) -> SystemRunner {
    SystemRunner::new(Programs::new(dir))
}

/// The commits of the story, in the order they were made.
struct Story {
    repo: Repo,
    added: String,
    moved: String,
    second: String,
    first: String,
    removed: String,
    edited: String,
}

/// The document starts in `docs/drafts`, is moved to its home, `second` and
/// then `first` change, `gone` is deleted, and the document is edited.
fn story() -> Story {
    let mut repo = Repo::new("history");
    repo.put("Cargo.toml", MANIFEST);
    repo.put("lib.rs", LIB);
    repo.put("src/alpha.rs", ALPHA_RS);
    repo.put("-rf.rs", "pub fn dashed() {}\n");
    repo.put("--output=x.rs", "pub fn output() {}\n");
    repo.put("docs/drafts/alpha.md", &capability(""));
    repo.put("docs/architecture/overview.md", ARCH);
    repo.put(
        "work/changes/grow.md",
        &change_record("grow", "[cap:alpha]", "  - cap:alpha#contract\n", ""),
    );
    repo.put("work/evidence/bound.md", &evidence_record());
    repo.put("work/assessments/alpha-contract.md", &assessment_record());
    let added = repo.commit("add the workspace");
    fs::create_dir_all(repo.root.join("docs/capabilities")).expect("a directory");
    repo.git(&["mv", "docs/drafts/alpha.md", "docs/capabilities/alpha.md"]);
    let moved = repo.commit("move the document home");
    repo.put(
        "src/alpha.rs",
        &ALPHA_RS.replacen(
            "pub fn second() {\n    let _ = 1;",
            "pub fn second() {\n    let _ = 2;",
            1,
        ),
    );
    let second = repo.commit("change second");
    let text = fs::read_to_string(repo.root.join("src/alpha.rs")).expect("the file");
    repo.put(
        "src/alpha.rs",
        &text.replacen(
            "pub fn first() {\n    let _ = 1;",
            "pub fn first() {\n    let _ = 3;",
            1,
        ),
    );
    let first = repo.commit("change first");
    let text = fs::read_to_string(repo.root.join("src/alpha.rs")).expect("the file");
    repo.put(
        "src/alpha.rs",
        &text.replacen("\npub fn gone() {}\n", "", 1),
    );
    let removed = repo.commit("delete the function");
    repo.put("docs/capabilities/alpha.md", &capability("Edited.\n"));
    let edited = repo.commit("edit the document");
    Story {
        repo,
        added,
        moved,
        second,
        first,
        removed,
        edited,
    }
}

fn commits_of<'h>(history: &'h History, source: &Source) -> Vec<&'h str> {
    history
        .entries()
        .iter()
        .filter(|entry| entry.sources().contains(source))
        .map(|entry| entry.commit().as_str())
        .collect()
}

fn symbol(path: &str) -> Source {
    Source::Symbol(path.to_string())
}

// --------------------------------------------------------------- real git

#[test]
fn a_capability_history_holds_its_document_through_a_move_and_its_bound_code() {
    let _lock = serial();
    let story = story();
    let before = snapshot(&story.repo.root);
    let history = story
        .repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect("a history");
    assert_eq!(history.id(), "cap:alpha");
    assert_eq!(history.path(), "docs/capabilities/alpha.md");

    // The document's commits include those from before the move, each with
    // the name the document had then.
    let document = commits_of(&history, &Source::Document);
    assert_eq!(document, [&story.edited, &story.moved, &story.added]);
    let paths = |commit: &str| {
        history
            .entries()
            .iter()
            .find(|entry| entry.commit().as_str() == commit)
            .map(|entry| entry.paths().to_vec())
            .expect("an entry")
    };
    assert_eq!(paths(&story.moved), ["docs/capabilities/alpha.md"]);
    assert!(paths(&story.added).contains(&"docs/drafts/alpha.md".to_string()));
    assert_eq!(paths(&story.edited), ["docs/capabilities/alpha.md"]);

    // `second` changed in one of the four commits that touched its file: the
    // report lists that one (and the commit that wrote it), never the edits
    // of `first` or the deletion of `gone`.
    let second = commits_of(&history, &symbol("demo::alpha::second"));
    assert_eq!(second, [&story.second, &story.added]);
    assert!(!second.contains(&story.first.as_str()));
    assert!(!second.contains(&story.removed.as_str()));
    assert_eq!(paths(&story.second), ["src/alpha.rs"]);

    // The files named like options have their history, and nothing named
    // `x.rs` was written by any `git`.
    assert_eq!(
        commits_of(&history, &symbol("demo::rf::dashed")),
        [&story.added]
    );
    assert_eq!(
        commits_of(&history, &symbol("demo::out::output")),
        [&story.added]
    );
    for path in ["-rf.rs", "--output=x.rs"] {
        assert!(paths(&story.added).contains(&path.to_string()), "{path}");
    }
    assert!(!story.repo.root.join("x.rs").exists());
    assert!(
        !std::env::current_dir()
            .expect("a cwd")
            .join("x.rs")
            .exists()
    );

    // The deleted symbol names the commit that removed it and its parent.
    assert_eq!(history.removed().len(), 1);
    let gone = &history.removed()[0];
    assert_eq!(gone.binding(), "demo::alpha::gone");
    assert_eq!(gone.identifier(), "gone");
    assert_eq!(gone.dirs(), ["."]);
    match gone.removal() {
        Removal::Found {
            commit,
            subject,
            last_resolved,
            ..
        } => {
            assert_eq!(commit.as_str(), story.removed);
            assert_eq!(subject, "delete the function");
            assert_eq!(
                last_resolved
                    .as_ref()
                    .map(raf_workplan::exec::Commit::as_str),
                Some(story.first.as_str())
            );
        }
        other => panic!("{other:?}"),
    }

    // Each commit once, with every source; newest first by the instant of
    // its date.
    let all: Vec<&str> = history
        .entries()
        .iter()
        .map(|entry| entry.commit().as_str())
        .collect();
    assert_eq!(
        all,
        [&story.edited, &story.second, &story.moved, &story.added]
    );
    let added = &history.entries()[3];
    assert_eq!(
        added.sources(),
        [
            Source::Document,
            symbol("demo::alpha::second"),
            symbol("demo::out::output"),
            symbol("demo::rf::dashed"),
        ]
    );
    assert!(
        history
            .entries()
            .windows(2)
            .all(|pair| pair[0].date().seconds() > pair[1].date().seconds())
    );

    // The history wrote nothing: the tree, `.git` included, is byte-identical.
    assert_eq!(snapshot(&story.repo.root), before);
}

#[test]
fn each_kind_of_id_has_the_history_of_its_file() {
    let _lock = serial();
    let story = story();
    for (id, path) in [
        ("cap:alpha", "docs/capabilities/alpha.md"),
        ("change:grow", "work/changes/grow.md"),
        ("arch:overview", "docs/architecture/overview.md"),
        ("evidence:bound", "work/evidence/bound.md"),
        (
            "assessment:alpha-contract",
            "work/assessments/alpha-contract.md",
        ),
    ] {
        let history = story
            .repo
            .history(id, &HistoryOptions::new())
            .unwrap_or_else(|error| panic!("{id}: {error}"));
        assert_eq!(history.path(), path, "{id}");
        let last = history.entries().last().expect("a commit");
        assert_eq!(last.commit().as_str(), story.added, "{id}");
        // The document of the capability had its first name then.
        let first_name = path.replace("capabilities", "drafts");
        assert!(last.paths().contains(&first_name), "{id}");
        if id != "cap:alpha" {
            assert_eq!(last.paths(), [first_name], "{id}");
            assert_eq!(history.removed().len(), 0, "{id}");
            assert!(
                history
                    .entries()
                    .iter()
                    .all(|entry| entry.sources() == [Source::Document]),
                "{id}"
            );
        }
    }
}

#[test]
fn since_limits_the_logs_and_not_the_removal_lookup() {
    let _lock = serial();
    let story = story();
    let since = |day: &str| HistoryOptions::new().with_since(Date::new(day).expect("a date"));
    let all = story
        .repo
        .history("cap:alpha", &since("2000-01-01"))
        .expect("a history");
    assert_eq!(all.entries().len(), 4);
    let none = story
        .repo
        .history("cap:alpha", &since("2099-12-31"))
        .expect("a history");
    assert_eq!(none.entries().len(), 0, "{:#?}", none.entries());
    assert_eq!(none.removed().len(), 1);
    assert!(matches!(none.removed()[0].removal(), Removal::Found { .. }));
}

#[test]
fn a_subject_with_unit_separators_and_a_hash_is_reported_whole() {
    let _lock = serial();
    let mut story = story();
    let subject = format!("odd\u{1f}{A}\u{1f}2026-01-01T00:00:00Z\u{1f}| tail");
    story
        .repo
        .put("docs/capabilities/alpha.md", &capability("Again.\n"));
    let odd = story.repo.commit(&subject);
    let history = story
        .repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect("a history");
    let first = &history.entries()[0];
    assert_eq!(first.commit().as_str(), odd);
    assert_eq!(first.subject(), subject);
    // In the table it is one escaped cell; in JSON one string.
    let table = render::table(&history);
    let row = table.lines().nth(2).expect("a row");
    assert!(row.contains("odd\\u{1f}0123456789abcdef"), "{row}");
    assert!(row.contains("\\| tail"), "{row}");
    assert_eq!(row.matches(" | ").count(), 3, "{row}");
    assert!(render::json(&history).contains("\"subject\":\"odd\\u001f0123456789abcdef"));
}

#[test]
fn the_table_and_the_json_carry_the_same_entries_in_the_same_order() {
    let _lock = serial();
    let story = story();
    let history = story
        .repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect("a history");
    let table = render::table(&history);
    let json = render::json(&history);
    let mut lines = table.lines();
    assert_eq!(lines.next(), Some("| Commit | Date | Subject | Source |"));
    assert_eq!(lines.next(), Some("| --- | --- | --- | --- |"));
    let rows: Vec<&str> = lines.by_ref().take_while(|line| !line.is_empty()).collect();
    let objects: Vec<&str> = json
        .lines()
        .filter(|line| line.starts_with("{\"kind\":\"history-entry\""))
        .collect();
    assert_eq!(rows.len(), history.entries().len());
    assert_eq!(objects.len(), rows.len());
    for ((row, object), entry) in rows.iter().zip(&objects).zip(history.entries()) {
        let commit = entry.commit().as_str();
        assert!(
            row.starts_with(&format!("| {} | {} | ", &commit[..7], entry.date().day())),
            "{row}"
        );
        assert!(row.contains(entry.subject()), "{row}");
        assert!(
            object.contains(&format!("\"commit\":\"{commit}\"")),
            "{object}"
        );
        assert!(
            object.contains(&format!("\"date\":\"{}\"", entry.date())),
            "{object}"
        );
        assert!(
            object.contains(&format!("\"subject\":\"{}\"", entry.subject())),
            "{object}"
        );
        for source in entry.sources() {
            match source {
                Source::Document => {
                    assert!(row.ends_with(" |") && row.contains("| document"), "{row}");
                    assert!(object.contains("{\"source\":\"document\"}"), "{object}");
                }
                Source::Symbol(path) => {
                    assert!(row.contains(&format!("`{path}`")), "{row}");
                    assert!(
                        object
                            .contains(&format!("{{\"source\":\"symbol\",\"binding\":\"{path}\"}}")),
                        "{object}"
                    );
                }
                _ => panic!("{source:?}"),
            }
        }
    }
    // The removed binding is the second table and the last object.
    let removed: Vec<&str> = lines.collect();
    assert_eq!(
        removed[0],
        "| Binding | Removed in | Date | Subject | Last resolved |"
    );
    assert!(removed[2].starts_with(&format!(
        "| `demo::alpha::gone` | {} | ",
        &story.removed[..7]
    )));
    assert!(removed[2].ends_with(&format!(" | {} |", &story.first[..7])));
    let last = json.lines().rev().nth(1).expect("a last object");
    assert!(
        last.starts_with("{\"kind\":\"history-removed\",\"binding\":\"demo::alpha::gone\""),
        "{last}"
    );
    assert!(
        last.contains(&format!("\"last_resolved\":\"{}\"", story.first)),
        "{last}"
    );
    assert!(json.starts_with("[\n") && json.ends_with("\n]\n"));
}

#[test]
fn a_shallow_clone_is_refused_with_the_missing_history_named_and_a_full_clone_is_not() {
    let _lock = serial();
    let story = story();
    let origin = format!("file://{}", story.repo.root.display());
    let shallow = story.repo.scratch.path().join("shallow");
    let full = story.repo.scratch.path().join("full");
    git(
        story.repo.scratch.path(),
        &["clone", "-q", "--depth", "1", &origin, "shallow"],
    );
    git(story.repo.scratch.path(), &["clone", "-q", &origin, "full"]);

    let runner = runner_in(&shallow);
    let error = on_disk(&shallow, |plan| {
        plan.history("cap:alpha", &HistoryOptions::new(), &runner)
    })
    .expect_err("a shallow clone");
    assert!(matches!(error, HistoryError::Shallow), "{error:?}");
    let message = error.to_string();
    assert!(message.contains("shallow clone"), "{message}");
    assert!(
        message.contains("commits before its shallow boundary are missing"),
        "{message}"
    );
    assert!(message.contains("git fetch --unshallow"), "{message}");

    let runner = runner_in(&full);
    let history = on_disk(&full, |plan| {
        plan.history("cap:alpha", &HistoryOptions::new(), &runner)
    })
    .expect("a full clone");
    assert_eq!(history.entries().len(), 4);
}

#[test]
fn a_root_commit_that_changed_the_identifier_has_no_parent() {
    let _lock = serial();
    let mut repo = Repo::new("history-root");
    repo.put("Cargo.toml", MANIFEST);
    repo.put("lib.rs", "pub mod alpha {}\n");
    repo.put(
        "docs/capabilities/alpha.md",
        &document(
            "alpha",
            "Alpha.",
            "implementing",
            ["[demo::never]", "[]", "[]"],
            "[]",
            &[],
        ),
    );
    let root = repo.commit("the only commit");
    // Under `log.showRoot=false` a log that does not ask for the first commit
    // shows nothing for it.
    assert_eq!(repo.git(&["log", "--format=%H", "-Snever", "--", "."]), "");
    let history = repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect("a history");
    // The document log still has the commit that created the document.
    assert_eq!(commits_of(&history, &Source::Document), [&root]);
    // The only text `never` is in the document, added by the root commit.
    match history.removed()[0].removal() {
        Removal::Found {
            commit,
            last_resolved,
            ..
        } => {
            assert_eq!(commit.as_str(), root);
            assert_eq!(*last_resolved, None);
        }
        other => panic!("{other:?}"),
    }
    assert!(render::table(&history).contains("none: the first commit"));
    assert!(render::json(&history).contains("\"last_resolved\":null"));
}

#[test]
fn a_merge_whose_resolution_changed_the_document_is_in_its_history_and_a_clean_one_is_not() {
    let _lock = serial();
    let mut repo = Repo::new("history-merge");
    repo.put("Cargo.toml", MANIFEST);
    repo.put("lib.rs", "pub mod alpha {}\n");
    repo.put("docs/capabilities/alpha.md", &capability(""));
    let added = repo.commit("add the document");
    repo.git(&["checkout", "-q", "-b", "side"]);
    repo.put(
        "docs/capabilities/alpha.md",
        &capability("From the side.\n"),
    );
    let side = repo.commit("edit on the side");
    repo.git(&["checkout", "-q", "main"]);
    repo.put("docs/capabilities/alpha.md", &capability("From main.\n"));
    let main = repo.commit("edit on main");
    // The merge stops on the conflict; the resolution is a third text.
    let merging = git_command(&repo.root)
        .args(["merge", "-q", "--no-edit", "side"])
        .output()
        .expect("git runs");
    assert!(!merging.status.success(), "the merge conflicts");
    repo.put("docs/capabilities/alpha.md", &capability("Resolved.\n"));
    let resolved = repo.commit("merge the side, resolving the document");
    assert_eq!(
        repo.git(&["rev-list", "--parents", "-n", "1", "HEAD"])
            .split(' ')
            .count(),
        3
    );
    // A clean merge that does not change the document.
    repo.git(&["checkout", "-q", "-b", "other"]);
    repo.put("notes.txt", "notes\n");
    repo.commit("add notes");
    repo.git(&["checkout", "-q", "main"]);
    let output = git_command(&repo.root)
        .args(["merge", "-q", "--no-ff", "--no-edit", "other"])
        .env("GIT_AUTHOR_DATE", "2026-02-01T12:00:00Z")
        .env("GIT_COMMITTER_DATE", "2026-02-01T12:00:00Z")
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let clean = repo.git(&["rev-parse", "HEAD"]);

    let history = repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect("a history");
    let document = commits_of(&history, &Source::Document);
    assert_eq!(
        document,
        [&resolved, &main, &side, &added],
        "clean {clean} notes {:?}",
        history.entries()[0]
    );
    assert!(!document.contains(&clean.as_str()));
    let merge = &history.entries()[0];
    assert_eq!(merge.paths(), ["docs/capabilities/alpha.md"]);
}

/// A `git` that runs the real one with `TZ` set to `zone`.
#[cfg(unix)]
fn git_in_zone(dir: &Path, zone: &str) -> PathBuf {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let path = dir.join(format!("git-{}", zone.replace('/', "-")));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o755)
        .open(&path)
        .expect("a wrapper file");
    write!(file, "#!/bin/sh\nTZ='{zone}' exec git \"$@\"\n").expect("a wrapper body");
    drop(file);
    path
}

#[cfg(unix)]
#[test]
fn since_counts_from_the_start_of_the_day_in_utc_in_every_time_zone() {
    let _lock = serial();
    let repo = Repo::new("history-since");
    repo.put("Cargo.toml", MANIFEST);
    repo.put("lib.rs", "pub mod alpha {}\n");
    let mut made = Vec::new();
    for (index, date) in [
        "2026-03-31T23:30:00+00:00",
        "2026-04-01T00:30:00+00:00",
        "2026-04-01T23:30:00+00:00",
    ]
    .iter()
    .enumerate()
    {
        repo.put(
            "docs/capabilities/alpha.md",
            &capability(&format!("Version {index}.\n")),
        );
        made.push(repo.commit_at(&format!("version {index}"), date));
    }
    let bin = repo.scratch.path().join("bin");
    fs::create_dir_all(&bin).expect("a bin directory");
    let options = HistoryOptions::new().with_since(Date::new("2026-04-01").expect("a date"));
    for zone in ["Pacific/Kiritimati", "Etc/GMT+12", "UTC"] {
        let runner = SystemRunner::new(Programs::new(&repo.root).with_git(git_in_zone(&bin, zone)));
        let history = on_disk(&repo.root, |plan| {
            plan.history("cap:alpha", &options, &runner)
        })
        .unwrap_or_else(|error| panic!("{zone}: {error}"));
        let commits: Vec<&str> = history
            .entries()
            .iter()
            .map(|entry| entry.commit().as_str())
            .collect();
        // The day before is out; both commits of the day are in.
        assert_eq!(commits, [&made[2], &made[1]], "{zone}");
    }
    // The neighbour: the day before takes all three.
    let options = HistoryOptions::new().with_since(Date::new("2026-03-31").expect("a date"));
    let history = repo.history("cap:alpha", &options).expect("a history");
    assert_eq!(history.entries().len(), 3);
}

// ------------------------------------------------------- in-process fake

/// A workspace in memory: the story's files at its last commit.
fn memory() -> (MemoryTree, Workspace) {
    let source = Workspace::load(
        &MemoryTree::from([
            ("Cargo.toml", MANIFEST),
            ("lib.rs", LIB),
            ("src/alpha.rs", ALPHA_RS),
            ("-rf.rs", "pub fn dashed() {}\n"),
            ("--output=x.rs", "pub fn output() {}\n"),
        ]),
        &["."],
    )
    .expect("the workspace loads");
    let capability = document(
        "alpha",
        "Alpha.",
        "implementing",
        [
            "[demo::alpha::second, demo::rf::dashed, nocrate::gone, demo::alpha::vanished]",
            "[]",
            "[]",
        ],
        "[]",
        &[],
    );
    let docs = MemoryTree::from([
        ("docs/capabilities/alpha.md", capability.as_str()),
        ("docs/architecture/overview.md", ARCH),
        (
            "docs/architecture/broken.md",
            "---\ntype: Architecture\nid: arch:broken\n---\n",
        ),
    ]);
    (docs, source)
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "a scripted answer of the fake is a result"
)]
fn ok(stdout: &str) -> Result<Output, raf_workplan::exec::ExecError> {
    Ok(Output::new(Some(0), stdout.as_bytes().to_vec(), Vec::new()))
}

/// The elements as string slices.
fn strs(words: &[String]) -> Vec<&str> {
    words.iter().map(String::as_str).collect()
}

/// The argv of a row's text: split at each space, a U+00A0 standing for a
/// space inside an element.
fn argv(text: &str) -> Vec<String> {
    text.split(' ')
        .map(|word| word.replace('\u{a0}', " "))
        .collect()
}

/// The answers of a full capability history: two documents commits, one of
/// them also a commit of `second`, one commit of `-rf.rs`, and the removal of
/// `vanished` with its parent.
fn scripted() -> FakeRunner {
    let doc = format!(
        "--no-pager log --no-show-signature --follow --cc --root {FORMAT} --name-only -- docs/capabilities/alpha.md"
    );
    let second =
        format!("--no-pager log --no-show-signature -L5,7:src/alpha.rs --no-patch {FORMAT} --");
    let dashed = format!("--no-pager log --no-show-signature -L1,1:-rf.rs --no-patch {FORMAT} --");
    let pickaxe = "--no-pager log --no-show-signature --root -Svanished -n 1 --format=%H%x1f%P%x1f%aI%x1f%s -- .";
    FakeRunner::new()
        .on(Program::Git, &["--version"], ok(VERSION))
        .on(
            Program::Git,
            &["rev-parse", "--is-shallow-repository"],
            ok("false\n"),
        )
        .on(
            Program::Git,
            &strs(&argv(&doc)),
            ok(&format!(
                "\x1e{B}\x1f2026-01-03T12:00:00+05:00\x1fboth\n\ndocs/capabilities/alpha.md\n\
                 \x1e{A}\x1f2026-01-01T12:00:00-03:00\x1fadd\n\ndocs/drafts/alpha.md\n"
            )),
        )
        .on(
            Program::Git,
            &strs(&argv(&second)),
            ok(&format!("\x1e{B}\x1f2026-01-03T12:00:00+05:00\x1fboth\n")),
        )
        .on(
            Program::Git,
            &strs(&argv(&dashed)),
            // Same instant as `B`, in another zone: ordered by the id.
            ok(&format!("\x1e{C}\x1f2026-01-03T07:00:00Z\x1fdash\n")),
        )
        .on(
            Program::Git,
            &strs(&argv(pickaxe)),
            ok(&format!(
                "{A}\x1f{P}\x1f2026-01-01T12:00:00-03:00\x1fremove vanished\n"
            )),
        )
}

#[test]
fn every_question_is_asked_in_order_as_exactly_its_argv() {
    let (docs, source) = memory();
    let plan = Workplan::load(&docs, &source);
    let runner = scripted();
    let history = plan
        .history("cap:alpha", &HistoryOptions::new(), &runner)
        .expect("a history");
    let calls: Vec<String> = runner
        .calls()
        .iter()
        .map(raf_workplan::exec::Call::command)
        .collect();
    assert_eq!(
        calls,
        [
            "git --version".to_string(),
            "git rev-parse --is-shallow-repository".to_string(),
            format!(
                "git --no-pager log --no-show-signature --follow --cc --root {FORMAT} --name-only -- docs/capabilities/alpha.md"
            ),
            format!(
                "git --no-pager log --no-show-signature -L5,7:src/alpha.rs --no-patch {FORMAT} --"
            ),
            format!("git --no-pager log --no-show-signature -L1,1:-rf.rs --no-patch {FORMAT} --"),
            "git --no-pager log --no-show-signature --root -Svanished -n 1 --format=%H%x1f%P%x1f%aI%x1f%s -- ."
                .to_string(),
        ]
    );
    // Every call is a query: 16 MiB, the C locale.
    for call in runner.calls() {
        assert_eq!(call.limits().max_output(), DEFAULT_OUTPUT_BOUND);
        assert!(call.limits().c_locale());
    }
    // `B` and `C` share an instant and are ordered by id, not by the text of
    // their dates; `B` holds both of its sources and both paths.
    let commits: Vec<&str> = history
        .entries()
        .iter()
        .map(|e| e.commit().as_str())
        .collect();
    assert_eq!(commits, [C, B, A]);
    assert_eq!(
        history.entries()[1].sources(),
        [Source::Document, symbol("demo::alpha::second")]
    );
    assert_eq!(
        history.entries()[1].paths(),
        ["docs/capabilities/alpha.md", "src/alpha.rs"]
    );
    assert_eq!(history.entries()[0].paths(), ["-rf.rs"]);
    // The binding of a crate no package provides asks nothing.
    let removed: Vec<(&str, &Removal)> = history
        .removed()
        .iter()
        .map(|r| (r.binding(), r.removal()))
        .collect();
    assert!(
        matches!(
            removed[..],
            [
                ("demo::alpha::vanished", Removal::Found { .. }),
                ("nocrate::gone", Removal::NoCrate)
            ]
        ),
        "{removed:?}"
    );
    assert!(render::table(&history).contains("no package of the workspace provides its crate"));
}

#[test]
fn a_symbol_log_that_git_refuses_names_the_binding_and_the_file_to_commit() {
    let (docs, source) = memory();
    let plan = Workplan::load(&docs, &source);
    let second =
        format!("--no-pager log --no-show-signature -L5,7:src/alpha.rs --no-patch {FORMAT} --");
    let refusing = scripted().on(
        Program::Git,
        &strs(&argv(&second)),
        Ok(Output::new(
            Some(128),
            Vec::new(),
            b"fatal: There is no path src/alpha.rs in the commit\n".to_vec(),
        )),
    );
    let error = plan
        .history("cap:alpha", &HistoryOptions::new(), &refusing)
        .expect_err("git refused the span");
    match &error {
        HistoryError::SymbolLog {
            binding,
            file,
            error,
        } if matches!(**error, GitError::Failed { .. }) => assert_eq!(
            (binding.as_str(), file.as_str()),
            ("demo::alpha::second", "src/alpha.rs")
        ),
        other => panic!("{other:?}"),
    }
    let message = error.to_string();
    assert!(
        message.contains("There is no path src/alpha.rs"),
        "{message}"
    );
    assert!(message.contains("commit `src/alpha.rs`"), "{message}");
    // A failing log of the document stays a failure of `git`.
    let doc = format!(
        "--no-pager log --no-show-signature --follow --cc --root {FORMAT} --name-only -- docs/capabilities/alpha.md"
    );
    let failing = scripted().on(
        Program::Git,
        &strs(&argv(&doc)),
        Ok(Output::new(Some(128), Vec::new(), b"fatal: bad\n".to_vec())),
    );
    let error = plan
        .history("cap:alpha", &HistoryOptions::new(), &failing)
        .expect_err("git failed");
    assert!(
        matches!(error, HistoryError::Git(GitError::Failed { .. })),
        "{error:?}"
    );
    // The neighbour: the same history with every answer given.
    plan.history("cap:alpha", &HistoryOptions::new(), &scripted())
        .expect("a history");
}

#[test]
fn since_reaches_both_logs_before_the_last_separator() {
    let (docs, source) = memory();
    let plan = Workplan::load(&docs, &source);
    let runner = FakeRunner::new()
        .on(Program::Git, &["--version"], ok(VERSION))
        .on(Program::Git, &["rev-parse", "--is-shallow-repository"], ok("false\n"))
        .on(
            Program::Git,
            &strs(&argv(&format!("--no-pager log --no-show-signature --follow --cc --root {FORMAT} --name-only --since=2026-02-01\u{a0}00:00:00\u{a0}+0000 -- docs/architecture/overview.md"))),
            ok(""),
        );
    let options = HistoryOptions::new().with_since(Date::new("2026-02-01").expect("a date"));
    let history = plan
        .history("arch:overview", &options, &runner)
        .expect("a history");
    assert_eq!(history.entries().len(), 0);
    assert_eq!(render::json(&history), "[]\n");
    assert_eq!(runner.calls().len(), 3);
}

#[test]
fn an_id_or_a_date_that_cannot_be_asked_about_starts_no_process() {
    let (docs, source) = memory();
    let plan = Workplan::load(&docs, &source);
    let runner = FakeRunner::new();
    for id in ["-rf", "--output=x", "", "coverage:default", "cap", "alpha"] {
        let error = plan
            .history(id, &HistoryOptions::new(), &runner)
            .expect_err(id);
        assert!(
            matches!(error, HistoryError::UnknownKind { .. }),
            "{id}: {error:?}"
        );
        assert!(
            error
                .to_string()
                .contains("cap:, change:, arch:, evidence:, assessment:")
        );
    }
    for id in [
        "cap:-rf",
        "cap:missing",
        "change:grow",
        "evidence:x",
        "assessment:x",
    ] {
        let error = plan
            .history(id, &HistoryOptions::new(), &runner)
            .expect_err(id);
        assert!(
            matches!(error, HistoryError::Unknown { .. }),
            "{id}: {error:?}"
        );
    }
    let error = plan
        .history("arch:missing", &HistoryOptions::new(), &runner)
        .expect_err("unknown");
    assert!(error.to_string().contains("arch:overview"), "{error}");
    // A file with the id that did not load is named with its findings.
    let error = plan
        .history("arch:broken", &HistoryOptions::new(), &runner)
        .expect_err("not loaded");
    assert!(
        matches!(&error, HistoryError::NotLoaded { findings, .. } if !findings.is_empty()),
        "{error:?}"
    );
    // A `--since` value is a date or nothing is built at all.
    for since in [
        "-1",
        "--all",
        "2026-13-01",
        "2026-02-29",
        "yesterday",
        "2026-1-1",
    ] {
        assert!(Date::new(since).is_err(), "{since}");
    }
    assert_eq!(runner.calls().len(), 0, "{:?}", runner.calls());
    // The neighbour: a known id asks `git`.
    let runner = scripted();
    assert!(
        plan.history("cap:alpha", &HistoryOptions::new(), &runner)
            .is_ok()
    );
    assert_ne!(runner.calls().len(), 0);
}

#[test]
fn a_binding_that_names_no_single_symbol_is_refused_before_any_process() {
    let source = Workspace::load(
        &MemoryTree::from([
            ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
            (
                "src/lib.rs",
                "pub mod left { pub fn dup() {} }\npub mod right { pub fn dup() {} }\npub use left::*;\npub use right::*;\npub fn single() {}\n",
            ),
        ]),
        &["."],
    )
    .expect("the workspace loads");
    let doc = |code: &str| {
        document(
            "alpha",
            "Alpha.",
            "implementing",
            [code, "[]", "[]"],
            "[]",
            &[],
        )
    };
    let ambiguous = doc("[demo::dup]");
    let docs = MemoryTree::from([("docs/capabilities/alpha.md", ambiguous.as_str())]);
    let plan = Workplan::load(&docs, &source);
    let runner = FakeRunner::new();
    let error = plan
        .history("cap:alpha", &HistoryOptions::new(), &runner)
        .expect_err("ambiguous");
    assert!(
        matches!(&error, HistoryError::Binding { path, .. } if path == "demo::dup"),
        "{error:?}"
    );
    assert_eq!(runner.calls().len(), 0);
    // The neighbour: a binding of one symbol is asked about.
    let single = doc("[demo::single]");
    let docs = MemoryTree::from([("docs/capabilities/alpha.md", single.as_str())]);
    let plan = Workplan::load(&docs, &source);
    let runner = FakeRunner::new()
        .on(Program::Git, &["--version"], ok(VERSION))
        .on(Program::Git, &["rev-parse", "--is-shallow-repository"], ok("false\n"))
        .on(
            Program::Git,
            &strs(&argv(&format!("--no-pager log --no-show-signature --follow --cc --root {FORMAT} --name-only -- docs/capabilities/alpha.md"))),
            ok(""),
        )
        .on(
            Program::Git,
            &strs(&argv(&format!("--no-pager log --no-show-signature -L5,5:src/lib.rs --no-patch {FORMAT} --"))),
            ok(""),
        );
    plan.history("cap:alpha", &HistoryOptions::new(), &runner)
        .expect("one symbol");
}

#[test]
fn an_old_git_or_a_shallow_answer_stops_before_any_history_is_read() {
    let (docs, source) = memory();
    let plan = Workplan::load(&docs, &source);
    let runner = FakeRunner::new().on(Program::Git, &["--version"], ok("git version 2.29.2\n"));
    let error = plan
        .history("arch:overview", &HistoryOptions::new(), &runner)
        .expect_err("too old");
    assert!(
        matches!(error, HistoryError::Git(GitError::VersionTooOld { .. })),
        "{error:?}"
    );
    assert_eq!(runner.calls().len(), 1);
    let runner = FakeRunner::new()
        .on(Program::Git, &["--version"], ok("git version 2.30.0\n"))
        .on(
            Program::Git,
            &["rev-parse", "--is-shallow-repository"],
            ok("true\n"),
        );
    let error = plan
        .history("arch:overview", &HistoryOptions::new(), &runner)
        .expect_err("shallow");
    assert!(matches!(error, HistoryError::Shallow), "{error:?}");
    assert_eq!(runner.calls().len(), 2);
}

#[test]
fn a_log_past_sixteen_mebibytes_is_an_error_and_the_bound_itself_is_read() {
    let (docs, source) = memory();
    let plan = Workplan::load(&docs, &source);
    let doc = format!(
        "--no-pager log --no-show-signature --follow --cc --root {FORMAT} --name-only -- docs/architecture/overview.md"
    );
    let with = |stdout: String| {
        FakeRunner::new()
            .on(Program::Git, &["--version"], ok(VERSION))
            .on(
                Program::Git,
                &["rev-parse", "--is-shallow-repository"],
                ok("false\n"),
            )
            .on(Program::Git, &strs(&argv(&doc)), ok(&stdout))
    };
    // One record whose subject fills the output, then its path.
    let tail = "\n\ndocs/architecture/overview.md\n";
    let mut big = format!("\x1e{A}\x1f2026-01-01T00:00:00Z\x1f");
    big.push_str(&"s".repeat(DEFAULT_OUTPUT_BOUND + 1 - big.len() - tail.len()));
    big.push_str(tail);
    let error = plan
        .history("arch:overview", &HistoryOptions::new(), &with(big.clone()))
        .expect_err("too large");
    assert!(
        matches!(&error, HistoryError::Git(GitError::LogTooLarge { limit, .. }) if *limit == DEFAULT_OUTPUT_BOUND),
        "{error:?}"
    );
    assert!(
        error
            .to_string()
            .contains("printed more than 16777216 bytes"),
        "{error}"
    );
    big.remove(big.len() - tail.len() - 1);
    assert_eq!(big.len(), DEFAULT_OUTPUT_BOUND);
    let history = plan
        .history("arch:overview", &HistoryOptions::new(), &with(big))
        .expect("exactly the bound");
    assert_eq!(history.entries().len(), 1);
}

// ------------------------------------------------------------- stub `git`

#[cfg(unix)]
mod stubs {
    use super::*;
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;

    /// A `git` that records each argv (elements separated by 0x1f, one call
    /// to a line) and answers as a repository whose logs list one commit.
    fn stub(dir: &Path, version: &str, log: &str) -> PathBuf {
        let path = dir.join("git");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o755)
            .open(&path)
            .expect("a stub file");
        write!(
            file,
            "#!/bin/sh\ndir=$(dirname \"$0\")\nprintf '%s\\037' \"$@\" >> \"$dir/argv\"\nprintf '\\n' >> \"$dir/argv\"\n\
             case \"$1\" in\n--version) echo '{version}' ;;\nrev-parse) echo false ;;\n*) {log} ;;\nesac\n"
        )
        .expect("a stub body");
        drop(file);
        path
    }

    fn calls(dir: &Path) -> Vec<Vec<String>> {
        fs::read_to_string(dir.join("argv"))
            .unwrap_or_default()
            .lines()
            .map(|line| {
                let mut parts: Vec<String> = line.split('\u{1f}').map(str::to_string).collect();
                assert_eq!(parts.pop().as_deref(), Some(""));
                parts
            })
            .collect()
    }

    fn plan_runner(scratch: &Scratch, git: &Path) -> SystemRunner {
        let work = scratch.path().join("work");
        fs::create_dir_all(&work).expect("a work directory");
        SystemRunner::new(Programs::new(work).with_git(git))
    }

    #[test]
    fn a_path_named_like_an_option_reaches_git_after_the_separator_or_the_colon() {
        let _lock = serial();
        let scratch = Scratch::new("history-stub");
        let bin = scratch.path().join("bin");
        fs::create_dir_all(&bin).expect("a bin directory");
        let git = stub(&bin, "git version 2.45.1", "true");
        let runner = plan_runner(&scratch, &git);
        let (_, source) = memory();
        let capability = document(
            "alpha",
            "Alpha.",
            "implementing",
            ["[demo::rf::dashed, demo::out::output]", "[]", "[]"],
            "[]",
            &[],
        );
        let docs = MemoryTree::from([
            ("docs/capabilities/alpha.md", capability.as_str()),
            ("docs/architecture/overview.md", ARCH),
        ]);
        let plan = Workplan::load(&docs, &source);
        let options = HistoryOptions::new().with_since(Date::new("2026-01-01").expect("a date"));
        plan.history("cap:alpha", &options, &runner)
            .expect("a history");
        let expected: Vec<Vec<String>> = [
            "--version".to_string(),
            "rev-parse\u{1f}--is-shallow-repository".to_string(),
            format!("--no-pager\u{1f}log\u{1f}--no-show-signature\u{1f}--follow\u{1f}--cc\u{1f}--root\u{1f}{FORMAT}\u{1f}--name-only\u{1f}--since=2026-01-01 00:00:00 +0000\u{1f}--\u{1f}docs/capabilities/alpha.md"),
            format!("--no-pager\u{1f}log\u{1f}--no-show-signature\u{1f}-L1,1:--output=x.rs\u{1f}--no-patch\u{1f}{FORMAT}\u{1f}--since=2026-01-01 00:00:00 +0000\u{1f}--"),
            format!("--no-pager\u{1f}log\u{1f}--no-show-signature\u{1f}-L1,1:-rf.rs\u{1f}--no-patch\u{1f}{FORMAT}\u{1f}--since=2026-01-01 00:00:00 +0000\u{1f}--"),
        ]
        .iter()
        .map(|line| line.split('\u{1f}').map(str::to_string).collect())
        .collect();
        assert_eq!(calls(&bin), expected);
        assert!(!scratch.path().join("work/x.rs").exists());

        // The document form with such a path, straight through the git
        // module: the path is the one word after `--`.
        fs::remove_file(bin.join("argv")).expect("a fresh record");
        let dash = raf_workplan::exec::TreePath::new("-rf.rs").expect("a tree path");
        raf_workplan::git::document_log(&runner, &dash, None).expect("a log");
        assert_eq!(
            calls(&bin),
            [[
                "--no-pager",
                "log",
                "--no-show-signature",
                "--follow",
                "--cc",
                "--root",
                FORMAT,
                "--name-only",
                "--",
                "-rf.rs"
            ]
            .map(str::to_string)
            .to_vec()]
        );
    }

    #[test]
    fn a_git_reporting_2_29_stops_the_history_before_any_log() {
        let _lock = serial();
        let scratch = Scratch::new("history-old");
        let bin = scratch.path().join("bin");
        fs::create_dir_all(&bin).expect("a bin directory");
        let git = stub(&bin, "git version 2.29.9", "true");
        let runner = plan_runner(&scratch, &git);
        let (docs, source) = memory();
        let plan = Workplan::load(&docs, &source);
        let error = plan
            .history("arch:overview", &HistoryOptions::new(), &runner)
            .expect_err("too old");
        assert!(
            matches!(error, HistoryError::Git(GitError::VersionTooOld { .. })),
            "{error:?}"
        );
        assert_eq!(calls(&bin), [["--version".to_string()]]);
    }

    #[test]
    fn a_real_process_that_prints_past_the_bound_is_an_error() {
        let _lock = serial();
        let scratch = Scratch::new("history-big");
        let bin = scratch.path().join("bin");
        fs::create_dir_all(&bin).expect("a bin directory");
        let git = stub(&bin, "git version 2.45.1", "head -c 17000000 /dev/zero");
        let runner = plan_runner(&scratch, &git);
        let (docs, source) = memory();
        let plan = Workplan::load(&docs, &source);
        let error = plan
            .history("arch:overview", &HistoryOptions::new(), &runner)
            .expect_err("too large");
        assert!(
            matches!(error, HistoryError::Git(GitError::LogTooLarge { .. })),
            "{error:?}"
        );
    }
}
