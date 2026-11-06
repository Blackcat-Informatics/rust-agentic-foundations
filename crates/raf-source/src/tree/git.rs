// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A sealed Tree over native immutable Git objects, without executable helpers.

use super::{Entry, EntryKind, FileId, Opened, Tree, require_first_party};
use crate::meter::{ItemAdmission, Meter, Usage};
use crate::{
    LoadError, LoadErrorKind, LoadOptions, LockedCatalog, LockedPackageId, LockedSource,
    LockedSourceKind, RelPath, TreeError,
};
use purrdf_hash::{hex, sha1::Sha1};
use purrdf_lex::allocation::Memory;
use purrdf_lex::walk::{WorkList, try_equal_range_by, try_sort_unstable_by};
use std::path::{Path, PathBuf};

/// A committed repository read by the native Git library, without checkout reads.
///
/// Only this host constructor admits a source to the locked package join. The
/// native object library owns decoding; public authority/FileId constructors cannot fabricate
/// admission. Complete committed listings govern paths and absence. This trusts
/// the native decoder and object store, not authorship or a clean checkout.
/// Decoder internals are outside RAF's portable meters; acquired buffers and
/// model processing retain their original grants.
pub struct CommittedGitTree<'a> {
    repository: PathBuf,
    native: crate::git::Repository,
    source: &'a LockedSource,
    commit: [u8; 20],
    entries: Vec<Record>,
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
impl std::fmt::Debug for CommittedGitTree<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommittedGitTree")
            .field("source", &self.source)
            .field("repository", &self.repository)
            .finish_non_exhaustive()
    }
}
struct Record {
    path: String,
    kind: EntryKind,
    id: [u8; 20],
    size: usize,
}

impl<'a> CommittedGitTree<'a> {
    /// Open an exact catalog Git commit natively, without processes or fetching.
    ///
    /// Paths come solely from committed objects. A dirty worktree is neither
    /// read nor certified clean. Locked source identities require SHA-1;
    /// portable callers retain the ordinary Tree boundary.
    ///
    /// # Errors
    /// Refuses missing repositories/commits, non-Git or unsupported revisions,
    /// invalid objects/modes and original resource failures.
    pub fn open(
        catalog: &'a LockedCatalog,
        package: LockedPackageId,
        repository: impl AsRef<Path>,
        options: &LoadOptions,
    ) -> Result<Self, LoadError> {
        let path = RelPath::root();
        let source = catalog
            .package(package)
            .and_then(|row| row.source())
            .filter(|source| source.kind() == LockedSourceKind::Git)
            .ok_or_else(|| refused(&path, "select an existing Git lock row with a full commit"))?;
        Self::checked(
            source,
            repository.as_ref(),
            GitMeter::Owned(Meter::new(options)),
        )
    }
    pub(crate) fn under_meter<'m>(
        &'m self,
        meter: &'m Meter,
    ) -> Result<CommittedGitTree<'m>, LoadError> {
        CommittedGitTree::checked(self.source, &self.repository, GitMeter::Borrowed(meter))
    }
    fn checked(
        source: &'a LockedSource,
        repository: &Path,
        meter: GitMeter<'a>,
    ) -> Result<Self, LoadError> {
        let path = RelPath::root();
        let revision = source
            .revision()
            .filter(|revision| revision.len() == 40)
            .ok_or_else(|| {
                refused(
                    &path,
                    "committed Git source loading requires a full SHA-1 revision",
                )
            })?;
        let commit = parse_id(revision.as_bytes(), &path)?;
        let repository = super::disk::canonical_root(repository)
            .map_err(|source| git_io(&path, "repository", source))?;
        meter.items(repository.as_os_str().len(), &path, None)?;
        let native = crate::git::Repository::open_with_allocation_limit(
            &repository,
            meter.options().max_item_bytes,
        )
        .map_err(|error| git_io(&path, "repository", std::io::Error::other(error)))?;
        meter.work(revision.len().saturating_add(1), &path, None)?;
        meter.read_bytes(&path, revision.len())?;
        meter.items(revision.len(), &path, None)?;
        let actual = native
            .resolve_commit(revision)
            .map_err(|error| git_io(&path, "commit", std::io::Error::other(error)))?;
        if actual != revision {
            return Err(refused(
                &path,
                "the full locked ID does not name that exact commit",
            ));
        }
        let mut admission = ItemAdmission::new(&meter, &path);
        let listing = native
            .tree_admitted(
                revision,
                meter.options().max_total_bytes,
                &mut Memory::new(&mut admission),
            )
            .map_err(|error| match error.storage() {
                Some(storage) => admission.error(storage),
                None => git_io(&path, "tree", std::io::Error::other(error)),
            })?;
        let entries = admit_listing(listing, &meter, &path)?;
        Ok(Self {
            repository,
            native,
            source,
            commit,
            entries,
            meter,
        })
    }
    /// The exact locked source selected for this committed host Tree.
    #[must_use]
    pub const fn source(&self) -> &LockedSource {
        self.source
    }
    /// RAF's acquired protocol, native storage and processing charges.
    #[must_use]
    pub fn usage(&self) -> Usage {
        self.meter.usage()
    }

    fn record(&self, path: &str, file: &RelPath) -> Result<Option<&Record>, LoadError> {
        let found = try_equal_range_by(&self.entries, |record| {
            self.meter.work(
                record
                    .path
                    .len()
                    .saturating_add(path.len())
                    .saturating_add(1),
                file,
                None,
            )?;
            Ok::<_, LoadError>(record.path.as_str().cmp(path))
        })?;
        Ok(self.entries.get(found.start).filter(|_| !found.is_empty()))
    }
    fn locate(&self, path: &RelPath) -> Result<Option<(EntryKind, String)>, LoadError> {
        let mut kind = EntryKind::Dir;
        let mut canonical = String::new();
        let mut parents = WorkList::<_, 16>::new();
        let mut admission = ItemAdmission::new(&self.meter, path);
        let mut memory = Memory::new(&mut admission);
        let result = (|| {
            for segment in path.segments() {
                self.meter
                    .work(segment.len().saturating_add(1), path, None)?;
                if kind != EntryKind::Dir {
                    return Err(refused(
                        path,
                        "Git path traverses a non-directory or symlink",
                    ));
                }
                if segment == ".." {
                    canonical.truncate(
                        parents
                            .pop()
                            .ok_or_else(|| refused(path, "Git path leaves its source root"))?,
                    );
                    kind = EntryKind::Dir;
                    continue;
                }
                self.meter.directory(parents.len(), path)?;
                parents
                    .try_push_admitted(canonical.len(), &mut memory)
                    .map_err(native)?;
                if !canonical.is_empty() {
                    memory.push_str(&mut canonical, "/").map_err(native)?;
                }
                memory.push_str(&mut canonical, segment).map_err(native)?;
                let Some(record) = self.record(&canonical, path)? else {
                    return Ok(None);
                };
                kind = record.kind;
            }
            Ok(Some((kind, canonical)))
        })();
        result.map_err(|error| recover(error, &mut admission))
    }
}

impl Tree for CommittedGitTree<'_> {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        match self.read_unseen(path, max_bytes, &|_| false)? {
            Opened::Read { text, .. } => Ok(text),
            Opened::Seen(_) => Err(tree_failure(
                path,
                refused(path, "forced committed blob read returned Seen"),
            )),
        }
    }
    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        require_first_party(path)?;
        let (kind, canonical) = self
            .locate(path)
            .map_err(|error| tree_failure(path, error))?
            .ok_or_else(|| TreeError::NotFound {
                path: path.to_string(),
            })?;
        if kind != EntryKind::File {
            return Err(TreeError::NotRegular {
                path: path.to_string(),
            });
        }
        let record = self
            .record(&canonical, path)
            .map_err(|error| tree_failure(path, error))?
            .ok_or_else(|| {
                tree_failure(
                    path,
                    refused(path, "located Git file has no committed record"),
                )
            })?;
        self.meter
            .items(crate::cost::shared_text(canonical.len()), path, None)
            .map_err(|error| tree_failure(path, error))?;
        let canonical_path = RelPath::new(&canonical).map_err(TreeError::from)?;
        let id = FileId::git(self.commit, &canonical_path);
        let previously_seen = seen(&id);
        let limit = if previously_seen {
            self.meter.options().max_file_bytes
        } else {
            max_bytes.min(self.meter.options().max_file_bytes)
        };
        if u64::try_from(record.size).unwrap_or(u64::MAX) > limit {
            return Err(TreeError::TooLarge {
                path: path.to_string(),
                limit,
            });
        }
        let mut admission = ItemAdmission::new(&self.meter, path);
        let mut memory = Memory::new(&mut admission);
        let result = (|| {
            let object = memory
                .format(&format_args!("{}", hex::Lower(&record.id)))
                .map_err(native)?;
            self.meter.work(1, path, None)?;
            let bytes = self
                .native
                .blob_admitted(&object, record.size, &mut memory)
                .map_err(|error| match error.storage() {
                    Some(storage) => native(storage),
                    None => git_io(path, "blob", std::io::Error::other(error)),
                })?;
            self.meter.read_bytes(path, bytes.len())?;
            if bytes.len() != record.size {
                return Err(refused(
                    path,
                    "committed blob size differs from its complete listing",
                ));
            }
            self.meter
                .work(bytes.len().saturating_add(64), path, None)?;
            let length = memory
                .format(&format_args!("{}", bytes.len()))
                .map_err(native)?;
            let mut hash = Sha1::new();
            hash.update(b"blob ");
            hash.update(length.as_bytes());
            hash.update(&[0]);
            hash.update(&bytes);
            if hash.finalize() != record.id {
                return Err(refused(
                    path,
                    "committed blob hash does not match its listed object ID",
                ));
            }
            if previously_seen {
                return Ok(Opened::Seen(id));
            }
            let text = std::str::from_utf8(&bytes).map_err(|_| {
                LoadError::tree(
                    path,
                    None,
                    &crate::Origin::Discovery,
                    TreeError::NotUtf8 {
                        path: path.to_string(),
                    },
                )
            })?;
            let text = memory.string(text).map_err(native)?;
            Ok(Opened::Read { id, text })
        })();
        result.map_err(|error| tree_failure(path, recover(error, &mut admission)))
    }
    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        require_first_party(path)?;
        let (kind, canonical) = self
            .locate(path)
            .map_err(|error| tree_failure(path, error))?
            .ok_or_else(|| TreeError::NotFound {
                path: path.to_string(),
            })?;
        if kind != EntryKind::Dir {
            return Err(TreeError::NotRegular {
                path: path.to_string(),
            });
        }
        let mut admission = ItemAdmission::new(&self.meter, path);
        let mut memory = Memory::new(&mut admission);
        let result = (|| {
            let mut output = Vec::new();
            for record in &self.entries {
                self.meter.work(
                    record
                        .path
                        .len()
                        .saturating_add(canonical.len())
                        .saturating_add(1),
                    path,
                    None,
                )?;
                let (parent, name) = record
                    .path
                    .rsplit_once('/')
                    .unwrap_or(("", record.path.as_str()));
                if parent == canonical {
                    let name = memory.string(name).map_err(native)?;
                    memory
                        .push(&mut output, Entry::new(name, record.kind))
                        .map_err(native)?;
                }
            }
            Ok(output)
        })();
        result.map_err(|error| tree_failure(path, recover(error, &mut admission)))
    }
    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        require_first_party(path)?;
        self.locate(path)
            .map(|entry| entry.map(|entry| entry.0))
            .map_err(|error| tree_failure(path, error))
    }
}
fn admit_listing(
    listing: Vec<crate::git::Entry>,
    meter: &Meter,
    path: &RelPath,
) -> Result<Vec<Record>, LoadError> {
    let mut admission = ItemAdmission::new(meter, path);
    let mut memory = Memory::new(&mut admission);
    let result = (|| {
        let mut records = Vec::new();
        for entry in listing {
            meter.work(entry.path.len().saturating_add(64), path, None)?;
            meter.read_bytes(path, entry.path.len().saturating_add(64))?;
            for name in entry.path.split('/') {
                validate_name(name, path)?;
            }
            meter.items(crate::cost::shared_text(entry.path.len()), path, None)?;
            let checked = RelPath::new(&entry.path)
                .map_err(|_| refused(path, "unsupported committed path"))?;
            if checked.as_str() != entry.path {
                return Err(refused(path, "committed path changes under normalization"));
            }
            let kind = match entry.kind {
                crate::git::EntryKind::Tree => EntryKind::Dir,
                crate::git::EntryKind::Blob => EntryKind::File,
                crate::git::EntryKind::Symlink => EntryKind::Symlink,
                crate::git::EntryKind::Submodule => {
                    return Err(refused(path, "Git submodules are not admitted"));
                }
            };
            let record = Record {
                path: memory.string(&entry.path).map_err(native)?,
                kind,
                id: parse_id(entry.id.as_bytes(), path)?,
                size: entry.size,
            };
            memory.push(&mut records, record).map_err(native)?;
        }
        try_sort_unstable_by(&mut records, |a, b| {
            meter.work(
                a.path.len().saturating_add(b.path.len()).saturating_add(1),
                path,
                None,
            )?;
            Ok::<_, LoadError>(a.path.cmp(&b.path))
        })?;
        Ok(records)
    })();
    result.map_err(|error| recover(error, &mut admission))
}

fn git_io(path: &RelPath, operation: &'static str, source: std::io::Error) -> LoadError {
    LoadError::new(path, None, LoadErrorKind::GitIo { operation, source })
}

fn parse_id(bytes: &[u8], path: &RelPath) -> Result<[u8; 20], LoadError> {
    if bytes.len() != 40 {
        return Err(refused(
            path,
            "Git source requires complete SHA-1 object IDs",
        ));
    }
    let mut id = [0; 20];
    let text = std::str::from_utf8(bytes).map_err(|_| refused(path, "invalid Git object ID"))?;
    hex::decode_to_slice(text, &mut id).map_err(|_| refused(path, "invalid Git object ID"))?;
    Ok(id)
}
fn validate_name(name: &str, path: &RelPath) -> Result<(), LoadError> {
    if name.is_empty() || name == "." || name == ".." || name.contains(['\\', '\0']) {
        return Err(refused(path, "Git directory has an invalid path component"));
    }
    Ok(())
}
fn refused(path: &RelPath, message: &str) -> LoadError {
    LoadError::new(
        path,
        None,
        LoadErrorKind::Locked {
            field: "git-source",
            message: message.to_owned(),
            source: None,
        },
    )
}
fn native(source: purrdf_lex::allocation::StorageError) -> LoadError {
    LoadError::new(&RelPath::root(), None, LoadErrorKind::Storage(source))
}
fn recover(error: LoadError, admission: &mut ItemAdmission<'_>) -> LoadError {
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
