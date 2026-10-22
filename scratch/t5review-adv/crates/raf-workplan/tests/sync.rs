// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Sync and write: idempotence, determinism, agreement with `check`, writing
//! to a disk, the `docs_sync` example, and the catalogue against the roadmap.

mod support;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use raf_source::{DiskTree, RelPath, Workspace};
use raf_workplan::{Finding, LinkContext, SyncError, SyncReport, Workplan};
use support::cases::all_cases;
use support::synced::{
    ALPHA, BETA, DELTA, GAMMA, INDEX, apply, base, check_all, generated_findings, options, sync,
    synced, unsynced, with_plan,
};
use support::{Repo, demo_files};

// --- in memory ---------------------------------------------------------------

#[test]
fn syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal() {
    let repo = unsynced();
    let first = sync(&repo).expect("syncs");
    assert!(!first.is_empty());
    let once = apply(&repo, &first);
    let second = sync(&once).expect("syncs again");
    assert!(second.is_empty(), "{second:#?}");
    assert_eq!(apply(&once, &second), once);
    // And a third time, from the synced result of the second.
    assert!(sync(&synced(&once)).expect("syncs").is_empty());
}

#[test]
fn the_report_is_a_pure_function_of_the_documents_and_is_sorted_by_path() {
    let repo = unsynced();
    let one = sync(&repo).expect("syncs");
    let two = sync(&repo).expect("syncs");
    assert_eq!(one, two);
    let paths: Vec<&str> = one.edits.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(paths, [ALPHA, BETA, GAMMA, INDEX]);
}

#[test]
fn nothing_is_in_the_report_for_a_repository_that_is_in_sync() {
    assert!(sync(&base()).expect("syncs").is_empty());
    assert_eq!(SyncReport::default(), sync(&base()).expect("syncs"));
}

#[test]
fn a_document_that_does_not_load_stops_the_sync_since_the_tables_would_miss_it() {
    for (path, from, to, unloaded) in [
        (BETA, "state: implementing", "state: finished", "cap:beta"),
        (DELTA, "type: Capability", "type: Capabilty", "cap:delta"),
    ] {
        let mut repo = base();
        let text = repo.get_mut(path).expect("the file");
        *text = text.replacen(from, to, 1);
        match sync(&repo) {
            Err(SyncError::Unloaded(ids)) => assert_eq!(ids, [unloaded]),
            other => panic!("{path}: {other:?}"),
        }
        assert!(
            !check_all(&repo).is_empty(),
            "the document's own findings are in check"
        );
    }
}

/// The kinds that say the generated content of a repository is wrong, and the
/// reader errors of an index that is not one.
fn is_generated_kind(finding: &Finding) -> bool {
    finding.kind.starts_with("okf-parse-index-")
        || [
            "index-missing",
            "generated-stale",
            "generated-unknown-kind",
            "generated-misplaced",
            "index-hand-written",
            "mixed-line-endings",
        ]
        .contains(&finding.kind)
}

/// Whether `check` has no finding about generated content.
fn generated_clean(repo: &Repo) -> bool {
    !check_all(repo).iter().any(is_generated_kind)
}

#[test]
fn check_is_clean_exactly_when_sync_has_nothing_to_write() {
    let mut repos: Vec<(String, Repo)> = vec![
        ("synced base".to_string(), base()),
        ("unsynced".to_string(), unsynced()),
    ];
    repos.extend(
        support::cases::generated_cases()
            .into_iter()
            .map(|case| (case.name, case.repo)),
    );
    repos.extend(all_cases().into_iter().map(|case| (case.name, case.repo)));
    let mut compared = 0;
    let mut clean = 0;
    for (name, repo) in &repos {
        let outcome = sync(repo);
        if matches!(outcome, Err(SyncError::Unloaded(_))) {
            // A document that does not read is the finding of its own rule.
            continue;
        }
        compared += 1;
        let nothing = matches!(&outcome, Ok(report) if report.is_empty());
        assert_eq!(
            generated_clean(repo),
            nothing,
            "{name}: check says {:#?}, sync says {outcome:?}",
            generated_findings(repo)
        );
        clean += usize::from(nothing);
    }
    assert!(compared > 30, "only {compared} fixtures compared");
    assert!(clean >= 1, "the synced base is among the clean ones");
}

#[test]
fn check_and_sync_agree_after_each_repair() {
    let mut repo = unsynced();
    for _ in 0..2 {
        let report = sync(&repo).expect("syncs");
        repo = apply(&repo, &report);
        assert!(generated_clean(&repo));
    }
}

// --- on a disk ---------------------------------------------------------------

/// A unique directory under the system temporary directory, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "raf-workplan-{tag}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path)
            .unwrap_or_else(|e| panic!("cannot create {}: {e}", path.display()));
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // The directory is under the temporary directory and only litter.
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Write `repo` and the demo workspace under `root`.
fn write_repo(root: &Path, repo: &Repo) {
    let put = |path: &str, text: &str| {
        let target = root.join(path);
        fs::create_dir_all(target.parent().expect("a parent"))
            .unwrap_or_else(|e| panic!("cannot create the directory of {path}: {e}"));
        fs::write(&target, text).unwrap_or_else(|e| panic!("cannot write {path}: {e}"));
    };
    for (path, text) in demo_files() {
        put(path, text);
    }
    for (path, text) in repo {
        put(path, text);
    }
}

fn read(root: &Path, path: &str) -> String {
    fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("cannot read {path}: {e}"))
}

/// Load the documents under `root` from the disk and run `f` on the plan.
fn on_disk<R>(root: &Path, f: impl FnOnce(&Workplan<'_>) -> R) -> R {
    let tree =
        DiskTree::new(root).unwrap_or_else(|e| panic!("cannot open {}: {e}", root.display()));
    let source = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("workspace: {e}"));
    f(&Workplan::load_with(
        &tree,
        &source,
        &LinkContext::of_tree(&tree),
    ))
}

/// The findings of `check` of a plan.
fn checked(plan: &Workplan<'_>) -> Vec<Finding> {
    plan.check()
}

fn rel(path: &str) -> RelPath {
    RelPath::new(path).unwrap_or_else(|e| panic!("`{path}` is not a path: {e}"))
}

#[test]
fn write_puts_every_edit_on_disk_and_a_second_run_writes_nothing() {
    let scratch = Scratch::new("write");
    write_repo(scratch.path(), &unsynced());
    let report = on_disk(scratch.path(), |plan| plan.sync(&options())).expect("syncs");
    assert_eq!(report.edits.len(), 4);
    Workplan::write(&report, scratch.path()).expect("writes");
    let expected = base();
    for path in [ALPHA, BETA, GAMMA, INDEX] {
        assert_eq!(read(scratch.path(), path), expected[path], "{path}");
    }
    // The untouched files are as they were, and no temporary file is left.
    assert_eq!(read(scratch.path(), DELTA), unsynced()[DELTA]);
    let leftovers: Vec<String> = fs::read_dir(scratch.path().join("docs/capabilities"))
        .expect("lists")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains("tmp"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
    // Second run: nothing to write, and writing the empty report changes nothing.
    let second = on_disk(scratch.path(), |plan| plan.sync(&options())).expect("syncs");
    assert!(second.is_empty());
    Workplan::write(&second, scratch.path()).expect("writes nothing");
    assert_eq!(read(scratch.path(), INDEX), expected[INDEX]);
    assert!(
        on_disk(scratch.path(), checked)
            .iter()
            .all(|f| !is_generated_kind(f))
    );
}

#[test]
fn write_refuses_a_target_that_is_not_an_existing_regular_file_or_the_new_index() {
    let scratch = Scratch::new("refuse");
    write_repo(scratch.path(), &unsynced());
    let root = scratch.path();
    let before = read(root, ALPHA);
    let good = (rel(ALPHA), "rewritten\n".to_string());
    let refused = |bad: &str| {
        let report = SyncReport {
            // A valid edit first: it must not be written when a later one is refused.
            edits: vec![good.clone(), (rel(bad), "x\n".to_string())],
        };
        let error = Workplan::write(&report, root).expect_err(bad);
        assert!(
            error.to_string().contains(bad) || bad.contains("outside"),
            "{bad}: {error}"
        );
        assert_eq!(read(root, ALPHA), before, "{bad}: nothing is written");
    };
    // Missing file other than the index.
    refused("docs/capabilities/ghost.md");
    assert!(!root.join("docs/capabilities/ghost.md").exists());
    // A directory.
    refused("docs/capabilities");
    // The index in a directory that does not exist.
    fs::remove_dir_all(root.join("docs/architecture")).expect("removes");
    refused("docs/architecture/index.md");
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        // A symbolic link to a file inside the root.
        symlink(root.join(BETA), root.join("docs/capabilities/link.md")).expect("links");
        refused("docs/capabilities/link.md");
        assert!(
            read(root, BETA).contains("# beta"),
            "the link target is untouched"
        );
        // A directory link that leaves the root.
        let outside = Scratch::new("outside");
        symlink(outside.path(), root.join("docs/capabilities/outside")).expect("links");
        refused("docs/capabilities/outside/x.md");
        assert_eq!(fs::read_dir(outside.path()).expect("lists").count(), 0);
    }
    // The same path twice.
    let twice = SyncReport {
        edits: vec![good.clone(), good],
    };
    assert!(Workplan::write(&twice, root).is_err());
    assert_eq!(read(root, ALPHA), before);
}

/// A directory link inside the root on the way to a target, at `docs` or at
/// `docs/capabilities`, is refused before anything is written, though the
/// file behind it is a regular file; the neighbour with real directories
/// writes the same report.
#[cfg(unix)]
#[test]
fn write_refuses_a_target_behind_a_directory_link_inside_the_root() {
    for link in ["docs", "docs/capabilities"] {
        let scratch = Scratch::new("in-root-link");
        let root = scratch.path();
        write_repo(root, &unsynced());
        let report = on_disk(root, |plan| plan.sync(&options())).expect("syncs");
        assert!(!report.is_empty());
        fs::rename(root.join(link), root.join("moved")).expect("moves");
        let up = "../".repeat(link.matches('/').count());
        std::os::unix::fs::symlink(format!("{up}moved"), root.join(link)).expect("links");
        let before = support::disk::snapshot(root);
        let error = Workplan::write(&report, root).expect_err(link);
        assert!(
            error
                .to_string()
                .contains(&format!("the directory {link} is refused")),
            "{link}: {error}"
        );
        assert_eq!(
            support::disk::snapshot(root),
            before,
            "{link}: nothing is written"
        );
        // The neighbour: the directory is real again.
        fs::remove_file(root.join(link)).expect("unlinks");
        fs::rename(root.join("moved"), root.join(link)).expect("moves back");
        Workplan::write(&report, root).expect("writes");
        assert_eq!(read(root, ALPHA), base()[ALPHA]);
    }
}

#[cfg(unix)]
#[test]
fn write_keeps_the_permissions_of_the_file_it_replaces() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new("mode");
    write_repo(scratch.path(), &unsynced());
    let target = scratch.path().join(ALPHA);
    fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).expect("chmod");
    let report = on_disk(scratch.path(), |plan| plan.sync(&options())).expect("syncs");
    Workplan::write(&report, scratch.path()).expect("writes");
    let mode = fs::metadata(&target).expect("stat").permissions().mode() & 0o777;
    assert_eq!(mode, 0o640);
}

#[test]
fn write_creates_the_index_only_where_the_directory_exists() {
    let scratch = Scratch::new("index");
    write_repo(scratch.path(), &unsynced());
    assert!(!scratch.path().join(INDEX).exists());
    let report = SyncReport {
        edits: vec![(rel(INDEX), "created\n".to_string())],
    };
    Workplan::write(&report, scratch.path()).expect("creates the index");
    assert_eq!(read(scratch.path(), INDEX), "created\n");
}

// --- the example ---------------------------------------------------------------

/// The tests that run the `docs_sync` example each start `cargo run`, which may
/// build the example and then starts it, while the others do the same: an
/// executable that one `cargo` is still writing when another starts it makes
/// the start fail with "text file busy". Every test that runs the example holds
/// this lock from its first process to its last, as the other process tests of
/// the crate hold theirs.
static EXAMPLE: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    EXAMPLE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Run the `docs_sync` example with `args`, from the crate directory.
fn docs_sync(args: &[&str]) -> Output {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command.current_dir(env!("CARGO_MANIFEST_DIR")).args([
        "run",
        "--quiet",
        "--locked",
        "-p",
        "raf-workplan",
        "--example",
        "docs_sync",
    ]);
    if !cfg!(debug_assertions) {
        command.arg("--release");
    }
    command.arg("--").args(args);
    command
        .output()
        .unwrap_or_else(|e| panic!("cannot run the docs_sync example: {e}"))
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn the_docs_sync_example_writes_the_expected_files_once_and_nothing_on_a_second_run() {
    let _serial = serial();
    let scratch = Scratch::new("example");
    let root = scratch.path();
    write_repo(root, &unsynced());
    let root_arg = root.to_str().expect("a UTF-8 path");
    let spdx = [
        "--copyright",
        "2026 Example Org <dev@example.org>",
        "--licence",
        "CC-BY-4.0",
    ];
    let with = |extra: &[&'static str]| -> Vec<&str> {
        let mut args = vec![root_arg];
        args.extend_from_slice(extra);
        args.extend_from_slice(&spdx);
        args
    };

    // --check on a stale tree: exit 1, names each file, writes nothing.
    let checked = docs_sync(&with(&["--check"]));
    assert_eq!(checked.status.code(), Some(1), "{}", text(&checked.stderr));
    let listed: BTreeSet<String> = text(&checked.stdout).lines().map(str::to_string).collect();
    assert_eq!(
        listed,
        [ALPHA, BETA, GAMMA, INDEX]
            .iter()
            .map(|p| format!("stale {p}"))
            .collect()
    );
    assert_eq!(read(root, ALPHA), unsynced()[ALPHA]);
    assert!(!root.join(INDEX).exists());

    // The first run writes the expected files and says so.
    let first = docs_sync(&with(&[]));
    assert_eq!(first.status.code(), Some(0), "{}", text(&first.stderr));
    let wrote: BTreeSet<String> = text(&first.stdout).lines().map(str::to_string).collect();
    assert_eq!(
        wrote,
        [ALPHA, BETA, GAMMA, INDEX]
            .iter()
            .map(|p| format!("rewrote {p}"))
            .collect()
    );
    let expected = base();
    for path in [ALPHA, BETA, GAMMA, INDEX] {
        assert_eq!(read(root, path), expected[path], "{path}");
    }

    // The second run writes nothing and prints nothing; --check agrees.
    let second = docs_sync(&with(&[]));
    assert_eq!(second.status.code(), Some(0), "{}", text(&second.stderr));
    assert_eq!(text(&second.stdout), "");
    let again = docs_sync(&with(&["--check"]));
    assert_eq!(again.status.code(), Some(0), "{}", text(&again.stderr));
    assert_eq!(text(&again.stdout), "");
    for path in [ALPHA, BETA, GAMMA, INDEX] {
        assert_eq!(read(root, path), expected[path], "{path}");
    }
}

#[test]
fn the_docs_sync_example_in_a_git_checkout_and_with_bad_arguments() {
    let _serial = serial();
    let scratch = Scratch::new("example-git");
    let root = scratch.path();
    write_repo(root, &unsynced());
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .unwrap_or_else(|e| panic!("git is needed for this test: {e}"));
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            text(&output.stderr)
        );
    };
    git(&["init", "--quiet"]);
    git(&["add", "-A"]);
    let root_arg = root.to_str().expect("a UTF-8 path");
    let run = docs_sync(&[
        root_arg,
        "--copyright",
        "2026 Example",
        "--licence",
        "CC-BY-4.0",
    ]);
    assert_eq!(run.status.code(), Some(0), "{}", text(&run.stderr));
    assert_eq!(text(&run.stdout).lines().count(), 4);
    assert_eq!(read(root, GAMMA), base()[GAMMA]);

    // Arguments: unknown options are refused; the SPDX flags are not needed once
    // the index exists.
    let again = docs_sync(&[root_arg]);
    assert_eq!(again.status.code(), Some(0), "{}", text(&again.stderr));
    for args in [
        vec![root_arg, "--copyright"],
        vec![
            root_arg,
            "--frobnicate",
            "--copyright",
            "x",
            "--licence",
            "y",
        ],
        vec![root_arg, "extra", "--copyright", "x", "--licence", "y"],
    ] {
        let refused = docs_sync(&args);
        assert_eq!(refused.status.code(), Some(2), "{args:?}");
        assert!(text(&refused.stderr).contains("docs_sync:"), "{args:?}");
        assert_eq!(text(&refused.stdout), "");
    }

    // A tree that sync refuses is a message and exit 1, with nothing written.
    let mut broken = base();
    broken
        .get_mut(INDEX)
        .expect("index")
        .push_str("\nHand written.\n");
    let scratch = Scratch::new("example-refused");
    write_repo(scratch.path(), &broken);
    let refused = docs_sync(&[
        scratch.path().to_str().expect("a UTF-8 path"),
        "--copyright",
        "x",
        "--licence",
        "y",
    ]);
    assert_eq!(refused.status.code(), Some(2));
    assert!(
        text(&refused.stderr).contains("index-hand-written"),
        "{}",
        text(&refused.stderr)
    );
    assert_eq!(read(scratch.path(), INDEX), broken[INDEX]);
}

// --- the catalogue is the roadmap ------------------------------------------------------

/// Undo the escaping of a table cell.
fn unescape(cell: &str) -> String {
    let mut out = String::new();
    let mut chars = cell.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            out.extend(chars.next());
        } else {
            out.push(c);
        }
    }
    out.replace("&lt;", "<")
}

/// The cells of a table row: split on pipes that are not escaped.
fn cells(row: &str) -> Vec<String> {
    let inner = row
        .strip_prefix("| ")
        .and_then(|row| row.strip_suffix(" |"))
        .unwrap_or_else(|| panic!("not a row: {row:?}"));
    let mut cells = vec![String::new()];
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let last = cells.last_mut().expect("a cell");
                last.push(c);
                last.extend(chars.next());
            }
            '|' => {
                cells.push(String::new());
                // The separator is " | ": drop the space after it.
                if chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
            _ => cells.last_mut().expect("a cell").push(c),
        }
    }
    // The space before each separator belongs to it.
    cells
        .into_iter()
        .map(|cell| cell.strip_suffix(' ').map_or(cell.clone(), str::to_string))
        .collect()
}

/// The catalogue rows as `(state, slug, link target, description, gaps)`.
fn catalogue_rows(index: &str) -> Vec<(String, String, String, String, String)> {
    let mut state = String::new();
    let mut rows = Vec::new();
    for line in index.lines() {
        if let Some(name) = line.strip_prefix("## ") {
            state = name.to_lowercase();
        } else if line.starts_with("| [") {
            let cells = cells(line);
            assert_eq!(cells.len(), 3, "{line}");
            let link = &cells[0];
            let (slug, target) = link
                .strip_prefix('[')
                .and_then(|rest| rest.split_once("]("))
                .and_then(|(slug, rest)| rest.strip_suffix(')').map(|t| (slug, t)))
                .unwrap_or_else(|| panic!("not a link: {link}"));
            rows.push((
                state.clone(),
                slug.to_string(),
                target.to_string(),
                unescape(&cells[1]),
                unescape(&cells[2]),
            ));
        }
    }
    rows
}

/// What the roadmap says, in the shape of `catalogue_rows`.
fn roadmap_rows(plan: &Workplan<'_>) -> Vec<(String, String, String, String, String)> {
    plan.status()
        .entries()
        .iter()
        .map(|entry| {
            (
                entry.state().name().to_string(),
                entry.slug().to_string(),
                format!("./{}", entry.link_target()),
                entry.description().to_string(),
                entry
                    .gaps()
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        })
        .collect()
}

#[test]
fn the_roadmap_rows_equal_the_catalogue_rows() {
    let repo = base();
    with_plan(&repo, |plan| {
        let rows = catalogue_rows(&repo[INDEX]);
        assert_eq!(rows.len(), 4);
        assert_eq!(rows, roadmap_rows(plan));
        // The description with a pipe and quotes came back unescaped.
        assert!(rows.iter().any(|r| r.3 == "Being built | with \"care\"."));
        // The states, in order.
        let states: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
        assert_eq!(
            states,
            ["delivered", "implementing", "designed", "considered"]
        );
        let order: Vec<&str> = raf_workplan::Roadmap::ORDER
            .iter()
            .map(|s| s.name())
            .collect();
        assert_eq!(states, order);
    });
}

/// The repository root: `RAF_GATE_ROOT` when set, this repository otherwise.
fn repo_root() -> PathBuf {
    let root = std::env::var_os("RAF_GATE_ROOT").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        PathBuf::from,
    );
    root.canonicalize()
        .unwrap_or_else(|e| panic!("cannot resolve the repository root {}: {e}", root.display()))
}

#[test]
fn the_catalogue_of_this_repository_equals_its_roadmap() {
    let root = repo_root();
    let tree = DiskTree::new(&root).unwrap_or_else(|e| panic!("cannot open the tree: {e}"));
    let source = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("workspace: {e}"));
    let plan = Workplan::load(&tree, &source);
    let report = plan
        .sync(&options())
        .unwrap_or_else(|e| panic!("sync: {e}"));
    // The index is either about to be created or already there.
    let index = report
        .edits
        .iter()
        .find(|(path, _)| path.as_str() == INDEX)
        .map_or_else(|| read(&root, INDEX), |(_, text)| text.clone());
    let rows = catalogue_rows(&index);
    assert!(rows.len() >= 10, "{} capabilities", rows.len());
    assert_eq!(rows, roadmap_rows(&plan));
    // The tree is not written by a sync.
    assert!(sync_is_pure(&plan));
}

/// Whether two syncs of the same plan are equal.
fn sync_is_pure(plan: &Workplan<'_>) -> bool {
    plan.sync(&options()).ok() == plan.sync(&options()).ok()
}

#[test]
fn the_docs_sync_example_asks_for_the_spdx_flags_only_when_the_index_must_be_created() {
    let _serial = serial();
    let scratch = Scratch::new("example-flags");
    write_repo(scratch.path(), &unsynced());
    let root_arg = scratch.path().to_str().expect("a UTF-8 path");
    for (args, flag) in [
        (vec![root_arg], "--copyright"),
        (vec![root_arg, "--copyright", "2026 Example"], "--licence"),
        (vec![root_arg, "--licence", "CC-BY-4.0"], "--copyright"),
    ] {
        let run = docs_sync(&args);
        assert_eq!(run.status.code(), Some(2), "{args:?}");
        assert!(
            text(&run.stderr).contains(flag),
            "{args:?}: {}",
            text(&run.stderr)
        );
        assert!(!scratch.path().join(INDEX).exists());
    }
}

#[test]
fn the_docs_sync_example_opens_the_repository_as_the_gate_does() {
    let _serial = serial();
    // A root that holds no capability document is not a repository to sync:
    // the example exits 2 with the gate's message and writes nothing.
    let empty = Scratch::new("example-empty");
    fs::write(empty.path().join("note.txt"), "x\n").expect("write");
    let refused = docs_sync(&[
        empty.path().to_str().expect("a UTF-8 path"),
        "--copyright",
        "x",
        "--licence",
        "y",
    ]);
    assert_eq!(refused.status.code(), Some(2));
    assert!(
        text(&refused.stderr).contains("docs/capabilities"),
        "{}",
        text(&refused.stderr)
    );
    assert_eq!(text(&refused.stdout), "");
    assert_eq!(fs::read_dir(empty.path()).expect("lists").count(), 1);

    // A root that does not exist is the gate's "cannot open" message.
    let gone = empty.path().join("gone");
    let refused = docs_sync(&[gone.to_str().expect("a UTF-8 path")]);
    assert_eq!(refused.status.code(), Some(2));
    assert!(text(&refused.stderr).contains("docs_sync:"));
}

#[cfg(unix)]
#[test]
fn write_changes_no_target_when_a_later_file_cannot_be_written() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new("atomic");
    let root = scratch.path();
    write_repo(root, &unsynced());
    // Three edits in path order; the second is in a directory that is read-only.
    fs::create_dir_all(root.join("docs/b")).expect("mkdir");
    fs::write(root.join("docs/b/two.md"), "two\n").expect("write");
    fs::create_dir_all(root.join("docs/a")).expect("mkdir");
    fs::write(root.join("docs/a/one.md"), "one\n").expect("write");
    fs::create_dir_all(root.join("docs/c")).expect("mkdir");
    fs::write(root.join("docs/c/three.md"), "three\n").expect("write");
    fs::set_permissions(root.join("docs/b"), fs::Permissions::from_mode(0o555)).expect("chmod");
    // A process that may write there anyway (root) cannot show the failure.
    let probe = fs::File::create(root.join("docs/b/probe"));
    if probe.is_ok() {
        fs::set_permissions(root.join("docs/b"), fs::Permissions::from_mode(0o755)).expect("chmod");
        return;
    }
    let report = SyncReport {
        edits: vec![
            (rel("docs/a/one.md"), "ONE\n".to_string()),
            (rel("docs/b/two.md"), "TWO\n".to_string()),
            (rel("docs/c/three.md"), "THREE\n".to_string()),
        ],
    };
    let result = Workplan::write(&report, root);
    fs::set_permissions(root.join("docs/b"), fs::Permissions::from_mode(0o755)).expect("chmod");
    assert!(result.is_err());
    assert_eq!(read(root, "docs/a/one.md"), "one\n");
    assert_eq!(read(root, "docs/b/two.md"), "two\n");
    assert_eq!(read(root, "docs/c/three.md"), "three\n");
    for dir in ["docs/a", "docs/b", "docs/c"] {
        let names: Vec<_> = fs::read_dir(root.join(dir)).expect("lists").collect();
        assert_eq!(names.len(), 1, "{dir}: no temporary file is left");
    }
}

#[test]
fn write_never_touches_a_file_that_is_already_there_under_a_temporary_name() {
    let scratch = Scratch::new("tmpname");
    write_repo(scratch.path(), &unsynced());
    let dir = scratch.path().join("docs/capabilities");
    let decoy = dir.join(format!(".alpha.md.{}.0.sync.tmp", std::process::id()));
    fs::write(&decoy, "decoy").expect("write");
    let plain = dir.join(".alpha.md.sync.tmp");
    fs::write(&plain, "plain").expect("write");
    let report = on_disk(scratch.path(), |plan| plan.sync(&options())).expect("syncs");
    Workplan::write(&report, scratch.path()).expect("writes");
    assert_eq!(fs::read_to_string(&decoy).expect("read"), "decoy");
    assert_eq!(fs::read_to_string(&plain).expect("read"), "plain");
    assert_eq!(read(scratch.path(), ALPHA), base()[ALPHA]);
}
