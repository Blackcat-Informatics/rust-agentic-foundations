// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native read-only Git access. No API starts a process or writes repository state.

use crate::{DiskTree, RelPath, TreeError};
use purrdf_lex::allocation::{Admission, Memory, Resident, StorageError};
use std::path::{Path, PathBuf};

/// A refused native repository operation, retaining the original cause.
#[derive(Debug)]
pub struct Error {
    operation: &'static str,
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl Error {
    pub(crate) fn storage(&self) -> Option<StorageError> {
        self.source.downcast_ref::<StorageError>().copied()
    }
    fn native(
        operation: &'static str,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            operation,
            source: Box::new(source),
        }
    }

    fn invalid(operation: &'static str, message: &'static str) -> Self {
        Self::native(
            operation,
            std::io::Error::new(std::io::ErrorKind::InvalidData, message),
        )
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native Git {}: {}", self.operation, self.source)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

/// A committed entry's exact mode, without executing filters or following links.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// A directory tree.
    Tree,
    /// A regular file, executable or not.
    Blob,
    /// A symbolic-link blob.
    Symlink,
    /// A submodule commit locator; never opened automatically.
    Submodule,
}

impl EntryKind {
    fn from_mode(mode: u16) -> Result<Self, Error> {
        match mode {
            0o040_000 => Ok(Self::Tree),
            0o100_644 | 0o100_755 => Ok(Self::Blob),
            0o120_000 => Ok(Self::Symlink),
            0o160_000 => Ok(Self::Submodule),
            _ => Err(Error::invalid(
                "tree mode",
                "unsupported committed entry mode",
            )),
        }
    }
}

/// A complete committed entry with its native object identity and size.
#[derive(Debug, Clone)]
pub struct Entry {
    /// Canonical UTF-8 repository-relative path.
    pub path: String,
    /// The exact supported mode class.
    pub kind: EntryKind,
    /// Full lowercase native object identifier.
    pub id: String,
    /// Uncompressed blob size, zero for directories and submodule locators.
    pub size: usize,
}

/// An admitted commit's immutable metadata.
#[derive(Debug, Clone)]
pub struct Commit {
    /// Full object identifier.
    pub id: String,
    /// Full root tree identifier.
    pub tree: String,
    /// Ordered full parent identifiers.
    pub parents: Vec<String>,
    /// Author time in strict ISO 8601, retaining the written zone.
    pub author_date: String,
    /// Committer timestamp for deterministic revision ordering.
    pub commit_seconds: i64,
    /// First message paragraph, with physical line breaks replaced by spaces.
    pub subject: String,
}

/// An isolated repository handle. Only read operations are exposed.
///
/// Configuration is strict and repository-local. RAF enables only native
/// index, revision and hash features; its read paths never invoke helpers,
/// transports, hooks or filters. Missing objects fail locally.
/// Native decoding internals are outside Source's portable meters;
/// callers retain explicit bounds on decoded objects and acquired results.
#[derive(Debug)]
pub struct Repository {
    inner: gix::Repository,
    directory: PathBuf,
    selected_index: Option<PathBuf>,
}

const OBJECT_BOUND: usize = 16 * 1024 * 1024;

impl Repository {
    /// Discover a repository above `directory`, without environment selectors.
    ///
    /// # Errors
    /// Fails on inaccessible paths, malformed repositories or configuration.
    /// `None` means discovery reached a boundary without finding a repository.
    pub fn discover(directory: impl AsRef<Path>) -> Result<Option<Self>, Error> {
        Self::discover_bounded(directory, OBJECT_BOUND)
    }

    fn discover_bounded(
        directory: impl AsRef<Path>,
        allocation_limit: usize,
    ) -> Result<Option<Self>, Error> {
        if allocation_limit == 0 || i64::try_from(allocation_limit).is_err() {
            return Err(Error::invalid(
                "allocation limit",
                "native per-allocation grant must be nonzero and fit the configuration integer",
            ));
        }
        let directory = directory
            .as_ref()
            .canonicalize()
            .map_err(|e| Error::native("directory", e))?;
        let options = gix::open::Options::isolated()
            .strict_config(true)
            .config_overrides([
                "core.useReplaceRefs=false".to_owned(),
                format!("gitoxide.objects.allocLimit={allocation_limit}"),
            ]);
        match gix::discover_opts(
            &directory,
            gix::discover::upwards::Options::default(),
            options,
        ) {
            Ok(inner) => Ok(Some(Self {
                inner,
                directory,
                selected_index: None,
            })),
            Err(error)
                if error
                    .downcast_any_ref::<gix::discover::upwards::Error>()
                    .is_some_and(|e| {
                        matches!(
                            e,
                            gix::discover::upwards::Error::NoGitRepository { .. }
                                | gix::discover::upwards::Error::NoGitRepositoryWithinCeiling { .. }
                                | gix::discover::upwards::Error::NoGitRepositoryWithinFs { .. }
                        )
                    }) =>
            {
                Ok(None)
            }
            Err(error) => Err(Error::native("discover", error)),
        }
    }

    /// Open the repository containing the exact selected directory.
    ///
    /// # Errors
    /// Fails when discovery has no repository or encounters invalid data.
    pub fn open(directory: impl AsRef<Path>) -> Result<Self, Error> {
        Self::discover(directory)?
            .ok_or_else(|| Error::invalid("open", "directory is outside a Git repository"))
    }

    /// Open with an explicit native decoder per-allocation grant.
    ///
    /// This bounds loose/packed inflation and index-derived allocations, not
    /// aggregate model memory or traversal work. Source's portable meters still
    /// govern acquired buffers and model processing separately.
    ///
    /// # Errors
    /// Refuses zero/unrepresentable grants and all failures of [`Self::open`].
    pub fn open_with_allocation_limit(
        directory: impl AsRef<Path>,
        allocation_limit: usize,
    ) -> Result<Self, Error> {
        Self::discover_bounded(directory, allocation_limit)?
            .ok_or_else(|| Error::invalid("open", "directory is outside a Git repository"))
    }

    /// Open with one explicitly selected index; no process environment is changed.
    ///
    /// # Errors
    /// As [`Self::open`]. Relative indexes are anchored at the selected directory.
    pub fn open_with_index(
        directory: impl AsRef<Path>,
        index: Option<&Path>,
    ) -> Result<Self, Error> {
        let mut repo = Self::open(directory)?;
        repo.selected_index = index.map(|path| {
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                repo.directory.join(path)
            }
        });
        Ok(repo)
    }

    /// The originally selected canonical directory.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// The selected checkout's private Git directory.
    #[must_use]
    pub fn git_dir(&self) -> &Path {
        self.inner.git_dir()
    }

    /// The shared Git directory, including for linked checkouts.
    #[must_use]
    pub fn common_dir(&self) -> &Path {
        self.inner.common_dir()
    }

    /// The checkout root, absent for bare repositories.
    #[must_use]
    pub fn work_dir(&self) -> Option<&Path> {
        self.inner.workdir()
    }

    /// The explicitly selected or ordinary private index path.
    #[must_use]
    pub fn index_path(&self) -> PathBuf {
        self.selected_index
            .clone()
            .unwrap_or_else(|| self.git_dir().join("index"))
    }

    fn entry_size(&self, id: gix::ObjectId, kind: EntryKind) -> Result<usize, Error> {
        if !matches!(kind, EntryKind::Blob | EntryKind::Symlink) {
            return Ok(0);
        }
        let header = self
            .inner
            .find_header(id)
            .map_err(|error| Error::native("blob header", error))?;
        if header.kind() != gix::objs::Kind::Blob {
            return Err(Error::invalid("blob", "blob mode names a non-blob object"));
        }
        usize::try_from(header.size()).map_err(|error| Error::native("blob size", error))
    }

    fn id(&self, text: &str) -> Result<gix::ObjectId, Error> {
        let id =
            gix::ObjectId::from_hex(text.as_bytes()).map_err(|e| Error::native("object ID", e))?;
        if id.kind() != self.inner.object_hash() {
            return Err(Error::invalid(
                "object ID",
                "identifier hash format differs from repository",
            ));
        }
        Ok(id)
    }

    fn object(&self, id: gix::ObjectId, max: usize) -> Result<gix::Object<'_>, Error> {
        let header = self
            .inner
            .find_header(id)
            .map_err(|e| Error::native("object header", e))?;
        if header.size() > max as u64 {
            return Err(Error::invalid(
                "object size",
                "decoded object exceeds the selected byte bound",
            ));
        }
        let object = self
            .inner
            .find_object(id)
            .map_err(|e| Error::native("object", e))?;
        if object.data.len() > max || object.data.len() as u64 != header.size() {
            return Err(Error::invalid(
                "object size",
                "decoded object differs from admitted size",
            ));
        }
        let actual = gix::objs::compute_hash(id.kind(), object.kind, &object.data)
            .map_err(|e| Error::native("object hash", e))?;
        if actual != id {
            return Err(Error::invalid(
                "object hash",
                "object bytes do not match their identity",
            ));
        }
        Ok(object)
    }

    /// Resolve HEAD or a full reference to an existing commit.
    ///
    /// # Errors
    /// Refuses missing, malformed or non-commit objects and unsupported selectors.
    pub fn resolve_commit(&self, reference: &str) -> Result<String, Error> {
        let id = if reference == "HEAD" {
            self.inner
                .head_id()
                .map_err(|e| Error::native("HEAD", e))?
                .detach()
        } else if reference.starts_with("refs/") {
            self.inner
                .find_reference(reference)
                .map_err(|e| Error::native("reference", e))?
                .peel_to_id()
                .map_err(|e| Error::native("reference target", e))?
                .detach()
        } else {
            self.id(reference)?
        };
        let object = self.object(id, OBJECT_BOUND)?;
        if object.kind != gix::objs::Kind::Commit {
            return Err(Error::invalid("commit", "selected object is not a commit"));
        }
        Ok(id.to_string())
    }

    /// Return origin's symbolic default reference, without guessing a branch.
    ///
    /// # Errors
    /// Refuses absent, direct or malformed origin HEAD references.
    pub fn default_branch_ref(&self) -> Result<String, Error> {
        let reference = self
            .inner
            .find_reference("refs/remotes/origin/HEAD")
            .map_err(|e| Error::native("origin HEAD", e))?;
        match reference.target() {
            gix::refs::TargetRef::Symbolic(name) => Ok(name.as_bstr().to_string()),
            gix::refs::TargetRef::Object(_) => Err(Error::invalid(
                "origin HEAD",
                "origin HEAD must be a symbolic reference",
            )),
        }
    }

    /// Find a best common ancestor, or `None` for unrelated histories.
    ///
    /// # Errors
    /// Refuses invalid identifiers, missing objects and traversal failures.
    pub fn merge_base(&self, one: &str, two: &str) -> Result<Option<String>, Error> {
        self.inner
            .merge_base(self.id(one)?, self.id(two)?)
            .map(|id| id.map(|id| id.to_string()))
            .map_err(|e| Error::native("merge base", e))
    }

    /// Read existing index paths below the selected directory.
    ///
    /// # Errors
    /// Refuses oversized/malformed indexes, invalid UTF-8 and noncanonical paths.
    /// Metadata inspection uses a verified disk handle rooted at the index parent;
    /// final symlinks outside that root refuse. Native decoding reopens by name,
    /// so this is not an atomic observation against a concurrent directory swap.
    pub fn tracked_paths(&self, max: usize) -> Result<Option<Vec<String>>, Error> {
        if self.directory.starts_with(self.git_dir()) {
            return Ok(None);
        }
        let Some(root) = self.work_dir() else {
            return Ok(None);
        };
        let Ok(prefix) = self.directory.strip_prefix(root) else {
            return Ok(None);
        };
        let index_path = self.index_path();
        let parent = index_path
            .parent()
            .ok_or_else(|| Error::invalid("index", "index must have a parent directory"))?;
        let name = index_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| Error::invalid("index", "index filename must be UTF-8"))?;
        let path = RelPath::new(name).map_err(|e| Error::native("index path", e))?;
        let disk = DiskTree::new(parent).map_err(|e| Error::native("index directory", e))?;
        match disk.byte_len(&path) {
            Ok(bytes) if bytes > max as u64 => {
                return Err(Error::invalid(
                    "index",
                    "index exceeds the selected byte bound",
                ));
            }
            Ok(_) => {}
            Err(TreeError::NotFound { .. }) if self.selected_index.is_none() => {
                return Ok(Some(Vec::new()));
            }
            Err(e) => return Err(Error::native("index metadata", e)),
        }
        let index = gix::index::File::at(
            index_path,
            self.inner.object_hash(),
            false,
            gix::index::decode::Options {
                alloc_limit_bytes: Some(max),
                ..Default::default()
            },
        )
        .map_err(|e| Error::native("index", e))?;
        let mut paths = Vec::new();
        let mut acquired = 0usize;
        for entry in index.entries() {
            let name = std::str::from_utf8(entry.path(&index))
                .map_err(|e| Error::native("index path", e))?;
            canonical_path(name)?;
            let Ok(relative) = Path::new(name).strip_prefix(prefix) else {
                continue;
            };
            let relative = relative
                .to_str()
                .ok_or_else(|| Error::invalid("index path", "non-UTF-8 relative path"))?;
            acquired = acquired
                .checked_add(relative.len() + 1)
                .filter(|n| *n <= max)
                .ok_or_else(|| {
                    Error::invalid("index paths", "paths exceed the selected byte bound")
                })?;
            if paths.last().is_none_or(|last| last != relative) {
                paths.push(relative.to_owned());
            }
        }
        Ok(Some(paths))
    }

    /// Read bounded immutable commit metadata.
    /// Messages must be UTF-8; absent encoding headers and the UTF-8/UTF8
    /// encoding names are supported without transcoding.
    ///
    /// # Errors
    /// Refuses missing, corrupt, oversized or incorrectly typed objects.
    pub fn commit(&self, id: &str) -> Result<Commit, Error> {
        let id = self.id(id)?;
        let object = self.object(id, OBJECT_BOUND)?;
        if object.kind != gix::objs::Kind::Commit {
            return Err(Error::invalid("commit", "object is not a commit"));
        }
        let decoded = gix::objs::CommitRef::from_bytes(&object.data, id.kind())
            .map_err(|e| Error::native("commit decoding", e))?;
        let author = decoded
            .author()
            .map_err(|e| Error::native("author", e))?
            .time()
            .map_err(|e| Error::native("author date", e))?;
        let committer = decoded
            .committer()
            .map_err(|e| Error::native("committer", e))?
            .time()
            .map_err(|e| Error::native("committer date", e))?;
        let author_date = author
            .format(gix::date::time::format::ISO8601_STRICT)
            .map_err(|e| Error::native("author date", e))?;
        if decoded.encoding.is_some_and(|encoding| {
            !encoding.eq_ignore_ascii_case(b"UTF-8") && !encoding.eq_ignore_ascii_case(b"UTF8")
        }) {
            return Err(Error::invalid(
                "commit encoding",
                "only UTF-8 commit message encodings are supported",
            ));
        }
        let message = std::str::from_utf8(decoded.message)
            .map_err(|error| Error::native("commit message UTF-8", error))?;
        let subject = message
            .split("\n\n")
            .next()
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .collect::<Vec<_>>()
            .join(" ");
        Ok(Commit {
            id: id.to_string(),
            tree: decoded.tree().to_string(),
            parents: decoded.parents().map(|p| p.to_string()).collect(),
            author_date,
            commit_seconds: committer.seconds,
            subject,
        })
    }

    /// Whether the repository declares a shallow history boundary.
    ///
    /// # Errors
    /// Refuses unreadable or malformed shallow metadata.
    pub fn is_shallow(&self) -> Result<bool, Error> {
        self.inner
            .is_shallow()
            .map_err(|e| Error::native("shallow metadata", e))
    }

    /// Whether repository-local history display uses selected-directory paths.
    ///
    /// # Errors
    /// Refuses a malformed boolean rather than selecting a fallback format.
    pub fn relative_history_paths(&self) -> Result<bool, Error> {
        self.inner
            .config_snapshot()
            .try_boolean("diff.relative")
            .map(|value| value.unwrap_or(false))
            .map_err(|error| Error::native("relative history paths", error))
    }

    /// List all committed directories and leaves under an exact commit.
    ///
    /// # Errors
    /// Refuses malformed names/modes, missing objects and cumulative byte exhaustion.
    pub fn tree(&self, commit: &str, max: usize) -> Result<Vec<Entry>, Error> {
        self.tree_admitted(commit, max, &mut Memory::new(&mut Resident))
    }

    pub(crate) fn tree_admitted<S: Admission + ?Sized>(
        &self,
        commit: &str,
        max: usize,
        memory: &mut Memory<'_, S>,
    ) -> Result<Vec<Entry>, Error> {
        let commit = self.commit(commit)?;
        let mut pending = Vec::new();
        memory
            .push(&mut pending, (self.id(&commit.tree)?, String::new()))
            .map_err(|e| Error::native("tree storage", e))?;
        let mut entries = Vec::new();
        let mut acquired = 0usize;
        while let Some((id, prefix)) = pending.pop() {
            let object = self.object(id, max)?;
            if object.kind != gix::objs::Kind::Tree {
                return Err(Error::invalid("tree", "directory object is not a tree"));
            }
            for entry in gix::objs::TreeRefIter::from_bytes(&object.data, id.kind()) {
                let entry = entry.map_err(|e| Error::native("tree decoding", e))?;
                let name = std::str::from_utf8(entry.filename)
                    .map_err(|e| Error::native("tree path", e))?;
                if name.contains('/') {
                    return Err(Error::invalid(
                        "tree path",
                        "tree entry name contains a slash",
                    ));
                }
                canonical_path(name)?;
                let length = prefix
                    .len()
                    .saturating_add(name.len())
                    .saturating_add(usize::from(!prefix.is_empty()));
                memory
                    .admission_mut()
                    .checkpoint(
                        u64::try_from(length.saturating_add(64))
                            .map_err(|e| Error::native("tree work", e))?,
                    )
                    .map_err(|e| Error::native("tree work", e))?;
                acquired = acquired
                    .checked_add(length + size_of::<Entry>() + 64)
                    .filter(|n| *n <= max)
                    .ok_or_else(|| {
                        Error::invalid("tree", "complete tree listing exceeds selected byte bound")
                    })?;
                let path = if prefix.is_empty() {
                    memory.string(name)
                } else {
                    memory.format(&format_args!("{prefix}/{name}"))
                }
                .map_err(|e| Error::native("tree storage", e))?;
                let kind = EntryKind::from_mode(entry.mode.value())?;
                let size = self.entry_size(entry.oid.to_owned(), kind)?;
                if kind == EntryKind::Tree {
                    let copy = memory
                        .string(&path)
                        .map_err(|e| Error::native("tree storage", e))?;
                    memory
                        .push(&mut pending, (entry.oid.to_owned(), copy))
                        .map_err(|e| Error::native("tree storage", e))?;
                }
                let id = memory
                    .format(&format_args!("{}", entry.oid))
                    .map_err(|e| Error::native("tree storage", e))?;
                memory
                    .push(
                        &mut entries,
                        Entry {
                            path,
                            kind,
                            id,
                            size,
                        },
                    )
                    .map_err(|e| Error::native("tree storage", e))?;
            }
            memory
                .release_string(prefix)
                .map_err(|e| Error::native("tree storage", e))?;
        }
        memory
            .release_vec(pending)
            .map_err(|e| Error::native("tree storage", e))?;
        purrdf_lex::walk::try_sort_unstable_by(&mut entries, |a, b| {
            memory
                .admission_mut()
                .checkpoint(
                    u64::try_from(a.path.len().saturating_add(b.path.len()).saturating_add(1))
                        .map_err(|e| Error::native("tree ordering work", e))?,
                )
                .map_err(|e| Error::native("tree ordering work", e))?;
            Ok::<_, Error>(a.path.cmp(&b.path))
        })?;
        if entries.windows(2).any(|p| p[0].path == p[1].path) {
            return Err(Error::invalid("tree", "tree repeats a path"));
        }
        Ok(entries)
    }

    /// Read an admitted blob by exact identifier, without filters or checkout access.
    ///
    /// # Errors
    /// Refuses absent, corrupt, oversized or non-blob objects.
    pub fn blob(&self, id: &str, max: usize) -> Result<Vec<u8>, Error> {
        self.blob_admitted(id, max, &mut Memory::new(&mut Resident))
    }

    pub(crate) fn blob_admitted<S: Admission + ?Sized>(
        &self,
        id: &str,
        max: usize,
        memory: &mut Memory<'_, S>,
    ) -> Result<Vec<u8>, Error> {
        let object = self.object(self.id(id)?, max)?;
        if object.kind != gix::objs::Kind::Blob {
            return Err(Error::invalid("blob", "selected object is not a blob"));
        }
        let mut output = Vec::new();
        memory
            .reserve(&mut output, object.data.len())
            .map_err(|e| Error::native("blob storage", e))?;
        output.extend_from_slice(&object.data);
        Ok(output)
    }

    /// Look up a path in an exact committed tree without following symbolic links.
    ///
    /// # Errors
    /// Refuses invalid paths, corrupt/missing objects and intermediate non-directories.
    pub fn entry_at(&self, commit: &str, path: &str) -> Result<Option<Entry>, Error> {
        canonical_path(path)?;
        let mut id = self.id(&self.commit(commit)?.tree)?;
        let mut components = path.split('/').peekable();
        while let Some(component) = components.next() {
            let object = self.object(id, OBJECT_BOUND)?;
            if object.kind != gix::objs::Kind::Tree {
                return Err(Error::invalid("path", "intermediate object is not a tree"));
            }
            let mut found = None;
            let mut names = std::collections::BTreeSet::new();
            for entry in gix::objs::TreeRefIter::from_bytes(&object.data, id.kind()) {
                let entry = entry.map_err(|e| Error::native("tree decoding", e))?;
                let name = std::str::from_utf8(entry.filename)
                    .map_err(|e| Error::native("tree path", e))?;
                canonical_path(name)?;
                if name.contains('/') || !names.insert(name) {
                    return Err(Error::invalid(
                        "tree",
                        "invalid or duplicate tree entry name",
                    ));
                }
                if name == component {
                    found = Some((entry.mode.value(), entry.oid.to_owned()));
                }
            }
            let Some((mode, next)) = found else {
                return Ok(None);
            };
            let kind = EntryKind::from_mode(mode)?;
            if components.peek().is_some() {
                if kind != EntryKind::Tree {
                    return Err(Error::invalid(
                        "path",
                        "intermediate path is not a directory",
                    ));
                }
                id = next;
            } else {
                let size = self.entry_size(next, kind)?;
                return Ok(Some(Entry {
                    path: path.to_owned(),
                    kind,
                    id: next.to_string(),
                    size,
                }));
            }
        }
        Err(Error::invalid("path", "empty path"))
    }

    /// Walk HEAD ancestors in newest-committer-first order with bounded metadata.
    ///
    /// # Errors
    /// Refuses missing/corrupt commits or a traversal exceeding `max` bytes.
    pub fn ancestors(&self, max: usize) -> Result<Vec<Commit>, Error> {
        let head = self.commit(&self.resolve_commit("HEAD")?)?;
        let mut pending = std::collections::BinaryHeap::new();
        pending.push((head.commit_seconds, head.id.clone()));
        let mut seen = std::collections::BTreeSet::new();
        let mut output = Vec::new();
        let mut acquired = 0usize;
        while let Some((_, id)) = pending.pop() {
            if !seen.insert(id.clone()) {
                continue;
            }
            let commit = self.commit(&id)?;
            acquired = acquired
                .checked_add(commit.subject.len() + commit.parents.len() * 64 + 256)
                .filter(|n| *n <= max)
                .ok_or_else(|| {
                    Error::invalid("history", "commit traversal exceeds selected byte bound")
                })?;
            for parent in &commit.parents {
                if !seen.contains(parent) {
                    let metadata = self.commit(parent)?;
                    pending.push((metadata.commit_seconds, metadata.id));
                }
            }
            output.push(commit);
        }
        Ok(output)
    }
}

fn canonical_path(path: &str) -> Result<(), Error> {
    if path.is_empty()
        || path
            .split('/')
            .any(|p| p.is_empty() || matches!(p, "." | "..") || p.contains(['\\', '\0']))
    {
        return Err(Error::invalid(
            "path",
            "Git paths must have nonempty canonical components",
        ));
    }
    Ok(())
}
