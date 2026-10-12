// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The disk backend, and the only module of the crate that touches the file
//! system.
//!
//! A file read goes through these steps, and none of them trusts a check that
//! the next one could outlive:
//!
//! 1. the candidate path is canonicalised, which follows every symlink and
//!    applies every `..` against the real directories;
//! 2. the canonical path must lie under the canonical root;
//! 3. the canonical path is opened non-blocking and without following a final
//!    symlink (Unix), so naming a FIFO cannot hang the open;
//! 4. the opened handle is verified to lie under the root, by the strongest
//!    check the platform has (see [`verify_open_handle`]);
//! 5. the handle must be a regular file;
//! 6. at most `max_bytes + 1` bytes are read, and the file is refused when
//!    the extra byte arrives.
//!
//! A listing opens the canonical directory the same way, verifies that handle
//! with the same per-platform check, and reads the entries through the handle
//! while it is held.

use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read as _};
use std::path::{Path, PathBuf};

use super::{Entry, EntryKind, Tree};
use crate::error::{TreeError, is_symlink_loop};
use crate::path::RelPath;

#[cfg(test)]
mod tests;

/// A callback that runs between the prefix check and the open.
#[cfg(test)]
type OpenHook = Box<dyn Fn(&Path) + Send + Sync>;

/// A tree read from a directory on disk.
///
/// The root is canonicalised once, in [`DiskTree::new`]. A symlink inside the
/// tree is followed only when its target is also inside the tree.
pub struct DiskTree {
    root: PathBuf,
    /// Runs with the canonical path after the prefix check and before the
    /// open, so a test can change the disk in exactly that window.
    #[cfg(test)]
    before_open: Option<OpenHook>,
}

impl std::fmt::Debug for DiskTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiskTree")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl DiskTree {
    /// Open a tree rooted at `root`, canonicalising it once.
    ///
    /// A symlinked root is followed: the tree is rooted at its target.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::NotFound`] when the root does not exist,
    /// [`TreeError::NotRegular`] when it is not a directory, and
    /// [`TreeError::SymlinkLoop`] or [`TreeError::Io`] when the operating
    /// system fails.
    pub fn new(root: impl AsRef<Path>) -> Result<Self, TreeError> {
        let given = root.as_ref();
        let shown = given.display().to_string();
        let root = fs::canonicalize(given).map_err(|e| TreeError::from_io(&shown, e))?;
        let metadata = fs::metadata(&root).map_err(|e| TreeError::from_io(&shown, e))?;
        if !metadata.is_dir() {
            return Err(TreeError::NotRegular { path: shown });
        }
        Ok(Self {
            root,
            #[cfg(test)]
            before_open: None,
        })
    }

    /// The path on disk that `path` names, before any symlink is followed.
    fn candidate(&self, path: &RelPath) -> PathBuf {
        if path.is_root() {
            self.root.clone()
        } else {
            self.root.join(path.as_str())
        }
    }

    /// Canonicalise `candidate` and require the canonical root as its prefix.
    fn contain(&self, candidate: &Path, shown: &str) -> Result<PathBuf, TreeError> {
        let canonical = fs::canonicalize(candidate).map_err(|e| TreeError::from_io(shown, e))?;
        if canonical.starts_with(&self.root) {
            Ok(canonical)
        } else {
            Err(TreeError::OutsideRoot {
                path: shown.to_owned(),
            })
        }
    }
}

impl Tree for DiskTree {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        let shown = path.to_string();
        let canonical = self.contain(&self.candidate(path), &shown)?;
        #[cfg(test)]
        if let Some(hook) = &self.before_open {
            hook(&canonical);
        }
        let file = open_handle(&canonical).map_err(|e| open_error(&shown, e))?;
        verify_open_handle(&file, &canonical, &self.root, &shown)?;
        read_capped(file, max_bytes, &shown)
    }

    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        let shown = path.to_string();
        let candidate = self.candidate(path);
        let canonical = self.contain(&candidate, &shown)?;
        let own = fs::symlink_metadata(&candidate).map_err(|e| TreeError::from_io(&shown, e))?;
        if !own.is_dir() {
            return Err(TreeError::NotRegular { path: shown });
        }
        #[cfg(test)]
        if let Some(hook) = &self.before_open {
            hook(&canonical);
        }
        // The handle is held until the entries are read, so the listing is of
        // the directory that was verified.
        let (_held, listing) = open_verified_directory(&canonical, &self.root, &shown)?;
        let mut entries = Vec::new();
        for item in fs::read_dir(&listing).map_err(|e| TreeError::from_io(&shown, e))? {
            let item = item.map_err(|e| TreeError::from_io(&shown, e))?;
            let name = item
                .file_name()
                .into_string()
                .map_err(|raw| TreeError::NonUtf8Name {
                    path: shown.clone(),
                    name: raw.to_string_lossy().into_owned(),
                })?;
            let file_type = item
                .file_type()
                .map_err(|e| TreeError::from_io(&shown, e))?;
            entries.push(Entry {
                name,
                kind: kind_of(file_type),
            });
        }
        entries.sort();
        Ok(entries)
    }

    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        let shown = path.to_string();
        let (Some(name), Some(parent)) = (path.file_name(), path.parent()) else {
            return Ok(Some(EntryKind::Dir));
        };
        if name == ".." {
            return match self.contain(&self.candidate(path), &shown) {
                Ok(_) => Ok(Some(EntryKind::Dir)),
                Err(TreeError::NotFound { .. }) => Ok(None),
                Err(other) => Err(other),
            };
        }
        let directory = match self.contain(&self.candidate(&parent), &shown) {
            Ok(directory) => directory,
            Err(TreeError::NotFound { .. }) => return Ok(None),
            Err(other) => return Err(other),
        };
        match fs::symlink_metadata(directory.join(name)) {
            Ok(metadata) => Ok(Some(kind_of(metadata.file_type()))),
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
                ) =>
            {
                Ok(None)
            }
            Err(e) => Err(TreeError::from_io(&shown, e)),
        }
    }
}

fn kind_of(file_type: fs::FileType) -> EntryKind {
    if file_type.is_symlink() {
        EntryKind::Symlink
    } else if file_type.is_dir() {
        EntryKind::Dir
    } else if file_type.is_file() {
        EntryKind::File
    } else {
        EntryKind::Other
    }
}

/// Classify a failure to open: a symlink in the final position (the canonical
/// path was swapped for a link after it was checked) is not a regular file.
fn open_error(shown: &str, error: io::Error) -> TreeError {
    if is_symlink_loop(&error) {
        TreeError::NotRegular {
            path: shown.to_owned(),
        }
    } else {
        TreeError::from_io(shown, error)
    }
}

/// Open `path` for reading without blocking and without following a final
/// symlink.
#[cfg(unix)]
fn open_handle(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt as _;
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW)
        .open(path)
}

/// Open `path` for reading; the platform has no non-blocking open here, so the
/// regular-file check on the handle is the guard.
#[cfg(not(unix))]
fn open_handle(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).open(path)
}

/// Open the directory `path` for listing without blocking and without
/// following a final symlink.
#[cfg(unix)]
fn open_directory(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt as _;
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
}

/// A directory cannot be opened as a handle without Unix; dispatch in
/// [`open_verified_directory`] never selects this.
#[cfg(not(unix))]
fn open_directory(path: &Path) -> io::Result<File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        format!("cannot open the directory {} as a handle", path.display()),
    ))
}

/// Open `canonical` as a directory and verify the handle against `root` the
/// way a file read does. Returns the handle, which the caller keeps open while
/// it lists, and the path to list: on Linux the handle's own descriptor path,
/// so the entries are those of the verified directory.
fn open_verified_directory(
    canonical: &Path,
    root: &Path,
    shown: &str,
) -> Result<(Option<File>, PathBuf), TreeError> {
    if cfg!(unix) {
        let handle = open_directory(canonical).map_err(|e| open_error(shown, e))?;
        let on_handle = handle
            .metadata()
            .map_err(|e| TreeError::from_io(shown, e))?;
        let listing = if cfg!(target_os = "linux") {
            let link = descriptor_link(&handle, shown)?;
            verify_linux_link(root, &link, shown)?;
            link
        } else {
            let after = fs::canonicalize(canonical).map_err(|e| TreeError::from_io(shown, e))?;
            let on_disk = fs::metadata(&after).map_err(|e| TreeError::from_io(shown, e))?;
            verify_same_file(
                root,
                &after,
                file_id(&on_disk, shown)?,
                file_id(&on_handle, shown)?,
                shown,
            )?;
            canonical.to_path_buf()
        };
        ensure_directory(&on_handle, shown)?;
        Ok((Some(handle), listing))
    } else {
        let on_disk = fs::metadata(canonical).map_err(|e| TreeError::from_io(shown, e))?;
        let after = fs::canonicalize(canonical).map_err(|e| TreeError::from_io(shown, e))?;
        verify_portable_directory(root, &on_disk, &after, shown)?;
        Ok((None, canonical.to_path_buf()))
    }
}

/// Read at most `max_bytes` bytes of text from the verified handle.
fn read_capped(file: File, max_bytes: u64, shown: &str) -> Result<String, TreeError> {
    let mut bytes = Vec::new();
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|e| TreeError::from_io(shown, e))?;
    if u64::try_from(bytes.len()).map_or(true, |len| len > max_bytes) {
        return Err(TreeError::TooLarge {
            path: shown.to_owned(),
            limit: max_bytes,
        });
    }
    String::from_utf8(bytes).map_err(|_| TreeError::NotUtf8 {
        path: shown.to_owned(),
    })
}

/// A file's identity on disk: its device and inode numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileId {
    dev: u64,
    ino: u64,
}

#[cfg(unix)]
#[cfg_attr(
    unix,
    allow(
        clippy::unnecessary_wraps,
        reason = "the signature is shared with the platforms where the call can fail"
    )
)]
fn file_id(metadata: &Metadata, _shown: &str) -> Result<FileId, TreeError> {
    use std::os::unix::fs::MetadataExt as _;
    Ok(FileId {
        dev: metadata.dev(),
        ino: metadata.ino(),
    })
}

#[cfg(not(unix))]
fn file_id(_metadata: &Metadata, shown: &str) -> Result<FileId, TreeError> {
    Err(unsupported(shown, "file identity"))
}

/// The path of the kernel's view of an open handle (`/proc/self/fd/<fd>`).
#[cfg(unix)]
#[cfg_attr(
    unix,
    allow(
        clippy::unnecessary_wraps,
        reason = "the signature is shared with the platforms where the call can fail"
    )
)]
fn descriptor_link(file: &File, _shown: &str) -> Result<PathBuf, TreeError> {
    use std::os::fd::AsRawFd as _;
    Ok(PathBuf::from(format!("/proc/self/fd/{}", file.as_raw_fd())))
}

#[cfg(not(unix))]
fn descriptor_link(_file: &File, shown: &str) -> Result<PathBuf, TreeError> {
    Err(unsupported(shown, "descriptor paths"))
}

/// An operation the platform lacks. Dispatch in [`verify_open_handle`] never
/// selects the check that needs it, so this is a guard and not a path.
#[cfg(not(unix))]
fn unsupported(shown: &str, what: &str) -> TreeError {
    TreeError::Io {
        path: shown.to_owned(),
        source: io::Error::new(io::ErrorKind::Unsupported, format!("no {what} here")),
    }
}

/// Refuse a handle that is not a regular file.
fn ensure_regular(metadata: &Metadata, shown: &str) -> Result<(), TreeError> {
    if metadata.file_type().is_file() {
        Ok(())
    } else {
        Err(TreeError::NotRegular {
            path: shown.to_owned(),
        })
    }
}

/// Refuse a handle that is not a directory.
fn ensure_directory(metadata: &Metadata, shown: &str) -> Result<(), TreeError> {
    if metadata.file_type().is_dir() {
        Ok(())
    } else {
        Err(TreeError::NotRegular {
            path: shown.to_owned(),
        })
    }
}

fn outside(shown: &str) -> TreeError {
    TreeError::OutsideRoot {
        path: shown.to_owned(),
    }
}

/// A failure to read the descriptor link. A missing link (no `/proc`) is an
/// operating-system failure, never absence of the file being read.
fn link_read_error(shown: &str, source: io::Error) -> TreeError {
    TreeError::Io {
        path: shown.to_owned(),
        source,
    }
}

/// Linux: the kernel's own path for the open handle must lie under the root.
///
/// `handle_link` is `/proc/self/fd/<fd>`. This reads the file the handle
/// really is, so no check is separated from the open it guards.
fn verify_linux_link(root: &Path, handle_link: &Path, shown: &str) -> Result<(), TreeError> {
    let target = fs::read_link(handle_link).map_err(|e| link_read_error(shown, e))?;
    if target.starts_with(root) {
        Ok(())
    } else {
        Err(outside(shown))
    }
}

/// Other Unix: the path canonicalised again after the open must lie under the
/// root and must be the file the handle is (same device and inode).
fn verify_same_file(
    root: &Path,
    canonical_after: &Path,
    on_disk: FileId,
    on_handle: FileId,
    shown: &str,
) -> Result<(), TreeError> {
    if canonical_after.starts_with(root) && on_disk == on_handle {
        Ok(())
    } else {
        Err(outside(shown))
    }
}

/// Without Unix, for a directory: the handle must be a directory and the path
/// canonicalised again after the check must still lie under the root.
fn verify_portable_directory(
    root: &Path,
    on_handle: &Metadata,
    canonical_after: &Path,
    shown: &str,
) -> Result<(), TreeError> {
    ensure_directory(on_handle, shown)?;
    if canonical_after.starts_with(root) {
        Ok(())
    } else {
        Err(outside(shown))
    }
}

/// Without Unix: the handle must be a regular file, and the path
/// canonicalised again after the open must still lie under the root.
fn verify_portable(
    root: &Path,
    on_handle: &Metadata,
    canonical_after: &Path,
    shown: &str,
) -> Result<(), TreeError> {
    ensure_regular(on_handle, shown)?;
    if canonical_after.starts_with(root) {
        Ok(())
    } else {
        Err(outside(shown))
    }
}

/// Verify that the opened `file` is a regular file inside `root`.
///
/// * Linux: [`verify_linux_link`]. A rename after the open cannot change what
///   the handle is, so no window remains between check and open.
/// * Other Unix: [`verify_same_file`] against a second canonicalisation. A
///   rename between that canonicalisation and the read can still swap the
///   path, but not the handle, whose identity was compared.
/// * Elsewhere: [`verify_portable`]. A rename racing the second
///   canonicalisation is a residual window.
///
/// Every branch compiles on every platform; the platform selects one.
fn verify_open_handle(
    file: &File,
    canonical: &Path,
    root: &Path,
    shown: &str,
) -> Result<(), TreeError> {
    let on_handle = file.metadata().map_err(|e| TreeError::from_io(shown, e))?;
    if cfg!(target_os = "linux") {
        verify_linux_link(root, &descriptor_link(file, shown)?, shown)?;
        ensure_regular(&on_handle, shown)
    } else if cfg!(unix) {
        let after = fs::canonicalize(canonical).map_err(|e| TreeError::from_io(shown, e))?;
        let on_disk = fs::metadata(&after).map_err(|e| TreeError::from_io(shown, e))?;
        verify_same_file(
            root,
            &after,
            file_id(&on_disk, shown)?,
            file_id(&on_handle, shown)?,
            shown,
        )?;
        ensure_regular(&on_handle, shown)
    } else {
        let after = fs::canonicalize(canonical).map_err(|e| TreeError::from_io(shown, e))?;
        verify_portable(root, &on_handle, &after, shown)
    }
}
