// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Pure parsers for what libtest and rustdoc print.
//!
//! None of them starts a process or fails: a line that is not of the form a
//! parser reads is noise and is skipped, and the caller decides what a missing
//! entry means. Every match is on a whole line, so a name that is a prefix or
//! a suffix of another never stands for it.

use crate::exec::Pattern;

/// What a line of `--list` output says an entry is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryKind {
    /// `<name>: test`. An `#[ignore]`d test is listed as a test too.
    Test,
    /// `<name>: benchmark`.
    Benchmark,
}

/// One entry of libtest's `--list` output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ListEntry {
    pub(crate) name: String,
    pub(crate) kind: EntryKind,
}

/// The entries of a libtest `--list` run: every line `<name>: test` or
/// `<name>: benchmark`. The count line (`1 test, 0 benchmarks`), blank lines
/// and anything else are skipped.
pub(crate) fn test_list(stdout: &str) -> Vec<ListEntry> {
    stdout
        .lines()
        .filter_map(|line| {
            let (name, kind) = if let Some(name) = line.strip_suffix(": test") {
                (name, EntryKind::Test)
            } else {
                (line.strip_suffix(": benchmark")?, EntryKind::Benchmark)
            };
            // A name is a path of identifiers (the pattern of a test name), so
            // free text before the colon of a diagnostic is noise.
            Pattern::Name.matches(name).then(|| ListEntry {
                name: name.to_string(),
                kind,
            })
        })
        .collect()
}

/// The counts of one `test result:` line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Counts {
    pub(crate) passed: usize,
    pub(crate) failed: usize,
    pub(crate) ignored: usize,
}

/// A `test result:` line that could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BadResult(pub(crate) String);

/// Every `test result: ok. 1 passed; 0 failed; 0 ignored; ...` line, in order.
/// A line that starts so and does not carry the three counts is an error: it
/// is a result that cannot be counted.
pub(crate) fn results(stdout: &str) -> Result<Vec<Counts>, BadResult> {
    stdout
        .lines()
        .filter_map(|line| line.strip_prefix("test result: ").map(|rest| (line, rest)))
        .map(|(line, rest)| count_line(rest).ok_or_else(|| BadResult(line.to_string())))
        .collect()
}

/// `ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
/// without its `test result: ` prefix.
fn count_line(rest: &str) -> Option<Counts> {
    let (status, counts) = rest.split_once(". ")?;
    if !matches!(status, "ok" | "FAILED") {
        return None;
    }
    let mut passed = None;
    let mut failed = None;
    let mut ignored = None;
    for part in counts.split("; ") {
        let (number, what) = part.split_once(' ')?;
        let slot = match what {
            "passed" => &mut passed,
            "failed" => &mut failed,
            "ignored" => &mut ignored,
            _ => continue,
        };
        if slot.is_some() {
            return None;
        }
        *slot = Some(number.parse::<usize>().ok()?);
    }
    Some(Counts {
        passed: passed?,
        failed: failed?,
        ignored: ignored?,
    })
}

/// One entry of rustdoc's `--list` output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocEntry {
    /// The whole name, as `test <name> ... ok` repeats it:
    /// `src/lib.rs - inner::foo (line 22)`.
    pub(crate) name: String,
    /// The item part: the text between ` - ` and ` (line `. `None` for the
    /// documentation of a crate or module root, which has no item part
    /// (`src/lib.rs - (line 3)`).
    pub(crate) item: Option<String>,
}

/// The documentation tests of a rustdoc `--list` run: every line
/// `<relative-source-path> - <item-path> (line <n>): test`. Rustdoc prints one
/// block for the tests it runs together and another for the `compile_fail`
/// ones; both are read. Other lines are skipped.
pub(crate) fn doc_list(stdout: &str) -> Vec<DocEntry> {
    stdout.lines().filter_map(doc_entry).collect()
}

fn doc_entry(line: &str) -> Option<DocEntry> {
    let name = line.strip_suffix(": test")?;
    let (file, rest) = name.split_once(" - ")?;
    // rustdoc prints the file part with the platform's separator; the pattern is
    // checked on the `/` form, and the printed name is kept for the `ok`-line match.
    if !Pattern::SourcePath.matches(&file.replace('\\', "/")) {
        return None;
    }
    let item = if let Some(at) = rest.rfind(" (line ") {
        line_number(&rest[at + " (line ".len()..])?;
        Some(rest[..at].to_string())
    } else {
        line_number(rest.strip_prefix("(line ")?)?;
        None
    };
    if item.as_deref() == Some("") {
        return None;
    }
    Some(DocEntry {
        name: name.to_string(),
        item,
    })
}

/// `<digits>)`: the end of a name.
fn line_number(tail: &str) -> Option<()> {
    let digits = tail.strip_suffix(')')?;
    (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())).then_some(())
}

/// What the run output says about one documentation test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DocStatus {
    /// `test <name> ... ok`: built and run, and passed.
    Ok,
    /// `test <name> - compile ... ok`: a `no_run` test, built and not run.
    CompileOnly,
    /// `test <name> - compile fail ... ok`: a `compile_fail` test, which
    /// passes when the build fails.
    CompileFail,
    /// The line ends in `FAILED`.
    Failed,
    /// The test was ignored.
    Ignored,
    /// No line names the test.
    Missing,
}

/// The status of the documentation test `name` in a rustdoc run, read from
/// the whole line `test <name> ... ok` (and its `- compile` forms) with
/// nothing between the name and `...` that this function does not name.
///
/// The first line that names the test decides. The name ends in `)`, so the
/// name of a test is never a prefix of another's.
pub(crate) fn doc_status(stdout: &str, name: &str) -> DocStatus {
    let prefix = format!("test {name} ");
    for line in stdout.lines() {
        let Some(rest) = line.strip_prefix(&prefix) else {
            continue;
        };
        return match rest {
            "... ok" => DocStatus::Ok,
            "- compile ... ok" => DocStatus::CompileOnly,
            "- compile fail ... ok" => DocStatus::CompileFail,
            _ if rest.ends_with("FAILED") => DocStatus::Failed,
            _ if rest.starts_with("... ignored") || rest.contains(" ... ignored") => {
                DocStatus::Ignored
            }
            // A line that starts with the name and says something else is not
            // proof of a run.
            _ => DocStatus::Missing,
        };
    }
    DocStatus::Missing
}
