// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The dev-dependency gate over repositories written to a scratch directory:
//! a clean one passes, a broken one panics with every finding, a repository
//! without capability documents is an error, and the link rules read the
//! tracked set the gate is given, a real symlink included.

mod support;

use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use raf_source::RelPath;
use raf_workplan::{Finding, Gate, GateError};
use support::Repo;
use support::synced::{ALPHA, BETA, DELTA, GROW, OVERVIEW, base, synced, unsynced};

/// A scratch directory under the system temporary directory, named by process
/// id, a nanosecond timestamp and a counter, and removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!(
            "raf-workplan-gate-{}-{nanos}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("a scratch directory");
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    /// Write one file, creating its directories.
    fn put(&self, path: &str, text: &str) {
        let target = self.0.join(path);
        fs::create_dir_all(target.parent().expect("a file has a directory"))
            .expect("the directory of a file");
        fs::write(target, text).expect("a file");
    }

    /// Write the documents of `repo` and the demo source they bind to: a
    /// single package whose manifest is at the root.
    fn repository(&self, repo: &Repo) {
        for (path, text) in repo {
            self.put(path, text);
        }
        for (path, text) in support::demo_files() {
            self.put(path, text);
        }
    }

    fn gate(&self) -> Gate {
        Gate::workspace(&self.0)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The message a closure panics with, and `None` when it does not panic.
fn panic_message(run: impl FnOnce()) -> Option<String> {
    let payload = catch_unwind(AssertUnwindSafe(run)).err()?;
    let text = payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(ToString::to_string));
    Some(text.expect("a panic with a text"))
}

fn rel(text: &str) -> RelPath {
    RelPath::new(text).expect("a relative path")
}

/// Every file below `dir`, as paths relative to it, symlinks included.
fn files_below(dir: &Path) -> Vec<RelPath> {
    fn walk(dir: &Path, prefix: &str, out: &mut Vec<RelPath>) {
        let mut names: Vec<_> = fs::read_dir(dir)
            .expect("a listable directory")
            .map(|entry| entry.expect("an entry"))
            .collect();
        names.sort_by_key(fs::DirEntry::file_name);
        for entry in names {
            let name = entry.file_name().into_string().expect("a UTF-8 name");
            let path = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            if entry.file_type().expect("a file type").is_dir() {
                walk(&entry.path(), &path, out);
            } else {
                out.push(rel(&path));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, "", &mut out);
    out
}

fn kinds(findings: &[Finding]) -> Vec<&'static str> {
    findings.iter().map(|f| f.kind).collect()
}

/// The synced base repository with `text` appended to the overview page.
fn with_overview_link(link: &str) -> Repo {
    let mut repo = base();
    let page = repo.get_mut(OVERVIEW).expect("the base has an overview");
    page.push_str("\nSee ");
    page.push_str(link);
    page.push_str(".\n");
    repo
}

#[test]
fn a_clean_repository_passes() {
    let scratch = Scratch::new();
    scratch.repository(&base());
    // The usage of the documentation: the directory, nothing else.
    assert_eq!(scratch.gate().check(), Vec::<Finding>::new());
    scratch.gate().assert_clean();
    let explicit = scratch
        .gate()
        .source_roots(&["."])
        .try_check()
        .expect("the gate looks at the repository");
    assert_eq!(explicit, Vec::<Finding>::new());
}

#[test]
fn a_broken_repository_panics_listing_every_finding() {
    let scratch = Scratch::new();
    let mut repo = base();
    // Three faults in three files: a key the profile does not allow, a link
    // that leads nowhere and a capability that depends on nothing there is.
    let alpha = repo.get_mut(ALPHA).expect("alpha");
    *alpha = alpha.replacen("state: designed", "state: designed\ncolour: red", 1);
    let overview = repo.get_mut(OVERVIEW).expect("overview");
    overview.push_str("\nSee [gone](nowhere.md).\n");
    let beta = repo.get_mut(BETA).expect("beta");
    *beta = beta.replacen("depends: [cap:gamma]", "depends: [cap:ghost]", 1);
    scratch.repository(&repo);

    let findings = scratch.gate().check();
    for kind in ["okf-unknown-key", "link-broken", "depends-missing"] {
        assert!(
            findings.iter().any(|f| f.kind == kind),
            "{kind} in {findings:#?}"
        );
    }
    let message = panic_message(|| scratch.gate().assert_clean()).expect("the gate panics");
    assert!(
        message.contains(&format!("{} finding(s)", findings.len())),
        "{message}"
    );
    for finding in &findings {
        let at = match &finding.location {
            raf_okf::Location::Line(line) => format!("{}:{line}", finding.file),
            _ => finding.file.clone(),
        };
        let line = format!("{} {at}", finding.kind);
        assert!(message.contains(&line), "`{line}` in:\n{message}");
        assert!(message.contains(&finding.message), "{message}");
        assert!(message.contains(&format!("({})", finding.fix)), "{message}");
    }
    // The broken link is reported at its line, as `file:line`.
    let overview_line = repo[OVERVIEW]
        .lines()
        .position(|line| line.contains("[gone]"))
        .expect("the link line")
        + 1;
    assert!(
        message.contains(&format!("link-broken {OVERVIEW}:{overview_line} ")),
        "{message}"
    );
}

#[test]
fn a_missing_capabilities_directory_is_an_error() {
    let scratch = Scratch::new();
    scratch.repository(&Repo::new());
    scratch.put(OVERVIEW, &base()[OVERVIEW]);
    let error = scratch
        .gate()
        .try_check()
        .expect_err("no capabilities directory");
    assert!(
        matches!(&error, GateError::NoCapabilities { reason, .. } if reason == "does not exist"),
        "{error:?}"
    );
    let message = panic_message(|| scratch.gate().assert_clean()).expect("the gate panics");
    assert!(message.contains("docs/capabilities"), "{message}");
    assert!(message.contains("does not exist"), "{message}");
    let message = panic_message(|| drop(scratch.gate().check())).expect("check panics too");
    assert!(message.contains("does not exist"), "{message}");
}

#[test]
fn a_capabilities_directory_without_a_capability_document_is_an_error() {
    let scratch = Scratch::new();
    scratch.repository(&Repo::new());
    // The index and a page in a subdirectory are not capability documents.
    scratch.put("docs/capabilities/index.md", "x\n");
    scratch.put("docs/capabilities/sub/nested.md", "x\n");
    scratch.put("docs/capabilities/notes.txt", "x\n");
    let error = scratch
        .gate()
        .try_check()
        .expect_err("no capability document");
    assert!(
        matches!(&error, GateError::NoCapabilities { reason, .. } if reason == "holds no capability document"),
        "{error:?}"
    );
    let message = panic_message(|| scratch.gate().assert_clean()).expect("the gate panics");
    assert!(
        message.contains("holds no capability document"),
        "{message}"
    );
}

#[test]
fn a_directory_that_does_not_exist_is_an_error() {
    let scratch = Scratch::new();
    let gone = scratch.path().join("gone");
    let error = Gate::workspace(&gone)
        .try_check()
        .expect_err("no such directory");
    assert!(matches!(error, GateError::Root { .. }), "{error:?}");
    assert!(panic_message(|| Gate::workspace(&gone).assert_clean()).is_some());
}

#[test]
fn a_source_workspace_that_does_not_load_is_an_error() {
    let scratch = Scratch::new();
    scratch.repository(&base());
    let error = scratch
        .gate()
        .source_roots(&["nowhere"])
        .try_check()
        .expect_err("no such root");
    assert!(matches!(&error, GateError::Source { .. }), "{error:?}");
    let message = error.to_string();
    assert!(message.contains("nowhere"), "{message}");
    assert!(message.contains("source_roots"), "{message}");
}

#[test]
fn a_missing_changes_directory_is_the_empty_set_of_changes() {
    let scratch = Scratch::new();
    // Only the delivered capability: no change record is needed or present.
    let mut only_gamma = unsynced();
    for path in [ALPHA, BETA, DELTA, GROW] {
        only_gamma.remove(path);
    }
    scratch.repository(&synced(&only_gamma));
    assert!(!scratch.path().join("work").exists());
    scratch.gate().assert_clean();

    // A capability that is not delivered is reported by its state rule.
    let other = Scratch::new();
    let mut with_beta = unsynced();
    for path in [ALPHA, DELTA, GROW] {
        with_beta.remove(path);
    }
    other.repository(&synced(&with_beta));
    assert!(!other.path().join("work").exists());
    let findings = other.gate().check();
    assert_eq!(kinds(&findings), ["no-open-record"], "{findings:#?}");
}

#[cfg(unix)]
#[test]
fn a_link_to_a_symlink_the_tracked_set_names_is_link_symlink() {
    let scratch = Scratch::new();
    scratch.repository(&with_overview_link("[alias](../../notes/alias.md)"));
    scratch.put("notes/real.md", "# Real\n");
    std::os::unix::fs::symlink("real.md", scratch.path().join("notes/alias.md"))
        .expect("a symlink on disk");
    assert!(
        fs::symlink_metadata(scratch.path().join("notes/alias.md"))
            .expect("the symlink")
            .file_type()
            .is_symlink()
    );
    // The caller's tracked list names the symlink among the files.
    let findings = scratch
        .gate()
        .tracked_paths(files_below(scratch.path()))
        .check();
    assert_eq!(kinds(&findings), ["link-symlink"], "{findings:#?}");
    assert_eq!(findings[0].file, OVERVIEW);
    assert!(
        findings[0].message.contains("../../notes/alias.md"),
        "{findings:#?}"
    );

    // The same list without the symlink makes the link broken, not a link to
    // a symlink: the verdict follows the tracked set.
    let without_it = scratch
        .gate()
        .tracked_paths(
            files_below(scratch.path())
                .into_iter()
                .filter(|path| path.as_str() != "notes/alias.md"),
        )
        .check();
    assert_eq!(kinds(&without_it), ["link-broken"], "{without_it:#?}");

    // Without a list, the walk of the directory finds the symlink itself.
    let walked = scratch.gate().check();
    assert_eq!(kinds(&walked), ["link-symlink"], "{walked:#?}");
}

#[test]
fn a_link_to_a_file_the_tracked_set_leaves_out_is_link_broken() {
    let scratch = Scratch::new();
    scratch.repository(&with_overview_link("[note](../../notes/untracked.md)"));
    scratch.put("notes/untracked.md", "# Untracked\n");
    let all = files_below(scratch.path());
    // On disk and tracked: clean.
    assert_eq!(
        scratch.gate().tracked_paths(all.clone()).check(),
        Vec::<Finding>::new()
    );
    // On disk and not tracked: broken.
    let findings = scratch
        .gate()
        .tracked_paths(
            all.into_iter()
                .filter(|path| path.as_str() != "notes/untracked.md"),
        )
        .check();
    assert_eq!(kinds(&findings), ["link-broken"], "{findings:#?}");
    assert_eq!(findings[0].file, OVERVIEW);
    // The walk of a directory that is no checkout counts the file as tracked:
    // the documented weakening of the fallback.
    assert_eq!(scratch.gate().check(), Vec::<Finding>::new());
}

/// Run `git` in `dir` with the variables of an enclosing git command removed,
/// so setting up the fixture never touches the repository of a hook that runs
/// the test. The gate itself runs `git` with the environment it has, as the
/// documentation says.
fn git(dir: &Path, args: &[&str]) {
    let mut command = Command::new("git");
    command.arg("-C").arg(dir).args(args);
    for variable in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_COMMON_DIR",
        "GIT_PREFIX",
    ] {
        command.env_remove(variable);
    }
    let output = command.output().expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn inside_a_git_checkout_the_tracked_paths_come_from_the_index() {
    let scratch = Scratch::new();
    scratch.repository(&with_overview_link("[note](../../notes/untracked.md)"));
    scratch.put("notes/untracked.md", "# Untracked\n");
    git(scratch.path(), &["init", "-q"]);
    // Everything but the note is added to the index.
    git(
        scratch.path(),
        &[
            "add",
            "--",
            "docs",
            "work",
            "Cargo.toml",
            "src",
            "tests",
            "examples",
        ],
    );
    let findings = scratch.gate().check();
    assert_eq!(kinds(&findings), ["link-broken"], "{findings:#?}");
    assert_eq!(findings[0].file, OVERVIEW);
    // Once the note is added to the index the repository is clean.
    git(scratch.path(), &["add", "--", "notes"]);
    scratch.gate().assert_clean();
}

#[test]
fn a_work_tree_whose_index_cannot_be_listed_is_an_error_and_not_a_walk() {
    let scratch = Scratch::new();
    scratch.repository(&base());
    git(scratch.path(), &["init", "-q"]);
    // `git` still answers that the directory is inside a work tree, but it
    // cannot read the index.
    scratch.put(".git/index", "this is not an index");
    let error = scratch
        .gate()
        .try_check()
        .expect_err("a corrupt index is not a reason to walk the directory");
    assert!(matches!(error, GateError::Tracked { .. }), "{error:?}");
    assert!(error.to_string().contains("git ls-files"), "{}", error);
    assert!(panic_message(|| scratch.gate().assert_clean()).is_some());
}

/// The variable that tells the child run of the next test where to look.
const CHILD_DIR: &str = "RAF_WORKPLAN_GATE_CHILD_DIR";

#[test]
fn a_git_dir_that_names_no_repository_is_an_error_and_not_a_walk() {
    let scratch = Scratch::new();
    scratch.repository(&base());
    // `set_var` is unsafe, so the environment is given to a run of this very
    // test binary, which runs the child test below.
    let output = Command::new(std::env::current_exe().expect("the test binary"))
        .args([
            "--exact",
            "the_gate_under_a_git_dir_that_names_nothing",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD_DIR, scratch.path())
        .env("GIT_DIR", scratch.path().join("no-such-git-dir"))
        .output()
        .expect("the child run starts");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("the gate refused"),
        "{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The child of the test above. It does nothing unless the parent names the
/// directory, so an ordinary run passes it.
#[test]
fn the_gate_under_a_git_dir_that_names_nothing() {
    let Some(dir) = std::env::var_os(CHILD_DIR) else {
        return;
    };
    let error = Gate::workspace(dir)
        .try_check()
        .expect_err("GIT_DIR names no repository");
    assert!(matches!(&error, GateError::Tracked { .. }), "{error:?}");
    assert!(error.to_string().contains("GIT_DIR"), "{error}");
    println!("the gate refused: {error}");
}
