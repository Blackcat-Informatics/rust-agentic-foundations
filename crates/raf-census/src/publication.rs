// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed Linux debt destinations sharing Source's original directory handle.

use crate::Error;
use raf_source::DiskTree;
use rustix::fs::{AtFlags, CWD, Mode, OFlags, linkat, openat, renameat, statat, unlinkat};
use std::fs::File;
use std::io::{Read as _, Write as _};
use std::os::fd::AsRawFd;
use std::os::unix::fs::MetadataExt as _;
use std::path::PathBuf;

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    WholeBody,
    Embedded,
}

impl Kind {
    pub(crate) const fn target(self) -> &'static str {
        match self {
            Self::WholeBody => crate::DEBT_FILE,
            Self::Embedded => crate::embedded::DEBT_FILE,
        }
    }
    const fn temporary(self) -> &'static str {
        match self {
            Self::WholeBody => ".census-debt.ttl.write",
            Self::Embedded => ".census-embedded-debt.ttl.write",
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum PublicationMode {
    Create,
    Replace,
}

pub(crate) struct Publication<'tree> {
    tree: &'tree DiskTree,
    directory: File,
    kind: Kind,
    #[cfg(test)]
    after_publish: Option<fn(&File, Kind)>,
    #[cfg(test)]
    synchronization_error: Option<rustix::io::Errno>,
}

impl<'tree> Publication<'tree> {
    pub(crate) fn acquire(tree: &'tree DiskTree, kind: Kind) -> Result<Self, Error> {
        let root = tree.directory_handle().map_err(tree_error)?;
        let directory = openat(
            &root,
            ".",
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map(File::from)
        .map_err(|e| io("workspace directory", e.into()))?;
        if identity(&root, "workspace directory")? != identity(&directory, "workspace directory")? {
            return Err(changed("workspace directory"));
        }
        match directory.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err(io(
                    "workspace directory",
                    std::io::Error::new(
                        std::io::ErrorKind::WouldBlock,
                        "debt publication is busy; retry after the other writer ends",
                    ),
                ));
            }
            Err(std::fs::TryLockError::Error(error)) => {
                return Err(io(
                    "workspace directory lock (exclusive directory locking support required)",
                    error,
                ));
            }
        }
        let publication = Self {
            tree,
            directory,
            kind,
            #[cfg(test)]
            after_publish: None,
            #[cfg(test)]
            synchronization_error: None,
        };
        publication.validate()?;
        Ok(publication)
    }

    pub(crate) fn exists(&self) -> Result<bool, Error> {
        match statat(
            &self.directory,
            self.kind.target(),
            AtFlags::SYMLINK_NOFOLLOW,
        ) {
            Ok(_) => Ok(true),
            Err(rustix::io::Errno::NOENT) => Ok(false),
            Err(error) => Err(io(self.kind.target(), error.into())),
        }
    }

    /// Read exactly the no-follow regular record pinned below this directory.
    /// The byte ceiling is input admission, not a native parser heap guarantee.
    pub(crate) fn read_existing(&self) -> Result<Option<String>, Error> {
        let name = self.kind.target();
        let Some(reference) = regular_reference(&self.directory, name)? else {
            return Ok(None);
        };
        let file = reopen(&reference, name)?;
        let mut text = String::new();
        file.take(crate::MAX_DEBT_BYTES as u64 + 1)
            .read_to_string(&mut text)
            .map_err(|error| io(name, error))?;
        if text.len() > crate::MAX_DEBT_BYTES {
            return Err(Error::Options(format!(
                "{name}: debt exceeds its input byte limit"
            )));
        }
        self.check_named(name, &reference)?;
        Ok(Some(text))
    }

    pub(crate) fn publish(&self, text: &str, mode: PublicationMode) -> Result<(), Error> {
        let temporary = self.kind.temporary();
        let file = openat(
            &self.directory,
            temporary,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o666),
        )
        .map(File::from)
        .map_err(|error| io(temporary, error.into()))?;
        let prepare = (|| {
            (&file)
                .write_all(text.as_bytes())
                .map_err(|error| io(temporary, error))?;
            file.sync_all().map_err(|error| io(temporary, error))?;
            self.validate()?;
            self.check_named(temporary, &file)?;
            match mode {
                PublicationMode::Create => linkat(
                    &self.directory,
                    temporary,
                    &self.directory,
                    self.kind.target(),
                    AtFlags::empty(),
                ),
                PublicationMode::Replace => renameat(
                    &self.directory,
                    temporary,
                    &self.directory,
                    self.kind.target(),
                ),
            }
            .map_err(|error| io(self.kind.target(), error.into()))
        })();
        if let Err(error) = prepare {
            return Err(match self.discard_owned(temporary, &file) {
                Ok(()) => error,
                Err(cleanup) => compound(error, cleanup),
            });
        }
        #[cfg(test)]
        if let Some(after_publish) = self.after_publish {
            after_publish(&self.directory, self.kind);
        }
        let cleanup = match mode {
            PublicationMode::Create => self.discard_owned(temporary, &file),
            PublicationMode::Replace => Ok(()),
        };
        let synchronized = self.synchronize();
        let (stage, error) = match (cleanup, synchronized) {
            (Ok(()), Ok(())) => return Ok(()),
            (Err(error), Ok(())) => ("temporary cleanup", error),
            (Ok(()), Err(error)) => ("directory synchronization", error),
            (Err(error), Err(sync)) => (
                "temporary cleanup and directory synchronization",
                compound(error, sync),
            ),
        };
        Err(Error::Published {
            file: self.kind.target().into(),
            stage,
            source: Box::new(error),
        })
    }

    fn validate(&self) -> Result<(), Error> {
        drop(self.tree.directory_handle().map_err(tree_error)?);
        Ok(())
    }

    fn synchronize(&self) -> Result<(), Error> {
        #[cfg(test)]
        if let Some(error) = self.synchronization_error {
            return Err(io("workspace directory", error.into()));
        }
        self.directory
            .sync_all()
            .map_err(|error| io("workspace directory", error))
    }

    fn check_named(&self, name: &str, file: &File) -> Result<(), Error> {
        let named = statat(&self.directory, name, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|error| io(name, error.into()))?;
        if (named.st_dev, named.st_ino) == identity(file, name)? {
            Ok(())
        } else {
            Err(changed(name))
        }
    }

    fn discard_owned(&self, name: &str, file: &File) -> Result<(), Error> {
        self.check_named(name, file)?;
        unlinkat(&self.directory, name, AtFlags::empty()).map_err(|error| io(name, error.into()))
    }
}

fn regular_reference(directory: &File, name: &str) -> Result<Option<File>, Error> {
    let file = match openat(
        directory,
        name,
        OFlags::PATH | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    ) {
        Ok(file) => File::from(file),
        Err(rustix::io::Errno::NOENT) => return Ok(None),
        Err(error) => return Err(io(name, error.into())),
    };
    if !file.metadata().map_err(|error| io(name, error))?.is_file() {
        return Err(Error::Options(format!(
            "{name}: expected a regular metadata file; resolve the unexpected entry"
        )));
    }
    Ok(Some(file))
}

fn reopen(reference: &File, name: &str) -> Result<File, Error> {
    let path = PathBuf::from(format!("/proc/self/fd/{}", reference.as_raw_fd()));
    let file = openat(CWD, path, OFlags::RDONLY | OFlags::CLOEXEC, Mode::empty())
        .map(File::from)
        .map_err(|error| io(name, error.into()))?;
    if identity(reference, name)? != identity(&file, name)? {
        return Err(changed(name));
    }
    Ok(file)
}

fn identity(file: &File, name: &str) -> Result<(u64, u64), Error> {
    let metadata = file.metadata().map_err(|error| io(name, error))?;
    Ok((metadata.dev(), metadata.ino()))
}

fn io(file: &str, source: std::io::Error) -> Error {
    Error::Io {
        file: file.into(),
        source,
    }
}

fn tree_error(source: raf_source::TreeError) -> Error {
    Error::Tree {
        file: "workspace directory".into(),
        source,
    }
}

fn changed(name: &str) -> Error {
    Error::Options(format!(
        "{name}: metadata identity changed; inspect the destination before retrying"
    ))
}

fn compound(error: Error, cleanup: Error) -> Error {
    Error::Compound {
        primary: Box::new(error),
        secondary: Box::new(cleanup),
    }
}

#[cfg(test)]
mod tests {
    use super::{Kind, Publication, PublicationMode};
    use crate::Error;
    use raf_source::DiskTree;
    use rustix::fs::{AtFlags, Mode, OFlags, openat, unlinkat};
    use std::fs::{self, File};
    use std::io::Write as _;

    fn replace_temporary(directory: &File, kind: Kind) {
        unlinkat(directory, kind.temporary(), AtFlags::empty()).unwrap();
        let replacement = File::from(
            openat(
                directory,
                kind.temporary(),
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::from_raw_mode(0o600),
            )
            .unwrap(),
        );
        (&replacement).write_all(b"unowned replacement").unwrap();
    }

    #[test]
    fn published_records_and_unowned_temporaries_survive_typed_finalization_failures() {
        for create in [true, false] {
            let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
            let tree = DiskTree::new(scratch.path()).unwrap();
            let kind = if create {
                Kind::Embedded
            } else {
                Kind::WholeBody
            };
            let mode = if create {
                PublicationMode::Create
            } else {
                PublicationMode::Replace
            };
            let target = scratch.path().join(kind.target());
            if !create {
                fs::write(&target, "previous complete record").unwrap();
            }
            let mut publication = Publication::acquire(&tree, kind).unwrap();
            publication.synchronization_error = Some(rustix::io::Errno::IO);
            if create {
                publication.after_publish = Some(replace_temporary);
            }
            let error = publication
                .publish("fresh complete record", mode)
                .unwrap_err();
            assert_eq!(
                fs::read_to_string(&target).unwrap(),
                "fresh complete record"
            );
            let Error::Published { source, .. } = error else {
                panic!("publication must remain distinguishable from a pre-publication failure");
            };
            if create {
                let Error::Compound { primary, secondary } = *source else {
                    panic!("both finalization causes must remain typed");
                };
                assert!(matches!(*primary, Error::Options(_)));
                assert!(
                    matches!(*secondary, Error::Io { source, .. } if source.raw_os_error() == Some(rustix::io::Errno::IO.raw_os_error()))
                );
                assert_eq!(
                    fs::read(scratch.path().join(kind.temporary())).unwrap(),
                    b"unowned replacement"
                );
            } else {
                assert!(
                    matches!(*source, Error::Io { source, .. } if source.raw_os_error() == Some(rustix::io::Errno::IO.raw_os_error()))
                );
                assert!(!scratch.path().join(kind.temporary()).exists());
            }
        }
    }
}
