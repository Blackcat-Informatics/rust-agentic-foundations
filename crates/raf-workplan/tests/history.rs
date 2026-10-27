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
        self.commit_dated(message, date, date)
    }

    /// Add everything and commit with `message`, authored at `author` and
    /// committed at `committer`, as a rebase or an amend leaves a commit.
    fn commit_dated(&self, message: &str, author: &str, committer: &str) -> String {
        self.git(&["add", "-A"]);
        let output = git_command(&self.root)
            .args(["commit", "-q", "-m", message])
            .env("GIT_AUTHOR_DATE", author)
            .env("GIT_COMMITTER_DATE", committer)
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
    SystemRunner::new(Programs::new(dir).discover_git())
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

#[cfg(unix)]
#[test]
fn a_fixture_commit_obeys_an_installed_hook_and_the_successful_neighbor() {
    use std::os::unix::fs::PermissionsExt as _;

    let _lock = serial();
    let repo = Repo::new("history-fixture-hooks");
    repo.put("lib.rs", "pub fn chosen() {}\n");
    repo.git(&["add", "lib.rs"]);
    let hook = repo.root.join(".git/hooks/pre-commit");
    fs::write(
        &hook,
        "#!/bin/sh\nprintf 'fixture hook refused\\n' >&2\nexit 1\n",
    )
    .expect("a refusing hook");
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).expect("an executable hook");
    let refused = git_command(&repo.root)
        .args(["commit", "-q", "-m", "refused candidate"])
        .output()
        .expect("git runs");
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("fixture hook refused"));
    assert!(
        !git_command(&repo.root)
            .args(["rev-parse", "--verify", "HEAD"])
            .output()
            .expect("git runs")
            .status
            .success()
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), "A  lib.rs");

    fs::write(
        &hook,
        "#!/bin/sh\nprintf 'accepted\\n' > .git/hook-receipt\n",
    )
    .expect("a successful hook");
    repo.git(&["commit", "-q", "-m", "accepted candidate"]);
    assert_eq!(
        fs::read_to_string(repo.root.join(".git/hook-receipt")).expect("the hook ran"),
        "accepted\n"
    );
    assert_eq!(repo.git(&["rev-list", "--count", "HEAD"]), "1");
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
}

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
fn removed_binding_lookup_tracks_substring_counts_instead_of_semantic_removal() {
    let _lock = serial();
    let mut repo = Repo::new("history-removal-substrings");
    repo.put("Cargo.toml", MANIFEST);
    repo.put("lib.rs", "pub fn other() {}\n");
    repo.put(
        "docs/capabilities/alpha.md",
        &document(
            "alpha",
            "Alpha.",
            "implementing",
            ["[demo::gone]", "[]", "[]"],
            "[]",
            &[],
        ),
    );
    let initial = repo.commit("write an unresolved binding");
    repo.put("lib.rs", "pub fn other() {}\npub fn gone() {}\n");
    let added = repo.commit("add the function");
    assert_eq!(
        repo.history("cap:alpha", &HistoryOptions::new())
            .expect("a resolved binding")
            .removed(),
        []
    );

    repo.put("lib.rs", "pub fn other() {}\npub fn gone_long() {}\n");
    let renamed = repo.commit("rename without changing the substring count");
    let history = repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect("a history");
    assert_eq!(history.removed().len(), 1);
    let Removal::Found {
        commit,
        last_resolved,
        ..
    } = history.removed()[0].removal()
    else {
        panic!("the identifier addition remains the last count change");
    };
    assert_eq!(commit.as_str(), added);
    assert_eq!(
        last_resolved
            .as_ref()
            .map(raf_workplan::exec::Commit::as_str),
        Some(initial.as_str())
    );

    repo.put("README.md", "Unrelated prose mentions gone.\n");
    let unrelated = repo.commit("mention the identifier outside Rust");
    let history = repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect("a history");
    let Removal::Found {
        commit,
        last_resolved,
        ..
    } = history.removed()[0].removal()
    else {
        panic!("an unrelated count change also answers the heuristic");
    };
    assert_eq!(commit.as_str(), unrelated);
    assert_eq!(
        last_resolved
            .as_ref()
            .map(raf_workplan::exec::Commit::as_str),
        Some(renamed.as_str())
    );
}

#[test]
fn symbol_history_follows_a_file_rename_but_stops_at_a_move_between_files() {
    let _lock = serial();
    let mut repo = Repo::new("history-symbol-moves");
    repo.put("Cargo.toml", MANIFEST);
    repo.put("lib.rs", "pub fn chosen() { let _ = 1; }\n");
    repo.put(
        "docs/capabilities/alpha.md",
        &document(
            "alpha",
            "Alpha.",
            "implementing",
            ["[demo::chosen]", "[]", "[]"],
            "[]",
            &[],
        ),
    );
    let added = repo.commit("add the chosen function");
    repo.git(&["mv", "lib.rs", "renamed.rs"]);
    repo.put("Cargo.toml", &MANIFEST.replace("lib.rs", "renamed.rs"));
    let renamed = repo.commit("rename the crate file");
    let history = repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect("renamed history");
    assert_eq!(commits_of(&history, &symbol("demo::chosen")), [&added]);

    repo.put("renamed.rs", "mod moved;\npub use moved::chosen;\n");
    repo.put("moved.rs", "pub fn chosen() { let _ = 1; }\n");
    let moved = repo.commit("move the function into another file");
    let history = repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect("moved history");
    assert_eq!(commits_of(&history, &symbol("demo::chosen")), [&moved]);
    assert!(!commits_of(&history, &symbol("demo::chosen")).contains(&renamed.as_str()));
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
fn a_nested_workspace_keeps_git_path_bases_beside_diff_relative() {
    let _lock = serial();
    let mut repo = Repo::new("history-nested-paths");
    repo.put("nested/Cargo.toml", MANIFEST);
    repo.put("nested/lib.rs", "pub mod child;\npub fn entry() {}\n");
    repo.put("nested/child.rs", "pub fn go() {}\n");
    let page = document(
        "alpha",
        "Alpha.",
        "implementing",
        ["[demo::child, demo::entry]", "[]", "[]"],
        "[]",
        &[],
    );
    repo.put("nested/docs/capabilities/alpha.md", &page);
    repo.commit("base");
    repo.put("nested/child.rs", "pub fn go() { let _ = 1; }\n");
    let module = repo.commit("module edit");
    repo.put(
        "nested/lib.rs",
        "pub mod child;\npub fn entry() { let _ = 2; }\n",
    );
    let span = repo.commit("span edit");
    repo.put(
        "nested/docs/capabilities/alpha.md",
        &page.replace("Alpha.", "Alpha now."),
    );
    let document = repo.commit("document edit");

    let root = repo.root.join("nested");
    let runner = runner_in(&root);
    for relative in [false, true] {
        repo.git(&[
            "config",
            "diff.relative",
            if relative { "true" } else { "false" },
        ]);
        let history = on_disk(&root, |plan| {
            plan.history("cap:alpha", &HistoryOptions::new(), &runner)
        })
        .expect("history from the nested workspace");
        let paths = |commit: &str| {
            history
                .entries()
                .iter()
                .find(|entry| entry.commit().as_str() == commit)
                .expect("edited entry")
                .paths()
                .to_vec()
        };
        let prefix = if relative { "" } else { "nested/" };
        assert_eq!(
            paths(&document),
            [format!("{prefix}docs/capabilities/alpha.md")]
        );
        assert_eq!(paths(&module), [format!("{prefix}child.rs")]);
        assert_eq!(paths(&span), ["lib.rs"]);
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
}

#[test]
fn the_table_preserves_typed_entries_sources_and_removed_bindings_in_order() {
    let _lock = serial();
    let story = story();
    let history = story
        .repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect("a history");
    let table = render::table(&history);
    let mut lines = table.lines();
    assert_eq!(lines.next(), Some("| Commit | Date | Subject | Source |"));
    assert_eq!(lines.next(), Some("| --- | --- | --- | --- |"));
    let rows: Vec<&str> = lines.by_ref().take_while(|line| !line.is_empty()).collect();
    assert_eq!(rows.len(), history.entries().len());
    for (row, entry) in rows.iter().zip(history.entries()) {
        let commit = entry.commit().as_str();
        assert!(row.starts_with(&format!("| {} | {} | ", &commit[..7], entry.date().day())));
        assert!(row.contains(entry.subject()));
        for source in entry.sources() {
            match source {
                Source::Document => assert!(row.ends_with(" |") && row.contains("| document")),
                Source::Symbol(path) => assert!(row.contains(&format!("`{path}`"))),
                _ => panic!("unknown source"),
            }
        }
    }
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
    assert_eq!(history.removed().len(), 1);
    assert_eq!(history.removed()[0].binding(), "demo::alpha::gone");
}

#[test]
fn a_partial_clone_history_can_fetch_git_objects_without_writing_working_files_or_index() {
    let _lock = serial();
    let story = story();
    story
        .repo
        .git(&["config", "uploadpack.allowFilter", "true"]);
    let blob = story
        .repo
        .git(&["rev-parse", &format!("{}:src/alpha.rs", story.second)]);
    let missing_blob = format!("?{blob}");
    let files = snapshot(&story.repo.root);
    for filtered in [true, false] {
        let name = if filtered { "partial" } else { "complete" };
        let root = story.repo.scratch.path().join(name);
        let mut args = vec!["clone", "-q", "--no-local", "--no-checkout"];
        if filtered {
            args.push("--filter=blob:none");
        }
        args.extend([story.repo.root.to_str().expect("fixture UTF-8"), name]);
        git(story.repo.scratch.path(), &args);
        git(&root, &["read-tree", "HEAD"]);
        // Populate the known working files without checkout, so it does not
        // fetch the blobs before the actual history caller has a chance to.
        for (path, bytes) in &files {
            if !path.starts_with(".git/") {
                let target = root.join(path);
                fs::create_dir_all(target.parent().expect("a fixture parent")).unwrap();
                fs::write(target, bytes).unwrap();
            }
        }
        let missing = git(&root, &["rev-list", "--objects", "--missing=print", "HEAD"]);
        assert_eq!(
            missing.lines().any(|line| line == missing_blob),
            filtered,
            "the clone's actual missing object receipt: {missing}"
        );
        let before = snapshot(&root);
        let history = on_disk(&root, |plan| {
            plan.history("cap:alpha", &HistoryOptions::new(), &runner_in(&root))
        })
        .expect("the actual history fetches what Git needs");
        assert_eq!(history.entries().len(), 4);
        assert_eq!(history.removed().len(), 1);
        let after = snapshot(&root);
        let changed = support::disk::changed(&before, &after);
        if filtered {
            assert!(
                changed.iter().any(|path| path.starts_with(".git/objects/")),
                "the actual Git query fetched missing objects: {changed:?}"
            );
            assert!(changed.iter().all(|path| path.starts_with(".git/")));
            assert_eq!(before.get(".git/index"), after.get(".git/index"));
            let remaining = git(&root, &["rev-list", "--objects", "--missing=print", "HEAD"]);
            assert!(
                !remaining.lines().any(|line| line == missing_blob),
                "the symbol's historical blob was actually fetched"
            );
        } else {
            assert_eq!(
                changed.len(),
                0,
                "the complete clone needs no fetch: {changed:?}"
            );
        }
    }
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
        let runner = SystemRunner::new(
            Programs::new(&repo.root)
                .discover_git()
                .with_git(git_in_zone(&bin, zone)),
        );
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

/// A workspace of nested file modules beside an inline module and a function
/// of the crate root, with the capability `alpha` bound to `bindings`.
fn modules(tag: &str, bindings: &str) -> Repo {
    let repo = Repo::new(tag);
    repo.put("Cargo.toml", MANIFEST);
    repo.put(
        "lib.rs",
        "pub mod outer;\npub mod sibling;\n\npub mod inline {\n    pub fn d() {}\n}\n\npub fn top() {}\n",
    );
    repo.put("outer.rs", "pub mod inner;\n\npub fn a() {}\n");
    repo.put("outer/inner.rs", "pub fn b() {}\n");
    repo.put("sibling.rs", "pub fn c() {}\n");
    repo.put(
        "docs/capabilities/alpha.md",
        &document(
            "alpha",
            "Alpha.",
            "implementing",
            [bindings, "[]", "[]"],
            "[]",
            &[],
        ),
    );
    repo
}

/// Replace `from` with `to` in the file `path` of `repo`.
fn edit(repo: &Repo, path: &str, from: &str, to: &str) {
    let text = fs::read_to_string(repo.root.join(path)).expect("the file");
    assert!(text.contains(from), "{path}: {from}");
    repo.put(path, &text.replacen(from, to, 1));
}

#[test]
fn a_module_binding_lists_the_commits_of_its_files_and_not_of_a_sibling() {
    let _lock = serial();
    let mut repo = modules("history-modules", "[demo::outer, demo::inline, demo]");
    let added = repo.commit("add the modules");
    edit(
        &repo,
        "outer.rs",
        "pub fn a() {}",
        "pub fn a() {\n    let _ = 1;\n}",
    );
    let own = repo.commit("edit the module's own file");
    edit(
        &repo,
        "outer/inner.rs",
        "pub fn b() {}",
        "pub fn b() {\n    let _ = 2;\n}",
    );
    let child = repo.commit("edit a child module's file");
    edit(
        &repo,
        "sibling.rs",
        "pub fn c() {}",
        "pub fn c() {\n    let _ = 3;\n}",
    );
    let sibling = repo.commit("edit a sibling module");
    edit(
        &repo,
        "lib.rs",
        "    pub fn d() {}",
        "    pub fn d() {\n        let _ = 4;\n    }",
    );
    let inline = repo.commit("edit the inline module");
    edit(
        &repo,
        "lib.rs",
        "pub fn top() {}",
        "pub fn top() {\n    let _ = 5;\n}",
    );
    let top = repo.commit("edit a function of the crate root");
    let history = repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect("a history");
    let paths = |commit: &str| {
        history
            .entries()
            .iter()
            .find(|entry| entry.commit().as_str() == commit)
            .map(|entry| entry.paths().to_vec())
            .expect("an entry")
    };

    // The module stands for its own file and the file of the module below
    // it, as its digest does: an edit in either is listed, an edit of a
    // sibling module or of the crate root is not.
    let outer = commits_of(&history, &symbol("demo::outer"));
    assert_eq!(outer, [&child, &own, &added]);
    assert!(!outer.contains(&sibling.as_str()));
    assert!(!outer.contains(&top.as_str()));
    assert!(!outer.contains(&inline.as_str()));
    assert_eq!(paths(&own), ["outer.rs"]);
    assert_eq!(paths(&child), ["outer/inner.rs"]);
    // The commit that wrote the declaration and the files lists all three.
    for path in ["lib.rs", "outer.rs", "outer/inner.rs"] {
        assert!(paths(&added).contains(&path.to_string()), "{path}");
    }
    assert!(!paths(&added).contains(&"sibling.rs".to_string()));

    // An inline module has no file of its own: its span is its history.
    assert_eq!(
        commits_of(&history, &symbol("demo::inline")),
        [&inline, &added]
    );
    // The crate root stands for nothing below it: the log of its span
    // lists the edits of `lib.rs` and none of the files of its modules.
    let root = commits_of(&history, &symbol("demo"));
    assert!(root.contains(&top.as_str()), "{root:?}");
    assert!(root.contains(&added.as_str()), "{root:?}");
    for commit in [&own, &child, &sibling] {
        assert!(!root.contains(&commit.as_str()), "{root:?}");
    }
    // No entry is listed twice.
    let mut all: Vec<&str> = history
        .entries()
        .iter()
        .map(|entry| entry.commit().as_str())
        .collect();
    let count = all.len();
    all.sort_unstable();
    all.dedup();
    assert_eq!(all.len(), count);
}

#[test]
fn a_module_file_that_is_not_committed_names_the_binding_and_the_file_until_it_is() {
    let _lock = serial();
    let mut repo = modules("history-module-uncommitted", "[demo::outer, demo::sibling]");
    // Everything but the module's child file is committed; that file is in
    // the working tree only, so `git log --follow` lists nothing for it.
    repo.put(".gitignore", "outer/\n");
    repo.commit("add all but the child module");
    let error = repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect_err("a file with no commit");
    match &error {
        HistoryError::Uncommitted { binding, file } => assert_eq!(
            (binding.as_str(), file.as_str()),
            ("demo::outer", "outer/inner.rs")
        ),
        other => panic!("{other:?}"),
    }
    let message = error.to_string();
    assert!(
        message.contains("the file `outer/inner.rs` of the bound module `demo::outer`"),
        "{message}"
    );
    assert!(message.contains("commit `outer/inner.rs`"), "{message}");
    // The neighbour: once the file is committed, the module has its history.
    repo.put(".gitignore", "");
    let added = repo.commit("add the child module");
    let history = repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect("a history");
    let outer = commits_of(&history, &symbol("demo::outer"));
    assert_eq!(outer[0], added);
}

#[test]
fn since_selects_by_the_author_date_and_not_by_the_committer_date() {
    let _lock = serial();
    let repo = Repo::new("history-author-date");
    repo.put("Cargo.toml", MANIFEST);
    repo.put("lib.rs", "pub fn alpha() {}\n");
    repo.put("docs/capabilities/alpha.md", &capability("Version 0.\n"));
    let base = repo.commit_at("base", "2025-06-01T12:00:00+00:00");
    // Authored long before the day, committed on it, as a rebase leaves a
    // commit: Git's own `--since` would list it.
    repo.put("docs/capabilities/alpha.md", &capability("Version 1.\n"));
    let rebased = repo.commit_dated(
        "authored before, committed on the day",
        "2026-01-01T12:00:00+00:00",
        "2026-10-16T09:00:00+00:00",
    );
    // Authored on the day, committed before it: Git's own `--since` would
    // leave it out.
    repo.put("docs/capabilities/alpha.md", &capability("Version 2.\n"));
    let late = repo.commit_dated(
        "authored on the day, committed before",
        "2026-10-15T00:30:00+00:00",
        "2026-01-02T12:00:00+00:00",
    );
    let since = |day: &str| HistoryOptions::new().with_since(Date::new(day).expect("a date"));
    let history = repo
        .history("cap:alpha", &since("2026-10-15"))
        .expect("a history");
    let commits: Vec<&str> = history
        .entries()
        .iter()
        .map(|entry| entry.commit().as_str())
        .collect();
    assert_eq!(commits, [&late], "{:#?}", history.entries());
    // Every entry shown satisfies the filter by the date it shows.
    for entry in history.entries() {
        assert!(entry.date().as_str() >= "2026-10-15", "{entry:?}");
    }
    // The neighbours: a day before the oldest author date takes all three,
    // newest first by author date; a day after the newest takes none.
    let history = repo
        .history("cap:alpha", &since("2025-06-01"))
        .expect("a history");
    let commits: Vec<&str> = history
        .entries()
        .iter()
        .map(|entry| entry.commit().as_str())
        .collect();
    assert_eq!(commits, [&late, &rebased, &base]);
    let history = repo
        .history("cap:alpha", &since("2026-10-16"))
        .expect("a history");
    assert_eq!(history.entries().len(), 0, "{:#?}", history.entries());
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
    assert!(
        message.starts_with(
            "`git log -L` could not read the lines of the bound symbol `demo::alpha::second` in `src/alpha.rs` from `HEAD`: "
        ),
        "{message}"
    );
    // Committing the file is one way to read the lines, named as such and
    // not as the cause.
    assert!(
        message.ends_with(
            "; when `src/alpha.rs` is not committed, or its copy in `HEAD` ends before the line the symbol starts at in the working copy, one way to read them is to commit `src/alpha.rs` with the symbol at the lines it has now and ask again"
        ),
        "{message}"
    );
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
fn since_reaches_no_process_and_keeps_each_entry_by_its_author_date() {
    let (docs, source) = memory();
    let plan = Workplan::load(&docs, &source);
    let runner = FakeRunner::new()
        .on(Program::Git, &["--version"], ok(VERSION))
        .on(Program::Git, &["rev-parse", "--is-shallow-repository"], ok("false\n"))
        .on(
            Program::Git,
            &strs(&argv(&format!("--no-pager log --no-show-signature --follow --cc --root {FORMAT} --name-only -- docs/architecture/overview.md"))),
            ok(&format!(
                "\x1e{B}\x1f2026-02-01T01:00:00+02:00\x1fbefore the day in UTC\n\ndocs/architecture/overview.md\n\
                 \x1e{A}\x1f2026-01-31T19:00:00-05:00\x1fat the start of the day in UTC\n\ndocs/architecture/overview.md\n\
                 \x1e{C}\x1f2026-01-31T23:59:59Z\x1fthe second before\n\ndocs/architecture/overview.md\n"
            )),
        );
    let options = HistoryOptions::new().with_since(Date::new("2026-02-01").expect("a date"));
    let history = plan
        .history("arch:overview", &options, &runner)
        .expect("a history");
    // The log is asked for whole, with no date in its argv, and filtered by
    // the instant of each author date: 2026-02-01T00:00:00Z and after.
    assert_eq!(runner.calls().len(), 3);
    assert!(
        runner
            .calls()
            .iter()
            .all(|call| !call.command().contains("--since")),
        "{:?}",
        runner.calls()
    );
    let subjects: Vec<&str> = history
        .entries()
        .iter()
        .map(raf_workplan::history::Entry::subject)
        .collect();
    assert_eq!(subjects, ["at the start of the day in UTC"]);
    // The neighbour: the day before keeps all three.
    let options = HistoryOptions::new().with_since(Date::new("2026-01-31").expect("a date"));
    let history = plan
        .history("arch:overview", &options, &runner)
        .expect("a history");
    assert_eq!(history.entries().len(), 3);
}

/// A workspace in memory with a file module `outer`, its child `inner` and
/// a sibling, and the capability `alpha` bound to `outer`.
fn module_workspace() -> (MemoryTree, Workspace) {
    let source = Workspace::load(
        &MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
            ),
            ("src/lib.rs", "pub mod outer;\npub mod sibling;\n"),
            ("src/outer.rs", "pub mod inner;\n"),
            ("src/outer/inner.rs", "pub fn b() {}\n"),
            ("src/sibling.rs", "pub fn c() {}\n"),
        ]),
        &["."],
    )
    .expect("the workspace loads");
    let capability = document(
        "alpha",
        "Alpha.",
        "implementing",
        ["[demo::outer]", "[]", "[]"],
        "[]",
        &[],
    );
    let docs = MemoryTree::from([("docs/capabilities/alpha.md", capability.as_str())]);
    (docs, source)
}

#[test]
fn a_module_binding_asks_its_span_and_then_the_log_of_each_of_its_files_in_order() {
    let (docs, source) = module_workspace();
    let plan = Workplan::load(&docs, &source);
    let file_log = |path: &str| {
        format!(
            "--no-pager log --no-show-signature --follow --cc --root {FORMAT} --name-only -- {path}"
        )
    };
    let span =
        format!("--no-pager log --no-show-signature -L1,1:src/lib.rs --no-patch {FORMAT} --");
    let answered = |inner: &str| {
        FakeRunner::new()
            .on(Program::Git, &["--version"], ok(VERSION))
            .on(
                Program::Git,
                &["rev-parse", "--is-shallow-repository"],
                ok("false\n"),
            )
            .on(
                Program::Git,
                &strs(&argv(&file_log("docs/capabilities/alpha.md"))),
                ok(""),
            )
            .on(
                Program::Git,
                &strs(&argv(&span)),
                ok(&format!("\x1e{A}\x1f2026-01-01T12:00:00Z\x1fadd\n")),
            )
            .on(
                Program::Git,
                &strs(&argv(&file_log("src/outer.rs"))),
                ok(&format!(
                    "\x1e{A}\x1f2026-01-01T12:00:00Z\x1fadd\n\nsrc/outer.rs\n"
                )),
            )
            .on(
                Program::Git,
                &strs(&argv(&file_log("src/outer/inner.rs"))),
                ok(inner),
            )
    };
    let runner = answered(&format!(
        "\x1e{B}\x1f2026-01-02T12:00:00Z\x1fedit inner\n\nsrc/outer/inner.rs\n\
         \x1e{A}\x1f2026-01-01T12:00:00Z\x1fadd\n\nsrc/outer/inner.rs\n"
    ));
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
            format!("git {}", file_log("docs/capabilities/alpha.md")),
            format!("git {span}"),
            format!("git {}", file_log("src/outer.rs")),
            format!("git {}", file_log("src/outer/inner.rs")),
        ]
    );
    // The sibling module's file is never asked about; each commit is one
    // entry with every path.
    assert!(calls.iter().all(|call| !call.contains("sibling")));
    let commits: Vec<&str> = history
        .entries()
        .iter()
        .map(|e| e.commit().as_str())
        .collect();
    assert_eq!(commits, [B, A]);
    assert_eq!(history.entries()[0].sources(), [symbol("demo::outer")]);
    assert_eq!(
        history.entries()[1].paths(),
        ["src/lib.rs", "src/outer.rs", "src/outer/inner.rs"]
    );
    // A file of the module with an empty log has no commit: the history
    // fails and names it.
    let error = plan
        .history("cap:alpha", &HistoryOptions::new(), &answered(""))
        .expect_err("an uncommitted file");
    assert!(
        matches!(&error, HistoryError::Uncommitted { binding, file } if binding == "demo::outer" && file == "src/outer/inner.rs"),
        "{error:?}"
    );
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
        SystemRunner::new(Programs::new(work).discover_git().with_git(git))
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
            format!("--no-pager\u{1f}log\u{1f}--no-show-signature\u{1f}--follow\u{1f}--cc\u{1f}--root\u{1f}{FORMAT}\u{1f}--name-only\u{1f}--\u{1f}docs/capabilities/alpha.md"),
            format!("--no-pager\u{1f}log\u{1f}--no-show-signature\u{1f}-L1,1:--output=x.rs\u{1f}--no-patch\u{1f}{FORMAT}\u{1f}--"),
            format!("--no-pager\u{1f}log\u{1f}--no-show-signature\u{1f}-L1,1:-rf.rs\u{1f}--no-patch\u{1f}{FORMAT}\u{1f}--"),
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
        raf_workplan::git::document_log(&runner, &dash).expect("a log");
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
