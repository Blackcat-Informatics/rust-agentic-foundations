// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Independent namespace and successful-exec admission before payload verdicts.

use std::io::{self, BufRead as _, Read as _};
use std::path::Path;
use std::process::{Child, Command};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;

#[cfg(target_os = "linux")]
use command_fds::{CommandFdExt as _, FdMapping};
use purrdf_lex::json::{self, Value};

use super::{ExecError, control};

const STATUS_LIMIT: usize = 4096;

/// The unreaped launcher and its admitted kernel signalling identity.
pub(super) struct Supervised {
    pub(super) child: Child,
    #[cfg(target_os = "linux")]
    identity: std::os::fd::OwnedFd,
}

impl Supervised {
    pub(super) fn admit(mut child: Child) -> io::Result<Self> {
        #[cfg(target_os = "linux")]
        {
            use rustix::process::{Pid, PidfdFlags, pidfd_open};
            // No wait has occurred: even an exited child retains this PID until
            // it is reaped. Never acquire authority from launcher JSON PIDs.
            let identity = i32::try_from(child.id())
                .map_err(io::Error::other)
                .and_then(|raw| {
                    Pid::from_raw(raw).ok_or_else(|| io::Error::other("invalid owned launcher PID"))
                })
                .and_then(|pid| pidfd_open(pid, PidfdFlags::empty()).map_err(Into::into));
            match identity {
                Ok(identity) => Ok(Self { child, identity }),
                Err(error) => Err(failed_admission(&mut child, &error)),
            }
        }
        #[cfg(not(target_os = "linux"))]
        Err(failed_admission(
            &mut child,
            &io::Error::new(
                io::ErrorKind::Unsupported,
                "Linux pidfd supervision is required",
            ),
        ))
    }

    /// Signal only the admitted launcher; its death tears down the namespace.
    pub(super) fn terminate(&mut self) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            use rustix::process::{Signal, pidfd_send_signal};
            match pidfd_send_signal(&self.identity, Signal::KILL) {
                Ok(()) | Err(rustix::io::Errno::SRCH) => {}
                Err(error) => return Err(error.into()),
            }
            self.child.wait().map(|_| ())
        }
        #[cfg(not(target_os = "linux"))]
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Linux pidfd supervision is required",
        ))
    }
}

fn failed_admission(child: &mut Child, error: &io::Error) -> io::Error {
    // Bootstrap failure is never a contained verdict. The still-owned Child
    // remains the cleanup authority before pidfd admission has succeeded.
    let cleanup = child.kill().and_then(|()| child.wait().map(|_| ()));
    let detail = match cleanup {
        Ok(()) => String::new(),
        Err(cleanup) => {
            format!("; owned launcher cleanup failed, it may still be running: {cleanup}")
        }
    };
    io::Error::new(
        error.kind(),
        format!("required Linux pidfd supervision is unavailable: {error}{detail}"),
    )
}

pub(super) struct Admission {
    namespaces: io::PipeReader,
    executable: io::PipeReader,
    parent_namespace: u64,
}

#[cfg(target_os = "linux")]
fn helper() -> io::Result<std::os::fd::OwnedFd> {
    use rustix::fs::{MemfdFlags, SealFlags, fcntl_add_seals, memfd_create};
    use std::io::Write as _;
    use std::os::fd::AsRawFd as _;
    use std::os::unix::fs::PermissionsExt as _;

    let fd = memfd_create(
        c"raf-exec-host",
        MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING | MemfdFlags::EXEC,
    )?;
    let mut file = std::fs::File::from(fd);
    file.write_all(crate::native_image::EXEC_HOST)?;
    file.set_permissions(std::fs::Permissions::from_mode(0o500))?;
    fcntl_add_seals(
        &file,
        SealFlags::SEAL | SealFlags::SHRINK | SealFlags::GROW | SealFlags::WRITE,
    )?;
    let readonly = std::fs::File::open(format!("/proc/self/fd/{}", file.as_raw_fd()))?;
    drop(file);
    Ok(readonly.into())
}

impl Admission {
    pub(super) fn program() -> &'static Path {
        // --sync-fd keeps the sealed executable only in the normal PID-1 reaper.
        Path::new("/proc/1/fd/4")
    }

    #[cfg(not(target_os = "linux"))]
    pub(super) fn prepare(_command: &mut Command) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native descendant containment requires Linux PID namespaces; this host has no admitted containment backend",
        ))
    }

    #[cfg(target_os = "linux")]
    pub(super) fn prepare(command: &mut Command) -> io::Result<Self> {
        use std::os::unix::fs::MetadataExt as _;
        let parent_namespace = std::fs::metadata("/proc/self/ns/pid")?.ino();
        let executable_file = helper().map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("required sealed executable memfd/proc access is unavailable: {error}"),
            )
        })?;
        let (namespaces, namespace_writer) = io::pipe()?;
        let (executable, exec_writer) = io::pipe()?;
        command
            .args([
                "--unshare-pid",
                "--die-with-parent",
                "--json-status-fd",
                "3",
                "--sync-fd",
                "4",
            ])
            .stdin(exec_writer)
            .fd_mappings(vec![
                FdMapping {
                    parent_fd: namespace_writer.into(),
                    child_fd: 3,
                },
                FdMapping {
                    parent_fd: executable_file,
                    child_fd: 4,
                },
            ])
            .map_err(io::Error::other)?;
        Ok(Self {
            namespaces,
            executable,
            parent_namespace,
        })
    }

    pub(super) fn capture(self) -> Status {
        let (send, receive) = mpsc::channel();
        let namespaces = send.clone();
        thread::spawn(move || {
            if let Err(message) =
                namespace_records(self.namespaces, self.parent_namespace, &namespaces)
            {
                let _ = namespaces.send(Notice::Failure(message));
            }
        });
        thread::spawn(move || {
            if let Err(error) = executable_records(self.executable, &send) {
                let _ = send.send(Notice::Failure(error.to_string()));
            }
        });
        Status {
            receive,
            namespace: false,
            executable: false,
            namespace_exit: None,
            executable_exit: None,
            spawn: None,
            failure: None,
            closed: false,
        }
    }
}

enum Notice {
    Namespace,
    NamespaceExit(u8),
    Executable,
    ExecutableExit(u8),
    Spawn(io::ErrorKind, String),
    Failure(String),
}

fn integer(value: &Value) -> Option<u64> {
    value.as_number()?.as_u64()
}

fn namespace_record(line: &str, first: bool, parent: u64) -> Result<Notice, String> {
    let value = json::read_with(
        line,
        json::Limits {
            max_depth: 1,
            max_values: 16,
            max_string_bytes: 64,
            unique_members: true,
        },
    )
    .map_err(|error| error.to_string())?;
    let object = value
        .as_object()
        .ok_or("launcher status is not an object")?;
    if !first {
        let code = object
            .get("exit-code")
            .and_then(integer)
            .and_then(|code| u8::try_from(code).ok());
        return match code {
            Some(code) if object.iter().len() == 1 => Ok(Notice::NamespaceExit(code)),
            _ => Err("invalid launcher exit status".into()),
        };
    }
    for (name, value) in object.iter() {
        if !matches!(
            name.as_str(),
            "child-pid"
                | "cgroup-namespace"
                | "ipc-namespace"
                | "mnt-namespace"
                | "net-namespace"
                | "pid-namespace"
                | "user-namespace"
                | "uts-namespace"
                | "time-namespace"
        ) || integer(value).is_none_or(|id| id == 0)
        {
            return Err(format!("invalid namespace status field {name:?}"));
        }
    }
    if object.get("child-pid").and_then(integer).is_none()
        || object
            .get("pid-namespace")
            .and_then(integer)
            .is_none_or(|id| id == parent)
    {
        return Err("launcher did not establish a distinct PID namespace".into());
    }
    Ok(Notice::Namespace)
}

fn namespace_records(
    reader: io::PipeReader,
    parent: u64,
    send: &Sender<Notice>,
) -> Result<(), String> {
    let mut reader = io::BufReader::new(reader.take((STATUS_LIMIT + 1) as u64));
    let mut total = 0;
    let mut count = 0;
    loop {
        let mut line = Vec::new();
        if reader
            .read_until(b'\n', &mut line)
            .map_err(|error| error.to_string())?
            == 0
        {
            return Ok(());
        }
        total += line.len();
        if total > STATUS_LIMIT || count >= 2 {
            return Err("launcher status exceeds its private record or byte limit".into());
        }
        // Forced teardown may close a record before its last write. Completion
        // still requires both complete final records; cancellation does not.
        if !line.ends_with(b"\n") {
            return Ok(());
        }
        let text = std::str::from_utf8(&line).map_err(|error| error.to_string())?;
        let notice = namespace_record(text, count == 0, parent)?;
        if send.send(notice).is_err() {
            return Ok(());
        }
        count += 1;
    }
}

fn executable_error(reader: &mut impl io::Read, tag: u8) -> io::Result<Notice> {
    let mut raw = [0; 4];
    let mut length = [0; 2];
    reader.read_exact(&mut raw)?;
    reader.read_exact(&mut length)?;
    let length = usize::from(u16::from_le_bytes(length));
    if length > control::MAX_ERROR {
        return Err(io::Error::other(
            "helper diagnostic exceeds its private byte limit",
        ));
    }
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes)?;
    let message = String::from_utf8(bytes).map_err(io::Error::other)?;
    let raw = i32::from_le_bytes(raw);
    let kind = if raw == 0 {
        io::ErrorKind::Other
    } else {
        io::Error::from_raw_os_error(raw).kind()
    };
    Ok(if tag == control::SPAWN_ERROR {
        Notice::Spawn(kind, message)
    } else {
        Notice::Failure(message)
    })
}

fn executable_records(mut reader: io::PipeReader, send: &Sender<Notice>) -> io::Result<()> {
    let mut started = false;
    let mut finished = false;
    loop {
        let mut tag = [0];
        if reader.read(&mut tag)? == 0 {
            return Ok(());
        }
        let notice = match tag[0] {
            control::READY if !started && !finished => {
                started = true;
                Notice::Executable
            }
            control::FINISHED if started && !finished => {
                let mut code = [0];
                if reader.read(&mut code)? == 0 {
                    return Ok(());
                }
                finished = true;
                Notice::ExecutableExit(code[0])
            }
            control::SPAWN_ERROR if !started && !finished => {
                finished = true;
                executable_error(&mut reader, tag[0])?
            }
            control::WAIT_ERROR if started && !finished => {
                finished = true;
                executable_error(&mut reader, tag[0])?
            }
            _ => return Err(io::Error::other("invalid helper control record sequence")),
        };
        if send.send(notice).is_err() {
            return Ok(());
        }
    }
}

pub(super) struct Status {
    receive: Receiver<Notice>,
    namespace: bool,
    executable: bool,
    namespace_exit: Option<u8>,
    executable_exit: Option<u8>,
    spawn: Option<(io::ErrorKind, String)>,
    failure: Option<String>,
    closed: bool,
}

impl Status {
    fn notice(&mut self, notice: Notice) {
        match notice {
            Notice::Namespace => self.namespace = true,
            Notice::NamespaceExit(code) => self.namespace_exit = Some(code),
            Notice::Executable => self.executable = true,
            Notice::ExecutableExit(code) => self.executable_exit = Some(code),
            Notice::Spawn(kind, message) => self.spawn = Some((kind, message)),
            Notice::Failure(message) => self.failure = Some(message),
        }
    }

    pub(super) fn poll(&mut self) {
        loop {
            match self.receive.try_recv() {
                Ok(notice) => self.notice(notice),
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    self.closed = true;
                    return;
                }
            }
        }
    }

    pub(super) fn drain(&mut self) {
        while let Ok(notice) = self.receive.recv() {
            self.notice(notice);
        }
        self.closed = true;
    }

    pub(super) fn confirmed(&self) -> bool {
        self.namespace && self.executable
    }
    pub(super) fn closed(&self) -> bool {
        self.closed
    }
    pub(super) fn failed(&self) -> bool {
        self.spawn.is_some() || self.failure.is_some()
    }

    pub(super) fn verify(
        &self,
        command: &str,
        program: &Path,
        code: Option<i32>,
        stderr: &[u8],
        complete: bool,
    ) -> Result<(), ExecError> {
        if let Some((kind, message)) = &self.spawn {
            return Err(ExecError::Spawn {
                command: command.to_owned(),
                program: program.display().to_string(),
                kind: *kind,
                message: format!("{message}; launcher stderr: {}", stderr.escape_ascii()),
            });
        }
        let issue = if let Some(message) = &self.failure {
            Some(message.as_str())
        } else if !self.confirmed() {
            Some("namespace setup or executable admission did not complete")
        } else if complete
            && (!self.closed
                || self.namespace_exit.is_none()
                || self.namespace_exit != self.executable_exit
                || self.namespace_exit.map(i32::from) != code)
        {
            Some("helper and launcher completion statuses are missing or disagree")
        } else {
            None
        };
        issue.map_or(Ok(()), |issue| Err(ExecError::Io { command: command.to_owned(),
            message: format!("mandatory Linux PID containment/setup or payload exec was not admitted for {}: {issue}; launcher stderr: {}", program.display(), stderr.escape_ascii()) }))
    }
}
