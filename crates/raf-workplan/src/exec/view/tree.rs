// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Tree policy over immutable native captures; host acquisition stays in Source.

use purrdf_lex::walk::WorkList;
use std::path::{Component, Path};

use raf_source::{Entry, EntryKind, FileId, Opened, RelPath, SourceAuthorityId, Tree, TreeError};

use super::CapturedView;
use super::capture::{Capture, Node};
use crate::staged::Staged;

fn missing(path: &RelPath) -> TreeError {
    TreeError::NotFound {
        path: path.to_string(),
    }
}
fn outside(path: &RelPath) -> TreeError {
    TreeError::OutsideRoot {
        path: path.to_string(),
    }
}
fn regular(path: &RelPath) -> TreeError {
    TreeError::NotRegular {
        path: path.to_string(),
    }
}

impl Capture {
    fn resolve(&self, path: &RelPath, final_link: bool) -> Result<RelPath, TreeError> {
        self.resolve_from(path, final_link, &self.prefix)
    }

    pub(super) fn retained_input(&self, path: &RelPath) -> Result<(), TreeError> {
        let target = self.resolve_from(path, true, &RelPath::root())?;
        match self.get(&target) {
            Some(Node::File(_)) => Ok(()),
            None => Err(missing(path)),
            Some(_) => Err(regular(path)),
        }
    }

    fn resolve_from(
        &self,
        path: &RelPath,
        final_link: bool,
        prefix: &RelPath,
    ) -> Result<RelPath, TreeError> {
        if path.authority() != SourceAuthorityId::FIRST_PARTY {
            return Err(TreeError::WrongAuthority {
                path: path.to_string(),
                authority: path.authority(),
            });
        }
        let mut parts: Vec<String> = prefix.segments().map(str::to_owned).collect();
        let floor = parts.len();
        let mut pending: WorkList<String, 16> = WorkList::new();
        let initial: Vec<&str> = path.segments().collect();
        pending.extend(initial.into_iter().rev().map(str::to_owned));
        let mut links = 0;
        while let Some(part) = pending.pop() {
            if part == "." || part.is_empty() {
                continue;
            }
            if part == ".." {
                if parts.len() == floor {
                    return Err(outside(path));
                }
                parts.pop();
                continue;
            }
            parts.push(part);
            let current = RelPath::new(&parts.join("/")).map_err(|_| outside(path))?;
            if let Some(Node::Link(link)) = self.get(&current)
                && (!pending.is_empty() || final_link)
            {
                links += 1;
                if links > 40 {
                    return Err(TreeError::SymlinkLoop {
                        path: path.to_string(),
                    });
                }
                parts.pop();
                let target = if link.target.is_absolute() {
                    let root = self.root.join(prefix.as_str());
                    let relative = link.target.strip_prefix(&root).map_err(|_| outside(path))?;
                    parts.truncate(floor);
                    relative
                } else {
                    link.target.as_path()
                };
                let target = components(target, path)?;
                for component in target.into_iter().rev() {
                    pending.push(component);
                }
            } else if !pending.is_empty() {
                match self.get(&current) {
                    Some(Node::Dir(_)) => {}
                    None => return Err(missing(path)),
                    Some(_) => return Err(regular(path)),
                }
            }
        }
        RelPath::new(&parts.join("/")).map_err(|_| outside(path))
    }
}

fn components(target: &Path, path: &RelPath) -> Result<Vec<String>, TreeError> {
    target
        .components()
        .map(|part| match part {
            Component::Normal(name) => {
                name.to_str()
                    .map(str::to_owned)
                    .ok_or_else(|| TreeError::NonUtf8Name {
                        path: path.to_string(),
                        name: name.to_string_lossy().into_owned(),
                    })
            }
            Component::ParentDir => Ok("..".to_string()),
            Component::CurDir => Ok(".".to_string()),
            _ => Err(outside(path)),
        })
        .collect()
}

impl Tree for Capture {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        match self.read_unseen(path, max_bytes, &|_| false)? {
            Opened::Read { text, .. } => Ok(text),
            _ => Err(regular(path)),
        }
    }
    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        let target = self.resolve(path, true)?;
        let Some(Node::File(file)) = self.get(&target) else {
            return Err(if self.get(&target).is_some() {
                regular(path)
            } else {
                missing(path)
            });
        };
        if seen(&file.id) {
            return Ok(Opened::Seen(file.id.clone()));
        }
        if !u64::try_from(file.bytes.len()).is_ok_and(|size| size <= max_bytes) {
            return Err(TreeError::TooLarge {
                path: path.to_string(),
                limit: max_bytes,
            });
        }
        let text = std::str::from_utf8(&file.bytes).map_err(|_| TreeError::NotUtf8 {
            path: path.to_string(),
        })?;
        Ok(Opened::Read {
            id: file.id.clone(),
            text: text.to_string(),
        })
    }
    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        let target = self.resolve(path, false)?;
        match self.get(&target) {
            Some(Node::Dir(dir)) => Ok(dir.entries.clone()),
            Some(_) => Err(regular(path)),
            None => Err(missing(path)),
        }
    }
    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        let target = match self.resolve(path, false) {
            Ok(target) => target,
            Err(TreeError::NotFound { .. }) => return Ok(None),
            Err(error) => return Err(error),
        };
        Ok(self.get(&target).map(|node| match node {
            Node::File(_) => EntryKind::File,
            Node::Dir(_) => EntryKind::Dir,
            Node::Link(_) => EntryKind::Symlink,
        }))
    }
}

impl CapturedView {
    fn staged(&self) -> Staged<'_> {
        Staged::new(&self.capture, &self.request.hidden, &self.request.rewritten)
            .with_removed(&self.request.removed)
    }
}
impl Tree for CapturedView {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        self.staged().read(path, max_bytes)
    }
    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        self.staged().read_unseen(path, max_bytes, seen)
    }
    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        self.staged().list(path)
    }
    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        self.staged().entry(path)
    }
}
