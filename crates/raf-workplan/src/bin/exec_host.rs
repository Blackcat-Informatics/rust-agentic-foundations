// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Private native exec admission and supervision inside the PID namespace.

#[cfg(target_os = "linux")]
#[path = "../exec/system/control.rs"]
mod control;

#[cfg(target_os = "linux")]
fn error(writer: &mut impl std::io::Write, tag: u8, error: &std::io::Error) -> std::io::Result<()> {
    let message = error.to_string();
    let mut end = message.len().min(control::MAX_ERROR);
    while !message.is_char_boundary(end) {
        end -= 1;
    }
    let length = u16::try_from(end).map_err(std::io::Error::other)?;
    writer.write_all(&[tag])?;
    writer.write_all(&error.raw_os_error().unwrap_or(0).to_le_bytes())?;
    writer.write_all(&length.to_le_bytes())?;
    writer.write_all(&message.as_bytes()[..end])
}

#[cfg(target_os = "linux")]
fn run() -> std::io::Result<u8> {
    use std::io::Write as _;
    use std::os::unix::process::ExitStatusExt as _;
    use std::process::{Command, Stdio};

    let mut writer = std::fs::OpenOptions::new()
        .write(true)
        .open("/proc/self/fd/0")?;
    let mut arguments = std::env::args_os().skip(1);
    let Some(program) = arguments.next() else {
        let missing = std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "missing executable argument",
        );
        error(&mut writer, control::SPAWN_ERROR, &missing)?;
        return Ok(125);
    };
    let mut child = match Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(source) => {
            error(&mut writer, control::SPAWN_ERROR, &source)?;
            return Ok(125);
        }
    };
    // Command::spawn has completed its native exec-error handshake. Its child
    // has null stdin and cannot retain this newly opened CLOEXEC control file.
    writer.write_all(&[control::READY])?;
    let status = match child.wait() {
        Ok(status) => status,
        Err(source) => {
            error(&mut writer, control::WAIT_ERROR, &source)?;
            return Ok(125);
        }
    };
    let code = status
        .code()
        .or_else(|| status.signal().map(|signal| 128 + signal))
        .and_then(|code| u8::try_from(code).ok())
        .ok_or_else(|| std::io::Error::other("unrepresentable native exit status"))?;
    writer.write_all(&[control::FINISHED, code])?;
    Ok(code)
}

fn main() -> std::io::Result<std::process::ExitCode> {
    #[cfg(target_os = "linux")]
    {
        run().map(std::process::ExitCode::from)
    }
    #[cfg(not(target_os = "linux"))]
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "native exec admission requires Linux",
    ))
}
