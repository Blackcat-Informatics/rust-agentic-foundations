// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native history of documents and bound code over real fixture repositories:
//! moves, line changes, removals, merges, dates and read-only refusals.

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use raf_workplan::git::GitError;
use raf_workplan::git::Repository;
use raf_workplan::history::{History, HistoryError, HistoryOptions, Removal, Source, render};
use raf_workplan::identifiers::Date;

use support::cases::{assessment_record, evidence_record};
use support::checkout::{git, git_command, serial};
use support::disk::{Scratch, on_disk, put, snapshot};
use support::fixtures::change_record;
use support::synced::document;

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

    fn runner(&self) -> Repository {
        runner_in(&self.root)
    }

    fn history(&self, id: &str, options: &HistoryOptions) -> Result<History, HistoryError> {
        let runner = self.runner();
        on_disk(&self.root, |plan| plan.history(id, options, &runner))
    }
}

fn runner_in(dir: &Path) -> Repository {
    Repository::open(dir).expect("native repository")
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
                    .map(raf_workplan::identifiers::Commit::as_str),
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
            .map(raf_workplan::identifiers::Commit::as_str),
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
            .map(raf_workplan::identifiers::Commit::as_str),
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
    for relative in [false, true] {
        repo.git(&[
            "config",
            "diff.relative",
            if relative { "true" } else { "false" },
        ]);
        let runner = runner_in(&root);
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
    let hash = "0123456789abcdef0123456789abcdef01234567";
    let subject = format!("odd\u{1f}{hash}\u{1f}2026-01-01T00:00:00Z\u{1f}| tail");
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
fn missing_partial_clone_objects_refuse_without_writes_and_a_complete_clone_succeeds() {
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
        let result = on_disk(&root, |plan| {
            plan.history("cap:alpha", &HistoryOptions::new(), &runner_in(&root))
        });
        let after = snapshot(&root);
        let changed = support::disk::changed(&before, &after);
        if filtered {
            assert!(result.is_err(), "missing native objects must refuse");
        } else {
            let history = result.expect("complete immutable objects");
            assert_eq!(history.entries().len(), 4);
            assert_eq!(history.removed().len(), 1);
        }
        assert!(changed.is_empty(), "history changed {name}: {changed:?}");
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

#[test]
fn since_counts_from_the_start_of_the_day_in_utc() {
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
    let options = HistoryOptions::new().with_since(Date::new("2026-04-01").expect("a date"));
    let history = repo.history("cap:alpha", &options).expect("a history");
    let commits: Vec<&str> = history
        .entries()
        .iter()
        .map(|entry| entry.commit().as_str())
        .collect();
    // The day before is out; both commits of the day are in.
    assert_eq!(commits, [&made[2], &made[1]]);
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

#[test]
fn unknown_and_unloaded_ids_refuse_without_changing_the_repository() {
    let _lock = serial();
    let mut repo = Repo::new("history-invalid-id");
    repo.put("Cargo.toml", MANIFEST);
    repo.put("lib.rs", "pub fn alpha() {}\n");
    repo.put("docs/architecture/overview.md", ARCH);
    repo.put(
        "docs/architecture/broken.md",
        "---\ntype: Architecture\nid: arch:broken\n---\n",
    );
    repo.commit("add documents");
    let before = snapshot(&repo.root);
    for id in ["-rf", "--output=x", "", "coverage:default", "cap", "alpha"] {
        let error = repo.history(id, &HistoryOptions::new()).expect_err(id);
        assert!(
            matches!(error, HistoryError::UnknownKind { .. }),
            "{id}: {error:?}"
        );
    }
    for id in ["cap:missing", "change:grow", "evidence:x", "assessment:x"] {
        let error = repo.history(id, &HistoryOptions::new()).expect_err(id);
        assert!(
            matches!(error, HistoryError::Unknown { .. }),
            "{id}: {error:?}"
        );
    }
    let error = repo
        .history("arch:missing", &HistoryOptions::new())
        .expect_err("unknown");
    assert!(error.to_string().contains("arch:overview"), "{error}");
    let error = repo
        .history("arch:broken", &HistoryOptions::new())
        .expect_err("not loaded");
    assert!(
        matches!(&error, HistoryError::NotLoaded { findings, .. } if !findings.is_empty()),
        "{error:?}"
    );
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
    let history = repo
        .history("arch:overview", &HistoryOptions::new())
        .expect("a loaded id");
    assert_eq!(history.entries().len(), 1);
    assert_eq!(snapshot(&repo.root), before);
}

#[test]
fn ambiguous_bindings_refuse_and_a_single_symbol_has_native_history() {
    let _lock = serial();
    let mut repo = Repo::new("history-ambiguous");
    repo.put("Cargo.toml", MANIFEST);
    repo.put(
        "lib.rs",
        "pub mod left { pub fn dup() {} }\npub mod right { pub fn dup() {} }\npub use left::*;\npub use right::*;\npub fn single() {}\n",
    );
    let doc = |binding| {
        document(
            "alpha",
            "Alpha.",
            "implementing",
            [binding, "[]", "[]"],
            "[]",
            &[],
        )
    };
    repo.put("docs/capabilities/alpha.md", &doc("[demo::dup]"));
    let added = repo.commit("add ambiguous binding");
    let before = snapshot(&repo.root);
    let error = repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect_err("ambiguous");
    assert!(
        matches!(&error, HistoryError::Binding { path, .. } if path == "demo::dup"),
        "{error:?}"
    );
    assert_eq!(snapshot(&repo.root), before);
    repo.put("docs/capabilities/alpha.md", &doc("[demo::single]"));
    let history = repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect("one symbol");
    assert_eq!(commits_of(&history, &symbol("demo::single")), [&added]);
}

#[test]
fn a_span_beyond_the_committed_file_names_the_binding_until_committed() {
    let _lock = serial();
    let mut repo = Repo::new("history-committed-span");
    repo.put("Cargo.toml", MANIFEST);
    repo.put("lib.rs", "pub fn first() {}\n");
    repo.put(
        "docs/capabilities/alpha.md",
        &document(
            "alpha",
            "Alpha.",
            "implementing",
            ["[demo::second]", "[]", "[]"],
            "[]",
            &[],
        ),
    );
    repo.commit("add one line");
    repo.put("lib.rs", "pub fn first() {}\n\npub fn second() {}\n");
    let before = snapshot(&repo.root);
    let error = repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect_err("span beyond HEAD");
    assert!(
        matches!(&error, HistoryError::SymbolLog { binding, file, .. } if binding == "demo::second" && file == "lib.rs"),
        "{error:?}"
    );
    assert!(error.to_string().contains("committed file"), "{error}");
    assert_eq!(snapshot(&repo.root), before);
    let added = repo.commit("add the second symbol");
    let history = repo
        .history("cap:alpha", &HistoryOptions::new())
        .expect("a committed span");
    assert_eq!(commits_of(&history, &symbol("demo::second")), [&added]);
}

#[test]
fn oversized_native_commits_refuse_and_the_exact_object_bound_is_read() {
    const BOUND: usize = 16 * 1024 * 1024;

    let _lock = serial();
    let mut repo = Repo::new("history-object-bound");
    repo.put("Cargo.toml", MANIFEST);
    repo.put("lib.rs", "pub fn alpha() {}\n");
    repo.put("docs/architecture/overview.md", ARCH);
    repo.commit("add the architecture");
    let tree = repo.git(&["rev-parse", "HEAD^{tree}"]);
    let object_path = repo.scratch.path().join("commit.txt");
    for extra in [1, 0] {
        let mut object = format!(
            "tree {tree}\nauthor Fixture <fixture@example.invalid> 1767268800 +0000\ncommitter Fixture <fixture@example.invalid> 1767268800 +0000\n\nBounded commit\n\n"
        );
        object.push_str(&"s".repeat(BOUND + extra - object.len()));
        fs::write(&object_path, object).expect("a raw fixture commit");
        let id = repo.git(&[
            "hash-object",
            "-t",
            "commit",
            "-w",
            object_path.to_str().expect("fixture UTF-8"),
        ]);
        repo.git(&["update-ref", "refs/heads/main", &id]);
        let before = snapshot(&repo.root);
        let result = repo.history("arch:overview", &HistoryOptions::new());
        if extra == 1 {
            assert!(
                matches!(result, Err(HistoryError::Git(GitError::Native(_)))),
                "{result:?}"
            );
        } else {
            let history = result.expect("exact native object byte bound");
            assert_eq!(history.entries().len(), 1);
            assert_eq!(history.entries()[0].commit().as_str(), id);
        }
        assert_eq!(snapshot(&repo.root), before);
    }
}
