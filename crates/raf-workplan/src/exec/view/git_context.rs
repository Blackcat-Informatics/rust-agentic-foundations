// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Explicit owned Git routing; original observations retain the caller's selectors.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::exec::{ExecError, GitSelector, Invocation, Limits, Output, Runner};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum GitContext {
    /// Discover a newly registered worktree through its real `.git` locator.
    Discover,
    /// All descendants use this worktree and its own index, irrespective of
    /// inherited selectors for the original repository.
    Owned {
        admin: PathBuf,
        common: PathBuf,
        root: PathBuf,
        index: PathBuf,
    },
}

impl GitContext {
    pub(crate) const fn selector(&self) -> Option<GitSelector> {
        match self {
            Self::Discover => None,
            Self::Owned { .. } => Some(GitSelector::Directory),
        }
    }
    pub(crate) fn apply(&self, command: &mut Command) {
        for name in [
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_COMMON_DIR",
            "GIT_INDEX_FILE",
        ] {
            command.env_remove(name);
        }
        if let Self::Owned {
            admin,
            common,
            root,
            index,
        } = self
        {
            command
                .env("GIT_DIR", admin)
                .env("GIT_WORK_TREE", root)
                .env("GIT_COMMON_DIR", common)
                .env("GIT_INDEX_FILE", index);
        }
    }

    pub(crate) fn selected(admin: &Path, common: &Path, root: &Path, index: &Path) -> Self {
        Self::Owned {
            admin: admin.to_owned(),
            common: common.to_owned(),
            root: root.to_owned(),
            index: index.to_owned(),
        }
    }

    pub(crate) fn owned(admin: &Path, common: &Path, root: &Path) -> Self {
        Self::selected(admin, common, root, &admin.join("index"))
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
