// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One retained Unix directory identity and descriptor-only ancestry checks.

#[cfg(target_os = "linux")]
use std::os::fd::AsFd;
use std::os::fd::OwnedFd;
use std::path::Path;

use rustix::fs::{CWD, Mode, OFlags, Stat, fstat, openat};

use super::outside;
use crate::TreeError;

const CLIMB_LIMIT: usize = 4096;

pub(super) struct Root {
    pub(super) dir: OwnedFd,
    status: Stat,
}

impl Root {
    pub(super) fn open(path: &Path, shown: &str) -> Result<Self, TreeError> {
        let dir = openat(
            CWD,
            path,
            directory_flags() | OFlags::NOFOLLOW,
            Mode::empty(),
        )
        .map_err(|e| TreeError::from_io(shown, e.into()))?;
        Self::from_open(dir, shown)
    }

    #[cfg(target_os = "linux")]
    pub(super) fn open_at(
        directory: impl AsFd,
        path: &Path,
        shown: &str,
    ) -> Result<Self, TreeError> {
        let dir = openat(directory, path, directory_flags(), Mode::empty())
            .map_err(|e| TreeError::from_io(shown, e.into()))?;
        Self::from_open(dir, shown)
    }

    fn from_open(dir: OwnedFd, shown: &str) -> Result<Self, TreeError> {
        let status = fstat(&dir).map_err(|e| TreeError::from_io(shown, e.into()))?;
        Ok(Self { dir, status })
    }

    #[cfg(target_os = "linux")]
    pub(super) fn identity(&self) -> crate::FileId {
        crate::FileId::from_device(self.status.st_dev, self.status.st_ino)
    }

    pub(super) fn is(&self, status: &Stat) -> bool {
        same(status, &self.status)
    }
}

pub(super) fn same(left: &Stat, right: &Stat) -> bool {
    left.st_dev == right.st_dev && left.st_ino == right.st_ino
}

fn directory_flags() -> OFlags {
    #[cfg(target_os = "linux")]
    let access = OFlags::PATH;
    #[cfg(not(target_os = "linux"))]
    let access = OFlags::RDONLY;
    access | OFlags::DIRECTORY | OFlags::CLOEXEC
}

/// Reach the original root through directory descriptions, never its locator.
pub(super) fn climb(mut current: OwnedFd, root: &Root, shown: &str) -> Result<(), TreeError> {
    let mut status = fstat(&current).map_err(|_| outside(shown))?;
    for _ in 0..CLIMB_LIMIT {
        if root.is(&status) {
            return Ok(());
        }
        let up =
            openat(&current, "..", directory_flags(), Mode::empty()).map_err(|_| outside(shown))?;
        let above = fstat(&up).map_err(|_| outside(shown))?;
        if same(&above, &status) {
            return Err(outside(shown));
        }
        current = up;
        status = above;
    }
    Err(outside(shown))
}
