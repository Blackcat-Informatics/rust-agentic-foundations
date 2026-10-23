// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! An opaque immutable snapshot checked against an exact locked Git commit.

use std::fmt;

use gix_object::{CommitRefIter, TreeRefIter};
use purrdf_hash::{hex, sha1::Sha1};
use purrdf_lex::allocation::Memory;
use purrdf_lex::walk::{WorkList, try_equal_range_by, try_sort_unstable_by};

use super::{Entry, EntryKind, FileId, Opened, Tree, require_first_party};
use crate::meter::{ItemAdmission, Meter, Usage};
use crate::{
    LoadError, LoadErrorKind, LoadOptions, LockedCatalog, LockedPackageId, LockedSource,
    LockedSourceKind, Origin, RelPath, TreeError,
};

/// A requested SHA-1 Git object identity, not a claim of verified contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GitObjectId([u8; 20]);

impl GitObjectId {
    /// Construct a requested identity; the snapshot checks returned object bytes.
    #[must_use]
    pub const fn from_sha1(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// The complete binary SHA-1 identity.
    #[must_use]
    pub const fn sha1(&self) -> &[u8; 20] {
        &self.0
    }
}

impl fmt::Display for GitObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        hex::Lower(&self.0).fmt(f)
    }
}

/// The Git object framing returned by an object supplier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitObjectKind {
    /// A commit object.
    Commit,
    /// A directory tree object.
    Tree,
    /// A file or symbolic-link blob.
    Blob,
    /// An annotated tag, not an admitted commit snapshot root.
    Tag,
}

impl GitObjectKind {
    const fn name(self) -> &'static str {
        match self {
            Self::Commit => "commit",
            Self::Tree => "tree",
            Self::Blob => "blob",
            Self::Tag => "tag",
        }
    }
}

/// Borrowed raw object evidence; construction does not verify its identity.
#[derive(Debug, Clone, Copy)]
pub struct GitObject<'a> {
    /// The actual object kind used in Git's hashed framing.
    pub kind: GitObjectKind,
    /// Exact decoded binary body, without a loose-object header or compression.
    pub body: &'a [u8],
}

/// Bounded object acquisition, kept separate from portable semantic verification.
///
/// A supplier must refuse before acquiring a body larger than `max_bytes`.
/// The verifier also checks the returned length before hashing or decoding.
/// Already owned supplier buffers are not allocations admitted by this library.
/// Missing objects are failures, even when a semantic path might otherwise exist.
pub trait GitObjects {
    /// Supply exact raw bytes for the requested object, without automatic fetch.
    ///
    /// # Errors
    /// Reports missing, oversized or failed acquisition. No error means absence
    /// of a source path; the snapshot preserves it as an object-evidence failure.
    fn read_object(&self, id: GitObjectId, max_bytes: u64) -> Result<GitObject<'_>, TreeError>;
}

/// Immutable Git contents bound to one complete locked SHA-1 commit.
///
/// Only the checking constructor can create this type. Every exposed tree/blob
/// is checked against authenticated parent references; public authority/FileId
/// constructors cannot manufacture this admission. Directory absence comes from
/// complete checked tree objects, not a mutable checkout listing. This does not
/// prove repository authorship, active configuration, or a loaded `PackageId`.
/// SHA-256 Git sources remain explicitly unsupported by this native hash profile.
pub struct VerifiedGitTree<'a> {
    objects: &'a dyn GitObjects,
    source: &'a LockedSource,
    commit: GitObjectId,
    root: GitObjectId,
    meter: GitMeter<'a>,
}

enum GitMeter<'a> {
    Owned(Meter),
    Borrowed(&'a Meter),
}

impl std::ops::Deref for GitMeter<'_> {
    type Target = Meter;
    fn deref(&self) -> &Meter {
        match self {
            Self::Owned(meter) => meter,
            Self::Borrowed(meter) => meter,
        }
    }
}

impl fmt::Debug for VerifiedGitTree<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VerifiedGitTree")
            .field("source", &self.source)
            .field("commit", &self.commit)
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

struct CheckedEntry {
    name: String,
    kind: EntryKind,
    id: GitObjectId,
}

impl<'a> VerifiedGitTree<'a> {
    /// Admit a catalog row's actual commit and root tree under the supplied bounds.
    ///
    /// # Errors
    /// Refuses unknown/non-Git rows, unsupported object formats, missing objects,
    /// wrong kind/hash, malformed objects and original resource failures.
    pub fn new(
        catalog: &'a LockedCatalog,
        package: LockedPackageId,
        objects: &'a dyn GitObjects,
        options: &LoadOptions,
    ) -> Result<Self, LoadError> {
        let path = RelPath::root();
        let source = catalog
            .package(package)
            .and_then(|row| row.source())
            .filter(|source| source.kind() == LockedSourceKind::Git)
            .ok_or_else(|| {
                refused(
                    &path,
                    "select an existing Git lock row with a complete commit",
                )
            })?;
        Self::checked(source, objects, GitMeter::Owned(Meter::new(options)))
    }

    pub(crate) fn under_meter<'m>(
        &'m self,
        meter: &'m Meter,
    ) -> Result<VerifiedGitTree<'m>, LoadError> {
        VerifiedGitTree::checked(self.source, self.objects, GitMeter::Borrowed(meter))
    }

    fn checked(
        source: &'a LockedSource,
        objects: &'a dyn GitObjects,
        meter: GitMeter<'a>,
    ) -> Result<Self, LoadError> {
        let path = RelPath::root();
        let revision = source.revision().filter(|revision| revision.len() == 40)
            .ok_or_else(|| refused(&path, "this Git verifier requires a full SHA-1 revision; other formats need an admitted verifier"))?;
        let mut hash = [0; 20];
        hex::decode_to_slice(revision, &mut hash)
            .map_err(|_| refused(&path, "locked SHA-1 revision is not valid hexadecimal"))?;
        let commit = GitObjectId(hash);
        let body = object(objects, commit, GitObjectKind::Commit, &meter, &path)?;
        // The maintained iterator unfolds continued extra headers into exact-
        // capacity buffers. Their disjoint source spans total at most body.len().
        // Admit that local bound before the decoder constructs any such buffer.
        meter.items(body.len(), &path, None)?;
        let mut root = None;
        for token in CommitRefIter::from_bytes(body, gix_hash::Kind::Sha1) {
            let token = token.map_err(|error| decode(&path, error))?;
            if let gix_object::commit::ref_iter::Token::Tree { id } = token {
                if root.is_some() {
                    return Err(refused(&path, "commit repeats its root tree"));
                }
                root = Some(object_id(id.as_bytes(), &path)?);
            }
        }
        let root = root.ok_or_else(|| refused(&path, "commit has no decoded root tree"))?;
        let tree = Self {
            objects,
            source,
            commit,
            root,
            meter,
        };
        tree.entries(root, &path)?;
        Ok(tree)
    }

    /// The exact locked source whose commit this snapshot checks.
    #[must_use]
    pub const fn source(&self) -> &LockedSource {
        self.source
    }

    /// Checked object acquisition/processing and native retained scratch charges.
    #[must_use]
    pub fn usage(&self) -> Usage {
        self.meter.usage()
    }

    fn entries(&self, id: GitObjectId, path: &RelPath) -> Result<Vec<CheckedEntry>, LoadError> {
        let body = object(self.objects, id, GitObjectKind::Tree, &self.meter, path)?;
        let mut admission = ItemAdmission::new(&self.meter, path);
        let mut memory = Memory::new(&mut admission);
        let result = (|| {
            let mut entries = Vec::new();
            // Exhaust the raw fallible iterator; lookup helpers discard decode errors.
            for entry in TreeRefIter::from_bytes(body, gix_hash::Kind::Sha1) {
                let entry = entry.map_err(|error| decode(path, error))?;
                let name = std::str::from_utf8(entry.filename.as_ref())
                    .map_err(|_| refused(path, "Git directory contains a non-UTF-8 name"))?;
                self.meter
                    .items(crate::cost::shared_text(name.len()), path, None)?;
                validate_name(name, path)?;
                let kind = match entry.mode.value() {
                    0o040_000 => EntryKind::Dir,
                    0o100_644 | 0o100_755 => EntryKind::File,
                    0o120_000 => EntryKind::Symlink,
                    0o160_000 => {
                        return Err(refused(path, "Git submodule traversal is not admitted"));
                    }
                    _ => return Err(refused(path, "Git directory has an unsupported entry mode")),
                };
                let name = memory.string(name).map_err(native)?;
                memory
                    .push(
                        &mut entries,
                        CheckedEntry {
                            name,
                            kind,
                            id: object_id(entry.oid.as_bytes(), path)?,
                        },
                    )
                    .map_err(native)?;
            }
            try_sort_unstable_by(&mut entries, |left, right| {
                self.meter.work(
                    left.name
                        .len()
                        .saturating_add(right.name.len())
                        .saturating_add(1),
                    path,
                    None,
                )?;
                Ok::<_, LoadError>(left.name.cmp(&right.name))
            })?;
            if entries.windows(2).any(|pair| pair[0].name == pair[1].name) {
                return Err(refused(path, "Git directory repeats an entry name"));
            }
            Ok(entries)
        })();
        result.map_err(|error| recover_storage(error, &mut admission))
    }

    fn locate(
        &self,
        path: &RelPath,
    ) -> Result<Option<(EntryKind, GitObjectId, String)>, LoadError> {
        let mut current = (EntryKind::Dir, self.root);
        let mut parents = WorkList::<_, 16>::new();
        let mut canonical = String::new();
        let mut admission = ItemAdmission::new(&self.meter, path);
        let mut memory = Memory::new(&mut admission);
        let result = (|| {
            for segment in path.segments() {
                self.meter
                    .work(segment.len().saturating_add(1), path, None)?;
                if current.0 != EntryKind::Dir {
                    return Err(refused(
                        path,
                        "Git path traverses a non-directory or symlink",
                    ));
                }
                if segment == ".." {
                    let (parent, length) = parents
                        .pop()
                        .ok_or_else(|| refused(path, "Git path leaves its source root"))?;
                    current = parent;
                    canonical.truncate(length);
                    continue;
                }
                self.meter.directory(parents.len(), path)?;
                let entries = self.entries(current.1, path)?;
                let matching = try_equal_range_by(&entries, |entry| {
                    self.meter.work(
                        segment
                            .len()
                            .saturating_add(entry.name.len())
                            .saturating_add(1),
                        path,
                        None,
                    )?;
                    Ok::<_, LoadError>(entry.name.as_str().cmp(segment))
                })?;
                let Some(entry) = entries.get(matching.start).filter(|_| !matching.is_empty())
                else {
                    return Ok(None);
                };
                parents
                    .try_push_admitted((current, canonical.len()), &mut memory)
                    .map_err(native)?;
                if !canonical.is_empty() {
                    memory.push_str(&mut canonical, "/").map_err(native)?;
                }
                memory.push_str(&mut canonical, segment).map_err(native)?;
                current = (entry.kind, entry.id);
            }
            Ok(Some((current.0, current.1, canonical)))
        })();
        result.map_err(|error| recover_storage(error, &mut admission))
    }

    fn checked_blob(&self, path: &RelPath) -> Result<(&[u8], String), TreeError> {
        require_first_party(path)?;
        let located = self
            .locate(path)
            .map_err(|source| tree_failure(path, source))?;
        let Some((kind, id, canonical)) = located else {
            return Err(TreeError::NotFound {
                path: path.to_string(),
            });
        };
        if kind != EntryKind::File {
            return Err(TreeError::NotRegular {
                path: path.to_string(),
            });
        }
        object(self.objects, id, GitObjectKind::Blob, &self.meter, path)
            .map(|body| (body, canonical))
            .map_err(|source| tree_failure(path, source))
    }
}

impl Tree for VerifiedGitTree<'_> {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        match self.read_unseen(path, max_bytes, &|_| false)? {
            Opened::Read { text, .. } => Ok(text),
            Opened::Seen(_) => Err(tree_failure(
                path,
                refused(path, "a forced Git blob read returned no text"),
            )),
        }
    }

    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        let (body, canonical) = self.checked_blob(path)?;
        self.meter
            .items(crate::cost::shared_text(canonical.len()), path, None)
            .map_err(|source| tree_failure(path, source))?;
        let canonical = RelPath::new(&canonical).map_err(TreeError::from)?;
        let id = FileId::git(self.commit.0, &canonical);
        // A first Seen answer cannot bypass the referenced blob's hash verification.
        if seen(&id) {
            return Ok(Opened::Seen(id));
        }
        if u64::try_from(body.len()).unwrap_or(u64::MAX) > max_bytes {
            return Err(TreeError::TooLarge {
                path: path.to_string(),
                limit: max_bytes,
            });
        }
        let text = std::str::from_utf8(body).map_err(|_| TreeError::NotUtf8 {
            path: path.to_string(),
        })?;
        let mut admission = ItemAdmission::new(&self.meter, path);
        let text = Memory::new(&mut admission)
            .string(text)
            .map_err(|error| tree_failure(path, admission.error(error)))?;
        Ok(Opened::Read { id, text })
    }

    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        require_first_party(path)?;
        let Some((kind, id, _)) = self
            .locate(path)
            .map_err(|source| tree_failure(path, source))?
        else {
            return Err(TreeError::NotFound {
                path: path.to_string(),
            });
        };
        if kind != EntryKind::Dir {
            return Err(TreeError::NotRegular {
                path: path.to_string(),
            });
        }
        let entries = self
            .entries(id, path)
            .map_err(|source| tree_failure(path, source))?;
        let mut admission = ItemAdmission::new(&self.meter, path);
        let mut output = Vec::new();
        Memory::new(&mut admission)
            .reserve(&mut output, entries.len())
            .map_err(|error| tree_failure(path, admission.error(error)))?;
        for entry in entries {
            output.push(Entry::new(entry.name, entry.kind));
        }
        Ok(output)
    }

    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        require_first_party(path)?;
        self.locate(path)
            .map(|entry| entry.map(|entry| entry.0))
            .map_err(|source| tree_failure(path, source))
    }
}

fn object<'a>(
    objects: &'a dyn GitObjects,
    id: GitObjectId,
    kind: GitObjectKind,
    meter: &Meter,
    path: &RelPath,
) -> Result<&'a [u8], LoadError> {
    let limit = meter.options().max_file_bytes;
    let object = objects
        .read_object(id, limit)
        .map_err(|error| LoadError::tree(path, None, &Origin::Discovery, error))?;
    if u64::try_from(object.body.len()).unwrap_or(u64::MAX) > limit {
        return Err(LoadError::tree(
            path,
            None,
            &Origin::Discovery,
            TreeError::TooLarge {
                path: path.to_string(),
                limit,
            },
        ));
    }
    meter.read_bytes(path, object.body.len())?;
    meter.work(object.body.len().saturating_add(64), path, None)?;
    if object.kind != kind {
        return Err(refused(path, "supplied Git object has the wrong kind"));
    }
    let mut digest = Sha1::new();
    digest.update(object.kind.name().as_bytes());
    digest.update(b" ");
    let mut admission = ItemAdmission::new(meter, path);
    let mut memory = Memory::new(&mut admission);
    let length = memory
        .format(&format_args!("{}", object.body.len()))
        .map_err(|error| admission.error(error))?;
    digest.update(length.as_bytes());
    digest.update(&[0]);
    digest.update(object.body);
    if digest.finalize() != id.0 {
        return Err(refused(
            path,
            "Git object hash does not match its authenticated reference",
        ));
    }
    Ok(object.body)
}

fn object_id(bytes: &[u8], path: &RelPath) -> Result<GitObjectId, LoadError> {
    bytes
        .try_into()
        .map(GitObjectId)
        .map_err(|_| refused(path, "Git reference has an unsupported hash format"))
}

fn validate_name(name: &str, path: &RelPath) -> Result<(), LoadError> {
    if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\', '\0']) {
        return Err(refused(path, "Git directory has an invalid path component"));
    }
    let checked = RelPath::new(name)
        .map_err(|_| refused(path, "Git name is not a supported relative component"))?;
    if checked.as_str() != name {
        return Err(refused(path, "Git name changes under path normalization"));
    }
    Ok(())
}

fn refused(path: &RelPath, message: &str) -> LoadError {
    LoadError::new(
        path,
        None,
        LoadErrorKind::Locked {
            field: "git-object",
            message: message.to_owned(),
        },
    )
}

fn decode(path: &RelPath, source: gix_object::decode::Error) -> LoadError {
    LoadError::new(path, None, LoadErrorKind::GitDecode(source))
}

fn native(source: purrdf_lex::allocation::StorageError) -> LoadError {
    LoadError::new(&RelPath::root(), None, LoadErrorKind::Storage(source))
}

fn recover_storage(error: LoadError, admission: &mut ItemAdmission<'_>) -> LoadError {
    match error.kind {
        LoadErrorKind::Storage(source) => admission.error(source),
        _ => error,
    }
}

fn tree_failure(path: &RelPath, source: LoadError) -> TreeError {
    TreeError::GitSource {
        path: path.to_string(),
        source: Box::new(source),
    }
}
