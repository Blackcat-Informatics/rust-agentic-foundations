// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Git may cache a tree into its index; observe the original through an owned copy.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use raf_source::{CapturedFile, DiskTree, EntryKind, FileId, RelPath, TreeError};

use super::super::capture::MAX_FILE_BYTES;
use super::super::git_context::{GitContext, InGit, OBJECTS};
use super::query;
use crate::exec::{ExecError, Program, Runner, ViewError};

pub(super) struct IndexSnapshot {
    pub tree: Vec<u8>,
    pub original: Option<Arc<CapturedFile>>,
    pub bytes: Option<Arc<[u8]>>,
}

pub(super) fn identity(bytes: Vec<u8>) -> Result<String, ExecError> {
    let index = super::text(bytes, "read index tree")?.trim().to_string();
    if index.len() != 40 || !index.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ViewError::refusal(
            "capture index",
            "Git did not return one full SHA-1 tree identifier",
        )
        .into());
    }
    Ok(index)
}

pub(super) fn route<T>(
    runner: &dyn Runner,
    root: &Path,
    admin: &Path,
    common: &Path,
    index: &Path,
    authorities: &[PathBuf],
    action: impl FnOnce(&dyn Runner) -> Result<T, ExecError>,
) -> Result<T, ExecError> {
    let copy = IndexCopy::new(admin)?;
    let result = (|| {
        let objects = copy.path.join(OBJECTS);
        super::objects::create(&objects, authorities)?;
        let scoped = InGit {
            runner,
            context: GitContext::isolated(admin, common, root, index, &objects),
        };
        action(&scoped)
    })();
    finish(result, copy.release())
}

pub(super) fn tree(
    runner: &dyn Runner,
    root: &Path,
    admin: &Path,
    common: &Path,
    original: &Path,
    authorities: &[PathBuf],
) -> Result<IndexSnapshot, ExecError> {
    let parent = original
        .parent()
        .ok_or_else(|| ViewError::refusal("capture index", "index has no parent"))?;
    let name = original
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| ViewError::refusal("capture index", "index name is not UTF-8"))?;
    let path = RelPath::new(name).map_err(|e| ViewError::new("capture effective index path", e))?;
    let source =
        DiskTree::new(parent).map_err(|e| ViewError::new("open effective index authority", e))?;
    let file = match source.capture_file(&path, MAX_FILE_BYTES) {
        Ok(file) => Some(Arc::new(file)),
        Err(TreeError::NotFound { .. }) => None,
        Err(error) => return Err(ViewError::new("capture effective index bytes", error).into()),
    };
    let copy = IndexCopy::new(admin)?;
    let result = (|| {
        let index = copy.path.join("index");
        if let Some(file) = &file {
            let mut output = fs::File::create_new(&index)
                .map_err(|e| ViewError::new("create owned index", e))?;
            output
                .write_all(&file.bytes)
                .map_err(|e| ViewError::new("copy captured index bytes", e))?;
        }
        let objects = copy.path.join(OBJECTS);
        super::objects::create(&objects, authorities)?;
        let scoped = InGit {
            runner,
            context: GitContext::isolated(admin, common, root, &index, &objects),
        };
        query(
            &scoped,
            root,
            Program::Git,
            &[
                "-c",
                "splitIndex.sharedIndexExpire=never",
                "update-index",
                "--no-split-index",
            ],
        )?;
        let flattened = DiskTree::new(&copy.path)
            .map_err(|e| ViewError::new("open flattened index authority", e))?;
        let bytes = match flattened.capture_file(
            &RelPath::new("index").map_err(|e| ViewError::new("flattened index path", e))?,
            MAX_FILE_BYTES,
        ) {
            Ok(file) => Some(Arc::<[u8]>::from(file.bytes.as_ref())),
            Err(TreeError::NotFound { .. }) => None,
            Err(error) => return Err(ViewError::new("capture flattened index bytes", error).into()),
        };
        let tree = query(&scoped, root, Program::Git, &["write-tree"])?;
        Ok(IndexSnapshot {
            tree,
            original: file,
            bytes,
        })
    })();
    finish(result, copy.release())
}

struct IndexCopy {
    path: PathBuf,
    identity: Option<FileId>,
}

impl IndexCopy {
    fn new(admin: &Path) -> Result<Self, ExecError> {
        static NEXT: Mutex<u64> = Mutex::new(0);
        for _ in 0..128 {
            let serial = {
                let mut next = NEXT
                    .lock()
                    .map_err(|e| ViewError::refusal("reserve index observation", e.to_string()))?;
                let serial = *next;
                *next = next.checked_add(1).ok_or_else(|| {
                    ViewError::refusal("reserve index observation", "identity space exhausted")
                })?;
                serial
            };
            let path = admin.join(format!(
                "raf-index-observation-{}-{serial}",
                std::process::id()
            ));
            match super::create_private(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(ViewError::new("reserve owned index directory", error).into());
                }
            }
            let mut copy = Self {
                path,
                identity: None,
            };
            let identity = DiskTree::new(&copy.path)
                .and_then(|tree| tree.capture_directory_bounded(&RelPath::root(), 0))
                .map_err(|e| ExecError::from(ViewError::new("identify owned index directory", e)));
            return match identity {
                Ok(directory) => {
                    copy.identity = Some(directory.id);
                    Ok(copy)
                }
                Err(primary) => finish(Err(primary), copy.release()),
            };
        }
        Err(ViewError::refusal(
            "reserve index observation",
            "no unused owned directory name",
        )
        .into())
    }

    fn release(&self) -> Result<(), ExecError> {
        let Some(identity) = &self.identity else {
            return super::unidentified(&self.path);
        };
        let directory = DiskTree::new(&self.path)
            .and_then(|tree| tree.capture_directory_bounded(&RelPath::root(), 3))
            .map_err(|e| ViewError::new("verify owned index before cleanup", e))?;
        if directory.id != *identity
            || directory
                .entries
                .iter()
                .any(|entry| match entry.name.as_str() {
                    "index" | "index.lock" => entry.kind != EntryKind::File,
                    OBJECTS => entry.kind != EntryKind::Dir,
                    _ => true,
                })
        {
            return Err(ViewError::refusal(
                "release owned index",
                "directory identity or owned membership changed; refusing removal",
            )
            .into());
        }
        fs::remove_dir_all(&self.path)
            .map_err(|e| ViewError::new("release owned index directory", e).into())
    }
}

fn finish<T>(result: Result<T, ExecError>, cleanup: Result<(), ExecError>) -> Result<T, ExecError> {
    match (result, cleanup) {
        (result, Ok(())) => result,
        (Ok(_), Err(cleanup)) => Err(cleanup),
        (Err(primary), Err(cleanup)) => Err(ExecError::ViewCleanup {
            primary: Box::new(primary),
            cleanup: Box::new(cleanup),
        }),
    }
}
