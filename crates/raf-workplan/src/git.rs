// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The questions the workplan asks of `git`, each one a typed
//! [`crate::exec::Invocation`] given to a [`Runner`].
//!
//! A [`Runner`] holds the program and the directory it runs in
//! ([`Programs`](crate::exec::Programs)), so every function here asks about the
//! repository that directory is in. Output that is parsed is read in the C
//! locale ([`Limits::query`]).
//!
//! * [`require_version`] refuses a Git older than [`GitVersion::MINIMUM`], the
//!   first release whose `rev-parse` handles `--end-of-options`.
//! * [`show_prefix`] says where the directory is below the top of its work
//!   tree, because `git cat-file` names a path from the top.
//! * [`tracked_paths`] lists the paths of the index, and says when the
//!   directory is not in a work tree at all.
//! * [`default_branch_ref`], [`resolve_commit`], [`merge_base`] and [`blob_at`]
//!   ([`default_merge_base`] runs the first three) are the four commands that read a file as it was at the merge base of the
//!   current branch and the default branch. The default branch is the full
//!   reference name `git symbolic-ref refs/remotes/origin/HEAD` prints, so a local
//!   branch named `origin/main` is never mistaken for it. It is resolved to a
//!   commit id once, and every later command receives that id, so a branch named
//!   like an option cannot reach `git` as an option. A path follows
//!   `<merge-base>:` inside one argument.
//!
//! * [`is_shallow`], [`document_log`], [`symbol_log`] and [`last_change`] are
//!   the questions of the history command: whether the repository is a
//!   shallow clone, the commits of a document through its moves, the commits
//!   that changed the lines of a symbol, and the newest commit that changed
//!   how often an identifier occurs in a crate, with its parent. A path
//!   reaches `git` after `--` or after the colon of one `-L` argument, an
//!   identifier inside one `-S` argument and a date inside one `--since`
//!   argument.
//!
//! Every failure is a [`GitError`]. None is a skip: a repository whose default
//! branch is unknown, whose history has no merge base, or whose file is larger
//! than the document bound has no answer.

use std::error::Error;
use std::fmt;

use raf_okf::MAX_TEXT_BYTES;
use raf_source::RelPath;

use crate::exec::{
    Commit, ExecError, Invocation, Limits, Output, PatternError, RefName, Runner, SourcePath,
    Stream,
};

/// A version of `git`: `major.minor.patch`.
///
/// Vendor suffixes are tolerated, so `git version 2.45.1.windows.1` and
/// `git version 2.39.5 (Apple Git-154)` are 2.45.1 and 2.39.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GitVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl GitVersion {
    /// The oldest Git the workplan runs with: 2.30, the first release whose
    /// `rev-parse` handles `--end-of-options`.
    pub const MINIMUM: Self = Self {
        major: 2,
        minor: 30,
        patch: 0,
    };

    /// The version the output of `git --version` states.
    ///
    /// The text is `git version <major>.<minor>[.<patch>...]` and anything a
    /// vendor adds after the first space or after the patch number. The major
    /// and the minor number are required and are digits only; a patch number
    /// that is absent, or does not begin with a digit, is 0.
    ///
    /// # Errors
    ///
    /// [`GitError::VersionUnreadable`] when the text is not of that form.
    pub fn parse(text: &str) -> Result<Self, GitError> {
        let unreadable = || GitError::VersionUnreadable {
            output: shown(text),
        };
        let line = text.lines().next().unwrap_or("").trim();
        let rest = line.strip_prefix("git version ").ok_or_else(unreadable)?;
        let token = rest.split_whitespace().next().ok_or_else(unreadable)?;
        let mut parts = token.split('.');
        let major = whole_number(parts.next()).ok_or_else(unreadable)?;
        let minor = whole_number(parts.next()).ok_or_else(unreadable)?;
        let patch = parts.next().map_or(0, leading_number);
        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    /// The major version.
    #[must_use]
    pub fn major(&self) -> u32 {
        self.major
    }

    /// The minor version.
    #[must_use]
    pub fn minor(&self) -> u32 {
        self.minor
    }

    /// The patch version.
    #[must_use]
    pub fn patch(&self) -> u32 {
        self.patch
    }
}

impl fmt::Display for GitVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// The merge base of the current commit and the default branch, as
/// [`merge_base`] returns it.
///
/// It is the only revision [`blob_at`] reads a file at. There is no
/// constructor: a value comes from [`merge_base`] (or [`default_merge_base`])
/// and from nowhere else, so a revision a caller made up, or took from a
/// document, cannot be asked about. `git` answers a commit it does not have in
/// the words it uses for a missing path, which would read as "the file is
/// absent"; this type makes that question impossible to ask.
///
/// ```compile_fail,E0308
/// use raf_workplan::exec::{Commit, SourcePath};
/// use raf_workplan::git::{blob_at};
///
/// fn made_up(runner: &dyn raf_workplan::exec::Runner) {
///     let rev = Commit::new("0123456789abcdef0123456789abcdef01234567").unwrap();
///     let path = SourcePath::new("docs/capabilities/x.md").unwrap();
///     // A `Commit` is not a `MergeBase`.
///     let _ = blob_at(runner, &rev, &path);
/// }
/// ```
///
/// ```compile_fail,E0423
/// use raf_workplan::exec::Commit;
/// use raf_workplan::git::MergeBase;
///
/// let rev = Commit::new("0123456789abcdef0123456789abcdef01234567").unwrap();
/// let _ = MergeBase(rev);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeBase(Commit);

impl MergeBase {
    /// The commit id.
    #[must_use]
    pub fn commit(&self) -> &Commit {
        &self.0
    }

    /// The commit id as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for MergeBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Where the working directory is below the top of its work tree, as
/// `git rev-parse --show-prefix` prints it: empty at the top, and otherwise
/// the directories from the top with a `/` after each, such as `crates/app/`.
///
/// `git cat-file blob <rev>:<path>` reads a path relative to the top of the
/// work tree, not to the working directory, so a workspace below the top of a
/// repository (a monorepo) reads its files at the prefix. [`Prefix::join`] puts
/// the prefix before the path of a document. A prefix is validated by the
/// pattern of a relative source path, so one with a space, a non-ASCII letter,
/// a `.` or `..` segment or any other character outside that pattern is an
/// error and not a guess.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Prefix(String);

impl Prefix {
    /// The prefix as text: empty, or ending in `/`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether the working directory is the top of its work tree.
    #[must_use]
    pub fn is_top(&self) -> bool {
        self.0.is_empty()
    }

    /// The path of a file of the workspace as the work tree names it: the
    /// prefix and then `path`.
    ///
    /// # Errors
    ///
    /// The [`PatternError`] of the joined path, which only a path of the
    /// workspace that is too unusual to be a relative source path can cause.
    pub fn join(&self, path: &SourcePath) -> Result<SourcePath, PatternError> {
        SourcePath::new(&format!("{}{path}", self.0))
    }
}

/// A component that is all digits.
fn whole_number(part: Option<&str>) -> Option<u32> {
    let part = part?;
    if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    part.parse().ok()
}

/// The digits a component begins with, or 0 when it does not begin with one.
fn leading_number(part: &str) -> u32 {
    let end = part
        .bytes()
        .position(|byte| !byte.is_ascii_digit())
        .unwrap_or(part.len());
    part[..end].parse().unwrap_or(0)
}

/// Why a question to `git` has no answer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GitError {
    /// `git` did not run to a result: it could not be started, ran too long,
    /// wrote too much, or could not be read.
    Exec(ExecError),
    /// `git` ran and exited with a failure the question does not expect.
    Failed {
        /// The invocation, as text for the message.
        command: String,
        /// The exit code, or `None` when a signal ended the process.
        code: Option<i32>,
        /// What it wrote to standard error.
        stderr: String,
    },
    /// `git` answered with text that is not what the question allows.
    Answer {
        /// The invocation, as text for the message.
        command: String,
        /// What was expected.
        expected: &'static str,
        /// What it answered, cut for the message.
        answer: String,
    },
    /// `GIT_DIR` or `GIT_WORK_TREE` is set and names a repository `git`
    /// cannot use.
    Environment {
        /// The variable.
        name: &'static str,
        /// What `git` said.
        message: String,
    },
    /// A tracked path is not UTF-8.
    NotUtf8 {
        /// Why.
        reason: String,
    },
    /// A tracked path is not a relative path of the tree.
    NotAPath {
        /// The path, as text.
        path: String,
        /// Why not.
        reason: String,
    },
    /// The output of `git --version` is not a version.
    VersionUnreadable {
        /// What it said, cut for the message.
        output: String,
    },
    /// `git` is older than [`GitVersion::MINIMUM`].
    VersionTooOld {
        /// The version found.
        found: GitVersion,
    },
    /// `refs/remotes/origin/HEAD` does not name a branch, so the repository has
    /// no default branch to compare with.
    NoDefaultBranch {
        /// What `git` said.
        stderr: String,
    },
    /// The current commit and the default branch have no common ancestor.
    NoMergeBase {
        /// The commit of the default branch.
        default: Commit,
    },
    /// A log printed more than the bound on its output.
    LogTooLarge {
        /// The invocation, as text for the message.
        command: String,
        /// The bound, in bytes.
        limit: usize,
    },
    /// A file at the merge base is larger than the document bound.
    BlobTooLarge {
        /// The repository-relative path.
        path: SourcePath,
        /// The bound, in bytes.
        limit: usize,
    },
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exec(error) => write!(f, "{error}"),
            Self::Failed {
                command,
                code,
                stderr,
            } => {
                write!(f, "`{command}` failed")?;
                match code {
                    Some(code) => write!(f, " (exit {code})")?,
                    None => write!(f, " (ended by a signal)")?,
                }
                if !stderr.is_empty() {
                    write!(f, ": {stderr}")?;
                }
                Ok(())
            }
            Self::Answer {
                command,
                expected,
                answer,
            } => write!(
                f,
                "`{command}` answered `{}`, which is not {expected}",
                answer.escape_debug()
            ),
            Self::Environment { name, message } => {
                write!(f, "{name} is set and `git` cannot use it: {message}")
            }
            Self::NotUtf8 { reason } => write!(f, "a tracked path is not UTF-8: {reason}"),
            Self::NotAPath { path, reason } => write!(f, "`{path}` is not a path: {reason}"),
            Self::VersionUnreadable { output } => write!(
                f,
                "`git --version` answered `{}`, which is not `git version <major>.<minor>...`; install a Git of {} or newer",
                output.escape_debug(),
                GitVersion::MINIMUM
            ),
            Self::VersionTooOld { found } => write!(
                f,
                "git {found} is older than {}, the first release whose `rev-parse` handles `--end-of-options`; install Git {} or newer",
                GitVersion::MINIMUM,
                GitVersion::MINIMUM
            ),
            Self::NoDefaultBranch { stderr } => {
                write!(
                    f,
                    "the repository has no default branch: `git symbolic-ref refs/remotes/origin/HEAD` failed"
                )?;
                if !stderr.is_empty() {
                    write!(f, " ({stderr})")?;
                }
                write!(
                    f,
                    "; the remote is assumed to be named `origin`, and `git remote set-head origin --auto` sets its default branch"
                )
            }
            Self::NoMergeBase { default } => write!(
                f,
                "the current commit and the default branch ({default}) have no common ancestor, so there is no merge base to compare with"
            ),
            Self::LogTooLarge { command, limit } => write!(
                f,
                "`{command}` printed more than {limit} bytes, the most of a log that is read; no part of it is used, and `--since` narrows it"
            ),
            Self::BlobTooLarge { path, limit } => write!(
                f,
                "`{path}` at the merge base is larger than {limit} bytes, the bound on a document; it is not read"
            ),
        }
    }
}

impl Error for GitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Exec(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ExecError> for GitError {
    fn from(error: ExecError) -> Self {
        Self::Exec(error)
    }
}

/// `text` cut to a length a message can show.
fn shown(text: &str) -> String {
    const SHOWN: usize = 80;
    let mut cut: String = text.chars().take(SHOWN).collect();
    if text.chars().nth(SHOWN).is_some() {
        cut.push_str("...");
    }
    cut
}

/// Run one invocation under the query limits.
fn query(runner: &dyn Runner, invocation: &Invocation) -> Result<Output, GitError> {
    Ok(runner.run(invocation, &Limits::query())?)
}

/// The failure of an invocation that exited with a failing code.
fn failed(invocation: &Invocation, output: &Output) -> GitError {
    GitError::Failed {
        command: invocation.to_string(),
        code: output.code,
        stderr: output.stderr_text(),
    }
}

/// The one line `git` answered, without its line ending.
fn one_line<'o>(
    invocation: &Invocation,
    output: &'o Output,
    expected: &'static str,
) -> Result<&'o str, GitError> {
    let answer = |text: &str| GitError::Answer {
        command: invocation.to_string(),
        expected,
        answer: shown(text),
    };
    let text = std::str::from_utf8(&output.stdout)
        .map_err(|_| answer(&String::from_utf8_lossy(&output.stdout)))?;
    let line = text.strip_suffix('\n').unwrap_or(text);
    Ok(line.strip_suffix('\r').unwrap_or(line))
}

/// The answer, when it matches the pattern of the type it becomes.
fn typed<T>(
    invocation: &Invocation,
    line: &str,
    expected: &'static str,
    build: impl FnOnce(&str) -> Result<T, PatternError>,
) -> Result<T, GitError> {
    build(line).map_err(|_| GitError::Answer {
        command: invocation.to_string(),
        expected,
        answer: shown(line),
    })
}

/// The version of the `git` the runner starts, when it is
/// [`GitVersion::MINIMUM`] or newer.
///
/// Run this before any other `git` question that depends on the options of a
/// recent Git: it is the check that makes `--end-of-options` safe.
///
/// # Errors
///
/// [`GitError::VersionTooOld`] for a Git older than 2.30 (2.29 is refused and
/// 2.30 is accepted); [`GitError::VersionUnreadable`] when the output of
/// `git --version` is not a version; [`GitError::Failed`] when the command
/// fails; [`GitError::Exec`] when it cannot run.
pub fn require_version(runner: &dyn Runner) -> Result<GitVersion, GitError> {
    let invocation = Invocation::git_version();
    let output = query(runner, &invocation)?;
    if !output.success() {
        return Err(failed(&invocation, &output));
    }
    let text = std::str::from_utf8(&output.stdout).map_err(|_| GitError::VersionUnreadable {
        output: shown(&String::from_utf8_lossy(&output.stdout)),
    })?;
    let found = GitVersion::parse(text)?;
    if found < GitVersion::MINIMUM {
        return Err(GitError::VersionTooOld { found });
    }
    Ok(found)
}

/// The path of the runner's directory below the top of its work tree:
/// `git rev-parse --show-prefix`. A linked work tree is its own top, so its
/// prefix is empty.
///
/// # Errors
///
/// [`GitError::Failed`] when the command fails, as it does outside a work
/// tree; [`GitError::Answer`] when the answer is not empty and not a relative
/// directory path of the source-path pattern ending in `/`, so a directory that
/// the pattern cannot carry is refused; [`GitError::Exec`] when it cannot run.
pub fn show_prefix(runner: &dyn Runner) -> Result<Prefix, GitError> {
    let invocation = Invocation::git_show_prefix();
    let output = query(runner, &invocation)?;
    if !output.success() {
        return Err(failed(&invocation, &output));
    }
    let line = one_line(&invocation, &output, "a directory path ending in `/`")?;
    if line.is_empty() {
        return Ok(Prefix::default());
    }
    let valid = line
        .strip_suffix('/')
        .is_some_and(|directory| SourcePath::new(directory).is_ok());
    if valid {
        Ok(Prefix(line.to_string()))
    } else {
        Err(GitError::Answer {
            command: invocation.to_string(),
            expected: "a directory path of letters, digits, `_`, `.` and `-` ending in `/`",
            answer: shown(line),
        })
    }
}

/// The tracked paths of the work tree the runner's directory is in, relative
/// to that directory, as `git ls-files -z` lists them.
///
/// `None` is returned in exactly one case: `git` runs and answers that the
/// directory is not inside a git work tree (it is outside any repository, or
/// inside a `.git` directory). Every other failure is an error: `git` cannot
/// run, `git` fails for another reason, `GIT_DIR` or `GIT_WORK_TREE` names a
/// repository that cannot be used, `git ls-files` fails inside a work tree, or
/// a path is not UTF-8 or not a relative path.
///
/// # Errors
///
/// A [`GitError`]: [`GitError::Exec`], [`GitError::Environment`],
/// [`GitError::Answer`], [`GitError::Failed`], [`GitError::NotUtf8`] or
/// [`GitError::NotAPath`].
pub fn tracked_paths(runner: &dyn Runner) -> Result<Option<Vec<RelPath>>, GitError> {
    let inside = Invocation::git_inside_work_tree();
    let answer = query(runner, &inside)?;
    if answer.success() {
        match answer.stdout.trim_ascii() {
            b"true" => {}
            b"false" => return Ok(None),
            other => {
                return Err(GitError::Answer {
                    command: inside.to_string(),
                    expected: "`true` or `false`",
                    answer: shown(&String::from_utf8_lossy(other)),
                });
            }
        }
    } else {
        let message = answer.stderr_text();
        let environment = ["GIT_DIR", "GIT_WORK_TREE"]
            .into_iter()
            .find(|name| std::env::var_os(name).is_some());
        return match environment {
            Some(name) => Err(GitError::Environment { name, message }),
            None if message.contains("not a git repository") => Ok(None),
            None => Err(failed(&inside, &answer)),
        };
    }
    let list = Invocation::git_ls_files();
    let output = query(runner, &list)?;
    if !output.success() {
        return Err(failed(&list, &output));
    }
    let mut paths = Vec::new();
    for name in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
    {
        let text = std::str::from_utf8(name).map_err(|error| GitError::NotUtf8 {
            reason: error.to_string(),
        })?;
        paths.push(RelPath::new(text).map_err(|error| GitError::NotAPath {
            path: text.to_string(),
            reason: error.to_string(),
        })?);
    }
    Ok(Some(paths))
}

/// The default branch of the remote `origin`, as the full reference name
/// `git symbolic-ref refs/remotes/origin/HEAD` prints, such as
/// `refs/remotes/origin/main`.
///
/// # Errors
///
/// [`GitError::NoDefaultBranch`] when the command fails, which names
/// `git remote set-head origin --auto`; [`GitError::Answer`] when its answer is
/// not a full reference name; [`GitError::Exec`] when it cannot run.
pub fn default_branch_ref(runner: &dyn Runner) -> Result<RefName, GitError> {
    let invocation = Invocation::git_default_ref();
    let output = query(runner, &invocation)?;
    if !output.success() {
        return Err(GitError::NoDefaultBranch {
            stderr: output.stderr_text(),
        });
    }
    let line = one_line(&invocation, &output, "a full reference name")?;
    typed(&invocation, line, "a full reference name", RefName::new)
}

/// The commit a reference names: `git rev-parse --verify --end-of-options
/// <ref>^{commit}`.
///
/// # Errors
///
/// [`GitError::Failed`] when the reference does not name a commit;
/// [`GitError::Answer`] when the answer is not a commit id; [`GitError::Exec`]
/// when the command cannot run.
pub fn resolve_commit(runner: &dyn Runner, reference: &RefName) -> Result<Commit, GitError> {
    let invocation = Invocation::git_resolve(reference);
    let output = query(runner, &invocation)?;
    if !output.success() {
        return Err(failed(&invocation, &output));
    }
    let line = one_line(&invocation, &output, "a commit id")?;
    typed(&invocation, line, "a commit id", Commit::new)
}

/// The merge base of the current commit (`HEAD`) and `default`:
/// `git merge-base HEAD <default>`. On the default branch itself the merge base
/// is `HEAD`.
///
/// # Errors
///
/// [`GitError::NoMergeBase`] when the two have no common ancestor;
/// [`GitError::Failed`] when the command fails otherwise; [`GitError::Answer`]
/// when the answer is not one commit id; [`GitError::Exec`] when it cannot run.
pub fn merge_base(runner: &dyn Runner, default: &Commit) -> Result<MergeBase, GitError> {
    let invocation = Invocation::git_merge_base(default);
    let output = query(runner, &invocation)?;
    match output.code {
        Some(0) => {}
        Some(1) => {
            return Err(GitError::NoMergeBase {
                default: default.clone(),
            });
        }
        _ => return Err(failed(&invocation, &output)),
    }
    let line = one_line(&invocation, &output, "one commit id")?;
    typed(&invocation, line, "one commit id", Commit::new).map(MergeBase)
}

/// The merge base of the current commit and the default branch of the remote
/// `origin`: the three questions that lead to it in order,
/// [`default_branch_ref`], [`resolve_commit`] and [`merge_base`]. The commit
/// id the second question returns is the only thing the third receives.
///
/// # Errors
///
/// The [`GitError`] of the first question that fails; a repository with no
/// default branch is [`GitError::NoDefaultBranch`], which names
/// `git remote set-head origin --auto`.
pub fn default_merge_base(runner: &dyn Runner) -> Result<MergeBase, GitError> {
    let reference = default_branch_ref(runner)?;
    let default = resolve_commit(runner, &reference)?;
    merge_base(runner, &default)
}

/// The bytes of the file `path` at the merge base `rev`: `git cat-file blob
/// <rev>:<path>`. The file is read up to the document bound of 1 MiB
/// ([`MAX_TEXT_BYTES`]); a larger file is an error and is never cut.
///
/// `None` is returned when `git` says the path does not exist at `rev`, so the
/// caller can tell a file that is new from a failure. Every other failure is an
/// error, including a path that names a directory. `git` says the same of a
/// commit it does not have, which is why `rev` is a [`MergeBase`], a value only
/// [`merge_base`] makes. `path` is named from the top of the work tree (see
/// [`Prefix::join`]).
///
/// # Errors
///
/// [`GitError::BlobTooLarge`] for a file over the bound; [`GitError::Failed`]
/// when the command fails otherwise; [`GitError::Exec`] when it cannot run.
pub fn blob_at(
    runner: &dyn Runner,
    rev: &MergeBase,
    path: &SourcePath,
) -> Result<Option<Vec<u8>>, GitError> {
    let invocation = Invocation::git_blob(rev.commit(), path);
    let limits = Limits::query().with_max_output(MAX_TEXT_BYTES);
    let output = match runner.run(&invocation, &limits) {
        Ok(output) => output,
        Err(ExecError::OutputTooLarge {
            stream: Stream::Stdout,
            ..
        }) => {
            return Err(GitError::BlobTooLarge {
                path: path.clone(),
                limit: MAX_TEXT_BYTES,
            });
        }
        Err(error) => return Err(error.into()),
    };
    if output.success() {
        return Ok(Some(output.stdout));
    }
    // The words are git's English ones. The query limits set `LC_ALL=C`, which
    // is what keeps them stable; a runner that stopped setting it would turn an
    // absent file into an error here (never into an absent file), because
    // anything that is not these two messages is an error.
    let stderr = output.stderr_text();
    let absent = stderr.starts_with("fatal: path '")
        && (stderr.contains("' does not exist in '")
            || stderr.contains("' exists on disk, but not in '"));
    if absent {
        Ok(None)
    } else {
        Err(failed(&invocation, &output))
    }
}

mod log;

pub use log::{
    AuthorDate, LastChange, LogEntry, document_log, is_shallow, last_change, symbol_log,
};

#[cfg(test)]
mod tests;
