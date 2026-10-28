// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Read-only object aliases retain the native alternate closure without recursion.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use purrdf_lex::allocation::{Memory, Resident};
use raf_source::{CapturedDirectory, DiskTree, RelPath};

use super::super::capture::MAX_ENTRIES;
use super::super::git_context::OBJECT_READS;
use super::ViewError;

pub(super) fn layout(
    private: &Path,
    primary: &Path,
    secondary: &Path,
    sources: &[PathBuf],
) -> Result<Vec<OsString>, ViewError> {
    layout_with_budget(
        private,
        primary,
        secondary,
        sources,
        Budget {
            entries: MAX_ENTRIES,
            bytes: crate::exec::Limits::query().max_output(),
        },
    )
}

struct Budget {
    entries: usize,
    bytes: usize,
}

impl Budget {
    fn capture(
        &mut self,
        tree: &DiskTree,
        operation: &str,
    ) -> Result<CapturedDirectory, ViewError> {
        let directory = tree
            .capture_directory_bounded(&RelPath::root(), self.entries)
            .map_err(|e| ViewError::new(operation, e))?;
        self.entries -= directory.entries.len();
        Ok(directory)
    }

    fn charge<const N: usize>(&mut self, values: [&OsStr; N]) -> Result<(), ViewError> {
        let size = values.iter().try_fold(0_usize, |sum, value| {
            sum.checked_add(value.len())
                .and_then(|sum| sum.checked_add(1))
        });
        self.bytes = size
            .and_then(|size| self.bytes.checked_sub(size))
            .ok_or_else(|| {
                ViewError::refusal(
                    "admit Git object view",
                    "aggregate mount metadata byte limit exceeded; reduce Git object authorities",
                )
            })?;
        Ok(())
    }
}

fn layout_with_budget(
    private: &Path,
    primary: &Path,
    secondary: &Path,
    sources: &[PathBuf],
    mut budget: Budget,
) -> Result<Vec<OsString>, ViewError> {
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut args = Vec::new();
    for (target, flag) in [(primary, "--bind"), (secondary, "--ro-bind")] {
        mount(&mut args, flag, private, target, &mut memory, &mut budget)?;
        mount(
            &mut args,
            "--ro-bind",
            &private.join("info"),
            &target.join("info"),
            &mut memory,
            &mut budget,
        )?;
        let reads = target.join(OBJECT_READS);
        budget.charge([
            OsStr::new("--perms"),
            OsStr::new("0700"),
            OsStr::new("--tmpfs"),
            reads.as_os_str(),
        ])?;
        memory
            .extend(
                &mut args,
                [
                    OsString::from("--perms"),
                    OsString::from("0700"),
                    OsString::from("--tmpfs"),
                    reads.as_os_str().to_owned(),
                ],
            )
            .map_err(|e| ViewError::new("reserve namespace object read aliases", e))?;
        for (ordinal, source) in sources.iter().enumerate() {
            let alias = reads.join(ordinal.to_string());
            directory(&mut args, "--dir", &alias, &mut memory, &mut budget)?;
            let tree = DiskTree::new(source)
                .map_err(|e| ViewError::new("open original Git object authority", e))?;
            let observed = budget.capture(&tree, "observe original Git object membership")?;
            for entry in observed.entries.iter().filter(|entry| entry.name != "info") {
                mount(
                    &mut args,
                    "--ro-bind",
                    &source.join(&entry.name),
                    &alias.join(&entry.name),
                    &mut memory,
                    &mut budget,
                )?;
            }
            let info = alias.join("info");
            directory(&mut args, "--dir", &info, &mut memory, &mut budget)?;
            match std::fs::metadata(source.join("info")) {
                Ok(_) => {
                    let original = source.join("info");
                    let tree = DiskTree::new(&original)
                        .map_err(|e| ViewError::new("open Git object read metadata", e))?;
                    let observed = budget.capture(&tree, "observe Git object read metadata")?;
                    for entry in observed
                        .entries
                        .iter()
                        .filter(|entry| entry.name != "alternates")
                    {
                        mount(
                            &mut args,
                            "--ro-bind",
                            &original.join(&entry.name),
                            &info.join(&entry.name),
                            &mut memory,
                            &mut budget,
                        )?;
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(ViewError::new("inspect Git object read metadata", error));
                }
            }
        }
        directory(&mut args, "--remount-ro", &reads, &mut memory, &mut budget)?;
    }
    Ok(args)
}

fn directory(
    args: &mut Vec<OsString>,
    flag: &str,
    path: &Path,
    memory: &mut Memory<'_, Resident>,
    budget: &mut Budget,
) -> Result<(), ViewError> {
    budget.charge([OsStr::new(flag), path.as_os_str()])?;
    memory
        .extend(args, [OsString::from(flag), path.as_os_str().to_owned()])
        .map_err(|e| ViewError::new("project Git object read directory", e))
}

fn mount(
    args: &mut Vec<OsString>,
    flag: &'static str,
    source: &Path,
    target: &Path,
    memory: &mut Memory<'_, Resident>,
    budget: &mut Budget,
) -> Result<(), ViewError> {
    budget.charge([OsStr::new(flag), source.as_os_str(), target.as_os_str()])?;
    super::layout::mount(args, flag, source, target, memory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_read_layout_refuses_aggregate_growth_beside_individually_admitted_stores() {
        let root = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
        for name in ["one", "two", "owned"] {
            let mut command = std::process::Command::new("git");
            crate::exec::view::git_context::GitContext::Discover.apply(&mut command);
            let output = command
                .args(["init", "-q", "--bare", "--template="])
                .arg(root.path().join(name))
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let sources = [
            root.path().join("one/objects"),
            root.path().join("two/objects"),
        ];
        let private = root.path().join("owned/objects");
        let primary = root.path().join("visible/objects");
        let secondary = root.path().join("visible/owned");
        let mut single_bytes = 0;
        for source in &sources {
            let args = layout_with_budget(
                &private,
                &primary,
                &secondary,
                std::slice::from_ref(source),
                Budget {
                    entries: 4,
                    bytes: 64 * 1024,
                },
            )
            .unwrap();
            single_bytes = single_bytes.max(args.iter().map(|arg| arg.len() + 1).sum());
        }
        let entries = layout_with_budget(
            &private,
            &primary,
            &secondary,
            &sources,
            Budget {
                entries: 4,
                bytes: 64 * 1024,
            },
        )
        .unwrap_err();
        assert!(
            entries
                .to_string()
                .contains("observe original Git object membership"),
            "{entries}"
        );
        let bytes = layout_with_budget(
            &private,
            &primary,
            &secondary,
            &sources,
            Budget {
                entries: 8,
                bytes: single_bytes,
            },
        )
        .unwrap_err();
        assert!(
            bytes
                .to_string()
                .contains("aggregate mount metadata byte limit"),
            "{bytes}"
        );
        let roomy = layout_with_budget(
            &private,
            &primary,
            &secondary,
            &sources,
            Budget {
                entries: 8,
                bytes: 64 * 1024,
            },
        )
        .unwrap();
        for source in &sources {
            assert!(
                roomy
                    .iter()
                    .any(|arg| arg.as_os_str() == source.join("pack").as_os_str())
            );
        }
    }
}
