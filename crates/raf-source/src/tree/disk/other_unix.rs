// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Unix systems other than Linux: a walk beneath a descriptor of the root.
//!
//! The root is opened once. A path is walked one component at a time with
//! `openat` relative to the directory reached so far, never following a
//! symlink (`O_NOFOLLOW`) and never blocking (`O_NONBLOCK`). The walk holds the
//! descriptor of that directory only, and records the device and inode of the
//! directories it came through: a `..` opens `..` of the current directory and
//! must find the recorded parent, and a `..` at the root leaves the root.
//!
//! A symlink is read with `readlinkat` and its target walked in its place: a
//! relative target from the directory that holds the link, an absolute one
//! from the file system root. A walk that is outside the root may only descend:
//! it is back inside when it reaches a directory that is the root (same device
//! and inode), and a `..` taken outside is `OutsideRoot`, so a link cannot leave
//! the root and climb back into it. Outside the root only directories are
//! opened (`O_DIRECTORY`, which opens no device and no FIFO), a final component
//! is never opened, and every failure is `OutsideRoot`. An outside path
//! therefore answers like a missing one, except a path that leads into the
//! root itself by descending, such as an outside symlink to the root.
//!
//! No path is resolved by name. An entry that changes between two calls of
//! one step (a symlink that becomes a directory, or the reverse) is stepped
//! again, within the symlink budget. A directory renamed out of the root while
//! the walk is inside it is still walked; the check after the use refuses it.
//!
//! The file is read, the directory listed (`fdopendir` of the verified
//! descriptor) and the entry looked up (`fstatat` of the verified directory,
//! without following) through the handle. After each use the handle must
//! still lie under the root: a file must still be the entry it was found as in
//! its directory, and that directory, or a verified directory itself, must
//! reach the root through `..`, so a file or directory moved out of the root
//! during the use is refused.

use std::collections::VecDeque;
use std::ffi::{OsStr, OsString};
use std::fs::{File, Metadata};
use std::os::fd::OwnedFd;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

use rustix::fs::{
    AtFlags, CWD, Dir, FileType, Mode, OFlags, Stat, fstat, openat, readlinkat, statat,
};

use super::{Entry, EntryKind, Want, device_identity, not_regular, outside, read_capped};
use crate::error::TreeError;
use crate::path::RelPath;
use crate::tree::FileId;

use super::DiskTree;

/// The most symlinks one walk follows before it ends with `SymlinkLoop`, as
/// Linux's own limit.
const SYMLINK_HOPS: usize = 40;

/// The most `..` steps the check after a use climbs looking for the root.
const CLIMB_LIMIT: usize = 4096;

/// A descriptor of the root and its status, opened once per tree.
pub(super) struct Root {
    dir: OwnedFd,
    status: Stat,
}

impl Root {
    /// Open the canonical root `path` as a directory.
    pub(super) fn open(path: &Path, shown: &str) -> Result<Self, TreeError> {
        let dir = openat(
            CWD,
            path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|e| TreeError::from_io(shown, e.into()))?;
        let status = fstat(&dir).map_err(|e| TreeError::from_io(shown, e.into()))?;
        Ok(Self { dir, status })
    }

    /// True when `status` is the root's.
    fn is(&self, status: &Stat) -> bool {
        same(status, &self.status)
    }
}

/// True when two statuses name one file.
fn same(left: &Stat, right: &Stat) -> bool {
    left.st_dev == right.st_dev && left.st_ino == right.st_ino
}

/// A handle reached by the walk.
pub(super) struct Handle {
    file: File,
    metadata: Metadata,
    /// For a file, the directory it was found in and its name there.
    parent: Option<(OwnedFd, OsString)>,
}

/// One step of a walk.
enum Step {
    /// `..`: back to the directory the walk came from.
    Up,
    /// A name in the directory reached so far.
    Name(OsString),
}

/// The steps of `/`-separated `text`, without empty and `.` components.
fn steps(text: &[u8]) -> Vec<Step> {
    text.split(|byte| *byte == b'/')
        .filter(|part| !part.is_empty() && *part != b".")
        .map(|part| {
            if part == b".." {
                Step::Up
            } else {
                Step::Name(OsStr::from_bytes(part).to_os_string())
            }
        })
        .collect()
}

/// Walk `path` beneath the root and verify the handle against `want`.
pub(super) fn open(
    tree: &DiskTree,
    path: &RelPath,
    want: Want,
    shown: &str,
) -> Result<Handle, TreeError> {
    hook!(tree, before_open);
    let (fd, parent) = walk(tree, path, want, shown)?;
    hook!(tree, after_open);
    let file = File::from(fd);
    let metadata = file.metadata().map_err(|e| TreeError::from_io(shown, e))?;
    want.require(&metadata, shown)?;
    Ok(Handle {
        file,
        metadata,
        parent,
    })
}

/// The descriptor a walk ends on, and for a file the directory that holds it
/// and its name there.
type Walked = (OwnedFd, Option<(OwnedFd, OsString)>);

/// The state of one walk.
struct Walk<'a> {
    tree: &'a DiskTree,
    shown: &'a str,
    /// The directory reached so far, and its status.
    current: OwnedFd,
    current_status: Stat,
    /// The status of each directory the walk came through inside the root,
    /// for `..`; no descriptor is held for them.
    ancestors: Vec<Stat>,
    /// Whether `current` lies under the root.
    inside: bool,
    /// The steps still to take.
    queue: VecDeque<Step>,
    /// The symlinks followed so far.
    hops: usize,
    /// A listing refuses a final symlink, but only once its target is known
    /// to lie inside the root, so one that leads out is `OutsideRoot`.
    final_link: bool,
}

impl Walk<'_> {
    /// The root of the walk.
    fn root(&self) -> &Root {
        &self.tree.walk_root
    }

    /// A failure: inside the root, what the system said; outside it,
    /// `OutsideRoot` whatever the system said.
    fn fail(&self, error: std::io::Error) -> TreeError {
        if self.inside {
            TreeError::from_io(self.shown, error)
        } else {
            outside(self.shown)
        }
    }

    /// `..`: back to the directory the walk came from, which `..` of the
    /// current directory must still be. At the root, or anywhere outside it,
    /// it is `OutsideRoot`: a walk never climbs back into the root.
    fn up(&mut self) -> Result<(), TreeError> {
        if !self.inside {
            return Err(outside(self.shown));
        }
        let Some(parent) = self.ancestors.pop() else {
            return Err(outside(self.shown));
        };
        let opened = openat(
            &self.current,
            "..",
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| outside(self.shown))?;
        let reached = fstat(&opened).map_err(|_| outside(self.shown))?;
        if !same(&reached, &parent) {
            // The current directory was moved since the walk entered it.
            return Err(outside(self.shown));
        }
        self.current = opened;
        self.current_status = reached;
        Ok(())
    }

    /// Step into the directory `opened`; reaching the root from outside puts
    /// the walk inside it.
    fn descend(&mut self, opened: OwnedFd) -> Result<(), TreeError> {
        let reached = fstat(&opened).map_err(|e| self.fail(e.into()))?;
        if !self.inside && self.root().is(&reached) {
            self.ancestors.clear();
            self.inside = true;
        } else if self.inside {
            let left = std::mem::replace(&mut self.current_status, reached);
            self.ancestors.push(left);
            self.current = opened;
            return Ok(());
        }
        self.current = opened;
        self.current_status = reached;
        Ok(())
    }

    /// Count one more symlink, or one more retaken step; past the budget the
    /// walk ends with `SymlinkLoop`, or `OutsideRoot` outside the root.
    fn hop(&mut self) -> Result<(), TreeError> {
        self.hops += 1;
        if self.hops > SYMLINK_HOPS {
            return Err(if self.inside {
                TreeError::SymlinkLoop {
                    path: self.shown.to_owned(),
                }
            } else {
                outside(self.shown)
            });
        }
        Ok(())
    }

    /// The entry `name` changed between two calls of one step (a symlink
    /// became something else, or the reverse): take the step again.
    fn retake(&mut self, name: OsString) -> Result<(), TreeError> {
        self.hop()?;
        self.queue.push_front(Step::Name(name));
        Ok(())
    }

    /// Put the target of the symlink `name` in its place: a relative target
    /// from the directory that holds the link, an absolute one from the file
    /// system root.
    fn follow(&mut self, name: OsString) -> Result<(), TreeError> {
        self.hop()?;
        let target = match readlinkat(&self.current, name.as_os_str(), Vec::new()) {
            Ok(target) => target,
            // No longer a symlink.
            Err(rustix::io::Errno::INVAL) => return self.retake(name),
            Err(e) => return Err(self.fail(e.into())),
        };
        let target = target.as_bytes();
        if target.starts_with(b"/") {
            self.current = openat(
                CWD,
                "/",
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|e| self.fail(e.into()))?;
            self.ancestors.clear();
            self.current_status = fstat(&self.current).map_err(|e| self.fail(e.into()))?;
            self.inside = self.root().is(&self.current_status);
        }
        for step in steps(target).into_iter().rev() {
            self.queue.push_front(step);
        }
        Ok(())
    }

    /// Take the step `name`; `Some` when it was the final one.
    fn name(&mut self, name: OsString, want: Want) -> Result<Option<Walked>, TreeError> {
        let last = self.queue.is_empty();
        if last && !self.inside {
            return Err(outside(self.shown));
        }
        let directory = !last || want != Want::File;
        let mut flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK;
        if directory {
            flags |= OFlags::DIRECTORY;
        }
        #[cfg(test)]
        self.tree.note_open(flags.bits());
        match openat(&self.current, name.as_os_str(), flags, Mode::empty()) {
            Ok(_) if last && self.final_link => Err(not_regular(self.shown)),
            Ok(opened) if last => {
                let parent = if want == Want::File {
                    let directory = self
                        .current
                        .try_clone()
                        .map_err(|e| TreeError::from_io(self.shown, e))?;
                    Some((directory, name))
                } else {
                    None
                };
                Ok(Some((opened, parent)))
            }
            Ok(opened) => self.descend(opened).map(|()| None),
            Err(error) => {
                hook!(self.tree, mid_step);
                let status = statat(&self.current, name.as_os_str(), AtFlags::SYMLINK_NOFOLLOW)
                    .map_err(|e| self.fail(e.into()))?;
                let kind = FileType::from_raw_mode(status.st_mode);
                if kind != FileType::Symlink {
                    if changed(error, kind, directory) {
                        return self.retake(name).map(|()| None);
                    }
                    if last && directory && error == rustix::io::Errno::NOTDIR {
                        return Err(not_regular(self.shown));
                    }
                    return Err(self.fail(error.into()));
                }
                self.final_link |= last && want == Want::Listing;
                self.follow(name).map(|()| None)
            }
        }
    }
}

/// True when an open that failed with `error` would now succeed, because the
/// entry it met changed before its status was read: a symlink that is gone
/// (`ELOOP`, or `EMLINK` on FreeBSD; `ENOTDIR` for a directory open on Linux),
/// or an entry that has appeared.
fn changed(error: rustix::io::Errno, kind: FileType, directory: bool) -> bool {
    use rustix::io::Errno;
    match error {
        Errno::LOOP | Errno::MLINK | Errno::NOENT => true,
        Errno::NOTDIR => directory && kind == FileType::Directory,
        _ => false,
    }
}

/// Walk `path` from the root. Returns the descriptor of its final component
/// and, for a file, the directory that holds it and its name there.
fn walk(tree: &DiskTree, path: &RelPath, want: Want, shown: &str) -> Result<Walked, TreeError> {
    let root = &tree.walk_root;
    let current = root
        .dir
        .try_clone()
        .map_err(|e| TreeError::from_io(shown, e))?;
    let current_status = fstat(&current).map_err(|e| TreeError::from_io(shown, e.into()))?;
    let mut walk = Walk {
        tree,
        shown,
        current,
        current_status,
        ancestors: Vec::new(),
        inside: true,
        queue: steps(path.as_str().as_bytes()).into(),
        hops: 0,
        final_link: false,
    };
    while let Some(step) = walk.queue.pop_front() {
        match step {
            Step::Up => walk.up()?,
            Step::Name(name) => {
                if let Some(walked) = walk.name(name, want)? {
                    return Ok(walked);
                }
            }
        }
    }
    // The path ends on the directory reached last: the root, or after `..`.
    if !walk.inside {
        Err(outside(shown))
    } else if walk.final_link {
        Err(not_regular(shown))
    } else {
        Ok((walk.current, None))
    }
}

/// Climb from the directory `current` through `..` until the root is reached;
/// reaching the file system root first, or failing, is `OutsideRoot`.
fn climb(mut current: OwnedFd, root: &Root, shown: &str) -> Result<(), TreeError> {
    let mut status = fstat(&current).map_err(|_| outside(shown))?;
    for _ in 0..CLIMB_LIMIT {
        if root.is(&status) {
            return Ok(());
        }
        let up = openat(
            &current,
            "..",
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| outside(shown))?;
        let above = fstat(&up).map_err(|_| outside(shown))?;
        if same(&above, &status) {
            return Err(outside(shown));
        }
        current = up;
        status = above;
    }
    Err(outside(shown))
}

/// What a file type from `rustix` is.
fn kind_of(file_type: FileType) -> EntryKind {
    match file_type {
        FileType::Symlink => EntryKind::Symlink,
        FileType::Directory => EntryKind::Dir,
        FileType::RegularFile => EntryKind::File,
        _ => EntryKind::Other,
    }
}

impl Handle {
    /// Fresh metadata from the verified descriptor.
    pub(super) fn metadata(&self, shown: &str) -> Result<Metadata, TreeError> {
        self.file
            .metadata()
            .map_err(|e| TreeError::from_io(shown, e))
    }

    /// Describe a link relative to the verified containing directory.
    pub(super) fn link(
        &self,
        name: &str,
        shown: &str,
        between: impl FnOnce(),
    ) -> Result<(FileId, PathBuf), TreeError> {
        let io = |e: rustix::io::Errno| TreeError::from_io(shown, e.into());
        let before = statat(&self.file, name, AtFlags::SYMLINK_NOFOLLOW).map_err(io)?;
        if FileType::from_raw_mode(before.st_mode) != FileType::Symlink {
            return Err(not_regular(shown));
        }
        let target = readlinkat(&self.file, name, Vec::new()).map_err(io)?;
        between();
        let repeated = readlinkat(&self.file, name, Vec::new()).map_err(io)?;
        let after = statat(&self.file, name, AtFlags::SYMLINK_NOFOLLOW).map_err(io)?;
        if !same(&before, &after) || before.st_mode != after.st_mode || target != repeated {
            return Err(super::capture::changed(shown));
        }
        Ok((
            FileId::from_device(after.st_dev, after.st_ino),
            PathBuf::from(OsStr::from_bytes(target.to_bytes())),
        ))
    }

    /// The device and inode of the handle.
    #[expect(
        clippy::unnecessary_wraps,
        reason = "the signature is shared with Windows, where the identity can fail"
    )]
    pub(super) fn identity(&self, _root: &Path, _shown: &str) -> Result<FileId, TreeError> {
        Ok(device_identity(&self.metadata))
    }

    /// The handle must still lie under the root: a file must still be the
    /// entry it was found as, and its directory, or the verified directory,
    /// must reach the root through `..`.
    pub(super) fn recheck(&self, root: &Root, shown: &str) -> Result<(), TreeError> {
        let start = match &self.parent {
            Some((parent, name)) => {
                let found = statat(parent, name.as_os_str(), AtFlags::SYMLINK_NOFOLLOW)
                    .map_err(|_| outside(shown))?;
                let held = fstat(&self.file).map_err(|_| outside(shown))?;
                if !same(&found, &held) {
                    return Err(outside(shown));
                }
                parent.try_clone()
            }
            None => self.file.try_clone().map(OwnedFd::from),
        }
        .map_err(|e| TreeError::from_io(shown, e))?;
        climb(start, root, shown)
    }

    /// Read at most `max_bytes` raw bytes from the handle.
    pub(super) fn read(&self, max_bytes: u64, shown: &str) -> Result<Vec<u8>, TreeError> {
        read_capped(&self.file, max_bytes, shown)
    }

    /// List the directory of the handle.
    pub(super) fn entries(&self, shown: &str) -> Result<Vec<Entry>, TreeError> {
        let io = |e: rustix::io::Errno| TreeError::from_io(shown, e.into());
        let mut listing = Dir::read_from(&self.file).map_err(io)?;
        let mut entries = Vec::new();
        while let Some(item) = listing.read() {
            let item = item.map_err(io)?;
            let raw = item.file_name();
            let bytes = raw.to_bytes();
            if bytes == b"." || bytes == b".." {
                continue;
            }
            let name = std::str::from_utf8(bytes).map_err(|_| TreeError::NonUtf8Name {
                path: shown.to_owned(),
                name: String::from_utf8_lossy(bytes).into_owned(),
            })?;
            let file_type = match item.file_type() {
                FileType::Unknown => FileType::from_raw_mode(
                    statat(&self.file, raw, AtFlags::SYMLINK_NOFOLLOW)
                        .map_err(io)?
                        .st_mode,
                ),
                known => known,
            };
            entries.push(Entry {
                name: name.to_owned(),
                kind: kind_of(file_type),
            });
        }
        Ok(entries)
    }

    /// What `name` is in the directory of the handle, without following it.
    pub(super) fn lookup(&self, name: &str, shown: &str) -> Result<Option<EntryKind>, TreeError> {
        match statat(&self.file, name, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(status) => Ok(Some(kind_of(FileType::from_raw_mode(status.st_mode)))),
            Err(rustix::io::Errno::NOENT | rustix::io::Errno::NOTDIR) => Ok(None),
            Err(e) => Err(TreeError::from_io(shown, e.into())),
        }
    }
}
