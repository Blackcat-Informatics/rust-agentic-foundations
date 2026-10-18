// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Tests of the system runner.
//!
//! The runs of real processes go through [`SystemRunner::run_with`] with a
//! clock the test controls, so no test reads or waits on the clock of the
//! machine. Under [`Frozen`] every look reads the same time, so only the end of
//! the process or an output bound can end the run, and the time a run reports
//! is that reading exactly. Under [`Stepped`] each pause the runner asks for
//! moves the reading by that much, so the limit passes after a fixed number of
//! looks whatever the speed of the host. Both count every pause and fail the
//! test past a ceiling, so a runner that never decides fails instead of hanging.
//!
//! A process that has to outlive its limit blocks reading a FIFO that the test
//! holds open (the hold), after writing its process id to another one (the
//! pids); a leader that exits and leaves it behind first waits on a third (go)
//! until its pidfd is taken. On Linux the test takes a pidfd of each process
//! before the kill can be asked for; once the run is over it releases the hold,
//! blocks on each pidfd until the process has ended and looks at it in
//! `/proc`. A killed process never ran the rest of its script, so the marker
//! file that the script writes after the hold is absent; one that survived the
//! kill writes the marker and ends, so no step can block for ever.

use std::io::{self, Cursor};
use std::time::Duration;

use super::{Capture, Look, POLL_CEILING, next_pause, verdict};
use crate::exec::Stream;

#[cfg(unix)]
use rustix::io::Errno;
#[cfg(unix)]
use rustix::process::{Pid, WaitId, WaitIdOptions, waitid};
#[cfg(unix)]
use std::os::unix::process::CommandExt as _;
#[cfg(unix)]
use std::process::{Command, Stdio};

#[cfg(unix)]
use super::{SystemRunner, group_signal_outcome, kill_group, terminate};
#[cfg(unix)]
use crate::exec::clock::Clock;
#[cfg(unix)]
use crate::exec::{ExecError, Invocation, Limits, Programs, TrustedArgv};

#[cfg(unix)]
/// A process in a group of its own that has exited and is not reaped yet.
fn exited_group_leader() -> std::process::Child {
    let mut command = Command::new("sh");
    command
        .args(["-c", "exit 0"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    let child = command.spawn().expect("sh starts");
    // Block until the shell has exited without collecting it, so the group
    // holds only an exited, unreaped process whatever the host's speed.
    let pid = Pid::from_raw(i32::try_from(child.id()).expect("a pid")).expect("a pid");
    waitid(
        WaitId::Pid(pid),
        WaitIdOptions::EXITED | WaitIdOptions::NOWAIT,
    )
    .expect("the shell exits");
    child
}

#[cfg(unix)]
#[test]
fn a_group_whose_leader_exited_before_the_kill_is_killed_without_error() {
    let mut child = exited_group_leader();
    kill_group(&mut child).expect("an exited, unreaped group is done");
    child.wait().expect("the leader is collected");
}

#[cfg(unix)]
#[test]
fn terminating_a_leader_that_exited_before_the_kill_succeeds() {
    let mut child = exited_group_leader();
    terminate(&mut child).expect("an exited leader is terminated");
}

#[cfg(unix)]
#[test]
fn a_group_that_already_exited_is_done() {
    assert!(group_signal_outcome(Ok(()), false).is_ok());
    assert!(group_signal_outcome(Err(Errno::SRCH), false).is_ok());
}

#[cfg(unix)]
#[test]
fn a_refused_signal_is_done_only_when_the_leader_has_exited() {
    assert!(group_signal_outcome(Err(Errno::PERM), true).is_ok());
    assert!(group_signal_outcome(Err(Errno::PERM), false).is_err());
}

#[cfg(unix)]
#[test]
fn any_other_failure_is_reported_even_after_the_leader_exited() {
    assert!(group_signal_outcome(Err(Errno::INVAL), true).is_err());
}

const MS: fn(u64) -> Duration = Duration::from_millis;

#[test]
fn the_verdict_follows_its_precedence() {
    let limit = MS(100);
    // (overflow on stdout, overflow on stderr, done, elapsed, expected)
    let table = [
        (false, false, false, MS(0), Look::Again),
        (false, false, false, MS(99), Look::Again),
        (false, false, false, MS(100), Look::Late),
        (false, false, false, MS(101), Look::Late),
        (false, false, true, MS(0), Look::Done),
        (false, false, true, MS(100), Look::Done),
        (false, false, true, MS(500), Look::Done),
        (true, false, false, MS(0), Look::Over(Stream::Stdout)),
        (false, true, false, MS(0), Look::Over(Stream::Stderr)),
        (true, false, true, MS(0), Look::Over(Stream::Stdout)),
        (false, true, true, MS(0), Look::Over(Stream::Stderr)),
        (true, false, false, MS(500), Look::Over(Stream::Stdout)),
        (false, true, false, MS(500), Look::Over(Stream::Stderr)),
        (true, false, true, MS(500), Look::Over(Stream::Stdout)),
        (false, true, true, MS(500), Look::Over(Stream::Stderr)),
        (true, true, false, MS(0), Look::Over(Stream::Stdout)),
        (true, true, true, MS(0), Look::Over(Stream::Stdout)),
        (true, true, false, MS(500), Look::Over(Stream::Stdout)),
        (true, true, true, MS(500), Look::Over(Stream::Stdout)),
    ];
    for (out, err, done, elapsed, expected) in table {
        assert_eq!(
            verdict(out, err, done, elapsed, limit),
            expected,
            "out {out}, err {err}, done {done}, elapsed {elapsed:?}"
        );
    }
}

#[test]
fn a_zero_limit_is_passed_at_once_unless_the_run_is_done() {
    assert_eq!(
        verdict(false, false, false, Duration::ZERO, Duration::ZERO),
        Look::Late
    );
    assert_eq!(
        verdict(false, false, true, Duration::ZERO, Duration::ZERO),
        Look::Done
    );
}

#[test]
fn the_pauses_double_up_to_the_ceiling_and_stay_there() {
    assert_eq!(POLL_CEILING, MS(20));
    let mut pause = MS(1);
    let mut seen = vec![pause];
    for _ in 0..7 {
        pause = next_pause(pause);
        seen.push(pause);
    }
    assert_eq!(
        seen,
        [1, 2, 4, 8, 16, 20, 20, 20].map(MS),
        "1, 2, 4, 8, 16, then the ceiling"
    );
    assert_eq!(next_pause(MS(20)), MS(20));
    assert_eq!(next_pause(MS(15)), MS(20));
    assert_eq!(next_pause(MS(10)), MS(20));
    assert_eq!(next_pause(MS(9)), MS(18));
}

#[test]
fn the_capture_stops_one_byte_past_its_bound_and_flags_the_overflow() {
    let over = Capture::start(io::repeat(b'x'), 1000);
    let kept = over
        .result
        .recv()
        .expect("the reader answers")
        .expect("the read succeeds");
    assert_eq!(
        kept.len(),
        1001,
        "reading stops at the first byte past the bound"
    );
    assert!(over.overflowed(), "one byte past the bound is flagged");

    let at = Capture::start(Cursor::new(vec![b'x'; 1000]), 1000);
    let kept = at
        .result
        .recv()
        .expect("the reader answers")
        .expect("the read succeeds");
    assert_eq!(kept.len(), 1000);
    assert!(!at.overflowed(), "a stream at the bound is not flagged");
}

// ------------------------------------------------- the clocks of the tests

/// A borrowed clock is the clock of a run, so the test can read what the
/// clock saw once the run is over.
#[cfg(unix)]
impl<C: Clock + ?Sized> Clock for &mut C {
    fn elapsed(&self) -> Duration {
        (**self).elapsed()
    }

    fn pause(&mut self, pause: Duration) {
        (**self).pause(pause);
    }
}

/// The most pauses a run under [`Frozen`] may ask for. Each pause only yields
/// the thread, so a short process ends within a small part of this; a runner
/// that never decides reaches it and fails the test instead of hanging.
#[cfg(unix)]
const FROZEN_CEILING: usize = 5_000_000;

/// The most pauses a run under [`Stepped`] may ask for. The limit of two
/// seconds passes on the 104th (31 ms in the first five, then 20 ms each, then
/// the 9 ms that remain).
#[cfg(unix)]
const STEPPED_CEILING: usize = 200;

/// A clock whose reading never moves: a pause yields the thread and is counted.
#[cfg(unix)]
struct Frozen {
    elapsed: Duration,
    pauses: usize,
    ceiling: usize,
}

#[cfg(unix)]
impl Frozen {
    /// A clock that reads `elapsed` at every look.
    fn at(elapsed: Duration) -> Self {
        Self {
            elapsed,
            pauses: 0,
            ceiling: FROZEN_CEILING,
        }
    }
}

#[cfg(unix)]
impl Clock for Frozen {
    fn elapsed(&self) -> Duration {
        self.elapsed
    }

    fn pause(&mut self, _pause: Duration) {
        self.pauses += 1;
        assert!(
            self.pauses <= self.ceiling,
            "the run paused {} times under a clock that never moves and decided nothing",
            self.pauses
        );
        std::thread::yield_now();
    }
}

/// A clock that moves by each pause the runner asks for, and nothing else. On
/// Linux its first pause first reads the process ids from the pid FIFO of a
/// [`Subject`] (see [`Gate`]), so the processes are known before the limit
/// can pass.
#[cfg(unix)]
struct Stepped {
    now: Duration,
    pauses: usize,
    ceiling: usize,
    #[cfg(target_os = "linux")]
    gate: Option<Gate>,
    #[cfg(target_os = "linux")]
    seen: Option<Seen>,
}

#[cfg(unix)]
impl Clock for Stepped {
    fn elapsed(&self) -> Duration {
        self.now
    }

    fn pause(&mut self, pause: Duration) {
        self.pauses += 1;
        assert!(
            self.pauses <= self.ceiling,
            "the run paused {} times, past its ceiling of {}, with the clock at {:?}",
            self.pauses,
            self.ceiling,
            self.now
        );
        #[cfg(target_os = "linux")]
        if let Some(gate) = self.gate.take() {
            self.seen = Some(gate.open());
        }
        self.now += pause;
    }
}

// ------------------------------------------------------- the processes run

/// A command of `/bin/sh -c <script>`, as the caller's own command runs:
/// no program is looked up on `PATH` and no file is written to be executed.
#[cfg(unix)]
fn sh(script: &str) -> Invocation {
    let words = ["/bin/sh", "-c", script].map(str::to_string).to_vec();
    Invocation::trusted(&TrustedArgv::new(words).expect("a command"))
}

/// A runner in the directory of temporary files.
#[cfg(unix)]
fn runner() -> SystemRunner {
    SystemRunner::new(Programs::new(std::env::temp_dir()))
}

/// A directory of its own under the directory of temporary files, removed on
/// drop. Its name is this process and a count, made with `create_dir`, so two
/// tests never share one.
#[cfg(unix)]
struct Scratch(std::path::PathBuf);

#[cfg(unix)]
impl Scratch {
    fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static NEXT: AtomicUsize = AtomicUsize::new(0);
        loop {
            let path = std::env::temp_dir().join(format!(
                "raf-workplan-runner-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match std::fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("{}: {error}", path.display()),
            }
        }
    }
}

#[cfg(unix)]
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// What a process that outlives its limit leaves behind it.
#[cfg(unix)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// The process itself blocks.
    Alone,
    /// The process starts one that blocks, holding the output pipes, and waits
    /// for it.
    Waits,
    /// The process starts one that blocks, holding the output pipes, and exits.
    Leaves,
}

/// The two seconds of the runs that outlive their limit.
#[cfg(unix)]
const LIMIT: Duration = Duration::from_secs(2);

/// A run of a process of `shape` that outlives its limit, and on Linux what
/// became of its processes.
#[cfg(unix)]
struct Outlived {
    command: String,
    result: Result<crate::exec::Output, ExecError>,
    #[cfg(target_os = "linux")]
    seen: Seen,
    #[cfg(target_os = "linux")]
    settled: Settled,
}

/// Run a process of `shape` under a stepped clock with a two-second limit.
#[cfg(target_os = "linux")]
fn outlive(shape: Shape) -> Outlived {
    let subject = Subject::new();
    let invocation = sh(&subject.script(shape));
    let mut clock = Stepped {
        now: Duration::ZERO,
        pauses: 0,
        ceiling: STEPPED_CEILING,
        gate: Some(subject.gate(shape)),
        seen: None,
    };
    let limits = Limits::trusted().with_timeout(LIMIT);
    let result = runner().run_with(&invocation, &limits, || &mut clock);
    let Some(seen) = clock.seen.take() else {
        panic!("the run ended before its first pause: {result:?}");
    };
    let settled = subject.settle(&seen);
    Outlived {
        command: invocation.to_string(),
        result,
        seen,
        settled,
    }
}

/// Run a process of `shape` under a stepped clock with a two-second limit.
/// Without the FIFOs of Linux the process never ends by running a loop of the
/// shell, and only the error is asserted.
#[cfg(all(unix, not(target_os = "linux")))]
fn outlive(shape: Shape) -> Outlived {
    let script = match shape {
        Shape::Alone => "while :; do :; done",
        Shape::Waits => "{ while :; do :; done; } & wait",
        Shape::Leaves => "{ while :; do :; done; } & exit 0",
    };
    let invocation = sh(script);
    let mut clock = Stepped {
        now: Duration::ZERO,
        pauses: 0,
        ceiling: STEPPED_CEILING,
    };
    let limits = Limits::trusted().with_timeout(LIMIT);
    let result = runner().run_with(&invocation, &limits, || &mut clock);
    Outlived {
        command: invocation.to_string(),
        result,
    }
}

/// The error of a run killed at its limit, with the limit as the time it ran.
#[cfg(unix)]
fn killed_at_the_limit(run: &Outlived) {
    assert_eq!(
        run.result,
        Err(ExecError::Timeout {
            command: run.command.clone(),
            timeout: LIMIT,
            elapsed: LIMIT,
        })
    );
}

// --------------------------------------- the processes of a run, on Linux

/// The FIFOs and the marker of one process that outlives its limit.
#[cfg(target_os = "linux")]
struct Subject {
    scratch: Scratch,
    /// The pid FIFO, open for reading and writing since before the spawn, so
    /// the script's write never blocks and the read never ends early.
    pids: std::fs::File,
    /// The hold FIFO, open the same way, so the release is a plain write that
    /// lands whether or not the script has opened it yet.
    hold: std::fs::File,
    /// The FIFO a leader that leaves its descendant behind reads before it
    /// exits, open the same way: the leader stays unreaped until its pidfd is
    /// taken.
    go: std::fs::File,
}

#[cfg(target_os = "linux")]
impl Subject {
    fn new() -> Self {
        let scratch = Scratch::new();
        let fifo = |name: &str| {
            let path = scratch.0.join(name);
            rustix::fs::mknodat(
                rustix::fs::CWD,
                &path,
                rustix::fs::FileType::Fifo,
                rustix::fs::Mode::from_raw_mode(0o600),
                0,
            )
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        };
        let pids = fifo("pids");
        let hold = fifo("hold");
        let go = fifo("go");
        Self {
            scratch,
            pids,
            hold,
            go,
        }
    }

    /// The absolute path of `name` in the scratch directory, quoted for the
    /// shell.
    fn quoted(&self, name: &str) -> String {
        let path = self.scratch.0.join(name);
        let text = path.to_str().expect("a UTF-8 temporary directory");
        assert!(!text.contains('\''), "{text}");
        format!("'{text}'")
    }

    fn marker(&self) -> std::path::PathBuf {
        self.scratch.0.join("marker")
    }

    /// The script: the process ids go to the pid FIFO, the process that
    /// blocks reads the hold FIFO, and after the hold it writes the marker. A
    /// leader that exits reads the go FIFO first. Every word is a builtin of
    /// the shell, so it runs with an empty `PATH`.
    fn script(&self, shape: Shape) -> String {
        let (pids, hold, go, marker) = (
            self.quoted("pids"),
            self.quoted("hold"),
            self.quoted("go"),
            self.quoted("marker"),
        );
        let blocks = format!("read x < {hold}; echo survived > {marker}");
        match shape {
            Shape::Alone => format!("echo $$ > {pids}; {blocks}; exit 0"),
            Shape::Waits => format!("{{ {blocks}; }} & echo \"$$ $!\" > {pids}; wait"),
            Shape::Leaves => {
                format!("{{ {blocks}; }} & echo \"$$ $!\" > {pids}; read y < {go}; exit 0")
            }
        }
    }

    /// The gate of a stepped clock for a process of `shape`. When the process
    /// leaves one behind and exits, the gate lets it exit once the pidfds are
    /// taken and blocks until it has, so the runner's next look finds its
    /// status in.
    fn gate(&self, shape: Shape) -> Gate {
        let clone = |file: &std::fs::File| file.try_clone().expect("a FIFO is shared");
        Gate {
            pids: clone(&self.pids),
            go: (shape == Shape::Leaves).then(|| clone(&self.go)),
        }
    }

    /// What became of the processes once the run is over: whether the leader
    /// is reaped as the run returns; then, with the hold released and each
    /// process ended, how each stands in `/proc` and whether the marker was
    /// written. A process the kill reached never read the release, so the
    /// marker is there only if one survived the kill.
    fn settle(&self, seen: &Seen) -> Settled {
        use std::io::Write as _;
        use std::os::fd::AsFd as _;

        let reaped = waitid(
            WaitId::PidFd(seen.leader.fd.as_fd()),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG,
        );
        (&self.hold).write_all(b"\n").expect("the hold is released");
        seen.leader.wait_for_end();
        if let Some(descendant) = &seen.descendant {
            descendant.wait_for_end();
        }
        Settled {
            leader: seen.leader.fate(),
            descendant: seen.descendant.as_ref().map(Process::fate),
            leader_reaped: matches!(reaped, Err(Errno::CHILD)),
            marker: self.marker().exists(),
        }
    }
}

/// The first pause of a [`Stepped`] clock: read the process ids the script
/// writes and take a pidfd of each before the limit can pass.
#[cfg(target_os = "linux")]
struct Gate {
    pids: std::fs::File,
    /// The go FIFO, for a leader that exits.
    go: Option<std::fs::File>,
}

#[cfg(target_os = "linux")]
impl Gate {
    fn open(self) -> Seen {
        use std::io::{Read as _, Write as _};

        let mut line = Vec::new();
        while !line.contains(&b'\n') {
            let mut chunk = [0_u8; 64];
            let read = (&self.pids).read(&mut chunk).expect("the pid FIFO is read");
            // The test holds a writer, so a read never finds the end.
            assert_ne!(read, 0, "the pid FIFO ended");
            line.extend_from_slice(&chunk[..read]);
        }
        let line = String::from_utf8(line).expect("process ids");
        let mut pids = line.split_whitespace().map(Process::open);
        let leader = pids.next().expect("the leader's process id");
        let descendant = pids.next();
        if let Some(mut go) = self.go {
            go.write_all(b"\n").expect("the leader is let go");
            leader.wait_for_end();
        }
        Seen { leader, descendant }
    }
}

/// The processes of a run as the gate found them. Dropping it sends SIGKILL to
/// each of them and to the group, so a run that fails leaves nothing behind.
#[cfg(target_os = "linux")]
struct Seen {
    leader: Process,
    descendant: Option<Process>,
}

#[cfg(target_os = "linux")]
impl Drop for Seen {
    fn drop(&mut self) {
        use rustix::process::{Signal, kill_process_group, pidfd_send_signal};

        for process in std::iter::once(&self.leader).chain(&self.descendant) {
            let _ = pidfd_send_signal(&process.fd, Signal::KILL);
        }
        let _ = kill_process_group(self.leader.pid, Signal::KILL);
    }
}

/// One process, held by a pidfd, and its `/proc/<pid>/stat` when it was taken.
#[cfg(target_os = "linux")]
struct Process {
    pid: Pid,
    fd: std::os::fd::OwnedFd,
    stat: Stat,
}

#[cfg(target_os = "linux")]
impl Process {
    fn open(word: &str) -> Self {
        use rustix::process::{PidfdFlags, pidfd_open};

        let raw: i32 = word.parse().expect("a process id");
        let pid = Pid::from_raw(raw).expect("a process id");
        // The process is blocked reading a FIFO the test holds and is not
        // reaped, so the id is still this process.
        let fd = pidfd_open(pid, PidfdFlags::empty()).expect("a pidfd of a live process");
        let stat = Stat::of(pid).expect("the stat of a live process");
        Self { pid, fd, stat }
    }

    /// How the process stands now: gone, dead and not reaped, dying of a
    /// pending SIGKILL, or running.
    fn fate(&self) -> Fate {
        let Some(now) = Stat::of(self.pid) else {
            return Fate::Gone;
        };
        if now.starttime != self.stat.starttime {
            // Another process has the id.
            return Fate::Gone;
        }
        if matches!(now.state, 'Z' | 'X') {
            return Fate::Dead(now.state);
        }
        let Ok(status) = std::fs::read_to_string(format!("/proc/{}/status", self.pid.as_raw_pid()))
        else {
            return Fate::Gone;
        };
        let pending = ["SigPnd:", "ShdPnd:"].into_iter().any(|key| {
            status
                .lines()
                .find_map(|line| line.strip_prefix(key))
                .and_then(|mask| u64::from_str_radix(mask.trim(), 16).ok())
                .is_some_and(|mask| mask & SIGKILL_BIT != 0)
        });
        if pending {
            Fate::KillPending(now.state)
        } else {
            Fate::Running(now.state)
        }
    }

    /// Block until the process has ended: an event of the kernel, not a time.
    fn wait_for_end(&self) {
        use rustix::event::{PollFd, PollFlags, poll};

        let mut fds = [PollFd::new(&self.fd, PollFlags::IN)];
        loop {
            match poll(&mut fds, None) {
                Ok(_) if fds[0].revents().contains(PollFlags::IN) => return,
                Ok(_) | Err(Errno::INTR) => {}
                Err(error) => panic!("poll of the pidfd of {}: {error}", self.pid.as_raw_pid()),
            }
        }
    }
}

/// SIGKILL (signal 9) in the masks of `/proc/<pid>/status`.
#[cfg(target_os = "linux")]
const SIGKILL_BIT: u64 = 1 << 8;

/// The fields of `/proc/<pid>/stat` the tests read.
#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy)]
struct Stat {
    state: char,
    pgrp: i32,
    /// When the process started, in ticks since boot: with the id, it names
    /// one process even if the id is used again.
    starttime: u64,
}

#[cfg(target_os = "linux")]
impl Stat {
    fn of(pid: Pid) -> Option<Self> {
        let text = std::fs::read_to_string(format!("/proc/{}/stat", pid.as_raw_pid())).ok()?;
        // The fields after the command name, which ends at the last `)`; the
        // state is the third field of the line, the group the fifth and the
        // start the twenty-second.
        let (_, rest) = text.rsplit_once(')')?;
        let fields: Vec<&str> = rest.split_whitespace().collect();
        Some(Self {
            state: fields.first()?.chars().next()?,
            pgrp: fields.get(2)?.parse().ok()?,
            starttime: fields.get(19)?.parse().ok()?,
        })
    }
}

/// How a process stood once its pidfd reported its end.
#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fate {
    /// Reaped.
    Gone,
    /// Dead and not reaped yet.
    Dead(char),
    /// SIGKILL is pending; the process is on its way out.
    KillPending(char),
    /// Still running, in this state, with no SIGKILL pending.
    Running(char),
}

#[cfg(target_os = "linux")]
impl Fate {
    /// Gone, dead, or dying of SIGKILL.
    fn ended(self) -> bool {
        !matches!(self, Self::Running(_))
    }
}

/// What became of the processes of a run once it was over.
#[cfg(target_os = "linux")]
#[derive(Debug)]
struct Settled {
    leader: Fate,
    descendant: Option<Fate>,
    leader_reaped: bool,
    marker: bool,
}

/// The kill reached the leader and what it started: the leader is reaped as
/// the run returns, the marker the script writes after the hold was never
/// written, and once each pidfd reported its end, each process is gone, dead
/// or dying of SIGKILL. Both are in the group the run started.
#[cfg(target_os = "linux")]
fn nothing_outlived_the_kill(run: &Outlived) {
    let settled = &run.settled;
    let leader = run.seen.leader.pid.as_raw_pid();
    assert_eq!(
        run.seen.leader.stat.pgrp, leader,
        "the leader leads a group"
    );
    if let Some(descendant) = &run.seen.descendant {
        assert_eq!(descendant.stat.pgrp, leader, "in the leader's group");
    }
    assert!(
        settled.leader_reaped,
        "the leader was not reaped: {settled:?}"
    );
    assert!(
        !settled.marker,
        "a process ran on past the kill and wrote the marker: {settled:?}"
    );
    assert!(settled.leader.ended(), "{settled:?}");
    assert!(settled.descendant.is_none_or(Fate::ended), "{settled:?}");
}

// ------------------------------------------------------------- time bound

#[cfg(unix)]
#[test]
fn a_process_that_outlives_its_limit_is_killed() {
    let run = outlive(Shape::Alone);
    killed_at_the_limit(&run);
    let text = run.result.as_ref().unwrap_err().to_string();
    assert!(
        text.ends_with("ran 2.0 s of its 2 s limit and was killed; the run has no verdict"),
        "{text}"
    );
    #[cfg(target_os = "linux")]
    {
        assert!(run.seen.descendant.is_none());
        nothing_outlived_the_kill(&run);
    }
}

#[cfg(unix)]
#[test]
fn a_process_inside_its_limit_passes() {
    let mut clock = Frozen::at(Duration::from_millis(7));
    let limits = Limits::trusted().with_timeout(Duration::from_secs(20));
    let output = runner()
        .run_with(&sh("echo done"), &limits, || &mut clock)
        .expect("the process runs to its end");
    assert!(output.success(), "{output:?}");
    assert_eq!(output.stdout, b"done\n");
    assert_eq!(output.stderr, b"");
    assert_eq!(
        output.elapsed,
        Duration::from_millis(7),
        "the clock's reading"
    );
}

/// The process starts one that holds the output pipes, as a test binary that
/// `cargo` started does, and waits for it; the run is killed at its limit and
/// nothing of the group survives.
#[cfg(unix)]
fn the_family_does_not_outlive_the_kill() {
    let run = outlive(Shape::Waits);
    killed_at_the_limit(&run);
    #[cfg(target_os = "linux")]
    {
        assert!(run.seen.descendant.is_some());
        nothing_outlived_the_kill(&run);
    }
}

#[cfg(unix)]
#[test]
fn what_the_process_started_does_not_outlive_the_kill() {
    the_family_does_not_outlive_the_kill();
}

#[cfg(unix)]
#[test]
fn the_group_is_killed_when_no_kill_program_is_on_the_path() {
    const MARKER: &str = "RAF_EXEC_TEST_WITHOUT_KILL";
    const NAME: &str =
        "exec::system::tests::the_group_is_killed_when_no_kill_program_is_on_the_path";
    if std::env::var_os(MARKER).is_none() {
        // Start this test again in a process whose `PATH` is an empty
        // directory; the `PATH` of this process is left as it is.
        let empty = Scratch::new();
        let output = Command::new(std::env::current_exe().expect("the test binary"))
            .args(["--exact", NAME, "--nocapture"])
            .env("PATH", &empty.0)
            .env(MARKER, "1")
            .stdin(Stdio::null())
            .output()
            .expect("the test binary starts");
        let log = String::from_utf8_lossy(&output.stdout).into_owned()
            + &String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{log}");
        assert!(
            log.contains("1 passed"),
            "the inner test did not run: {log}"
        );
        return;
    }
    // The precondition: this process can start no `kill` program by name.
    let missing = Command::new("kill")
        .stdin(Stdio::null())
        .status()
        .expect_err("no kill program on the path");
    assert_eq!(missing.kind(), io::ErrorKind::NotFound);
    the_family_does_not_outlive_the_kill();
}

#[cfg(unix)]
#[test]
fn a_descendant_that_holds_the_pipes_after_the_exit_is_not_waited_for_past_the_limit() {
    // The leader exits at once (on Linux the gate waits for that) and leaves a
    // process holding the pipes: the status is in, the streams are not, and the
    // run is killed at the limit, not waited for.
    let run = outlive(Shape::Leaves);
    killed_at_the_limit(&run);
    #[cfg(target_os = "linux")]
    {
        assert!(run.seen.descendant.is_some());
        nothing_outlived_the_kill(&run);
    }
}

// ----------------------------------------------------------- output bound

/// A command that writes `bytes` bytes to one stream with a builtin.
#[cfg(unix)]
fn writes(bytes: usize, stream: Stream) -> Invocation {
    let text = "x".repeat(bytes);
    match stream {
        Stream::Stdout => sh(&format!("printf %s {text}")),
        Stream::Stderr => sh(&format!("printf %s {text} >&2")),
    }
}

#[cfg(unix)]
#[test]
fn output_at_the_bound_passes_and_one_byte_over_is_refused_on_each_stream() {
    let ran = Duration::from_millis(1500);
    let limits = Limits::trusted().with_max_output(1000);
    for stream in [Stream::Stdout, Stream::Stderr] {
        let mut clock = Frozen::at(ran);
        let output = runner()
            .run_with(&writes(1000, stream), &limits, || &mut clock)
            .unwrap_or_else(|error| panic!("{stream}: {error}"));
        let (kept, other) = match stream {
            Stream::Stdout => (&output.stdout, &output.stderr),
            Stream::Stderr => (&output.stderr, &output.stdout),
        };
        assert_eq!(kept.len(), 1000, "{stream}");
        assert!(other.is_empty(), "{stream}");
        assert_eq!(output.elapsed, ran);

        let over = writes(1001, stream);
        let mut clock = Frozen::at(ran);
        let error = runner()
            .run_with(&over, &limits, || &mut clock)
            .expect_err("one byte over the bound");
        assert_eq!(
            error,
            ExecError::OutputTooLarge {
                command: over.to_string(),
                stream,
                limit: 1000,
                elapsed: ran,
            }
        );
        let text = error.to_string();
        assert!(
            text.ends_with(&format!(
                "wrote more than 1000 bytes to {stream} in 1.5 s and was killed; the run has no verdict"
            )),
            "{text}"
        );
    }
}

#[cfg(unix)]
#[test]
fn a_process_that_never_stops_writing_is_killed_at_the_bound() {
    // The clock never moves, so the limit cannot pass: only the bound can end
    // the run.
    let ran = Duration::from_secs(3);
    let mut clock = Frozen::at(ran);
    let endless = sh("exec yes");
    let limits = Limits::trusted()
        .with_max_output(4096)
        .with_timeout(Duration::from_mins(1));
    let error = runner()
        .run_with(&endless, &limits, || &mut clock)
        .expect_err("killed at the bound");
    assert_eq!(
        error,
        ExecError::OutputTooLarge {
            command: endless.to_string(),
            stream: Stream::Stdout,
            limit: 4096,
            elapsed: ran,
        }
    );
}
