// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The actual registered CLI cannot orphan its running repository target.

use std::io::Write as _;
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use rustix::process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal};

#[path = "../../../../tests/support/process_receipt.rs"]
mod receipt;

#[path = "../../../../tests/support/process_teardown.rs"]
mod teardown;

struct Run {
    cli: Child,
    target: Option<std::os::fd::OwnedFd>,
}

impl Drop for Run {
    fn drop(&mut self) {
        if self.cli.try_wait().expect("observe owned CLI").is_none() {
            self.cli.kill().expect("stop owned CLI");
        }
        self.cli.wait().expect("reap owned CLI");
        if let Some(target) = &self.target {
            match pidfd_send_signal(target, Signal::KILL) {
                Ok(()) | Err(rustix::io::Errno::SRCH) => {}
                Err(error) => panic!("stop owned registered target: {error}"),
            }
            receipt::wait(target).expect("registered target has ended");
        }
    }
}

pub(super) fn parent_death(root: &Path, caller: &Path) {
    let before = super::snapshot(root);
    let control = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let ready = receipt::listener(&control.path().join("ready")).unwrap();
    let hold = control.path().join("hold");
    rustix::fs::mknodat(
        rustix::fs::CWD,
        &hold,
        rustix::fs::FileType::Fifo,
        rustix::fs::Mode::from_raw_mode(0o600),
        0,
    )
    .unwrap();
    let mut hold = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(i32::try_from(rustix::fs::OFlags::NONBLOCK.bits()).unwrap())
        .open(hold)
        .unwrap();
    let cli = Command::new(env!("CARGO_BIN_EXE_cargo-census"))
        .current_dir(caller)
        .args(["census", "--workspace"])
        .arg(root)
        .args(["check", "--package", "demo", "--example", "registered"])
        .env("RAF_REGISTERED_CONTAINMENT_ROOT", control.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = Pid::from_raw(i32::try_from(cli.id()).unwrap()).unwrap();
    let cli_fd = pidfd_open(pid, PidfdFlags::empty()).unwrap();
    let mut run = Run { cli, target: None };
    let (role, target) = receipt::until_receipt(&ready, &cli_fd)
        .expect("wait for actual registered target")
        .expect("CLI must not exit before running its target");
    assert_eq!(role, "registered");
    run.target = Some(pidfd_open(target, PidfdFlags::empty()).unwrap());
    let teardown =
        teardown::Teardown::capture(target, pid).expect("capture owned namespace ancestry");
    run.cli.kill().expect("kill the actual registered CLI");
    run.cli.wait().expect("reap the registered CLI");
    teardown
        .after_cli_death()
        .expect("namespace teardown follows CLI death");
    hold.write_all(b"\n")
        .expect("release any surviving owned target");
    receipt::wait(run.target.as_ref().unwrap()).expect("target dies with its CLI");
    assert!(!control.path().join("continued").exists());
    assert_eq!(super::snapshot(root), before);
    assert!(!root.join("census-debt.ttl").exists());
}
