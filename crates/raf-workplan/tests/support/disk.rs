// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A repository on a disk: a scratch directory, the files of a fixture written
//! under it, a snapshot of every byte below it, and the loader.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use raf_source::{DiskTree, Workspace};
use raf_workplan::{LinkContext, Workplan};

use super::{Repo, demo_files};

/// A unique directory under the system temporary directory, removed on drop.
pub(crate) struct Scratch(PathBuf);

impl Scratch {
    pub(crate) fn new(tag: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "raf-workplan-{tag}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path)
            .unwrap_or_else(|e| panic!("cannot create {}: {e}", path.display()));
        Self(path)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }

    /// The directory a real `cargo` run in this scratch directory builds into:
    /// a sibling of it, so that the build writes nothing below the tree that is
    /// loaded, committed and snapshotted, and removed with it.
    pub(crate) fn target(&self) -> PathBuf {
        let mut name = self.0.file_name().expect("a named directory").to_owned();
        name.push("-target");
        self.0.with_file_name(name)
    }

    /// The text of a `.cargo/config.toml` that gives this scratch directory
    /// its own target and build directories. Two fixtures that name the same
    /// package at the same version and path relative to their root share a
    /// build fingerprint, so a shared directory lets one fixture's `cargo`
    /// rewrite, lock or run an executable another one is using.
    pub(crate) fn cargo_config(&self) -> String {
        let target = self.target();
        format!(
            "[build]\ntarget-dir = '{0}'\nbuild-dir = '{0}'\n",
            target.display()
        )
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // The directories are under the temporary directory and only litter.
        let _ = fs::remove_dir_all(&self.0);
        let _ = fs::remove_dir_all(self.target());
    }
}

/// Write `text` at `path` below `root`, creating the directories.
pub(crate) fn put(root: &Path, path: &str, text: &str) {
    let target = root.join(path);
    fs::create_dir_all(target.parent().expect("a parent"))
        .unwrap_or_else(|e| panic!("cannot create the directory of {path}: {e}"));
    fs::write(&target, text).unwrap_or_else(|e| panic!("cannot write {path}: {e}"));
}

/// Write the demo workspace and `repo` under `root`.
pub(crate) fn write_repo(root: &Path, repo: &Repo) {
    for (path, text) in demo_files() {
        put(root, path, text);
    }
    for (path, text) in repo {
        put(root, path, text);
    }
}

/// The text of the file `path` below `root`.
pub(crate) fn read(root: &Path, path: &str) -> String {
    fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("cannot read {path}: {e}"))
}

/// Every file below `root`: its path with `/` separators and its bytes. A
/// symbolic link is listed by its target text and is not followed.
pub(crate) fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, into: &mut BTreeMap<String, Vec<u8>>) {
        for entry in
            fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()))
        {
            let entry = entry.expect("an entry");
            let path = entry.path();
            let name = path
                .strip_prefix(root)
                .expect("below the root")
                .to_string_lossy()
                .replace('\\', "/");
            let kind = entry.file_type().expect("a file type");
            if kind.is_symlink() {
                let target = fs::read_link(&path).expect("a link target");
                into.insert(
                    format!("{name} -> link"),
                    target.to_string_lossy().into_owned().into_bytes(),
                );
            } else if kind.is_dir() {
                walk(root, &path, into);
            } else {
                into.insert(name, fs::read(&path).expect("file bytes"));
            }
        }
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files);
    files
}

/// The paths in `after` that are not in `before` or hold other bytes there.
pub(crate) fn changed(
    before: &BTreeMap<String, Vec<u8>>,
    after: &BTreeMap<String, Vec<u8>>,
) -> Vec<String> {
    let mut paths: Vec<String> = after
        .iter()
        .filter(|(path, bytes)| before.get(*path) != Some(*bytes))
        .map(|(path, _)| path.clone())
        .collect();
    paths.extend(
        before
            .keys()
            .filter(|path| !after.contains_key(*path))
            .cloned(),
    );
    paths.sort();
    paths
}

/// Load the documents under `root` from the disk and run `f` on the plan.
pub(crate) fn on_disk<R>(root: &Path, f: impl FnOnce(&Workplan<'_>) -> R) -> R {
    let tree =
        DiskTree::new(root).unwrap_or_else(|e| panic!("cannot open {}: {e}", root.display()));
    let source = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("workspace: {e}"));
    f(&Workplan::load_with(
        &tree,
        &source,
        &LinkContext::of_tree(&tree),
    ))
}
