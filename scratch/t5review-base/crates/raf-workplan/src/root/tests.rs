// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Root discovery against scripted answers: the one command, its limits, and
//! every way its output can fail to give a root. The real `cargo` is
//! exercised by `tests/root.rs`.

use super::{RootError, discover, from_metadata};
use crate::exec::{ExecError, FakeRunner, Limits, METADATA_OUTPUT_BOUND, Output, Program, Stream};

const ARGV: [&str; 5] = ["metadata", "--format-version", "1", "--no-deps", "--frozen"];

fn runner(answer: Result<Output, ExecError>) -> FakeRunner {
    FakeRunner::new().on(Program::Cargo, &ARGV, answer)
}

fn said(text: &str) -> Output {
    Output::new(Some(0), text.as_bytes().to_vec(), Vec::new())
}

#[cfg(unix)]
const ROOT: &str = "/work/space";
#[cfg(windows)]
const ROOT: &str = "C:/work/space";

fn metadata() -> String {
    format!(r#"{{"packages":[],"version":1,"workspace_root":"{ROOT}"}}"#)
}

#[test]
fn the_root_is_the_workspace_root_member_and_the_command_is_exactly_one() {
    let runner = runner(Ok(said(&metadata())));
    assert_eq!(discover(&runner), Ok(ROOT.into()));
    let calls = runner.calls();
    assert_eq!(calls.len(), 1, "exactly one process");
    assert_eq!(calls[0].program(), Program::Cargo);
    assert_eq!(calls[0].args(), ARGV);
    assert_eq!(
        calls[0].command(),
        "cargo metadata --format-version 1 --no-deps --frozen"
    );
    assert_eq!(calls[0].limits(), &Limits::metadata());
    assert_eq!(calls[0].limits().max_output(), 64 * 1024 * 1024);
}

#[test]
fn a_failing_cargo_is_an_error_with_what_it_said() {
    let failing = runner(Ok(Output::new(
        Some(101),
        metadata().into_bytes(),
        b"error: could not find `Cargo.toml` in `/tmp` or any parent directory\n".to_vec(),
    )));
    let error = discover(&failing).expect_err("cargo failed");
    assert!(
        matches!(&error, RootError::Failed { code: Some(101), stderr } if stderr.contains("could not find")),
        "{error:?}"
    );
    assert!(error.to_string().contains("--no-deps --frozen"), "{error}");
    let killed = runner(Ok(Output::new(None, metadata().into_bytes(), Vec::new())));
    assert!(matches!(
        discover(&killed),
        Err(RootError::Failed { code: None, .. })
    ));
}

#[test]
fn cargo_that_cannot_run_or_writes_too_much_is_an_error() {
    let unscripted = discover(&FakeRunner::new()).expect_err("nothing scripted");
    assert!(matches!(
        unscripted,
        RootError::Exec(ExecError::Unscripted { .. })
    ));
    let timeout = runner(Err(ExecError::Timeout {
        command: "cargo metadata".to_string(),
        timeout: std::time::Duration::from_secs(300),
        elapsed: std::time::Duration::from_secs(301),
    }));
    assert!(matches!(
        discover(&timeout),
        Err(RootError::Exec(ExecError::Timeout { .. }))
    ));

    // The bound is 64 MiB: a document padded to exactly the bound is read, and
    // one byte more is refused before it is looked at.
    let padding = METADATA_OUTPUT_BOUND - metadata().len();
    let exact = format!("{}{}", metadata(), " ".repeat(padding));
    assert_eq!(exact.len(), METADATA_OUTPUT_BOUND);
    assert_eq!(discover(&runner(Ok(said(&exact)))), Ok(ROOT.into()));
    let over = format!("{exact} ");
    let error = discover(&runner(Ok(said(&over)))).expect_err("one byte over");
    assert!(
        matches!(
            &error,
            RootError::Exec(ExecError::OutputTooLarge { stream: Stream::Stdout, limit, .. })
                if *limit == METADATA_OUTPUT_BOUND
        ),
        "{error:?}"
    );
}

#[test]
fn output_that_does_not_give_a_root_is_an_error() {
    let bytes = runner(Ok(Output::new(Some(0), vec![b'{', 0xff, b'}'], Vec::new())));
    assert_eq!(discover(&bytes), Err(RootError::NotUtf8));
    for text in [
        "",
        "Cargo.toml\n",
        "{}",
        r#"{"x":{"workspace_root":"/nested"}}"#,
        r#"{"workspace_root":1}"#,
        r#"{"workspace_root":"/a","workspace_root":"/b"}"#,
        r#"{"workspace_root":"/a"} trailing"#,
    ] {
        let error = discover(&runner(Ok(said(text)))).expect_err(text);
        assert!(matches!(error, RootError::Json(_)), "{text}: {error:?}");
        assert!(
            error
                .to_string()
                .contains("does not give the workspace root"),
            "{error}"
        );
    }
}

#[test]
fn a_root_that_is_not_an_absolute_path_is_refused() {
    for value in ["", "relative/dir", ".", "..", "~/work", "a\\u0000b"] {
        let text = format!(r#"{{"workspace_root":"{value}"}}"#);
        let error = from_metadata(&text).expect_err(value);
        assert!(
            matches!(error, RootError::NotAbsolute { .. }),
            "{value}: {error:?}"
        );
    }
    // A NUL inside an otherwise absolute path.
    let text = format!(r#"{{"workspace_root":"{ROOT}\u0000x"}}"#);
    assert!(matches!(
        from_metadata(&text),
        Err(RootError::NotAbsolute { .. })
    ));
    // The neighbour of each refusal passes.
    assert_eq!(from_metadata(&metadata()), Ok(ROOT.into()));
}
