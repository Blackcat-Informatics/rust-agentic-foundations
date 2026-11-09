// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Descriptor-relative staging; callers retain document and ownership policy.

use raf_source::{DiskTree, RelPath, SourceAuthorityId};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    fmt, fs,
    io::{self, Read as _, Seek as _, SeekFrom, Write as _},
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

mod native;

const MAX_BYTES: usize = 64 * 1024 * 1024;
static NEXT: AtomicUsize = AtomicUsize::new(0);

fn next_serial() -> io::Result<usize> {
    let mut current = NEXT.load(Ordering::Relaxed);
    loop {
        let next = current
            .checked_add(1)
            .ok_or_else(|| refused("metadata temporary name counter exhausted"))?;
        match NEXT.compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return Ok(current),
            Err(observed) => current = observed,
        }
    }
}

fn refused(reason: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason.into())
}

fn at(subject: &'static str, path: &str, source: io::Error) -> io::Error {
    io::Error::new(
        source.kind(),
        RefusedPath {
            subject,
            path: path.to_owned(),
            source,
        },
    )
}

#[derive(Debug)]
struct RefusedPath {
    subject: &'static str,
    path: String,
    source: io::Error,
}

impl fmt::Display for RefusedPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} is refused: {}",
            self.subject, self.path, self.source
        )
    }
}

impl std::error::Error for RefusedPath {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// One reader authority, retained from admission through publication.
pub(crate) struct Root {
    path: PathBuf,
    tree: DiskTree,
    directory: Rc<fs::File>,
    parents: RefCell<BTreeMap<RelPath, Rc<fs::File>>>,
}

impl Root {
    pub(crate) fn open(path: &Path) -> io::Result<Rc<Self>> {
        let tree = DiskTree::new(path).map_err(io::Error::other)?;
        let directory = native::root(&tree)?;
        let path = native::root_path(&directory)?;
        // Source rechecks its canonical root against this retained descriptor.
        // The caller's locator is never independently resolved for publication.
        let _verified = native::root(&tree)?;
        Ok(Rc::new(Self {
            path,
            tree,
            directory: Rc::new(directory),
            parents: RefCell::new(BTreeMap::new()),
        }))
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
    pub(crate) fn tree(&self) -> &DiskTree {
        &self.tree
    }

    fn verify(&self, path: &RelPath) -> io::Result<()> {
        if path.authority() != SourceAuthorityId::FIRST_PARTY {
            return Err(refused("metadata publication requires a first-party path"));
        }
        let current = native::root(&self.tree)?;
        if native::identity(&current)? != native::identity(&self.directory)? {
            return Err(refused("metadata root directory identity changed"));
        }
        crate::files::protect_git(&self.path, path).map_err(|error| error.source)
    }

    fn directory(self: &Rc<Self>, path: &RelPath) -> io::Result<Directory> {
        self.verify(path)?;
        let mut current = Rc::clone(&self.directory);
        let mut chain = Vec::new();
        let mut prefix = String::new();
        for name in path.as_str().split('/').filter(|name| !name.is_empty()) {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(name);
            let opened = native::directory(&current, name)
                .map_err(|error| at("the directory", &prefix, error))?;
            let relative = RelPath::new(&prefix).map_err(io::Error::other)?;
            let mut parents = self.parents.borrow_mut();
            if let Some(admitted) = parents.get(&relative) {
                if native::identity(admitted)? != native::identity(&opened)? {
                    return Err(refused(format!(
                        "metadata directory {relative} changed since admission"
                    )));
                }
                current = Rc::clone(admitted);
            } else {
                current = Rc::new(opened);
                parents.insert(relative, Rc::clone(&current));
            }
            chain.push((name.to_owned(), native::identity(&current)?));
        }
        Ok(Directory {
            root: Rc::clone(self),
            path: path.clone(),
            file: current,
            chain,
        })
    }

    /// Retain existing ancestors even when the eventual parent is absent.
    pub(crate) fn admit_parent(self: &Rc<Self>, path: &RelPath) -> io::Result<bool> {
        self.verify(path)?;
        let parent = path.parent().unwrap_or_else(RelPath::root);
        match self.directory(&parent) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub(crate) fn capture(self: &Rc<Self>, path: &RelPath) -> io::Result<Target> {
        self.verify(path)?;
        if path.is_root() {
            return Err(refused("a metadata destination needs a file name"));
        }
        let parent = path.parent().unwrap_or_else(RelPath::root);
        let directory = self
            .directory(&parent)
            .map_err(|error| at("metadata destination", path.as_str(), error))?;
        let name = path
            .as_str()
            .rsplit('/')
            .next()
            .ok_or_else(|| refused("missing file name"))?
            .to_owned();
        let current = observe(&directory.file, &name)
            .map_err(|error| at("metadata destination", path.as_str(), error))?
            .map(|(_, state)| state);
        directory.verify()?;
        Ok(Target {
            directory,
            path: path.clone(),
            name,
            current,
        })
    }

    /// Create only this directory; ancestors must already be admitted directories.
    pub(crate) fn create_directory(self: &Rc<Self>, path: &RelPath) -> io::Result<bool> {
        self.verify(path)?;
        if path.is_root() {
            return Ok(false);
        }
        let parent = path.parent().unwrap_or_else(RelPath::root);
        let directory = self.directory(&parent)?;
        let name = path
            .as_str()
            .rsplit('/')
            .next()
            .ok_or_else(|| refused("missing directory name"))?;
        directory.verify()?;
        match native::directory(&directory.file, name) {
            Ok(_) => {
                directory.verify()?;
                Ok(false)
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                native::mkdir(&directory.file, name)?;
                let _created = native::directory(&directory.file, name)?;
                directory.verify()?;
                Ok(true)
            }
            Err(error) => Err(error),
        }
    }

    /// Retain inode slots and probe actual target checks before creating temporaries.
    /// Two extra slots cover a temporary open and named-file checks. Probing with
    /// those slots held also exercises the current native discovery requirements;
    /// this does not isolate the caller from concurrent host resource changes.
    pub(crate) fn reserve_staging<'a>(
        self: &Rc<Self>,
        count: usize,
        targets: impl Iterator<Item = &'a Target>,
    ) -> io::Result<StagingGrant> {
        let required = if count == 0 {
            0
        } else {
            count
                .checked_add(2)
                .ok_or_else(|| refused("metadata descriptor admission count overflow"))?
        };
        let mut slots = Vec::new();
        for _ in 0..required {
            slots.try_reserve(1).map_err(io::Error::other)?;
            slots.push(self.directory.try_clone().map_err(|error| {
                at(
                    "metadata descriptor admission",
                    &format!(
                        "{count} staged inode slots plus scratch; increase the open-file limit or reduce the batch"
                    ),
                    error,
                )
            })?);
        }
        for target in targets {
            if !Rc::ptr_eq(self, &target.directory.root) {
                return Err(refused("staging target belongs to another root"));
            }
            target.check()?;
        }
        slots.truncate(count);
        Ok(StagingGrant {
            root: Rc::clone(self),
            slots,
        })
    }
}

/// Descriptor reservations are replaced by retained temporary-inode handles.
pub(crate) struct StagingGrant {
    root: Rc<Root>,
    slots: Vec<fs::File>,
}

struct Directory {
    root: Rc<Root>,
    path: RelPath,
    file: Rc<fs::File>,
    chain: Vec<(String, (u64, u64))>,
}

impl Directory {
    fn verify(&self) -> io::Result<()> {
        let current = self.root.directory(&self.path)?;
        if current.chain != self.chain
            || native::identity(&current.file)? != native::identity(&self.file)?
        {
            return Err(refused(
                "metadata directory no longer has its admitted binding",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Snapshot {
    identity: (u64, u64),
    links: u64,
    permissions: fs::Permissions,
    digest: String,
}

fn contents(file: &fs::File) -> io::Result<(Vec<u8>, Snapshot)> {
    let (before, identity, links) = native::stat(file)?;
    if !before.is_file() || links == 0 {
        return Err(refused("metadata files must be regular and remain linked"));
    }
    if before.len() > MAX_BYTES as u64 {
        return Err(refused("metadata file exceeds 64 MiB"));
    }
    let mut bytes = Vec::new();
    let mut borrowed = file;
    borrowed.seek(SeekFrom::Start(0))?;
    borrowed
        .take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    let (after, after_identity, after_links) = native::stat(file)?;
    if bytes.len() > MAX_BYTES
        || before.len() != after.len()
        || before.permissions() != after.permissions()
        || after_identity != identity
        || after_links != links
    {
        return Err(refused("metadata file changed while its bytes were read"));
    }
    // Diagram ownership guards already use the native BLAKE3 byte identity.
    let digest = purrdf_hash::hex::Lower(purrdf_hash::blake3::hash(&bytes).as_bytes()).to_string();
    Ok((
        bytes,
        Snapshot {
            identity,
            links,
            permissions: after.permissions(),
            digest,
        },
    ))
}

fn observe(directory: &fs::File, name: &str) -> io::Result<Option<(Vec<u8>, Snapshot)>> {
    let path = match native::reference(directory, name) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if !path.metadata()?.is_file() {
        return Err(refused("metadata destination is not a regular file"));
    }
    let opened = native::read(&path)?;
    if native::identity(&path)? != native::identity(&opened)? {
        return Err(refused(
            "metadata destination changed before its bytes were opened",
        ));
    }
    contents(&opened).map(Some)
}

/// A destination and its original presence, identity, mode and exact byte digest.
pub(crate) struct Target {
    directory: Directory,
    path: RelPath,
    name: String,
    current: Option<Snapshot>,
}

impl Target {
    pub(crate) fn exists(&self) -> bool {
        self.current.is_some()
    }

    pub(crate) fn verify_content(&self, expected: Option<&str>) -> io::Result<()> {
        let actual = self.current.as_ref().map(|state| state.digest.as_str());
        if actual != expected {
            return Err(refused(format!(
                "metadata destination {} disagrees with its ownership admission",
                self.path
            )));
        }
        Ok(())
    }

    fn check(&self) -> io::Result<()> {
        self.directory.verify()?;
        let current = observe(&self.directory.file, &self.name)
            .map_err(|error| at("metadata destination", self.path.as_str(), error))?
            .map(|(_, state)| state);
        self.directory.verify()?;
        if current != self.current {
            return Err(refused(format!(
                "metadata destination {} changed since admission",
                self.path
            )));
        }
        Ok(())
    }

    pub(crate) fn read_text(&self, limit: usize) -> io::Result<String> {
        self.directory.verify()?;
        let (bytes, state) = observe(&self.directory.file, &self.name)?
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        self.directory.verify()?;
        if Some(&state) != self.current.as_ref() || bytes.len() > limit {
            return Err(refused(
                "metadata bytes changed or exceed their document bound",
            ));
        }
        String::from_utf8(bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    pub(crate) fn remove(&self) -> io::Result<()> {
        self.check()?;
        if !self.exists() {
            return Err(io::Error::from(io::ErrorKind::NotFound));
        }
        native::unlink(&self.directory.file, &self.name)
            .map_err(|error| at("metadata destination", self.path.as_str(), error))
    }

    pub(crate) fn stage(self, text: &str) -> io::Result<Staged> {
        let mut grant = self
            .directory
            .root
            .reserve_staging(1, std::iter::once(&self))?;
        self.stage_reserved(text, &mut grant)
    }

    pub(crate) fn stage_reserved(self, text: &str, grant: &mut StagingGrant) -> io::Result<Staged> {
        if !Rc::ptr_eq(&self.directory.root, &grant.root) || grant.slots.is_empty() {
            return Err(refused("metadata staging needs an admitted inode slot"));
        }
        self.check()?;
        if text.len() > MAX_BYTES {
            return Err(refused("metadata output exceeds 64 MiB"));
        }
        for _ in 0..1000 {
            let serial = next_serial()?;
            let name = format!(".{}.{}.sync.tmp", self.name, serial);
            let mut file = match native::create(&self.directory.file, &name) {
                Ok(file) => file,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(at("metadata temporary", &name, error)),
            };
            // The new file replaces one reserved descriptor, keeping this
            // operation within the capacity exercised before staging began.
            grant.slots.pop();
            let written = (|| {
                // Capture native creation permissions while the file is empty,
                // then make it private before placing any document bytes in it.
                let permissions = self.current.as_ref().map_or_else(
                    || file.metadata().map(|metadata| metadata.permissions()),
                    |current| Ok(current.permissions.clone()),
                )?;
                native::private(&file)?;
                file.write_all(text.as_bytes())?;
                file.set_permissions(permissions.clone())?;
                file.sync_all()?;
                let (_, state) = contents(&file)?;
                if state.links != 1
                    || state.digest != crate::digest::semantic_hash(text)
                    || state.permissions != permissions
                {
                    return Err(refused(
                        "staged metadata disagrees with the intended bytes or mode",
                    ));
                }
                Ok(state)
            })();
            let state = match written {
                Ok(state) => state,
                Err(error) => {
                    return Err(with_cleanup(
                        error,
                        discard_owned(&self.directory, &name, &file),
                    ));
                }
            };
            let temporary = self
                .directory
                .root
                .path
                .join(self.directory.path.as_str())
                .join(&name);
            return Ok(Staged {
                target: self,
                name,
                temporary,
                file,
                state,
            });
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "no unused metadata temporary name after 1000 attempts",
        ))
    }
}

/// A complete staged file. Callers explicitly publish it or report cleanup failures.
pub(crate) struct Staged {
    target: Target,
    name: String,
    temporary: PathBuf,
    file: fs::File,
    state: Snapshot,
}

impl fmt::Debug for Staged {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Staged")
            .field("temporary", &self.temporary)
            .field("target", &self.target.path)
            .finish_non_exhaustive()
    }
}

impl Staged {
    #[cfg(test)]
    pub(crate) fn temporary_path(&self) -> &Path {
        &self.temporary
    }
    pub(crate) fn target_path(&self) -> &RelPath {
        &self.target.path
    }
    pub(crate) fn permissions(&self) -> fs::Permissions {
        self.state.permissions.clone()
    }

    /// An ownership intent replaces the same original manifest before its final text.
    pub(crate) fn follows(&mut self, intent: &Self) -> io::Result<()> {
        if !Rc::ptr_eq(&self.target.directory.root, &intent.target.directory.root)
            || self.target.path != intent.target.path
            || self.target.current != intent.target.current
            || self.target.directory.chain != intent.target.directory.chain
        {
            return Err(refused(
                "ownership intent does not replace the same admitted manifest",
            ));
        }
        self.target.current = Some(intent.state.clone());
        Ok(())
    }

    fn check(&self) -> io::Result<()> {
        self.target.directory.verify()?;
        let named = native::reference(&self.target.directory.file, &self.name)?;
        if native::identity(&named)? != self.state.identity {
            return Err(refused(
                "metadata temporary name no longer identifies this operation's file",
            ));
        }
        let (_, current) = contents(&self.file)?;
        if current != self.state {
            return Err(refused("staged metadata bytes or mode changed"));
        }
        Ok(())
    }

    pub(crate) fn publish(&self) -> io::Result<()> {
        self.target.check()?;
        self.check()?;
        native::rename(
            &self.target.directory.file,
            &self.name,
            &self.target.name,
            !self.target.exists(),
        )
        .map_err(|error| at("metadata destination", self.target.path.as_str(), error))
    }

    pub(crate) fn discard(&self) -> io::Result<()> {
        discard_owned(&self.target.directory, &self.name, &self.file)
    }
}

/// Cleanup owns an inode, not whatever bytes now occupy its former name.
fn discard_owned(directory: &Directory, name: &str, file: &fs::File) -> io::Result<()> {
    directory.verify()?;
    let named = match native::reference(&directory.file, name) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        result => result?,
    };
    if native::identity(&named)? != native::identity(file)? {
        return Err(refused(
            "metadata cleanup refused a replaced temporary file",
        ));
    }
    directory.verify()?;
    native::unlink(&directory.file, name)
}

pub(crate) fn cleanup_after(mut error: io::Error, staged: &[Staged]) -> io::Error {
    for file in staged {
        error = with_cleanup(error, file.discard());
    }
    error
}

pub(crate) fn with_cleanup(primary: io::Error, cleanup: io::Result<()>) -> io::Error {
    match cleanup {
        Ok(()) => primary,
        Err(cleanup) => io::Error::new(primary.kind(), CleanupError { primary, cleanup }),
    }
}

#[derive(Debug)]
struct CleanupError {
    primary: io::Error,
    cleanup: io::Error,
}
impl fmt::Display for CleanupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}; metadata cleanup also failed: {}",
            self.primary, self.cleanup
        )
    }
}
impl std::error::Error for CleanupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.primary)
    }
}
