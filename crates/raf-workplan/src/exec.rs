// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The one place a process is started.
//!
//! Every `cargo` and `git` process of the workplan is an [`Invocation`] given to
//! a [`Runner`]. The invocation is a [`Program`] and a list of [`Arg`]s; an
//! argument is fixed text or a validated value, and a value can be built only
//! through one of the patterns of the completion gate (see [`Pattern`]), so text
//! from a document cannot reach a command line by any other route.
//!
//! [`SystemRunner`] starts the process directly with no shell and the argv
//! vector as given, with its working directory set to [`Programs::cwd`] and its
//! standard input null. The environment is the caller's, plus `LC_ALL=C` for a
//! process whose output is parsed ([`Limits::c_locale`]). Standard output and
//! standard error are captured, each up to a bound; a process that writes more,
//! or runs longer than its wall-clock limit, is killed and the run is an
//! [`ExecError`]. On Unix the process leads a process group of its own and a
//! kill (SIGKILL to the group, sent directly and not through a `kill` program)
//! reaches the group, so a test binary that `cargo` started does not outlive its
//! timeout. A descendant that leaves the group, as one that calls `setsid` or
//! `setpgid` does, is not reached and may outlive the kill; elsewhere than Unix
//! only the process itself is killed. A kill that cannot be sent is an
//! [`ExecError::Io`] that says the process may still be running.
//!
//! [`FakeRunner`] answers invocations from a script, in process and on every
//! platform, and records what it was asked.
//!
//! ```rust
//! use raf_workplan::exec::{FakeRunner, Invocation, Limits, Output, Program, Runner};
//!
//! let runner = FakeRunner::new().on(
//!     Program::Git,
//!     &["--version"],
//!     Ok(Output::new(Some(0), b"git version 2.45.1\n".to_vec(), Vec::new())),
//! );
//! let output = runner.run(&Invocation::git_version(), &Limits::query())?;
//! assert!(output.success());
//! assert_eq!(runner.calls()[0].command(), "git --version");
//! # Ok::<(), raf_workplan::exec::ExecError>(())
//! ```

mod fake;
mod invocation;
mod pattern;
mod system;

use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub use fake::{Call, FakeRunner};
pub use invocation::{Arg, Invocation, Literal, Program};
pub use pattern::{
    Commit, DocFilter, ExampleName, Filter, Package, Pattern, PatternError, RefName, SourcePath,
    Target, TargetName,
};
pub use system::SystemRunner;

/// The bound on each of standard output and standard error: 16 MiB.
pub const DEFAULT_OUTPUT_BOUND: usize = 16 * 1024 * 1024;

/// The bound on the output of `cargo metadata`: 64 MiB.
pub const METADATA_OUTPUT_BOUND: usize = 64 * 1024 * 1024;

/// The wall-clock limit of a `cargo test` or `cargo run` that may compile
/// first: 1800 seconds.
pub const BUILD_TIMEOUT: Duration = Duration::from_mins(30);

/// The wall-clock limit of the run of an example: 300 seconds.
pub const EXAMPLE_TIMEOUT: Duration = Duration::from_mins(5);

/// The wall-clock limit of a `git` or `cargo metadata` query: 300 seconds.
pub const QUERY_TIMEOUT: Duration = Duration::from_mins(5);

/// The programs to start and the directory to start them in.
///
/// [`Programs::new`] takes `cargo` from `$CARGO` (cargo sets it for a
/// subcommand) and from `PATH` as `cargo` otherwise, and `git` from `PATH`. Each
/// can be replaced, which is how a test substitutes a stub.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Programs {
    /// The `cargo` to start: a path, or a bare name looked up on `PATH`.
    pub cargo: PathBuf,
    /// The `git` to start: a path, or a bare name looked up on `PATH`.
    pub git: PathBuf,
    /// The working directory of every process.
    pub cwd: PathBuf,
}

impl Programs {
    /// The programs of this process's environment, run in `cwd`.
    #[must_use]
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self::with_cargo_var(cwd, std::env::var_os("CARGO"))
    }

    /// As [`Programs::new`] with the value of `$CARGO` given, `None` for unset.
    /// A value that is set but empty is kept, and starting it fails.
    #[must_use]
    pub fn with_cargo_var(cwd: impl Into<PathBuf>, cargo: Option<OsString>) -> Self {
        Self {
            cargo: cargo.map_or_else(|| PathBuf::from("cargo"), PathBuf::from),
            git: PathBuf::from("git"),
            cwd: cwd.into(),
        }
    }

    /// Replace the `cargo` program.
    #[must_use]
    pub fn with_cargo(mut self, cargo: impl Into<PathBuf>) -> Self {
        self.cargo = cargo.into();
        self
    }

    /// Replace the `git` program.
    #[must_use]
    pub fn with_git(mut self, git: impl Into<PathBuf>) -> Self {
        self.git = git.into();
        self
    }

    /// The program an invocation starts.
    #[must_use]
    pub fn path_of(&self, program: Program) -> &Path {
        match program {
            Program::Cargo => &self.cargo,
            Program::Git => &self.git,
        }
    }
}

/// What bounds a run: how much output is kept, how long it may take, and
/// whether its output is parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    max_output: usize,
    timeout: Duration,
    c_locale: bool,
}

impl Limits {
    /// These bounds. `c_locale` sets `LC_ALL=C` in the process, which a
    /// caller that parses the output needs and one that does not leaves off.
    #[must_use]
    pub fn new(max_output: usize, timeout: Duration, c_locale: bool) -> Self {
        Self {
            max_output,
            timeout,
            c_locale,
        }
    }

    /// A `git` query: 16 MiB per stream, 300 seconds, `LC_ALL=C`.
    #[must_use]
    pub fn query() -> Self {
        Self::new(DEFAULT_OUTPUT_BOUND, QUERY_TIMEOUT, true)
    }

    /// `cargo metadata`: 64 MiB per stream, 300 seconds, `LC_ALL=C`.
    #[must_use]
    pub fn metadata() -> Self {
        Self::new(METADATA_OUTPUT_BOUND, QUERY_TIMEOUT, true)
    }

    /// A `cargo test` whose output is parsed: 16 MiB per stream, 1800 seconds,
    /// `LC_ALL=C`.
    #[must_use]
    pub fn build() -> Self {
        Self::new(DEFAULT_OUTPUT_BOUND, BUILD_TIMEOUT, true)
    }

    /// The run of an example, judged by its exit status alone: 16 MiB per
    /// stream, 300 seconds, the caller's locale.
    #[must_use]
    pub fn example() -> Self {
        Self::new(DEFAULT_OUTPUT_BOUND, EXAMPLE_TIMEOUT, false)
    }

    /// The same limits with another output bound.
    #[must_use]
    pub fn with_max_output(mut self, max_output: usize) -> Self {
        self.max_output = max_output;
        self
    }

    /// The same limits with another wall-clock limit.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// The most bytes kept of each of standard output and standard error.
    #[must_use]
    pub fn max_output(&self) -> usize {
        self.max_output
    }

    /// The wall-clock limit.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Whether the process runs with `LC_ALL=C`.
    #[must_use]
    pub fn c_locale(&self) -> bool {
        self.c_locale
    }
}

/// The result of a process that ran to its end.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Output {
    /// The exit code, or `None` when a signal ended the process.
    pub code: Option<i32>,
    /// What it wrote to standard output.
    pub stdout: Vec<u8>,
    /// What it wrote to standard error.
    pub stderr: Vec<u8>,
    /// How long it ran.
    pub elapsed: Duration,
}

impl Output {
    /// An output with no elapsed time.
    #[must_use]
    pub fn new(code: Option<i32>, stdout: Vec<u8>, stderr: Vec<u8>) -> Self {
        Self {
            code,
            stdout,
            stderr,
            elapsed: Duration::ZERO,
        }
    }

    /// The same output with this elapsed time.
    #[must_use]
    pub fn with_elapsed(mut self, elapsed: Duration) -> Self {
        self.elapsed = elapsed;
        self
    }

    /// Whether the process exited with code 0.
    #[must_use]
    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}

/// One of the two captured streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Stream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

impl fmt::Display for Stream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Stdout => "standard output",
            Self::Stderr => "standard error",
        })
    }
}

/// Why a process did not run to a result. Each case is an error and none is a
/// skip: a process that could not be started, ran too long or wrote too much
/// has no verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExecError {
    /// The process could not be started.
    Spawn {
        /// The invocation, as text for the message.
        command: String,
        /// The program that was looked for.
        program: String,
        /// The kind of the operating-system error.
        kind: io::ErrorKind,
        /// The operating-system error.
        message: String,
    },
    /// The process ran past its wall-clock limit and was killed.
    Timeout {
        /// The invocation, as text for the message.
        command: String,
        /// The limit.
        timeout: Duration,
        /// How long it had run when it was killed.
        elapsed: Duration,
    },
    /// The process wrote more than the bound to one stream and was killed.
    OutputTooLarge {
        /// The invocation, as text for the message.
        command: String,
        /// The stream that overflowed.
        stream: Stream,
        /// The bound, in bytes.
        limit: usize,
    },
    /// Reading the output or waiting for the process failed, or a process that
    /// had to be killed could not be.
    Io {
        /// The invocation, as text for the message.
        command: String,
        /// The operating-system error.
        message: String,
    },
    /// A [`FakeRunner`] was asked for an invocation it has no answer for.
    Unscripted {
        /// The invocation, as text for the message.
        command: String,
    },
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn {
                command,
                program,
                message,
                ..
            } => write!(
                f,
                "cannot start `{command}`: `{program}` {message}; install it, put it on PATH, or for cargo set $CARGO"
            ),
            Self::Timeout {
                command,
                timeout,
                elapsed,
            } => write!(
                f,
                "`{command}` ran {:.1} s of its {:.0} s limit and was killed; the run has no verdict",
                elapsed.as_secs_f64(),
                timeout.as_secs_f64()
            ),
            Self::OutputTooLarge {
                command,
                stream,
                limit,
            } => write!(
                f,
                "`{command}` wrote more than {limit} bytes to {stream} and was killed; the run has no verdict"
            ),
            Self::Io { command, message } => {
                write!(f, "`{command}` could not be read to its end: {message}")
            }
            Self::Unscripted { command } => write!(
                f,
                "no scripted answer for `{command}`; add one with `FakeRunner::on`"
            ),
        }
    }
}

impl Error for ExecError {}

/// Starts the processes of the workplan.
pub trait Runner {
    /// Run the invocation to its end under the limits.
    ///
    /// # Errors
    ///
    /// An [`ExecError`] when the process cannot be started, runs past the
    /// wall-clock limit, writes more than the output bound to a stream, or
    /// cannot be read to its end. A process that exits with a failing code is
    /// an `Ok` [`Output`]; judging the code is the caller's.
    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError>;
}

impl<R: Runner + ?Sized> Runner for &R {
    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError> {
        (**self).run(invocation, limits)
    }
}

#[cfg(test)]
mod tests;
