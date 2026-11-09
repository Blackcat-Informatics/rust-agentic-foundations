// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Primary native metadata operations, with explicit unsupported-platform refusal.

use raf_source::DiskTree;
use std::{
    fs::{File, Metadata},
    io,
    path::PathBuf,
};

#[cfg(target_os = "linux")]
use rustix::fs::{self, AtFlags, Mode, OFlags, RenameFlags};

#[cfg(target_os = "linux")]
pub(super) fn root(tree: &DiskTree) -> io::Result<File> {
    tree.directory_handle().map_err(io::Error::other)
}
#[cfg(target_os = "linux")]
pub(super) fn identity(file: &File) -> io::Result<(u64, u64)> {
    Ok(stat(file)?.1)
}
#[cfg(target_os = "linux")]
pub(super) fn stat(file: &File) -> io::Result<(Metadata, (u64, u64), u64)> {
    use std::os::unix::fs::MetadataExt as _;
    let metadata = file.metadata()?;
    let identity = (metadata.dev(), metadata.ino());
    let links = metadata.nlink();
    Ok((metadata, identity, links))
}
#[cfg(target_os = "linux")]
fn open(directory: &File, name: &str, flags: OFlags, mode: Mode) -> io::Result<File> {
    fs::openat(
        directory,
        name,
        flags | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        mode,
    )
    .map(File::from)
    .map_err(Into::into)
}
#[cfg(target_os = "linux")]
pub(super) fn directory(parent: &File, name: &str) -> io::Result<File> {
    open(
        parent,
        name,
        OFlags::PATH | OFlags::DIRECTORY,
        Mode::empty(),
    )
}
#[cfg(target_os = "linux")]
pub(super) fn reference(parent: &File, name: &str) -> io::Result<File> {
    open(parent, name, OFlags::PATH, Mode::empty())
}
#[cfg(target_os = "linux")]
pub(super) fn read(reference: &File) -> io::Result<File> {
    // The borrowed descriptor stays live until open returns. Following this
    // kernel link reopens that exact object, without resolving its old name.
    File::open(descriptor_path(reference))
}
#[cfg(target_os = "linux")]
fn descriptor_path(file: &File) -> PathBuf {
    use std::os::fd::AsRawFd as _;
    PathBuf::from(format!("/proc/self/fd/{}", file.as_raw_fd()))
}
#[cfg(target_os = "linux")]
pub(super) fn root_path(file: &File) -> io::Result<PathBuf> {
    std::fs::read_link(descriptor_path(file))
}
#[cfg(target_os = "linux")]
pub(super) fn create(parent: &File, name: &str) -> io::Result<File> {
    open(
        parent,
        name,
        OFlags::RDWR | OFlags::CREATE | OFlags::EXCL,
        Mode::from_bits_truncate(0o666),
    )
}
#[cfg(target_os = "linux")]
pub(super) fn private(file: &File) -> io::Result<()> {
    fs::fchmod(file, Mode::from_bits_truncate(0o600)).map_err(Into::into)
}
#[cfg(target_os = "linux")]
pub(super) fn mkdir(parent: &File, name: &str) -> io::Result<()> {
    fs::mkdirat(parent, name, Mode::from_bits_truncate(0o777)).map_err(Into::into)
}
#[cfg(target_os = "linux")]
pub(super) fn unlink(parent: &File, name: &str) -> io::Result<()> {
    fs::unlinkat(parent, name, AtFlags::empty()).map_err(Into::into)
}
#[cfg(target_os = "linux")]
pub(super) fn rename(parent: &File, from: &str, to: &str, absent: bool) -> io::Result<()> {
    if absent {
        fs::renameat_with(parent, from, parent, to, RenameFlags::NOREPLACE)
    } else {
        fs::renameat(parent, from, parent, to)
    }
    .map_err(Into::into)
}

#[cfg(not(target_os = "linux"))]
macro_rules! unsupported {
    ($($name:ident($($arg:ident: $ty:ty),*) -> $result:ty;)*) => { $(
        pub(super) fn $name($($arg: $ty),*) -> io::Result<$result> {
            $(let _ = $arg;)*
            Err(io::Error::new(io::ErrorKind::Unsupported, "descriptor-relative metadata publication requires the primary native Linux target"))
        }
    )* };
}
#[cfg(not(target_os = "linux"))]
unsupported! {
    root(tree: &DiskTree) -> File;
    root_path(file: &File) -> PathBuf;
    identity(file: &File) -> (u64, u64);
    stat(file: &File) -> (Metadata, (u64, u64), u64);
    directory(parent: &File, name: &str) -> File;
    reference(parent: &File, name: &str) -> File;
    read(reference: &File) -> File;
    create(parent: &File, name: &str) -> File;
    private(file: &File) -> ();
    mkdir(parent: &File, name: &str) -> ();
    unlink(parent: &File, name: &str) -> ();
    rename(parent: &File, from: &str, to: &str, absent: bool) -> ();
}
