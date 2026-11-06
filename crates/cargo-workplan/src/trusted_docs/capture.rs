// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed tool operations with bounded streams and ordinary owned-child cleanup.

use super::{Checked, Operation};
use crate::exit::Failure;

pub(super) struct Output {
    pub(super) stdout: Vec<u8>,
    pub(super) stderr: Vec<u8>,
}

#[cfg(not(unix))]
pub(super) fn run(_checked: &Checked, _operation: Operation) -> Result<Output, Failure> {
    Err(Failure::new(
        "trusted documentation compilation requires the native Unix capture backend",
    ))
}

#[cfg(unix)]
pub(super) fn run(checked: &Checked, operation: Operation) -> Result<Output, Failure> {
    use std::os::fd::OwnedFd;
    use std::os::unix::net::UnixStream;
    use std::process::Stdio;
    let (mut stdout, stdout_child) = UnixStream::pair()?;
    let (mut stderr, stderr_child) = UnixStream::pair()?;
    stdout.set_nonblocking(true)?;
    stderr.set_nonblocking(true)?;
    let mut command = command(checked, operation)?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(OwnedFd::from(stdout_child)))
        .stderr(Stdio::from(OwnedFd::from(stderr_child)));
    let mut child = command
        .spawn()
        .map_err(|error| Failure::new(format!("cannot start {}: {error}", operation.label())))?;
    let watchdog = super::clock::Watchdog::start(operation.is_build());
    let mut output = Output {
        stdout: Vec::new(),
        stderr: Vec::new(),
    };
    loop {
        if let Err(error) = drain(&mut stdout, &mut output.stdout, operation.output_limit())
            .and_then(|()| drain(&mut stderr, &mut output.stderr, operation.output_limit()))
        {
            return Err(stop(&mut child, error));
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                // All direct-child writes precede its exit. Drain what is now
                // available; descendants holding sockets are outside this scope.
                drain(&mut stdout, &mut output.stdout, operation.output_limit())?;
                drain(&mut stderr, &mut output.stderr, operation.output_limit())?;
                if status.success() {
                    return Ok(output);
                }
                return Err(Failure::new(format!(
                    "{} failed ({status}); stdout tail:\n{}\nstderr tail:\n{}",
                    operation.label(),
                    tail(&output.stdout),
                    tail(&output.stderr)
                )));
            }
            Ok(None) => {}
            Err(error) => {
                return Err(stop(
                    &mut child,
                    Failure::new(format!("cannot observe {}: {error}", operation.label())),
                ));
            }
        }
        if watchdog.expired() {
            return Err(stop(
                &mut child,
                Failure::new(format!(
                    "{} exceeded its direct-child watchdog; descendant teardown is not guaranteed",
                    operation.label()
                )),
            ));
        }
        super::clock::Watchdog::pause();
    }
}

#[cfg(unix)]
fn drain(
    stream: &mut std::os::unix::net::UnixStream,
    output: &mut Vec<u8>,
    limit: usize,
) -> Result<(), Failure> {
    use std::io::Read as _;
    loop {
        let mut buffer = [0_u8; 8192];
        let remaining = limit - output.len();
        let wanted = buffer.len().min(remaining.saturating_add(1));
        let count = match stream.read(&mut buffer[..wanted]) {
            Ok(0) => return Ok(()),
            Ok(count) => count,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error.into()),
        };
        if count > remaining {
            return Err(Failure::new(format!(
                "trusted documentation stream exceeds its {limit} byte bound; no partial stream is admitted"
            )));
        }
        let required = output.len() + count;
        if required > output.capacity() {
            let capacity = required.max(output.capacity().saturating_mul(2)).min(limit);
            output.try_reserve_exact(capacity - output.len())?;
        }
        output.extend_from_slice(&buffer[..count]);
    }
}

#[cfg(unix)]
fn tail(bytes: &[u8]) -> String {
    String::from_utf8_lossy(&bytes[bytes.len().saturating_sub(8192)..]).into_owned()
}

#[cfg(unix)]
fn stop(child: &mut std::process::Child, primary: Failure) -> Failure {
    match child.try_wait() {
        Ok(Some(_)) => return primary,
        Ok(None) => {}
        Err(error) => {
            return Failure::new(format!(
                "{primary}; cannot observe owned process {} for cleanup: {error}; it may remain running",
                child.id()
            ));
        }
    }
    if let Err(error) = child.kill() {
        return match child.try_wait() {
            Ok(Some(_)) => primary,
            _ => Failure::new(format!(
                "{primary}; cannot stop owned process {}: {error}; it may remain running",
                child.id()
            )),
        };
    }
    match child.wait() {
        Ok(_) => primary,
        Err(error) => Failure::new(format!(
            "{primary}; cannot reap owned process {}: {error}",
            child.id()
        )),
    }
}

#[cfg(unix)]
fn command(checked: &Checked, operation: Operation) -> Result<std::process::Command, Failure> {
    let program = match operation {
        Operation::CargoVersion | Operation::Metadata | Operation::Documentation => {
            &checked.tools.cargo
        }
        Operation::RustcVersion => &checked.tools.rustc,
        Operation::RustdocVersion => &checked.tools.rustdoc,
    };
    let mut command = std::process::Command::new(program);
    command
        .current_dir(&checked.root)
        .env("RUSTC", &checked.tools.rustc)
        .env("RUSTDOC", &checked.tools.rustdoc)
        .env("RUSTC_WRAPPER", "")
        .env("RUSTC_WORKSPACE_WRAPPER", "")
        .env("CARGO_BUILD_RUSTC", &checked.tools.rustc)
        .env("CARGO_BUILD_RUSTDOC", &checked.tools.rustdoc)
        .env("CARGO_BUILD_RUSTC_WRAPPER", "")
        .env("CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER", "")
        .env("CARGO_NET_OFFLINE", "true")
        .env("CARGO_TERM_COLOR", "never");
    for name in [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTDOCFLAGS",
        "CARGO_ENCODED_RUSTDOCFLAGS",
        "CARGO_TARGET_DIR",
        "CARGO_BUILD_TARGET_DIR",
        "CARGO_BUILD_TARGET",
        "RUST_TARGET_PATH",
    ] {
        command.env_remove(name);
    }
    match operation {
        Operation::CargoVersion => {
            command.arg("--version");
        }
        Operation::RustcVersion | Operation::RustdocVersion => {
            command.arg("-vV");
        }
        Operation::Metadata | Operation::Documentation => {
            for (key, value) in [
                ("build.target-dir", &checked.target),
                ("build.rustc", &checked.tools.rustc),
                ("build.rustdoc", &checked.tools.rustdoc),
            ] {
                let text = value.to_str().ok_or_else(|| {
                    Failure::new("selected documentation paths must remain UTF-8")
                })?;
                let quoted = purrdf_lex::json::write_compact(&purrdf_lex::json::Value::from(text));
                command.args(["--config", &format!("{key}={quoted}")]);
            }
            command.args([
                "--config",
                "build.rustc-wrapper=\"\"",
                "--config",
                "build.rustc-workspace-wrapper=\"\"",
            ]);
            command
                .arg(if operation.is_build() {
                    "doc"
                } else {
                    "metadata"
                })
                .args(["--frozen", "--offline", "--manifest-path"])
                .arg(checked.root.join("Cargo.toml"));
            if operation.is_build() {
                command.args(["--no-deps", "--lib", "--message-format=json"]);
                for package in checked.options.packages() {
                    command.arg(format!("--package={package}"));
                }
            } else {
                command.arg("--format-version=1");
            }
            if checked.options.no_default_features() {
                command.arg("--no-default-features");
            }
            if !checked.options.features().is_empty() {
                command.arg(format!(
                    "--features={}",
                    checked.options.features().join(",")
                ));
            }
        }
    }
    Ok(command)
}
