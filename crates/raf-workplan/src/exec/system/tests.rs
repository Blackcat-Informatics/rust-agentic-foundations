// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Tests of the system runner.
//!
//! The runs of real processes go through [`SystemRunner::run_with`] with a
//! clock the test controls, so no test here asserts on, or waits for, the
//! clock of the machine; only the list pass in [`cargo`] runs under it, as a
//! watchdog whose reading nothing asserts. Under [`Frozen`] every look reads
//! the same time, so only the end of the process or an output bound can end
//! the run, and the time a run reports is that reading exactly. Under
//! [`Stepped`] each pause the runner asks for moves the reading by that much,
//! so the limit passes after a bounded number of looks whatever the speed of
//! the host. Both count every reading and every pause and fail the test past
//! a ceiling, so a runner that never decides, or that stops pausing, fails
//! instead of spinning.
//!
//! A process that has to outlive its limit first ignores SIGHUP, SIGINT and
//! SIGTERM, publishes its kernel credentials over a socket and blocks reading
//! another that the test holds open (the hold); a leader that exits and leaves
//! it behind first waits on a third (go) until its pidfd is taken. On Linux the
//! stepped clock reads the receipts without blocking and does not move until it
//! has a pidfd of each process, so a script that ends early ends the run
//! instead of stopping the test. Once the run is over the test releases the
//! hold, sends SIGCONT to each process and blocks on each pidfd until the
//! process has ended. A process killed with SIGKILL never ran the rest of its
//! script, so the marker file that the script writes after the hold is
//! absent; one that survived the kill, or was only stopped, writes the marker
//! and ends, so no step can block for ever.
//!
//! [`cargo`] runs a bound test and a bound example that never end through the
//! executor and the real `cargo` in the same way.

use std::io::{self, Cursor};
use std::time::Duration;

use super::{Capture, Look, POLL_CEILING, next_pause, verdict};
use crate::exec::Stream;

#[cfg(target_os = "linux")]
#[path = "../../../../../tests/support/process_receipt.rs"]
mod receipt;

#[cfg(target_os = "linux")]
#[path = "../../../../../tests/support/process_teardown.rs"]
#[allow(
    dead_code,
    reason = "shared teardown also supplies the actual CLI-death fixtures"
)]
mod teardown;

/// The direct-child termination path is shared by every platform. A pipe
/// controlled by this test keeps the descendant alive until after its leader
/// has actually been killed and reaped; no clock determines the ordering.
#[test]
fn killing_only_the_direct_child_leaves_its_descendant_running() {
    use std::io::{BufRead as _, Read as _, Write as _};
    use std::process::{Command, Stdio};

    const ROLE: &str = "RAF_DIRECT_CHILD_ROLE";
    const TEST: &str =
        "exec::system::tests::killing_only_the_direct_child_leaves_its_descendant_running";
    const READY: &str = "RAF_DESCENDANT_READY";
    if let Some(role) = std::env::var_os(ROLE) {
        direct_child_role(role.to_str().expect("an ASCII role"), ROLE, TEST, READY);
        return;
    }
    let root =
        purrdf_testkit::TempDir::new_in(std::env::temp_dir()).expect("create direct-child fixture");
    let (input, mut release) = io::pipe().expect("create descendant control pipe");
    let mut command = Command::new(std::env::current_exe().expect("the test executable"));
    command
        .args(["--exact", TEST, "--nocapture"])
        .env(ROLE, "leader")
        .env("RAF_DIRECT_CHILD_ROOT", root.path())
        .env("RUST_BACKTRACE", "0")
        .stdin(input)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    command.process_group(0);
    let mut leader = command.spawn().expect("start the owned leader");
    drop(command);
    let mut output = io::BufReader::new(leader.stdout.take().expect("leader output"));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut ready = false;
        for _ in 0..16 {
            let mut line = String::new();
            assert_ne!(output.read_line(&mut line).expect("read readiness"), 0);
            if line.trim_end() == READY {
                ready = true;
                break;
            }
        }
        assert!(ready, "the descendant acknowledged its inherited input");
        super::terminate_process(&mut leader).expect("kill and reap the direct child");
        assert!(
            !leader
                .try_wait()
                .expect("inspect reaped child")
                .unwrap()
                .success()
        );
        assert!(!root.path().join("continued").exists());
        release
            .write_all(b"x")
            .expect("release surviving descendant");
        let mut tail = String::new();
        output
            .read_to_string(&mut tail)
            .expect("descendant closes output");
        assert_eq!(
            std::fs::read(root.path().join("continued")).unwrap(),
            b"continued"
        );
    }));
    // EOF also releases the descendant if any assertion failed. Close its
    // output before cleanup so a failed child cannot block on a full pipe.
    drop(release);
    drop(output);
    drop(leader.stderr.take());
    if leader.try_wait().expect("inspect owned leader").is_none() {
        leader.kill().expect("clean up owned leader");
    }
    leader.wait().expect("reap owned leader");
    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}

fn direct_child_role(role: &str, role_key: &str, test: &str, ready: &str) {
    use std::io::{Read as _, Write as _};
    use std::process::{Command, Stdio};

    match role {
        "leader" => {
            let status = Command::new(std::env::current_exe().expect("the test executable"))
                .args(["--exact", test, "--nocapture"])
                .env(role_key, "descendant")
                .stdin(Stdio::inherit())
                .status()
                .expect("start and wait for descendant");
            assert!(status.success());
        }
        "descendant" => {
            println!("\n{ready}");
            io::stdout().flush().expect("publish descendant readiness");
            let mut byte = [0];
            io::stdin()
                .read_exact(&mut byte)
                .expect("controller releases input");
            assert_eq!(byte, [b'x']);
            let root = std::env::var_os("RAF_DIRECT_CHILD_ROOT").expect("owned fixture root");
            std::fs::write(
                std::path::PathBuf::from(root).join("continued"),
                b"continued",
            )
            .expect("write descendant continuation");
        }
        _ => panic!("unknown direct-child role: {role:?}"),
    }
}

#[cfg(unix)]
use rustix::io::Errno;
#[cfg(unix)]
use rustix::process::{Pid, WaitId, WaitIdOptions, waitid};
#[cfg(unix)]
use std::os::unix::process::CommandExt as _;
#[cfg(unix)]
use std::process::{Command, Stdio};

#[cfg(unix)]
use super::{
    SystemRunner, group_signal_outcome, kill_group, leader_ended, leader_signal_ends_it, terminate,
};
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

#[cfg(unix)]
#[test]
fn a_signal_to_the_leader_ends_it_only_when_delivered_or_it_is_gone() {
    assert!(leader_signal_ends_it(&Ok(())));
    assert!(leader_signal_ends_it(&Err(Errno::SRCH.into())));
    assert!(!leader_signal_ends_it(&Err(Errno::PERM.into())));
    assert!(!leader_signal_ends_it(&Err(io::Error::other(
        "not an errno"
    ))));
}

#[cfg(unix)]
#[test]
fn a_leader_that_exited_has_ended_and_is_collected() {
    let mut child = exited_group_leader();
    assert!(leader_ended(&mut child));
    let status = child.try_wait().expect("a status").expect("collected");
    assert_eq!(status.code(), Some(0), "the leader's own exit is kept");
}

#[cfg(unix)]
#[test]
fn a_leader_still_running_is_killed_and_collected_before_it_counts_as_ended() {
    use std::os::unix::process::ExitStatusExt as _;

    // `cat` blocks on a pipe the test holds open, so it is running until the
    // signal ends it.
    let mut child = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .expect("cat starts");
    assert!(leader_ended(&mut child));
    let status = child.try_wait().expect("a status").expect("collected");
    assert_eq!(status.signal(), Some(9), "the leader ended by SIGKILL");
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

    fn spawned(&mut self, process: u32) {
        (**self).spawned(process);
    }
}

/// The most looks, and the most pauses, a run under [`Frozen`] may make, and a
/// run under [`Stepped`] before its gate opens. Each pause there only yields
/// the thread, so a short process ends within a small part of this; a runner
/// that never decides reaches it and fails the test instead of spinning.
#[cfg(unix)]
const FROZEN_CEILING: usize = 5_000_000;

/// The most looks, and the most pauses, a run under [`Stepped`] may make once
/// its clock moves: twice the number of pauses of [`POLL_CEILING`] in `limit`.
/// The limit passes within that many pauses and five more (at most five of
/// the doubling pauses, then 20 ms each, then what remains), and the runner
/// reads the clock once at each look and once for the result; for the limit
/// of 2007 ms that is 105 pauses against a ceiling of 200.
#[cfg(unix)]
fn stepped_ceiling(limit: Duration) -> usize {
    let pauses = limit.as_millis() / POLL_CEILING.as_millis();
    usize::try_from(pauses).expect("a limit of the tests") * 2
}

/// The count of one kind of call to a test clock. Past its ceiling it fails
/// the test, so a runner that never decides fails whether it pauses or not.
#[cfg(unix)]
struct Count {
    what: &'static str,
    done: std::cell::Cell<usize>,
    ceiling: usize,
}

#[cfg(unix)]
impl Count {
    fn new(what: &'static str, ceiling: usize) -> Self {
        Self {
            what,
            done: std::cell::Cell::new(0),
            ceiling,
        }
    }

    fn tick(&self) {
        let done = self.done.get() + 1;
        self.done.set(done);
        assert!(
            done <= self.ceiling,
            "the run made {done} {}, past the ceiling of {}",
            self.what,
            self.ceiling
        );
    }
}

/// A clock whose reading never moves: each reading and each pause is counted,
/// and a pause yields the thread.
#[cfg(unix)]
struct Frozen {
    elapsed: Duration,
    looks: Count,
    pauses: Count,
}

#[cfg(unix)]
impl Frozen {
    /// A clock that reads `elapsed` at every look.
    fn at(elapsed: Duration) -> Self {
        Self {
            elapsed,
            looks: Count::new("looks under a clock that never moves", FROZEN_CEILING),
            pauses: Count::new("pauses under a clock that never moves", FROZEN_CEILING),
        }
    }
}

#[cfg(unix)]
impl Clock for Frozen {
    fn elapsed(&self) -> Duration {
        self.looks.tick();
        self.elapsed
    }

    fn pause(&mut self, _pause: Duration) {
        self.pauses.tick();
        std::thread::yield_now();
    }
}

/// A clock that moves by each pause the runner asks for, and nothing else;
/// each reading and each pause is counted, against the ceiling of its limit
/// (see [`stepped_ceiling`]). On Linux it has a gate (an [`Opening`]): until
/// the gate opens, a pause does not move the clock, so the processes of a
/// [`Subject`] are known before the limit can pass.
#[cfg(unix)]
struct Stepped {
    now: Duration,
    looks: Count,
    pauses: Count,
    /// The ceiling once the gate has opened.
    #[cfg(target_os = "linux")]
    ceiling: usize,
    #[cfg(target_os = "linux")]
    gate: Option<Box<dyn Opening>>,
    #[cfg(target_os = "linux")]
    seen: Option<Seen>,
    /// An already-ended launcher leaves only asynchronous reader completion.
    #[cfg(target_os = "linux")]
    completed: bool,
}

#[cfg(unix)]
impl Stepped {
    fn moving(ceiling: usize) -> (Count, Count) {
        (
            Count::new("looks under a stepped clock", ceiling),
            Count::new("pauses under a stepped clock", ceiling),
        )
    }

    /// A clock that moves from its first pause, for a run of `limit`.
    fn new(limit: Duration) -> Self {
        let (looks, pauses) = Self::moving(stepped_ceiling(limit));
        Self {
            now: Duration::ZERO,
            looks,
            pauses,
            #[cfg(target_os = "linux")]
            ceiling: stepped_ceiling(limit),
            #[cfg(target_os = "linux")]
            gate: None,
            #[cfg(target_os = "linux")]
            seen: None,
            #[cfg(target_os = "linux")]
            completed: false,
        }
    }

    /// A clock for a run of `limit` that moves once `gate` has opened.
    #[cfg(target_os = "linux")]
    fn gated(gate: impl Opening + 'static, limit: Duration) -> Self {
        Self {
            now: Duration::ZERO,
            looks: Count::new("looks before the gate opened", FROZEN_CEILING),
            pauses: Count::new("pauses before the gate opened", FROZEN_CEILING),
            ceiling: stepped_ceiling(limit),
            gate: Some(Box::new(gate)),
            seen: None,
            completed: false,
        }
    }
}

#[cfg(unix)]
impl Clock for Stepped {
    fn elapsed(&self) -> Duration {
        self.looks.tick();
        self.now
    }

    fn pause(&mut self, pause: Duration) {
        #[cfg(target_os = "linux")]
        if let Some(gate) = &mut self.gate {
            if let Some(seen) = gate.turn() {
                self.completed = seen.leader.ended();
                self.seen = Some(seen);
                self.gate = None;
                if !self.completed {
                    (self.looks, self.pauses) = Self::moving(self.ceiling);
                }
            } else {
                self.pauses.tick();
                std::thread::yield_now();
                return;
            }
        }
        #[cfg(target_os = "linux")]
        if self.completed {
            // The process has ended. Let capture/admission threads deliver
            // their end events under the original counted frozen allowance;
            // advancing synthetic time here would assert scheduling latency.
            self.pauses.tick();
            std::thread::yield_now();
            return;
        }
        self.pauses.tick();
        self.now += pause;
    }

    fn spawned(&mut self, process: u32) {
        #[cfg(target_os = "linux")]
        if let Some(gate) = &mut self.gate {
            gate.spawned(process);
        }
        #[cfg(not(target_os = "linux"))]
        let _ = process;
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

/// A native exclusively created temporary directory, removed by its owner.
#[cfg(unix)]
struct Scratch(purrdf_testkit::TempDir);

#[cfg(unix)]
impl Scratch {
    fn new() -> Self {
        Self(purrdf_testkit::TempDir::new_in(std::env::temp_dir()).expect("create process fixture"))
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

/// The limit of the runs that outlive it: two seconds and seven milliseconds.
/// The pauses before the gate opens do not move a stepped clock, so the
/// moving pauses may start anywhere in the runner's schedule and add up to
/// 31, 30, 28, 24, 16 or 0 ms and then 20 ms each; none of those sums is this
/// limit, so a pause that is not cut to what remains passes it.
#[cfg(unix)]
const LIMIT: Duration = Duration::from_millis(2007);

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

/// Run a process of `shape` under a stepped clock with the limit [`LIMIT`].
#[cfg(target_os = "linux")]
fn outlive(shape: Shape) -> Outlived {
    let subject = Subject::new();
    let invocation = sh(&subject.script(shape));
    let mut clock = Stepped::gated(subject.gate(shape), LIMIT);
    let limits = Limits::trusted().with_timeout(LIMIT);
    let result = runner().run_with(&invocation, &limits, || &mut clock);
    let Some(seen) = clock.seen.take() else {
        panic!("the run ended before the gate opened on the processes of its script: {result:?}");
    };
    let settled = subject.settle(&seen);
    Outlived {
        command: invocation.to_string(),
        result,
        seen,
        settled,
    }
}

/// Run a process of `shape` under a stepped clock with the limit [`LIMIT`].
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
    let mut clock = Stepped::new(LIMIT);
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
    /// Kernel credentials identify the sender in the receiving host namespace.
    pids: std::os::unix::net::UnixDatagram,
    /// The hold FIFO, open for reading and writing, so the release is a plain
    /// write that lands whether or not the script has opened it yet.
    hold: std::fs::File,
    /// The FIFO a leader that leaves its descendant behind reads before it
    /// exits, open the same way: the leader stays unreaped until its pidfd is
    /// taken.
    go: std::fs::File,
}

#[cfg(target_os = "linux")]
impl Subject {
    fn new() -> Self {
        use std::os::unix::fs::OpenOptionsExt as _;

        let scratch = Scratch::new();
        let fifo = |name: &str, flags: rustix::fs::OFlags| {
            let path = scratch.0.path().join(name);
            rustix::fs::mknodat(
                rustix::fs::CWD,
                &path,
                rustix::fs::FileType::Fifo,
                rustix::fs::Mode::from_raw_mode(0o600),
                0,
            )
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            let flags = i32::try_from(flags.bits()).expect("open flags");
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(flags)
                .open(&path)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        };
        let pids =
            receipt::listener(&scratch.0.path().join("pids")).expect("process receipt socket");
        let hold = fifo("hold", rustix::fs::OFlags::empty());
        let go = fifo("go", rustix::fs::OFlags::empty());
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
        let path = self.scratch.0.path().join(name);
        let text = path.to_str().expect("a UTF-8 temporary directory");
        assert!(!text.contains('\''), "{text}");
        format!("'{text}'")
    }

    fn marker(&self) -> std::path::PathBuf {
        self.scratch.0.path().join("marker")
    }

    /// Ignore catchable terminal signals, then replace the shell with the
    /// native fixture role. Its descendants inherit the ignored dispositions.
    /// Kernel credential receipts precede the controlled FIFO releases.
    fn script(&self, shape: Shape) -> String {
        let command = TrustedArgv::new(vec![
            std::env::current_exe()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
            "--exact".into(),
            "exec::system::tests::a_process_that_outlives_its_limit_is_killed".into(),
            "--nocapture".into(),
        ])
        .unwrap()
        .text();
        format!(
            "trap '' HUP INT TERM; RAF_EXEC_SUBJECT_ROOT={} RAF_EXEC_SUBJECT_SHAPE={shape:?} RAF_EXEC_SUBJECT_ROLE=leader exec {command}",
            self.quoted("")
        )
    }

    /// The gate of a stepped clock for a process of `shape`.
    fn gate(&self, shape: Shape) -> Gate {
        let clone = |file: &std::fs::File| file.try_clone().expect("a FIFO is shared");
        Gate {
            pids: self.pids.try_clone().expect("process receipt socket clone"),
            leader: None,
            payload: None,
            descendant: None,
            expects_descendant: shape != Shape::Alone,
            go: (shape == Shape::Leaves).then(|| clone(&self.go)),
            found: None,
        }
    }

    /// What became of the processes once the run is over: whether the leader
    /// is reaped as the run returns; then, with the hold released, each
    /// process sent SIGCONT (so one that a stop left behind runs on) and each
    /// process ended, whether the marker was written. A process the kill
    /// reached never read the release, so the marker is there only if one
    /// survived the kill.
    fn settle(&self, seen: &Seen) -> Settled {
        use rustix::process::{Signal, pidfd_send_signal};
        use std::io::Write as _;
        use std::os::fd::AsFd as _;

        let reaped = waitid(
            WaitId::PidFd(seen.leader.fd.as_fd()),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG,
        );
        (&self.hold).write_all(b"\n").expect("the hold is released");
        for process in seen.processes() {
            // A process that has ended is not an error here.
            let _ = pidfd_send_signal(&process.fd, Signal::CONT);
        }
        for process in seen.processes() {
            process.wait_for_end();
        }
        Settled {
            leader_reaped: matches!(reaped, Err(Errno::CHILD)),
            marker: self.marker().exists(),
        }
    }
}

/// What a [`Stepped`] clock waits for before it moves.
#[cfg(target_os = "linux")]
trait Opening {
    /// The runner has started the process of the run with this id.
    fn spawned(&mut self, _process: u32) {}

    /// One turn, at a pause of the runner: the processes once the limit may
    /// pass, or `None` to look again at the next pause.
    fn turn(&mut self) -> Option<Seen>;
}

/// The gate of a [`Subject`] script: the process credentials it publishes, a
/// pidfd of each, and for a leader that exits, its end. Each turn looks
/// without blocking.
#[cfg(target_os = "linux")]
struct Gate {
    /// The readiness socket, read without blocking.
    pids: std::os::unix::net::UnixDatagram,
    leader: Option<Process>,
    payload: Option<Process>,
    descendant: Option<Process>,
    expects_descendant: bool,
    /// The go FIFO, for a leader that exits.
    go: Option<std::fs::File>,
    /// The processes, once their pidfds are taken.
    found: Option<Seen>,
}

#[cfg(target_os = "linux")]
impl Opening for Gate {
    fn spawned(&mut self, process: u32) {
        self.leader = Some(Process::open(&process.to_string()));
    }

    fn turn(&mut self) -> Option<Seen> {
        use std::io::Write as _;

        if self.found.is_none() {
            match receipt::receive(&self.pids) {
                Ok((role, pid)) => {
                    let process = Process::open(&pid.as_raw_pid().to_string());
                    match role.as_str() {
                        "leader" => assert!(self.payload.replace(process).is_none()),
                        "descendant" => assert!(self.descendant.replace(process).is_none()),
                        _ => panic!("unknown process receipt role {role:?}"),
                    }
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    ) => {}
                Err(error) => panic!("the credential receipt is read: {error}"),
            }
            if self.payload.is_none() || (self.expects_descendant && self.descendant.is_none()) {
                return None;
            }
            let leader = self.leader.take().expect("the runner's actual launcher");
            let payload = self.payload.take();
            let descendant = self.descendant.take();
            let teardown = descendant.as_ref().map(|process| {
                teardown::Teardown::capture(process.pid, rustix::process::getpid())
                    .expect("capture the owned namespace before releasing any process")
            });
            if let Some(go) = &self.go {
                (&*go).write_all(b"\n").expect("the leader is let go");
            } else {
                assert!(
                    !leader.ended(),
                    "the process ended before its limit could pass"
                );
            }
            self.found = Some(Seen {
                leader,
                payload,
                descendant,
                teardown,
            });
        }
        let leader_exits = self.go.is_some();
        let seen = self.found.as_ref()?;
        if leader_exits && !seen.leader.ended() {
            return None;
        }
        self.found.take()
    }
}

/// The processes of a run as the gate found them. Dropping it sends SIGKILL to
/// each of them through its pidfd, and on a failing test to the group as
/// well, so a run that fails leaves nothing behind; after a clean end the
/// group's id may already belong to another group.
#[cfg(target_os = "linux")]
struct Seen {
    leader: Process,
    payload: Option<Process>,
    descendant: Option<Process>,
    teardown: Option<teardown::Teardown>,
}

#[cfg(target_os = "linux")]
impl Seen {
    fn processes(&self) -> impl Iterator<Item = &Process> {
        std::iter::once(&self.leader)
            .chain(&self.payload)
            .chain(&self.descendant)
    }
}

#[cfg(target_os = "linux")]
impl Drop for Seen {
    fn drop(&mut self) {
        use rustix::process::{Signal, kill_process_group, pidfd_send_signal};

        for process in self.processes() {
            let _ = pidfd_send_signal(&process.fd, Signal::KILL);
        }
        if std::thread::panicking() {
            let _ = kill_process_group(self.leader.pid, Signal::KILL);
        }
    }
}

/// One process, held by a pidfd, and its group when the pidfd was taken.
#[cfg(target_os = "linux")]
struct Process {
    pid: Pid,
    fd: std::os::fd::OwnedFd,
    pgrp: i32,
}

#[cfg(target_os = "linux")]
impl Process {
    fn open(word: &str) -> Self {
        use rustix::process::{PidfdFlags, pidfd_open};

        let raw = purrdf_xsd::numeric::parse_integer(word)
            .ok()
            .and_then(|number| number.as_i128())
            .and_then(|number| i32::try_from(number).ok())
            .expect("a process id in the signed 32-bit range");
        let pid = Pid::from_raw(raw).expect("a process id");
        // The process is blocked reading a FIFO the test holds and is not
        // reaped, so the id is still this process.
        let fd = pidfd_open(pid, PidfdFlags::empty())
            .unwrap_or_else(|error| panic!("a pidfd of process {raw}, still running: {error}"));
        let pgrp = group_of(pid).expect("the stat of a live process");
        Self { pid, fd, pgrp }
    }

    /// Whether the process has ended, without blocking.
    fn ended(&self) -> bool {
        use rustix::event::{PollFd, PollFlags, Timespec, poll};

        let no_wait = Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        let mut fds = [PollFd::new(&self.fd, PollFlags::IN)];
        loop {
            match poll(&mut fds, Some(&no_wait)) {
                Ok(_) => return fds[0].revents().contains(PollFlags::IN),
                Err(Errno::INTR) => {}
                Err(error) => panic!("poll of the pidfd of {}: {error}", self.pid.as_raw_pid()),
            }
        }
    }

    /// Block until the process has ended: an event of the kernel, not a time.
    /// It is asked for only after the hold is released and SIGCONT sent, when
    /// a process that survived the kill has nothing left to wait on.
    fn wait_for_end(&self) {
        receipt::wait(&self.fd).expect("owned process terminates");
    }
}

/// The process group of `pid`, the fifth field of `/proc/<pid>/stat` (the
/// fields after the command name, which ends at the last `)`, begin with the
/// third).
#[cfg(target_os = "linux")]
fn group_of(pid: Pid) -> Option<i32> {
    let text = std::fs::read_to_string(format!("/proc/{}/stat", pid.as_raw_pid())).ok()?;
    let (_, rest) = text.rsplit_once(')')?;
    purrdf_xsd::numeric::parse_integer(rest.split_whitespace().nth(2)?)
        .ok()?
        .as_i128()
        .and_then(|number| i32::try_from(number).ok())
}

/// What became of the processes of a run once it was over.
#[cfg(target_os = "linux")]
#[derive(Debug)]
struct Settled {
    leader_reaped: bool,
    marker: bool,
}

/// The kill reached the leader and what it started: both are in the group
/// the run started, the leader is reaped as the run returns, and the marker
/// the script writes after the hold was never written. The script ignores
/// SIGHUP, SIGINT and SIGTERM and every process gets SIGCONT after the
/// release, so only SIGKILL keeps the marker away.
#[cfg(target_os = "linux")]
fn nothing_outlived_the_kill(seen: &Seen, settled: &Settled) {
    let leader = seen.leader.pid.as_raw_pid();
    assert_eq!(seen.leader.pgrp, leader, "the leader leads a group");
    if let Some(descendant) = &seen.descendant {
        assert_eq!(descendant.pgrp, leader, "in the leader's group");
    }
    assert!(
        settled.leader_reaped,
        "the leader was not reaped: {settled:?}"
    );
    assert!(
        !settled.marker,
        "a process ran on past the kill and wrote the marker: {settled:?}"
    );
}

// ------------------------------------------------------------- time bound

#[cfg(unix)]
#[test]
fn a_process_that_outlives_its_limit_is_killed() {
    #[cfg(target_os = "linux")]
    if let Some(root) = std::env::var_os("RAF_EXEC_SUBJECT_ROOT") {
        subject_process(std::path::Path::new(&root));
        return;
    }
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
        nothing_outlived_the_kill(&run.seen, &run.settled);
    }
}

#[cfg(target_os = "linux")]
fn subject_process(root: &std::path::Path) {
    use std::io::BufRead as _;
    use std::os::unix::net::UnixDatagram;

    let role = std::env::var("RAF_EXEC_SUBJECT_ROLE").expect("owned process role");
    let shape = std::env::var("RAF_EXEC_SUBJECT_SHAPE").expect("owned process shape");
    if role == "descendant"
        && let Ok(method) = std::env::var("RAF_EXEC_SUBJECT_ESCAPE")
    {
        match method.as_str() {
            "setsid" => {
                rustix::process::setsid().expect("detach owned descendant");
            }
            "setpgid" => {
                rustix::process::setpgid(None, None).expect("detach owned group");
            }
            _ => panic!("unknown escape operation {method:?}"),
        }
    }
    let mut descendant = if role == "leader" && shape != "Alone" {
        Some(
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "exec::system::tests::a_process_that_outlives_its_limit_is_killed",
                    "--nocapture",
                ])
                .env("RAF_EXEC_SUBJECT_ROLE", "descendant")
                .stdin(Stdio::null())
                .spawn()
                .expect("owned descendant starts"),
        )
    } else {
        None
    };
    UnixDatagram::unbound()
        .unwrap()
        .send_to(role.as_bytes(), root.join("pids"))
        .expect("send kernel process credentials");
    let leaving = role == "leader" && shape == "Leaves";
    if role == "leader" && shape == "Waits" {
        assert!(descendant.as_mut().unwrap().wait().unwrap().success());
    } else {
        let mut released = String::new();
        io::BufReader::new(
            std::fs::File::open(root.join(if leaving { "go" } else { "hold" })).unwrap(),
        )
        .read_line(&mut released)
        .unwrap();
        assert_eq!(released, "\n");
        if !leaving {
            std::fs::write(root.join("marker"), "survived\n").unwrap();
        }
    }
}

#[cfg(unix)]
#[test]
fn a_process_inside_its_limit_passes() {
    #[cfg(target_os = "linux")]
    if let Some(mode) = std::env::var_os("RAF_EXEC_CONTAINMENT_PREREQUISITE") {
        let limits = Limits::trusted().with_max_output(if mode == "overflow" { 8 } else { 4096 });
        let result = if mode == "hang" {
            let mut clock = Stepped::new(limits.timeout());
            runner().run_with(&sh("exit 17"), &limits, || &mut clock)
        } else {
            let mut clock = Frozen::at(Duration::ZERO);
            runner().run_with(&sh("exit 17"), &limits, || &mut clock)
        };
        let error = result.expect_err("unavailable containment cannot become a payload verdict");
        if mode == "missing" {
            assert!(matches!(
                error,
                ExecError::Spawn {
                    kind: io::ErrorKind::NotFound,
                    ..
                }
            ));
        } else {
            let diagnostic = error.to_string();
            assert!(matches!(error, ExecError::Io { .. }), "{diagnostic}");
            assert!(diagnostic.contains("Linux PID containment"), "{diagnostic}");
            if mode == "denied" {
                assert!(
                    diagnostic.contains("fixture namespace unavailable"),
                    "{diagnostic}"
                );
            }
        }
        return;
    }
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
    #[cfg(target_os = "linux")]
    for (script, code) in [("exit 137", 137), ("kill -KILL $$", 137), ("exit 0", 0)] {
        let mut clock = Frozen::at(Duration::ZERO);
        let output = runner()
            .run_with(&sh(script), &limits.with_max_output(0), || &mut clock)
            .expect("successful namespace admission is distinct from payload failure");
        assert_eq!(output.code, Some(code));
        assert_eq!(output.success(), code == 0);
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
    }
    #[cfg(target_os = "linux")]
    containment_prerequisite_neighbors();
}

#[cfg(target_os = "linux")]
fn containment_prerequisite_neighbors() {
    use std::os::unix::fs::PermissionsExt as _;

    for mode in ["missing", "denied", "overflow", "hang"] {
        let directory = Scratch::new();
        if mode != "missing" {
            let launcher = directory.0.path().join("bwrap");
            let body = match mode {
                "overflow" => {
                    "#!/bin/sh\nprintf 'fixture namespace unavailable overflow\\n' >&2\nexit 1\n"
                }
                "hang" => "#!/bin/sh\nwhile :; do :; done\n",
                _ => "#!/bin/sh\nprintf 'fixture namespace unavailable\\n' >&2\nexit 1\n",
            };
            std::fs::write(&launcher, body).unwrap();
            std::fs::set_permissions(launcher, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "exec::system::tests::a_process_inside_its_limit_passes",
                "--nocapture",
            ])
            .env("RAF_EXEC_CONTAINMENT_PREREQUISITE", mode)
            .env("PATH", directory.0.path())
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            output.stdout.escape_ascii(),
            output.stderr.escape_ascii()
        );
    }
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
        nothing_outlived_the_kill(&run.seen, &run.settled);
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
        #[cfg(target_os = "linux")]
        {
            let path = std::env::var_os("PATH").expect("the native containment tool is installed");
            let bwrap = std::env::split_paths(&path)
                .map(|directory| directory.join("bwrap"))
                .find(|path| path.is_file())
                .expect("mandatory bwrap is on PATH")
                .canonicalize()
                .unwrap();
            std::os::unix::fs::symlink(bwrap, empty.0.path().join("bwrap")).unwrap();
        }
        let output = Command::new(std::env::current_exe().expect("the test binary"))
            .args(["--exact", NAME, "--nocapture"])
            .env("PATH", empty.0.path())
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
    // The initial command exits after both pidfds are held. Namespace teardown
    // ends the descendant that retained the pipes without waiting for the limit.
    let run = outlive(Shape::Leaves);
    assert!(
        run.result
            .as_ref()
            .expect("the initial command exits")
            .success()
    );
    #[cfg(target_os = "linux")]
    {
        assert!(run.seen.descendant.is_some());
        nothing_outlived_the_kill(&run.seen, &run.settled);
    }
}

#[cfg(target_os = "linux")]
#[test]
fn a_descendant_that_leaves_its_group_is_killed_with_its_namespace() {
    for method in ["setsid", "setpgid"] {
        for shape in [Shape::Waits, Shape::Leaves] {
            let subject = Subject::new();
            let invocation = sh(&format!(
                "export RAF_EXEC_SUBJECT_ESCAPE={method}; {}",
                subject.script(shape)
            ));
            let mut clock = Stepped::gated(subject.gate(shape), LIMIT);
            let result =
                runner().run_with(&invocation, &Limits::trusted().with_timeout(LIMIT), || {
                    &mut clock
                });
            let seen = clock
                .seen
                .take()
                .expect("both processes acknowledged before the controlled limit");
            if shape == Shape::Leaves {
                assert!(
                    result
                        .expect("the command exits after its receipt")
                        .success()
                );
            } else {
                assert_eq!(
                    result,
                    Err(ExecError::Timeout {
                        command: invocation.to_string(),
                        timeout: LIMIT,
                        elapsed: LIMIT,
                    })
                );
            }
            let escaped = seen.descendant.as_ref().expect("the escaped descendant");
            assert_ne!(escaped.pgrp, seen.leader.pgrp);
            assert_eq!(escaped.pgrp, escaped.pid.as_raw_pid());
            seen.teardown
                .as_ref()
                .expect("the detached process has an owned namespace receipt")
                .after_launcher_death()
                .expect("the namespace init cannot survive its reaped launcher");
            assert!(
                !subject.marker().exists(),
                "the detached child did not continue"
            );
            let settled = subject.settle(&seen);
            assert!(settled.leader_reaped);
            assert!(
                !settled.marker,
                "the detached process cannot continue after namespace teardown"
            );
            assert!(escaped.ended(), "the released escaped process has ended");
        }
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

#[cfg(unix)]
#[test]
fn both_streams_are_drained_at_once() {
    // Standard error is written in full first, then standard output, each
    // more than a pipe holds. A runner that read one stream to its end before
    // the other would wait for output that cannot come; the clock never
    // moves, so the limit cannot end that wait, and the ceiling of the clock
    // fails the test instead.
    let ran = Duration::from_millis(900);
    let mut clock = Frozen::at(ran);
    let both =
        sh("head -c 400000 /dev/zero | tr '\\0' e >&2; head -c 400000 /dev/zero | tr '\\0' o");
    let output = runner()
        .run_with(&both, &Limits::trusted(), || &mut clock)
        .expect("both streams are read to their end");
    assert!(output.success(), "{:?}", output.code);
    assert_eq!(output.stderr.len(), 400_000);
    assert_eq!(output.stdout.len(), 400_000);
    assert!(output.stderr.iter().all(|byte| *byte == b'e'));
    assert!(output.stdout.iter().all(|byte| *byte == b'o'));
    assert_eq!(output.elapsed, ran);
}

#[cfg(target_os = "linux")]
mod cargo;
