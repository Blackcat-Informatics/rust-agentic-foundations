// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Bounded direct Git commands with one drained pipe and owned child cleanup.

use super::{native, recover, refused};
use crate::meter::{ItemAdmission, Meter};
use crate::{LoadError, LoadErrorKind, RelPath};
use purrdf_lex::allocation::Memory;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};

pub(super) fn read(
    repository: &Path,
    args: &[&str],
    cap: usize,
    meter: &Meter,
    path: &RelPath,
) -> Result<Vec<u8>, LoadError> {
    meter.work(1, path, None)?;
    let (mut reader, writer) = io::pipe().map_err(|source| io(path, "pipe", source))?;
    let stderr = writer
        .try_clone()
        .map_err(|source| io(path, "stderr pipe", source))?;
    let mut child = {
        let mut command = Command::new("git");
        command.env_clear();
        for key in ["PATH", "SystemRoot", "WINDIR", "TEMP", "TMP", "TMPDIR"] {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }
        let null = if cfg!(windows) { "NUL" } else { "/dev/null" };
        command
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", null)
            .env("GIT_CONFIG_SYSTEM", null)
            .env("GIT_NO_LAZY_FETCH", "1")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .env("GIT_OPTIONAL_LOCKS", "0")
            .args([
                "--no-pager",
                "--no-lazy-fetch",
                "--no-replace-objects",
                "--no-optional-locks",
                "-c",
                "protocol.allow=never",
                "-C",
            ])
            .arg(repository)
            .args(args)
            .stdin(Stdio::null())
            .stdout(writer)
            .stderr(stderr);
        command
            .spawn()
            .map_err(|source| io(path, "spawn required installed Git", source))?
    };
    let output = collect(&mut reader, cap, meter, path);
    drop(reader);
    let output = match output {
        Ok(output) => output,
        Err(error) => return Err(stop(&mut child, path, error)),
    };
    let status = match child.wait() {
        Ok(status) => status,
        Err(source) => return Err(stop(&mut child, path, io(path, "wait", source))),
    };
    if !status.success() {
        let mut admission = ItemAdmission::new(meter, path);
        let diagnostic =
            std::str::from_utf8(&output).unwrap_or("Git failed with non-UTF-8 diagnostic output");
        let diagnostic = Memory::new(&mut admission)
            .string(diagnostic)
            .map_err(|source| admission.error(source))?;
        return Err(LoadError::new(
            path,
            None,
            LoadErrorKind::GitExit { status, diagnostic },
        ));
    }
    Ok(output)
}

fn collect(
    reader: &mut impl Read,
    cap: usize,
    meter: &Meter,
    path: &RelPath,
) -> Result<Vec<u8>, LoadError> {
    let mut admission = ItemAdmission::new(meter, path);
    let mut memory = Memory::new(&mut admission);
    let result = (|| {
        let mut output = Vec::new();
        loop {
            let mut buffer = [0; 8192];
            let remaining = cap.saturating_sub(output.len());
            let wanted = buffer.len().min(remaining.saturating_add(1));
            let mut count = 0;
            while count < wanted {
                match reader.read(&mut buffer[count..wanted]) {
                    Ok(0) => break,
                    Ok(read) => count += read,
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                    Err(source) => return Err(io(path, "read bounded output", source)),
                }
            }
            meter.work(count.saturating_add(1), path, None)?;
            meter.read_bytes(path, count)?;
            if count > remaining {
                return Err(refused(
                    path,
                    "Git output exceeds its admitted protocol size",
                ));
            }
            if count == 0 {
                break;
            }
            let required = output
                .len()
                .checked_add(count)
                .ok_or_else(|| refused(path, "Git output length overflow"))?;
            if required > output.capacity() {
                let capacity = required.max(output.capacity().saturating_mul(2)).min(cap);
                memory.reserve(&mut output, capacity).map_err(native)?;
            }
            output.extend_from_slice(&buffer[..count]);
            if count < wanted {
                break;
            }
        }
        Ok(output)
    })();
    result.map_err(|error| recover(error, &mut admission))
}

fn stop(child: &mut Child, path: &RelPath, mut primary: LoadError) -> LoadError {
    if let Err(source) = child.kill() {
        primary = cleanup(primary, path, "kill", source);
    }
    if let Err(source) = child.wait() {
        primary = cleanup(primary, path, "reap", source);
    }
    primary
}
fn cleanup(
    primary: LoadError,
    path: &RelPath,
    operation: &'static str,
    source: io::Error,
) -> LoadError {
    LoadError::new(
        path,
        None,
        LoadErrorKind::GitCleanup {
            primary: Box::new(primary),
            operation,
            source,
        },
    )
}
pub(super) fn io(path: &RelPath, operation: &'static str, source: io::Error) -> LoadError {
    LoadError::new(path, None, LoadErrorKind::GitIo { operation, source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LoadOptions;
    #[test]
    fn protocol_size_is_checked_before_growth_and_exact_size_has_a_valid_neighbor() {
        let path = RelPath::root();
        let meter = Meter::new(&LoadOptions::default());
        assert!(collect(&mut &b"abc"[..], 2, &meter, &path).is_err());
        assert_eq!(
            collect(&mut &b"abc"[..], 3, &meter, &path).expect("exact"),
            b"abc"
        );
        assert!(collect(&mut &b"x"[..], 0, &meter, &path).is_err());
        assert_eq!(collect(&mut &b""[..], 0, &meter, &path).expect("empty"), []);
    }
}
