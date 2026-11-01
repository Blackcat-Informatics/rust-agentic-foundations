// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Owned native namespace teardown barriers, independent of scheduling speed.

use std::fs::File;
use std::io::{self, Read as _, Write as _};
use std::os::fd::OwnedFd;

use rustix::fs::{Mode, OFlags, fstat, openat};
use rustix::process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal};

struct Process {
    pid: Pid,
    directory: File,
    fd: OwnedFd,
    parent: Pid,
    namespace: u64,
    namespace_init: bool,
}

fn contents(directory: &File, name: &str) -> io::Result<String> {
    let mut text = String::new();
    File::from(openat(
        directory,
        name,
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
    )?)
    .read_to_string(&mut text)?;
    Ok(text)
}

fn state(directory: &File) -> io::Result<(char, Pid)> {
    let stat = contents(directory, "stat")?;
    let (_, fields) = stat
        .rsplit_once(") ")
        .ok_or_else(|| io::Error::other("invalid owned process stat"))?;
    let mut fields = fields.split_whitespace();
    let state = fields
        .next()
        .and_then(|value| value.chars().next())
        .ok_or_else(|| io::Error::other("missing owned process state"))?;
    let parent = fields
        .next()
        .and_then(|value| value.parse().ok())
        .and_then(Pid::from_raw)
        .ok_or_else(|| io::Error::other("missing owned process parent"))?;
    Ok((state, parent))
}

impl Process {
    fn capture(pid: Pid) -> io::Result<Self> {
        if pid.as_raw_pid() == 1 {
            return Err(io::Error::other("refusing to capture host PID 1"));
        }
        let directory = File::open(format!("/proc/{}", pid.as_raw_pid()))?;
        let fd = pidfd_open(pid, PidfdFlags::empty())?;
        // Read through the already-open proc directory after acquiring the
        // pidfd. An exited/reused numeric PID cannot supply a new process here.
        let (_, parent) = state(&directory)?;
        let status = contents(&directory, "status")?;
        let namespace_init = status
            .lines()
            .find_map(|line| line.strip_prefix("NSpid:"))
            .and_then(|ids| ids.split_whitespace().last())
            == Some("1");
        let namespace_fd = openat(
            &directory,
            "ns/pid",
            OFlags::RDONLY | OFlags::CLOEXEC,
            Mode::empty(),
        )?;
        let namespace = fstat(namespace_fd)?.st_ino;
        Ok(Self {
            pid,
            directory,
            fd,
            parent,
            namespace,
            namespace_init,
        })
    }

    fn exited(&self) -> io::Result<bool> {
        use rustix::event::{PollFd, PollFlags, Timespec, poll};
        let mut fds = [PollFd::new(&self.fd, PollFlags::IN)];
        loop {
            match poll(
                &mut fds,
                Some(&Timespec {
                    tv_sec: 0,
                    tv_nsec: 0,
                }),
            ) {
                Ok(_) => return Ok(fds[0].revents().contains(PollFlags::IN)),
                Err(rustix::io::Errno::INTR) => {}
                Err(error) => return Err(error.into()),
            }
        }
    }

    fn parent_died(&self) -> io::Result<()> {
        // Called only after the parent is reaped/observably dead. Linux queues
        // PDEATHSIG during parent exit before that observation. SIGSTOP cannot
        // mask an already pending SIGKILL; without it, the owned process stops
        // and supplies a negative witness instead of an unbounded pidfd wait.
        match pidfd_send_signal(&self.fd, Signal::STOP) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => {}
            Err(error) => return Err(error.into()),
        }
        loop {
            if self.exited()? {
                return Ok(());
            }
            match state(&self.directory) {
                Ok(('T' | 't', _)) => {
                    return Err(io::Error::other(format!(
                        "owned containment ancestor {} survived its parent's death",
                        self.pid.as_raw_pid()
                    )));
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    super::receipt::wait(&self.fd)?;
                    return Ok(());
                }
                Err(error) => return Err(error),
            }
            std::thread::yield_now();
        }
    }
}

pub(crate) struct Teardown {
    owned: Vec<Process>,
    init: usize,
    monitor: usize,
}

impl Teardown {
    pub(crate) fn capture(target: Pid, cli: Pid) -> io::Result<Self> {
        let launcher = Process::capture(cli)?;
        let mut owned = Vec::new();
        let mut next = target;
        while next != cli {
            if owned.len() == 16 || owned.iter().any(|process: &Process| process.pid == next) {
                return Err(io::Error::other(
                    "owned containment ancestry does not reach its CLI",
                ));
            }
            let process = Process::capture(next)?;
            next = process.parent;
            owned.push(process);
        }
        for (index, process) in owned.iter().enumerate() {
            let expected = owned.get(index + 1).map_or(cli, |parent| parent.pid);
            if state(&process.directory)?.1 != expected {
                return Err(io::Error::other(
                    "owned containment ancestry changed during capture",
                ));
            }
        }
        let init = owned
            .iter()
            .position(|process| process.namespace_init && process.namespace != launcher.namespace)
            .ok_or_else(|| io::Error::other("owned target has no native namespace PID 1"))?;
        let monitor = init + 1;
        if monitor + 1 != owned.len()
            || owned[monitor].namespace != launcher.namespace
            || owned[..=init]
                .iter()
                .any(|process| process.namespace != owned[init].namespace)
        {
            return Err(io::Error::other(
                "owned namespace boundary does not lead directly through its monitor to its CLI",
            ));
        }
        Ok(Self {
            owned,
            init,
            monitor,
        })
    }

    pub(crate) fn after_cli_death(&self) -> io::Result<()> {
        self.owned[self.monitor].parent_died()?;
        self.after_launcher_death()
    }

    pub(crate) fn after_launcher_death(&self) -> io::Result<()> {
        self.owned[self.init].parent_died()
    }
}

impl Drop for Teardown {
    fn drop(&mut self) {
        for process in self.owned.iter().rev() {
            match pidfd_send_signal(&process.fd, Signal::KILL) {
                Ok(()) | Err(rustix::io::Errno::SRCH) => {}
                Err(error) if std::thread::panicking() => {
                    writeln!(io::stderr(), "owned containment cleanup failed: {error}")
                        .expect("write owned containment cleanup diagnostic");
                }
                Err(error) => {
                    std::panic::panic_any(format!("owned containment cleanup failed: {error}"));
                }
            }
        }
    }
}
