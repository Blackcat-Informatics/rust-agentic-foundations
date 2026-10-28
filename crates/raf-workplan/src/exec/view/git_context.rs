// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Explicit owned Git routing; original observations retain the caller's selectors.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::exec::{ExecError, GitSelector, Invocation, Limits, Output, Runner};

pub(super) const OBJECTS: &str = "raf-owned-objects";
pub(super) const OBJECT_READS: &str = "raf-object-reads";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum GitContext {
    /// Discover a newly registered worktree through its real `.git` locator.
    Discover,
    /// All descendants use this worktree and its own index, irrespective of
    /// inherited selectors for the original repository.
    Selected {
        admin: PathBuf,
        common: PathBuf,
        root: PathBuf,
        index: PathBuf,
        objects: PathBuf,
    },
}

impl GitContext {
    pub(crate) const fn selector(&self) -> Option<GitSelector> {
        match self {
            Self::Discover => None,
            Self::Selected { .. } => Some(GitSelector::Directory),
        }
    }
    pub(crate) fn apply(&self, command: &mut Command) {
        for name in [
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_COMMON_DIR",
            "GIT_INDEX_FILE",
            "GIT_PREFIX",
            "GIT_OBJECT_DIRECTORY",
            "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        ] {
            command.env_remove(name);
        }
        if let Self::Selected {
            admin,
            common,
            root,
            index,
            objects,
        } = self
        {
            command
                .env("GIT_DIR", admin)
                .env("GIT_WORK_TREE", root)
                .env("GIT_COMMON_DIR", common)
                .env("GIT_INDEX_FILE", index)
                .env("GIT_OBJECT_DIRECTORY", objects);
        }
    }

    pub(crate) fn owned(admin: &Path, common: &Path, root: &Path) -> Self {
        Self::isolated(
            admin,
            common,
            root,
            &admin.join("index"),
            &admin.join(OBJECTS),
        )
    }

    pub(crate) fn isolated(
        admin: &Path,
        common: &Path,
        root: &Path,
        index: &Path,
        objects: &Path,
    ) -> Self {
        Self::Selected {
            admin: admin.to_owned(),
            common: common.to_owned(),
            root: root.to_owned(),
            index: index.to_owned(),
            objects: objects.to_owned(),
        }
    }
}

pub(super) struct InGit<'a> {
    pub runner: &'a dyn Runner,
    pub context: GitContext,
}

impl Runner for InGit<'_> {
    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError> {
        self.runner
            .run(&invocation.clone().in_git(self.context.clone()), limits)
    }
}
