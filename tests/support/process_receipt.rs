// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Kernel-translated process receipts across owned PID namespaces.

use std::io::{self, IoSliceMut};
use std::mem::MaybeUninit;
use std::os::unix::net::UnixDatagram;
use std::path::Path;

use rustix::net::{RecvAncillaryBuffer, RecvAncillaryMessage, RecvFlags, recvmsg};
use rustix::process::Pid;

pub(crate) fn wait(fd: &std::os::fd::OwnedFd) -> io::Result<()> {
    use rustix::event::{PollFd, PollFlags, poll};
    loop {
        let mut fds = [PollFd::new(fd, PollFlags::IN)];
        match poll(&mut fds, None) {
            Ok(_) if fds[0].revents().contains(PollFlags::IN) => return Ok(()),
            Ok(_) | Err(rustix::io::Errno::INTR) => {}
            Err(error) => return Err(error.into()),
        }
    }
}

pub(crate) fn until_receipt(
    socket: &UnixDatagram,
    launcher: &std::os::fd::OwnedFd,
) -> io::Result<Option<(String, Pid)>> {
    use rustix::event::{PollFd, PollFlags, poll};
    loop {
        let mut fds = [
            PollFd::new(socket, PollFlags::IN),
            PollFd::new(launcher, PollFlags::IN),
        ];
        match poll(&mut fds, None) {
            Ok(_) if fds[0].revents().contains(PollFlags::IN) => return receive(socket).map(Some),
            Ok(_) if fds[1].revents().contains(PollFlags::IN) => return Ok(None),
            Ok(_) | Err(rustix::io::Errno::INTR) => {}
            Err(error) => return Err(error.into()),
        }
    }
}

pub(crate) fn listener(path: &Path) -> io::Result<UnixDatagram> {
    let socket = UnixDatagram::bind(path)?;
    rustix::net::sockopt::set_socket_passcred(&socket, true)?;
    socket.set_nonblocking(true)?;
    Ok(socket)
}

pub(crate) fn receive(socket: &UnixDatagram) -> io::Result<(String, Pid)> {
    let mut bytes = [0_u8; 32];
    let mut space =
        [MaybeUninit::uninit(); rustix::net::__cmsg_space(size_of::<rustix::net::UCred>())];
    let mut control = RecvAncillaryBuffer::new(&mut space);
    let message = recvmsg(
        socket,
        &mut [IoSliceMut::new(&mut bytes)],
        &mut control,
        RecvFlags::empty(),
    )?;
    if !message.flags.is_empty() {
        return Err(io::Error::other("truncated process receipt"));
    }
    let mut pid = None;
    for message in control.drain() {
        let RecvAncillaryMessage::ScmCredentials(credentials) = message else {
            return Err(io::Error::other(
                "unexpected process receipt control message",
            ));
        };
        if pid.replace(credentials.pid).is_some() {
            return Err(io::Error::other("duplicate process credentials"));
        }
    }
    let role = std::str::from_utf8(&bytes[..message.bytes])
        .map_err(io::Error::other)?
        .to_owned();
    Ok((
        role,
        pid.ok_or_else(|| io::Error::other("missing kernel process credentials"))?,
    ))
}
