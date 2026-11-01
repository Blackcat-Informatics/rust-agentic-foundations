// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual CLI death terminates its contained command before publication.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read as _, Write as _};
use std::os::unix::fs::OpenOptionsExt as _;
use std::os::unix::process::{CommandExt as _, ExitStatusExt as _};
use std::path::Path;
use std::process::{Child, Stdio};

use rustix::event::{PollFd, PollFlags, Timespec, poll};
use rustix::io::Errno;
use rustix::process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal};

use super::{RECORD, record};
use crate::support::{Repo, Shape, changed, command};

#[path = "../../../../tests/support/process_receipt.rs"]
mod receipt;

#[path = "../../../../tests/support/process_teardown.rs"]
mod teardown;

const ROOT: &str = "RAF_EVIDENCE_SUPERVISOR_CONTROL";
const NAME: &str =
    "supervisor::a_command_is_killed_when_its_supervisor_dies_and_records_only_while_alive";

fn fifo(root: &Path, name: &str, nonblocking: bool) -> File {
    let path = root.join(name);
    rustix::fs::mknodat(
        rustix::fs::CWD,
        &path,
        rustix::fs::FileType::Fifo,
        rustix::fs::Mode::from_raw_mode(0o600),
        0,
    )
    .expect("create the control FIFO");
    OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(if nonblocking {
            i32::try_from(rustix::fs::OFlags::NONBLOCK.bits()).unwrap()
        } else {
            0
        })
        .open(path)
        .expect("hold both ends of the control FIFO")
}

/// Always stop and reap the actual CLI, including an assertion's unwind.
struct Supervisor(Child);

impl Drop for Supervisor {
    fn drop(&mut self) {
        let mut cleanup = || -> io::Result<()> {
            if self.0.try_wait()?.is_none() {
                self.0.kill()?;
            }
            self.0.wait()?;
            Ok(())
        };
        if let Err(error) = cleanup() {
            if std::thread::panicking() {
                eprintln!("supervisor cleanup failed: {error}");
            } else {
                panic!("supervisor cleanup failed: {error}");
            }
        }
    }
}

struct RunningCommand(std::os::fd::OwnedFd);

impl RunningCommand {
    fn ended(&self) -> bool {
        let mut fds = [PollFd::new(&self.0, PollFlags::IN)];
        loop {
            match poll(
                &mut fds,
                Some(&Timespec {
                    tv_sec: 0,
                    tv_nsec: 0,
                }),
            ) {
                Ok(_) => return fds[0].revents().contains(PollFlags::IN),
                Err(Errno::INTR) => {}
                Err(error) => panic!("inspect the command's pidfd: {error}"),
            }
        }
    }

    fn wait(&self) {
        receipt::wait(&self.0).expect("observe owned command termination");
    }
}

impl Drop for RunningCommand {
    fn drop(&mut self) {
        match pidfd_send_signal(&self.0, Signal::KILL) {
            Ok(()) | Err(Errno::SRCH) => self.wait(),
            Err(error) if std::thread::panicking() => eprintln!("command cleanup failed: {error}"),
            Err(error) => panic!("command cleanup failed: {error}"),
        }
    }
}

fn command_receipt(
    pids: &mut std::os::unix::net::UnixDatagram,
    supervisor: &Supervisor,
    supervisor_fd: &std::os::fd::OwnedFd,
) -> Pid {
    let (role, pid) = receipt::until_receipt(pids, supervisor_fd)
        .expect("wait for the command receipt")
        .unwrap_or_else(|| {
            panic!(
                "CLI {} ended before starting its command",
                supervisor.0.id()
            )
        });
    assert_eq!(role, "command");
    pid
}

fn held_command(root: &Path) {
    std::os::unix::net::UnixDatagram::unbound()
        .unwrap()
        .send_to(b"command", root.join("pids"))
        .expect("publish owned process credentials");
    let mut released = [0_u8; 1];
    File::open(root.join("hold"))
        .unwrap()
        .read_exact(&mut released)
        .unwrap();
    assert_eq!(released, [b'\n']);
    fs::write(root.join("continued"), "after release\n").unwrap();
    println!("\npeak=1");
}

#[test]
fn a_command_is_killed_when_its_supervisor_dies_and_records_only_while_alive() {
    if let Some(root) = std::env::var_os(ROOT) {
        held_command(Path::new(&root));
        return;
    }

    for interrupt in [Some(Signal::INT), Some(Signal::HUP), None] {
        let repo = Repo::synced("supervisor-lifetime", false);
        let before = repo.snapshot();
        let control = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
        let mut pids = receipt::listener(&control.path().join("pids")).unwrap();
        let mut hold = fifo(control.path(), "hold", false);
        let executable = std::env::current_exe().unwrap();
        let args = record(&[
            "--",
            executable.to_str().unwrap(),
            "--exact",
            NAME,
            "--nocapture",
        ]);
        let mut supervisor = Supervisor(
            command(repo.root(), Shape::Direct, &args)
                .env(ROOT, control.path())
                .process_group(0)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        let supervisor_pid = Pid::from_raw(i32::try_from(supervisor.0.id()).unwrap()).unwrap();
        let supervisor_fd = pidfd_open(supervisor_pid, PidfdFlags::empty()).unwrap();
        let command_pid = command_receipt(&mut pids, &supervisor, &supervisor_fd);
        let running = RunningCommand(pidfd_open(command_pid, PidfdFlags::empty()).unwrap());
        assert_eq!(
            rustix::process::getpgid(Some(supervisor_pid)).unwrap(),
            supervisor_pid
        );
        assert_ne!(
            rustix::process::getpgid(Some(command_pid)).unwrap(),
            supervisor_pid
        );
        assert_ne!(supervisor_pid, command_pid);
        assert!(!running.ended());
        let teardown = interrupt.map(|_| {
            teardown::Teardown::capture(command_pid, supervisor_pid)
                .expect("capture owned namespace ancestry")
        });

        if let Some(signal) = interrupt {
            rustix::process::kill_process_group(supervisor_pid, signal).unwrap();
            let status = supervisor.0.wait().unwrap();
            assert_eq!(status.signal(), Some(signal.as_raw()));
            assert_eq!(repo.snapshot(), before);
            teardown
                .as_ref()
                .unwrap()
                .after_cli_death()
                .expect("namespace teardown follows supervisor death");
        }
        // Namespace PID 1 has completed teardown before an interrupted run's
        // input is released; surviving ancestors fail and are forcibly cleaned.
        hold.write_all(b"\n").unwrap();
        running.wait();
        assert!(running.ended(), "the owned command has ended");
        if interrupt.is_some() {
            assert!(!control.path().join("continued").exists());
            assert_eq!(
                repo.snapshot(),
                before,
                "no surviving actor publishes evidence"
            );
            assert!(!repo.root().join(RECORD).exists());
        } else {
            assert_eq!(
                fs::read_to_string(control.path().join("continued")).unwrap(),
                "after release\n"
            );
            let status = supervisor.0.wait().unwrap();
            let mut stdout = String::new();
            let mut stderr = String::new();
            supervisor
                .0
                .stdout
                .take()
                .unwrap()
                .read_to_string(&mut stdout)
                .unwrap();
            supervisor
                .0
                .stderr
                .take()
                .unwrap()
                .read_to_string(&mut stderr)
                .unwrap();
            assert!(status.success(), "{stdout}\n{stderr}");
            assert_eq!(changed(&before, &repo.snapshot()), [RECORD]);
            let text = repo.read(RECORD);
            assert!(
                text.contains("verdict: pass") && text.contains("peak: 1"),
                "{text}"
            );
        }
    }
}
