// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A bound test and a bound example that never end, run by the executor
//! through the real `cargo` against a crate generated into a scratch
//! directory, on Linux.
//!
//! The executor runs every process of a binding through one runner, and the
//! runner here picks the clock of each invocation ([`Clocked`]): the clock of
//! the machine for the list pass of a test, which ends by itself, and a
//! [`Stepped`] clock for the one other run, which moves only by the pauses the
//! runner asks for. A limit of five seconds then passes after a bounded number
//! of looks whatever the speed of the host, and nothing waits for it.
//!
//! The test and the example of the crate call one function of its library: it
//! ignores SIGHUP, SIGINT and SIGTERM, writes the id of its process to the pid
//! FIFO of a [`Subject`], blocks reading the hold FIFO and then writes the
//! marker. The gate of the stepped clock ([`Watch`]) takes a pidfd of the
//! process the runner starts (`cargo`) as it starts, and blocks until the id
//! is written or that process has ended, so a `cargo` that ends first ends the
//! run instead of stopping the test. With the id read it takes a pidfd of the
//! process that wrote it and lets the clock move. Once the run is over the
//! hold is released, each process is sent SIGCONT and waited for through its
//! pidfd: a process that the kill reached never wrote the marker, and only
//! SIGKILL keeps it away.

use std::cell::RefCell;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use raf_source::{DiskTree, MemoryTree, Workspace};
use rustix::io::Errno;

use super::{Opening, Process, Seen, Settled, Stepped, Subject, nothing_outlived_the_kill};
use crate::Workplan;
use crate::bound::{BoundOutcome, Ended, Evidence, Executor};
use crate::exec::clock::WallClock;
use crate::exec::system::SystemRunner;
use crate::exec::{BUILD_TIMEOUT, ExecError, Invocation, Limits, Output, Programs, Runner};

/// The limit of the runs that never end: five seconds of the stepped clock.
const LIMIT: Duration = Duration::from_secs(5);

/// The packages of the generated crates: one for the test and one for the
/// example. A target directory named by the environment (`CARGO_TARGET_DIR`)
/// overrides the one each crate names, and is then shared by every crate; two
/// names, and the example in only one of them, keep the two tests of this
/// process from rewriting each other's executables there.
const TEST_PACKAGE: &str = "fx-forever-test";

/// The package of the crate with the example (see [`TEST_PACKAGE`]).
const EXAMPLE_PACKAGE: &str = "fx-forever-example";

/// The `cargo` that runs this test binary: `$CARGO` as `cargo test` sets it.
fn cargo() -> PathBuf {
    std::env::var_os("CARGO")
        .map(PathBuf::from)
        .or_else(|| option_env!("CARGO").map(PathBuf::from))
        .expect("cargo test sets CARGO")
}

// ------------------------------------------------------ the runner and gate

/// The system runner with a clock chosen for each invocation. A list pass
/// runs under the clock of the machine and the limit of the gate
/// ([`BUILD_TIMEOUT`]): it ends by itself, and what it takes is not what is
/// tested. Any other run takes the stepped clock, once, under the limits the
/// executor asked for. The limits asked for each invocation are kept.
struct Clocked {
    runner: SystemRunner,
    stepped: RefCell<Option<Stepped>>,
    ran: RefCell<Option<Stepped>>,
    asked: RefCell<Vec<(Vec<String>, Limits)>>,
}

impl Clocked {
    fn new(runner: SystemRunner, stepped: Stepped) -> Self {
        Self {
            runner,
            stepped: RefCell::new(Some(stepped)),
            ran: RefCell::new(None),
            asked: RefCell::new(Vec::new()),
        }
    }
}

impl Runner for Clocked {
    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError> {
        let argv = invocation.argv();
        let list = argv.iter().any(|word| word == "--list");
        self.asked.borrow_mut().push((argv, *limits));
        if list {
            let gate = limits.with_timeout(BUILD_TIMEOUT);
            return self.runner.run_with(invocation, &gate, WallClock::start);
        }
        let mut clock = self
            .stepped
            .borrow_mut()
            .take()
            .expect("one run under the stepped clock");
        let result = self.runner.run_with(invocation, limits, || &mut clock);
        *self.ran.borrow_mut() = Some(clock);
        result
    }
}

/// The gate of a run through `cargo`: a pidfd of the process the runner
/// started, taken as it starts, then the process id the test or the example
/// writes to the pid FIFO. A turn blocks until the id is written or the
/// started process has ended, events of the kernel and not a time.
struct Watch {
    /// The pid FIFO, open for reading and writing since before the build and
    /// read without blocking once the poll has found it readable.
    pids: std::fs::File,
    /// What has been read of the line.
    line: Vec<u8>,
    /// The process the runner started, until the gate opens.
    leader: Option<Process>,
    /// Whether the process the runner started ended before the id came.
    leader_ended: bool,
}

impl Watch {
    fn new(subject: &Subject) -> Self {
        Self {
            pids: subject.pids.try_clone().expect("a FIFO is shared"),
            line: Vec::new(),
            leader: None,
            leader_ended: false,
        }
    }
}

impl Opening for Watch {
    fn spawned(&mut self, process: u32) {
        assert!(
            self.leader.is_none(),
            "one process is started under the stepped clock"
        );
        // The runner has not collected it yet, so the id is still this process.
        self.leader = Some(Process::open(&process.to_string()));
    }

    fn turn(&mut self) -> Option<Seen> {
        use rustix::event::{PollFd, PollFlags, poll};
        use std::io::Read as _;

        if self.leader_ended {
            return None;
        }
        let leader = self
            .leader
            .as_ref()
            .expect("the runner names the process it started");
        let (written, ended) = {
            let mut fds = [
                PollFd::new(&self.pids, PollFlags::IN),
                PollFd::new(&leader.fd, PollFlags::IN),
            ];
            match poll(&mut fds, None) {
                Ok(_) => {}
                Err(Errno::INTR) => return None,
                Err(error) => panic!("poll of the pid FIFO and of `cargo`: {error}"),
            }
            (
                fds[0].revents().contains(PollFlags::IN),
                fds[1].revents().contains(PollFlags::IN),
            )
        };
        if written {
            let mut chunk = [0_u8; 64];
            match (&self.pids).read(&mut chunk) {
                // The test holds a writer, so a read never finds the end.
                Ok(0) => panic!("the pid FIFO ended"),
                Ok(read) => self.line.extend_from_slice(&chunk[..read]),
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                    ) => {}
                Err(error) => panic!("the pid FIFO is read: {error}"),
            }
        }
        if !self.line.contains(&b'\n') {
            self.leader_ended = ended;
            return None;
        }
        let line = String::from_utf8(std::mem::take(&mut self.line)).expect("a process id");
        let word = line.trim();
        let leader = self.leader.take()?;
        // `cargo run` on Unix replaces itself with the example, which then
        // writes the id of the process the runner started.
        let descendant = (word != leader.pid.as_raw_pid().to_string()).then(|| Process::open(word));
        assert!(!leader.ended(), "`cargo` ended before the limit could pass");
        Some(Seen { leader, descendant })
    }
}

impl Drop for Watch {
    /// A run that fails before the gate opens leaves nothing behind.
    fn drop(&mut self) {
        use rustix::process::{Signal, kill_process_group, pidfd_send_signal};

        if let Some(leader) = &self.leader {
            let _ = pidfd_send_signal(&leader.fd, Signal::KILL);
            if std::thread::panicking() {
                let _ = kill_process_group(leader.pid, Signal::KILL);
            }
        }
    }
}

// ------------------------------------------------------- the generated crate

/// A crate generated into the scratch directory of a [`Subject`], whose test
/// `forever`, and example `forever` when it has one, never end by themselves,
/// and whose test `passes` passes. It builds into a target directory of its
/// own beside it, and both go with the scratch directory.
struct Crate {
    dir: PathBuf,
    source: Workspace,
}

impl Crate {
    fn generate(subject: &Subject, package: &str, example: bool) -> Self {
        let root = subject.scratch.0.path();
        let dir = root.join("crate");
        let target = root.join("target");
        let put = |path: &str, text: &str| {
            let file = dir.join(path);
            std::fs::create_dir_all(file.parent().expect("a file has a directory"))
                .expect("the directory of a file");
            std::fs::write(file, text).expect("a file");
        };
        put(
            "Cargo.toml",
            &format!(
                "[package]\nname = \"{package}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n"
            ),
        );
        put(
            "Cargo.lock",
            &format!("version = 4\n\n[[package]]\nname = \"{package}\"\nversion = \"0.1.0\"\n"),
        );
        put(
            ".cargo/config.toml",
            &format!(
                "[build]\ntarget-dir = '{0}'\nbuild-dir = '{0}'\n",
                target.display()
            ),
        );
        put("src/lib.rs", &library(subject));
        if example {
            let library = package.replace('-', "_");
            put(
                "examples/forever.rs",
                &format!("fn main() {{\n    {library}::outlive();\n}}\n"),
            );
        }
        let source = Workspace::load(&DiskTree::new(&dir).expect("a tree"), &["."])
            .expect("the crate loads");
        Self { dir, source }
    }

    /// Build what a binding needs before its run, so that the run starts the
    /// built test binary or example. It takes real time and nothing is
    /// asserted about it.
    fn warm(&self, args: &[&str]) {
        let output = Command::new(cargo())
            .args(args)
            .arg("--frozen")
            .current_dir(&self.dir)
            .stdin(Stdio::null())
            .output()
            .expect("cargo starts");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// A runner for the real `cargo`.
    fn runner(&self) -> SystemRunner {
        SystemRunner::new(Programs::with_cargo_var(&self.dir, Some(cargo().into())))
    }

    /// Run the binding `entry` of `list` through `runner`.
    fn judge<R: Runner>(
        &self,
        runner: R,
        list: &str,
        entry: &str,
        limits: Option<(Limits, Limits)>,
    ) -> Result<Evidence, ExecError> {
        let doc = capability(list, entry);
        let docs = MemoryTree::from([("docs/capabilities/fixture.md", doc.as_str())]);
        let plan = Workplan::load(&docs, &self.source);
        let binding = plan
            .graph()
            .bindings()
            .first()
            .expect("the capability lists a binding");
        assert!(
            binding.resolution().is_ok(),
            "{entry} must resolve: {:?}",
            binding.resolution()
        );
        let mut executor = Executor::new(&self.source, runner).expect("the index lists its tests");
        if let Some((build, example)) = limits {
            executor = executor.with_limits(build, example);
        }
        executor.run(binding)
    }
}

/// The library of the crate. The paths of the FIFOs and of the marker are
/// written into it as Rust string literals.
fn library(subject: &Subject) -> String {
    let literal = |name: &str| {
        let path = subject.scratch.0.path().join(name);
        format!("{:?}", path.to_str().expect("a UTF-8 temporary directory"))
    };
    let (pids, hold, marker) = (literal("pids"), literal("hold"), literal("marker"));
    format!(
        r#"//! A crate whose test and example never end by themselves.

use std::io::{{Read as _, Write as _}};

extern "C" {{
    fn signal(signum: i32, handler: usize) -> usize;
}}

/// Ignore SIGHUP, SIGINT and SIGTERM, write the id of this process to the
/// pid FIFO, block reading the hold FIFO, then write the marker.
pub fn outlive() {{
    for signum in [1, 2, 15] {{
        // SIG_IGN is 1.
        unsafe {{
            signal(signum, 1);
        }}
    }}
    // The test holds the FIFO open for reading, so the open does not block.
    let mut pids = std::fs::OpenOptions::new()
        .write(true)
        .open({pids})
        .expect("the pid FIFO opens");
    writeln!(pids, "{{}}", std::process::id()).expect("the process id is written");
    // The test holds the FIFO open for writing, so only the read blocks.
    let mut hold = std::fs::File::open({hold}).expect("the hold FIFO opens");
    let _ = hold.read(&mut [0_u8; 1]);
    std::fs::write({marker}, "survived\n").expect("the marker is written");
}}

#[cfg(test)]
mod tests {{
    #[test]
    fn passes() {{}}

    #[test]
    fn forever() {{
        super::outlive();
    }}
}}
"#
    )
}

/// The capability document that lists `entry` in `list`.
fn capability(list: &str, entry: &str) -> String {
    let mut sections = String::new();
    for name in [
        "Purpose",
        "Contract",
        "Usage",
        "Examples",
        "Constraints",
        "Explanation",
        "Verification",
    ] {
        write!(sections, "## {name}\n\nText for {name}.\n\n").expect("a string");
    }
    let entry = format!("\"{entry}\"");
    let (tests, examples) = match list {
        "tests" => (entry.as_str(), ""),
        _ => ("", entry.as_str()),
    };
    format!(
        "---\ntype: Capability\nid: cap:fixture\ntitle: Fixture\ndescription: Fixture.\n\
         state: implementing\nbindings:\n  code: []\n  tests: [{tests}]\n  \
         examples: [{examples}]\ndepends: []\n---\n\n# Fixture\n\n{sections}"
    )
}

// ------------------------------------------------------------------- runs

/// A run of a binding that never ends, and what became of its processes.
struct NeverEnding {
    crate_: Crate,
    evidence: Evidence,
    asked: Vec<(Vec<String>, Limits)>,
    seen: Seen,
    settled: Settled,
    /// Kept to the end: its scratch directory holds the crate.
    _subject: Subject,
}

/// Generate the crate of `package`, with an example when `list` is
/// `examples`, build it with `warm`, and run the binding `entry` of `list`
/// with `limits` through a [`Clocked`] runner.
fn never_ending(
    package: &str,
    list: &str,
    entry: &str,
    warm: &[&str],
    limits: (Limits, Limits),
) -> NeverEnding {
    let subject = Subject::new();
    let crate_ = Crate::generate(&subject, package, list == "examples");
    crate_.warm(warm);
    let runner = Clocked::new(crate_.runner(), Stepped::gated(Watch::new(&subject), LIMIT));
    let evidence = crate_
        .judge(&runner, list, entry, Some(limits))
        .expect("a kill is a verdict");
    let mut clock = runner
        .ran
        .take()
        .expect("the binding ran under the stepped clock");
    let Some(seen) = clock.seen.take() else {
        panic!("the run ended before the gate opened on the process that never ends: {evidence}");
    };
    let settled = subject.settle(&seen);
    NeverEnding {
        crate_,
        evidence,
        asked: runner.asked.take(),
        seen,
        settled,
        _subject: subject,
    }
}

fn words(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

/// The reason of a run killed at [`LIMIT`], word for word.
fn killed_at_the_limit(evidence: &Evidence) {
    let killed = evidence.runs().last().expect("a run");
    assert_eq!(killed.ended(), Ended::TimedOut, "{evidence}");
    assert_eq!(
        killed.elapsed(),
        LIMIT,
        "the reading of the stepped clock at the kill"
    );
    assert_eq!(
        evidence.outcome(),
        &BoundOutcome::Failed {
            reason: format!(
                "`{}` ran 5.0 s of its 5 s limit and was killed; the run has no verdict",
                killed.invocation()
            ),
        }
    );
}

#[test]
fn a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran() {
    let build = Limits::build().with_timeout(LIMIT);
    let example = Limits::example().with_timeout(LIMIT);
    let ran = never_ending(
        TEST_PACKAGE,
        "tests",
        "fx_forever_test::tests::forever",
        &["test", "--no-run"],
        (build, example),
    );
    let runs = ran.evidence.runs();
    assert_eq!(runs.len(), 2, "{}", ran.evidence);
    assert_eq!(runs[0].ended(), Ended::Exited(Some(0)), "the list pass");
    assert_eq!(
        runs[1].argv(),
        words(&[
            "test",
            "--frozen",
            "--package=fx-forever-test",
            "--lib",
            "--",
            "--exact",
            "--",
            "tests::forever"
        ])
    );
    killed_at_the_limit(&ran.evidence);
    // The list pass ran under the clock of the machine, so the time of the
    // binding is at least the limit and not exactly it.
    assert!(ran.evidence.wall() >= LIMIT, "{:?}", ran.evidence.wall());
    assert_eq!(
        ran.asked,
        vec![(runs[0].argv(), build), (runs[1].argv(), build)],
        "both passes are asked for under the limits given"
    );
    let binary = ran
        .seen
        .descendant
        .as_ref()
        .expect("the test binary that cargo started wrote its id");
    assert_ne!(binary.pid, ran.seen.leader.pid);
    nothing_outlived_the_kill(&ran.seen, &ran.settled);
    // The neighbour in the same crate passes through the runner of the gate.
    let neighbour = ran
        .crate_
        .judge(
            ran.crate_.runner(),
            "tests",
            "fx_forever_test::tests::passes",
            None,
        )
        .expect("the neighbour runs");
    assert_eq!(neighbour.outcome(), &BoundOutcome::Passed, "{neighbour}");
}

#[test]
fn an_example_that_never_ends_is_killed_and_reported_with_the_time_it_ran() {
    let build = Limits::build().with_timeout(LIMIT);
    let example = Limits::example().with_timeout(LIMIT);
    let ran = never_ending(
        EXAMPLE_PACKAGE,
        "examples",
        "fx_forever_example::example_crate::forever",
        &["build", "--examples"],
        (build, example),
    );
    let runs = ran.evidence.runs();
    assert_eq!(runs.len(), 1, "{}", ran.evidence);
    assert_eq!(
        runs[0].argv(),
        words(&[
            "run",
            "--frozen",
            "--package=fx-forever-example",
            "--example=forever"
        ])
    );
    killed_at_the_limit(&ran.evidence);
    // The one run is the stepped clock's, so the time of the binding is the
    // limit exactly.
    assert_eq!(ran.evidence.wall(), LIMIT);
    assert_eq!(
        ran.asked,
        vec![(runs[0].argv(), example)],
        "the run is asked for under the limit of an example"
    );
    nothing_outlived_the_kill(&ran.seen, &ran.settled);
}
