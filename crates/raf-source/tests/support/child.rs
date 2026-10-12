// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Run a parse in a child process: the test binary runs itself.
//!
//! A deep parse can end the process, so the parent cannot judge it in
//! process. The parent starts this same test binary, selects the one test that
//! calls [`run_child_if_requested`], and names the work in an environment
//! variable. The child prints one result line and exits. The parent maps what
//! happened to exactly one [`Outcome`].

use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use raf_source::{LoadErrorKind, RelPath, parse_source, syn};

/// The environment variable that names the work for a child.
const SELECTOR: &str = "RAF_SOURCE_CHILD";
/// Marks the child's one result line. The test harness may print its own
/// text before it on the same line.
const MARK: &str = "RAF_CHILD ";
/// The exit status of a typed refusal.
const REFUSED: i32 = 3;
/// The runtime's text when a thread runs out of stack.
const OVERFLOW_MESSAGE: &str = "has overflowed its stack";
/// The exit status of any other load error.
const FAILED: i32 = 4;

/// What became of one parse.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// The pre-scan refused the file, naming it and the budget.
    Refused { file: String, budget: String },
    /// The process exited 0 with this index.
    Index(Vec<String>),
    /// The process ended abnormally: a signal or a non-zero exit.
    Aborted { detail: String },
    /// The load failed in some other typed way.
    Failed { message: String },
}

/// The names of the top-level items of a file.
pub(crate) fn symbols(file: &syn::File) -> Vec<String> {
    file.items
        .iter()
        .map(|item| match item {
            syn::Item::Fn(item) => item.sig.ident.to_string(),
            syn::Item::Struct(item) => item.ident.to_string(),
            syn::Item::Enum(item) => item.ident.to_string(),
            syn::Item::Const(item) => item.ident.to_string(),
            syn::Item::Static(item) => item.ident.to_string(),
            syn::Item::Type(item) => item.ident.to_string(),
            syn::Item::Mod(item) => item.ident.to_string(),
            syn::Item::Trait(item) => item.ident.to_string(),
            syn::Item::Union(item) => item.ident.to_string(),
            _ => "<unnamed>".to_owned(),
        })
        .collect()
}

/// In the child: parse the selected text, print one line and exit. Without a
/// selector this returns at once, so the test that calls it passes when it
/// runs as an ordinary test.
pub(crate) fn run_child_if_requested(text_of: impl FnOnce(&str) -> String) {
    let Some(selector) = env::var_os(SELECTOR) else {
        return;
    };
    let selector = selector.to_string_lossy().into_owned();
    let (name, text) = (selector.clone(), text_of(&selector));
    let file = RelPath::new(&format!("{name}.rs")).expect("the selector is a plain name");
    let code = match parse_source(&file, &text, symbols) {
        Ok(found) => {
            println!("{MARK}index {}", found.join(","));
            0
        }
        Err(error) => {
            if let LoadErrorKind::Budget { budget, .. } = &error.kind {
                println!("{MARK}refused {file} {budget}");
                REFUSED
            } else {
                println!("{MARK}failed {error}");
                FAILED
            }
        }
    };
    std::process::exit(code);
}

/// Run the test named `test` of this binary as a child on `selector`.
pub(crate) fn run_child(test: &str, selector: &str) -> Outcome {
    let executable: PathBuf = env::current_exe().expect("the test binary has a path");
    let output = Command::new(executable)
        .args(["--exact", test, "--nocapture", "--test-threads=1"])
        .env(SELECTOR, selector)
        .env_remove("RUST_MIN_STACK")
        .stdin(Stdio::null())
        .output()
        .expect("start the child");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    classify(
        output.status.code(),
        died_by_signal(output.status),
        &stdout,
        &stderr,
    )
}

/// True when the process was ended by a signal.
#[cfg(unix)]
fn died_by_signal(status: std::process::ExitStatus) -> bool {
    use std::os::unix::process::ExitStatusExt as _;
    status.signal().is_some()
}

/// Without Unix there are no signals; the stack-overflow status is judged by
/// its exit code in [`classify`].
#[cfg(not(unix))]
fn died_by_signal(_status: std::process::ExitStatus) -> bool {
    false
}

/// The exit status the Windows runtime reports for a stack overflow.
const WINDOWS_STACK_OVERFLOW: i32 = 0xC000_00FD_u32.cast_signed();

/// Map what a child did to exactly one outcome.
///
/// * a result line with exit 0 is an index, and with exit 3 a refusal;
/// * an abort is only death by signal, the Windows stack-overflow status, or
///   the runtime's own stack-overflow message;
/// * everything else (a panic, an unknown selector, a non-zero exit without a
///   result line, exit 0 without one) is a failure.
pub(crate) fn classify(code: Option<i32>, signaled: bool, stdout: &str, stderr: &str) -> Outcome {
    let line = stdout
        .lines()
        .find_map(|line| line.split_once(MARK))
        .map(|(_, rest)| rest.to_owned());
    match (code, line) {
        (Some(0), Some(line)) => match line.strip_prefix("index ") {
            Some(list) => Outcome::Index(if list.is_empty() {
                Vec::new()
            } else {
                list.split(',').map(str::to_owned).collect()
            }),
            None => Outcome::Failed {
                message: format!("exit 0 with a line that is not an index: {line}"),
            },
        },
        (Some(REFUSED), Some(line)) => match line.strip_prefix("refused ") {
            Some(rest) => match rest.split_once(' ') {
                Some((file, budget)) => Outcome::Refused {
                    file: file.to_owned(),
                    budget: budget.to_owned(),
                },
                None => Outcome::Failed {
                    message: format!("a refusal without a budget: {rest}"),
                },
            },
            None => Outcome::Failed {
                message: format!("exit {REFUSED} with a line that is not a refusal: {line}"),
            },
        },
        (Some(FAILED), Some(line)) => Outcome::Failed { message: line },
        (code, line) => {
            let overflow = stderr.contains(OVERFLOW_MESSAGE);
            if line.is_none() && (signaled || overflow || code == Some(WINDOWS_STACK_OVERFLOW)) {
                Outcome::Aborted {
                    detail: format!("status {code:?}; stderr tail: {}", tail(stderr)),
                }
            } else {
                Outcome::Failed {
                    message: format!(
                        "status {code:?} is not an abort; result line {line:?}; stderr tail: {}",
                        tail(stderr)
                    ),
                }
            }
        }
    }
}

fn tail(text: &str) -> String {
    const KEEP: usize = 4000;
    let start = text.char_indices().rev().nth(KEEP).map_or(0, |(at, _)| at);
    text[start..].to_owned()
}

/// Judge an outcome against the allowed three. A refusal must name the file;
/// an index must equal `expected`; an abort stands as it is. Returns the
/// problem, or `None` when the outcome is allowed.
pub(crate) fn problem(outcome: &Outcome, file: &str, expected: &[&str]) -> Option<String> {
    match outcome {
        Outcome::Refused {
            file: named,
            budget,
        } => {
            if named != file {
                Some(format!("refusal names {named:?}, not {file:?}"))
            } else if budget.is_empty() {
                Some("refusal does not name a budget".to_owned())
            } else {
                None
            }
        }
        Outcome::Index(found) => {
            if found
                .iter()
                .map(String::as_str)
                .eq(expected.iter().copied())
            {
                None
            } else {
                Some(format!(
                    "exit 0 with the index {found:?}, expected {expected:?}"
                ))
            }
        }
        Outcome::Aborted { .. } => None,
        Outcome::Failed { message } => {
            Some(format!("failed outside the three outcomes: {message}"))
        }
    }
}
