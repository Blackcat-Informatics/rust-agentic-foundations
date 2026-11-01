// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Owned installed-Git materialization for the real supplied runner.

mod index;
mod materialize;
mod objects;
mod selection;

pub(super) use selection::output_root;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use raf_source::{DiskTree, FileId, RelPath};

use super::capture::Capture;
use super::filesystem::{FilesystemView, InFilesystem};
use super::git_context::{GitContext, InGit};
use super::{CapturedView, PreparedView, ViewContext, ViewError, ViewRequest};
use crate::exec::{ExecError, Invocation, Limits, Program, Runner, SystemRunner};

pub(super) fn create_private(path: &Path) -> std::io::Result<()> {
    let mut directory = std::fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        directory.mode(0o700);
    }
    directory.create(path)
}

#[derive(Debug)]
struct Lease {
    context: ViewContext,
    destination: PathBuf,
    identity: FileId,
    parent: Option<(PathBuf, FileId)>,
    baseline: Capture,
    copied_request: ViewRequest,
    filesystem: Arc<FilesystemView>,
}

struct CreatedPath {
    path: PathBuf,
    identity: Option<FileId>,
}
#[derive(Default)]
struct OwnedResources {
    parent: Option<CreatedPath>,
    destination: Option<CreatedPath>,
}
impl OwnedResources {
    fn release(&self, runner: &SystemRunner, root: &Path) -> Result<(), ExecError> {
        if let Some(created) = &self.destination {
            if let Some(identity) = &created.identity {
                materialize::release(runner, root, &created.path, identity)?;
            } else {
                unidentified(&created.path)?;
            }
        }
        if let Some(created) = &self.parent {
            if let Some(identity) = &created.identity {
                remove_parent(Some(&(created.path.clone(), identity.clone())))?;
            } else {
                unidentified(&created.path)?;
            }
        }
        Ok(())
    }
}
fn unidentified(path: &Path) -> Result<(), ExecError> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ViewError::new(format!("inspect reserved {}", path.display()), error).into()),
        Ok(_) => Err(ViewError::refusal("release reserved resource", format!("{} was created but its identity could not be established; refusing unverified removal", path.display())).into()),
    }
}

struct Reservation {
    selection: super::capture::Selection,
    flattened_index: Option<Arc<[u8]>>,
    excluded: Vec<PathBuf>,
    destination: PathBuf,
    slot: u64,
    identity: FileId,
    parent: Option<(PathBuf, FileId)>,
}

/// Shared ownership survives an explicit clone of a configured `SystemRunner`.
#[derive(Debug, Default)]
pub(crate) struct HostViews {
    owner: Arc<u8>,
    next: Mutex<u64>,
    leases: Mutex<Vec<Lease>>,
}

impl HostViews {
    pub(crate) fn filesystem(
        &self,
        context: &ViewContext,
    ) -> Result<Arc<FilesystemView>, ExecError> {
        self.directory(context)?;
        self.leases
            .lock()
            .map_err(|e| ViewError::refusal("resolve filesystem view", e.to_string()))?
            .iter()
            .find(|lease| lease.context == *context)
            .map(|lease| lease.filesystem.clone())
            .ok_or_else(|| {
                ViewError::refusal("resolve filesystem view", "unknown or released context").into()
            })
    }
    fn owner(&self) -> Result<u64, ExecError> {
        u64::try_from(Arc::as_ptr(&self.owner) as usize)
            .map_err(|e| ViewError::new("runner view identity", e).into())
    }
    pub(crate) fn directory(&self, context: &ViewContext) -> Result<PathBuf, ExecError> {
        if context.owner != self.owner()? {
            return Err(ViewError::refusal(
                "run prepared view",
                "context belongs to another runner",
            )
            .into());
        }
        let leases = self
            .leases
            .lock()
            .map_err(|e| ViewError::refusal("run prepared view", e.to_string()))?;
        let lease = leases
            .iter()
            .find(|lease| lease.context == *context)
            .ok_or_else(|| {
                ViewError::refusal(
                    "run prepared view",
                    "context is unknown or already released",
                )
            })?;
        context
            .physical_root
            .clone()
            .filter(|path| path.starts_with(&lease.destination))
            .ok_or_else(|| {
                ViewError::refusal(
                    "run prepared view",
                    "physical root does not belong to the lease",
                )
                .into()
            })
    }
    fn slot(&self) -> Result<u64, ExecError> {
        let mut next = self
            .next
            .lock()
            .map_err(|e| ViewError::refusal("allocate runner context", e.to_string()))?;
        let slot = *next;
        *next = next
            .checked_add(1)
            .ok_or_else(|| ViewError::refusal("prepare view", "runner context space exhausted"))?;
        Ok(slot)
    }
    pub(crate) fn prepare(
        &self,
        runner: &SystemRunner,
        request: &ViewRequest,
    ) -> Result<PreparedView, ExecError> {
        let slot = self.slot()?;
        let owner = self.owner()?;
        let selection::Observation {
            selection,
            mut excluded,
            flattened_index,
        } = selection::observe(runner, request, None, None, &[])?;
        let parent = selection
            .common
            .parent()
            .ok_or_else(|| {
                ViewError::refusal(
                    "prepare view",
                    "Git common directory has no repository parent",
                )
            })?
            .join(".worktrees");
        if !selection
            .worktrees
            .iter()
            .any(|root| root.join(".worktrees") == parent)
        {
            return Err(ViewError::refusal(
                "admit canonical execution parent",
                "Git common-directory parent is not a registered primary worktree",
            )
            .into());
        }
        let mut resources = OwnedResources::default();
        let result = (|| {
            let created_parent = match std::fs::create_dir(&parent) {
                Ok(()) => {
                    resources.parent = Some(CreatedPath {
                        path: parent.clone(),
                        identity: None,
                    });
                    true
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => false,
                Err(error) => return Err(ViewError::new("create owned view parent", error).into()),
            };
            let main = parent.parent().ok_or_else(|| {
                ViewError::refusal("prepare view parent", "missing canonical repository root")
            })?;
            let path =
                RelPath::new(".worktrees").map_err(|e| ViewError::new("owned parent path", e))?;
            let observed = DiskTree::new(main)
                .and_then(|tree| tree.capture_directory_bounded(&path, super::capture::MAX_ENTRIES))
                .map_err(|e| ViewError::new("verify execution parent", e))?;
            let parent_identity = observed.id;
            if let Some(created) = &mut resources.parent {
                created.identity = Some(parent_identity.clone());
            }
            let destination = parent.join(format!("workplan-completed-{owner}-{slot}"));
            create_private(&destination)
                .map_err(|e| ViewError::new("reserve owned execution root", e))?;
            resources.destination = Some(CreatedPath {
                path: destination.clone(),
                identity: None,
            });
            let identity = DiskTree::new(&destination)
                .and_then(|tree| {
                    tree.capture_directory_bounded(&RelPath::root(), super::capture::MAX_ENTRIES)
                })
                .map_err(|e| ViewError::new("identify owned execution root", e))?
                .id;
            if let Some(created) = &mut resources.destination {
                created.identity = Some(identity.clone());
            }
            let owned_parent = created_parent.then(|| (parent.clone(), parent_identity));
            excluded.push(if created_parent {
                parent.clone()
            } else {
                destination.clone()
            });
            self.prepare_at(
                runner,
                request,
                Reservation {
                    selection,
                    flattened_index,
                    excluded,
                    destination,
                    slot,
                    identity,
                    parent: owned_parent,
                },
            )
        })();
        match result {
            Ok(prepared) => Ok(prepared),
            Err(primary) => match resources.release(runner, request.root()) {
                Ok(()) => Err(primary),
                Err(cleanup) => Err(ExecError::ViewCleanup {
                    primary: Box::new(primary),
                    cleanup: Box::new(cleanup),
                }),
            },
        }
    }
    fn prepare_at(
        &self,
        runner: &SystemRunner,
        request: &ViewRequest,
        reservation: Reservation,
    ) -> Result<PreparedView, ExecError> {
        let Reservation {
            selection,
            flattened_index,
            excluded,
            destination,
            slot,
            identity,
            parent,
        } = reservation;
        let capture = selection::capture(runner, request, selection, excluded)?;
        let view = Arc::new(CapturedView {
            request: request.clone(),
            capture,
            flattened_index,
        });
        let modes = materialize::write(runner, &view, &destination)?;
        let discovery = InGit {
            runner,
            context: GitContext::Discover,
        };
        let admin = selection::git_path(&discovery, &destination, "--absolute-git-dir")?;
        let filesystem = Arc::new(FilesystemView::new(&view, &destination, admin)?);
        let scoped = InFilesystem {
            runner,
            view: filesystem.clone(),
        };
        let copied_request = ViewRequest {
            root: destination.join(view.capture.prefix.as_str()),
            tracked: view.tracked()?,
            required: request
                .required
                .iter()
                .filter(|path| !request.removes(path))
                .cloned()
                .chain(request.rewritten.iter().map(|(path, _)| path.clone()))
                .collect(),
            ..request.clone()
        };
        let workspaces = view
            .capture
            .selection
            .as_ref()
            .ok_or_else(|| {
                ViewError::refusal("prepare copied metadata", "missing original selection")
            })?
            .workspaces
            .as_slice();
        let selection::Observation {
            selection: copied_selection,
            excluded: copied_excluded,
            ..
        } = selection::observe(
            &scoped,
            &copied_request,
            None,
            Some(&filesystem),
            workspaces,
        )?;
        let baseline = Capture::read(&copied_request, &copied_excluded, Some(copied_selection))?;
        view.verify_copy(&baseline, &modes)?;

        let context = ViewContext {
            owner: self.owner()?,
            authority: self.owner.clone(),
            slot,
            digest: view.digest().to_owned(),
            logical_root: view.capture.root.join(view.capture.prefix.as_str()),
            physical_root: Some(destination.join(view.capture.prefix.as_str())),
            proof: super::ViewProof::of(&view),
            filesystem: Some(filesystem.clone()),
        };
        self.leases
            .lock()
            .map_err(|e| ViewError::refusal("register owned view", e.to_string()))?
            .push(Lease {
                context: context.clone(),
                destination: destination.clone(),
                identity,
                parent,
                baseline,
                copied_request,
                filesystem,
            });
        Ok(PreparedView { context, view })
    }
    pub(crate) fn validate(
        &self,
        runner: &SystemRunner,
        context: &ViewContext,
    ) -> Result<(), ExecError> {
        self.directory(context)?;
        {
            let leases = self
                .leases
                .lock()
                .map_err(|e| ViewError::refusal("revalidate copied inputs", e.to_string()))?;
            let lease = leases
                .iter()
                .find(|lease| lease.context == *context)
                .ok_or_else(|| ViewError::refusal("revalidate copied inputs", "unknown context"))?;
            let scoped = InFilesystem {
                runner,
                view: lease.filesystem.clone(),
            };
            let selection::Observation {
                selection,
                excluded,
                ..
            } = selection::observe(
                &scoped,
                &lease.copied_request,
                None,
                Some(&lease.filesystem),
                &lease
                    .baseline
                    .selection
                    .as_ref()
                    .ok_or_else(|| {
                        ViewError::refusal("revalidate copied metadata", "missing copied selection")
                    })?
                    .workspaces,
            )?;
            let actual = Capture::read(&lease.copied_request, &excluded, Some(selection))?;
            if actual != lease.baseline {
                return Err(lease
                    .baseline
                    .changed(&actual, "revalidate completed physical view")
                    .into());
            }
        }
        Ok(())
    }
    pub(crate) fn revalidate(
        &self,
        runner: &SystemRunner,
        prepared: &PreparedView,
    ) -> Result<(), ExecError> {
        self.directory(&prepared.context)?;
        self.validate(runner, &prepared.context)?;
        let (destination, own_exclusion) = {
            let leases = self
                .leases
                .lock()
                .map_err(|e| ViewError::refusal("revalidate owned view", e.to_string()))?;
            let lease = leases
                .iter()
                .find(|lease| lease.context == prepared.context)
                .ok_or_else(|| ViewError::refusal("revalidate owned view", "unknown context"))?;
            (
                lease.destination.clone(),
                lease
                    .parent
                    .as_ref()
                    .map_or_else(|| lease.destination.clone(), |(parent, _)| parent.clone()),
            )
        };
        let selection::Observation {
            selection,
            mut excluded,
            ..
        } = selection::observe(
            runner,
            &prepared.view.request,
            Some(&destination),
            None,
            &prepared
                .view
                .capture
                .selection
                .as_ref()
                .ok_or_else(|| {
                    ViewError::refusal("revalidate original metadata", "missing original selection")
                })?
                .workspaces,
        )?;
        excluded.push(own_exclusion);
        let actual = Capture::read(&prepared.view.request, &excluded, Some(selection))?;
        if actual != prepared.view.capture {
            return Err(prepared
                .view
                .capture
                .changed(&actual, "revalidate original repository")
                .into());
        }
        Ok(())
    }
    pub(crate) fn finish(
        &self,
        runner: &SystemRunner,
        prepared: &PreparedView,
    ) -> Result<(), ExecError> {
        self.directory(&prepared.context)?;
        let mut leases = self
            .leases
            .lock()
            .map_err(|e| ViewError::refusal("release owned view", e.to_string()))?;
        let index = leases
            .iter()
            .position(|lease| lease.context == prepared.context)
            .ok_or_else(|| ViewError::refusal("release owned view", "unknown context"))?;
        let lease = &leases[index];
        materialize::release(
            runner,
            prepared.view.request.root(),
            &lease.destination,
            &lease.identity,
        )?;
        remove_parent(lease.parent.as_ref())?;
        leases.remove(index);
        Ok(())
    }
}

fn query(
    runner: &dyn Runner,
    directory: &Path,
    program: Program,
    args: &[&str],
) -> Result<Vec<u8>, ExecError> {
    let args: Vec<String> = args.iter().map(|value| (*value).to_string()).collect();
    command(runner, directory, program, &args)
}
fn command(
    runner: &dyn Runner,
    directory: &Path,
    program: Program,
    args: &[String],
) -> Result<Vec<u8>, ExecError> {
    let invocation = Invocation::host(program, args, directory);
    let output = runner.run(&invocation, &Limits::query())?;
    if !output.success() {
        return Err(ViewError::refusal(
            &invocation.to_string(),
            format!(
                "exit {:?}: {}",
                output.code,
                String::from_utf8_lossy(&output.stderr)
            ),
        )
        .into());
    }
    Ok(output.stdout)
}
fn text(bytes: Vec<u8>, operation: &str) -> Result<String, ExecError> {
    String::from_utf8(bytes).map_err(|e| ViewError::new(operation, e).into())
}

fn remove_parent(parent: Option<&(PathBuf, FileId)>) -> Result<(), ExecError> {
    if let Some((path, identity)) = parent {
        let actual = DiskTree::new(path)
            .and_then(|tree| {
                tree.capture_directory_bounded(&RelPath::root(), super::capture::MAX_ENTRIES)
            })
            .map_err(|e| ViewError::new("verify owned execution parent", e))?;
        if actual.id != *identity || !actual.entries.is_empty() {
            return Err(ViewError::refusal(
                "release owned execution parent",
                "directory changed or another user added entries; refusing removal",
            )
            .into());
        }
        std::fs::remove_dir(path)
            .map_err(|e| ViewError::new("release owned execution parent", e))?;
    }
    Ok(())
}
