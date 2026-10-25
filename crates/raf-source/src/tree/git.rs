// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A sealed host Tree over installed Git's immutable committed contents.

#[cfg(any(unix, windows))]
mod command;
#[cfg(not(any(unix, windows)))]
mod command {
    use super::{LoadError, LoadErrorKind, Meter, RelPath, refused};
    use std::path::Path;
    pub(super) fn read(
        _repository: &Path,
        _args: &[&str],
        _cap: usize,
        _meter: &Meter,
        path: &RelPath,
    ) -> Result<Vec<u8>, LoadError> {
        Err(refused(
            path,
            "committed Git source loading requires a supported process host",
        ))
    }
    pub(super) fn io(path: &RelPath, operation: &'static str, source: std::io::Error) -> LoadError {
        LoadError::new(path, None, LoadErrorKind::GitIo { operation, source })
    }
}

use super::{Entry, EntryKind, FileId, Opened, Tree, require_first_party};
use crate::meter::{ItemAdmission, Meter, Usage};
use crate::{
    LoadError, LoadErrorKind, LoadOptions, LockedCatalog, LockedPackageId, LockedSource,
    LockedSourceKind, RelPath, TreeError,
};
use purrdf_hash::{hex, sha1::Sha1};
use purrdf_lex::allocation::{Admission, Memory};
use purrdf_lex::walk::{WorkList, try_equal_range_by, try_sort_unstable_by};
use std::path::{Path, PathBuf};

/// A committed repository read by trusted installed Git, without checkout reads.
///
/// Only this host constructor admits a source to the locked package join. Git
/// owns object decoding; public authority/FileId constructors cannot fabricate
/// admission. Complete committed listings govern paths and absence. This trusts
/// Git and its object store, not repository authorship or a clean checkout.
/// Git's internal CPU and allocations are outside RAF's portable meters.
pub struct CommittedGitTree<'a> {
    repository: PathBuf,
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
    /// Open an exact catalog Git commit through installed Git, without fetching.
    ///
    /// Paths come solely from committed objects. A dirty worktree is neither
    /// read nor certified clean. SHA-1 object format and host process access are
    /// required; portable callers retain the ordinary Tree boundary.
    ///
    /// # Errors
    /// Refuses missing Git/repository/commit, non-Git or unsupported revisions,
    /// command failures, invalid protocol/modes and original resource failures.
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
        if !cfg!(any(unix, windows)) {
            return Err(refused(
                &path,
                "committed Git source loading requires a supported process host",
            ));
        }
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
            .map_err(|source| command::io(&path, "repository", source))?;
        meter.items(repository.as_os_str().len(), &path, None)?;
        let mut admission = ItemAdmission::new(&meter, &path);
        let expression = Memory::new(&mut admission)
            .format(&format_args!("{revision}^{{commit}}"))
            .map_err(|error| admission.error(error))?;
        let actual = command::read(
            &repository,
            &["rev-parse", "--verify", "--end-of-options", &expression],
            41,
            &meter,
            &path,
        )?;
        if actual.strip_suffix(b"\n") != Some(revision.as_bytes()) {
            return Err(refused(
                &path,
                "the full locked ID does not name that exact commit",
            ));
        }
        let output = command::read(
            &repository,
            &["ls-tree", "-r", "-t", "-z", "-l", "--full-tree", revision],
            meter.options().max_total_bytes,
            &meter,
            &path,
        )?;
        let entries = parse_listing(&output, &meter, &path)?;
        Ok(Self {
            repository,
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
            let bytes = command::read(
                &self.repository,
                &["cat-file", "blob", &object],
                record.size,
                &self.meter,
                path,
            )?;
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
fn parse_listing(bytes: &[u8], meter: &Meter, path: &RelPath) -> Result<Vec<Record>, LoadError> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    if !bytes.is_empty() && bytes.last() != Some(&0) {
        return Err(refused(path, "unterminated Git directory protocol"));
    }
    let mut admission = ItemAdmission::new(meter, path);
    let mut memory = Memory::new(&mut admission);
    let result = (|| {
        let mut records = Vec::new();
        let lines = &bytes[..bytes.len() - 1];
        for line in lines.split(|byte| *byte == 0) {
            let record = parse_record(line, meter, path, &mut memory)?;
            memory.push(&mut records, record).map_err(native)?;
        }
        try_sort_unstable_by(&mut records, |left, right| {
            meter.work(
                left.path
                    .len()
                    .saturating_add(right.path.len())
                    .saturating_add(1),
                path,
                None,
            )?;
            Ok::<_, LoadError>(left.path.cmp(&right.path))
        })?;
        for pair in records.windows(2) {
            meter.work(1, path, None)?;
            if pair[0].path == pair[1].path {
                return Err(refused(path, "Git directory repeats a path"));
            }
        }
        for record in &records {
            if let Some((parent, _)) = record.path.rsplit_once('/') {
                let found = try_equal_range_by(&records, |other| {
                    meter.work(
                        parent
                            .len()
                            .saturating_add(other.path.len())
                            .saturating_add(1),
                        path,
                        None,
                    )?;
                    Ok::<_, LoadError>(other.path.as_str().cmp(parent))
                })?;
                if !records
                    .get(found.start)
                    .is_some_and(|entry| !found.is_empty() && entry.kind == EntryKind::Dir)
                {
                    return Err(refused(
                        path,
                        "Git listing lacks an actual parent directory",
                    ));
                }
            }
        }
        Ok(records)
    })();
    result.map_err(|error| recover(error, &mut admission))
}
fn parse_record<S: Admission + ?Sized>(
    line: &[u8],
    meter: &Meter,
    path: &RelPath,
    memory: &mut Memory<'_, S>,
) -> Result<Record, LoadError> {
    meter.work(line.len().saturating_add(1), path, None)?;
    let tab = line
        .iter()
        .position(|byte| *byte == b'\t')
        .ok_or_else(|| refused(path, "Git listing has no name delimiter"))?;
    let header = std::str::from_utf8(&line[..tab])
        .map_err(|_| refused(path, "non-ASCII Git listing header"))?;
    let mut fields = header.split_ascii_whitespace();
    let mode = fields
        .next()
        .ok_or_else(|| refused(path, "missing Git mode"))?;
    let kind = fields
        .next()
        .ok_or_else(|| refused(path, "missing Git type"))?;
    let id = parse_id(
        fields
            .next()
            .ok_or_else(|| refused(path, "missing Git object ID"))?
            .as_bytes(),
        path,
    )?;
    let size = fields
        .next()
        .ok_or_else(|| refused(path, "missing Git size"))?;
    if fields.next().is_some() {
        return Err(refused(path, "unexpected Git listing fields"));
    }
    let kind = match (mode, kind) {
        ("040000", "tree") if size == "-" => EntryKind::Dir,
        ("100644" | "100755", "blob") => EntryKind::File,
        ("120000", "blob") => EntryKind::Symlink,
        ("160000", _) => return Err(refused(path, "Git submodules are not admitted")),
        _ => return Err(refused(path, "unsupported Git mode/type/size")),
    };
    let size = if kind == EntryKind::Dir {
        0
    } else {
        if size.is_empty() || !size.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(refused(path, "invalid Git blob size"));
        }
        // The protocol is unsigned digits, narrower than XSD's lexical grammar.
        // Bound significant digits before native parsing to keep its value inline.
        let significant = size.trim_start_matches('0');
        let significant = if significant.is_empty() {
            "0"
        } else {
            significant
        };
        let max_digits = usize::try_from(usize::MAX.ilog10() + 1)
            .map_err(|_| refused(path, "unsupported host size width"))?;
        if significant.len() > max_digits {
            return Err(refused(
                path,
                "Git blob size exceeds supported address space",
            ));
        }
        let value = purrdf_xsd::numeric::parse_integer_typed(
            significant,
            purrdf_xsd::XsdDatatype::NonNegativeInteger,
        )
        .map_err(|_| refused(path, "invalid Git blob size"))?;
        let value = value
            .as_i128()
            .ok_or_else(|| refused(path, "Git blob size exceeds supported address space"))?;
        usize::try_from(value)
            .map_err(|_| refused(path, "Git blob size exceeds supported address space"))?
    };
    let name = std::str::from_utf8(&line[tab + 1..])
        .map_err(|_| refused(path, "Git listing contains a non-UTF-8 path"))?;
    for segment in name.split('/') {
        validate_name(segment, path)?;
    }
    meter.items(crate::cost::shared_text(name.len()), path, None)?;
    let checked = RelPath::new(name).map_err(|_| refused(path, "unsupported committed path"))?;
    if checked.as_str() != name {
        return Err(refused(path, "committed path changes under normalization"));
    }
    let path = memory.string(name).map_err(native)?;
    Ok(Record {
        path,
        kind,
        id,
        size,
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_directory_protocol_refuses_malformed_records_beside_exact_metadata() {
        let path = RelPath::root();
        let valid = b"040000 tree 0000000000000000000000000000000000000001       -\tsrc\0\
100644 blob 0000000000000000000000000000000000000002       6\tsrc/lib.rs\0";
        let meter = Meter::new(&LoadOptions::default());
        assert_eq!(
            parse_listing(valid, &meter, &path)
                .expect("complete metadata")
                .len(),
            2
        );
        for bytes in [
            b"\0".as_slice(),
            &valid[..valid.len() - 1],
            b"100644 blob 0000000000000000000000000000000000000002 6\tsrc/lib.rs\0",
            b"100644 blob 0000000000000000000000000000000000000002 BAD\tfile\0",
            b"100644 blob 0000000000000000000000000000000000000002 6\t../file\0",
            b"160000 commit 0000000000000000000000000000000000000002 -\tsub\0",
            b"100644 blob 0000000000000000000000000000000000000002 6\t\xff\0",
        ] {
            assert!(parse_listing(bytes, &meter, &path).is_err(), "{bytes:?}");
        }
        assert!(
            parse_listing(b"", &meter, &path)
                .expect("empty tree")
                .is_empty()
        );
        let mut resident = purrdf_lex::allocation::Resident;
        let mut memory = Memory::new(&mut resident);
        let maximum = memory
            .format(&format_args!("{}", usize::MAX))
            .expect("maximum");
        let overflow = memory
            .format(&format_args!("{maximum}0"))
            .expect("overflow");
        for (size, expected) in [
            ("0", Some(0)),
            ("0000000000000000000000000000006", Some(6)),
            (maximum.as_str(), Some(usize::MAX)),
            ("+6", None),
            ("-1", None),
            ("1.0", None),
            (overflow.as_str(), None),
        ] {
            let listing = memory
                .format(&format_args!(
                    "100644 blob 0000000000000000000000000000000000000002 {size}\tfile\0"
                ))
                .expect("protocol fixture");
            let result = parse_listing(listing.as_bytes(), &meter, &path);
            match expected {
                Some(size) => assert_eq!(result.expect("exact native size")[0].size, size),
                None => assert!(result.is_err(), "{size}"),
            }
        }
    }
}
