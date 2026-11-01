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

mod containment;
mod control;

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
/// Native Linux execution requires bubblewrap PID namespaces and binds the
/// command's lifetime to this host. Other hosts refuse execution. Completed
/// payload status uses bubblewrap's shell encoding: a signal is `128 + signal`,
/// indistinguishable from an explicit exit with that same code.
///
/// A bare program name is looked up by the operating system on the `PATH` of
/// this process, or on the directories given to [`SystemRunner::search_path`].
#[derive(Debug, Clone)]
pub struct SystemRunner {
    programs: Programs,
    search_path: Option<OsString>,
    views: Arc<super::view::HostViews>,
}

struct Launch {
    command: Command,
    program: PathBuf,
    admission: containment::Admission,
    git_selector: Option<GitSelector>,
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
        let git = if filesystem.is_some() {
            Some(&super::view::GitContext::Discover)
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

    fn configure_environment(invocation: &Invocation, limits: &Limits, command: &mut Command) {
        // The caller's own command keeps the caller's environment.
        if limits.c_locale() && invocation.program() != Program::Trusted {
            command.env("LC_ALL", "C");
        }
        if let Some(observer) = invocation.rustdoc_observation() {
            if invocation.is_rustdoc_delegate() {
                command.env("RUSTDOC", observer.delegate());
                command.env_remove(super::RustdocObservationOptions::MARKER);
                command.env_remove(super::RustdocObservationOptions::DELEGATE);
                command.env_remove(super::RustdocObservationOptions::RECEIPTS);
                command.env_remove(super::RustdocObservationOptions::ASSEMBLY);
                command.env_remove(super::RustdocObservationOptions::ASSEMBLY_DIGEST);
            } else {
                command.env_remove(super::RustdocObservationOptions::ASSEMBLY);
                command.env_remove(super::RustdocObservationOptions::ASSEMBLY_DIGEST);
                command.envs(observer.environment());
            }
        }
    }

    fn prepare_command(
        &self,
        invocation: &Invocation,
        limits: &Limits,
    ) -> Result<Launch, ExecError> {
        invocation.validate_rustdoc_context()?;
        let command_text = invocation.to_string();
        let program = self.program_path(invocation)?;
        let trusted = invocation.program() == Program::Trusted;
        let mut argv: Vec<OsString> = invocation.argv().into_iter().map(OsString::from).collect();
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
        let spawned_program = PathBuf::from("bwrap");
        let mut command = Command::new(&spawned_program);
        let admission =
            containment::Admission::prepare(&mut command).map_err(|error| ExecError::Spawn {
                command: command_text.clone(),
                program: spawned_program.display().to_string(),
                kind: error.kind(),
                message: format!("cannot prepare mandatory Linux PID containment: {error}"),
            })?;
        let git_selector = self.apply_git_context(invocation, filesystem.as_deref(), &mut command);
        argv.insert(0, program.as_os_str().to_owned());
        if let Some(filesystem) = &filesystem {
            command.args(filesystem.command(
                &directory,
                containment::Admission::program(),
                &argv,
            )?);
        } else {
            command
                .args(["--dev-bind", "/", "/", "--proc", "/proc", "--"])
                .arg(containment::Admission::program())
                .args(argv);
        }
        command
            .current_dir(directory)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        Self::configure_environment(invocation, limits, &mut command);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt as _;
            command.process_group(0);
        }
        Ok(Launch {
            command,
            program,
            admission,
            git_selector,
        })
    }

    fn run_process<C: Clock>(
        &self,
        invocation: &Invocation,
        limits: &Limits,
        start: impl FnOnce() -> C,
    ) -> Result<Output, ExecError> {
        let command_text = invocation.to_string();
        let Launch {
            mut command,
            program,
            admission,
            git_selector,
        } = self.prepare_command(invocation, limits)?;
        let mut clock = start();
        let child = command.spawn().map_err(|error| ExecError::Spawn {
            command: command_text.clone(),
            program: "bwrap".into(),
            kind: error.kind(),
            message: format!("cannot start mandatory Linux PID containment launcher: {error}"),
        })?;
        // FD mappings are owned by Command. Keeping it alive would retain the
        // metadata writer and prevent EOF even after the launcher has exited.
        drop(command);
        let mut supervised =
            containment::Supervised::admit(child).map_err(|error| ExecError::Spawn {
                command: command_text.clone(),
                program: "bwrap".into(),
                kind: error.kind(),
                message: error.to_string(),
            })?;
        let mut admission = admission.capture();
        clock.spawned(supervised.child.id());
        let (Some(stdout), Some(stderr)) = (
            supervised.child.stdout.take(),
            supervised.child.stderr.take(),
        ) else {
            let failure = ExecError::Io {
                command: command_text.clone(),
                message: "a standard stream was not captured".to_string(),
            };
            return Err(abandon(&mut supervised, failure, command_text));
        };
        let bound = limits.max_output();
        let out = Capture::start(stdout, bound);
        let err = Capture::start(stderr, bound);
        let mut stderr = None;
        let outcome = wait(
            &mut supervised.child,
            &out,
            &err,
            &mut stderr,
            &mut admission,
            limits.timeout(),
            &mut clock,
        );
        if let Wait::Done(code, stdout) = outcome {
            admission.drain();
            let stderr = stderr.unwrap_or_default();
            admission.verify(&command_text, &program, code, &stderr, true)?;
            return Ok(Output {
                code,
                stdout,
                stderr,
                elapsed: clock.elapsed(),
                git_selector,
            });
        }
        // Termination must precede draining: PID-namespace teardown closes all
        // inherited metadata and output writers, including escaped descendants.
        if let Err(error) = supervised.terminate() {
            return Err(ExecError::Io {
                command: command_text,
                message: format!(
                    "{outcome:?}; process termination failed, so it may still be running: {error}"
                ),
            });
        }
        admission.drain();
        let stderr = match stderr {
            Some(bytes) => bytes,
            None => err.finish(&command_text)?,
        };
        admission.verify(&command_text, &program, None, &stderr, false)?;
        let failure = match outcome {
            Wait::Done(_, _) => std::panic::panic_any("completed runs returned above"),
            Wait::Admission => ExecError::Io {
                command: command_text.clone(),
                message: "private admission channel ended unexpectedly".into(),
            },
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
        Err(failure)
    }
}

/// Kill the process that `failure` gave up on. When it cannot be killed the
/// error says so, since the process may still be running.
fn abandon(child: &mut containment::Supervised, failure: ExecError, command: String) -> ExecError {
    match child.terminate() {
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

    /// Receive the final captured bytes after namespace teardown closes writers.
    fn finish(&self, command: &str) -> Result<Vec<u8>, ExecError> {
        self.result
            .recv()
            .map_err(|error| ExecError::Io {
                command: command.to_owned(),
                message: error.to_string(),
            })?
            .map_err(|error| ExecError::Io {
                command: command.to_owned(),
                message: error.to_string(),
            })
    }
}

/// How the wait for a process ended.
#[derive(Debug)]
enum Wait {
    /// The process exited and both streams reached their end.
    Done(Option<i32>, Vec<u8>),
    /// Private admission failed or ended before readiness.
    Admission,
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
    stderr: &mut Option<Vec<u8>>,
    admission: &mut containment::Status,
    timeout: Duration,
    clock: &mut C,
) -> Wait {
    let mut status: Option<ExitStatus> = None;
    let mut stdout: Option<Vec<u8>> = None;
    let mut pause = Duration::from_millis(1);
    loop {
        admission.poll();
        if admission.failed() || (admission.closed() && !admission.confirmed()) {
            return Wait::Admission;
        }
        if let Some(stream) =
            overflow(out.overflowed(), err.overflowed()).filter(|_| admission.confirmed())
        {
            return Wait::Over(stream);
        }
        if status.is_none() {
            match child.try_wait() {
                Ok(found) => status = found,
                Err(error) => return Wait::Failed(error),
            }
        }
        for (capture, slot) in [(out, &mut stdout), (err, &mut *stderr)] {
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
        let done = status.is_some() && stdout.is_some() && stderr.is_some() && admission.closed();
        let elapsed = clock.elapsed();
        match verdict(
            out.overflowed() && admission.confirmed(),
            err.overflowed() && admission.confirmed(),
            done,
            elapsed,
            timeout,
        ) {
            Look::Over(stream) => return Wait::Over(stream),
            Look::Done => {
                return Wait::Done(
                    status.and_then(|status| status.code()),
                    stdout.unwrap_or_default(),
                );
            }
            Look::Late => {
                return overflow(out.overflowed(), err.overflowed()).map_or(Wait::Late, Wait::Over);
            }
            Look::Again => {}
        }
        clock.pause(pause.min(timeout.saturating_sub(elapsed)));
        pause = next_pause(pause);
    }
}

/// Kill the process, and on Unix the group it leads, and collect it.
///
/// A failure to signal is reported: the process may then still be running.
#[cfg(test)]
fn terminate(child: &mut Child) -> io::Result<()> {
    #[cfg(unix)]
    let group = kill_group(child);
    #[cfg(not(unix))]
    let group = Ok(());
    let own = terminate_process(child);
    group.and(own)
}

/// Kill and reap only the direct child, without signalling its descendants.
#[cfg(test)]
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
#[cfg(all(test, unix))]
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
#[cfg(all(test, unix))]
fn leader_ended(child: &mut Child) -> bool {
    if matches!(child.try_wait(), Ok(Some(_))) {
        return true;
    }
    leader_signal_ends_it(&child.kill()) && child.wait().is_ok()
}

/// Whether a SIGKILL sent to the leader itself means it ends: it was delivered,
/// or the leader has no live process left (`ESRCH`). Any other failure, such as
/// `EPERM`, means the leader may go on running.
#[cfg(all(test, unix))]
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
#[cfg(all(test, unix))]
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
