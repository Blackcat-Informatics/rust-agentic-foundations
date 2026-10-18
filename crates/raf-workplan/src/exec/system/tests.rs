// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Tests of the system runner.

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
use super::{group_signal_outcome, kill_group, terminate};

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
