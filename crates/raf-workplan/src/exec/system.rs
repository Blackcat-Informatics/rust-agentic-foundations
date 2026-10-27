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
use super::{
    ExecError, GitSelector, Invocation, Limits, Output, Program, Programs, Runner, Stream,
};

/// The longest pause between two looks at a running process.
const POLL_CEILING: Duration = Duration::from_millis(20);

fn inherited_git_selector() -> Option<GitSelector> {
    if std::env::var_os("GIT_DIR").is_some() {
        Some(GitSelector::Directory)
    } else if std::env::var_os("GIT_WORK_TREE").is_some() {
        Some(GitSelector::WorkTree)
    } else {
        None
    }
}

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
    views: Arc<super::view::HostViews>,
}

impl SystemRunner {
    /// A runner for these programs.
    #[must_use]
    pub fn new(programs: Programs) -> Self {
        Self {
            programs,
            search_path: None,
            views: Arc::default(),
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
    fn prepare_view(&self, request: &super::ViewRequest) -> Result<super::PreparedView, ExecError> {
        self.views.prepare(self, request)
    }
    fn revalidate_view(&self, view: &super::PreparedView) -> Result<(), ExecError> {
        self.views.revalidate(self, view)
    }
    fn finish_view(&self, view: &super::PreparedView) -> Result<(), ExecError> {
        self.views.finish(self, view)
    }
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
        if let Some(context) = invocation.view() {
            self.views.validate(self, context)?;
        }
        let result = self.run_process(invocation, limits, start);
        if let Some(context) = invocation.view()
            && let Err(changed) = self.views.validate(self, context)
        {
            return match result {
                Ok(_) => Err(changed),
                Err(primary) => Err(ExecError::ViewCleanup {
                    primary: Box::new(primary),
                    cleanup: Box::new(changed),
                }),
            };
        }
        result
    }

    fn apply_git_context(
        &self,
        invocation: &Invocation,
        filesystem: Option<&super::view::FilesystemView>,
        command: &mut Command,
    ) -> Option<GitSelector> {
        let filesystem_git = filesystem.map(super::view::FilesystemView::git);
        let git = if let Some(git) = &filesystem_git {
            Some(git)
        } else if let Some(git) = invocation.git_context() {
            Some(git)
        } else if invocation.program() == Program::Git && self.programs.discover_git {
            Some(&super::view::GitContext::Discover)
        } else {
            None
        };
        if let Some(git) = git {
            git.apply(command);
        }
        (invocation.program() == Program::Git)
            .then(|| git.map_or_else(inherited_git_selector, super::view::GitContext::selector))
            .flatten()
    }

    fn run_process<C: Clock>(
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
        let directory = if let Some(context) = invocation.view() {
            self.views.directory(context)?
        } else {
            invocation
                .directory()
                .unwrap_or(&self.programs.cwd)
                .to_path_buf()
        };
        let filesystem = if let Some(context) = invocation.view() {
            Some(self.views.filesystem(context)?)
        } else {
            invocation.filesystem()
        };
        let spawned_program = if filesystem.is_some() {
            PathBuf::from("bwrap")
        } else {
            program.clone()
        };
        let mut command = Command::new(&spawned_program);
        let git_selector = self.apply_git_context(invocation, filesystem.as_deref(), &mut command);
        if let Some(filesystem) = &filesystem {
            command.args(filesystem.command(&directory, &program, &argv)?);
        } else {
            command.args(argv);
        }
        command
            .current_dir(directory)
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
            program: spawned_program.display().to_string(),
            kind: error.kind(),
            message: error.to_string(),
        })?;
        clock.spawned(child.id());
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
                    git_selector,
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
    let own = terminate_process(child);
    group.and(own)
}

/// Kill and reap only the direct child, without signalling its descendants.
fn terminate_process(child: &mut Child) -> io::Result<()> {
    // `Child::kill` succeeds on a process that has already exited. Still wait
    // after a signal failure, and report a wait failure when signalling worked.
    let killed = child.kill();
    let waited = child.wait().map(|_| ());
    killed.and(waited)
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
    // are exiting or have exited but are not reaped yet. The leader having
    // ended is what tells that case from a group that cannot be signalled, and
    // it is looked at only then. Elsewhere EPERM means a live member that may
    // not be signalled, and is always reported.
    let leader_ended = cfg!(target_os = "macos")
        && signalled == Err(rustix::io::Errno::PERM)
        && leader_ended(child);
    group_signal_outcome(signalled, leader_ended)
}

/// Whether the leader has ended, collecting it when it has.
///
/// A leader that is still exiting is not yet seen by `try_wait` on macOS,
/// although the group signal already passed it over. The leader is the child
/// itself, so it is sent SIGKILL directly: when that is delivered, or finds the
/// leader already on its way out, the leader ends and the wait that follows
/// returns once it has (`terminate` waits for it in any case). A leader that
/// refuses its own signal has not ended and is not waited for.
#[cfg(unix)]
fn leader_ended(child: &mut Child) -> bool {
    if matches!(child.try_wait(), Ok(Some(_))) {
        return true;
    }
    leader_signal_ends_it(&child.kill()) && child.wait().is_ok()
}

/// Whether a SIGKILL sent to the leader itself means it ends: it was delivered,
/// or the leader has no live process left (`ESRCH`). Any other failure, such as
/// `EPERM`, means the leader may go on running.
#[cfg(unix)]
fn leader_signal_ends_it(signalled: &io::Result<()>) -> bool {
    match signalled {
        Ok(()) => true,
        Err(error) => rustix::io::Errno::from_io_error(error) == Some(rustix::io::Errno::SRCH),
    }
}

/// What a failed or successful group signal means. A group with no live member
/// (`ESRCH`) is done; so is one that answers `EPERM` while its leader has
/// ended, because a group signal fails only when it reached no member, and the
/// leader could be signalled, so it was already exiting (macOS only: `kill_group` passes `leader_ended` as
/// false elsewhere). Any other
/// failure means the group may still be running and is reported.
#[cfg(unix)]
fn group_signal_outcome(
    signalled: Result<(), rustix::io::Errno>,
    leader_ended: bool,
) -> io::Result<()> {
    use rustix::io::Errno;

    match signalled {
        Ok(()) | Err(Errno::SRCH) => Ok(()),
        Err(Errno::PERM) if leader_ended => Ok(()),
        Err(errno) => Err(errno.into()),
    }
}

#[cfg(test)]
mod tests;
