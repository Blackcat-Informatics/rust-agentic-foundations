// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A tree as it will be once some writes are done, without doing them: one
//! file not shown, and other files with new text.

use raf_source::{Entry, EntryKind, FileId, Opened, RelPath, Tree, TreeError};

/// `inner` without the file `hidden`, and with the files of `replaced` holding
/// their new text.
///
/// The hidden file is not read, not listed and not found by `entry`. A
/// replaced file is read as its new text, with an identity of its own; it must
/// already exist or be an admitted generated addition. Replacement keys retain
/// their source authority; every other path is answered by `inner`.
pub(crate) struct Staged<'t> {
    inner: &'t dyn Tree,
    hidden: Option<&'t RelPath>,
    replaced: &'t [(RelPath, String)],
    removed: &'t [RelPath],
}

impl<'t> Staged<'t> {
    pub(crate) fn new(
        inner: &'t dyn Tree,
        hidden: &'t RelPath,
        replaced: &'t [(RelPath, String)],
    ) -> Self {
        Self {
            inner,
            hidden: Some(hidden),
            replaced,
            removed: &[],
        }
    }
    pub(crate) fn replacements(inner: &'t dyn Tree, replaced: &'t [(RelPath, String)]) -> Self {
        Self {
            inner,
            hidden: None,
            replaced,
            removed: &[],
        }
    }

    pub(crate) fn with_removed(mut self, removed: &'t [RelPath]) -> Self {
        self.removed = removed;
        self
    }

    fn absent(&self, path: &RelPath) -> bool {
        Some(path) == self.hidden || self.removed.binary_search(path).is_ok()
    }

    fn generated_directory(&self, path: &RelPath) -> bool {
        self.replaced.iter().any(|(child, _)| {
            child.authority() == path.authority()
                && child
                    .as_str()
                    .strip_prefix(path.as_str())
                    .is_some_and(|tail| tail.starts_with('/'))
        })
    }

    fn not_found(path: &RelPath) -> TreeError {
        TreeError::NotFound {
            path: path.as_str().to_string(),
        }
    }

    fn text(&self, path: &RelPath, max_bytes: u64) -> Option<Result<&'t str, TreeError>> {
        purrdf_lex::assoc::get(self.replaced, path).map(|text| {
            if u64::try_from(text.len()).is_ok_and(|len| len <= max_bytes) {
                Ok(text.as_str())
            } else {
                Err(TreeError::TooLarge {
                    path: path.as_str().to_string(),
                    limit: max_bytes,
                })
            }
        })
    }
}

impl Tree for Staged<'_> {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        if self.absent(path) {
            return Err(Self::not_found(path));
        }
        match self.text(path, max_bytes) {
            Some(text) => text.map(str::to_string),
            None => self.inner.read(path, max_bytes),
        }
    }

    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        if self.absent(path) {
            return Err(Self::not_found(path));
        }
        if purrdf_lex::assoc::get(self.replaced, path).is_some() {
            let id = FileId::from_key(format!("staged:{path}")).in_source(path.authority());
            if seen(&id) {
                return Ok(Opened::Seen(id));
            }
            let text = match self.text(path, max_bytes) {
                Some(text) => text?.to_string(),
                None => return Err(Self::not_found(path)),
            };
            return Ok(Opened::Read { id, text });
        }
        self.inner.read_unseen(path, max_bytes, seen)
    }

    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        let mut entries = match self.inner.list(path) {
            Ok(entries) => entries,
            Err(TreeError::NotFound { .. }) if self.generated_directory(path) => Vec::new(),
            Err(error) => return Err(error),
        };
        let mut kept = Vec::new();
        for entry in entries {
            let child = path.join(&entry.name).map_err(|_| TreeError::OutsideRoot {
                path: format!("{path}/{}", entry.name),
            })?;
            if !self.absent(&child) {
                kept.push(entry);
            }
        }
        entries = kept;
        for (replaced, _) in self.replaced {
            if replaced.authority() == path.authority()
                && let Some(tail) = replaced
                    .as_str()
                    .strip_prefix(path.as_str())
                    .and_then(|tail| tail.strip_prefix('/'))
                && let Some((name, _)) = tail.split_once('/')
                && !entries.iter().any(|entry| entry.name == name)
            {
                entries.push(Entry::new(name, EntryKind::Dir));
            }
            if replaced.parent().as_ref() == Some(path)
                && let Some(name) = replaced.file_name()
                && !entries.iter().any(|entry| entry.name == name)
            {
                entries.push(Entry::new(name, EntryKind::File));
            }
        }
        purrdf_lex::walk::try_sort_unstable_by(&mut entries, |a, b| Ok::<_, TreeError>(a.cmp(b)))?;
        Ok(entries)
    }

    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        if self.absent(path) {
            return Ok(None);
        }
        if purrdf_lex::assoc::get(self.replaced, path).is_some() {
            return Ok(Some(EntryKind::File));
        }
        if self.generated_directory(path) && self.inner.entry(path)?.is_none() {
            return Ok(Some(EntryKind::Dir));
        }
        self.inner.entry(path)
    }
}

#[cfg(test)]
mod tests {
    use raf_source::{EntryKind, MemoryTree, RelPath, Tree, TreeError};

    use super::Staged;

    fn path(text: &str) -> RelPath {
        RelPath::new(text).expect("a path")
    }

    fn tree() -> MemoryTree {
        MemoryTree::from([
            ("work/changes/a.md", "a"),
            ("work/changes/b.md", "b"),
            ("docs/a.md", "docs"),
        ])
    }

    #[test]
    fn the_hidden_file_is_not_read_listed_or_found() {
        let inner = tree();
        let hidden = path("work/changes/a.md");
        let staged = Staged::new(&inner, &hidden, &[]);

        assert!(matches!(
            staged.read(&hidden, 100),
            Err(TreeError::NotFound { .. })
        ));
        assert!(matches!(
            staged.read_unseen(&hidden, 100, &|_| false),
            Err(TreeError::NotFound { .. })
        ));
        assert!(matches!(
            staged.identity(&hidden),
            Err(TreeError::NotFound { .. })
        ));
        assert_eq!(staged.entry(&hidden).expect("an answer"), None);
        let names: Vec<String> = staged
            .list(&path("work/changes"))
            .expect("a listing")
            .into_iter()
            .map(|entry| entry.name)
            .collect();
        assert_eq!(names, ["b.md"]);
    }

    #[test]
    fn every_other_path_is_answered_as_the_inner_tree_does() {
        let inner = tree();
        let hidden = path("work/changes/a.md");
        let staged = Staged::new(&inner, &hidden, &[]);

        assert_eq!(
            staged.read(&path("work/changes/b.md"), 100).expect("read"),
            "b"
        );
        assert_eq!(staged.read(&path("docs/a.md"), 100).expect("read"), "docs");
        assert_eq!(
            staged.entry(&path("work/changes/b.md")).expect("an answer"),
            Some(EntryKind::File)
        );
        // A file of the same name in another directory is not hidden.
        let names: Vec<String> = staged
            .list(&path("docs"))
            .expect("a listing")
            .into_iter()
            .map(|entry| entry.name)
            .collect();
        assert_eq!(names, ["a.md"]);
    }

    #[test]
    fn a_replaced_file_reads_as_its_new_text_within_the_bound() {
        let inner = tree();
        let hidden = path("work/changes/a.md");
        let docs = path("docs/a.md");
        let edits = [(docs.clone(), "new text".to_string())];
        let staged = Staged::new(&inner, &hidden, &edits);

        assert_eq!(staged.read(&docs, 100).expect("read"), "new text");
        assert!(matches!(
            staged.read(&docs, 3),
            Err(TreeError::TooLarge { limit: 3, .. })
        ));
        // Its identity is its own and `seen` is honoured.
        let id = staged.identity(&docs).expect("an identity");
        assert_ne!(
            Some(&id),
            staged.identity(&path("work/changes/b.md")).ok().as_ref()
        );
        let opened = staged
            .read_unseen(&docs, 100, &|seen| *seen == id)
            .expect("seen");
        assert!(matches!(opened, raf_source::Opened::Seen(_)));
        // A file that is not replaced is the inner file.
        assert_eq!(
            staged.read(&path("work/changes/b.md"), 100).expect("read"),
            "b"
        );
    }
}
