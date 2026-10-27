// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native mount view: copied inputs at logical paths and authentic Git administration.

mod layout;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use purrdf_lex::allocation::{Memory, Resident};

use super::{CapturedView, GitContext, ViewError};
use crate::exec::{ExecError, Invocation, Limits, Output, Program, Runner, SystemRunner};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FilesystemView {
    pub original: PathBuf,
    pub copied: PathBuf,
    pub common: PathBuf,
    pub admin: PathBuf,
    outputs: Vec<(PathBuf, PathBuf)>,
    aliases: Vec<(PathBuf, PathBuf)>,
    layout: Vec<OsString>,
}

impl FilesystemView {
    pub(crate) fn git(&self) -> GitContext {
        GitContext::owned(&self.admin, &self.common, &self.original)
    }
    fn host_git(&self) -> GitContext {
        GitContext::owned(&self.admin, &self.common, &self.copied)
    }
    pub(super) fn logical(&self, physical: &Path) -> Result<PathBuf, ViewError> {
        let suffix = physical
            .strip_prefix(&self.copied)
            .map_err(|e| ViewError::new("process directory outside owned copy", e))?;
        Ok(self.original.join(suffix))
    }
    pub(super) fn new(
        view: &CapturedView,
        copied: &Path,
        admin: PathBuf,
    ) -> Result<Self, ExecError> {
        let selection = view.capture.selection.as_ref().ok_or_else(|| {
            ViewError::refusal("prepare filesystem view", "missing native Git selection")
        })?;
        if !admin.starts_with(&selection.common) || admin == selection.common {
            return Err(ViewError::refusal(
                "prepare filesystem view",
                "owned Git administration is outside the observed common directory",
            )
            .into());
        }
        let mut outputs = Vec::new();
        for (ordinal, output) in selection.outputs.iter().enumerate() {
            if !output.starts_with(&view.capture.root) {
                continue;
            }
            let backing = match std::fs::metadata(output) {
                Ok(_) => output.clone(),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    let backing = admin.join(format!("raf-execution-output-{ordinal}"));
                    std::fs::create_dir(&backing)
                        .map_err(|e| ViewError::new("create owned execution output", e))?;
                    backing
                }
                Err(error) => {
                    return Err(ViewError::new("inspect declared execution output", error).into());
                }
            };
            outputs.push((backing, output.clone()));
        }
        let aliases: Vec<_> = view
            .capture
            .output_aliases
            .iter()
            .map(|(path, target)| (target.clone(), view.capture.root.join(path.as_str())))
            .collect();
        let mut exceptions = vec![selection.common.clone(), copied.to_owned()];
        exceptions.extend(outputs.iter().map(|(_, target)| target.clone()));
        exceptions.extend(aliases.iter().map(|(_, path)| path.clone()));
        let layout = layout::copied(view, copied, &exceptions)?;
        Ok(Self {
            original: view.capture.root.clone(),
            copied: copied.to_path_buf(),
            common: selection.common.clone(),
            admin,
            outputs,
            aliases,
            layout,
        })
    }

    pub(crate) fn command(
        &self,
        directory: &Path,
        program: &Path,
        argv: &[String],
    ) -> Result<Vec<OsString>, ViewError> {
        let mut admission = Resident;
        let mut memory = Memory::new(&mut admission);
        let mut words = Vec::new();
        memory
            .extend(
                &mut words,
                ["--dev-bind", "/", "/"].into_iter().map(OsString::from),
            )
            .map_err(|e| ViewError::new("allocate process filesystem argv", e))?;
        memory
            .extend(&mut words, self.layout.iter().cloned())
            .map_err(|e| ViewError::new("allocate process layout argv", e))?;
        self.mounts(&mut words, &mut memory)?;
        memory
            .extend(
                &mut words,
                [
                    OsString::from("--remount-ro"),
                    self.original.as_os_str().to_owned(),
                    OsString::from("--die-with-parent"),
                    OsString::from("--chdir"),
                    self.logical(directory)?.into_os_string(),
                    OsString::from("--"),
                ],
            )
            .map_err(|e| ViewError::new("allocate native process argv", e))?;
        memory
            .push(&mut words, program.as_os_str().to_owned())
            .map_err(|e| ViewError::new("allocate native program argv", e))?;
        memory
            .extend(&mut words, argv.iter().map(OsString::from))
            .map_err(|e| ViewError::new("allocate native caller argv", e))?;
        Ok(words)
    }
    fn mounts(
        &self,
        args: &mut Vec<OsString>,
        memory: &mut Memory<'_, Resident>,
    ) -> Result<(), ViewError> {
        for (source, target) in &self.outputs {
            memory
                .extend(
                    args,
                    [
                        OsString::from("--bind"),
                        source.as_os_str().to_owned(),
                        target.as_os_str().to_owned(),
                    ],
                )
                .map_err(|e| ViewError::new("allocate native process argv", e))?;
        }
        if !self.common.starts_with(&self.original) {
            memory
                .extend(
                    args,
                    [
                        OsString::from("--ro-bind"),
                        self.copied.join(".git").into_os_string(),
                        self.original.join(".git").into_os_string(),
                    ],
                )
                .map_err(|e| ViewError::new("allocate native process argv", e))?;
        }
        memory
            .extend(
                args,
                [
                    OsString::from("--bind"),
                    self.copied.as_os_str().to_owned(),
                    self.copied.as_os_str().to_owned(),
                    OsString::from("--ro-bind"),
                    self.common.as_os_str().to_owned(),
                    self.common.as_os_str().to_owned(),
                    OsString::from("--bind"),
                    self.admin.as_os_str().to_owned(),
                    self.admin.as_os_str().to_owned(),
                ],
            )
            .map_err(|e| ViewError::new("allocate native process argv", e))?;
        for (target, path) in &self.aliases {
            memory
                .extend(
                    args,
                    [
                        OsString::from("--symlink"),
                        target.as_os_str().to_owned(),
                        path.as_os_str().to_owned(),
                    ],
                )
                .map_err(|e| ViewError::new("allocate native process argv", e))?;
        }
        Ok(())
    }
}

pub(super) struct InFilesystem<'a> {
    pub runner: &'a SystemRunner,
    pub view: Arc<FilesystemView>,
}
impl Runner for InFilesystem<'_> {
    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError> {
        if invocation.program() == Program::Cargo {
            self.runner
                .run(&invocation.clone().in_filesystem(self.view.clone()), limits)
        } else {
            self.runner
                .run(&invocation.clone().in_git(self.view.host_git()), limits)
        }
    }
}
