// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The handle-contained disk backend. The separate committed Git host backend
//! uses installed Git and its object store rather than this filesystem strategy.
//!
//! A read, a listing and an entry lookup each open a handle (the file to read,
//! the directory to list, or the directory that holds the entry), verify it
//! against the root, use it, and verify it again after the use. How the handle
//! is opened and verified is the platform's [`Strategy`]:
//!
//! * Linux ([`linux`]): the canonical path is taken as an `O_PATH` reference,
//!   which performs no I/O; the kernel's own path for it must lie under the
//!   root before it is opened for reading or listed through that path.
//! * Other Unix systems ([`other_unix`]): the path is walked one component at
//!   a time from a descriptor of the root, with `openat` and without following
//!   any symlink; an in-root symlink is resolved by reading it and continuing
//!   the walk. A walk outside the root may only descend back into it, never
//!   climb, and opens nothing outside but directories, so an outside
//!   non-directory is never opened. No path is resolved by name.
//! * Windows ([`handle_path`]): the canonical path is opened as a handle and
//!   the path the system gives for that handle must lie under the root.
//!
//! On Windows a segment that Windows would not read as the name it spells (a
//! reserved device name, a name that ends in a dot or a space, or one that
//! names an alternate data stream) is refused before anything else. A read
//! takes at most `max_bytes + 1` bytes and refuses the file when the extra byte
//! arrives.
//!
//! A platform with neither Unix nor Windows has no strategy: [`DiskTree::new`]
//! fails there, and the calls below it are never reached.

#![cfg_attr(
    not(any(unix, windows)),
    allow(
        unused_variables,
        dead_code,
        reason = "with no strategy no tree is built, so the calls are compiled but never reached"
    )
)]

use std::fs::Metadata;
use std::io::Read;
use std::path::{Path, PathBuf};

use super::{Entry, EntryKind, FileId, Opened, Tree};
use crate::error::TreeError;
use crate::path::RelPath;

/// Resolve a supplied host root locator; each backend owns its access strategy.
pub(super) fn canonical_root(root: &Path) -> std::io::Result<PathBuf> {
    std::fs::canonicalize(root)
}

/// A callback that a test runs at one point of a call.
#[cfg(test)]
type Hook = Box<dyn Fn() + Send + Sync>;

/// Run the test hook `$hook` of `$tree`; outside test builds, nothing.
macro_rules! hook {
    ($tree:expr, $hook:ident) => {
        #[cfg(test)]
        if let Some(hook) = &$tree.$hook {
            hook();
        }
    };
}

/// Apply `$body` to the strategy's own handle inside a [`Handle`].
macro_rules! each {
    ($handle:expr, $inner:ident => $body:expr) => {
        match *$handle {
            #[cfg(target_os = "linux")]
            Handle::Linux(ref $inner) => $body,
            #[cfg(all(unix, not(target_os = "linux")))]
            Handle::OtherUnix(ref $inner) => $body,
            #[cfg(windows)]
            Handle::Windows(ref $inner) => $body,
        }
    };
}

#[cfg(windows)]
mod final_path;
#[cfg(windows)]
mod handle_path;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(all(unix, not(target_os = "linux")))]
mod other_unix;
// Without a strategy no tree can be built, so there is nothing to test.
#[cfg(all(test, any(unix, windows)))]
mod tests;

/// How a handle is opened and verified against the root.
///
/// The platform selects its native variant in production and test builds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strategy {
    /// An `O_PATH` reference verified through `/proc/self/fd`, then reopened.
    #[cfg(target_os = "linux")]
    Linux,
    /// A walk beneath a descriptor of the root, one `openat` per component.
    #[cfg(all(unix, not(target_os = "linux")))]
    OtherUnix,
    /// A handle opened by path and verified by the path the system gives for
    /// it: `GetFinalPathNameByHandleW` on Windows.
    #[cfg(windows)]
    Windows,
}

impl Strategy {
    /// The strategy of the platform the crate is built for.
    #[cfg(target_os = "linux")]
    #[expect(
        clippy::unnecessary_wraps,
        reason = "the signature is shared with the platforms that have no strategy"
    )]
    fn native(_shown: &str) -> Result<Self, TreeError> {
        Ok(Self::Linux)
    }

    /// The strategy of the platform the crate is built for.
    #[cfg(all(unix, not(target_os = "linux")))]
    #[expect(
        clippy::unnecessary_wraps,
        reason = "the signature is shared with the platforms that have no strategy"
    )]
    fn native(_shown: &str) -> Result<Self, TreeError> {
        Ok(Self::OtherUnix)
    }

    /// The strategy of the platform the crate is built for.
    #[cfg(windows)]
    #[expect(
        clippy::unnecessary_wraps,
        reason = "the signature is shared with the platforms that have no strategy"
    )]
    fn native(_shown: &str) -> Result<Self, TreeError> {
        Ok(Self::Windows)
    }

    /// Without Unix or Windows no handle can be verified, so no tree opens.
    #[cfg(not(any(unix, windows)))]
    fn native(shown: &str) -> Result<Self, TreeError> {
        Err(TreeError::Io {
            path: shown.to_owned(),
            source: std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "this platform cannot verify a handle against the root",
            ),
        })
    }
}

/// What a handle must be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Want {
    /// A regular file, to be read; a final symlink is followed.
    File,
    /// A directory that holds an entry; a final symlink is followed.
    Dir,
    /// A directory to be listed; a final symlink is refused.
    Listing,
}

impl Want {
    /// Refuse `metadata` of a handle that is not what is wanted.
    fn require(self, metadata: &Metadata, shown: &str) -> Result<(), TreeError> {
        let file_type = metadata.file_type();
        let fits = match self {
            Self::File => file_type.is_file(),
            Self::Dir | Self::Listing => file_type.is_dir(),
        };
        if fits {
            Ok(())
        } else {
            Err(not_regular(shown))
        }
    }
}

/// A verified handle, of the strategy that opened it.
enum Handle {
    #[cfg(target_os = "linux")]
    Linux(linux::Handle),
    #[cfg(all(unix, not(target_os = "linux")))]
    OtherUnix(other_unix::Handle),
    #[cfg(windows)]
    Windows(handle_path::Handle),
}

impl Handle {
    /// The identity of the verified file.
    fn identity(&self, root: &Path, shown: &str) -> Result<FileId, TreeError> {
        each!(self, inner => inner.identity(root, shown))
    }

    /// At most `max_bytes` bytes of the verified file.
    fn read(&self, max_bytes: u64, shown: &str) -> Result<Vec<u8>, TreeError> {
        each!(self, inner => inner.read(max_bytes, shown))
    }

    /// The entries of the verified directory, in the order the system gives.
    fn entries(&self, shown: &str) -> Result<Vec<Entry>, TreeError> {
        each!(self, inner => inner.entries(shown))
    }

    /// What `name` is in the verified directory, without following it.
    fn lookup(&self, name: &str, shown: &str) -> Result<Option<EntryKind>, TreeError> {
        each!(self, inner => inner.lookup(name, shown))
    }

    /// Verify the handle again, after a use.
    fn recheck(&self, tree: &DiskTree, shown: &str) -> Result<(), TreeError> {
        match *self {
            #[cfg(target_os = "linux")]
            Handle::Linux(ref inner) => inner.recheck(&tree.root, shown),
            #[cfg(all(unix, not(target_os = "linux")))]
            Handle::OtherUnix(ref inner) => inner.recheck(&tree.walk_root, shown),
            #[cfg(windows)]
            Handle::Windows(ref inner) => inner.recheck(&tree.root, shown),
        }
    }
}

/// True when an open with `flags` returns at once on a FIFO: an `O_PATH`
/// reference performs no I/O, and an `O_NONBLOCK` open does not wait for the
/// other end.
#[cfg(all(test, unix))]
fn never_blocks(flags: u32) -> bool {
    #[cfg(target_os = "linux")]
    let path = libc::O_PATH.cast_unsigned();
    #[cfg(not(target_os = "linux"))]
    let path = 0;
    flags & (path | libc::O_NONBLOCK.cast_unsigned()) != 0
}

/// A tree read from a directory on disk.
///
/// The root is canonicalised once, in [`DiskTree::new`]. A symlink inside the
/// tree is followed only when its target is also inside the tree.
pub struct DiskTree {
    root: PathBuf,
    strategy: Strategy,
    /// A descriptor of the root, from which the other-Unix walk starts.
    #[cfg(all(unix, not(target_os = "linux")))]
    walk_root: other_unix::Root,
    /// Runs after the containment check and before the open.
    #[cfg(test)]
    before_open: Option<Hook>,
    /// Runs after the open and before the handle is verified.
    #[cfg(test)]
    after_open: Option<Hook>,
    /// Runs after the handle is verified and before it is used.
    #[cfg(test)]
    after_verify: Option<Hook>,
    /// Runs inside one step of the other-Unix walk, after an open failed and
    /// before the status of the entry is read.
    #[cfg(all(test, unix, not(target_os = "linux")))]
    mid_step: Option<Hook>,
    /// The flags of every open that resolves a name below the root.
    #[cfg(all(test, unix))]
    opens: std::sync::Mutex<Vec<u32>>,
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
    /// system fails or cannot verify a handle against the root.
    pub fn new(root: impl AsRef<Path>) -> Result<Self, TreeError> {
        let given = root.as_ref();
        let shown = given.display().to_string();
        let root = canonical_root(given).map_err(|e| TreeError::from_io(&shown, e))?;
        let metadata = std::fs::metadata(&root).map_err(|e| TreeError::from_io(&shown, e))?;
        if !metadata.is_dir() {
            return Err(TreeError::NotRegular { path: shown });
        }
        let strategy = Strategy::native(&shown)?;
        Ok(Self {
            #[cfg(all(unix, not(target_os = "linux")))]
            walk_root: other_unix::Root::open(&root, &shown)?,
            root,
            strategy,
            #[cfg(test)]
            before_open: None,
            #[cfg(test)]
            after_open: None,
            #[cfg(test)]
            after_verify: None,
            #[cfg(all(test, unix, not(target_os = "linux")))]
            mid_step: None,
            #[cfg(all(test, unix))]
            opens: std::sync::Mutex::new(Vec::new()),
        })
    }

    /// Read the exact bytes through the same verified handles as [`Tree::read`].
    ///
    /// The file is checked against the root before and after reading. At most
    /// `max_bytes + 1` bytes are read, and the extra byte makes the read refuse.
    /// No UTF-8 validation or text conversion is applied.
    ///
    /// # Errors
    /// The errors of [`Tree::read`], except `NotUtf8` cannot occur. Foreign
    /// authorities, outside paths and non-regular files remain refusals.
    pub fn read_bytes(&self, path: &RelPath, max_bytes: u64) -> Result<Vec<u8>, TreeError> {
        super::require_first_party(path)?;
        let shown = path.to_string();
        let handle = self.open(path, Want::File, &shown)?;
        self.used(&handle, &shown, |handle| handle.read(max_bytes, &shown))
    }

    /// Take the actual file identity and read bytes from that same verified open.
    ///
    /// An identity accepted by `seen` returns [`super::OpenedBytes::Seen`]
    /// without reading its payload or applying the byte ceiling to that payload.
    /// Otherwise the complete bytes obey [`DiskTree::read_bytes`].
    ///
    /// # Errors
    /// As [`DiskTree::read_bytes`], including identity and handle verification failures.
    pub fn read_unseen_bytes(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<super::OpenedBytes, TreeError> {
        super::require_first_party(path)?;
        let shown = path.to_string();
        let handle = self.open(path, Want::File, &shown)?;
        let id = handle.identity(&self.root, &shown)?;
        if seen(&id) {
            return Ok(super::OpenedBytes::Seen(id));
        }
        let bytes = self.used(&handle, &shown, |handle| handle.read(max_bytes, &shown))?;
        Ok(super::OpenedBytes::Read { id, bytes })
    }

    /// Record the flags of an open that is about to resolve a name below the
    /// root, and require that it cannot wait. The check runs before the open,
    /// so a regression that drops the flag fails here and never reaches a
    /// FIFO that would block it.
    #[cfg(all(test, unix))]
    fn note_open(&self, flags: u32) {
        assert!(
            never_blocks(flags),
            "an open of a name below the root could block: flags {flags:#o}"
        );
        self.opens.lock().expect("the open record").push(flags);
    }

    /// The flags of every open recorded so far, oldest first.
    #[cfg(all(test, unix))]
    fn recorded_opens(&self) -> Vec<u32> {
        self.opens.lock().expect("the open record").clone()
    }

    /// The path on disk that `path` names, before any symlink is followed.
    #[cfg(any(target_os = "linux", windows))]
    fn candidate(&self, path: &RelPath) -> PathBuf {
        if path.is_root() {
            self.root.clone()
        } else {
            self.root.join(path.as_str())
        }
    }

    /// Canonicalise `candidate` and require the canonical root as its prefix.
    #[cfg(any(target_os = "linux", windows))]
    fn contain(&self, candidate: &Path, shown: &str) -> Result<PathBuf, TreeError> {
        let canonical =
            std::fs::canonicalize(candidate).map_err(|e| TreeError::from_io(shown, e))?;
        if canonical.starts_with(&self.root) {
            Ok(canonical)
        } else {
            Err(outside(shown))
        }
    }

    /// Open `path` as `want` and verify the handle against the root.
    fn open(&self, path: &RelPath, want: Want, shown: &str) -> Result<Handle, TreeError> {
        refuse_windows_names(path, shown)?;
        match self.strategy {
            #[cfg(target_os = "linux")]
            Strategy::Linux => linux::open(self, path, want, shown).map(Handle::Linux),
            #[cfg(all(unix, not(target_os = "linux")))]
            Strategy::OtherUnix => other_unix::open(self, path, want, shown).map(Handle::OtherUnix),
            #[cfg(windows)]
            Strategy::Windows => handle_path::open(self, path, want, shown).map(Handle::Windows),
        }
    }

    /// Use the verified `handle` with `work`, then verify it again; a handle
    /// that no longer passes discards what the use returned.
    fn used<T>(
        &self,
        handle: &Handle,
        shown: &str,
        work: impl FnOnce(&Handle) -> Result<T, TreeError>,
    ) -> Result<T, TreeError> {
        hook!(self, after_verify);
        let result = work(handle);
        handle.recheck(self, shown)?;
        result
    }
}

impl Tree for DiskTree {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        text(self.read_bytes(path, max_bytes)?, path)
    }

    /// The identity is, on Unix, the device and inode of the verified handle,
    /// and on Windows the path the system gives for it, below the root; the
    /// text, when it is read, comes from the same handle.
    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        match self.read_unseen_bytes(path, max_bytes, seen)? {
            super::OpenedBytes::Seen(id) => Ok(Opened::Seen(id)),
            super::OpenedBytes::Read { id, bytes } => Ok(Opened::Read {
                id,
                text: text(bytes, path)?,
            }),
        }
    }

    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        super::require_first_party(path)?;
        let shown = path.to_string();
        let handle = self.open(path, Want::Listing, &shown)?;
        let mut entries = self.used(&handle, &shown, |handle| handle.entries(&shown))?;
        entries.sort();
        Ok(entries)
    }

    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        super::require_first_party(path)?;
        let shown = path.to_string();
        refuse_windows_names(path, &shown)?;
        let (Some(name), Some(parent)) = (path.file_name(), path.parent()) else {
            return Ok(Some(EntryKind::Dir));
        };
        // `x/..` is the directory the path itself leads to, which may lie
        // outside the root even when `x` does not, so it is opened whole.
        let climbs = name == "..";
        let directory = if climbs { path } else { &parent };
        let handle = match self.open(directory, Want::Dir, &shown) {
            Ok(handle) => handle,
            Err(TreeError::NotFound { .. } | TreeError::NotRegular { .. }) => return Ok(None),
            Err(other) => return Err(other),
        };
        self.used(&handle, &shown, |handle| {
            if climbs {
                Ok(Some(EntryKind::Dir))
            } else {
                handle.lookup(name, &shown)
            }
        })
    }
}

/// What a file type from the standard library is.
#[cfg(any(target_os = "linux", windows))]
fn kind_of(file_type: std::fs::FileType) -> EntryKind {
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

/// What is at `path`, without following a final symlink.
#[cfg(any(target_os = "linux", windows))]
fn lookup(path: &Path, shown: &str) -> Result<Option<EntryKind>, TreeError> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(kind_of(metadata.file_type()))),
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
            ) =>
        {
            Ok(None)
        }
        Err(e) => Err(TreeError::from_io(shown, e)),
    }
}

/// The entries of the directory at `listing`, in the order the system gives.
#[cfg(any(target_os = "linux", windows))]
fn read_entries(listing: &Path, shown: &str) -> Result<Vec<Entry>, TreeError> {
    let mut entries = Vec::new();
    for item in std::fs::read_dir(listing).map_err(|e| TreeError::from_io(shown, e))? {
        let item = item.map_err(|e| TreeError::from_io(shown, e))?;
        let name = item
            .file_name()
            .into_string()
            .map_err(|raw| TreeError::NonUtf8Name {
                path: shown.to_owned(),
                name: raw.to_string_lossy().into_owned(),
            })?;
        let file_type = item.file_type().map_err(|e| TreeError::from_io(shown, e))?;
        entries.push(Entry {
            name,
            kind: kind_of(file_type),
        });
    }
    Ok(entries)
}

/// A listing refuses a path whose final component is not a real directory,
/// a symlink included, judged without following it.
#[cfg(any(target_os = "linux", windows))]
fn refuse_unless_directory(candidate: &Path, shown: &str) -> Result<(), TreeError> {
    let own = std::fs::symlink_metadata(candidate).map_err(|e| TreeError::from_io(shown, e))?;
    if own.is_dir() {
        Ok(())
    } else {
        Err(not_regular(shown))
    }
}

/// Classify a failure to open: a symlink in the final position (the canonical
/// path was swapped for a link after it was checked) is not a regular file.
#[cfg(any(target_os = "linux", windows))]
fn open_error(shown: &str, error: std::io::Error) -> TreeError {
    if crate::error::is_symlink_loop(&error) {
        not_regular(shown)
    } else {
        TreeError::from_io(shown, error)
    }
}

/// A failure to learn the path of a handle. A missing descriptor link (no
/// `/proc`) is an operating-system failure, never absence of the file.
#[cfg(any(target_os = "linux", windows))]
fn link_read_error(shown: &str, source: std::io::Error) -> TreeError {
    TreeError::Io {
        path: shown.to_owned(),
        source,
    }
}

/// The path the system gives for an open handle must lie under the root.
#[cfg(any(target_os = "linux", windows))]
fn verify_handle_path(root: &Path, handle_path: &Path, shown: &str) -> Result<(), TreeError> {
    if handle_path.starts_with(root) {
        Ok(())
    } else {
        Err(outside(shown))
    }
}

/// Linux: the kernel's own path for an open handle must lie under the root.
///
/// `handle_link` is `/proc/self/fd/<fd>`. This reads what the handle really
/// is, so no rename of a path can change the answer.
#[cfg(target_os = "linux")]
fn verify_linux_link(root: &Path, handle_link: &Path, shown: &str) -> Result<(), TreeError> {
    let target = std::fs::read_link(handle_link).map_err(|e| link_read_error(shown, e))?;
    verify_handle_path(root, &target, shown)
}

/// `/proc/self/fd/<fd>`: the kernel's own path for an open handle.
#[cfg(target_os = "linux")]
fn descriptor_path(handle: &std::fs::File) -> PathBuf {
    use std::os::fd::AsRawFd as _;
    PathBuf::from(format!("/proc/self/fd/{}", handle.as_raw_fd()))
}

/// Read at most `max_bytes` raw bytes from a verified handle.
fn read_capped(source: impl Read, max_bytes: u64, shown: &str) -> Result<Vec<u8>, TreeError> {
    let mut bytes = Vec::new();
    source
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|e| TreeError::from_io(shown, e))?;
    if u64::try_from(bytes.len()).map_or(true, |len| len > max_bytes) {
        return Err(TreeError::TooLarge {
            path: shown.to_owned(),
            limit: max_bytes,
        });
    }
    Ok(bytes)
}

fn text(bytes: Vec<u8>, path: &RelPath) -> Result<String, TreeError> {
    String::from_utf8(bytes).map_err(|_| TreeError::NotUtf8 {
        path: path.to_string(),
    })
}

fn outside(shown: &str) -> TreeError {
    TreeError::OutsideRoot {
        path: shown.to_owned(),
    }
}

fn not_regular(shown: &str) -> TreeError {
    TreeError::NotRegular {
        path: shown.to_owned(),
    }
}

/// Unix: the identity of a file is the device and inode of its handle.
#[cfg(unix)]
fn device_identity(metadata: &Metadata) -> FileId {
    use std::os::unix::fs::MetadataExt as _;
    FileId::from_device(metadata.dev(), metadata.ino())
}

/// Windows: refuse a path with a segment that Windows would not read as the
/// name it spells. Such a path is never canonicalised.
#[cfg(windows)]
fn refuse_windows_names(path: &RelPath, shown: &str) -> Result<(), TreeError> {
    if path.segments().any(is_ambiguous_on_windows) {
        Err(outside(shown))
    } else {
        Ok(())
    }
}

/// Elsewhere every segment names itself.
#[cfg(not(windows))]
#[expect(
    clippy::unnecessary_wraps,
    reason = "the signature is shared with Windows, where the call can refuse"
)]
fn refuse_windows_names(_path: &RelPath, _shown: &str) -> Result<(), TreeError> {
    Ok(())
}

/// True for a segment that is a reserved device name, bare or with any
/// extension and in any case; for one that ends in a dot or a space, which
/// Windows strips; and for one that holds a colon, which names an alternate
/// data stream. A `..` segment climbs and names no file.
#[cfg(windows)]
fn is_ambiguous_on_windows(segment: &str) -> bool {
    if segment == ".." {
        return false;
    }
    if segment.ends_with(['.', ' ']) || segment.contains(':') {
        return true;
    }
    let stem = segment
        .split('.')
        .next()
        .unwrap_or(segment)
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    let numbered = |prefix: &str| {
        stem.strip_prefix(prefix).is_some_and(|digit| {
            matches!(
                digit,
                "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    };
    matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) || numbered("COM")
        || numbered("LPT")
}
