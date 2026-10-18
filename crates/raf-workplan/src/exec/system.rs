// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The runner that starts real processes.

use std::ffi::OsString;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use super::clock::{Clock, WallClock};
use super::{ExecError, Invocation, Limits, Output, Program, Programs, Runner, Stream};

/// The longest pause between two looks at a running process.
const POLL_CEILING: Duration = Duration::from_millis(20);

/// Starts real processes: no shell, the argv vector as given, standard input
/// null, standard output and standard error captured within the bound of
/// [`Limits`], and a kill when the wall-clock limit passes.
///
/// A bare program name is looked up by the operating system on the `PATH` of
/// this process, or on the directories given to [`SystemRunner::search_path`].
#[derive(Debug, Clone)]
pub struct SystemRunner {
    programs: Programs,
    search_path: Option<OsString>,
}

impl SystemRunner {
    /// A runner for these programs.
    #[must_use]
    pub fn new(programs: Programs) -> Self {
        Self {
            programs,
            search_path: None,
        }
    }

    /// Look a bare program name up in these directories (separated as `PATH`
    /// is) instead of on the `PATH` of this process. The environment of the
    /// started process is not changed.
    #[must_use]
    pub fn search_path(mut self, directories: impl Into<OsString>) -> Self {
        self.search_path = Some(directories.into());
        self
    }

    /// The programs this runner starts.
    #[must_use]
    pub fn programs(&self) -> &Programs {
        &self.programs
    }

    /// The path to start for the invocation, or the error when a search path
    /// is set and the name is not in it.
    fn program_path(&self, invocation: &Invocation) -> Result<PathBuf, ExecError> {
        // The caller's own command names its program as its first word.
        let first = PathBuf::from(invocation.argv().first().cloned().unwrap_or_default());
        let named = self
            .programs
            .path_of(invocation.program())
            .unwrap_or(first.as_path());
        let Some(directories) = &self.search_path else {
            return Ok(named.to_path_buf());
        };
        if named.components().count() != 1 {
            return Ok(named.to_path_buf());
        }
        std::env::split_paths(directories)
            .map(|directory| directory.join(named))
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| ExecError::Spawn {
                command: invocation.to_string(),
                program: named.display().to_string(),
                kind: io::ErrorKind::NotFound,
                message: "is not in the search path".to_string(),
            })
    }
}

impl Runner for SystemRunner {
    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError> {
        self.run_with(invocation, limits, WallClock::start)
    }
}

impl SystemRunner {
    /// Run the invocation, starting `start` just before the process is.
    fn run_with<C: Clock>(
        &self,
        invocation: &Invocation,
        limits: &Limits,
        start: impl FnOnce() -> C,
    ) -> Result<Output, ExecError> {
        let command_text = invocation.to_string();
        let program = self.program_path(invocation)?;
        let trusted = invocation.program() == Program::Trusted;
        let mut argv = invocation.argv();
        if trusted {
            // The first word of the caller's own command is its program.
            argv.remove(0);
        }
        let mut command = Command::new(&program);
        command
            .args(argv)
            .current_dir(&self.programs.cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // The caller's own command keeps the caller's environment.
        if limits.c_locale() && !trusted {
            command.env("LC_ALL", "C");
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt as _;
            command.process_group(0);
        }
        let mut clock = start();
        let mut child = command.spawn().map_err(|error| ExecError::Spawn {
            command: command_text.clone(),
            program: program.display().to_string(),
            kind: error.kind(),
            message: error.to_string(),
        })?;
        let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
            let failure = ExecError::Io {
                command: command_text.clone(),
                message: "a standard stream was not captured".to_string(),
            };
            return Err(abandon(&mut child, failure, command_text));
        };
        let bound = limits.max_output();
        let out = Capture::start(stdout, bound);
        let err = Capture::start(stderr, bound);
        let outcome = wait(&mut child, &out, &err, limits.timeout(), &mut clock);
        let failure = match outcome {
            Wait::Done(code, stdout, stderr) => {
                return Ok(Output {
                    code,
                    stdout,
                    stderr,
                    elapsed: clock.elapsed(),
                });
            }
            Wait::Over(stream) => ExecError::OutputTooLarge {
                command: command_text.clone(),
                stream,
                limit: bound,
                elapsed: clock.elapsed(),
            },
            Wait::Late => ExecError::Timeout {
                command: command_text.clone(),
                timeout: limits.timeout(),
                elapsed: clock.elapsed(),
            },
            Wait::Failed(error) => ExecError::Io {
                command: command_text.clone(),
                message: error.to_string(),
            },
        };
        Err(abandon(&mut child, failure, command_text))
    }
}

/// Kill the process that `failure` gave up on. When it cannot be killed the
/// error says so, since the process may still be running.
fn abandon(child: &mut Child, failure: ExecError, command: String) -> ExecError {
    match terminate(child) {
        Ok(()) => failure,
        Err(error) => ExecError::Io {
            command,
            message: format!(
                "{failure}; and the process could not be killed, so it may still be running: {error}"
            ),
        },
    }
}

/// A thread that reads one stream up to a bound.
struct Capture {
    overflow: Arc<AtomicBool>,
    result: Receiver<io::Result<Vec<u8>>>,
}

impl Capture {
    /// Read `pipe` to its end on a thread of its own, keeping at most `bound`
    /// bytes. Reading stops at the first byte past the bound and the overflow
    /// flag is set before the result is sent.
    fn start(pipe: impl Read + Send + 'static, bound: usize) -> Self {
        let overflow = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&overflow);
        let (send, result) = mpsc::channel();
        let limit = u64::try_from(bound).map_or(u64::MAX, |bound| bound.saturating_add(1));
        thread::spawn(move || {
            let mut kept = Vec::new();
            let read = pipe.take(limit).read_to_end(&mut kept);
            let answer = read.map(|length| {
                if length > bound {
                    flag.store(true, Ordering::SeqCst);
                }
                kept
            });
            // The receiver is gone when the run was abandoned.
            let _ = send.send(answer);
        });
        Self { overflow, result }
    }

    fn overflowed(&self) -> bool {
        self.overflow.load(Ordering::SeqCst)
    }
}

/// How the wait for a process ended.
enum Wait {
    /// The process exited and both streams reached their end.
    Done(Option<i32>, Vec<u8>, Vec<u8>),
    /// A stream passed its bound.
    Over(Stream),
    /// The wall-clock limit passed.
    Late,
    /// Waiting or reading failed.
    Failed(io::Error),
}

/// What one look at a run decides, from what the look found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Look {
    /// A stream passed its bound.
    Over(Stream),
    /// The process exited and both streams reached their end.
    Done,
    /// The wall-clock limit passed.
    Late,
    /// Nothing is decided; look again after a pause.
    Again,
}

/// The stream that passed its bound, the standard output first.
fn overflow(overflow_out: bool, overflow_err: bool) -> Option<Stream> {
    if overflow_out {
        Some(Stream::Stdout)
    } else if overflow_err {
        Some(Stream::Stderr)
    } else {
        None
    }
}

/// Decide a look. A passed bound beats a finished run, and a finished run
/// beats the limit; the limit is passed once `elapsed` reaches `timeout`.
fn verdict(
    overflow_out: bool,
    overflow_err: bool,
    done: bool,
    elapsed: Duration,
    timeout: Duration,
) -> Look {
    if let Some(stream) = overflow(overflow_out, overflow_err) {
        Look::Over(stream)
    } else if done {
        Look::Done
    } else if elapsed >= timeout {
        Look::Late
    } else {
        Look::Again
    }
}

/// The pause after this one: twice as long, up to [`POLL_CEILING`].
fn next_pause(pause: Duration) -> Duration {
    (pause * 2).min(POLL_CEILING)
}

/// Poll the process and the two captures until all three are finished, a bound
/// is passed, the limit passes, or something fails.
fn wait<C: Clock>(
    child: &mut Child,
    out: &Capture,
    err: &Capture,
    timeout: Duration,
    clock: &mut C,
) -> Wait {
    let mut status: Option<ExitStatus> = None;
    let mut stdout: Option<Vec<u8>> = None;
    let mut stderr: Option<Vec<u8>> = None;
    let mut pause = Duration::from_millis(1);
    loop {
        if let Some(stream) = overflow(out.overflowed(), err.overflowed()) {
            return Wait::Over(stream);
        }
        if status.is_none() {
            match child.try_wait() {
                Ok(found) => status = found,
                Err(error) => return Wait::Failed(error),
            }
        }
        for (capture, slot) in [(out, &mut stdout), (err, &mut stderr)] {
            if slot.is_none() {
                match capture.result.try_recv() {
                    Ok(Ok(bytes)) => *slot = Some(bytes),
                    Ok(Err(error)) => return Wait::Failed(error),
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => {
                        return Wait::Failed(io::Error::other("an output reader stopped"));
                    }
                }
            }
        }
        // A stream that overflowed has just been reported as a result too.
        let done = status.is_some() && stdout.is_some() && stderr.is_some();
        let elapsed = clock.elapsed();
        match verdict(out.overflowed(), err.overflowed(), done, elapsed, timeout) {
            Look::Over(stream) => return Wait::Over(stream),
            Look::Done => {
                return Wait::Done(
                    status.and_then(|status| status.code()),
                    stdout.unwrap_or_default(),
                    stderr.unwrap_or_default(),
                );
            }
            Look::Late => return Wait::Late,
            Look::Again => {}
        }
        clock.pause(pause.min(timeout.saturating_sub(elapsed)));
        pause = next_pause(pause);
    }
}

/// Kill the process, and on Unix the group it leads, and collect it.
///
/// A failure to signal is reported: the process may then still be running.
fn terminate(child: &mut Child) -> io::Result<()> {
    #[cfg(unix)]
    let group = kill_group(child);
    #[cfg(not(unix))]
    let group = Ok(());
    // `Child::kill` succeeds on a process that has already exited.
    let own = child.kill();
    let _ = child.wait();
    group.and(own)
}

/// Send SIGKILL to the process group the child leads. The child is started
/// with `process_group(0)` (see `run`), so its id is the group id. The group
/// holds what `cargo` started, such as a test binary that keeps the output
/// pipes open. A group with no member left is not an error.
#[cfg(unix)]
fn kill_group(child: &mut Child) -> io::Result<()> {
    use rustix::process::{Pid, Signal, kill_process_group};

    let raw = i32::try_from(child.id()).map_err(io::Error::other)?;
    let pid = Pid::from_raw(raw).ok_or_else(|| io::Error::other("the process id is not valid"))?;
    let signalled = kill_process_group(pid, Signal::KILL);
    // macOS answers EPERM, not ESRCH, when the group holds only processes that
    // have already exited but are not reaped yet. The leader having exited is
    // what tells that case from a group that cannot be signalled.
    let leader_exited = matches!(child.try_wait(), Ok(Some(_)));
    group_signal_outcome(signalled, leader_exited)
}

/// What a failed or successful group signal means. A group with no live member
/// (`ESRCH`) is done; so is one that answers `EPERM` while its leader has
/// already exited, because only unreaped processes are left (macOS). Any other
/// failure means the group may still be running and is reported.
#[cfg(unix)]
fn group_signal_outcome(
    signalled: Result<(), rustix::io::Errno>,
    leader_exited: bool,
) -> io::Result<()> {
    use rustix::io::Errno;

    match signalled {
        Ok(()) | Err(Errno::SRCH) => Ok(()),
        Err(Errno::PERM) if leader_exited => Ok(()),
        Err(errno) => Err(errno.into()),
    }
}

#[cfg(test)]
mod tests;
