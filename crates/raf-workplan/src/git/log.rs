// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The questions of the history command: whether the repository is shallow,
//! the log of a document through its moves, the log of the lines of a symbol,
//! and the newest commit, with its parent, that changed how often an
//! identifier occurs in a directory.
//!
//! A log is read as records. The two history logs print, for each commit, a
//! line that begins with the record separator `0x1e` and holds the commit, the
//! author date and the subject separated by the unit separator `0x1f`; the
//! document log then lists the paths the commit touched, one to a line. A line
//! is split at `0x1f` at most twice, so the subject keeps every byte it has
//! other than a newline, `0x1e` and `0x1f` included; a subject that is not
//! UTF-8 has each invalid sequence shown as U+FFFD. A path that `git` quotes
//! (`"..."` with C escapes, as it writes a name with a control character, a
//! quote, a backslash or, under `core.quotePath`, a byte above 0x7f) is
//! unquoted. Anything else in the output, a commit id or a date that is not of
//! its form, and a path that is not UTF-8 are each an error: a log is never
//! read in part.
//!
//! Each question reads at most 16 MiB of output ([`Limits::query`]); more is
//! [`GitError::LogTooLarge`] and nothing is returned.

use std::cmp::Ordering;
use std::fmt;

use crate::exec::{
    Commit, Date, ExecError, Identifier, Invocation, Limits, LineRange, Output, Runner, Stream,
    TreePath, days_in_month,
};

use super::{GitError, failed, one_line, shown};

/// The record separator that opens each record of a history log.
const RECORD: u8 = 0x1e;

/// The unit separator between the fields of a record.
const UNIT: u8 = 0x1f;

/// The author date of a commit as `%aI` prints it, strict ISO 8601:
/// `YYYY-MM-DDTHH:MM:SS` and an offset `+HH:MM`, `-HH:MM` or `Z`.
///
/// Dates are ordered by the instant they name and then by their text, so two
/// commits made at one instant in different zones are ordered the same way on
/// every run.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthorDate {
    text: String,
    seconds: i64,
}

impl AuthorDate {
    /// The date `text` names, when it is of the strict form.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        if bytes.len() < 20 {
            return None;
        }
        let digits = |range: std::ops::Range<usize>| -> Option<i64> {
            let part = bytes.get(range)?;
            part.iter().all(u8::is_ascii_digit).then(|| {
                part.iter()
                    .fold(0, |total, b| total * 10 + i64::from(b - b'0'))
            })
        };
        let punctuation = [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')];
        if punctuation.iter().any(|(at, byte)| bytes[*at] != *byte) {
            return None;
        }
        let (year, month, day) = (digits(0..4)?, digits(5..7)?, digits(8..10)?);
        let (hour, minute, second) = (digits(11..13)?, digits(14..16)?, digits(17..19)?);
        let in_month = u32::try_from(year)
            .ok()
            .zip(u32::try_from(month).ok())
            .filter(|(_, month)| (1..=12).contains(month))
            .map(|(year, month)| i64::from(days_in_month(year, month)))?;
        if !(1..=in_month).contains(&day) || hour > 23 || minute > 59 || second > 60 {
            return None;
        }
        let offset = match &bytes[19..] {
            b"Z" => 0,
            [sign @ (b'+' | b'-'), _, _, b':', _, _] => {
                let (hours, minutes) = (digits(20..22)?, digits(23..25)?);
                if hours > 23 || minutes > 59 {
                    return None;
                }
                let size = hours * 3600 + minutes * 60;
                if *sign == b'+' { size } else { -size }
            }
            _ => return None,
        };
        let seconds =
            days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second
                - offset;
        Some(Self {
            text: text.to_string(),
            seconds,
        })
    }

    /// The date as `git` printed it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The day, `YYYY-MM-DD`, as the date writes it in its own zone.
    #[must_use]
    pub fn day(&self) -> &str {
        &self.text[..10]
    }

    /// The instant, in seconds since 1970-01-01T00:00:00Z.
    #[must_use]
    pub fn seconds(&self) -> i64 {
        self.seconds
    }
}

impl Ord for AuthorDate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.seconds
            .cmp(&other.seconds)
            .then_with(|| self.text.cmp(&other.text))
    }
}

impl PartialOrd for AuthorDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for AuthorDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// The days from 1970-01-01 to a day of the Gregorian calendar.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_index = (month + 9) % 12;
    let day_of_year = (153 * month_index + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// One commit of a log: its id, its author date, its subject and, for the
/// document log, the paths it touched as `git` names them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    commit: Commit,
    date: AuthorDate,
    subject: String,
    paths: Vec<String>,
}

impl LogEntry {
    /// The commit.
    #[must_use]
    pub fn commit(&self) -> &Commit {
        &self.commit
    }

    /// The author date.
    #[must_use]
    pub fn date(&self) -> &AuthorDate {
        &self.date
    }

    /// The subject: data, never interpreted.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// The paths the commit touched, in the order `git` listed them; never
    /// empty in a document log, and empty in the logs that list none.
    #[must_use]
    pub fn paths(&self) -> &[String] {
        &self.paths
    }
}

/// Run a log query: an output past the bound is [`GitError::LogTooLarge`].
fn log_query(runner: &dyn Runner, invocation: &Invocation) -> Result<Output, GitError> {
    let limits = Limits::query();
    match runner.run(invocation, &limits) {
        Ok(output) => Ok(output),
        Err(ExecError::OutputTooLarge {
            stream: Stream::Stdout,
            limit,
            ..
        }) => Err(GitError::LogTooLarge {
            command: invocation.to_string(),
            limit,
        }),
        Err(error) => Err(error.into()),
    }
}

/// The error of an output that is not what the log allows.
fn malformed(invocation: &Invocation, expected: &'static str, line: &[u8]) -> GitError {
    GitError::Answer {
        command: invocation.to_string(),
        expected,
        answer: shown(&String::from_utf8_lossy(line)),
    }
}

/// The header of one record: `<commit> 0x1f <date> 0x1f <subject>`.
fn header(invocation: &Invocation, line: &[u8]) -> Result<LogEntry, GitError> {
    const EXPECTED: &str = "a record of a commit id, an author date and a subject";
    let mut fields = line.splitn(3, |byte| *byte == UNIT);
    let (Some(commit), Some(date), Some(subject)) = (fields.next(), fields.next(), fields.next())
    else {
        return Err(malformed(invocation, EXPECTED, line));
    };
    let commit = std::str::from_utf8(commit)
        .ok()
        .and_then(|text| Commit::new(text).ok())
        .ok_or_else(|| malformed(invocation, "a commit id", line))?;
    let date = std::str::from_utf8(date)
        .ok()
        .and_then(AuthorDate::parse)
        .ok_or_else(|| malformed(invocation, "a strict ISO 8601 author date", line))?;
    Ok(LogEntry {
        commit,
        date,
        subject: String::from_utf8_lossy(subject).into_owned(),
        paths: Vec::new(),
    })
}

/// A path line of `--name-only`, unquoted when `git` quoted it.
fn path_line(invocation: &Invocation, line: &[u8]) -> Result<String, GitError> {
    let bytes = match line {
        [b'"', inner @ .., b'"'] => {
            unquote(inner).ok_or_else(|| malformed(invocation, "a quoted path", line))?
        }
        _ => line.to_vec(),
    };
    String::from_utf8(bytes).map_err(|error| GitError::NotUtf8 {
        reason: format!("`{invocation}` listed a path that is not UTF-8: {error}"),
    })
}

/// The bytes of a C-quoted name without its quotes, as `git` writes them:
/// `\a \b \t \n \v \f \r \" \\` and three octal digits for any other byte.
fn unquote(inner: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(inner.len());
    let mut at = 0;
    while let Some(byte) = inner.get(at).copied() {
        at += 1;
        if byte == b'"' {
            return None;
        }
        if byte != b'\\' {
            out.push(byte);
            continue;
        }
        let escape = inner.get(at).copied()?;
        at += 1;
        out.push(match escape {
            b'a' => 0x07,
            b'b' => 0x08,
            b't' => b'\t',
            b'n' => b'\n',
            b'v' => 0x0b,
            b'f' => 0x0c,
            b'r' => b'\r',
            b'"' => b'"',
            b'\\' => b'\\',
            b'0'..=b'3' => {
                let rest = inner.get(at..at + 2)?;
                if !rest.iter().all(|b| (b'0'..=b'7').contains(b)) {
                    return None;
                }
                at += 2;
                (escape - b'0') * 64 + (rest[0] - b'0') * 8 + (rest[1] - b'0')
            }
            _ => return None,
        });
    }
    Some(out)
}

/// The records of a history log. With `paths` the lines after a header are
/// the paths of its commit; without, a line that is not a header and not
/// empty is an error.
fn records(
    invocation: &Invocation,
    output: &Output,
    paths: bool,
) -> Result<Vec<LogEntry>, GitError> {
    let mut entries: Vec<LogEntry> = Vec::new();
    let text = output.stdout.strip_suffix(b"\n").unwrap_or(&output.stdout);
    if text.is_empty() {
        return Ok(entries);
    }
    for line in text.split(|byte| *byte == b'\n') {
        if let Some(rest) = line.strip_prefix(&[RECORD]) {
            entries.push(header(invocation, rest)?);
        } else if line.is_empty() {
        } else if let (true, Some(entry)) = (paths, entries.last_mut()) {
            entry.paths.push(path_line(invocation, line)?);
        } else {
            return Err(malformed(
                invocation,
                "a record that begins with 0x1e",
                line,
            ));
        }
    }
    Ok(entries)
}

/// Whether the repository is a shallow clone, whose history stops at a
/// boundary: `git rev-parse --is-shallow-repository`.
///
/// # Errors
///
/// [`GitError::Failed`] when the command fails; [`GitError::Answer`] when the
/// answer is not `true` or `false`; [`GitError::Exec`] when it cannot run.
pub fn is_shallow(runner: &dyn Runner) -> Result<bool, GitError> {
    let invocation = Invocation::git_shallow();
    let output = runner.run(&invocation, &Limits::query())?;
    if !output.success() {
        return Err(failed(&invocation, &output));
    }
    match one_line(&invocation, &output, "`true` or `false`")? {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(GitError::Answer {
            command: invocation.to_string(),
            expected: "`true` or `false`",
            answer: shown(other),
        }),
    }
}

/// The commits of a document, newest first as `git` lists them, through the
/// moves `git` detects, each with the name the document had at that commit:
/// `git --no-pager log --no-show-signature --follow --cc --root
/// --format=%x1e%H%x1f%aI%x1f%s --name-only [--since=<date> 00:00:00 +0000] --
/// <doc-path>`. `--cc` lists a merge whose resolution changed the document (a
/// merge that took one side unchanged is not listed, as it changed nothing),
/// and `--root` keeps a user's `log.showRoot=false` from hiding the commit
/// that created it. A commit that lists no path, as `--follow` lists a merge
/// that changed nothing in the document, is left out. `--since` counts from
/// 00:00:00 UTC of the day, whatever the clock and the time zone.
///
/// # Errors
///
/// [`GitError::LogTooLarge`] for an output over 16 MiB; [`GitError::Answer`]
/// or [`GitError::NotUtf8`] for an output that is not a log of this form;
/// [`GitError::Failed`] when the command fails; [`GitError::Exec`] when it
/// cannot run.
pub fn document_log(
    runner: &dyn Runner,
    path: &TreePath,
    since: Option<&Date>,
) -> Result<Vec<LogEntry>, GitError> {
    let invocation = Invocation::git_document_log(path, since);
    let output = log_query(runner, &invocation)?;
    if !output.success() {
        return Err(failed(&invocation, &output));
    }
    let mut entries = records(&invocation, &output, true)?;
    // `--follow` lists a merge that changed nothing in the document, with no
    // path; it did not shape the document and is not an entry.
    entries.retain(|entry| !entry.paths.is_empty());
    Ok(entries)
}

/// The commits that changed the lines of a file, newest first, traced back
/// from `HEAD`: `git --no-pager log --no-show-signature -L<start>,<end>:<file>
/// --no-patch --format=%x1e%H%x1f%aI%x1f%s [--since=<date> 00:00:00 +0000]
/// --`. The entries list no path.
///
/// # Errors
///
/// As [`document_log`]; a file that is not in `HEAD`, or has fewer lines there,
/// is [`GitError::Failed`].
pub fn symbol_log(
    runner: &dyn Runner,
    lines: LineRange,
    file: &TreePath,
    since: Option<&Date>,
) -> Result<Vec<LogEntry>, GitError> {
    let invocation = Invocation::git_symbol_log(lines, file, since);
    let output = log_query(runner, &invocation)?;
    if !output.success() {
        return Err(failed(&invocation, &output));
    }
    records(&invocation, &output, false)
}

/// The newest commit that changed how often an identifier occurs in a crate,
/// as [`last_change`] answers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastChange {
    entry: LogEntry,
    parent: Option<Commit>,
}

impl LastChange {
    /// The commit, its date and its subject; it lists no path.
    #[must_use]
    pub fn entry(&self) -> &LogEntry {
        &self.entry
    }

    /// Its first parent, or `None` for the first commit of the history.
    #[must_use]
    pub fn parent(&self) -> Option<&Commit> {
        self.parent.as_ref()
    }
}

/// The newest commit that changed how often `identifier` occurs in the files
/// below `dirs`, with its first parent, or `None` when no commit did: `git
/// --no-pager log --no-show-signature --root -S<identifier> -n 1
/// --format=%H%x1f%P%x1f%aI%x1f%s -- <crate-dir>...`. `--root` keeps a
/// user's `log.showRoot=false` from hiding the first commit. The parents are
/// the ids `%P` prints, each accepted only by the commit-id pattern; none for
/// the first commit of the history.
///
/// # Errors
///
/// As [`document_log`].
pub fn last_change(
    runner: &dyn Runner,
    identifier: &Identifier,
    dirs: &[TreePath],
) -> Result<Option<LastChange>, GitError> {
    const EXPECTED: &str = "a commit id, its parents, an author date and a subject";
    let invocation = Invocation::git_pickaxe(identifier, dirs);
    let output = log_query(runner, &invocation)?;
    if !output.success() {
        return Err(failed(&invocation, &output));
    }
    let text = output.stdout.strip_suffix(b"\n").unwrap_or(&output.stdout);
    if text.is_empty() {
        return Ok(None);
    }
    if text.contains(&b'\n') {
        return Err(malformed(&invocation, "one record", text));
    }
    let mut fields = text.splitn(3, |byte| *byte == UNIT);
    let (Some(commit), Some(parents), Some(rest)) = (fields.next(), fields.next(), fields.next())
    else {
        return Err(malformed(&invocation, EXPECTED, text));
    };
    let parents = std::str::from_utf8(parents)
        .ok()
        .and_then(|list| {
            if list.is_empty() {
                Some(Vec::new())
            } else {
                list.split(' ')
                    .map(|id| Commit::new(id).ok())
                    .collect::<Option<Vec<Commit>>>()
            }
        })
        .ok_or_else(|| malformed(&invocation, "the parent ids of a commit", text))?;
    let mut line = commit.to_vec();
    line.push(UNIT);
    line.extend_from_slice(rest);
    Ok(Some(LastChange {
        entry: header(&invocation, &line)?,
        parent: parents.into_iter().next(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::{FakeRunner, Program};

    const A: &str = "0123456789abcdef0123456789abcdef01234567";
    const B: &str = "fedcba9876543210fedcba9876543210fedcba98";

    #[allow(
        clippy::unnecessary_wraps,
        reason = "a scripted answer of the fake is a result"
    )]
    fn ok(stdout: &[u8]) -> Result<Output, ExecError> {
        Ok(Output::new(Some(0), stdout.to_vec(), Vec::new()))
    }

    fn path(text: &str) -> TreePath {
        TreePath::new(text).unwrap()
    }

    fn doc_argv(path: &str) -> Vec<String> {
        Invocation::git_document_log(&TreePath::new(path).unwrap(), None).argv()
    }

    fn run_doc(stdout: &[u8]) -> Result<Vec<LogEntry>, GitError> {
        let owned = doc_argv("docs/a.md");
        let args: Vec<&str> = owned.iter().map(String::as_str).collect();
        let runner = FakeRunner::new().on(Program::Git, &args, ok(stdout));
        document_log(&runner, &path("docs/a.md"), None)
    }

    #[test]
    fn a_strict_author_date_is_read_and_ordered_by_its_instant() {
        let utc = AuthorDate::parse("2026-05-02T10:00:00Z").unwrap();
        let east = AuthorDate::parse("2026-05-02T12:00:00+02:00").unwrap();
        let west = AuthorDate::parse("2026-05-02T05:00:01-05:00").unwrap();
        assert_eq!(utc.seconds(), east.seconds());
        assert!(west > utc && west > east);
        assert_eq!(east.day(), "2026-05-02");
        assert_eq!(
            AuthorDate::parse("1970-01-01T00:00:00+00:00")
                .unwrap()
                .seconds(),
            0
        );
        assert_eq!(
            AuthorDate::parse("2000-03-01T00:00:00Z").unwrap().seconds(),
            951_868_800
        );
        assert!(AuthorDate::parse("2028-02-29T00:00:00Z").is_some());
        for refused in [
            "",
            "2026-05-02",
            "2026-05-02 10:00:00 +0200",
            "2026-13-01T00:00:00Z",
            "2026-02-29T00:00:00Z",
            "2026-05-02T24:00:00Z",
            "2026-05-02T10:00:00+0200",
            "2026-05-02T10:00:00",
            "2026-05-02T10:00:00+02:00x",
            "2026-05-0xT10:00:00Z",
            "+026-05-02T10:00:00Z",
        ] {
            assert!(AuthorDate::parse(refused).is_none(), "{refused:?}");
        }
    }

    #[test]
    fn a_document_log_reads_each_record_with_its_paths() {
        let stdout = format!(
            "\x1e{A}\x1f2026-05-02T10:00:00+00:00\x1fsecond\n\ndocs/a.md\n\
             \x1e{B}\x1f2026-05-01T10:00:00+00:00\x1fa merge\n\
             \x1e{A}\x1f2026-04-01T10:00:00+00:00\x1ffirst\n\ndocs/old.md\n\"docs/\\303\\251\\t.md\"\n"
        );
        let entries = run_doc(stdout.as_bytes()).unwrap();
        // The merge that lists no path changed nothing in the document.
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].subject(), "second");
        assert_eq!(entries[0].paths(), ["docs/a.md"]);
        assert_eq!(entries[1].subject(), "first");
        assert_eq!(entries[1].paths(), ["docs/old.md", "docs/\u{e9}\t.md"]);
        assert_eq!(run_doc(b"").unwrap(), []);
    }

    #[test]
    fn a_subject_keeps_every_byte_but_a_newline() {
        let subject = format!("a\x1fb\x1e{A}\x1f2026-01-01T00:00:00Z\x1f| x \u{1b}[31m");
        let stdout = format!("\x1e{A}\x1f2026-05-02T10:00:00Z\x1f{subject}\n\ndocs/a.md\n");
        let entries = run_doc(stdout.as_bytes()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].subject(), subject);
        assert_eq!(entries[0].commit().as_str(), A);
        // Invalid UTF-8 is shown as U+FFFD, the rest kept.
        let mut bytes = format!("\x1e{A}\x1f2026-05-02T10:00:00Z\x1fbad ").into_bytes();
        bytes.extend([0xff, b'!', b'\n']);
        bytes.extend(b"\ndocs/a.md\n");
        assert_eq!(run_doc(&bytes).unwrap()[0].subject(), "bad \u{fffd}!");
    }

    #[test]
    fn output_that_is_not_a_log_of_the_form_is_an_error() {
        for bad in [
            "docs/a.md\n".to_string(),
            "garbage before\n\x1e".to_string(),
            format!("\x1e{A}\x1f2026-05-02T10:00:00Z\n"),
            "\x1eHEAD\x1f2026-05-02T10:00:00Z\x1fs\n".to_string(),
            format!("\x1e{A}\x1fyesterday\x1fs\n"),
            format!("\x1e{A}\x1f2026-05-02T10:00:00Z\x1fs\n\n\"unterminated\\\"\n"),
            format!("\x1e{A}\x1f2026-05-02T10:00:00Z\x1fs\n\n\"bad \\q\"\n"),
        ] {
            let error = run_doc(bad.as_bytes()).expect_err(&bad);
            assert!(
                matches!(error, GitError::Answer { .. }),
                "{bad:?}: {error:?}"
            );
        }
        let mut bytes = format!("\x1e{A}\x1f2026-05-02T10:00:00Z\x1fs\n\n").into_bytes();
        bytes.extend([b'a', 0xff, b'\n']);
        assert!(matches!(run_doc(&bytes), Err(GitError::NotUtf8 { .. })));
    }

    #[test]
    fn a_symbol_log_lists_no_path_and_refuses_any_other_line() {
        let lines = LineRange::new(3, 9).unwrap();
        let owned = Invocation::git_symbol_log(lines, &path("src/lib.rs"), None).argv();
        let args: Vec<&str> = owned.iter().map(String::as_str).collect();
        let good = format!("\x1e{A}\x1f2026-05-02T10:00:00Z\x1fs\n");
        let runner = FakeRunner::new().on(Program::Git, &args, ok(good.as_bytes()));
        let entries = symbol_log(&runner, lines, &path("src/lib.rs"), None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].paths(), [] as [String; 0]);
        // A patch printed despite `--no-patch` is not read as a path.
        let patch = format!("{good}\ndiff --git a/src/lib.rs b/src/lib.rs\n");
        let runner = FakeRunner::new().on(Program::Git, &args, ok(patch.as_bytes()));
        assert!(matches!(
            symbol_log(&runner, lines, &path("src/lib.rs"), None),
            Err(GitError::Answer { .. })
        ));
    }

    #[test]
    fn a_log_past_the_bound_is_an_error_that_says_so() {
        let owned = doc_argv("docs/a.md");
        let args: Vec<&str> = owned.iter().map(String::as_str).collect();
        let big = vec![b'a'; crate::exec::DEFAULT_OUTPUT_BOUND + 1];
        let runner = FakeRunner::new().on(Program::Git, &args, ok(&big));
        let error = document_log(&runner, &path("docs/a.md"), None).expect_err("too large");
        assert!(
            matches!(&error, GitError::LogTooLarge { limit, .. } if *limit == crate::exec::DEFAULT_OUTPUT_BOUND),
            "{error:?}"
        );
        assert!(error.to_string().contains("16777216"), "{error}");
        // The neighbour: exactly the bound is read.
        let mut exact = format!("\x1e{A}\x1f2026-05-02T10:00:00Z\x1f").into_bytes();
        let tail = b"\n\ndocs/a.md\n";
        exact.resize(crate::exec::DEFAULT_OUTPUT_BOUND - tail.len(), b's');
        exact.extend(tail);
        let runner = FakeRunner::new().on(Program::Git, &args, ok(&exact));
        assert_eq!(
            document_log(&runner, &path("docs/a.md"), None)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn the_pickaxe_answers_one_record_with_its_parent_or_none() {
        let id = Identifier::new("gone").unwrap();
        let dirs = [path("crates/a")];
        let owned = Invocation::git_pickaxe(&id, &dirs).argv();
        let args: Vec<&str> = owned.iter().map(String::as_str).collect();
        let answer = |stdout: &str| {
            let runner = FakeRunner::new().on(Program::Git, &args, ok(stdout.as_bytes()));
            last_change(&runner, &id, &dirs)
        };
        let one = format!("{A}\x1f{B}\x1f2026-05-02T10:00:00Z\x1fremove gone\x1f!\n");
        let found = answer(&one).unwrap().unwrap();
        assert_eq!(found.entry().commit().as_str(), A);
        assert_eq!(found.entry().subject(), "remove gone\x1f!");
        assert_eq!(found.parent().map(Commit::as_str), Some(B));
        // A merge: the first parent is the one reported.
        let merge = answer(&format!("{A}\x1f{B} {A}\x1f2026-05-02T10:00:00Z\x1fm\n"))
            .unwrap()
            .unwrap();
        assert_eq!(merge.parent().map(Commit::as_str), Some(B));
        // The first commit of the history has no parent.
        let root = answer(&format!("{A}\x1f\x1f2026-05-02T10:00:00Z\x1froot\n"))
            .unwrap()
            .unwrap();
        assert_eq!(root.parent(), None);
        assert_eq!(answer("").unwrap(), None);
        for bad in [
            format!("{one}{one}"),
            format!("{A}\x1fHEAD\x1f2026-05-02T10:00:00Z\x1fs\n"),
            format!("{A}\x1f{B}  {A}\x1f2026-05-02T10:00:00Z\x1fs\n"),
            format!("{A}\x1f{B}\n"),
            format!("{A}\x1f{B}\x1fnot a date\x1fs\n"),
        ] {
            assert!(
                matches!(answer(&bad), Err(GitError::Answer { .. })),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn the_shallow_answer_is_true_or_false() {
        for (answer, expected) in [
            ("true\n", Some(true)),
            ("false\n", Some(false)),
            ("yes\n", None),
        ] {
            let runner = FakeRunner::new().on(
                Program::Git,
                &["rev-parse", "--is-shallow-repository"],
                ok(answer.as_bytes()),
            );
            assert_eq!(is_shallow(&runner).ok(), expected, "{answer}");
        }
    }
}
