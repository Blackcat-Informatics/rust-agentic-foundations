// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Creating a new file below a root, exclusively.

use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::Path;

use raf_source::{DiskTree, EntryKind, RelPath, Tree};

/// Why a file was not created.
#[derive(Debug)]
pub(crate) enum CreateError {
    /// A file, or a link, is already at the path. It is untouched.
    Exists,
    /// The file could not be written, or a directory it needs is refused.
    Write {
        /// The path that was being written or judged.
        path: String,
        source: io::Error,
    },
}

fn to_io(error: raf_source::TreeError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, error)
}

fn write_error(path: &str, source: io::Error) -> CreateError {
    CreateError::Write {
        path: path.to_string(),
        source,
    }
}

/// Create `path` below `root` with `text`, exclusively.
///
/// `path` lies directly in `directory`, a `/`-separated path below `root`.
/// Each directory on the way to `directory` must be a directory or not exist;
/// a symbolic link or a file there is refused, and a missing directory is
/// created. The tree of `root` judges each of them, so none leads outside the
/// root. The file is created with `create_new`, so an existing file or link
/// at the path is [`CreateError::Exists`] and is not touched. A failure after
/// the file was created removes it again.
pub(crate) fn create_new(
    root: &Path,
    directory: &str,
    path: &RelPath,
    text: &str,
) -> Result<(), CreateError> {
    let tree = DiskTree::new(root).map_err(|error| write_error(path.as_str(), to_io(error)))?;
    let mut missing = false;
    let mut prefix = String::new();
    for part in directory.split('/') {
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(part);
        let rel = RelPath::new(&prefix).map_err(|error| {
            write_error(&prefix, io::Error::new(io::ErrorKind::InvalidInput, error))
        })?;
        match tree.entry(&rel) {
            Ok(Some(EntryKind::Dir)) => {}
            Ok(None) => missing = true,
            Ok(Some(_)) => {
                return Err(write_error(
                    &prefix,
                    io::Error::new(io::ErrorKind::InvalidInput, "it is not a directory"),
                ));
            }
            Err(error) => return Err(write_error(&prefix, to_io(error))),
        }
    }
    if missing {
        fs::create_dir_all(root.join(directory)).map_err(|error| write_error(directory, error))?;
    }
    let target = root.join(path.as_str());
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
    {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            return Err(CreateError::Exists);
        }
        Err(error) => return Err(write_error(path.as_str(), error)),
    };
    let written = file
        .write_all(text.as_bytes())
        .and_then(|()| file.sync_all());
    if let Err(error) = written {
        drop(file);
        // The file was created by this call just above; nothing else is
        // removed.
        let _ = fs::remove_file(&target);
        return Err(write_error(path.as_str(), error));
    }
    Ok(())
}
