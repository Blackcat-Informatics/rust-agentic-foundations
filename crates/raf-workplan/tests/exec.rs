// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The process layer: the four patterns (and the five for Git) accept and
//! refuse the values they state, the invocations are the rows of the argv
//! tables, the in-process fake records and answers on every platform, and the
//! real runner starts a process with no shell, null standard input, a stated
//! environment and directory, a bound on its output and a wall-clock kill.
//!
//! The real runner is exercised through stub programs, small shell scripts
//! written to a scratch directory, on Unix; a smoke test starts the real `git`
//! and `cargo` everywhere.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use raf_workplan::exec::{
    Arg, BUILD_TIMEOUT, Commit, DEFAULT_OUTPUT_BOUND, DocFilter, EXAMPLE_TIMEOUT, ExampleName,
    ExecError, FakeRunner, Filter, Identifier, Invocation, Limits, LineRange, Output, Package,
    Pattern, PatternError, Programs, RefName, Runner, SourcePath, Stream, SystemRunner, Target,
    TargetName, TreePath,
};
use raf_workplan::exec::{Program, QUERY_TIMEOUT};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const COMMIT_256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn pkg() -> Package {
    Package::new("raf-demo").expect("a package name")
}

fn name(text: &str) -> TargetName {
    TargetName::new(text).expect("a target name")
}

fn filter(text: &str) -> Filter {
    Filter::new(text).expect("a test name")
}

// ---------------------------------------------------------------- patterns

/// Values each pattern accepts and refuses. Every refusal is paired with the
/// accepted neighbour it is nearest to.
struct Table {
    pattern: Pattern,
    accepted: &'static [&'static str],
    refused: &'static [&'static str],
}

const HOSTILE: &[&str] = &[
    "",
    "-Zunstable-options",
    "--exact",
    "--include-ignored",
    "-rf",
    "-",
    "a;b",
    "a b",
    " a",
    "a ",
    "a\tb",
    "a\nb",
    "a\0b",
    "$(touch x)",
    "`id`",
    "a|b",
    "a&&b",
    "a>b",
    "a\"b",
    "a'b",
    "a\\b",
    "a*",
    "é",
    "naïve",
    "名前",
    "a\u{202e}b",
    "a\u{ff1a}\u{ff1a}b",
];

const TABLES: [Table; 9] = [
    Table {
        pattern: Pattern::Name,
        accepted: &[
            "a",
            "_",
            "a1",
            "a_b",
            "A::b",
            "a::b::c",
            "_x::_y9",
            "tests::it_works",
        ],
        refused: &[
            "1a",
            "a-b",
            "a::",
            "::a",
            "a:::b",
            "a:b",
            "a::1",
            "a::b-c",
            "a.b",
            "a/b",
            "a<T>",
            "a<T>::get",
            "a::b ",
            "r#type!",
        ],
    },
    Table {
        pattern: Pattern::TargetName,
        accepted: &[
            "a", "1", "_", "_a", "9lives", "my-crate", "a-b-c", "a_b-c", "A-",
        ],
        refused: &[
            "-a",
            "-",
            "a.b",
            "a/b",
            "a::b",
            "a=b",
            "--package=x",
            "a b",
            "a+",
        ],
    },
    Table {
        pattern: Pattern::ItemPath,
        accepted: &[
            "a",
            "a::b",
            "Wrapper<T>::get",
            "Wrapper<T>",
            "Wrapper<>::get",
            "Wrapper<'a>::get",
            "Wrapper<&'a_T>::get",
            "Map<K,V>::insert",
            "a::Wrapper<a::B>::get",
            "a<b>::c<d>::e",
            "a<x:y>",
        ],
        refused: &[
            "<T>",
            "Wrapper<T",
            "Wrapper<T>>",
            "Wrapper<<T>>",
            "Wrapper<T> ::get",
            "Wrapper<T>get",
            "Wrapper<T>::",
            "Wrapper<a b>::get",
            "Wrapper<&'a T>::get",
            "Wrapper<T-U>::get",
            "Wrapper<T;U>::get",
            "Wrapper<T>::-x",
            "-Wrapper<T>",
            "a::b<",
            "a<b>c",
            "1a::b",
            "a::1b",
            "a- b",
        ],
    },
    Table {
        pattern: Pattern::SourcePath,
        accepted: &[
            "a",
            "_",
            "9",
            "src/lib.rs",
            "src/a_b/c-d.rs",
            "docs/capabilities/workplan-cli.md",
            "a.b/c.d",
            "a..b",
        ],
        refused: &[
            "-a",
            ".a",
            ".",
            "..",
            "../a",
            "a/..",
            "a/../b",
            "a/./b",
            "a//b",
            "/a",
            "a b",
            "a:b",
            "a\\b",
            "a?b",
            "a=b",
            "src/é.rs",
            "a/b/",
        ],
    },
    Table {
        pattern: Pattern::RefName,
        accepted: &[
            "refs/remotes/origin/main",
            "refs/remotes/origin/feature/x-1",
            "refs/remotes/origin/--output=x",
            "refs/remotes/origin/-Zunstable-options",
            "refs/remotes/origin/v1.2.3",
            "refs/heads/ünï",
        ],
        refused: &[
            "origin/main",
            "refs",
            "refs/",
            "refs//x",
            "refs/heads/",
            "refs/heads/a..b",
            "refs/heads/.hidden",
            "refs/heads/a.",
            "refs/heads/x.lock",
            "refs/heads/a b",
            "refs/heads/a~1",
            "refs/heads/a^",
            "refs/heads/a:b",
            "refs/heads/a?",
            "refs/heads/a*",
            "refs/heads/a[1]",
            "refs/heads/a\\b",
            "refs/heads/a@{1}",
            "refs/heads/a\nb",
            "refs/heads/a;b",
            "-refs/heads/a",
            "--end-of-options",
        ],
    },
    Table {
        pattern: Pattern::Commit,
        accepted: &[COMMIT, COMMIT_256],
        refused: &[
            "",
            "0123",
            "0123456789ABCDEF0123456789ABCDEF01234567",
            "0123456789abcdef0123456789abcdef0123456",
            "0123456789abcdef0123456789abcdef012345678",
            "0123456789abcdef0123456789abcdef0123456g",
            "HEAD",
            "-0123456789abcdef0123456789abcdef0123456",
            "0123456789abcdef0123456789abcdef01234567 ",
            "0123456789abcdef0123456789abcdef01234567:x",
        ],
    },
    Table {
        pattern: Pattern::TreePath,
        accepted: &[
            "a",
            "src/lib.rs",
            "-rf.rs",
            "--output=x.rs",
            "-",
            "src/-rf.rs",
            "a b.rs",
            "src/\u{e9}t\u{e9}.rs",
            "a:b",
            "a=b",
            "a..b",
            "a;b",
            ".hidden",
        ],
        refused: &[
            "", ".", "..", "../a", "a/..", "a/../b", "a/./b", "a//b", "a/", "/a", ":a", ":(glob)*",
            "a*", "a?", "a[1]", "a\\b", "a\nb", "a\rb", "a\tb", "a\0b", "a\u{1b}b", "a\u{7f}b",
            "a\u{85}b",
        ],
    },
    Table {
        pattern: Pattern::Identifier,
        accepted: &["a", "_", "_a", "gone", "Gone9", "snake_case"],
        refused: &[
            "", "1a", "-a", "a-b", "a::b", "a b", "a<T>", "r#type", "é", "-Sx", "a.b",
        ],
    },
    Table {
        pattern: Pattern::Date,
        accepted: &[
            "2026-01-01",
            "2026-12-31",
            "2028-02-29",
            "2000-02-29",
            "1970-01-01",
            "2099-12-31",
        ],
        refused: &[
            "",
            "-2026-01-01",
            "--all",
            "2026-1-01",
            "2026-01-1",
            "26-01-01",
            "2026/01/01",
            "2026-00-01",
            "2026-13-01",
            "2026-01-00",
            "2026-01-32",
            "2026-02-29",
            "1900-02-29",
            "2026-04-31",
            "2026-01-01 ",
            "2026-01-01T00:00",
            "yesterday",
            "2026-0a-01",
            "1969-12-31",
            "2100-01-01",
            "0001-01-01",
            "2999-01-01",
        ],
    },
];

#[test]
fn every_pattern_accepts_what_it_states() {
    for table in &TABLES {
        for text in table.accepted {
            assert!(
                table.pattern.matches(text),
                "{:?} must accept {text:?}",
                table.pattern
            );
        }
    }
}

#[test]
fn every_pattern_refuses_what_it_does_not_state() {
    for table in &TABLES {
        for text in table.refused {
            assert!(
                !table.pattern.matches(text),
                "{:?} must refuse {text:?}",
                table.pattern
            );
        }
    }
}

#[test]
fn every_pattern_refuses_hostile_values() {
    // The tree path is the one pattern that takes text that looks like an
    // option or holds a shell character: it is placed only after `--` or
    // after the colon of one `-L` argument (see the row tests below), and it
    // still refuses every hostile value that is not printable path text.
    for pattern in Pattern::ALL
        .into_iter()
        .filter(|pattern| *pattern != Pattern::TreePath)
    {
        for text in HOSTILE {
            assert!(
                !pattern.matches(text),
                "{pattern:?} must refuse the hostile value {text:?}"
            );
        }
    }
}

#[test]
fn a_tree_path_refuses_every_hostile_value_that_is_not_printable_path_text() {
    for text in HOSTILE {
        let printable = !text.is_empty()
            && !text
                .chars()
                .any(|c| c.is_control() || matches!(c, '\\' | '*'));
        assert_eq!(
            Pattern::TreePath.matches(text),
            printable,
            "the tree path and {text:?}"
        );
    }
}

#[test]
fn the_patterns_are_the_ones_the_documents_state() {
    assert_eq!(Pattern::TreePath.expression(), "[^/:][^/]*(/[^/]+)*");
    assert_eq!(Pattern::Identifier.expression(), "[A-Za-z_][A-Za-z0-9_]*");
    assert_eq!(
        Pattern::Date.expression(),
        "(19[7-9][0-9]|20[0-9][0-9])-[0-9]{2}-[0-9]{2}"
    );
    assert_eq!(
        Pattern::Name.expression(),
        "[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*"
    );
    assert_eq!(
        Pattern::TargetName.expression(),
        "[A-Za-z0-9_][A-Za-z0-9_-]*"
    );
    assert_eq!(
        Pattern::SourcePath.expression(),
        "[A-Za-z0-9_][A-Za-z0-9_./-]*"
    );
    assert!(
        Pattern::ItemPath
            .expression()
            .contains("<[A-Za-z0-9_,':&]*>")
    );
    // The table above covers every pattern.
    let covered: Vec<Pattern> = TABLES.iter().map(|table| table.pattern).collect();
    for pattern in Pattern::ALL {
        assert!(covered.contains(&pattern), "{pattern:?} has no table");
    }
}

#[test]
fn a_generic_list_holds_exactly_the_characters_of_the_pattern() {
    for allowed in ['a', 'Z', '0', '_', ',', '\'', ':', '&'] {
        let item = format!("W<{allowed}>::get");
        assert!(DocFilter::new(&item).is_ok(), "{item}");
    }
    for refused in [
        '-', ' ', ';', '<', '>', '(', '*', '\\', '"', 'é', '\n', '=', '.',
    ] {
        let item = format!("W<{refused}>::get");
        assert!(DocFilter::new(&item).is_err(), "{item:?}");
    }
}

#[test]
fn each_newtype_is_built_only_by_its_pattern() {
    assert_eq!(Package::new("raf-demo").unwrap().as_str(), "raf-demo");
    assert!(Package::new("-Zunstable-options").is_err());
    assert!(TargetName::new("--exact").is_err());
    assert!(ExampleName::new("chunked-1").is_ok());
    assert!(ExampleName::new("a b").is_err());
    assert!(Filter::new("tests::it_works").is_ok());
    assert!(Filter::new("--include-ignored").is_err());
    assert!(Filter::new("a;b").is_err());
    assert!(DocFilter::new("Wrapper<T>::get").is_ok());
    assert!(DocFilter::new("-x").is_err());
    assert!(SourcePath::new("src/lib.rs").is_ok());
    assert!(SourcePath::new("a/../b").is_err());
    assert!(RefName::new("refs/remotes/origin/main").is_ok());
    assert!(RefName::new("origin/main").is_err());
    assert!(Commit::new(COMMIT).is_ok());
    assert!(Commit::new("HEAD").is_err());
}

#[test]
fn a_refusal_names_the_pattern_and_shows_the_value_safely() {
    let error: PatternError = Filter::new("-Zunstable-options").unwrap_err();
    assert_eq!(error.pattern(), Pattern::Name);
    assert_eq!(error.value(), "-Zunstable-options");
    let text = error.to_string();
    assert!(text.contains("-Zunstable-options"), "{text}");
    assert!(text.contains(Pattern::Name.expression()), "{text}");
    assert!(text.contains("not passed to a process"), "{text}");

    // Control characters are escaped so the message cannot forge a line.
    let text = Package::new("a\nb\u{1b}[31m").unwrap_err().to_string();
    assert!(!text.contains('\n') && !text.contains('\u{1b}'), "{text:?}");

    // A long value is cut.
    let long = "-".repeat(500);
    let error = Package::new(&long).unwrap_err();
    assert!(error.value().chars().count() < 100, "{}", error.value());
    assert!(error.value().ends_with("..."));

    // A neighbour: the value that matches builds.
    assert!(Package::new("raf-demo").is_ok());
}

// ------------------------------------------------------------- argv tables

/// A label, the invocation, its program and its argv: the elements separated
/// by one space, and a space inside an element written as U+00A0 (see
/// [`words`]).
type Row = (&'static str, Invocation, Program, &'static str);

/// The argv elements of a row's text: split at each space, then each U+00A0
/// read as the space it stands for.
fn words(text: &str) -> Vec<String> {
    text.split(' ')
        .map(|word| word.replace('\u{a0}', " "))
        .collect()
}

/// The documented rows, with the placeholders filled and the program left off.
fn rows() -> Vec<Row> {
    let mut rows = cargo_rows();
    rows.extend(git_rows());
    rows
}

/// The rows of `cargo`.
fn cargo_rows() -> Vec<Row> {
    let package = pkg();
    let lib = Target::Lib;
    let bin = Target::Bin(name("my-bin"));
    let test = Target::Test(name("it-tests"));
    let case = filter("tests::it_works");
    let item = DocFilter::new("Wrapper<T>::get").unwrap();
    vec![
        (
            "test, list pass, library",
            Invocation::test_list(&package, &lib, &case),
            Program::Cargo,
            "test --frozen --package=raf-demo --lib -- --list --exact -- tests::it_works",
        ),
        (
            "test, list pass, binary",
            Invocation::test_list(&package, &bin, &case),
            Program::Cargo,
            "test --frozen --package=raf-demo --bin=my-bin -- --list --exact -- tests::it_works",
        ),
        (
            "test, run pass, integration test",
            Invocation::test_run(&package, &test, &case),
            Program::Cargo,
            "test --frozen --package=raf-demo --test=it-tests -- --exact -- tests::it_works",
        ),
        (
            "test, run pass, library",
            Invocation::test_run(&package, &lib, &case),
            Program::Cargo,
            "test --frozen --package=raf-demo --lib -- --exact -- tests::it_works",
        ),
        (
            "doc test, list pass",
            Invocation::doc_list(&package),
            Program::Cargo,
            "test --frozen --package=raf-demo --doc -- --list",
        ),
        (
            "doc test, run pass",
            Invocation::doc_run(&package, &item),
            Program::Cargo,
            "test --frozen --package=raf-demo --doc -- Wrapper<T>::get",
        ),
        (
            "example target",
            Invocation::example_run(&package, &ExampleName::new("chunked-1").unwrap()),
            Program::Cargo,
            "run --frozen --package=raf-demo --example=chunked-1",
        ),
    ]
}

/// The rows of `git`.
fn git_rows() -> Vec<Row> {
    let rev = Commit::new(COMMIT).unwrap();
    let reference = RefName::new("refs/remotes/origin/main").unwrap();
    let path = SourcePath::new("docs/capabilities/workplan-cli.md").unwrap();
    vec![
        (
            "git version",
            Invocation::git_version(),
            Program::Git,
            "--version",
        ),
        (
            "git work tree query",
            Invocation::git_inside_work_tree(),
            Program::Git,
            "rev-parse --is-inside-work-tree",
        ),
        (
            "git work tree prefix",
            Invocation::git_show_prefix(),
            Program::Git,
            "rev-parse --show-prefix",
        ),
        (
            "git tracked paths",
            Invocation::git_ls_files(),
            Program::Git,
            "ls-files -z",
        ),
        (
            "git default branch",
            Invocation::git_default_ref(),
            Program::Git,
            "symbolic-ref refs/remotes/origin/HEAD",
        ),
        (
            "git resolve",
            Invocation::git_resolve(&reference),
            Program::Git,
            "rev-parse --verify --end-of-options refs/remotes/origin/main^{commit}",
        ),
        (
            "git merge base",
            Invocation::git_merge_base(&rev),
            Program::Git,
            "merge-base HEAD 0123456789abcdef0123456789abcdef01234567",
        ),
        (
            "git blob",
            Invocation::git_blob(&rev, &path),
            Program::Git,
            "cat-file blob 0123456789abcdef0123456789abcdef01234567:docs/capabilities/workplan-cli.md",
        ),
        (
            "git tree",
            Invocation::git_tree(&rev),
            Program::Git,
            "ls-tree -r -z --full-tree 0123456789abcdef0123456789abcdef01234567",
        ),
    ]
    .into_iter()
    .chain(history_rows())
    .collect()
}

/// The rows of the history command.
fn history_rows() -> Vec<Row> {
    let dash = TreePath::new("-rf.rs").unwrap();
    let output = TreePath::new("--output=x.rs").unwrap();
    let lines = LineRange::new(3, 9).unwrap();
    let gone = Identifier::new("gone").unwrap();
    vec![
        (
            "git shallow query",
            Invocation::git_shallow(),
            Program::Git,
            "rev-parse --is-shallow-repository",
        ),
        (
            "git document log",
            Invocation::git_document_log(&dash),
            Program::Git,
            "--no-pager log --no-show-signature --follow --cc --root --format=%x1e%H%x1f%aI%x1f%s --name-only -- -rf.rs",
        ),
        (
            "git document log of a path named like an option",
            Invocation::git_document_log(&output),
            Program::Git,
            "--no-pager log --no-show-signature --follow --cc --root --format=%x1e%H%x1f%aI%x1f%s --name-only -- --output=x.rs",
        ),
        (
            "git symbol log",
            Invocation::git_symbol_log(lines, &dash),
            Program::Git,
            "--no-pager log --no-show-signature -L3,9:-rf.rs --no-patch --format=%x1e%H%x1f%aI%x1f%s --",
        ),
        (
            "git symbol log of a path named like an option",
            Invocation::git_symbol_log(lines, &output),
            Program::Git,
            "--no-pager log --no-show-signature -L3,9:--output=x.rs --no-patch --format=%x1e%H%x1f%aI%x1f%s --",
        ),
        (
            "git pickaxe",
            Invocation::git_pickaxe(&gone, &[TreePath::new("crates/a").unwrap()]),
            Program::Git,
            "--no-pager log --no-show-signature --root -Sgone -n 1 --format=%H%x1f%P%x1f%aI%x1f%s -- crates/a",
        ),
        (
            "git pickaxe at the top",
            Invocation::git_pickaxe(&gone, &[TreePath::top(), dash.clone()]),
            Program::Git,
            "--no-pager log --no-show-signature --root -Sgone -n 1 --format=%H%x1f%P%x1f%aI%x1f%s -- . -rf.rs",
        ),
    ]
}

#[test]
fn every_invocation_is_the_row_of_its_table() {
    let rows = rows();
    assert_eq!(rows.len(), 23);
    for (label, invocation, program, expected) in rows {
        assert_eq!(invocation.program(), program, "{label}");
        let expected = words(expected);
        assert_eq!(invocation.argv(), expected, "{label}");
        assert_eq!(
            invocation.to_string(),
            format!("{program} {}", expected.join(" ")),
            "{label}"
        );
    }
}

#[test]
fn a_test_filter_always_follows_a_separator_and_never_leads() {
    for (label, invocation, _, _) in rows() {
        let argv = invocation.argv();
        for (index, arg) in invocation.args().iter().enumerate() {
            if matches!(arg, Arg::Filter(_) | Arg::DocFilter(_)) {
                assert_eq!(argv[index - 1].as_str(), "--", "{label}: {argv:?}");
            }
        }
    }
    // A tree path is only ever after a `--`, and no row carries a date: the
    // history compares `--since` with the author dates in process.
    for (label, invocation, _, _) in rows() {
        let argv = invocation.argv();
        let first = argv.iter().position(|arg| arg == "--");
        for (index, arg) in invocation.args().iter().enumerate() {
            if matches!(arg, Arg::Path(_)) {
                assert!(first.is_some_and(|at| at < index), "{label}: {argv:?}");
            }
        }
        assert!(
            !argv.iter().any(|arg| arg.starts_with("--since")),
            "{label}: {argv:?}"
        );
    }
    // The cargo rows carry --frozen; the git rows have none to carry.
    for (label, invocation, program, _) in rows() {
        let frozen = invocation.argv().contains(&"--frozen".to_string());
        assert_eq!(frozen, program == Program::Cargo, "{label}");
    }
}

#[test]
fn a_ref_cannot_be_read_as_an_option_inside_its_argument() {
    // A remote branch called `--output=x` is a full ref and reaches git after
    // `--end-of-options` as one argument that begins with `refs/`.
    let reference = RefName::new("refs/remotes/origin/--output=x").unwrap();
    let argv = Invocation::git_resolve(&reference).argv();
    assert_eq!(
        argv,
        [
            "rev-parse",
            "--verify",
            "--end-of-options",
            "refs/remotes/origin/--output=x^{commit}"
        ]
    );
}

// --------------------------------------------------------------------- fake

fn answer(text: &str) -> Output {
    Output::new(Some(0), text.as_bytes().to_vec(), Vec::new())
}

#[test]
fn the_fake_answers_a_scripted_invocation_and_records_it() {
    let runner = FakeRunner::new()
        .on(
            Program::Git,
            &["--version"],
            Ok(answer("git version 2.45.1\n")),
        )
        .on(
            Program::Cargo,
            &["run", "--frozen", "--package=raf-demo", "--example=demo"],
            Ok(answer("{}")),
        );
    let git = runner
        .run(&Invocation::git_version(), &Limits::query())
        .unwrap();
    assert!(git.success());
    assert_eq!(git.stdout, b"git version 2.45.1\n");
    let meta = runner
        .run(
            &Invocation::example_run(&pkg(), &ExampleName::new("demo").unwrap()),
            &Limits::query().with_max_output(64 * 1024 * 1024),
        )
        .unwrap();
    assert_eq!(meta.stdout, b"{}");

    let calls = runner.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].program(), Program::Git);
    assert_eq!(calls[0].args(), ["--version"]);
    assert_eq!(calls[0].command(), "git --version");
    assert_eq!(*calls[0].limits(), Limits::query());
    assert_eq!(
        calls[1].command(),
        "cargo run --frozen --package=raf-demo --example=demo"
    );
    assert_eq!(
        *calls[1].limits(),
        Limits::query().with_max_output(64 * 1024 * 1024)
    );
    assert_eq!(calls[1].limits().max_output(), 64 * 1024 * 1024);
}

#[test]
fn the_fake_repeats_an_answer_and_a_later_one_replaces_it() {
    let runner = FakeRunner::new()
        .on(Program::Git, &["--version"], Ok(answer("one")))
        .on(Program::Git, &["--version"], Ok(answer("two")));
    for _ in 0..3 {
        let output = runner
            .run(&Invocation::git_version(), &Limits::query())
            .unwrap();
        assert_eq!(output.stdout, b"two");
    }
    assert_eq!(runner.calls().len(), 3);
}

#[test]
fn the_fake_keeps_the_program_apart_from_the_arguments() {
    // The same argv for another program is another invocation.
    let runner = FakeRunner::new().on(Program::Cargo, &["--version"], Ok(answer("cargo")));
    let error = runner
        .run(&Invocation::git_version(), &Limits::query())
        .unwrap_err();
    assert!(matches!(error, ExecError::Unscripted { .. }), "{error}");
    // The recorded call remains: a test sees what it asked for.
    assert_eq!(runner.calls()[0].command(), "git --version");
}

#[test]
fn the_fake_refuses_an_invocation_it_has_no_answer_for() {
    let runner = FakeRunner::new().on(Program::Git, &["--version"], Ok(answer("x")));
    let error = runner
        .run(&Invocation::git_default_ref(), &Limits::query())
        .unwrap_err();
    assert_eq!(
        error,
        ExecError::Unscripted {
            command: "git symbolic-ref refs/remotes/origin/HEAD".to_string()
        }
    );
    assert!(error.to_string().contains("FakeRunner::on"));
    // Its neighbour, the scripted one, is answered.
    assert!(
        runner
            .run(&Invocation::git_version(), &Limits::query())
            .is_ok()
    );
}

#[test]
fn the_fake_returns_a_failing_exit_and_a_scripted_error() {
    let runner = FakeRunner::new()
        .on(
            Program::Git,
            &["--version"],
            Ok(Output::new(Some(128), Vec::new(), b"fatal".to_vec())),
        )
        .on(
            Program::Git,
            &["symbolic-ref", "refs/remotes/origin/HEAD"],
            Err(ExecError::Timeout {
                command: "git symbolic-ref refs/remotes/origin/HEAD".to_string(),
                timeout: QUERY_TIMEOUT,
                elapsed: QUERY_TIMEOUT,
            }),
        );
    let failed = runner
        .run(&Invocation::git_version(), &Limits::query())
        .unwrap();
    assert!(!failed.success());
    assert_eq!(failed.code, Some(128));
    assert_eq!(failed.stderr, b"fatal");
    let error = runner
        .run(&Invocation::git_default_ref(), &Limits::query())
        .unwrap_err();
    assert!(matches!(error, ExecError::Timeout { .. }));
}

#[test]
fn the_fake_applies_the_output_bound_as_the_real_runner_does() {
    let at = vec![b'x'; 8];
    let runner = FakeRunner::new()
        .on(
            Program::Git,
            &["--version"],
            Ok(Output::new(Some(0), at.clone(), Vec::new())),
        )
        .on(
            Program::Git,
            &["symbolic-ref", "refs/remotes/origin/HEAD"],
            Ok(Output::new(Some(0), Vec::new(), vec![b'x'; 9])),
        );
    let limits = Limits::query().with_max_output(8);
    // At the bound passes; one byte past it, on either stream, does not.
    assert_eq!(
        runner
            .run(&Invocation::git_version(), &limits)
            .unwrap()
            .stdout,
        at
    );
    let error = runner
        .run(&Invocation::git_default_ref(), &limits)
        .unwrap_err();
    assert!(
        matches!(
            error,
            ExecError::OutputTooLarge {
                stream: Stream::Stderr,
                limit: 8,
                ..
            }
        ),
        "{error:?}"
    );
}

#[test]
fn the_fake_applies_the_time_limit_as_the_real_runner_does() {
    let limit = Duration::from_secs(10);
    let runner = FakeRunner::new()
        .on(
            Program::Git,
            &["--version"],
            Ok(answer("in time").with_elapsed(limit)),
        )
        .on(
            Program::Git,
            &["symbolic-ref", "refs/remotes/origin/HEAD"],
            Ok(answer("late").with_elapsed(limit + Duration::from_millis(1))),
        );
    let limits = Limits::query().with_timeout(limit);
    // Exactly at the limit passes; a moment past it is a timeout.
    let output = runner.run(&Invocation::git_version(), &limits).unwrap();
    assert_eq!(output.stdout, b"in time");
    let error = runner
        .run(&Invocation::git_default_ref(), &limits)
        .unwrap_err();
    assert_eq!(
        error,
        ExecError::Timeout {
            command: "git symbolic-ref refs/remotes/origin/HEAD".to_string(),
            timeout: limit,
            elapsed: limit + Duration::from_millis(1),
        }
    );
}

#[test]
fn a_runner_reference_is_a_runner() {
    fn use_runner(runner: impl Runner) -> bool {
        runner
            .run(&Invocation::git_version(), &Limits::query())
            .is_ok()
    }
    let runner = FakeRunner::new().on(Program::Git, &["--version"], Ok(answer("x")));
    assert!(use_runner(&runner));
    assert_eq!(runner.calls().len(), 1);
}

// ------------------------------------------------- limits and the programs

#[test]
fn the_limits_are_the_documented_bounds() {
    assert_eq!(DEFAULT_OUTPUT_BOUND, 16 * 1024 * 1024);
    assert_eq!(BUILD_TIMEOUT, Duration::from_mins(30));
    assert_eq!(EXAMPLE_TIMEOUT, Duration::from_mins(5));

    let build = Limits::build();
    assert_eq!(build.max_output(), DEFAULT_OUTPUT_BOUND);
    assert_eq!(build.timeout(), BUILD_TIMEOUT);
    assert!(build.c_locale());

    let example = Limits::example();
    assert_eq!(example.max_output(), DEFAULT_OUTPUT_BOUND);
    assert_eq!(example.timeout(), EXAMPLE_TIMEOUT);
    assert!(!example.c_locale(), "an example's output is not parsed");

    let query = Limits::query();
    assert_eq!(query.max_output(), DEFAULT_OUTPUT_BOUND);
    assert_eq!(query.timeout(), QUERY_TIMEOUT);
    assert!(query.c_locale());

    let custom = Limits::new(10, Duration::from_secs(2), false);
    assert_eq!(custom.max_output(), 10);
    assert_eq!(custom.timeout(), Duration::from_secs(2));
    assert_eq!(
        build
            .with_max_output(5)
            .with_timeout(Duration::from_secs(1)),
        Limits::new(5, Duration::from_secs(1), true)
    );
}

#[test]
fn the_programs_come_from_cargo_and_the_path_and_can_be_replaced() {
    let here = PathBuf::from("/some/dir");
    let from_environment = Programs::new(&here);
    let expected = std::env::var_os("CARGO").map_or_else(|| PathBuf::from("cargo"), PathBuf::from);
    assert_eq!(from_environment.cargo, expected);
    assert_eq!(from_environment.git, PathBuf::from("git"));
    assert_eq!(from_environment.cwd, here);

    let unset = Programs::with_cargo_var(&here, None);
    assert_eq!(unset.cargo, PathBuf::from("cargo"));
    let set = Programs::with_cargo_var(&here, Some("/opt/stub/cargo".into()));
    assert_eq!(set.cargo, PathBuf::from("/opt/stub/cargo"));
    assert_eq!(
        set.path_of(Program::Cargo),
        Some(Path::new("/opt/stub/cargo"))
    );
    assert_eq!(set.path_of(Program::Git), Some(Path::new("git")));
    // The caller's own command names its program in its argv.
    assert_eq!(set.path_of(Program::Trusted), None);

    let replaced = set.with_cargo("c").with_git("g");
    assert_eq!(replaced.path_of(Program::Cargo), Some(Path::new("c")));
    assert_eq!(replaced.path_of(Program::Git), Some(Path::new("g")));
}

// ------------------------------------------------ the real runner, anywhere

fn here() -> PathBuf {
    std::env::current_dir().expect("a current directory")
}

/// Writing an executable and starting a process from another thread at once
/// can fail with "text file busy", because a forked child briefly holds the
/// descriptor that wrote the file. Every test of this file that writes a stub
/// holds this lock from the write to the end of its runs, and so does every
/// test that starts a real process, so that no fork overlaps a write.
static SPAWN: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SPAWN.lock().unwrap_or_else(PoisonError::into_inner)
}

#[test]
fn the_real_git_starts() {
    let _lock = serial();
    let runner = SystemRunner::new(Programs::new(here()));
    let git = runner
        .run(&Invocation::git_version(), &Limits::query())
        .expect("git starts");
    assert!(git.success(), "{git:?}");
    assert!(String::from_utf8_lossy(&git.stdout).starts_with("git version "));
    assert_eq!(git.stderr, b"");
}

#[test]
fn a_program_that_does_not_exist_is_a_spawn_error() {
    let _lock = serial();
    let programs = Programs::new(here()).with_git("raf-workplan-no-such-program");
    let runner = SystemRunner::new(programs);
    let error = runner
        .run(&Invocation::git_version(), &Limits::query())
        .unwrap_err();
    let ExecError::Spawn {
        kind,
        program,
        command,
        ..
    } = &error
    else {
        panic!("not a spawn error: {error:?}");
    };
    assert_eq!(*kind, std::io::ErrorKind::NotFound);
    assert_eq!(program, "raf-workplan-no-such-program");
    assert_eq!(command, "git --version");
    let text = error.to_string();
    assert!(text.contains("PATH") && text.contains("$CARGO"), "{text}");
    // The neighbour, the real git, starts.
    assert!(
        SystemRunner::new(Programs::new(here()))
            .run(&Invocation::git_version(), &Limits::query())
            .is_ok()
    );
}

#[test]
fn a_directory_that_does_not_exist_is_a_spawn_error() {
    let _lock = serial();
    let missing = std::env::temp_dir().join("raf-workplan-exec-no-such-directory");
    let runner = SystemRunner::new(Programs::new(&missing));
    let error = runner
        .run(&Invocation::git_version(), &Limits::query())
        .unwrap_err();
    assert!(matches!(error, ExecError::Spawn { .. }), "{error:?}");
}

#[test]
fn a_name_missing_from_the_search_path_is_a_spawn_error() {
    let _lock = serial();
    let empty = std::env::temp_dir().join("raf-workplan-exec-empty-search-path");
    let runner = SystemRunner::new(Programs::new(here())).search_path(&empty);
    let error = runner
        .run(&Invocation::git_version(), &Limits::query())
        .unwrap_err();
    let ExecError::Spawn { kind, message, .. } = &error else {
        panic!("not a spawn error: {error:?}");
    };
    assert_eq!(*kind, std::io::ErrorKind::NotFound);
    assert!(message.contains("search path"), "{message}");
}

// ----------------------------------------------- the real runner, with stubs

#[cfg(unix)]
mod stubs {
    use std::collections::BTreeSet;
    use std::fs;
    use std::os::unix::fs::OpenOptionsExt as _;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    /// A scratch directory with a `bin` directory for stubs and a `work`
    /// directory to run in, removed on drop.
    struct Scratch {
        root: PathBuf,
    }

    impl Scratch {
        /// The name is this process and a count, made with `create_dir`, so
        /// a directory left by an earlier process with the same id is passed
        /// over and two scratch directories are never one.
        fn new() -> Self {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let root = loop {
                let root = std::env::temp_dir().join(format!(
                    "raf-workplan-exec-{}-{}",
                    std::process::id(),
                    COUNTER.fetch_add(1, Ordering::Relaxed)
                ));
                match fs::create_dir(&root) {
                    Ok(()) => break root,
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => panic!("{}: {error}", root.display()),
                }
            };
            fs::create_dir(root.join("bin")).expect("a bin directory");
            fs::create_dir(root.join("work")).expect("a work directory");
            let root = fs::canonicalize(root).expect("a canonical scratch directory");
            Self { root }
        }

        fn bin(&self) -> PathBuf {
            self.root.join("bin")
        }

        fn work(&self) -> PathBuf {
            self.root.join("work")
        }

        /// Write an executable shell script called `name` into `bin`.
        fn stub(&self, name: &str, body: &str) -> PathBuf {
            self.script(name, body, 0o755)
        }

        fn script(&self, name: &str, body: &str, mode: u32) -> PathBuf {
            use std::io::Write as _;
            let path = self.bin().join(name);
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(mode)
                .open(&path)
                .expect("a stub file");
            write!(file, "#!/bin/sh\n{body}\n").expect("a stub body");
            path
        }

        fn read(&self, name: &str) -> Vec<u8> {
            fs::read(self.bin().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
        }

        fn text(&self, name: &str) -> String {
            String::from_utf8(self.read(name)).expect("UTF-8")
        }

        /// The recorded argv, one element for each NUL-terminated string.
        fn argv(&self) -> Vec<String> {
            let bytes = self.read("argv");
            let mut parts: Vec<&[u8]> = bytes.split(|b| *b == 0).collect();
            assert_eq!(parts.pop(), Some(&b""[..]), "the record ends in NUL");
            parts
                .into_iter()
                .map(|part| String::from_utf8(part.to_vec()).expect("UTF-8"))
                .collect()
        }

        fn runner(&self, stub: &Path) -> SystemRunner {
            SystemRunner::new(
                Programs::with_cargo_var(self.work(), Some(stub.into())).with_git(stub),
            )
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    /// A stub that records how it was started and prints a line to each stream.
    const RECORD: &str = r#"dir=$(dirname "$0")
printf '%s\0' "$@" > "$dir/argv"
pwd -P > "$dir/cwd"
printf '%s' "${LC_ALL-<unset>}" > "$dir/lc_all"
env | sed -n 's/^\([A-Za-z_][A-Za-z0-9_]*\)=.*/\1/p' | sort > "$dir/env"
if IFS= read -r line; then echo open > "$dir/stdin"; else echo closed > "$dir/stdin"; fi
echo to-stdout
echo to-stderr >&2"#;

    #[test]
    fn every_row_reaches_a_process_as_exactly_its_argv() {
        let _lock = serial();
        for (label, invocation, program, expected) in rows() {
            let scratch = Scratch::new();
            let stub = scratch.stub("record", RECORD);
            let runner = scratch.runner(&stub);
            let output = runner
                .run(&invocation, &Limits::query())
                .unwrap_or_else(|e| panic!("{label}: {e}"));
            assert!(output.success(), "{label}: {output:?}");
            let expected = words(expected);
            assert_eq!(scratch.argv(), expected, "{label} ({program})");
            assert_eq!(output.stdout, b"to-stdout\n");
            assert_eq!(output.stderr, b"to-stderr\n");
        }
    }

    #[test]
    fn a_stub_named_by_cargo_runs_for_cargo_and_a_stub_on_the_path_when_it_is_unset() {
        let _lock = serial();
        // $CARGO names the stub (the program is the path it is given).
        let named = Scratch::new();
        let stub = named.stub("any-name", RECORD);
        let programs = Programs::with_cargo_var(named.work(), Some(stub.clone().into()));
        assert_eq!(programs.cargo, stub);
        let invocation = Invocation::example_run(&pkg(), &ExampleName::new("demo").unwrap());
        SystemRunner::new(programs)
            .run(&invocation, &Limits::example())
            .unwrap();
        assert_eq!(
            named.argv(),
            ["run", "--frozen", "--package=raf-demo", "--example=demo"]
        );

        // $CARGO unset: the program is `cargo`, found on the search path.
        let found = Scratch::new();
        found.stub("cargo", RECORD);
        let programs = Programs::with_cargo_var(found.work(), None);
        assert_eq!(programs.cargo, PathBuf::from("cargo"));
        SystemRunner::new(programs)
            .search_path(found.bin())
            .run(
                &Invocation::example_run(&pkg(), &ExampleName::new("demo").unwrap()),
                &Limits::query().with_max_output(64 * 1024 * 1024),
            )
            .unwrap();
        assert_eq!(
            found.argv(),
            ["run", "--frozen", "--package=raf-demo", "--example=demo"]
        );

        // The first directory of the search path that holds the name wins, and
        // a directory without it is passed over.
        let first = Scratch::new();
        let second = Scratch::new();
        second.stub("git", RECORD);
        let joined = std::env::join_paths([first.bin(), second.bin()]).unwrap();
        SystemRunner::new(Programs::with_cargo_var(second.work(), None))
            .search_path(joined)
            .run(&Invocation::git_version(), &Limits::query())
            .unwrap();
        assert_eq!(second.argv(), ["--version"]);

        // A path given for the program is used as it is, not searched.
        let direct = Scratch::new();
        let stub = direct.stub("git", RECORD);
        let other = Scratch::new();
        SystemRunner::new(Programs::new(direct.work()).with_git(&stub))
            .search_path(other.bin())
            .run(&Invocation::git_version(), &Limits::query())
            .unwrap();
        assert_eq!(direct.argv(), ["--version"]);
    }

    #[test]
    fn the_process_runs_in_the_working_directory() {
        let _lock = serial();
        let scratch = Scratch::new();
        let stub = scratch.stub("record", RECORD);
        scratch
            .runner(&stub)
            .run(&Invocation::git_version(), &Limits::query())
            .unwrap();
        assert_eq!(
            scratch.text("cwd").trim_end(),
            scratch.work().to_str().unwrap()
        );
        assert_ne!(scratch.work(), here(), "the test's own directory differs");
    }

    #[test]
    fn standard_input_is_closed() {
        let _lock = serial();
        let scratch = Scratch::new();
        let stub = scratch.stub("record", RECORD);
        scratch
            .runner(&stub)
            .run(&Invocation::git_version(), &Limits::example())
            .unwrap();
        assert_eq!(scratch.text("stdin").trim(), "closed");
    }

    #[test]
    fn the_c_locale_is_set_when_the_output_is_parsed_and_left_alone_otherwise() {
        let _lock = serial();
        let inherited = std::env::var("LC_ALL").unwrap_or_else(|_| "<unset>".to_string());

        let parsed = Scratch::new();
        let stub = parsed.stub("record", RECORD);
        parsed
            .runner(&stub)
            .run(&Invocation::git_version(), &Limits::query())
            .unwrap();
        assert_eq!(parsed.text("lc_all"), "C");

        let plain = Scratch::new();
        let stub = plain.stub("record", RECORD);
        plain
            .runner(&stub)
            .run(&Invocation::git_version(), &Limits::example())
            .unwrap();
        assert_eq!(
            plain.text("lc_all"),
            inherited,
            "the caller's locale is kept"
        );
    }

    #[test]
    fn nothing_else_is_added_to_the_environment() {
        let _lock = serial();
        let parent: BTreeSet<String> = std::env::vars_os()
            .filter_map(|(key, _)| key.into_string().ok())
            .collect();
        // A shell may add these itself.
        let shell = ["PWD", "OLDPWD", "SHLVL", "_"];
        for limits in [Limits::query(), Limits::example()] {
            let scratch = Scratch::new();
            let stub = scratch.stub("record", RECORD);
            scratch
                .runner(&stub)
                .run(&Invocation::git_version(), &limits)
                .unwrap();
            let child: BTreeSet<String> = scratch.text("env").lines().map(str::to_string).collect();
            let added: Vec<&String> = child.difference(&parent).collect();
            for name in added {
                assert!(
                    shell.contains(&name.as_str()) || (name == "LC_ALL" && limits.c_locale()),
                    "{name} was added (c_locale {})",
                    limits.c_locale()
                );
            }
        }
    }

    #[test]
    fn a_failing_exit_code_and_a_signal_are_outputs_and_not_errors() {
        let _lock = serial();
        let scratch = Scratch::new();
        let exits = scratch.stub("exits", "echo bad >&2\nexit 7");
        let output = scratch
            .runner(&exits)
            .run(&Invocation::git_version(), &Limits::query())
            .unwrap();
        assert_eq!(output.code, Some(7));
        assert!(!output.success());
        assert_eq!(output.stderr, b"bad\n");

        let signalled = scratch.stub("signalled", "kill -s KILL $$");
        let output = scratch
            .runner(&signalled)
            .run(&Invocation::git_version(), &Limits::query())
            .unwrap();
        assert_eq!(output.code, None);
        assert!(!output.success());

        // The neighbour exits 0.
        let zero = scratch.stub("zero", "exit 0");
        assert!(
            scratch
                .runner(&zero)
                .run(&Invocation::git_version(), &Limits::query())
                .unwrap()
                .success()
        );
    }

    #[test]
    fn a_file_that_cannot_be_executed_is_a_spawn_error() {
        let _lock = serial();
        let scratch = Scratch::new();
        let plain = scratch.script("plain", "exit 0", 0o644);
        let error = scratch
            .runner(&plain)
            .run(&Invocation::git_version(), &Limits::query())
            .unwrap_err();
        let ExecError::Spawn { kind, .. } = &error else {
            panic!("not a spawn error: {error:?}");
        };
        assert_eq!(*kind, std::io::ErrorKind::PermissionDenied);
        // The same script, executable, runs.
        let runnable = scratch.stub("runnable", "exit 0");
        assert!(
            scratch
                .runner(&runnable)
                .run(&Invocation::git_version(), &Limits::query())
                .is_ok()
        );
    }

    // ---------------------------------------------------------- output bound

    /// A stub that writes this many bytes to standard output.
    fn writer(scratch: &Scratch, bytes: usize) -> PathBuf {
        scratch.stub(
            "writes-stdout",
            &format!("head -c {bytes} /dev/zero | tr '\\0' x"),
        )
    }

    #[test]
    fn the_default_bound_is_sixteen_mebibytes_and_a_larger_one_is_a_parameter() {
        let _lock = serial();
        let scratch = Scratch::new();
        // 17 MiB passes under the explicitly configured 64 MiB bound and fails
        // under the default.
        let size = 17 * 1024 * 1024;
        let stub = writer(&scratch, size);
        let metadata = scratch
            .runner(&stub)
            .run(
                &Invocation::git_version(),
                &Limits::query().with_max_output(64 * 1024 * 1024),
            )
            .unwrap();
        assert_eq!(metadata.stdout.len(), size);

        let other = Scratch::new();
        let stub = writer(&other, size);
        let error = other
            .runner(&stub)
            .run(&Invocation::git_version(), &Limits::query())
            .unwrap_err();
        assert!(
            matches!(
                error,
                ExecError::OutputTooLarge { limit, .. } if limit == DEFAULT_OUTPUT_BOUND
            ),
            "{error:?}"
        );
    }

    #[test]
    fn output_is_returned_byte_for_byte() {
        let _lock = serial();
        let scratch = Scratch::new();
        let stub = scratch.stub("bytes", "printf 'a\\000b\\377c\\r\\n'");
        let output = scratch
            .runner(&stub)
            .run(&Invocation::git_version(), &Limits::query())
            .unwrap();
        assert_eq!(output.stdout, b"a\0b\xffc\r\n");
        assert_eq!(output.stderr, b"");
    }
}
