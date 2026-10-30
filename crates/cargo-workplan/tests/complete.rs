// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `change complete` through the built binary and the real `git` and `cargo`:
//! the refusal that lists every failure and writes nothing (exit 1), the
//! completion that rewrites the generated blocks and removes the record
//! (exit 0), what stops it looking (exit 2), `-q`, the typed RDF output, and the
//! whole path from `change new` to a completed change.

mod support;

use raf_workplan::exec::Programs;
use raf_workplan::lifecycle::Lifecycle;
use support::completion::{CAP, DONE, GOOD, PASSES, RECORD, ready, shaped};
use support::{BLOCKS, Repo, capability, changed, rdf::Rdf};

/// The text the library gives for a refusal of the same repository.
fn library(repo: &Repo, slug: &str) -> String {
    Lifecycle::open(Programs::new(repo.root()).discover_git())
        .expect("the lifecycle opens")
        .complete(slug)
        .expect("the gate gives an answer")
        .to_string()
}

#[test]
fn a_capability_left_implementing_is_refused_and_nothing_is_written() {
    let repo = shaped("complete-implementing", "implementing", PASSES, GOOD, DONE);
    let before = repo.worktree();
    let run = repo.run(&["change", "complete", "shape"]).expect(1);
    assert_eq!(
        run.lines(),
        [
            "refused: 1 finding once the record is removed",
            "once the record is removed, docs/capabilities/fixture.md: `state`: state implementing requires an open change record that lists cap:fixture [no-open-record]; list the capability in the `changes` of an open change record, or move the state to delivered",
        ]
    );
    assert_eq!(run.stderr, "");
    assert_eq!(repo.worktree(), before, "a refusal wrote");
    assert!(repo.root().join(RECORD).is_file());
    // The text is the library's.
    assert_eq!(run.stdout, library(&repo, "shape"));
    assert_eq!(repo.both(&["change", "complete", "shape"]), run);

    // The neighbour: the author sets the state and it completes.
    repo.edit(CAP, "state: implementing", "state: delivered");
    repo.sync();
    repo.run(&["change", "complete", "shape"]).expect(0);
}

#[test]
fn a_completed_change_prints_what_was_removed_and_rewritten_and_leaves_a_clean_check() {
    let repo = ready("complete-done");
    repo.run(&["check"]).expect(1);
    let before = repo.worktree();
    let run = repo.run(&["change", "complete", "shape"]).expect(0);
    assert_eq!(
        run.stdout,
        "completed: removed work/changes/shape.md\nrewrote docs/capabilities/fixture.md\n"
    );
    assert_eq!(run.stderr, "");
    // The record is gone, the blocks it staled are rewritten, and nothing else
    // moved.
    assert!(!repo.root().join(RECORD).exists());
    assert_eq!(changed(&before, &repo.worktree()), [CAP, RECORD]);
    assert!(!repo.read(CAP).contains("work/changes/shape.md"));
    repo.both(&["check"]).expect(0);
    repo.git(&["add", "-A"]);
    // The capability and the removed record, beside the assessment the
    // fixture recorded before the run, which `complete` leaves as it is.
    let mut status: Vec<String> = repo
        .git(&["status", "--short"])
        .lines()
        .map(str::to_string)
        .collect();
    status.sort();
    assert_eq!(
        status,
        [
            "A  work/assessments/fixture-contract.md".to_string(),
            format!("D  {RECORD}"),
            format!("M  {CAP}"),
        ]
    );

    // Removing the live record does not remove its committed account.
    let historical = repo.git(&["show", "main:work/changes/shape.md"]);
    let old_record =
        String::from_utf8(before[RECORD].clone()).expect("the authored record is UTF-8");
    assert_eq!(historical, old_record.trim_end_matches('\n'));

    // A second run finds no record.
    let again = repo.run(&["change", "complete", "shape"]).expect(2);
    assert!(
        again.stderr.contains("no change record `shape`"),
        "{again:#?}"
    );
}

#[test]
fn both_shapes_complete_the_same_way() {
    relative_git_objects_keep_the_subdirectory_authority();
    original_git_selectors_do_not_escape_into_the_owned_view();
    let repo = ready("complete-shapes");
    let run = repo
        .both_writing(&["change", "complete", "shape"])
        .expect(0);
    assert!(
        run.stdout
            .starts_with("completed: removed work/changes/shape.md\n")
    );
    for (format, shape) in [
        ("turtle", support::Shape::Direct),
        ("ntriples", support::Shape::Cargo),
    ] {
        let repo = ready("complete-native-view");
        let before = repo.worktree();
        // Relative selection still yields canonical native process provenance.
        let run = support::run_in(
            &repo.root().join("src"),
            shape,
            &[
                "--root", "..", "--format", format, "change", "complete", "shape",
            ],
        )
        .expect(0);
        assert_eq!(changed(&before, &repo.worktree()), [CAP, RECORD]);
        let rdf = Rdf::of(&run);
        let runs = rdf.rows("VerificationRun");
        assert!(
            !runs.is_empty(),
            "actual completion retains its process receipts"
        );
        for run in runs {
            assert_eq!(run.required("viewKind"), "materialized");
            assert_eq!(run.required("filesystemBackend"), "bwrap");
            assert_eq!(run.required("isolatedRoot"), repo.root().to_str().unwrap());
            assert_eq!(
                std::path::Path::new(run.required("logicalRoot")),
                repo.root()
            );
            let physical = std::path::Path::new(run.required("physicalRoot"));
            assert_ne!(physical, repo.root());
            assert!(
                !physical.exists(),
                "completed-view cleanup precedes publication"
            );
            assert_eq!(run.required("viewDigest").len(), 64);
            assert_eq!(run.required("originalIndex").len(), 40);
            assert_eq!(run.required("originalHead").len(), 40);
            assert_eq!(run.required("mergeBase").len(), 40);
            assert!(run.number("originalTracked") > 0);
        }
    }
}

fn relative_git_objects_keep_the_subdirectory_authority() {
    let template = ready("complete-relative-template");
    let outer = Repo::bare("complete-relative-outer");
    for (path, bytes) in template.worktree() {
        if !path.ends_with('/') {
            outer.put(
                &format!("nested/{path}"),
                std::str::from_utf8(&bytes).unwrap(),
            );
        }
    }
    outer.checkout();
    let nested = outer.root().join("nested");
    let capability = nested.join(CAP);
    let text = std::fs::read_to_string(&capability).unwrap().replace(
        "Fixture states its contract here.",
        "Fixture states its nested contract here.",
    );
    std::fs::write(&capability, text).unwrap();
    support::assessed::assess_on_disk(&nested, "shape");
    let admin = std::path::PathBuf::from(outer.git(&["rev-parse", "--absolute-git-dir"]));
    std::fs::rename(admin.join("objects"), admin.join("relative-objects")).unwrap();
    std::fs::create_dir(admin.join("objects")).unwrap();
    let before = outer.worktree();
    let args = ["change", "complete", "shape"];
    let mut command = support::command(&nested, support::Shape::Direct, &args);
    command
        .env("GIT_OBJECT_DIRECTORY", admin.join("objects"))
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES");
    let missing = support::finish(&mut command).expect(2);
    assert!(
        !missing.stderr.is_empty(),
        "missing original objects retain their native cause"
    );
    assert_eq!(outer.worktree(), before);
    command.env("GIT_ALTERNATE_OBJECT_DIRECTORIES", ".git/relative-objects");
    // Native Git resolves this relative alternate after discovering its top.
    let native = std::process::Command::new("git")
        .current_dir(&nested)
        .args(["rev-parse", "--verify", "HEAD"])
        .env("GIT_OBJECT_DIRECTORY", admin.join("objects"))
        .env("GIT_ALTERNATE_OBJECT_DIRECTORIES", ".git/relative-objects")
        .output()
        .unwrap();
    assert!(
        native.status.success(),
        "{}",
        String::from_utf8_lossy(&native.stderr)
    );
    support::finish(&mut command).expect(0);
    assert_eq!(
        changed(&before, &outer.worktree()),
        [format!("nested/{CAP}"), format!("nested/{RECORD}")]
    );
    assert!(!nested.join(RECORD).exists());
}

const OWNED_GIT_ASSERTIONS: &str = r#"
        assert_eq!(super::add(1, 2), 3);
        for selector in ["GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR", "GIT_INDEX_FILE", "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES", "GIT_PREFIX"] {
            assert!(std::env::var_os(selector).is_none(), "inherited selector {selector}");
        }
        let index = std::process::Command::new("git")
            .args(["rev-parse", "--path-format=absolute", "--git-path", "index"])
            .output().unwrap();
        assert!(index.status.success());
        assert_eq!(
            std::path::Path::new(std::str::from_utf8(&index.stdout).unwrap().trim_end()),
            std::env::current_dir().unwrap().join(".git/index"),
            "Git discovers the projected owned index"
        );
        let listed = std::process::Command::new("git")
            .args(["ls-files", "--", "work/changes/shape.md"])
            .output().unwrap();
        assert!(listed.status.success());
        assert!(listed.stdout.is_empty(), "owned index retains removed record");
        let selected = std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "index-only-input"])
            .output().unwrap();
        assert!(selected.status.success(), "owned index lost the supplied alternate-index entry");
        let proof = std::process::Command::new("git")
            .args(["rev-parse", "--git-path", "raf-private-filter-proof"])
            .output().unwrap();
        assert!(proof.status.success());
        assert_eq!(std::fs::read(std::str::from_utf8(&proof.stdout).unwrap().trim_end()).unwrap(), b"private\n");
    "#;

fn original_git_selectors_do_not_escape_into_the_owned_view() {
    let repo = ready("complete-alternate-index");
    #[cfg(unix)]
    native_source_reads_logical_absolute_links(&repo);
    #[cfg(unix)]
    let _independent = independent_output_locator(&repo);
    repo.edit(
        "src/lib.rs",
        "assert_eq!(super::add(1, 2), 3);",
        OWNED_GIT_ASSERTIONS,
    );
    repo.assess("shape");
    let admin = std::path::PathBuf::from(repo.git(&["rev-parse", "--absolute-git-dir"]));
    let alternate = admin.join("alternate-index");
    std::fs::copy(admin.join("index"), &alternate).unwrap();
    repo.put("index-only-input", "staged in the supplied index\n");
    let staged = std::process::Command::new("git")
        .current_dir(repo.root())
        .args(["add", "--", "index-only-input"])
        .env("GIT_DIR", &admin)
        .env("GIT_WORK_TREE", repo.root())
        .env("GIT_COMMON_DIR", &admin)
        .env("GIT_INDEX_FILE", &alternate)
        .output()
        .unwrap();
    assert!(
        staged.status.success(),
        "{}",
        String::from_utf8_lossy(&staged.stderr)
    );
    #[cfg(unix)]
    private_preparation_filter(&repo);
    let before_index = std::fs::read(&alternate).unwrap();
    let primary_objects = admin.join("objects");
    let external_objects = admin.join("quoted:objects-é\t\"back\\slash\n");
    std::fs::rename(&primary_objects, &external_objects).unwrap();
    std::fs::create_dir(&primary_objects).unwrap();
    repo.git(&["config", "core.quotePath", "true"]);
    let object_alternates = purrdf_lex::json::write_compact(&purrdf_lex::json::Value::String(
        external_objects.to_str().unwrap().to_owned(),
    ));
    let object_inventory = || {
        let output = std::process::Command::new("git")
            .current_dir(repo.root())
            .args(["count-objects", "-v"])
            .env("GIT_DIR", &admin)
            .env("GIT_WORK_TREE", repo.root())
            .env("GIT_COMMON_DIR", &admin)
            .env("GIT_INDEX_FILE", &alternate)
            .env("GIT_OBJECT_DIRECTORY", &primary_objects)
            .env("GIT_ALTERNATE_OBJECT_DIRECTORIES", &object_alternates)
            .env_remove("GIT_PREFIX")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    };
    let before_objects = object_inventory();
    let before = repo.worktree();
    let args = ["--format", "turtle", "change", "complete", "shape"];
    let mut command = support::command(repo.root(), support::Shape::Direct, &args);
    command
        .env("GIT_DIR", &admin)
        .env("GIT_WORK_TREE", repo.root())
        .env("GIT_COMMON_DIR", &admin)
        .env("GIT_INDEX_FILE", &alternate)
        .env("GIT_OBJECT_DIRECTORY", &primary_objects)
        .env("GIT_ALTERNATE_OBJECT_DIRECTORIES", &object_alternates)
        .env("GIT_PREFIX", "parent-prefix/");
    let run = support::finish(&mut command).expect(0);
    assert_eq!(
        object_inventory(),
        before_objects,
        "preparation wrote into an original object store"
    );
    assert_eq!(
        std::fs::read(&alternate).unwrap(),
        before_index,
        "original alternate index changed"
    );
    assert_eq!(changed(&before, &repo.worktree()), [CAP, RECORD]);
    let rdf = Rdf::of(&run);
    let rows = rdf.rows("VerificationRun");
    assert!(!rows.is_empty());
    for row in rows {
        assert_eq!(
            row.required("originalIndexPath"),
            alternate.to_str().unwrap()
        );
        assert_ne!(row.required("ownedIndexPath"), alternate.to_str().unwrap());
        assert_eq!(
            row.required("originalGitDirectory"),
            admin.to_str().unwrap()
        );
        assert_eq!(row.required("filesystemBackend"), "bwrap");
    }
}

#[cfg(unix)]
fn private_preparation_filter(repo: &Repo) {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(repo.root(), std::fs::Permissions::from_mode(0o755)).unwrap();
    repo.put("private-input", "private copied contents\n");
    std::fs::set_permissions(
        repo.root().join("private-input"),
        std::fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    repo.put("private-dir/child", "protected by its original parent\n");
    std::fs::set_permissions(
        repo.root().join("private-dir"),
        std::fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    repo.put(
        "private_filter.rs",
        r#"
        use std::os::unix::fs::PermissionsExt as _;
        fn main() -> std::io::Result<()> {
            // Original observations may run the filter too. The legitimate
            // owned worktree has a Git file, unlike this fixture's original.
            if std::fs::metadata(".git")?.is_file() {
                let git = std::fs::read_to_string(".git")?;
                let admin = std::path::Path::new(git.trim_end().strip_prefix("gitdir: ").unwrap());
                let proof = admin.join("raf-private-filter-proof");
                if !proof.try_exists()? {
                    let mode = std::fs::metadata(".")?.permissions().mode() & 0o777;
                    assert_eq!(mode, 0o700, "private root during first generated staging");
                    std::fs::write(&proof, b"private\n")?;
                }
                for (path, expected) in [("private-input", 0o600), ("private-dir", 0o700)] {
                    let mode = std::fs::metadata(path)?.permissions().mode() & 0o777;
                    assert_eq!(mode, expected, "private preparation permissions for {path}");
                }
            }
            std::io::copy(&mut std::io::stdin().lock(), &mut std::io::stdout().lock())?;
            Ok(())
        }
    "#,
    );
    let compiled = std::process::Command::new("rustc")
        .current_dir(repo.root())
        .args([
            "--edition=2024",
            "private_filter.rs",
            "-o",
            ".private-filter",
        ])
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    repo.put(".gitattributes", "*.md filter=private-preparation\n");
    repo.git(&[
        "config",
        "filter.private-preparation.clean",
        "./.private-filter",
    ]);
    repo.git(&["config", "filter.private-preparation.required", "true"]);
}

#[cfg(unix)]
fn native_source_reads_logical_absolute_links(repo: &Repo) {
    repo.put(".private/module.rs", "pub fn value() -> u32 { 7 }\n");
    std::os::unix::fs::symlink(
        repo.root().join(".private/module.rs"),
        repo.root().join("absolute-source.rs"),
    )
    .expect("absolute native Source input");
    repo.edit(
        "src/lib.rs",
        "    a + b",
        "    absolute::value();\n    a + b",
    );
    repo.put(
        "src/lib.rs",
        &format!(
            "{}\n#[path = \"../absolute-source.rs\"] mod absolute;\n",
            repo.read("src/lib.rs")
        ),
    );
    repo.edit(
        CAP,
        "code: [demo::add]",
        "code: [demo::add, demo::absolute::value]",
    );
    let check = format!(
        r#"
        let run = std::process::Command::new({:?})
            .args(["--root", env!("CARGO_MANIFEST_DIR"), "status"])
            .output().unwrap();
        assert!(run.status.success(), "native Source refused logical link: {{}}", String::from_utf8_lossy(&run.stderr));
        assert_eq!(super::add(1, 2), 3);
    "#,
        support::BINARY
    );
    repo.edit("src/lib.rs", "assert_eq!(super::add(1, 2), 3);", &check);
}

#[cfg(unix)]
fn independent_output_locator(repo: &Repo) -> support::Scratch {
    let output = support::Scratch::new("independent-cargo-output");
    repo.put(
        ".independent/Cargo.toml",
        "[package]\nname = 'independent'\nversion = '0.1.0'\nedition = '2021'\n[workspace]\n",
    );
    repo.put(
        ".independent/Cargo.lock",
        "version = 4\n[[package]]\nname = 'independent'\nversion = '0.1.0'\n",
    );
    repo.put(".independent/src/lib.rs", "pub fn value() {}\n");
    repo.put(".independent/.cargo/config.toml", &output.cargo_config());
    std::os::unix::fs::symlink(
        output.target(),
        repo.root().join(".independent/output-locator"),
    )
    .expect("evidenced independent workspace output locator");
    // An unrelated malformed manifest is data, not a metadata-query candidate.
    repo.put(".invalid-fixture/Cargo.toml", "deliberately invalid TOML [");
    output
}

#[test]
fn every_failure_is_listed_and_each_condition_is_named_in_rdf() {
    // An item with no outcome is an obligation (condition 1) and a failure of
    // condition 2; a capability left implementing is reported once the record
    // is removed. The bound test fails too, but a static failure spends no
    // process on `cargo`, so it is not run and not reported.
    let repo = shaped(
        "complete-many",
        "implementing",
        "[demo::tests::fails]",
        GOOD,
        "items:\n  - claim: \"fixture is shaped\"",
    );
    let before = repo.worktree();
    let run = repo.run(&["change", "complete", "shape"]).expect(1);
    assert_eq!(
        run.lines()[0],
        "refused: 1 obligation, 1 item without outcome, 1 finding once the record is removed"
    );
    assert!(run.stdout.contains("item-without-outcome"));
    assert!(!run.stdout.contains("demo::tests::fails"));
    assert!(run.stdout.contains("[no-open-record]"));
    assert_eq!(repo.worktree(), before);

    let machine = repo
        .run(&["--format", "turtle", "change", "complete", "shape"])
        .expect(1);
    let rdf = Rdf::of(&machine);
    let refused = rdf.rows("Refused");
    assert_eq!(refused.len(), 1);
    assert_eq!(refused[0].number("failures"), 3);
    assert_eq!(
        refused[0].required("message"),
        "refused: 1 obligation, 1 item without outcome, 1 finding once the record is removed"
    );
    let refusals = rdf.rows("Refusal");
    let summary: Vec<(String, Option<usize>, String, bool)> = refusals
        .iter()
        .map(|row| {
            (
                row.required("condition").into(),
                row.text("number").map(|_| row.number("number")),
                row.required("source").into(),
                row.boolean("repeated"),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            (
                "obligations".to_string(),
                Some(1),
                "item-without-outcome".to_string(),
                false
            ),
            (
                "outcomes".to_string(),
                Some(2),
                "item-without-outcome".to_string(),
                true
            ),
            (
                "after-removal".to_string(),
                None,
                "no-open-record".to_string(),
                false
            ),
        ]
    );
    // The lines the text form prints are those that no other line repeats.
    let printed: Vec<&str> = refusals
        .iter()
        .filter(|row| !row.boolean("repeated"))
        .map(|row| row.required("message"))
        .collect();
    assert_eq!(&run.lines()[1..], printed.as_slice());
}

#[test]
fn a_failing_example_stands_under_conditions_one_and_five() {
    let repo = shaped(
        "complete-example",
        "delivered",
        PASSES,
        "[demo::example_crate::exiting]",
        DONE,
    );
    let run = repo.run(&["change", "complete", "shape"]).expect(1);
    assert_eq!(
        run.lines()[0],
        "refused: 1 obligation, 1 bound example failing"
    );
    assert!(
        run.lines()[1].starts_with("cap:fixture  examples  example-failing  "),
        "{run:#?}"
    );
    assert_eq!(run.lines().len(), 2, "{run:#?}");
    let machine = repo
        .run(&["--format", "turtle", "change", "complete", "shape"])
        .expect(1);
    let rdf = Rdf::of(&machine);
    let sources = rdf
        .rows("Refusal")
        .iter()
        .map(|r| (r.number("number"), r.required("source").to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(
        sources,
        [
            (1, "example-failing".to_owned()),
            (5, "bound-example".to_owned())
        ]
    );
    assert!(repo.root().join(RECORD).is_file());
    // Once the example builds and runs the change completes.
    repo.put("examples/exiting.rs", support::GOOD_EXAMPLE);
    repo.run(&["change", "complete", "shape"]).expect(0);
}

#[test]
fn a_failing_bound_test_is_condition_five_and_the_record_stays() {
    let repo = shaped(
        "complete-test",
        "delivered",
        "[demo::tests::fails]",
        GOOD,
        DONE,
    );
    let before = repo.worktree();
    let run = repo.run(&["change", "complete", "shape"]).expect(1);
    assert_eq!(run.lines()[0], "refused: 1 bound test failing");
    assert!(
        run.lines()[1].starts_with("fail cap:fixture tests `demo::tests::fails`: "),
        "{run:#?}"
    );
    assert_eq!(run.lines().len(), 2);
    assert_eq!(repo.worktree(), before);
    let machine = repo
        .run(&["--format", "turtle", "change", "complete", "shape"])
        .expect(1);
    let rdf = Rdf::of(&machine);
    let refusal = rdf.rows("Refusal")[0];
    assert_eq!(refusal.required("condition"), "bound-runs");
    assert_eq!(refusal.number("number"), 5);
    assert_eq!(refusal.required("source"), "bound-test");
    // The neighbour: repair the same test and refresh its source assessment.
    repo.edit(
        "src/lib.rs",
        "panic!(\"this test fails\");",
        "assert_eq!(super::add(1, 2), 3);",
    );
    repo.assess("shape");
    repo.sync();
    repo.run(&["change", "complete", "shape"]).expect(0);
}

#[test]
fn a_stale_generated_block_is_refused_until_the_sync_is_run() {
    let repo = shaped("complete-stale", "implementing", PASSES, GOOD, DONE);
    // The catalogue lists the capability under the state it is in.
    repo.edit(CAP, "state: implementing", "state: delivered");
    let run = repo.run(&["change", "complete", "shape"]).expect(1);
    assert!(
        run.lines()[0].starts_with("refused: 1 obligation, 1 stale generated block"),
        "{run:#?}"
    );
    assert!(run.stdout.contains("generated-stale"), "{run:#?}");
    assert!(repo.root().join(RECORD).is_file());
    repo.run(&["docs", "sync"]).expect(0);
    repo.run(&["change", "complete", "shape"]).expect(0);
}

#[test]
fn what_stops_it_looking_is_exit_two_and_the_record_stays() {
    let repo = ready("complete-two");
    let before = repo.worktree();

    let unknown = repo.both(&["change", "complete", "nowhere"]).expect(2);
    assert_eq!(unknown.stdout, "");
    assert!(
        unknown.stderr.contains("no change record `nowhere`"),
        "{unknown:#?}"
    );

    // An outcome that is not one: the record does not load.
    repo.edit(RECORD, "outcome: implemented", "outcome: bogus");
    let bad = repo.both(&["change", "complete", "shape"]).expect(2);
    assert_eq!(bad.stdout, "");
    assert!(bad.stderr.contains("did not load"), "{bad:#?}");
    repo.edit(RECORD, "outcome: bogus", "outcome: implemented");
    assert_eq!(repo.worktree(), before);

    // No default branch to compare with.
    repo.git(&["symbolic-ref", "--delete", "refs/remotes/origin/HEAD"]);
    let headless = repo.both(&["change", "complete", "shape"]).expect(2);
    assert_eq!(headless.stdout, "");
    assert!(
        headless
            .stderr
            .contains("git remote set-head origin --auto"),
        "{headless:#?}"
    );
    assert!(repo.root().join(RECORD).is_file());
    assert_eq!(repo.worktree(), before);
}

#[test]
fn quiet_hides_a_completion_and_keeps_a_refusal() {
    let refused = shaped("complete-quiet-refused", "implementing", PASSES, GOOD, DONE);
    let loud = refused.run(&["change", "complete", "shape"]).expect(1);
    let quiet = refused
        .both(&["-q", "change", "complete", "shape"])
        .expect(1);
    assert_eq!(quiet.stdout, loud.stdout);

    let done = ready("complete-quiet-done");
    let run = done
        .run(&["--quiet", "change", "complete", "shape"])
        .expect(0);
    assert_eq!((run.stdout.as_str(), run.stderr.as_str()), ("", ""));
    assert!(!done.root().join(RECORD).exists());
}

#[test]
fn rdf_for_a_completion_is_a_completed_object_and_a_rewrote_object_for_each_file() {
    let repo = ready("complete-rdf");
    let run = repo
        .run(&["--format", "turtle", "change", "complete", "shape"])
        .expect(0);
    let rdf = Rdf::of(&run);
    assert_eq!(rdf.result_count(), 2);
    assert_eq!(rdf.rows("Completed")[0].required("path"), RECORD);
    assert_eq!(rdf.rows("Completed")[0].number("ordinal"), 0);
    assert_eq!(rdf.rows("Rewritten")[0].required("path"), CAP);
    assert_eq!(rdf.rows("Rewritten")[0].number("ordinal"), 1);
}

/// The contract settled and not assessed owes its assessment alone; the
/// assessor then records it, through the library.
fn assess_the_contract(repo: &Repo) {
    let unassessed = repo.run(&["change", "reconcile", "shape"]).expect(1);
    assert_eq!(
        unassessed.lines(),
        [
            "cap:fixture  contract  assessment-missing  an assessment of the section recorded with assess (`cap:fixture#contract` has no assessment `assessment:fixture-contract`)"
        ],
        "{unassessed:#?}"
    );
    assert_eq!(
        repo.assess("shape"),
        ["work/assessments/fixture-contract.md"]
    );
}

#[test]
fn from_new_to_a_completed_change_through_the_binary() {
    let repo = Repo::real("complete-flow");
    support::completion::proof_targets(&repo, PASSES, GOOD);
    repo.put(
        CAP,
        &capability(
            "fixture",
            "delivered",
            ["[demo::add]", PASSES, GOOD],
            "[]",
            &[("Verification", BLOCKS)],
        )
        .replacen("state: delivered", "state: implementing", 1),
    );
    repo.sync();
    repo.checkout();

    // Open the change; the output says what to do, and a record is one file.
    repo.assert_capability("fixture", "implementing");
    let before = repo.worktree();
    let created = repo
        .run(&[
            "change",
            "new",
            "shape",
            "--capability",
            "cap:fixture",
            "--copyright",
            "2026 Example Org <dev@example.org>",
            "--licence",
            "CC-BY-4.0",
        ])
        .expect(0);
    assert!(
        created
            .stdout
            .starts_with("created work/changes/shape.md\n")
    );
    assert!(created.stdout.contains("no change of state is needed"));
    assert_eq!(changed(&before, &repo.worktree()), [RECORD]);
    repo.assert_capability("fixture", "implementing");
    assert!(repo.read(RECORD).starts_with(&format!(
        "---\n# {}2026 Example Org <dev@example.org>\n# {}CC-BY-4.0\ntype: Change\n",
        concat!("SPDX-", "FileCopyrightText: "),
        concat!("SPDX-", "License-Identifier: "),
    )));
    // The output names two steps and nothing else is done here: stage the
    // record, then sync. Each step is read from the printed text.
    let steps: Vec<&str> = created
        .stdout
        .lines()
        .skip_while(|line| !line.starts_with("The record makes generated blocks stale."))
        .skip(1)
        .collect();
    assert_eq!(
        steps,
        ["  git add work/changes/shape.md", "  docs sync"],
        "{created:#?}"
    );
    repo.run(&["check"]).expect(1);
    let staged: Vec<&str> = steps[0].trim().split(' ').skip(1).collect();
    repo.git(&staged);
    let words: Vec<&str> = steps[1].trim().split(' ').collect();
    repo.run(&words).expect(0);
    repo.run(&["check"]).expect(0);

    // Edit: the proposed text goes into the contract as a PROPOSED block.
    repo.edit(
        CAP,
        "Text for Contract of fixture.",
        "Text for Contract of fixture.\n\n> [!PROPOSED change:shape]\n> Fixture does it.\n\n",
    );
    repo.run(&["check"]).expect(0);
    let owed = repo.run(&["change", "reconcile", "shape"]).expect(1);
    assert_eq!(owed.lines().len(), 2, "{owed:#?}");
    assert!(owed.stdout.contains("  unresolved-proposal  "), "{owed:#?}");
    assert!(owed.stdout.contains("  assessment-missing  "), "{owed:#?}");
    let refused = repo.run(&["change", "complete", "shape"]).expect(1);
    assert!(refused.stdout.starts_with("refused: "), "{refused:#?}");
    repo.assert_capability("fixture", "implementing");

    // The block becomes the contract, and once the contract is assessed
    // nothing is owed.
    repo.edit(
        CAP,
        "\n> [!PROPOSED change:shape]\n> Fixture does it.\n",
        "Fixture does it.\n",
    );
    assess_the_contract(&repo);
    let empty = repo.both(&["change", "reconcile", "shape"]).expect(0);
    assert_eq!(empty.stdout, "");
    repo.assert_capability("fixture", "implementing");

    // The author sets the state; the catalogue then needs a sync, and the gate
    // says so until it is run.
    repo.edit(CAP, "state: implementing", "state: delivered");
    let stale = repo.run(&["change", "complete", "shape"]).expect(1);
    assert!(stale.stdout.contains("generated-stale"), "{stale:#?}");
    repo.run(&["docs", "sync"]).expect(0);
    repo.both(&["change", "reconcile", "shape"]).expect(0);
    repo.run(&["verify", "shape"]).expect(0);
    repo.assert_capability("fixture", "delivered");

    let before = repo.worktree();
    let done = repo.run(&["change", "complete", "shape"]).expect(0);
    assert_eq!(
        done.stdout,
        "completed: removed work/changes/shape.md\nrewrote docs/capabilities/fixture.md\n"
    );
    assert!(!repo.root().join(RECORD).exists());
    assert_eq!(changed(&before, &repo.worktree()), [CAP, RECORD]);
    let check = repo.run(&["check"]).expect(0);
    assert_eq!(check.stdout, "");
    repo.assert_capability("fixture", "delivered");
}

/// The complete policy refuses an underexplained section without writing;
/// the neighboring complete document succeeds through the same command.
#[test]
fn the_default_completion_refuses_an_underexplained_section_and_accepts_its_neighbor() {
    ready("complete-full-neighbor")
        .run(&["change", "complete", "shape"])
        .expect(0);
    let repo = ready("complete-strict");
    repo.edit(CAP,
        "Text for Examples of fixture. This demonstration describes the same small package and its documented behavior so readers can check the result against its executable fixtures.",
        "A few words.");
    let owed = repo.run(&["change", "reconcile", "shape"]);
    assert_eq!(repo.run(&["change", "reconcile", "shape"]), owed);
    let before = repo.worktree();
    let run = repo.run(&["change", "complete", "shape"]).expect(1);
    assert!(
        run.stdout.starts_with("refused: ") && run.stdout.contains("[strict-section-thin]"),
        "{run:#?}"
    );
    assert_eq!(repo.worktree(), before, "a refusal writes nothing");
    let repeated = repo.run(&["change", "complete", "shape"]).expect(1);
    assert_eq!(repeated, run);
}
