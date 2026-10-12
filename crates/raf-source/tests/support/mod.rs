// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Fixtures shared by the disk-tree test binaries.

#![allow(
    dead_code,
    reason = "each test binary uses a different subset of the helpers"
)]

pub(crate) mod child;
pub(crate) mod shapes;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// How long an operation may take before the test calls it blocked.
const WATCHDOG: Duration = Duration::from_secs(10);

/// A fresh directory under the system temporary directory, removed on drop.
pub(crate) struct Scratch {
    base: PathBuf,
}

impl Scratch {
    pub(crate) fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let base = std::env::temp_dir().join(format!(
            "raf-source-it-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&base).expect("create scratch directory");
        Self {
            base: fs::canonicalize(&base).expect("canonicalise scratch directory"),
        }
    }

    /// The scratch directory itself.
    pub(crate) fn base(&self) -> &Path {
        &self.base
    }

    /// `name` under the scratch directory.
    pub(crate) fn path(&self, name: &str) -> PathBuf {
        self.base.join(name)
    }

    /// Create the directory `name` and any parents.
    pub(crate) fn dir(&self, name: &str) -> PathBuf {
        let path = self.path(name);
        fs::create_dir_all(&path).expect("create directory");
        path
    }

    /// Write `text` to `name`, creating parents.
    pub(crate) fn file(&self, name: &str, text: &str) -> PathBuf {
        let path = self.path(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(&path, text).expect("write file");
        path
    }

    /// Make `name` a symlink to `target`.
    #[cfg(unix)]
    pub(crate) fn link(&self, name: &str, target: impl AsRef<Path>) -> PathBuf {
        let path = self.path(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        std::os::unix::fs::symlink(target, &path).expect("create symlink");
        path
    }

    /// Make `name` a FIFO. Opening one for a blocking read waits for a
    /// writer, so a test that finishes within the watchdog never did.
    #[cfg(unix)]
    pub(crate) fn fifo(&self, name: &str) -> PathBuf {
        let path = self.path(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        let status = std::process::Command::new("mkfifo")
            .arg(&path)
            .status()
            .expect("run mkfifo");
        assert!(status.success(), "mkfifo failed for {}", path.display());
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // Teardown of a temporary directory; a leftover fails nothing.
        drop(fs::remove_dir_all(&self.base));
    }
}

/// Run `work` on its own thread and return its result, failing the test when
/// it has not finished within the watchdog time.
pub(crate) fn within_watchdog<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        // The receiver is gone only when the watchdog already fired.
        drop(sender.send(work()));
    });
    receiver
        .recv_timeout(WATCHDOG)
        .expect("the operation blocked past the watchdog")
}
