// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Sealed registry contents authenticated against the selected archive checksum.

use super::git::{native as storage, recover};
use super::{Entry, EntryKind, FileId, Opened, SealedMeter, Tree, require_first_party};
use crate::meter::{ItemAdmission, Meter};
use crate::{
    LoadError, LoadErrorKind, LoadOptions, LockedCatalog, LockedPackage, LockedPackageId,
    LockedSource, LockedSourceKind, RelPath, TreeError,
};
use purrdf_lex::allocation::{Memory, Shared, StorageError};
use purrdf_lex::walk::{try_equal_range_by, try_sort_unstable_by};
use std::io;

/// Immutable package files from one checksum-authenticated registry archive.
///
/// This constructor consumes supplied bytes only: it never finds a cache, fetches,
/// extracts files or executes inspected code. Both compressed and decompressed
/// archive bytes are capped by `max_total_bytes`. Native gzip decoding and the
/// strict tar inventory run under RAF's item and work grants. RAF's
/// retained file payloads, listing and subsequent source operations are metered.
/// Authentication establishes equality to the lock checksum, not trusted authorship.
pub struct RegistryArchiveTree<'a> {
    package: &'a LockedPackage,
    source: &'a LockedSource,
    image: Shared<Image>,
    meter: SealedMeter<'a>,
}

struct Image {
    records: Vec<Record>,
    retained: usize,
}
struct Record {
    path: String,
    bytes: Option<Vec<u8>>,
}
impl<'a> RegistryArchiveTree<'a> {
    /// Authenticate and load the `.crate` bytes for an exact registry lock row.
    ///
    /// The native tar interpretation supplies paths and file bodies. RAF requires
    /// canonical paths under exactly `name-version/`, regular files/directories,
    /// a root Cargo manifest and an unambiguous file/directory inventory. Contents
    /// remain binary until a source read requires UTF-8.
    ///
    /// # Errors
    /// Refuses missing checksums, nonregistry rows, checksum mismatch, exhausted
    /// byte/storage/work grants, nested transport, native errors or diagnostics,
    /// links, special files and invalid package inventories. Original native
    /// causes remain available through the error chain.
    pub fn from_bytes(
        catalog: &'a LockedCatalog,
        package: LockedPackageId,
        archive: &[u8],
        options: &LoadOptions,
    ) -> Result<Self, LoadError> {
        let path = RelPath::root();
        let package = catalog
            .package(package)
            .filter(|row| {
                row.source().is_some_and(|source| {
                    matches!(
                        source.kind(),
                        LockedSourceKind::Registry | LockedSourceKind::SparseRegistry
                    )
                })
            })
            .ok_or_else(|| refused("select an existing registry lock row"))?;
        let checksum = package
            .checksum()
            .ok_or_else(|| refused("registry row needs its archive checksum"))?;
        let meter = Meter::new(options);
        meter.items(catalog.usage().item_bytes, &path, None)?;
        meter.read_bytes(&path, archive.len())?;
        meter.work(archive.len().saturating_add(1), &path, None)?;
        if purrdf_core::ContentDigest::of(archive).as_bytes() != checksum {
            return Err(refused(
                "supplied registry archive does not match the selected lock checksum",
            ));
        }
        let source = package
            .source()
            .ok_or_else(|| refused("registry source is absent"))?;
        let tar = decode(archive, options, &meter)?;
        let image = image(&tar, options, package, catalog.usage().item_bytes, &meter)?;
        Ok(Self {
            package,
            source,
            image,
            meter: SealedMeter::Owned(meter),
        })
    }

    /// The registry identity whose archive checksum authenticated these contents.
    #[must_use]
    pub fn source(&self) -> &LockedSource {
        self.source
    }
    pub(crate) fn matches(&self, row: &LockedPackage) -> bool {
        self.package.name() == row.name()
            && self.package.version() == row.version()
            && self.package.source() == row.source()
            && self.package.checksum() == row.checksum()
    }
    pub(crate) fn under_meter<'m>(
        &'m self,
        meter: &'m Meter,
    ) -> Result<RegistryArchiveTree<'m>, LoadError> {
        meter.items(self.image.retained, &RelPath::root(), None)?;
        let image = self.image.try_clone().map_err(native_error)?;
        Ok(RegistryArchiveTree {
            package: self.package,
            source: self.source,
            image,
            meter: SealedMeter::Borrowed(meter),
        })
    }
    fn record(&self, name: &str, path: &RelPath) -> Result<Option<&Record>, LoadError> {
        let found = try_equal_range_by(&self.image.records, |record| {
            self.meter.work(
                record
                    .path
                    .len()
                    .saturating_add(name.len())
                    .saturating_add(1),
                path,
                None,
            )?;
            Ok::<_, LoadError>(record.path.as_str().cmp(name))
        })?;
        Ok(self
            .image
            .records
            .get(found.start)
            .filter(|_| !found.is_empty()))
    }
    fn locate(&self, path: &RelPath) -> Result<Option<(EntryKind, String)>, LoadError> {
        super::inventory::locate(path, &self.meter, |canonical| {
            self.record(canonical, path).map(|record| {
                record.map(|record| {
                    if record.bytes.is_some() {
                        EntryKind::File
                    } else {
                        EntryKind::Dir
                    }
                })
            })
        })
    }
}

fn decode(archive: &[u8], options: &LoadOptions, meter: &Meter) -> Result<Vec<u8>, LoadError> {
    let path = RelPath::root();
    let tar = purrdf_deflate::gzip::decompress_with_limit(archive, options.max_total_bytes as u64)
        .map_err(native_error)?;
    // The native importer detects transport even for its byte API. Refuse
    // another envelope: the strict inventory frames tar bytes only.
    if purrdf_gts::transport::detect_transport(&tar, None).is_some() {
        return Err(refused(
            "registry archive contains another transport envelope",
        ));
    }
    meter.items(tar.capacity(), &path, None)?;
    meter.work(tar.len().saturating_add(1), &path, None)?;
    Ok(tar)
}

fn image(
    tar: &[u8],
    options: &LoadOptions,
    package: &LockedPackage,
    catalog_bytes: usize,
    meter: &Meter,
) -> Result<Shared<Image>, LoadError> {
    let path = RelPath::root();
    let mut admission = ItemAdmission::new(meter, &path);
    let limits = purrdf_gts::tar::inventory::Limits {
        source_bytes: tar.len() as u64,
        entries: tar.len() / 512 + 1,
        metadata_bytes: tar.len() as u64,
        storage_bytes: options.max_item_bytes,
        work: options.max_work as u64,
    };
    let inventory = purrdf_gts::tar::inventory::read(tar, limits, &mut admission)
        .map_err(|error| admission.recover(native_error(error)))?;
    let result = (|| {
        let mut memory = Memory::new(&mut admission);
        let prefix = memory
            .format(&format_args!("{}-{}/", package.name(), package.version()))
            .map_err(storage)?;
        let mut records = Vec::new();
        insert(&mut records, "", None, meter, &mut memory)?;
        for entry in inventory.entries() {
            let (head, tail) = entry
                .path()
                .and_then(purrdf_gts::tar::inventory::Name::utf8_parts)
                .map_err(native_error)?;
            let name = match head {
                Some(head) => memory
                    .format(&format_args!("{head}/{tail}"))
                    .map_err(storage)?,
                None => memory.string(tail).map_err(storage)?,
            };
            meter.work(name.len().saturating_add(1), &path, None)?;
            let relative = name
                .strip_prefix(&prefix)
                .ok_or_else(|| refused("registry entry is outside its package prefix"))?;
            let kind = match entry.kind() {
                purrdf_gts::files::FileEntryKind::File => EntryKind::File,
                purrdf_gts::files::FileEntryKind::Directory => EntryKind::Dir,
                _ => {
                    return Err(refused(
                        "registry entries must be regular files or directories",
                    ));
                }
            };
            let relative = if kind == EntryKind::Dir {
                relative.trim_end_matches('/')
            } else {
                relative
            };
            meter.items(crate::cost::shared_text(relative.len()), &path, None)?;
            let checked =
                RelPath::new(relative).map_err(|_| refused("invalid registry entry path"))?;
            if checked.as_str() != relative
                || checked.segments().any(|part| part == "..")
                || (relative.is_empty() && kind != EntryKind::Dir)
            {
                return Err(refused("registry paths must be canonical relative names"));
            }
            let bytes = if kind == EntryKind::File {
                let data = entry.payload().map_err(native_error)?;
                meter.work(data.len().saturating_add(1), &checked, None)?;
                let mut bytes = Vec::new();
                memory.reserve(&mut bytes, data.len()).map_err(storage)?;
                bytes.extend_from_slice(data);
                Some(bytes)
            } else {
                None
            };
            let mut ancestor = relative;
            while let Some((parent, _)) = ancestor.rsplit_once('/') {
                insert(&mut records, parent, None, meter, &mut memory)?;
                ancestor = parent;
            }
            insert(&mut records, relative, bytes, meter, &mut memory)?;
        }
        try_sort_unstable_by(&mut records, |a, b| {
            meter.work(
                a.path.len().saturating_add(b.path.len()).saturating_add(1),
                &path,
                None,
            )?;
            Ok::<_, LoadError>(a.path.cmp(&b.path))
        })?;
        meter.work(records.len().saturating_mul(11), &path, None)?;
        if !records
            .iter()
            .any(|record| record.path == "Cargo.toml" && record.bytes.is_some())
        {
            return Err(refused("registry package requires a root Cargo.toml file"));
        }
        meter.items(Shared::<Image>::allocation_layout().size(), &path, None)?;
        let retained = meter.usage().item_bytes.saturating_sub(catalog_bytes);
        let image = Shared::try_new(Image { records, retained })
            .map_err(|_| storage(StorageError::AllocationFailed))?;
        Ok(image)
    })();
    result.map_err(|error| recover(error, &mut admission))
}

fn insert<S: purrdf_lex::allocation::Admission + ?Sized>(
    records: &mut Vec<Record>,
    path: &str,
    bytes: Option<Vec<u8>>,
    meter: &Meter,
    memory: &mut Memory<'_, S>,
) -> Result<(), LoadError> {
    for existing in records.iter() {
        meter.work(
            existing
                .path
                .len()
                .saturating_add(path.len())
                .saturating_add(1),
            &RelPath::root(),
            None,
        )?;
        if existing.path == path {
            return if existing.bytes.is_none() && bytes.is_none() {
                Ok(())
            } else {
                Err(refused(
                    "registry inventory has conflicting file/directory paths",
                ))
            };
        }
    }
    let path = memory.string(path).map_err(storage)?;
    memory
        .push(records, Record { path, bytes })
        .map_err(storage)
}

impl Tree for RegistryArchiveTree<'_> {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        match self.read_unseen(path, max_bytes, &|_| false)? {
            Opened::Read { text, .. } => Ok(text),
            Opened::Seen(_) => unreachable!("the callback never marks a file seen"),
        }
    }
    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        require_first_party(path)?;
        let (_, canonical) = self
            .locate(path)
            .map_err(|error| tree_failure(path, error))?
            .ok_or_else(|| TreeError::NotFound {
                path: path.to_string(),
            })?;
        let record = self
            .record(&canonical, path)
            .map_err(|error| tree_failure(path, error))?
            .ok_or_else(|| TreeError::NotFound {
                path: path.to_string(),
            })?;
        let bytes = record.bytes.as_ref().ok_or_else(|| TreeError::NotRegular {
            path: path.to_string(),
        })?;
        self.meter
            .items(crate::cost::shared_text(record.path.len()), path, None)
            .map_err(|error| tree_failure(path, error))?;
        let id = FileId::from_key(record.path.as_bytes());
        if seen(&id) {
            return Ok(Opened::Seen(id));
        }
        if bytes.len() as u64 > max_bytes {
            return Err(TreeError::TooLarge {
                path: path.to_string(),
                limit: max_bytes,
            });
        }
        self.meter
            .read_bytes(path, bytes.len())
            .map_err(|error| tree_failure(path, error))?;
        self.meter
            .work(bytes.len().saturating_add(1), path, None)
            .map_err(|error| tree_failure(path, error))?;
        let text = std::str::from_utf8(bytes).map_err(|_| TreeError::NotUtf8 {
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
        let result = (|| {
            let mut memory = Memory::new(&mut admission);
            let mut output = Vec::new();
            for record in &self.image.records {
                self.meter.work(
                    record
                        .path
                        .len()
                        .saturating_add(path.as_str().len())
                        .saturating_add(1),
                    path,
                    None,
                )?;
                let relative = if canonical.is_empty() {
                    record.path.as_str()
                } else {
                    let Some(relative) = record
                        .path
                        .strip_prefix(&canonical)
                        .and_then(|rest| rest.strip_prefix('/'))
                    else {
                        continue;
                    };
                    relative
                };
                if relative.is_empty() || relative.contains('/') {
                    continue;
                }
                let name = memory.string(relative).map_err(storage)?;
                let kind = if record.bytes.is_some() {
                    EntryKind::File
                } else {
                    EntryKind::Dir
                };
                memory
                    .push(&mut output, Entry::new(name, kind))
                    .map_err(storage)?;
            }
            Ok(output)
        })();
        result.map_err(|error| tree_failure(path, recover(error, &mut admission)))
    }
    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        require_first_party(path)?;
        self.locate(path)
            .map(|record| record.map(|(kind, _)| kind))
            .map_err(|error| tree_failure(path, error))
    }
}

fn refused(message: &str) -> LoadError {
    LoadError::new(
        &RelPath::root(),
        None,
        LoadErrorKind::Locked {
            field: "registry-source",
            message: message.to_owned(),
            source: None,
        },
    )
}
fn native_error(source: impl std::error::Error + Send + Sync + 'static) -> LoadError {
    let path = RelPath::root();
    LoadError::tree(
        &path,
        None,
        &crate::Origin::Discovery,
        TreeError::Io {
            path: "registry archive".to_owned(),
            source: io::Error::other(source),
        },
    )
}
fn tree_failure(path: &RelPath, source: LoadError) -> TreeError {
    TreeError::RegistrySource {
        path: path.to_string(),
        source: Box::new(source),
    }
}
