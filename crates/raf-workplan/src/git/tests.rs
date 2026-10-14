// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The git questions against scripted answers: the version text, the exact argv
//! of each question, the shape of each answer and every refusal. The real
//! `git` is exercised by `tests/git.rs`.

use raf_okf::MAX_TEXT_BYTES;

use super::{
    GitError, GitVersion, blob_at, default_branch_ref, merge_base, require_version, resolve_commit,
    tracked_paths,
};
use crate::exec::{
    Commit, ExecError, FakeRunner, Limits, Output, Program, RefName, SourcePath, Stream,
};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER: &str = "fedcba9876543210fedcba9876543210fedcba98";

fn said(text: &str) -> Output {
    Output::new(Some(0), text.as_bytes().to_vec(), Vec::new())
}

fn failed(code: i32, stderr: &str) -> Output {
    Output::new(Some(code), Vec::new(), stderr.as_bytes().to_vec())
}

fn commit(text: &str) -> Commit {
    Commit::new(text).expect("a commit id")
}

fn reference(text: &str) -> RefName {
    RefName::new(text).expect("a reference name")
}

fn path(text: &str) -> SourcePath {
    SourcePath::new(text).expect("a source path")
}

// ------------------------------------------------------------------ version

#[test]
fn a_version_is_read_from_the_text_git_prints() {
    let table: [(&str, (u32, u32, u32)); 9] = [
        ("git version 2.45.1\n", (2, 45, 1)),
        ("git version 2.45.1", (2, 45, 1)),
        ("git version 2.45.1.windows.1\n", (2, 45, 1)),
        ("git version 2.39.5 (Apple Git-154)\n", (2, 39, 5)),
        ("git version 2.30.0\n", (2, 30, 0)),
        ("git version 2.30\n", (2, 30, 0)),
        ("git version 2.46.0-rc1\n", (2, 46, 0)),
        ("git version 2.47.0.vfs.0.1\n", (2, 47, 0)),
        ("git version 3.0.0\nsecond line\n", (3, 0, 0)),
    ];
    for (text, (major, minor, patch)) in table {
        let version = GitVersion::parse(text).unwrap_or_else(|error| panic!("{text:?}: {error}"));
        assert_eq!(
            (version.major(), version.minor(), version.patch()),
            (major, minor, patch),
            "{text:?}"
        );
        assert_eq!(version.to_string(), format!("{major}.{minor}.{patch}"));
    }
}

#[test]
fn a_text_that_is_not_a_version_is_refused() {
    for text in [
        "",
        "\n",
        "git",
        "git version",
        "git version ",
        "git version x.y",
        "git version 2",
        "git version 2.",
        "git version .30",
        "git version 2.x",
        "git version 2.-3",
        "git version -2.30",
        "git version 2.30x",
        "git version 99999999999.1",
        "Git version 2.45.1",
        "version 2.45.1",
        "2.45.1",
        "hub version 2.45.1",
        "\ngit version 2.45.1",
    ] {
        let error = GitVersion::parse(text).expect_err(text);
        assert!(
            matches!(error, GitError::VersionUnreadable { .. }),
            "{text:?}: {error:?}"
        );
    }
}

#[test]
fn versions_order_by_number_and_not_by_text() {
    let version = |text: &str| GitVersion::parse(text).expect("a version");
    assert!(version("git version 2.9.0") < GitVersion::MINIMUM);
    assert!(version("git version 2.29.9") < GitVersion::MINIMUM);
    assert!(version("git version 2.30.0") >= GitVersion::MINIMUM);
    assert!(version("git version 2.100.0") > version("git version 2.30.0"));
    assert!(version("git version 3.0.0") > version("git version 2.99.9"));
    assert!(version("git version 2.45.2") > version("git version 2.45.1"));
}

fn git_version(text: &str) -> FakeRunner {
    FakeRunner::new().on(Program::Git, &["--version"], Ok(said(text)))
}

#[test]
fn git_2_29_is_refused_and_2_30_and_2_45_are_accepted() {
    for older in [
        "git version 2.29.9\n",
        "git version 2.29.0\n",
        "git version 2.20.1\n",
        "git version 1.9.5\n",
        "git version 2.29.2.windows.1\n",
        "git version 2.29.3 (Apple Git-130)\n",
    ] {
        let error = require_version(&git_version(older)).expect_err(older);
        assert!(
            matches!(&error, GitError::VersionTooOld { found } if *found < GitVersion::MINIMUM),
            "{older:?}: {error:?}"
        );
        let message = error.to_string();
        assert!(message.contains("2.30.0"), "{message}");
        assert!(message.contains("--end-of-options"), "{message}");
    }
    for newer in [
        "git version 2.30.0\n",
        "git version 2.30.1\n",
        "git version 2.45.1\n",
        "git version 2.45.1.windows.1\n",
        "git version 2.39.5 (Apple Git-154)\n",
        "git version 3.1.0\n",
    ] {
        let version =
            require_version(&git_version(newer)).unwrap_or_else(|e| panic!("{newer:?}: {e}"));
        assert!(version >= GitVersion::MINIMUM, "{newer:?}");
    }
}

#[test]
fn the_version_question_is_exactly_git_version_in_the_c_locale() {
    let runner = git_version("git version 2.45.1\n");
    require_version(&runner).expect("a supported git");
    let calls = runner.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].program(), Program::Git);
    assert_eq!(calls[0].args(), ["--version"]);
    assert_eq!(calls[0].limits(), &Limits::query());
    assert!(calls[0].limits().c_locale());
}

#[test]
fn a_version_that_cannot_be_asked_or_read_is_an_error() {
    // Unreadable text, a failing exit, invalid UTF-8, a process that cannot run.
    let unreadable = require_version(&git_version("not git\n")).expect_err("unreadable");
    assert!(matches!(unreadable, GitError::VersionUnreadable { .. }));
    assert!(unreadable.to_string().contains("2.30.0"));

    let failing = FakeRunner::new().on(Program::Git, &["--version"], Ok(failed(127, "boom\n")));
    let error = require_version(&failing).expect_err("a failing exit");
    assert!(matches!(&error, GitError::Failed { code: Some(127), stderr, .. } if stderr == "boom"));

    let bytes = FakeRunner::new().on(
        Program::Git,
        &["--version"],
        Ok(Output::new(Some(0), vec![0xff, 0xfe], Vec::new())),
    );
    assert!(matches!(
        require_version(&bytes),
        Err(GitError::VersionUnreadable { .. })
    ));

    let unscripted = require_version(&FakeRunner::new()).expect_err("nothing scripted");
    assert!(matches!(
        unscripted,
        GitError::Exec(ExecError::Unscripted { .. })
    ));
}

// ------------------------------------------------------------ tracked paths

const INSIDE: [&str; 2] = ["rev-parse", "--is-inside-work-tree"];
const LIST: [&str; 2] = ["ls-files", "-z"];

#[test]
fn the_tracked_paths_are_the_nul_separated_listing_of_the_index() {
    let runner = FakeRunner::new()
        .on(Program::Git, &INSIDE, Ok(said("true\n")))
        .on(
            Program::Git,
            &LIST,
            Ok(said("Cargo.toml\0docs/a b.md\0src/lib.rs\0")),
        );
    let paths = tracked_paths(&runner)
        .expect("a listing")
        .expect("inside a work tree");
    let names: Vec<&str> = paths.iter().map(raf_source::RelPath::as_str).collect();
    assert_eq!(names, ["Cargo.toml", "docs/a b.md", "src/lib.rs"]);
    let calls = runner.calls();
    assert_eq!(calls[0].args(), INSIDE);
    assert_eq!(calls[1].args(), LIST);
    for call in &calls {
        assert_eq!(call.program(), Program::Git);
        assert!(call.limits().c_locale(), "{}", call.command());
    }
}

#[test]
fn a_directory_outside_a_work_tree_has_no_listing() {
    // `git` answers `false` inside a `.git` directory.
    let inside_git_dir = FakeRunner::new().on(Program::Git, &INSIDE, Ok(said("false\n")));
    assert_eq!(tracked_paths(&inside_git_dir), Ok(None));
    assert_eq!(inside_git_dir.calls().len(), 1, "no listing is asked for");

    // And fails with `not a git repository` outside any repository.
    let outside = FakeRunner::new().on(
        Program::Git,
        &INSIDE,
        Ok(failed(
            128,
            "fatal: not a git repository (or any of the parent directories): .git\n",
        )),
    );
    assert_eq!(tracked_paths(&outside), Ok(None));
}

#[test]
fn every_other_failure_of_the_listing_is_an_error_and_not_a_walk() {
    // A neighbour that is not "not a git repository" fails.
    let other = FakeRunner::new().on(
        Program::Git,
        &INSIDE,
        Ok(failed(
            128,
            "fatal: detected dubious ownership in repository\n",
        )),
    );
    let error = tracked_paths(&other).expect_err("another failure");
    assert!(
        matches!(
            &error,
            GitError::Failed {
                code: Some(128),
                ..
            }
        ),
        "{error:?}"
    );
    assert!(error.to_string().contains("dubious ownership"), "{error}");

    let odd = FakeRunner::new().on(Program::Git, &INSIDE, Ok(said("maybe\n")));
    let error = tracked_paths(&odd).expect_err("an answer of neither word");
    assert!(matches!(error, GitError::Answer { .. }), "{error:?}");

    let broken_index = FakeRunner::new()
        .on(Program::Git, &INSIDE, Ok(said("true\n")))
        .on(
            Program::Git,
            &LIST,
            Ok(failed(128, "fatal: index file corrupt\n")),
        );
    let error = tracked_paths(&broken_index).expect_err("an unreadable index");
    assert!(error.to_string().contains("git ls-files"), "{error}");
    assert!(error.to_string().contains("index file corrupt"), "{error}");

    let cannot_start = FakeRunner::new();
    assert!(matches!(
        tracked_paths(&cannot_start),
        Err(GitError::Exec(ExecError::Unscripted { .. }))
    ));
}

#[test]
fn a_tracked_path_must_be_utf8_and_a_relative_path_of_the_tree() {
    let listing = |bytes: &[u8]| {
        FakeRunner::new()
            .on(Program::Git, &INSIDE, Ok(said("true\n")))
            .on(
                Program::Git,
                &LIST,
                Ok(Output::new(Some(0), bytes.to_vec(), Vec::new())),
            )
    };
    let error = tracked_paths(&listing(b"ok.md\0bad\xff.md\0")).expect_err("not UTF-8");
    assert!(matches!(error, GitError::NotUtf8 { .. }), "{error:?}");
    for bad in ["/etc/passwd", "../outside", "a/../../b"] {
        let error = tracked_paths(&listing(format!("{bad}\0").as_bytes())).expect_err(bad);
        assert!(
            matches!(&error, GitError::NotAPath { path, .. } if path == bad),
            "{bad}: {error:?}"
        );
    }
    // A neighbour: an empty listing is the empty set of paths.
    assert_eq!(tracked_paths(&listing(b"")), Ok(Some(Vec::new())));
}

// ------------------------------------------------------------ default branch

const DEFAULT: [&str; 2] = ["symbolic-ref", "refs/remotes/origin/HEAD"];

#[test]
fn the_default_branch_is_the_full_reference_name() {
    let runner = FakeRunner::new().on(
        Program::Git,
        &DEFAULT,
        Ok(said("refs/remotes/origin/main\n")),
    );
    let name = default_branch_ref(&runner).expect("a default branch");
    assert_eq!(name.as_str(), "refs/remotes/origin/main");
    let calls = runner.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].args(), DEFAULT);
    assert!(calls[0].limits().c_locale());
}

#[test]
fn a_missing_default_branch_is_an_error_that_names_the_fix() {
    for stderr in [
        "fatal: ref refs/remotes/origin/HEAD is not a symbolic ref\n",
        "fatal: No such ref: refs/remotes/origin/HEAD\n",
        "",
    ] {
        let runner = FakeRunner::new().on(Program::Git, &DEFAULT, Ok(failed(128, stderr)));
        let error = default_branch_ref(&runner).expect_err("no default branch");
        assert!(
            matches!(error, GitError::NoDefaultBranch { .. }),
            "{error:?}"
        );
        let message = error.to_string();
        assert!(
            message.contains("git remote set-head origin --auto"),
            "{message}"
        );
        assert!(message.contains("origin"), "{message}");
        assert!(message.contains(stderr.trim()), "{message}");
    }
}

#[test]
fn an_answer_that_is_not_a_full_reference_name_never_becomes_one() {
    for answer in [
        "--output=x\n",
        "main\n",
        "origin/main\n",
        "refs/\n",
        "refs/remotes/origin/../x\n",
        "refs/remotes/origin/main x\n",
        "refs/remotes/origin/main\nrefs/remotes/origin/other\n",
        "refs/remotes/origin/a^b\n",
        "-refs/remotes/origin/main\n",
        "",
    ] {
        let runner = FakeRunner::new().on(Program::Git, &DEFAULT, Ok(said(answer)));
        let error = default_branch_ref(&runner).expect_err(answer);
        assert!(
            matches!(error, GitError::Answer { .. }),
            "{answer:?}: {error:?}"
        );
    }
    // A branch named like an option is a valid reference name once it is
    // full: nothing but the `refs/` prefix keeps it from being an option.
    let runner = FakeRunner::new().on(
        Program::Git,
        &DEFAULT,
        Ok(said("refs/remotes/origin/--output=x\n")),
    );
    assert_eq!(
        default_branch_ref(&runner).expect("a valid name").as_str(),
        "refs/remotes/origin/--output=x"
    );
}

// ----------------------------------------------------------- resolve a ref

#[test]
fn a_reference_is_resolved_to_a_commit_after_the_end_of_options() {
    let name = reference("refs/remotes/origin/main");
    let runner = FakeRunner::new().on(
        Program::Git,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            "refs/remotes/origin/main^{commit}",
        ],
        Ok(said(&format!("{COMMIT}\n"))),
    );
    assert_eq!(resolve_commit(&runner, &name), Ok(commit(COMMIT)));
    assert_eq!(runner.calls().len(), 1);
}

#[test]
fn a_reference_named_like_an_option_is_one_argument_after_the_end_of_options() {
    let name = reference("refs/remotes/origin/--output=x");
    let runner = FakeRunner::new().on(
        Program::Git,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            "refs/remotes/origin/--output=x^{commit}",
        ],
        Ok(said(&format!("{COMMIT}\n"))),
    );
    resolve_commit(&runner, &name).expect("resolved");
    let call = &runner.calls()[0];
    let marker = call
        .args()
        .iter()
        .position(|arg| arg == "--end-of-options")
        .expect("the marker");
    for (index, arg) in call.args().iter().enumerate() {
        if index > marker {
            assert!(arg.starts_with("refs/"), "{arg}");
        }
    }
}

#[test]
fn a_reference_that_resolves_to_nothing_or_to_text_is_an_error() {
    let name = reference("refs/remotes/origin/main");
    let argv = [
        "rev-parse",
        "--verify",
        "--end-of-options",
        "refs/remotes/origin/main^{commit}",
    ];
    let gone = FakeRunner::new().on(
        Program::Git,
        &argv,
        Ok(failed(128, "fatal: Needed a single revision\n")),
    );
    let error = resolve_commit(&gone, &name).expect_err("does not resolve");
    assert!(
        matches!(
            &error,
            GitError::Failed {
                code: Some(128),
                ..
            }
        ),
        "{error:?}"
    );
    assert!(error.to_string().contains("Needed a single revision"));
    for answer in [
        "",
        "HEAD\n",
        "0123\n",
        "0123456789ABCDEF0123456789ABCDEF01234567\n",
    ] {
        let runner = FakeRunner::new().on(Program::Git, &argv, Ok(said(answer)));
        assert!(
            matches!(resolve_commit(&runner, &name), Err(GitError::Answer { .. })),
            "{answer:?}"
        );
    }
    // Both hash lengths are commit ids.
    let long = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let runner = FakeRunner::new().on(Program::Git, &argv, Ok(said(&format!("{long}\n"))));
    assert_eq!(resolve_commit(&runner, &name), Ok(commit(long)));
}

// -------------------------------------------------------------- merge base

#[test]
fn the_merge_base_is_asked_of_head_and_the_commit() {
    let default = commit(COMMIT);
    let runner = FakeRunner::new().on(
        Program::Git,
        &["merge-base", "HEAD", COMMIT],
        Ok(said(&format!("{OTHER}\n"))),
    );
    assert_eq!(merge_base(&runner, &default), Ok(commit(OTHER)));
    assert_eq!(runner.calls().len(), 1);
}

#[test]
fn histories_without_a_common_ancestor_and_odd_answers_are_errors() {
    let default = commit(COMMIT);
    let argv = ["merge-base", "HEAD", COMMIT];
    let none = FakeRunner::new().on(Program::Git, &argv, Ok(failed(1, "")));
    let error = merge_base(&none, &default).expect_err("no merge base");
    assert!(matches!(&error, GitError::NoMergeBase { default: d } if *d == default));
    assert!(error.to_string().contains("no common ancestor"), "{error}");

    let broken = FakeRunner::new().on(Program::Git, &argv, Ok(failed(128, "fatal: bad object\n")));
    assert!(matches!(
        merge_base(&broken, &default),
        Err(GitError::Failed {
            code: Some(128),
            ..
        })
    ));
    let two = FakeRunner::new().on(
        Program::Git,
        &argv,
        Ok(said(&format!("{COMMIT}\n{OTHER}\n"))),
    );
    assert!(matches!(
        merge_base(&two, &default),
        Err(GitError::Answer { .. })
    ));
}

// ----------------------------------------------------------------- blob_at

fn blob_argv() -> [String; 3] {
    [
        "cat-file".to_string(),
        "blob".to_string(),
        format!("{COMMIT}:docs/capabilities/workplan-cli.md"),
    ]
}

fn blob_runner(answer: Result<Output, ExecError>) -> FakeRunner {
    let argv = blob_argv();
    let words: Vec<&str> = argv.iter().map(String::as_str).collect();
    FakeRunner::new().on(Program::Git, &words, answer)
}

#[test]
fn a_blob_is_read_from_the_merge_base_in_one_argument() {
    let runner = blob_runner(Ok(said("# Title\n")));
    let bytes = blob_at(
        &runner,
        &commit(COMMIT),
        &path("docs/capabilities/workplan-cli.md"),
    )
    .expect("a blob");
    assert_eq!(bytes.as_deref(), Some(b"# Title\n".as_slice()));
    let call = &runner.calls()[0];
    assert_eq!(call.args(), blob_argv());
    assert_eq!(call.limits().max_output(), MAX_TEXT_BYTES);
    assert!(call.limits().c_locale());
}

#[test]
fn a_blob_over_one_mebibyte_is_an_error_and_never_cut() {
    let at = path("docs/capabilities/workplan-cli.md");
    let exact = blob_runner(Ok(Output::new(
        Some(0),
        vec![b'a'; MAX_TEXT_BYTES],
        Vec::new(),
    )));
    let bytes = blob_at(&exact, &commit(COMMIT), &at)
        .expect("exactly the bound passes")
        .expect("a blob");
    assert_eq!(bytes.len(), MAX_TEXT_BYTES);

    let over = blob_runner(Ok(Output::new(
        Some(0),
        vec![b'a'; MAX_TEXT_BYTES + 1],
        Vec::new(),
    )));
    let error = blob_at(&over, &commit(COMMIT), &at).expect_err("one byte over");
    assert!(
        matches!(&error, GitError::BlobTooLarge { limit, path } if *limit == MAX_TEXT_BYTES && *path == at),
        "{error:?}"
    );
    assert!(error.to_string().contains("1048576"), "{error}");

    // The same error from the real runner's own report of an overflow.
    let overflow = blob_runner(Err(ExecError::OutputTooLarge {
        command: "git cat-file".to_string(),
        stream: Stream::Stdout,
        limit: MAX_TEXT_BYTES,
    }));
    assert!(matches!(
        blob_at(&overflow, &commit(COMMIT), &at),
        Err(GitError::BlobTooLarge { .. })
    ));
    // An overflow of standard error is not a large blob.
    let noisy = blob_runner(Err(ExecError::OutputTooLarge {
        command: "git cat-file".to_string(),
        stream: Stream::Stderr,
        limit: MAX_TEXT_BYTES,
    }));
    assert!(matches!(
        blob_at(&noisy, &commit(COMMIT), &at),
        Err(GitError::Exec(ExecError::OutputTooLarge { .. }))
    ));
}

#[test]
fn a_path_missing_at_the_merge_base_is_none_and_other_failures_are_errors() {
    let at = path("docs/capabilities/workplan-cli.md");
    for stderr in [
        format!("fatal: path 'docs/capabilities/workplan-cli.md' does not exist in '{COMMIT}'\n"),
        format!(
            "fatal: path 'docs/capabilities/workplan-cli.md' exists on disk, but not in '{COMMIT}'\n"
        ),
    ] {
        let runner = blob_runner(Ok(failed(128, &stderr)));
        assert_eq!(blob_at(&runner, &commit(COMMIT), &at), Ok(None), "{stderr}");
    }
    for stderr in [
        format!("fatal: git cat-file {COMMIT}:docs/capabilities/workplan-cli.md: bad file\n"),
        "fatal: bad object\n".to_string(),
        String::new(),
    ] {
        let runner = blob_runner(Ok(failed(128, &stderr)));
        let error = blob_at(&runner, &commit(COMMIT), &at).expect_err(&stderr);
        assert!(
            matches!(error, GitError::Failed { .. }),
            "{stderr}: {error:?}"
        );
    }
}
