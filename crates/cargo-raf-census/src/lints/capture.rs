// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Explicitly trusted production lint capture for the installed command.

use raf_census::{Error, Gate, MAX_LEDGER_BYTES, PRODUCTION_CLIPPY_ARGS, ProductionLintRun};
use raf_source::{DiskTree, RelPath};
use rustix::{
    event::{PollFd, PollFlags, poll},
    fs::{AtFlags, Mode, OFlags, RenameFlags, openat, renameat_with, statat, unlinkat},
    io::Errno,
    process::{Pid, WaitId, WaitIdOptions, waitid},
};
use std::{
    env, fs,
    io::{self, Read, Write},
    os::{
        fd::OwnedFd,
        unix::{fs::PermissionsExt, net::UnixStream},
    },
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
};

const CLIPPY: &str = "cargo-clippy";

pub(crate) fn capture(
    workspace: &Path,
    out: &Path,
    diagnostics: &mut dyn Write,
) -> Result<u8, Error> {
    let root = Gate::workspace(workspace).root()?;
    let target = Target::admit(&root, out)?;
    for key in [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_BUILD_RUSTFLAGS",
        "CARGO_BUILD_TARGET",
        "CLIPPY_ARGS",
        "CLIPPY_CONF_DIR",
    ] {
        if env::var_os(key).is_some_and(|value| !value.is_empty()) {
            return Err(Error::Options(format!(
                "production Clippy needs its declared arguments; remove {key} from this invocation"
            )));
        }
    }
    let before = super::inputs_at(&root)?.digest().to_hex();
    // Use Cargo's Clippy subcommand executable directly. Cargo aliases do not
    // get an opportunity to replace the command identified by the receipt.
    let tool = clippy_executable(&root)?;
    let rustc = version(&root, Path::new("rustc"), &["-vV"])?;
    let clippy = version(&root, &tool, &["clippy", "--version"])?;
    let (status, text) = run(&root, &tool, PRODUCTION_CLIPPY_ARGS)?;
    diagnostics
        .write_all(text.as_bytes())
        .map_err(|source| Error::Io {
            file: "<stderr>".into(),
            source,
        })?;
    let run = ProductionLintRun::capture(
        &before,
        &super::inputs_at(&root)?,
        PRODUCTION_CLIPPY_ARGS,
        format!("{rustc}{clippy}"),
        status,
        text,
    )?;
    target.publish(run.to_turtle()?.as_bytes())?;
    Ok(status)
}

fn clippy_executable(root: &Path) -> Result<PathBuf, Error> {
    let path = env::var_os("PATH");
    let mut directories: Vec<_> = path.iter().flat_map(env::split_paths).collect();
    let home = env::var_os("CARGO_HOME")
        .filter(|entry| !entry.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::home_dir().map(|home| home.join(".cargo")));
    if let Some(home) = home {
        let bin = root.join(home).join("bin");
        if !directories.contains(&bin) {
            directories.insert(0, bin);
        }
    }
    directories
        .iter()
        .map(|directory| root.join(directory).join(CLIPPY))
        .find(|candidate| {
            candidate
                .metadata()
                .is_ok_and(|entry| entry.is_file() && entry.permissions().mode() & 0o111 != 0)
        })
        .ok_or_else(|| {
            Error::Options(
                "install Clippy: no executable cargo-clippy in Cargo home/bin or PATH".into(),
            )
        })
}

fn version(root: &Path, tool: &Path, arguments: &[&str]) -> Result<String, Error> {
    let (status, output) = run(root, tool, arguments)?;
    if status != 0 || output.trim().is_empty() {
        return Err(Error::Options(format!(
            "{} {} did not identify the production compiler (exit {status}): {}",
            tool.display(),
            arguments.join(" "),
            output.escape_debug()
        )));
    }
    Ok(output)
}

fn run(root: &Path, tool: &Path, arguments: &[&str]) -> Result<(u8, String), Error> {
    let name = tool.display().to_string();
    let failure = |source| Error::Io {
        file: name.clone(),
        source,
    };
    let (mut reader, writer) = UnixStream::pair().map_err(failure)?;
    let stderr = writer.try_clone().map_err(failure)?;
    reader.set_nonblocking(true).map_err(failure)?;
    let (ended, notify) = UnixStream::pair().map_err(failure)?;
    let mut child = Command::new(tool)
        .args(arguments)
        .current_dir(root)
        .env("CARGO_ENCODED_RUSTFLAGS", "")
        .env_remove("CARGO")
        .stdin(Stdio::null())
        .stdout(Stdio::from(OwnedFd::from(writer)))
        .stderr(Stdio::from(OwnedFd::from(stderr)))
        .spawn()
        .map_err(failure)?;
    let Some(pid) = i32::try_from(child.id()).ok().and_then(Pid::from_raw) else {
        let error = Error::Options(format!("{name} started with an unrepresentable PID"));
        return Err(stop(&mut child, error));
    };
    let bytes = thread::scope(|scope| {
        let observer = thread::Builder::new().spawn_scoped(scope, move || {
            // Observe exit without reaping. The Child stays owned by the
            // capture and its inherited foreground group receives interrupts.
            let result = loop {
                match waitid(
                    WaitId::Pid(pid),
                    WaitIdOptions::EXITED | WaitIdOptions::NOWAIT,
                ) {
                    Err(Errno::INTR) => {}
                    result => break result,
                }
            };
            drop(notify);
            result
        });
        let observer = observer.map_err(|source| stop(&mut child, failure(source)))?;
        let bytes = collect(&mut reader, &ended, &name).map_err(|error| {
            // Close the bounded stream and let the adapter finish its Cargo
            // invocation, including the compiler children it owns.
            drop(reader);
            reap(&mut child, error)
        })?;
        observer
            .join()
            .map_err(|cause| {
                let message = cause
                    .downcast_ref::<&str>()
                    .copied()
                    .or_else(|| cause.downcast_ref::<String>().map(String::as_str))
                    .unwrap_or("unknown panic payload");
                let error = Error::Options(format!("{name} exit observer failed: {message}"));
                stop(&mut child, error)
            })?
            .map_err(|source| stop(&mut child, failure(source.into())))?;
        Ok::<_, Error>(bytes)
    })?;
    let status = child.wait().map_err(failure)?;
    let code = status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .ok_or_else(|| Error::Options(format!("{name} ended without an exit code: {status}")))?;
    let text = String::from_utf8(bytes).map_err(|source| failure(io::Error::other(source)))?;
    Ok((code, text))
}

fn collect(output: &mut UnixStream, ended: &UnixStream, tool: &str) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    let mut open = true;
    loop {
        let (readable, exited) = {
            let stream = if open { &*output } else { ended };
            let mut events = [
                PollFd::new(stream, PollFlags::IN),
                PollFd::new(ended, PollFlags::IN),
            ];
            match poll(&mut events, None) {
                Ok(_) => {}
                Err(Errno::INTR) => continue,
                Err(source) => {
                    return Err(Error::Io {
                        file: tool.into(),
                        source: source.into(),
                    });
                }
            }
            (
                !events[0].revents().is_empty(),
                !events[1].revents().is_empty(),
            )
        };
        if open && (readable || exited) {
            open = drain(output, &mut bytes, tool)?;
        }
        if exited {
            return Ok(bytes);
        }
    }
}

fn drain(output: &mut UnixStream, bytes: &mut Vec<u8>, tool: &str) -> Result<bool, Error> {
    let mut buffer = [0; 8192];
    loop {
        let remaining = MAX_LEDGER_BYTES - bytes.len();
        let wanted = buffer.len().min(remaining + 1);
        let count = match output.read(&mut buffer[..wanted]) {
            Ok(0) => return Ok(false),
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(true),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(source) => {
                return Err(Error::Io {
                    file: tool.into(),
                    source,
                });
            }
        };
        if count > remaining {
            return Err(Error::Options(format!(
                "{tool} production output exceeds {MAX_LEDGER_BYTES} bytes"
            )));
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
}

fn stop(child: &mut Child, original: Error) -> Error {
    if let Err(cause) = child.kill()
        && !matches!(child.try_wait(), Ok(Some(_)))
    {
        return Error::Options(format!("{original}; cannot stop compiler: {cause}"));
    }
    reap(child, original)
}

fn reap(child: &mut Child, original: Error) -> Error {
    match child.wait() {
        Ok(_) => original,
        Err(cause) => Error::Options(format!("{original}; cannot reap compiler: {cause}")),
    }
}

struct Target {
    tree: DiskTree,
    directory: fs::File,
    name: String,
}

impl Target {
    fn admit(root: &Path, path: &Path) -> Result<Self, Error> {
        let refuse = || {
            Error::Options(format!(
                "{}: use a fresh absolute receipt path in a build-owned directory outside the workspace",
                path.display()
            ))
        };
        let spelling = path.as_os_str().as_encoded_bytes();
        if !path.is_absolute() || spelling.ends_with(b"/") || spelling.ends_with(b"/.") {
            return Err(refuse());
        }
        let (Some(parent), Some(name)) = (path.parent(), path.file_name().and_then(|s| s.to_str()))
        else {
            return Err(refuse());
        };
        if name == ".git" || RelPath::new(name).is_err() {
            return Err(refuse());
        }
        let parent = parent.canonicalize().map_err(|source| Error::Io {
            file: path.display().to_string(),
            source,
        })?;
        if parent.starts_with(root) || crate::command::git_administration(root, &parent)? {
            return Err(refuse());
        }
        let tree = DiskTree::new(&parent).map_err(|source| Error::Tree {
            file: parent.display().to_string(),
            source,
        })?;
        let directory = tree.directory_handle().map_err(|source| Error::Tree {
            file: parent.display().to_string(),
            source,
        })?;
        match statat(&directory, name, AtFlags::SYMLINK_NOFOLLOW) {
            Err(Errno::NOENT) => Ok(Self {
                tree,
                directory,
                name: name.to_owned(),
            }),
            Ok(_) => Err(refuse()),
            Err(source) => Err(Error::Io {
                file: path.display().to_string(),
                source: source.into(),
            }),
        }
    }

    fn publish(&self, receipt: &[u8]) -> Result<(), Error> {
        let mut temporary = format!(".raf-production-lints-{}", std::process::id());
        if temporary == self.name {
            temporary.push_str(".tmp");
        }
        self.verify_directory()?;
        let failure = |source| Error::Io {
            file: self.name.clone(),
            source,
        };
        let mut file = fs::File::from(
            openat(
                &self.directory,
                temporary.as_str(),
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::from_raw_mode(0o600),
            )
            .map_err(|source| failure(source.into()))?,
        );
        let publish = (|| {
            file.write_all(receipt).map_err(failure)?;
            file.sync_all().map_err(failure)?;
            self.verify_directory()?;
            renameat_with(
                &self.directory,
                temporary.as_str(),
                &self.directory,
                self.name.as_str(),
                RenameFlags::NOREPLACE,
            )
            .map_err(|source| failure(source.into()))
        })();
        let Err(primary) = publish else {
            return Ok(());
        };
        match unlinkat(&self.directory, temporary.as_str(), AtFlags::empty()) {
            Err(cleanup) => Err(Error::Options(format!(
                "{primary}; cannot remove the temporary receipt: {cleanup}"
            ))),
            Ok(()) => Err(primary),
        }
    }

    fn verify_directory(&self) -> Result<(), Error> {
        self.tree.directory_handle().map_err(|source| Error::Tree {
            file: self.name.clone(),
            source,
        })?;
        Ok(())
    }
}
