// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native mount view: copied inputs at logical paths and authentic Git administration.

mod layout;
mod objects;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use purrdf_lex::walk::try_equal_range_by;

use super::git_context::OBJECTS;
use super::{CapturedView, GitContext, ViewError};
use crate::exec::{ExecError, Invocation, Limits, Output, Program, Runner, SystemRunner};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FilesystemView {
    pub original: PathBuf,
    pub copied: PathBuf,
    pub common: PathBuf,
    pub admin: PathBuf,
    pub exposed_common: PathBuf,
    pub exposed_admin: PathBuf,
    outputs: Vec<(PathBuf, PathBuf)>,
    aliases: Vec<(PathBuf, PathBuf)>,
    object_mounts: Vec<OsString>,
    layout: Vec<OsString>,
}

impl FilesystemView {
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
        let mut admission = Resident;
        let mut memory = Memory::new(&mut admission);
        let mut roots = OrderedTable::default();
        for output in &selection.outputs {
            let output = super::host::output_root(output)?;
            let admitted = try_equal_range_by(&selection.outputs, |path| {
                Ok::<_, ViewError>(path.cmp(&output))
            })?;
            if admitted.is_empty() {
                return Err(ViewError::refusal(
                    "prepare filesystem output",
                    format!(
                        "{} no longer resolves to an admitted output root",
                        output.display()
                    ),
                )
                .into());
            }
            if roots.get(&output).is_some() {
                continue;
            }
            let backing = match std::fs::metadata(&output) {
                Ok(_) => output.clone(),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    let backing = admin.join(format!("raf-execution-output-{}", roots.len()));
                    std::fs::create_dir(&backing)
                        .map_err(|e| ViewError::new("create owned execution output", e))?;
                    backing
                }
                Err(error) => {
                    return Err(ViewError::new("inspect declared execution output", error).into());
                }
            };
            roots
                .insert(output, backing, &mut memory)
                .map_err(|e| ViewError::new("index resolved output mounts", e))?;
        }
        let mut outputs = Vec::new();
        memory
            .extend(
                &mut outputs,
                roots
                    .ordered()
                    .map(|(root, backing)| (backing.clone(), root.clone())),
            )
            .map_err(|e| ViewError::new("retain resolved output mounts", e))?;
        let aliases: Vec<_> = view
            .capture
            .output_aliases
            .iter()
            .map(|(path, target)| (target.clone(), view.capture.root.join(path.as_str())))
            .collect();
        let exposed_admin = view.capture.root.join(".git");
        let exposed_common = exposed_admin.join("raf-shared-common");
        std::fs::create_dir(admin.join("raf-shared-common"))
            .map_err(|e| ViewError::new("reserve process Git common mount", e))?;
        pointer(&admin, "raf-process-commondir", "raf-shared-common\n")?;
        pointer(
            &admin,
            "raf-process-gitdir",
            &format!("{}\n", exposed_admin.display()),
        )?;
        let mut exceptions = vec![selection.common.clone(), copied.to_owned()];
        exceptions.extend(outputs.iter().map(|(_, target)| target.clone()));
        exceptions.extend(aliases.iter().map(|(_, path)| path.clone()));
        let layout = layout::copied(view, copied, &exceptions)?;
        let object_mounts = objects::layout(
            &admin.join(OBJECTS),
            &exposed_common.join("objects"),
            &exposed_admin.join(OBJECTS),
            &selection.object_directories,
        )?;
        Ok(Self {
            original: view.capture.root.clone(),
            copied: copied.to_path_buf(),
            common: selection.common.clone(),
            admin,
            exposed_common,
            exposed_admin,
            outputs,
            aliases,
            object_mounts,
            layout,
        })
    }

    pub(crate) fn command(
        &self,
        directory: &Path,
        program: &Path,
        argv: &[String],
    ) -> Result<Vec<OsString>, ViewError> {
        // These are fresh path observations, not an atomic filesystem snapshot.
        // Bubblewrap may create an absent external cache mountpoint on the host;
        // only its private backing belongs to execution-storage cleanup.
        for (_, root) in &self.outputs {
            let observed = super::host::output_root(root)
                .map_err(|e| ViewError::new("revalidate process output root", e))?;
            if observed != *root {
                return Err(ViewError::refusal(
                    "revalidate process output root",
                    format!("{} resolves to {}", root.display(), observed.display()),
                ));
            }
        }
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
        memory
            .extend(
                args,
                [
                    OsString::from("--bind"),
                    self.admin.as_os_str().to_owned(),
                    self.exposed_admin.as_os_str().to_owned(),
                    OsString::from("--ro-bind"),
                    self.common.as_os_str().to_owned(),
                    self.exposed_common.as_os_str().to_owned(),
                ],
            )
            .map_err(|e| ViewError::new("allocate native process argv", e))?;
        if !self.common.starts_with(&self.original) {
            memory
                .extend(
                    args,
                    [
                        OsString::from("--ro-bind"),
                        self.common.as_os_str().to_owned(),
                        self.common.as_os_str().to_owned(),
                    ],
                )
                .map_err(|e| ViewError::new("protect original Git administration", e))?;
        }
        memory
            .extend(args, self.object_mounts.iter().cloned())
            .map_err(|e| ViewError::new("project private Git object authority", e))?;
        for (backing, target) in [
            ("raf-process-commondir", "commondir"),
            ("raf-process-gitdir", "gitdir"),
        ] {
            for alias in [backing, target] {
                memory
                    .extend(
                        args,
                        [
                            OsString::from("--ro-bind"),
                            self.admin.join(backing).into_os_string(),
                            self.exposed_admin.join(alias).into_os_string(),
                        ],
                    )
                    .map_err(|e| ViewError::new("project read-only Git namespace pointer", e))?;
            }
        }
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

/// Create a namespace-only pointer without replacing physical Git metadata.
fn pointer(admin: &Path, name: &str, text: &str) -> Result<(), ViewError> {
    let path = raf_source::RelPath::new(name)
        .map_err(|e| ViewError::new("process Git pointer path", e))?;
    crate::files::create_new(admin, "", &path, text).map_err(|error| match error {
        crate::files::CreateError::Exists => ViewError::refusal(
            "reserve process Git pointer",
            format!("{} already exists", admin.join(name).display()),
        ),
        crate::files::CreateError::Write { path, source } => {
            ViewError::new(format!("write process Git pointer {path}"), source)
        }
    })
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
